use std::{
    collections::BTreeMap,
    sync::{
        Mutex,
        atomic::{AtomicU16, Ordering},
    },
};
use tauri::{AppHandle, Manager, State};
pub const IDS: [&str; 9] = [
    "applications",
    "files",
    "run",
    "calculator",
    "currency",
    "utilities",
    "web-search",
    "ai",
    "windows",
];
static ENABLED: AtomicU16 = AtomicU16::new(u16::MAX >> (16 - IDS.len()));
pub struct Modules {
    pub values: Mutex<BTreeMap<String, bool>>,
}
pub fn enabled(id: &str) -> bool {
    IDS.iter()
        .position(|v| *v == id)
        .is_some_and(|n| ENABLED.load(Ordering::SeqCst) & (1 << n) != 0)
}
pub fn require(id: &str) -> Result<(), String> {
    if enabled(id) {
        Ok(())
    } else {
        Err(format!("The {id} module is disabled."))
    }
}
impl Modules {
    pub fn load(app: &AppHandle) -> Self {
        let saved = app
            .path()
            .app_config_dir()
            .ok()
            .and_then(|p| std::fs::read(p.join("modules.json")).ok())
            .and_then(|s| serde_json::from_slice::<BTreeMap<String, bool>>(&s).ok())
            .unwrap_or_default();
        let values: BTreeMap<_, _> = IDS
            .iter()
            .map(|id| (id.to_string(), saved.get(*id).copied().unwrap_or(true)))
            .collect();
        ENABLED.store(
            IDS.iter().enumerate().fold(0, |bits, (n, id)| {
                bits | if values[*id] { 1 << n } else { 0 }
            }),
            Ordering::SeqCst,
        );
        Self {
            values: Mutex::new(values),
        }
    }
}
#[tauri::command]
pub fn get_modules(state: State<'_, Modules>) -> BTreeMap<String, bool> {
    state.values.lock().unwrap().clone()
}
#[tauri::command]
pub async fn set_module(
    app: AppHandle,
    id: String,
    enabled: bool,
) -> Result<BTreeMap<String, bool>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let n = IDS.iter().position(|v| *v == id).ok_or("Unknown module")?;
        let state = app.state::<Modules>();
        let mut values = state.values.lock().unwrap();
        let mut next = values.clone();
        next.insert(id.clone(), enabled);
        let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        std::fs::write(
            dir.join("modules.json.tmp"),
            serde_json::to_vec(&next).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        std::fs::rename(dir.join("modules.json.tmp"), dir.join("modules.json"))
            .map_err(|e| e.to_string())?;
        if enabled {
            ENABLED.fetch_or(1 << n, Ordering::SeqCst);
        } else {
            ENABLED.fetch_and(!(1 << n), Ordering::SeqCst);
        }
        if id == "files" && !enabled {
            app.state::<crate::files::FileIndex>().stop();
        }
        if id == "ai" && !enabled {
            app.state::<crate::ai::AiState>().0.lock().unwrap().clear();
            app.state::<crate::mcp::McpState>().stop();
        }
        if id == "applications" && !enabled {
            tauri::async_runtime::block_on(
                app.state::<crate::applications::ApplicationCatalog>()
                    .clear(),
            );
        }
        *values = next;
        Ok(values.clone())
    })
    .await
    .map_err(|e| e.to_string())?
}
