//! Filename-only index. All traversal, matching, persistence and opening stay native.
use ignore::WalkBuilder;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, Manager, State};
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

const MAX_ENTRIES: usize = 250_000;
const MAX_CACHE_BYTES: u64 = 96 * 1024 * 1024;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FilePreferences {
    pub roots: Vec<String>,
    pub exclusions: Vec<String>,
    pub include_hidden: bool,
    pub terminal_directory: String,
}

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileIndexStatus {
    pub state: String,
    pub indexed: usize,
    pub revision: u64,
    pub warnings: Vec<String>,
    pub preferences: Option<FilePreferences>,
    pub shell: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Entry {
    key: String,
    path: PathBuf,
    folder: bool,
    #[serde(default)]
    native_identity: Option<String>,
    #[serde(skip)]
    name: String,
    #[serde(skip)]
    normalized: String,
    #[serde(skip)]
    parent: String,
    #[serde(skip)]
    normalized_parent: String,
}
impl Entry {
    fn prepare(&mut self) {
        self.name = self
            .path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if self.name.is_empty() {
            self.name = self.path.to_string_lossy().into_owned();
        }
        self.normalized = normalize(&self.name);
        self.parent = self
            .path
            .parent()
            .unwrap_or(Path::new(""))
            .to_string_lossy()
            .into_owned();
        self.normalized_parent = normalize(&self.parent);
    }
    fn item(&self, session: &str, relevance: u16) -> FileItem {
        FileItem {
            id: format!("{session}:{}", self.key),
            persistent_key: format!("file:v1:{}", self.key),
            name: self.name.clone(),
            parent_path: self.parent.clone(),
            kind: if self.folder { "folder" } else { "file" }.into(),
            relevance,
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileItem {
    pub id: String,
    pub persistent_key: String,
    pub name: String,
    pub parent_path: String,
    pub kind: String,
    pub relevance: u16,
}
#[derive(Serialize, Deserialize)]
struct Cache {
    version: u8,
    roots: Vec<String>,
    entries: Vec<Entry>,
}
#[derive(Clone, Default, Serialize, Deserialize)]
struct Identities {
    version: u8,
    items: HashMap<String, String>,
}
#[derive(Default)]
struct Index {
    entries: HashMap<String, Arc<Entry>>,
    status: FileIndexStatus,
}
enum Work {
    Rebuild,
    Events(Event),
    Failed,
}

pub struct FileIndex {
    inner: Arc<RwLock<Index>>,
    preferences: Arc<RwLock<FilePreferences>>,
    sender: Mutex<Option<mpsc::SyncSender<Work>>>,
    started: AtomicBool,
    generation: Arc<AtomicU64>,
    session: String,
    cache_path: PathBuf,
    config_path: PathBuf,
    identity_path: PathBuf,
    identities: Arc<RwLock<Identities>>,
    save_gate: Arc<Mutex<()>>,
}

fn normalize(value: &str) -> String {
    value
        .nfkd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(char::to_lowercase)
        .collect()
}
fn path_key(path: &Path) -> String {
    path.to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_lowercase()
}
fn within(path: &Path, parent: &Path) -> bool {
    let path = path_key(path);
    let parent = path_key(parent).trim_end_matches(['\\', '/']).to_owned();
    path == parent
        || path.starts_with(&(parent.clone() + "\\"))
        || path.starts_with(&(parent + "/"))
}
fn allowed(path: &Path, roots: &[String]) -> bool {
    let candidate = path_key(path);
    roots.iter().any(|root| {
        let root = path_key(Path::new(root))
            .trim_end_matches(['\\', '/'])
            .to_string();
        candidate == root
            || candidate.starts_with(&(root.clone() + "\\"))
            || candidate.starts_with(&(root + "/"))
    })
}
fn valid_key(key: &str) -> bool {
    key.len() == 32
        && key
            .bytes()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}
fn ignored_path(path: &Path, preferences: &FilePreferences) -> bool {
    if !path.is_absolute()
        || path.components().any(|part| {
            matches!(
                part,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        })
    {
        return true;
    }
    let path = path_key(path);
    for root in &preferences.roots {
        let root = path_key(Path::new(root))
            .trim_end_matches(['\\', '/'])
            .to_owned();
        if path == root {
            return false;
        }
        if let Some(relative) = path.strip_prefix(&(root + "\\")) {
            let components: Vec<_> = relative.split(['\\', '/']).collect();
            return components.iter().enumerate().any(|(position, part)| {
                (!preferences.include_hidden && part.starts_with('.'))
                    || (preferences
                        .exclusions
                        .iter()
                        .any(|name| name.eq_ignore_ascii_case(part))
                        && (position + 1 < components.len() || Path::new(&path).is_dir()))
            });
        }
    }
    false
}
fn write_json(path: &Path, data: &impl Serialize) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let temporary = path.with_extension("json.tmp");
    let file = fs::File::create(&temporary).map_err(|e| e.to_string())?;
    let mut writer = std::io::BufWriter::new(file);
    serde_json::to_writer(&mut writer, data).map_err(|e| e.to_string())?;
    std::io::Write::flush(&mut writer).map_err(|e| e.to_string())?;
    writer.get_ref().sync_all().map_err(|e| e.to_string())?;
    drop(writer);
    fs::rename(temporary, path).map_err(|e| e.to_string())
}
fn read_json<T: serde::de::DeserializeOwned>(path: &Path, maximum: u64) -> Option<T> {
    let file = fs::File::open(path).ok()?;
    if file.metadata().ok()?.len() > maximum {
        return None;
    }
    serde_json::from_reader(std::io::BufReader::new(file)).ok()
}

impl FileIndex {
    pub fn new(app: &AppHandle) -> Self {
        let directory = app
            .path()
            .app_local_data_dir()
            .unwrap_or_else(|_| std::env::temp_dir().join("mycast"));
        let config_path = directory.join("file-preferences.json");
        let identity_path = directory.join("file-identities.v1.json");
        let identities = read_json::<Identities>(&identity_path, 4 * 1024 * 1024)
            .filter(|value| {
                value.version == 1
                    && value.items.len() <= 10000
                    && value
                        .items
                        .iter()
                        .all(|(key, value)| valid_key(key) && value.len() <= 256)
            })
            .unwrap_or(Identities {
                version: 1,
                ..Default::default()
            });
        let defaults = FilePreferences {
            roots: platform::common_folders(),
            exclusions: vec![".git".into(), "node_modules".into()],
            include_hidden: false,
            terminal_directory: app
                .path()
                .home_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
        };
        let preferences = read_json::<FilePreferences>(&config_path, 65536)
            .filter(|p| validate_preferences(p, false).is_ok())
            .unwrap_or(defaults);
        Self {
            inner: Arc::new(RwLock::new(Index {
                status: FileIndexStatus {
                    state: "idle".into(),
                    ..Default::default()
                },
                ..Default::default()
            })),
            preferences: Arc::new(RwLock::new(preferences)),
            sender: Mutex::new(None),
            started: AtomicBool::new(false),
            generation: Arc::new(AtomicU64::new(0)),
            session: uuid::Uuid::new_v4().simple().to_string()[..16].to_string(),
            cache_path: directory.join("file-index.v1.json"),
            config_path,
            identity_path,
            identities: Arc::new(RwLock::new(identities)),
            save_gate: Arc::new(Mutex::new(())),
        }
    }
    fn start(&self, app: &AppHandle) {
        if self.started.swap(true, Ordering::SeqCst) {
            return;
        }
        let (sender, receiver) = mpsc::sync_channel(512);
        *self.sender.lock().unwrap() = Some(sender.clone());
        let inner = self.inner.clone();
        let preferences = self.preferences.clone();
        let cache_path = self.cache_path.clone();
        let identities = self.identities.clone();
        let app = app.clone();
        std::thread::spawn(move || {
            index_worker(
                app,
                inner,
                preferences,
                cache_path,
                identities,
                sender,
                receiver,
            )
        });
    }
    fn rebuild(&self, app: &AppHandle) {
        if !self.started.load(Ordering::SeqCst) {
            self.start(app);
        } else if let Some(sender) = self.sender.lock().unwrap().as_ref() {
            let _ = sender.send(Work::Rebuild);
        }
    }
    fn resolve(&self, id: &str) -> Result<Entry, String> {
        let key = id
            .strip_prefix(&(self.session.clone() + ":"))
            .filter(|key| valid_key(key))
            .ok_or("Invalid file result. Search again.")?;
        self.inner
            .read()
            .unwrap()
            .entries
            .get(key)
            .map(|entry| entry.as_ref().clone())
            .ok_or("This file is no longer indexed. Search again or rebuild the index.".into())
    }
}

fn validate_preferences(preferences: &FilePreferences, verify: bool) -> Result<(), String> {
    if preferences.roots.is_empty() || preferences.roots.len() > 32 {
        return Err("Choose between one and 32 search folders.".into());
    }
    for root in &preferences.roots {
        if root.len() > 32768 || root.contains('\0') || !Path::new(root).is_absolute() {
            return Err(format!("Search folder is unavailable: {root}"));
        }
    }
    if preferences.exclusions.len() > 64
        || preferences
            .exclusions
            .iter()
            .any(|value| value.is_empty() || value.len() > 128 || value.contains(['\0', '/', '\\']))
    {
        return Err(
            "Exclusions must be folder names (up to 64, each at most 128 characters).".into(),
        );
    }
    if preferences.terminal_directory.contains('\0')
        || !Path::new(&preferences.terminal_directory).is_absolute()
        || (verify && !Path::new(&preferences.terminal_directory).is_dir())
    {
        return Err("Choose an available terminal working folder.".into());
    }
    Ok(())
}
#[tauri::command]
pub fn get_file_status(index: State<'_, FileIndex>) -> FileIndexStatus {
    let mut status = index.inner.read().unwrap().status.clone();
    status.preferences = Some(index.preferences.read().unwrap().clone());
    status.shell = platform::shell_path()
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    status
}
#[tauri::command]
pub fn rebuild_file_index(app: AppHandle, index: State<'_, FileIndex>) {
    index.rebuild(&app);
}
#[tauri::command]
pub async fn set_file_preferences(
    app: AppHandle,
    index: State<'_, FileIndex>,
    mut preferences: FilePreferences,
) -> Result<FileIndexStatus, String> {
    validate_preferences(&preferences, true)?;
    let mut seen = HashSet::new();
    preferences
        .roots
        .retain(|root| seen.insert(path_key(Path::new(root))));
    let reindex;
    {
        let _save = index.save_gate.lock().unwrap();
        let old = index.preferences.read().unwrap().clone();
        reindex = old.roots != preferences.roots
            || old.exclusions != preferences.exclusions
            || old.include_hidden != preferences.include_hidden;
        write_json(&index.config_path, &preferences)?;
        if reindex {
            index.inner.write().unwrap().entries.retain(|_, entry| {
                allowed(&entry.path, &preferences.roots) && !ignored_path(&entry.path, &preferences)
            });
        }
        *index.preferences.write().unwrap() = preferences;
    }
    if reindex {
        index.rebuild(&app);
    }
    Ok(get_file_status(index))
}

fn announce(app: &AppHandle, inner: &RwLock<Index>, state: &str, warnings: &[String]) {
    let status = {
        let mut index = inner.write().unwrap();
        index.status.state = state.into();
        index.status.indexed = index.entries.len();
        index.status.revision += 1;
        index.status.warnings = warnings.iter().take(12).cloned().collect();
        index.status.clone()
    };
    let _ = app.emit("file-index-changed", status);
}
fn save_cache(path: &Path, inner: &RwLock<Index>, roots: &[String]) -> Result<(), String> {
    #[derive(Serialize)]
    struct CacheView<'a> {
        version: u8,
        roots: &'a [String],
        entries: Vec<&'a Entry>,
    }
    let index = inner.read().unwrap();
    let cache = CacheView {
        version: 1,
        roots,
        entries: index.entries.values().map(Arc::as_ref).collect(),
    };
    write_json(path, &cache)
}

fn scan(
    paths: &[PathBuf],
    preferences: &FilePreferences,
    previous: &HashMap<String, Arc<Entry>>,
    cache_path: &Path,
    identities: &RwLock<Identities>,
) -> (HashMap<String, Arc<Entry>>, Vec<String>) {
    let mut paths = paths.to_vec();
    paths.sort();
    paths.dedup();
    let distinct = paths.clone();
    paths.retain(|path| {
        !distinct
            .iter()
            .any(|parent| parent != path && within(path, parent))
    });
    if paths.is_empty() {
        return (HashMap::new(), vec![]);
    }
    let by_path: HashMap<_, _> = previous
        .values()
        .map(|entry| (path_key(&entry.path), entry.as_ref()))
        .collect();
    let by_identity: HashMap<_, _> = previous
        .values()
        .filter_map(|entry| {
            entry
                .native_identity
                .as_ref()
                .map(|id| (id.clone(), entry.as_ref()))
        })
        .collect();
    let durable_by_identity: HashMap<_, _> = identities
        .read()
        .unwrap()
        .items
        .iter()
        .map(|(key, identity)| (identity.clone(), key.clone()))
        .collect();
    let output = Mutex::new(HashMap::new());
    let warnings = Mutex::new(Vec::new());
    let mut builder = WalkBuilder::new(&paths[0]);
    for path in paths.iter().skip(1) {
        builder.add(path);
    }
    builder
        .hidden(false)
        .ignore(false)
        .parents(false)
        .git_ignore(false)
        .git_global(false)
        .git_exclude(false)
        .follow_links(false)
        .threads(2);
    let filter_preferences = preferences.clone();
    let include_hidden = preferences.include_hidden;
    let cache_directory = cache_path.parent().unwrap_or(cache_path).to_path_buf();
    let roots = preferences.roots.clone();
    builder.filter_entry(move |entry| {
        let path = entry.path();
        if path.starts_with(&cache_directory) || !allowed(path, &roots) {
            return false;
        }
        let is_root = roots
            .iter()
            .any(|root| path_key(Path::new(root)) == path_key(path));
        if is_root {
            return true;
        }
        if ignored_path(path, &filter_preferences) {
            return false;
        }
        !platform::skip_entry(path, include_hidden)
    });
    builder.build_parallel().run(|| {
        Box::new(|result| {
            let item = match result {
                Ok(item) => item,
                Err(error) => {
                    let mut errors = warnings.lock().unwrap();
                    if errors.len() < 10 {
                        errors.push(error.to_string());
                    }
                    return ignore::WalkState::Continue;
                }
            };
            if item.file_type().is_none_or(|kind| kind.is_symlink()) {
                return ignore::WalkState::Continue;
            }
            let folder = item.file_type().is_some_and(|kind| kind.is_dir());
            let path = item.into_path();
            let prior = by_path.get(&path_key(&path)).copied();
            let identity = if prior.is_some_and(|entry| entry.native_identity.is_some())
                || (prior.is_none() && !durable_by_identity.is_empty())
            {
                platform::file_identity(&path)
            } else {
                None
            };
            let prior = prior
                .filter(|entry| {
                    entry.native_identity.is_none() || identity == entry.native_identity
                })
                .or_else(|| {
                    identity
                        .as_ref()
                        .and_then(|id| by_identity.get(id).copied())
                });
            let mut entry = Entry {
                key: prior
                    .map(|entry| entry.key.clone())
                    .or_else(|| {
                        identity
                            .as_ref()
                            .and_then(|identity| durable_by_identity.get(identity).cloned())
                    })
                    .unwrap_or_else(|| uuid::Uuid::new_v4().simple().to_string()),
                folder,
                path,
                native_identity: identity
                    .or_else(|| prior.and_then(|entry| entry.native_identity.clone())),
                name: String::new(),
                normalized: String::new(),
                parent: String::new(),
                normalized_parent: String::new(),
            };
            entry.prepare();
            let mut entries = output.lock().unwrap();
            if entries.len() >= MAX_ENTRIES {
                return ignore::WalkState::Quit;
            }
            entries.insert(entry.key.clone(), Arc::new(entry));
            ignore::WalkState::Continue
        })
    });
    let output = output.into_inner().unwrap();
    let mut warnings = warnings.into_inner().unwrap();
    if output.len() >= MAX_ENTRIES {
        warnings.push(format!("Index limit of {MAX_ENTRIES} entries reached. Narrow the selected folders or add exclusions."));
    }
    (output, warnings)
}

fn index_worker(
    app: AppHandle,
    inner: Arc<RwLock<Index>>,
    preferences: Arc<RwLock<FilePreferences>>,
    cache_path: PathBuf,
    identities: Arc<RwLock<Identities>>,
    sender: mpsc::SyncSender<Work>,
    receiver: mpsc::Receiver<Work>,
) {
    let overflow = Arc::new(AtomicBool::new(false));
    let mut watcher: Option<RecommendedWatcher> = None;
    let mut rebuild = true;
    let mut watch_warnings = Vec::new();
    // Cached entries are usable immediately; reconciliation follows before ready.
    let config = preferences.read().unwrap().clone();
    if let Some(cache) = read_json::<Cache>(&cache_path, MAX_CACHE_BYTES).filter(|cache| {
        cache.version == 1 && cache.roots == config.roots && cache.entries.len() <= MAX_ENTRIES
    }) {
        let mut entries = HashMap::new();
        for mut entry in cache.entries {
            if valid_key(&entry.key)
                && allowed(&entry.path, &config.roots)
                && !ignored_path(&entry.path, &config)
            {
                entry.prepare();
                entries.insert(entry.key.clone(), Arc::new(entry));
            }
        }
        hydrate_identities(&mut entries, &identities);
        inner.write().unwrap().entries = entries;
        announce(&app, &inner, "reconciling", &[]);
    }
    loop {
        if rebuild {
            let config = preferences.read().unwrap().clone();
            let mut warnings = Vec::new();
            let callback_sender = sender.clone();
            let callback_overflow = overflow.clone();
            let callback_config = config.clone();
            let callback_cache = cache_path.parent().unwrap_or(&cache_path).to_path_buf();
            let replacement = notify::recommended_watcher(move |result: notify::Result<Event>| {
                let work = match result {
                    Ok(event)
                        if matches!(
                            event.kind,
                            EventKind::Access(_)
                                | EventKind::Modify(notify::event::ModifyKind::Data(_))
                        ) =>
                    {
                        return;
                    }
                    Ok(event)
                        if !event.paths.is_empty()
                            && event.paths.iter().all(|path| {
                                !allowed(path, &callback_config.roots)
                                    || ignored_path(path, &callback_config)
                                    || within(path, &callback_cache)
                            }) =>
                    {
                        return;
                    }
                    Ok(event) => Work::Events(event),
                    Err(_) => Work::Failed,
                };
                if callback_sender.try_send(work).is_err() {
                    callback_overflow.store(true, Ordering::SeqCst);
                }
            });
            watcher = replacement.ok();
            if let Some(watcher) = watcher.as_mut() {
                let mut parents = HashSet::new();
                for root in &config.roots {
                    if let Err(error) = watcher.watch(Path::new(root), RecursiveMode::Recursive) {
                        warnings.push(format!(
                            "Cannot watch {root}: {error}. Use Rebuild to refresh."
                        ));
                    }
                    if let Some(parent) = Path::new(root)
                        .parent()
                        .filter(|parent| !allowed(parent, &config.roots))
                    {
                        if parents.insert(parent.to_path_buf()) {
                            let _ = watcher.watch(parent, RecursiveMode::NonRecursive);
                        }
                    }
                }
            } else {
                warnings
                    .push("Live folder updates are unavailable. Use Rebuild to refresh.".into());
            }
            watch_warnings = warnings.clone();
            announce(&app, &inner, "indexing", &warnings);
            let mut previous = inner.read().unwrap().entries.clone();
            hydrate_identities(&mut previous, &identities);
            let paths: Vec<_> = config.roots.iter().map(PathBuf::from).collect();
            let (mut entries, errors) = scan(&paths, &config, &previous, &cache_path, &identities);
            hydrate_identities(&mut entries, &identities);
            if config != *preferences.read().unwrap() {
                continue;
            }
            warnings.extend(errors);
            inner.write().unwrap().entries = entries;
            if let Err(error) = save_cache(&cache_path, &inner, &config.roots) {
                warnings.push(format!("Could not save file index: {error}"));
            }
            announce(&app, &inner, "ready", &warnings);
            rebuild = false;
        }
        let Ok(work) = receiver.recv() else {
            break;
        };
        let mut batch = vec![work];
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(1) {
            match receiver.recv_timeout(Duration::from_millis(160)) {
                Ok(work) => batch.push(work),
                Err(_) => break,
            }
        }
        if overflow.swap(false, Ordering::SeqCst)
            || batch.iter().any(|work| {
                matches!(work, Work::Rebuild | Work::Failed)
                    || matches!(work, Work::Events(event) if event.need_rescan())
                    || matches!(work, Work::Events(event) if matches!(event.kind, EventKind::Create(_) | EventKind::Remove(_)) && event.paths.iter().any(|path| preferences.read().unwrap().roots.iter().any(|root| path_key(path) == path_key(Path::new(root)))))
            })
        {
            rebuild = true;
            continue;
        }
        let config = preferences.read().unwrap().clone();
        let mut previous = inner.read().unwrap().entries.clone();
        hydrate_identities(&mut previous, &identities);
        let mut affected = Vec::<PathBuf>::new();
        for work in batch {
            if let Work::Events(event) = work {
                if matches!(
                    event.kind,
                    EventKind::Modify(notify::event::ModifyKind::Name(
                        notify::event::RenameMode::Both
                    ))
                ) && event.paths.len() == 2
                {
                    let old = &event.paths[0];
                    let new = &event.paths[1];
                    for entry in previous.values_mut() {
                        if let Ok(relative) = entry.path.strip_prefix(old) {
                            let path = new.join(relative);
                            let entry = Arc::make_mut(entry);
                            entry.path = path;
                            entry.prepare();
                        }
                    }
                }
                for path in event.paths {
                    if allowed(&path, &config.roots)
                        && !path.starts_with(cache_path.parent().unwrap_or(&cache_path))
                    {
                        affected.push(path);
                    }
                }
            }
        }
        if affected.is_empty() {
            continue;
        }
        affected.sort();
        affected.dedup();
        let paths: Vec<_> = affected
            .iter()
            .filter(|path| path.exists())
            .cloned()
            .collect();
        let (updated, warnings) = scan(&paths, &config, &previous, &cache_path, &identities);
        if config != *preferences.read().unwrap() {
            rebuild = true;
            continue;
        }
        previous.retain(|_, entry| !affected.iter().any(|path| within(&entry.path, path)));
        previous.extend(updated);
        if previous.len() > MAX_ENTRIES {
            rebuild = true;
            continue;
        }
        inner.write().unwrap().entries = previous;
        let mut warnings = [watch_warnings.clone(), warnings].concat();
        if let Err(error) = save_cache(&cache_path, &inner, &config.roots) {
            warnings.push(format!("Could not save file index: {error}"));
        }
        announce(&app, &inner, "ready", &warnings);
    }
    drop(watcher);
}
fn hydrate_identities(entries: &mut HashMap<String, Arc<Entry>>, identities: &RwLock<Identities>) {
    for (key, identity) in &identities.read().unwrap().items {
        if let Some(entry) = entries.get_mut(key) {
            if entry.native_identity.as_ref() != Some(identity) {
                Arc::make_mut(entry).native_identity = Some(identity.clone());
            }
        }
    }
}

#[derive(Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchUsage {
    count: f64,
    last_used: f64,
}
#[derive(Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSearchOptions {
    pub query: String,
    pub request_id: u64,
    #[serde(default)]
    pub favorites: Vec<String>,
    #[serde(default)]
    pub recents: Vec<String>,
    #[serde(default)]
    pub aliases: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub usage: HashMap<String, SearchUsage>,
    #[serde(default)]
    pub collection: String,
    #[serde(default)]
    pub browse: bool,
    #[serde(default)]
    pub now: f64,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSearchResults {
    pub items: Vec<FileItem>,
    pub request_id: u64,
    pub elapsed_ms: f64,
}

fn relevance(
    entry: &Entry,
    query: &str,
    exact_alias: &str,
    aliases: &[String],
    matcher: &mut Matcher,
    needle: Utf32Str<'_>,
    buffer: &mut Vec<char>,
) -> (u16, u16) {
    if aliases
        .iter()
        .any(|alias| alias.nfc().collect::<String>().to_lowercase() == exact_alias)
    {
        return (1100, 0);
    }
    if entry.normalized == query {
        return (1000, 0);
    }
    if entry.normalized.starts_with(query) {
        return (900, 0);
    }
    if aliases.iter().any(|alias| normalize(alias) == query) {
        return (850, 0);
    }
    if entry
        .normalized
        .split(|c: char| c.is_whitespace() || "_-.".contains(c))
        .any(|part| part.starts_with(query))
    {
        return (800, 0);
    }
    if entry.normalized.contains(query) {
        return (700, 0);
    }
    if aliases.iter().any(|alias| normalize(alias).contains(query)) {
        return (600, 0);
    }
    if entry.normalized_parent.contains(query) {
        return (500, 0);
    }
    if query.split_whitespace().count() > 1
        && query
            .split_whitespace()
            .all(|part| entry.normalized.contains(part) || entry.normalized_parent.contains(part))
    {
        return (400, 0);
    }
    if query.chars().count() >= 2 {
        let mut score = matcher
            .fuzzy_match(Utf32Str::new(&entry.normalized, buffer), needle)
            .unwrap_or(0);
        for alias in aliases {
            score = score.max(
                matcher
                    .fuzzy_match(Utf32Str::new(&normalize(alias), buffer), needle)
                    .unwrap_or(0),
            );
        }
        if score > 0 {
            return (100, score);
        }
    }
    (0, 0)
}
fn usage_score(usage: Option<&SearchUsage>, now: f64) -> f64 {
    usage
        .filter(|usage| usage.count.is_finite() && usage.count > 0.0 && usage.last_used.is_finite())
        .map(|usage| {
            (1.0 + usage.count).log2()
                + 4.0 / (1.0 + ((now - usage.last_used) / 86400000.0).max(0.0))
        })
        .unwrap_or(0.0)
}
fn rank_files(
    snapshot: &Index,
    options: &FileSearchOptions,
    generation: &AtomicU64,
    session: &str,
) -> FileSearchResults {
    let started = Instant::now();
    let query = normalize(options.query.trim());
    let exact_alias = options
        .query
        .trim()
        .nfc()
        .collect::<String>()
        .to_lowercase();
    let mut needle_buffer = Vec::new();
    let needle = Utf32Str::new(&query, &mut needle_buffer);
    let mut buffer = Vec::new();
    let mut matcher = Matcher::new(Config::DEFAULT);
    let favorites: HashSet<_> = options
        .favorites
        .iter()
        .filter_map(|key| key.strip_prefix("file:v1:"))
        .collect();
    let recents: HashSet<_> = options
        .recents
        .iter()
        .filter_map(|key| key.strip_prefix("file:v1:"))
        .collect();
    let aliases: HashMap<_, _> = options
        .aliases
        .iter()
        .filter_map(|(key, value)| {
            key.strip_prefix("file:v1:")
                .map(|key| (key, value.as_slice()))
        })
        .collect();
    let usage: HashMap<_, _> = options
        .usage
        .iter()
        .filter_map(|(key, value)| key.strip_prefix("file:v1:").map(|key| (key, value)))
        .collect();
    let favorite_order: HashMap<_, _> = options
        .favorites
        .iter()
        .enumerate()
        .filter_map(|(position, key)| {
            key.strip_prefix("file:v1:")
                .map(|key| (key, 20000.0 - position as f64))
        })
        .collect();
    let recent_order: HashMap<_, _> = options
        .recents
        .iter()
        .enumerate()
        .filter_map(|(position, key)| {
            key.strip_prefix("file:v1:")
                .map(|key| (key, 10000.0 - position as f64))
        })
        .collect();
    let mut best: Vec<(&Entry, u16, bool, f64, u16)> = Vec::with_capacity(41);
    let now = if options.now.is_finite() && options.now > 0.0 {
        options.now
    } else {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
            * 1000.0
    };
    let compare = |a: &(&Entry, u16, bool, f64, u16), b: &(&Entry, u16, bool, f64, u16)| {
        b.1.cmp(&a.1)
            .then(b.2.cmp(&a.2))
            .then(b.3.total_cmp(&a.3))
            .then(b.4.cmp(&a.4))
            .then_with(|| a.0.normalized.cmp(&b.0.normalized))
            .then_with(|| a.0.path.cmp(&b.0.path))
    };
    for (position, entry) in snapshot.entries.values().enumerate() {
        let entry = entry.as_ref();
        if position % 256 == 0 && generation.load(Ordering::SeqCst) != options.request_id {
            return FileSearchResults {
                items: vec![],
                request_id: options.request_id,
                elapsed_ms: started.elapsed().as_secs_f64() * 1000.0,
            };
        }
        let key = entry.key.as_str();
        let pinned = favorites.contains(key);
        let recent = recents.contains(key);
        if options.collection == "favorites" && !pinned
            || options.collection == "recents" && !recent
            || query.is_empty() && !options.browse && !pinned && !recent
        {
            continue;
        }
        let (score, fuzzy) = if query.is_empty() {
            (1, 0)
        } else {
            relevance(
                entry,
                &query,
                &exact_alias,
                aliases.get(key).copied().unwrap_or(&[]),
                &mut matcher,
                needle,
                &mut buffer,
            )
        };
        if score == 0 {
            continue;
        }
        let rank = if query.is_empty() {
            if options.collection == "recents" {
                recent_order.get(key).copied().unwrap_or(0.0)
            } else if pinned {
                favorite_order.get(key).copied().unwrap_or(0.0)
            } else {
                recent_order.get(key).copied().unwrap_or(0.0)
            }
        } else {
            usage_score(usage.get(key).copied(), now)
        };
        let candidate = (
            entry,
            score,
            pinned && !(query.is_empty() && options.collection == "recents"),
            rank,
            fuzzy,
        );
        if best.len() == 40 && compare(&candidate, best.last().unwrap()).is_ge() {
            continue;
        }
        let position = best
            .binary_search_by(|value| compare(value, &candidate))
            .unwrap_or_else(|position| position);
        if position < 40 {
            best.insert(position, candidate);
            best.truncate(40);
        }
    }
    FileSearchResults {
        items: best
            .into_iter()
            .map(|entry| entry.0.item(&session, entry.1))
            .collect(),
        request_id: options.request_id,
        elapsed_ms: started.elapsed().as_secs_f64() * 1000.0,
    }
}

#[tauri::command]
pub async fn search_files(
    app: AppHandle,
    index: State<'_, FileIndex>,
    options: FileSearchOptions,
) -> Result<FileSearchResults, String> {
    if options.query.len() > 1024
        || options.favorites.len() > 10000
        || options.recents.len() > 50
        || options.aliases.len() > 10000
        || options.usage.len() > 10000
        || options
            .aliases
            .values()
            .any(|aliases| aliases.len() > 10 || aliases.iter().any(|alias| alias.len() > 128))
    {
        return Err("File search request is too large.".into());
    }
    index.start(&app);
    index
        .generation
        .fetch_max(options.request_id, Ordering::SeqCst);
    let inner = index.inner.clone();
    let generation = index.generation.clone();
    let session = index.session.clone();
    tauri::async_runtime::spawn_blocking(move || {
        rank_files(&inner.read().unwrap(), &options, &generation, &session)
    })
    .await
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn resolve_files(
    app: AppHandle,
    index: State<'_, FileIndex>,
    keys: Vec<String>,
) -> Result<Vec<FileItem>, String> {
    if keys.len() > 10000 {
        return Err("Too many saved files.".into());
    }
    index.start(&app);
    let snapshot = index.inner.read().unwrap();
    Ok(keys
        .iter()
        .filter_map(|key| key.strip_prefix("file:v1:"))
        .filter_map(|key| snapshot.entries.get(key))
        .map(|entry| entry.item(&index.session, 0))
        .collect())
}
#[tauri::command]
pub async fn remember_file(index: State<'_, FileIndex>, id: String) -> Result<(), String> {
    let entry = index.resolve(&id)?;
    let inner = index.inner.clone();
    let identities = index.identities.clone();
    let identity_path = index.identity_path.clone();
    let gate = index.save_gate.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Some(identity) = platform::file_identity(&entry.path) else {
            return Ok(());
        };
        {
            let _save = gate.lock().unwrap();
            let mut saved = identities.write().unwrap();
            if saved.items.get(&entry.key) != Some(&identity) {
                if saved.items.len() >= 10000 && !saved.items.contains_key(&entry.key) {
                    return Ok(());
                }
                let mut updated = saved.clone();
                updated.items.insert(entry.key.clone(), identity.clone());
                write_json(&identity_path, &updated)?;
                *saved = updated;
            }
        }
        if let Some(current) = inner.write().unwrap().entries.get_mut(&entry.key) {
            Arc::make_mut(current).native_identity = Some(identity);
        }
        Ok(())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn file_action(
    app: AppHandle,
    index: State<'_, FileIndex>,
    id: String,
    action: String,
) -> Result<String, String> {
    let entry = index.resolve(&id)?;
    if !entry.path.exists() {
        return Err(
            "This file is no longer available. Rebuild the index to refresh results.".into(),
        );
    }
    if !allowed(&entry.path, &index.preferences.read().unwrap().roots) {
        return Err("This file is outside your selected folders.".into());
    }
    if platform::skip_entry(&entry.path, true)
        || entry
            .native_identity
            .as_ref()
            .is_some_and(|identity| platform::file_identity(&entry.path).as_ref() != Some(identity))
    {
        return Err(
            "This file changed since it was indexed. Rebuild the index and search again.".into(),
        );
    }
    match action.as_str() {
        "copy-path" => {
            use tauri_plugin_clipboard_manager::ClipboardExt;
            app.clipboard()
                .write_text(entry.path.to_string_lossy().as_ref())
                .map_err(|e| e.to_string())?;
            Ok("Copied path".into())
        }
        "open" | "reveal" => {
            let reveal = action == "reveal";
            tauri::async_runtime::spawn_blocking(move || platform::open(&entry.path, reveal))
                .await
                .map_err(|e| e.to_string())??;
            Ok(if reveal {
                "Revealed in Explorer"
            } else {
                "Opened file"
            }
            .into())
        }
        _ => Err("Unknown file action.".into()),
    }
}
#[tauri::command]
pub async fn choose_file_folder(app: AppHandle) -> Result<Option<String>, String> {
    let window = app
        .get_webview_window("main")
        .ok_or("Launcher is unavailable")?;
    tauri::async_runtime::spawn_blocking(move || platform::pick_folder(&window))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn run_shell_command(
    index: State<'_, FileIndex>,
    command: String,
) -> Result<String, String> {
    if command.encode_utf16().count() > 8192 || command.contains('\0') {
        return Err("Command is too long or contains a null character.".into());
    }
    let directory = index.preferences.read().unwrap().terminal_directory.clone();
    if !Path::new(&directory).is_dir() {
        return Err("Terminal working folder is unavailable. Choose another folder in Files & Run settings.".into());
    }
    tauri::async_runtime::spawn_blocking(move || platform::run(&command, &directory))
        .await
        .map_err(|e| e.to_string())??;
    Ok("Opened PowerShell".into())
}

#[cfg(windows)]
mod platform {
    use super::*;
    use std::os::windows::io::AsRawHandle;
    use std::{
        ffi::OsString,
        os::windows::{
            ffi::OsStringExt,
            fs::{MetadataExt, OpenOptionsExt},
            process::CommandExt,
        },
        process::Command,
    };
    use windows::{
        Win32::{
            Foundation::HANDLE,
            Storage::FileSystem::{FILE_ID_INFO, FileIdInfo, GetFileInformationByHandleEx},
            System::Com::{
                CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
                CoTaskMemFree, CoUninitialize,
            },
            UI::{
                Shell::{
                    FOLDERID_Desktop, FOLDERID_Documents, FOLDERID_Downloads, FOLDERID_Music,
                    FOLDERID_Pictures, FOLDERID_Videos, FOS_FORCEFILESYSTEM, FOS_PICKFOLDERS,
                    FileOpenDialog, IFileOpenDialog, KF_FLAG_DEFAULT, SEE_MASK_FLAG_NO_UI,
                    SEE_MASK_NOASYNC, SHELLEXECUTEINFOW, SHGetKnownFolderPath, SIGDN_FILESYSPATH,
                    ShellExecuteExW,
                },
                WindowsAndMessaging::SW_SHOWNORMAL,
            },
        },
        core::{GUID, PCWSTR},
    };
    struct Apartment;
    impl Apartment {
        fn new() -> Result<Self, String> {
            unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }
                .ok()
                .map_err(|e| e.to_string())?;
            Ok(Self)
        }
    }
    impl Drop for Apartment {
        fn drop(&mut self) {
            unsafe { CoUninitialize() };
        }
    }
    fn wide(value: &std::ffi::OsStr) -> Vec<u16> {
        use std::os::windows::ffi::OsStrExt;
        value.encode_wide().chain(Some(0)).collect()
    }
    fn known(id: &GUID) -> Option<String> {
        let value = unsafe { SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None) }.ok()?;
        let path = unsafe { OsString::from_wide(value.as_wide()) };
        unsafe { CoTaskMemFree(Some(value.0.cast())) };
        Some(PathBuf::from(path).to_string_lossy().into_owned())
    }
    pub fn common_folders() -> Vec<String> {
        [
            FOLDERID_Desktop,
            FOLDERID_Documents,
            FOLDERID_Downloads,
            FOLDERID_Pictures,
            FOLDERID_Music,
            FOLDERID_Videos,
        ]
        .iter()
        .filter_map(known)
        .collect()
    }
    pub fn skip_entry(path: &Path, hidden: bool) -> bool {
        let Ok(metadata) = fs::symlink_metadata(path) else {
            return true;
        };
        let attributes = metadata.file_attributes();
        if !hidden
            && (attributes & 6 != 0
                || path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with('.')))
        {
            return true;
        }
        // Cloud placeholders are reparse points too; skip name-surrogate links, not cloud folders.
        if attributes & 0x400 != 0 {
            // std file_type distinguishes symbolic links and junctions without opening content.
            if metadata.file_type().is_symlink() {
                return true;
            }
        }
        false
    }
    pub fn file_identity(path: &Path) -> Option<String> {
        let file = fs::OpenOptions::new()
            .access_mode(0)
            .share_mode(7)
            .custom_flags(0x02000000 | 0x00200000)
            .open(path)
            .ok()?;
        let mut info = FILE_ID_INFO::default();
        unsafe {
            GetFileInformationByHandleEx(
                HANDLE(file.as_raw_handle()),
                FileIdInfo,
                (&mut info as *mut FILE_ID_INFO).cast(),
                std::mem::size_of::<FILE_ID_INFO>() as u32,
            )
        }
        .ok()?;
        Some(format!(
            "{}:{:x?}:{}",
            info.VolumeSerialNumber,
            info.FileId.Identifier,
            file.metadata().ok()?.creation_time()
        ))
    }
    pub fn shell_path() -> PathBuf {
        let modern = PathBuf::from(
            std::env::var_os("ProgramFiles").unwrap_or_else(|| "C:\\Program Files".into()),
        )
        .join("PowerShell/7/pwsh.exe");
        if modern.is_file() {
            modern
        } else {
            PathBuf::from(std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into()))
                .join("System32/WindowsPowerShell/v1.0/powershell.exe")
        }
    }
    pub fn open(path: &Path, reveal: bool) -> Result<(), String> {
        if reveal {
            let explorer = PathBuf::from(
                std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into()),
            )
            .join("explorer.exe");
            Command::new(explorer)
                .arg(format!("/select,\"{}\"", path.display()))
                .spawn()
                .map_err(|e| e.to_string())?;
            return Ok(());
        }
        let _apartment = Apartment::new()?;
        let path = wide(path.as_os_str());
        let info = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: SEE_MASK_NOASYNC | SEE_MASK_FLAG_NO_UI,
            lpFile: PCWSTR(path.as_ptr()),
            nShow: SW_SHOWNORMAL.0,
            ..Default::default()
        };
        let mut info = info;
        unsafe { ShellExecuteExW(&mut info) }.map_err(|e| e.to_string())
    }
    pub fn pick_folder(window: &tauri::WebviewWindow) -> Result<Option<String>, String> {
        let _apartment = Apartment::new()?;
        let dialog: IFileOpenDialog =
            unsafe { CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER) }
                .map_err(|e| e.to_string())?;
        unsafe { dialog.SetOptions(FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM) }
            .map_err(|e| e.to_string())?;
        if let Err(error) = unsafe { dialog.Show(Some(window.hwnd().map_err(|e| e.to_string())?)) }
        {
            if error.code().0 as u32 == 0x800704c7 {
                return Ok(None);
            }
            return Err(error.to_string());
        }
        let item = unsafe { dialog.GetResult() }.map_err(|e| e.to_string())?;
        let value = unsafe { item.GetDisplayName(SIGDN_FILESYSPATH) }.map_err(|e| e.to_string())?;
        let path = unsafe { OsString::from_wide(value.as_wide()) };
        unsafe { CoTaskMemFree(Some(value.0.cast())) };
        Ok(Some(PathBuf::from(path).to_string_lossy().into_owned()))
    }
    pub fn run(command: &str, directory: &str) -> Result<(), String> {
        use base64::{Engine, engine::general_purpose::STANDARD};
        let script = format!(
            "Set-Location -LiteralPath '{}'\n{}",
            directory.replace('\'', "''"),
            command
        );
        let bytes: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
        let encoded = STANDARD.encode(bytes);
        let terminal = PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap_or_default())
            .join("Microsoft/WindowsApps/wt.exe");
        let shell = shell_path();
        let mut process = if terminal.is_file() {
            let mut process = Command::new(terminal);
            process.args(["-w", "new", "new-tab"]).arg(shell);
            process
        } else {
            let mut process = Command::new(shell);
            process.creation_flags(0x00000010);
            process
        };
        process
            .args(["-NoLogo", "-NoExit", "-EncodedCommand", &encoded])
            .current_dir(directory)
            .spawn()
            .map_err(|e| format!("Could not open PowerShell: {e}"))?;
        Ok(())
    }
}
#[cfg(not(windows))]
mod platform {
    use super::*;
    pub fn common_folders() -> Vec<String> {
        vec![]
    }
    pub fn skip_entry(path: &Path, hidden: bool) -> bool {
        !hidden
            && path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with('.'))
    }
    pub fn file_identity(_: &Path) -> Option<String> {
        None
    }
    pub fn shell_path() -> PathBuf {
        PathBuf::from("PowerShell")
    }
    pub fn open(_: &Path, _: bool) -> Result<(), String> {
        Err("File opening is supported on Windows only.".into())
    }
    pub fn pick_folder(_: &tauri::WebviewWindow) -> Result<Option<String>, String> {
        Err("Folder selection is supported on Windows only.".into())
    }
    pub fn run(_: &str, _: &str) -> Result<(), String> {
        Err("PowerShell launching is supported on Windows only.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn entry(number: usize, name: &str) -> Arc<Entry> {
        let mut entry = Entry {
            key: format!("{number:032x}"),
            path: PathBuf::from(format!("C:\\Fixture\\group{}\\{name}", number % 100)),
            folder: false,
            native_identity: None,
            name: String::new(),
            normalized: String::new(),
            parent: String::new(),
            normalized_parent: String::new(),
        };
        entry.prepare();
        Arc::new(entry)
    }
    fn options(query: &str) -> FileSearchOptions {
        FileSearchOptions {
            query: query.into(),
            request_id: 1,
            browse: true,
            now: 1_800_000_000_000.0,
            ..Default::default()
        }
    }
    #[test]
    fn file_search_ranking_and_aliases_before_limit() {
        let mut index = Index::default();
        for number in 0..100 {
            let entry = entry(number, &format!("Report {number:03}.pdf"));
            index.entries.insert(entry.key.clone(), entry);
        }
        let target = entry(1000, "Résumé final.txt");
        let key = format!("file:v1:{}", target.key);
        index.entries.insert(target.key.clone(), target);
        let mut search = options("REPORT");
        search.aliases.insert(key.clone(), vec!["report".into()]);
        let result = rank_files(&index, &search, &AtomicU64::new(1), "session");
        assert_eq!(result.items.len(), 40);
        assert_eq!(result.items[0].persistent_key, key);
        assert_eq!(result.items[0].relevance, 1100);
        let result = rank_files(&index, &options("rsm"), &AtomicU64::new(1), "session");
        assert_eq!(result.items[0].name, "Résumé final.txt");
        assert_eq!(
            rank_files(&index, &options("report"), &AtomicU64::new(2), "session")
                .items
                .len(),
            0
        );
    }
    #[test]
    fn file_search_boundaries_and_cache_roundtrip() {
        assert!(allowed(
            Path::new(r"C:\Users\Me\Documents\file.txt"),
            &[r"C:\Users\Me\Documents".into()]
        ));
        assert!(!allowed(
            Path::new(r"C:\Users\Me\Documents-other\file.txt"),
            &[r"C:\Users\Me\Documents".into()]
        ));
        let record = entry(2, "文档 and spaces.txt");
        let decoded: Entry =
            serde_json::from_str(&serde_json::to_string(record.as_ref()).unwrap()).unwrap();
        assert_eq!(decoded.key, record.key);
        assert_eq!(decoded.path, record.path);
        assert!(decoded.normalized.is_empty());
    }
    #[test]
    #[ignore = "Explicit opt-in 100k-entry warm search profile"]
    fn file_search_warm_profile() {
        let mut index = Index::default();
        for number in 0..100_000 {
            let entry = entry(number, &format!("Annual Report {number:06}.pdf"));
            index.entries.insert(entry.key.clone(), entry);
        }
        let mut timings = Vec::new();
        for _ in 0..3 {
            for query in [
                "report",
                "rpt",
                "report 999",
                "zzzzzz",
                "pdf",
                "Annual Report 001234.pdf",
            ] {
                let result = rank_files(&index, &options(query), &AtomicU64::new(1), "session");
                assert!(result.items.len() <= 40);
                timings.push(result.elapsed_ms);
            }
        }
        timings.sort_by(f64::total_cmp);
        let p95 = timings[(timings.len() as f64 * 0.95).ceil() as usize - 1];
        println!(
            "100000 entries; {} queries; median {:.2} ms; p95 {:.2} ms; target <=100ms: {}",
            timings.len(),
            timings[timings.len() / 2],
            p95,
            p95 <= 100.0
        );
    }
}
