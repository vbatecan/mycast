//! A small, demand-loaded catalogue of Start Menu shortcuts and packaged apps.
//! Discovery never resolves shortcuts, scans other folders, or watches files.
//! Only opaque IDs cross the IPC boundary; launch paths stay in native memory.

use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

#[cfg(windows)]
#[path = "application_icons.rs"]
mod icon_extraction;

use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{
    State,
    async_runtime::{Mutex, RwLock},
};
use uuid::Uuid;

const MAX_APPLICATIONS: usize = 4096;
const MAX_CACHED_ICONS: usize = 128;
const ICON_WORKERS: usize = 3;

#[derive(Default)]
struct IconCache {
    values: HashMap<String, Option<String>>,
    order: VecDeque<String>,
}

impl IconCache {
    fn get(&mut self, id: &str) -> Option<Option<String>> {
        let value = self.values.get(id)?.clone();
        self.order.retain(|cached| cached != id);
        self.order.push_back(id.to_string());
        Some(value)
    }

    fn insert(&mut self, id: String, value: Option<String>) {
        self.order.retain(|cached| cached != &id);
        if !self.values.contains_key(&id) && self.values.len() >= MAX_CACHED_ICONS {
            if let Some(oldest) = self.order.pop_front() {
                self.values.remove(&oldest);
            }
        }
        self.order.push_back(id.clone());
        self.values.insert(id, value);
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApplicationSource {
    StartMenu,
    WindowsApp,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Application {
    pub id: String,
    pub persistent_key: String,
    pub name: String,
    pub description: String,
    pub source: ApplicationSource,
    pub keywords: Vec<String>,
    pub can_run_as_admin: bool,
}

#[derive(Clone, Serialize)]
pub struct ApplicationList {
    pub applications: Vec<Application>,
    pub warnings: Vec<String>,
}

#[derive(Serialize)]
pub struct ApplicationLaunchResult {
    pub cancelled: bool,
    pub message: String,
}

#[derive(Clone)]
enum LaunchTarget {
    Shortcut(PathBuf),
    WindowsApp(String),
}

struct Candidate {
    name: String,
    description: String,
    source: ApplicationSource,
    keywords: Vec<String>,
    identity: String,
    target: LaunchTarget,
}

struct CatalogSnapshot {
    list: ApplicationList,
    launches: HashMap<String, (String, LaunchTarget)>,
    ids_by_identity: HashMap<String, String>,
    icons: Mutex<IconCache>,
}

struct Discovery {
    candidates: Vec<Candidate>,
    warnings: Vec<String>,
    sources_available: usize,
}

/// Only one discovery runs at once. Readers keep using an immutable previous
/// snapshot during refresh; COM objects never leave their discovery thread.
#[derive(Default)]
pub struct ApplicationCatalog {
    snapshot: RwLock<Option<Arc<CatalogSnapshot>>>,
    discovery: Mutex<()>,
    icon_workers: [Mutex<()>; ICON_WORKERS],
    next_icon_worker: AtomicUsize,
}

#[tauri::command]
pub async fn list_applications(
    catalog: State<'_, ApplicationCatalog>,
    refresh: Option<bool>,
) -> Result<ApplicationList, String> {
    if !refresh.unwrap_or(false) {
        if let Some(cached) = catalog.snapshot.read().await.as_ref() {
            return Ok(cached.list.clone());
        }
    }
    let _discovery = catalog.discovery.lock().await;
    let previous = catalog.snapshot.read().await.clone();
    // A concurrent first read may have populated the cache while this request
    // waited for the discovery gate.
    if !refresh.unwrap_or(false) {
        if let Some(cached) = previous.as_ref() {
            return Ok(cached.list.clone());
        }
    }

    // All filesystem and Shell work runs off the Tauri/WebView event thread.
    let discovered = tauri::async_runtime::spawn_blocking(platform::discover)
        .await
        .map_err(|error| format!("Application discovery could not finish: {error}"))?;

    if discovered.sources_available == 0 {
        if let Some(cached) = previous.as_ref() {
            let mut list = cached.list.clone();
            list.warnings = discovered.warnings;
            list.warnings.push(
                "Showing the previous catalogue because Windows application sources are unavailable."
                    .into(),
            );
            return Ok(list);
        }
    }

    let previous_ids = previous.as_ref().map(|cached| &cached.ids_by_identity);
    let mut applications = Vec::with_capacity(discovered.candidates.len());
    let mut launches = HashMap::new();
    let mut ids_by_identity = HashMap::new();
    for candidate in discovered.candidates {
        let persistent_key = format!("app:v1:{:x}", Sha256::digest(candidate.identity.as_bytes()));
        // Keep IDs stable across refreshes for entries with the same native
        // identity, while retaining no executable/path supplied by the UI.
        let id = previous_ids
            .and_then(|ids| ids.get(&candidate.identity))
            .cloned()
            .unwrap_or_else(|| format!("app_{}", Uuid::new_v4()));
        let can_run_as_admin = platform::can_elevate(&candidate.target);
        launches.insert(id.clone(), (candidate.name.clone(), candidate.target));
        ids_by_identity.insert(candidate.identity, id.clone());
        applications.push(Application {
            id,
            persistent_key,
            name: candidate.name,
            description: candidate.description,
            source: candidate.source,
            keywords: candidate.keywords,
            can_run_as_admin,
        });
    }
    applications.sort_by_cached_key(|application| {
        (
            application.name.to_lowercase(),
            application.description.to_lowercase(),
        )
    });
    let list = ApplicationList {
        applications,
        warnings: discovered.warnings,
    };
    *catalog.snapshot.write().await = Some(Arc::new(CatalogSnapshot {
        list: list.clone(),
        launches,
        ids_by_identity,
        icons: Mutex::new(IconCache::default()),
    }));
    Ok(list)
}

#[tauri::command]
pub async fn launch_application(
    app: tauri::AppHandle,
    catalog: State<'_, ApplicationCatalog>,
    id: String,
    mode: Option<crate::elevation::LaunchMode>,
) -> Result<ApplicationLaunchResult, String> {
    if id.len() > 64 {
        return Err("Invalid application ID.".into());
    }
    let (name, target) = {
        let snapshot = catalog.snapshot.read().await;
        snapshot
            .as_ref()
            .and_then(|cached| cached.launches.get(&id))
            .cloned()
            .ok_or(
                "This application is no longer in the catalogue. Refresh applications and try again.",
            )?
    };
    let elevated = matches!(mode, Some(crate::elevation::LaunchMode::Administrator));
    if elevated {
        let result = tauri::async_runtime::spawn_blocking(move || {
            let _guard = crate::elevation::Pending::new()?;
            platform::elevate(&target)
        })
        .await
        .map_err(|e| e.to_string())?;
        if result.is_err() || matches!(result, Ok(false)) {
            crate::elevation::restore(&app);
        }
        return result.map(|completed| ApplicationLaunchResult {
            message: if completed {
                format!("Opening {name} as administrator")
            } else {
                "Administrator launch cancelled".into()
            },
            cancelled: !completed,
        });
    }
    tauri::async_runtime::spawn_blocking(move || platform::launch(&target))
        .await
        .map_err(|error| format!("Application launch could not finish: {error}"))?
        .map_err(|error| format!("Could not open {name}: {error}"))?;
    Ok(ApplicationLaunchResult {
        message: format!("Opening {name}"),
        cancelled: false,
    })
}

#[tauri::command]
pub async fn get_application_icon(
    catalog: State<'_, ApplicationCatalog>,
    id: String,
) -> Result<Option<String>, String> {
    if id.len() > 64 {
        return Err("Invalid application ID.".into());
    }
    let Some(snapshot) = catalog.snapshot.read().await.clone() else {
        return Ok(None);
    };
    let Some((_, target)) = snapshot.launches.get(&id).cloned() else {
        return Ok(None);
    };
    if let Some(cached) = snapshot.icons.lock().await.get(&id) {
        return Ok(cached);
    }
    // Three independent gates bound Shell extraction without an additional
    // runtime dependency. Cache hits never wait for a worker.
    let worker = catalog.next_icon_worker.fetch_add(1, Ordering::Relaxed) % ICON_WORKERS;
    let _permit = catalog.icon_workers[worker].lock().await;
    if let Some(cached) = snapshot.icons.lock().await.get(&id) {
        return Ok(cached);
    }
    let icon = tauri::async_runtime::spawn_blocking(move || {
        #[cfg(windows)]
        {
            icon_extraction::extract(&target)
        }
        #[cfg(not(windows))]
        {
            let _ = target;
            None
        }
    })
    .await
    .map_err(|error| format!("Application icon could not be read: {error}"))?;
    // Each catalogue snapshot has its own bounded cache, so refresh naturally
    // invalidates old images and failed extractions without racing readers.
    snapshot.icons.lock().await.insert(id, icon.clone());
    Ok(icon)
}

#[cfg(windows)]
mod platform {
    use super::{ApplicationSource, Candidate, Discovery, LaunchTarget, MAX_APPLICATIONS};
    use std::{
        collections::{HashSet, VecDeque},
        ffi::OsString,
        fs,
        mem::size_of,
        os::windows::{ffi::OsStringExt, fs::MetadataExt},
        path::{Path, PathBuf},
        ptr,
    };
    use windows::{
        Win32::{
            Foundation::PROPERTYKEY,
            Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT,
            System::Com::{
                CLSCTX_INPROC_SERVER, CLSCTX_LOCAL_SERVER, COINIT_APARTMENTTHREADED,
                COINIT_DISABLE_OLE1DDE, CoCreateInstance, CoInitializeEx, CoTaskMemFree,
                CoUninitialize, IPersistFile, STGM_READ,
            },
            UI::{
                Shell::{
                    AO_NOERRORUI, ApplicationActivationManager, BHID_EnumItems,
                    FOLDERID_AppsFolder, FOLDERID_CommonStartMenu, FOLDERID_StartMenu,
                    IApplicationActivationManager, IEnumShellItems, IShellItem, IShellItem2,
                    IShellLinkW, KF_FLAG_DEFAULT, SEE_MASK_FLAG_NO_UI, SEE_MASK_NO_CONSOLE,
                    SEE_MASK_NOASYNC, SHCreateItemFromParsingName, SHCreateItemInKnownFolder,
                    SHELLEXECUTEINFOW, SHGetKnownFolderPath, SIGDN_NORMALDISPLAY, ShellExecuteExW,
                    ShellLink,
                },
                WindowsAndMessaging::SW_SHOWNORMAL,
            },
        },
        core::{GUID, Interface, PCWSTR, PWSTR, w},
    };

    const MAX_DEPTH: usize = 8;
    const MAX_FILESYSTEM_ENTRIES: usize = 16_000;
    const MAX_SHELL_ITEMS: usize = 8192;
    const MAX_SHORTCUT_BYTES: u64 = 1024 * 1024;
    const TEXT_BUFFER: usize = 32_768;

    // System.AppUserModel.ID, defined by Windows Propkey.h.
    // https://learn.microsoft.com/windows/win32/properties/props-system-appusermodel-id
    const APP_USER_MODEL_ID: PROPERTYKEY = PROPERTYKEY {
        fmtid: GUID::from_u128(0x9f4c2855_9f79_4b39_a8d0_e1d42de1d5f3),
        pid: 5,
    };

    pub(super) struct ComApartment;

    impl ComApartment {
        pub(super) fn initialize() -> Result<Self, String> {
            // Both discovery and launching are synchronous work on a blocking
            // worker; the apartment is always released on that same thread.
            unsafe {
                CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE)
                    .ok()
                    .map_err(|error| format!("Windows Shell initialization failed: {error}"))?;
            }
            Ok(Self)
        }
    }

    impl Drop for ComApartment {
        fn drop(&mut self) {
            unsafe { CoUninitialize() };
        }
    }

    struct ShellString(PWSTR);

    impl Drop for ShellString {
        fn drop(&mut self) {
            unsafe { CoTaskMemFree(Some(self.0.0 as *const std::ffi::c_void)) };
        }
    }

    fn shell_text(value: PWSTR) -> windows::core::Result<String> {
        let owned = ShellString(value);
        unsafe { Ok(owned.0.to_string()?) }
    }

    fn known_folder(id: &GUID) -> windows::core::Result<PathBuf> {
        let owned = ShellString(unsafe { SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None)? });
        Ok(PathBuf::from(OsString::from_wide(unsafe {
            owned.0.as_wide()
        })))
    }

    fn text(buffer: &[u16]) -> String {
        let end = buffer
            .iter()
            .position(|value| *value == 0)
            .unwrap_or(buffer.len());
        String::from_utf16_lossy(&buffer[..end])
    }

    fn compact(value: &str, maximum: usize) -> String {
        value
            .chars()
            .filter(|character| !character.is_control())
            .take(maximum)
            .collect::<String>()
            .trim()
            .to_string()
    }

    pub(super) fn wide_path(path: &Path) -> Result<Vec<u16>, String> {
        use std::os::windows::ffi::OsStrExt;
        let mut encoded: Vec<u16> = path.as_os_str().encode_wide().collect();
        if encoded.contains(&0) {
            return Err("The shortcut path is invalid.".into());
        }
        encoded.push(0);
        Ok(encoded)
    }

    fn packaged_aumid(item: &IShellItem2) -> Option<String> {
        let raw = unsafe { item.GetString(&APP_USER_MODEL_ID) }.ok()?;
        let value = shell_text(raw).ok()?;
        let mut parts = value.split('!');
        let package = parts.next()?;
        let application = parts.next()?;
        if parts.next().is_some()
            || package.is_empty()
            || application.is_empty()
            || value.len() > 256
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"._-!".contains(&byte))
        {
            return None;
        }
        Some(value)
    }

    struct Collector {
        discovery: Discovery,
        identities: HashSet<String>,
        visited_entries: usize,
        unreadable: usize,
        broken_shortcuts: usize,
        depth_limited: bool,
    }

    impl Collector {
        fn new() -> Self {
            Self {
                discovery: Discovery {
                    candidates: Vec::new(),
                    warnings: Vec::new(),
                    sources_available: 0,
                },
                identities: HashSet::new(),
                visited_entries: 0,
                unreadable: 0,
                broken_shortcuts: 0,
                depth_limited: false,
            }
        }

        fn full(&self) -> bool {
            self.discovery.candidates.len() >= MAX_APPLICATIONS
        }

        fn add(&mut self, candidate: Candidate) {
            if !self.full() && self.identities.insert(candidate.identity.clone()) {
                self.discovery.candidates.push(candidate);
            }
        }

        fn scan_start_menu(&mut self, root: &Path, scope: &str) {
            let mut directories = VecDeque::from([(root.to_path_buf(), 0usize)]);
            while let Some((directory, depth)) = directories.pop_front() {
                if self.full() || self.visited_entries >= MAX_FILESYSTEM_ENTRIES {
                    return;
                }
                let reader = match fs::read_dir(&directory) {
                    Ok(reader) => reader,
                    Err(_) => {
                        self.unreadable += 1;
                        continue;
                    }
                };
                if depth == 0 {
                    self.discovery.sources_available += 1;
                }
                let remaining = MAX_FILESYSTEM_ENTRIES - self.visited_entries;
                let mut entries: Vec<_> = reader.take(remaining).collect();
                self.visited_entries += entries.len();
                entries.sort_by_cached_key(|entry| {
                    entry
                        .as_ref()
                        .map(|entry| entry.file_name().to_string_lossy().to_lowercase())
                        .unwrap_or_default()
                });
                for entry in entries {
                    if self.full() {
                        return;
                    }
                    let entry = match entry {
                        Ok(entry) => entry,
                        Err(_) => {
                            self.unreadable += 1;
                            continue;
                        }
                    };
                    let path = entry.path();
                    let metadata = match fs::symlink_metadata(&path) {
                        Ok(metadata) => metadata,
                        Err(_) => {
                            self.unreadable += 1;
                            continue;
                        }
                    };
                    // Never follow junctions, symlinks, or cloud placeholders.
                    if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT.0 != 0 {
                        continue;
                    }
                    if metadata.is_dir() {
                        if depth < MAX_DEPTH {
                            directories.push_back((path, depth + 1));
                        } else {
                            self.depth_limited = true;
                        }
                    } else if metadata.is_file()
                        && path.extension().is_some_and(|extension| {
                            extension.to_string_lossy().eq_ignore_ascii_case("lnk")
                        })
                    {
                        if metadata.len() > MAX_SHORTCUT_BYTES {
                            self.broken_shortcuts += 1;
                            continue;
                        }
                        match read_shortcut(&path, root, scope) {
                            Ok(candidate) => self.add(candidate),
                            Err(_) => self.broken_shortcuts += 1,
                        }
                    }
                }
            }
        }

        fn scan_windows_apps(&mut self) -> windows::core::Result<()> {
            let folder: IShellItem = unsafe {
                SHCreateItemInKnownFolder(&FOLDERID_AppsFolder, KF_FLAG_DEFAULT, PCWSTR::null())?
            };
            let enumerator: IEnumShellItems =
                unsafe { folder.BindToHandler(None, &BHID_EnumItems)? };
            self.discovery.sources_available += 1;
            for _ in 0..MAX_SHELL_ITEMS {
                if self.full() {
                    return Ok(());
                }
                let mut batch: [Option<IShellItem>; 1] = [None];
                let mut fetched = 0;
                unsafe { enumerator.Next(&mut batch, Some(&mut fetched))? };
                if fetched == 0 {
                    return Ok(());
                }
                let Some(item) = batch[0].take() else {
                    continue;
                };
                let Ok(properties) = item.cast::<IShellItem2>() else {
                    continue;
                };
                // AppsFolder contains desktop applications too. They come from
                // Start Menu shortcuts; only packaged AUMIDs are added here.
                let Some(aumid) = packaged_aumid(&properties) else {
                    continue;
                };
                let Ok(name) =
                    (unsafe { item.GetDisplayName(SIGDN_NORMALDISPLAY) }).and_then(shell_text)
                else {
                    continue;
                };
                let name = compact(&name, 256);
                if name.is_empty() {
                    continue;
                }
                let package = aumid.split('!').next().unwrap_or_default();
                let friendly_package = package.rsplit_once('_').map_or(package, |(name, _)| name);
                self.add(Candidate {
                    name,
                    description: format!("Windows app · {}", compact(friendly_package, 160)),
                    source: ApplicationSource::WindowsApp,
                    keywords: vec![
                        "application".into(),
                        "windows".into(),
                        "store".into(),
                        compact(friendly_package, 160),
                    ],
                    identity: format!("aumid:{}", aumid.to_lowercase()),
                    target: LaunchTarget::WindowsApp(aumid),
                });
            }
            self.discovery.warnings.push(
                "Windows application enumeration reached its safety limit; some entries may be omitted."
                    .into(),
            );
            Ok(())
        }

        fn finish(mut self) -> Discovery {
            if self.unreadable > 0 {
                self.discovery.warnings.push(format!(
                    "Skipped {} unreadable Start Menu entries or folders.",
                    self.unreadable,
                ));
            }
            if self.broken_shortcuts > 0 {
                self.discovery.warnings.push(format!(
                    "Skipped {} unreadable or invalid shortcuts.",
                    self.broken_shortcuts,
                ));
            }
            if self.depth_limited || self.visited_entries >= MAX_FILESYSTEM_ENTRIES || self.full() {
                self.discovery.warnings.push(
                    "Application discovery reached a safety limit; some entries may be omitted."
                        .into(),
                );
            }
            self.discovery
        }
    }

    fn read_shortcut(path: &Path, root: &Path, scope: &str) -> Result<Candidate, String> {
        let encoded = wide_path(path)?;
        let link: IShellLinkW = unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) }
            .map_err(|error| error.to_string())?;
        let persist: IPersistFile = link.cast().map_err(|error| error.to_string())?;
        unsafe { persist.Load(PCWSTR(encoded.as_ptr()), STGM_READ) }
            .map_err(|error| error.to_string())?;

        let name = compact(&path.file_stem().unwrap_or_default().to_string_lossy(), 256);
        if name.is_empty() {
            return Err("Shortcut has no name.".into());
        }
        let mut target_buffer = vec![0u16; TEXT_BUFFER];
        let mut argument_buffer = vec![0u16; TEXT_BUFFER];
        let mut directory_buffer = vec![0u16; TEXT_BUFFER];
        let mut description_buffer = [0u16; 1024];
        // Do not call Resolve: it can search drives or contact network targets.
        let _ = unsafe { link.GetPath(&mut target_buffer, ptr::null_mut(), 0) };
        let _ = unsafe { link.GetArguments(&mut argument_buffer) };
        let _ = unsafe { link.GetWorkingDirectory(&mut directory_buffer) };
        let _ = unsafe { link.GetDescription(&mut description_buffer) };
        let target = text(&target_buffer);
        let arguments = text(&argument_buffer);
        let working_directory = text(&directory_buffer);
        let item: Option<IShellItem2> =
            unsafe { SHCreateItemFromParsingName(PCWSTR(encoded.as_ptr()), None) }.ok();
        let aumid = item.as_ref().and_then(packaged_aumid);
        let identity = if let Some(aumid) = aumid {
            format!("aumid:{}", aumid.to_lowercase())
        } else if !target.is_empty() {
            format!(
                "target\0{}\0{}\0{}",
                target.to_lowercase(),
                arguments,
                working_directory.to_lowercase()
            )
        } else {
            format!("shortcut:{}", path.to_string_lossy().to_lowercase())
        };
        let relative_folder = path
            .parent()
            .and_then(|parent| parent.strip_prefix(root).ok())
            .filter(|parent| !parent.as_os_str().is_empty())
            .map(|parent| compact(&parent.to_string_lossy().replace('\\', " / "), 120));
        let detail = compact(&text(&description_buffer), 240);
        let location = relative_folder
            .map(|folder| format!("{folder} · {scope}"))
            .unwrap_or_else(|| scope.to_string());
        let description = if detail.is_empty() {
            format!("Start Menu · {location}")
        } else {
            format!("{detail} · {location}")
        };
        let executable = Path::new(&target)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        Ok(Candidate {
            name,
            description,
            source: ApplicationSource::StartMenu,
            keywords: vec![
                "application".into(),
                "start menu".into(),
                location,
                compact(&executable, 120),
            ],
            identity,
            target: LaunchTarget::Shortcut(path.to_path_buf()),
        })
    }

    pub(super) fn discover() -> Discovery {
        let mut collector = Collector::new();
        let _apartment = match ComApartment::initialize() {
            Ok(apartment) => apartment,
            Err(error) => {
                collector.discovery.warnings.push(error);
                return collector.finish();
            }
        };
        // Per-user entries win when both Start Menus contain the same launch
        // target. Different arguments/work directories remain separate entries.
        for (folder, scope) in [
            (FOLDERID_StartMenu, "Personal"),
            (FOLDERID_CommonStartMenu, "All users"),
        ] {
            match known_folder(&folder) {
                Ok(root) => collector.scan_start_menu(&root, scope),
                Err(error) => collector
                    .discovery
                    .warnings
                    .push(format!("The {scope} Start Menu is unavailable: {error}")),
            }
        }
        if let Err(error) = collector.scan_windows_apps() {
            collector
                .discovery
                .warnings
                .push(format!("Some Windows apps could not be read: {error}"));
        }
        collector.finish()
    }

    fn elevated_target(target: &LaunchTarget) -> Result<(String, String, String), String> {
        let LaunchTarget::Shortcut(path) = target else {
            return Err("Packaged applications do not support administrator launching.".into());
        };
        let _apartment = ComApartment::initialize()?;
        let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT.0 != 0 {
            return Err("Shortcut is unavailable or changed. Refresh applications.".into());
        }
        let encoded = wide_path(path)?;
        let item: Option<IShellItem2> =
            unsafe { SHCreateItemFromParsingName(PCWSTR(encoded.as_ptr()), None) }.ok();
        if item.as_ref().and_then(packaged_aumid).is_some() {
            return Err("Packaged applications do not support administrator launching.".into());
        }
        let link: IShellLinkW = unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) }
            .map_err(|e| e.to_string())?;
        let persist: IPersistFile = link.cast().map_err(|e| e.to_string())?;
        unsafe { persist.Load(PCWSTR(encoded.as_ptr()), STGM_READ) }.map_err(|e| e.to_string())?;
        let mut executable = vec![0u16; TEXT_BUFFER];
        let mut arguments = vec![0u16; TEXT_BUFFER];
        let mut directory = vec![0u16; TEXT_BUFFER];
        unsafe { link.GetPath(&mut executable, ptr::null_mut(), 0) }.map_err(|e| e.to_string())?;
        unsafe { link.GetArguments(&mut arguments) }.map_err(|e| e.to_string())?;
        unsafe { link.GetWorkingDirectory(&mut directory) }.map_err(|e| e.to_string())?;
        let executable = text(&executable);
        let path = Path::new(&executable);
        if !path.is_absolute()
            || !path.is_file()
            || !path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("exe"))
        {
            return Err("Administrator launching requires an available desktop executable.".into());
        }
        Ok((executable, text(&arguments), text(&directory)))
    }
    pub(super) fn can_elevate(target: &LaunchTarget) -> bool {
        elevated_target(target).is_ok()
    }
    pub(super) fn elevate(target: &LaunchTarget) -> Result<bool, String> {
        let (exe, args, directory) = elevated_target(target)?;
        crate::elevation::run(&exe, &args, &directory)
    }

    pub(super) fn launch(target: &LaunchTarget) -> Result<(), String> {
        let _apartment = ComApartment::initialize()?;
        match target {
            LaunchTarget::Shortcut(path) => {
                let metadata = fs::symlink_metadata(path).map_err(|_| {
                    "The shortcut is no longer available. Refresh applications.".to_string()
                })?;
                if !metadata.is_file()
                    || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT.0 != 0
                {
                    return Err(
                        "The cached shortcut is no longer a regular file. Refresh applications."
                            .into(),
                    );
                }
                let encoded = wide_path(path)?;
                let mut info = SHELLEXECUTEINFOW {
                    cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
                    fMask: SEE_MASK_NOASYNC | SEE_MASK_FLAG_NO_UI | SEE_MASK_NO_CONSOLE,
                    lpVerb: w!("open"),
                    lpFile: PCWSTR(encoded.as_ptr()),
                    nShow: SW_SHOWNORMAL.0,
                    ..Default::default()
                };
                // Let Windows open the original .lnk so its stored arguments,
                // working directory, app activation and elevation are preserved.
                unsafe { ShellExecuteExW(&mut info) }.map_err(|error| error.to_string())
            }
            LaunchTarget::WindowsApp(aumid) => {
                // An out-of-process broker keeps activation arguments alive if
                // this blocking worker releases its COM apartment immediately.
                let manager: IApplicationActivationManager = unsafe {
                    CoCreateInstance(&ApplicationActivationManager, None, CLSCTX_LOCAL_SERVER)
                }
                .map_err(|error| error.to_string())?;
                let encoded: Vec<u16> = aumid.encode_utf16().chain(std::iter::once(0)).collect();
                unsafe {
                    manager.ActivateApplication(
                        PCWSTR(encoded.as_ptr()),
                        PCWSTR::null(),
                        AO_NOERRORUI,
                    )
                }
                .map(|_| ())
                .map_err(|error| error.to_string())
            }
        }
    }
}

#[cfg(not(windows))]
mod platform {
    use super::{Discovery, LaunchTarget};

    pub(super) fn can_elevate(_: &LaunchTarget) -> bool {
        false
    }
    pub(super) fn elevate(_: &LaunchTarget) -> Result<bool, String> {
        Err("Administrator launching requires Windows.".into())
    }
    pub(super) fn discover() -> Discovery {
        Discovery {
            candidates: Vec::new(),
            warnings: vec!["Installed application discovery is available on Windows only.".into()],
            sources_available: 0,
        }
    }

    pub(super) fn launch(_target: &LaunchTarget) -> Result<(), String> {
        Err("Installed applications can be opened on Windows only.".into())
    }
}
