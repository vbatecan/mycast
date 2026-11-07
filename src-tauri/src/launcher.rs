//! Focused launcher controls. Plugin APIs and executable registration stay native.
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Modifiers, Shortcut};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherStatus {
    pub platform: String,
    pub shortcut: Option<String>,
    pub shortcut_error: Option<String>,
    pub version: String,
    pub startup_available: bool,
    pub launch_at_startup: bool,
    pub startup_error: Option<String>,
    pub visible: bool,
    pub shortcut_capture: bool,
}

#[derive(Default)]
struct ShortcutRegistration {
    current: Option<Shortcut>,
    error: Option<String>,
}

#[derive(Default)]
pub struct LauncherControls {
    registration: Mutex<ShortcutRegistration>,
    operation: tauri::async_runtime::Mutex<()>,
    capture: Arc<AtomicBool>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ShortcutConfig {
    version: u8,
    shortcut: String,
}

fn parse_shortcut(value: &str) -> Result<Shortcut, String> {
    if value.len() > 128 || value.chars().any(char::is_control) {
        return Err("Invalid launcher shortcut.".into());
    }
    let shortcut: Shortcut = value
        .parse()
        .map_err(|error| format!("Invalid shortcut: {error}"))?;
    if !shortcut
        .mods
        .intersects(Modifiers::ALT | Modifiers::CONTROL | Modifiers::SUPER)
    {
        return Err("Include Ctrl, Alt, or Windows in the launcher shortcut.".into());
    }
    Ok(shortcut)
}

fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|path| path.join("launcher.json"))
        .map_err(|error| format!("Could not locate shortcut settings: {error}"))
}

fn load_shortcut(app: &AppHandle) -> Result<Option<Shortcut>, String> {
    let path = config_path(app)?;
    let bytes = match fs::read(path) {
        Ok(bytes) if bytes.len() <= 4096 => bytes,
        Ok(_) => return Err("Saved launcher shortcut settings are too large.".into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("Could not read launcher shortcut: {error}")),
    };
    let config: ShortcutConfig = serde_json::from_slice(&bytes)
        .map_err(|_| "Saved launcher shortcut settings are invalid.".to_string())?;
    if config.version != 1 {
        return Err("Saved launcher shortcut settings have an unsupported version.".into());
    }
    parse_shortcut(&config.shortcut).map(Some)
}

fn save_shortcut(app: &AppHandle, shortcut: Shortcut) -> Result<(), String> {
    let path = config_path(app)?;
    let parent = path
        .parent()
        .ok_or("Shortcut settings directory is unavailable.")?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Could not create shortcut settings: {error}"))?;
    let data = serde_json::to_vec(&ShortcutConfig {
        version: 1,
        shortcut: shortcut.to_string(),
    })
    .map_err(|error| error.to_string())?;
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, data)
        .map_err(|error| format!("Could not save shortcut settings: {error}"))?;
    fs::rename(&temporary, &path)
        .map_err(|error| format!("Could not replace shortcut settings: {error}"))
}

fn display_shortcut(shortcut: Shortcut) -> String {
    shortcut
        .to_string()
        .split('+')
        .map(|part| match part {
            "control" => "Ctrl",
            "alt" => "Alt",
            "shift" => "Shift",
            "super" => "Windows",
            other => other
                .strip_prefix("Key")
                .or_else(|| other.strip_prefix("Digit"))
                .unwrap_or(other),
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

pub fn initialize(app: &AppHandle) {
    let controls = app.state::<LauncherControls>();
    let mut errors = Vec::new();
    let saved = match load_shortcut(app) {
        Ok(saved) => saved,
        Err(error) => {
            errors.push(error);
            None
        }
    };
    let mut candidates = Vec::new();
    if let Some(saved) = saved {
        candidates.push(saved);
    }
    for value in ["Alt+Space", "Ctrl+Alt+Space"] {
        let shortcut = parse_shortcut(value).expect("Built-in shortcut is valid");
        if !candidates.contains(&shortcut) {
            candidates.push(shortcut);
        }
    }
    let mut current = None;
    for shortcut in candidates {
        match app.global_shortcut().register(shortcut) {
            Ok(()) => {
                current = Some(shortcut);
                break;
            }
            Err(error) => errors.push(format!(
                "{} is unavailable: {error}",
                display_shortcut(shortcut)
            )),
        }
    }
    if current.is_none() {
        errors.push("Reopen Mycast from its tray icon.".into());
    }
    *controls.registration.lock().unwrap() = ShortcutRegistration {
        current,
        error: (!errors.is_empty()).then(|| errors.join(" ")),
    };
}

#[tauri::command]
pub fn get_launcher_status(
    app: AppHandle,
    controls: State<'_, LauncherControls>,
) -> LauncherStatus {
    let registration = controls.registration.lock().unwrap();
    let startup_available = cfg!(all(windows, not(debug_assertions)));
    let (launch_at_startup, startup_error) = if startup_available {
        match app.autolaunch().is_enabled() {
            Ok(enabled) => (enabled, None),
            Err(error) => (
                false,
                Some(format!(
                    "Could not read Windows startup registration: {error}"
                )),
            ),
        }
    } else {
        (
            false,
            Some(if cfg!(debug_assertions) {
                "Startup is available in release builds. Development builds depend on the Vite server.".into()
            } else {
                "Launch at startup is available on Windows.".into()
            }),
        )
    };
    LauncherStatus {
        platform: std::env::consts::OS.into(),
        shortcut: registration.current.map(display_shortcut),
        shortcut_error: registration.error.clone(),
        version: app.package_info().version.to_string(),
        startup_available,
        launch_at_startup,
        startup_error,
        visible: app
            .get_webview_window("main")
            .and_then(|window| window.is_visible().ok())
            .unwrap_or(false),
        shortcut_capture: controls.capture.load(Ordering::SeqCst),
    }
}

#[tauri::command]
pub async fn set_launcher_shortcut(
    app: AppHandle,
    controls: State<'_, LauncherControls>,
    shortcut: Option<String>,
) -> Result<LauncherStatus, String> {
    let _operation = controls.operation.lock().await;
    let replacement = parse_shortcut(shortcut.as_deref().unwrap_or("Alt+Space"))?;
    let previous = controls.registration.lock().unwrap().current;
    if previous != Some(replacement) {
        // Acquire the replacement first. A conflict never releases the working shortcut.
        app.global_shortcut()
            .register(replacement)
            .map_err(|error| {
                format!(
                    "This shortcut is unavailable: {error}. The previous shortcut is still active."
                )
            })?;
        if let Err(error) = save_shortcut(&app, replacement) {
            let _ = app.global_shortcut().unregister(replacement);
            return Err(error);
        }
        if let Some(previous) = previous {
            if let Err(error) = app.global_shortcut().unregister(previous) {
                let _ = app.global_shortcut().unregister(replacement);
                let _ = save_shortcut(&app, previous);
                return Err(format!(
                    "Could not release the previous shortcut: {error}. It remains active."
                ));
            }
        }
    } else {
        save_shortcut(&app, replacement)?;
    }
    *controls.registration.lock().unwrap() = ShortcutRegistration {
        current: Some(replacement),
        error: None,
    };
    end_capture(&app);
    Ok(get_launcher_status(app.clone(), app.state()))
}

#[tauri::command]
pub fn set_shortcut_capture(
    app: AppHandle,
    controls: State<'_, LauncherControls>,
    active: bool,
) -> Result<(), String> {
    if active
        && !app.get_webview_window("main").is_some_and(|window| {
            window.is_visible().unwrap_or(false) && window.is_focused().unwrap_or(false)
        })
    {
        return Err("Open and focus Mycast before recording a shortcut.".into());
    }
    #[cfg(windows)]
    if active {
        crate::shortcut_capture::initialize(&app, controls.capture.clone())?;
    }
    controls.capture.store(active, Ordering::SeqCst);
    Ok(())
}

pub fn end_capture(app: &AppHandle) {
    if let Some(controls) = app.try_state::<LauncherControls>() {
        controls.capture.store(false, Ordering::SeqCst);
        let _ = app.emit("launcher-shortcut-capture-ended", "cancelled");
    }
}

/// Returns true when capture consumed the press instead of toggling the window.
pub fn capture_press(app: &AppHandle, shortcut: Shortcut) -> bool {
    let Some(controls) = app.try_state::<LauncherControls>() else {
        return false;
    };
    if controls.capture.swap(false, Ordering::SeqCst) {
        let _ = app.emit("launcher-shortcut-captured", shortcut.to_string());
        let _ = app.emit("launcher-shortcut-capture-ended", "completed");
        true
    } else {
        false
    }
}

#[tauri::command]
pub fn set_launch_at_startup(
    app: AppHandle,
    controls: State<'_, LauncherControls>,
    enabled: bool,
) -> Result<LauncherStatus, String> {
    if !cfg!(all(windows, not(debug_assertions))) {
        return Err("Startup is available in Windows release builds. Development builds depend on the Vite server.".into());
    }
    let manager = app.autolaunch();
    if enabled {
        manager.enable()
    } else {
        manager.disable()
    }
    .map_err(|error| format!("Could not change Windows startup registration: {error}"))?;
    let actual = manager
        .is_enabled()
        .map_err(|error| format!("Could not verify Windows startup registration: {error}"))?;
    if actual != enabled {
        return Err("Windows did not retain the requested startup setting.".into());
    }
    Ok(get_launcher_status(app, controls))
}
