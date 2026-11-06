use chrono::Local;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;
use uuid::Uuid;

/// Deliberate allowlist: no executable, directory, URI or arguments come from the UI.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativeAction {
    Notepad,
    Calculator,
    Explorer,
    WindowsSettings,
    Downloads,
    Documents,
    Desktop,
    CopyDate,
    CopyTime,
    CopyUuid,
}

#[derive(Serialize)]
pub struct ActionResult {
    message: String,
}

#[tauri::command]
pub fn hide_launcher(app: AppHandle) -> Result<(), String> {
    crate::launcher::end_capture(&app);
    let _ = app.emit("launcher-hidden", ());
    app.get_webview_window("main")
        .ok_or("Launcher window is unavailable")?
        .hide()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn quit_launcher(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub fn copy_text(app: AppHandle, text: String) -> Result<(), String> {
    if text.len() > 4096 {
        return Err("Text is too long to copy (maximum 4096 bytes).".into());
    }
    app.clipboard()
        .write_text(text)
        .map_err(|error| format!("Could not write to clipboard: {error}"))
}

#[tauri::command]
pub fn execute_native(app: AppHandle, action: NativeAction) -> Result<ActionResult, String> {
    let (value, message) = match action {
        NativeAction::CopyDate => (Local::now().format("%Y-%m-%d").to_string(), "Date copied"),
        NativeAction::CopyTime => (Local::now().format("%H:%M:%S").to_string(), "Time copied"),
        NativeAction::CopyUuid => (Uuid::new_v4().to_string(), "UUID copied"),
        action => return open_native(&app, action),
    };
    copy_text(app, value)?;
    Ok(ActionResult {
        message: message.into(),
    })
}

#[cfg(windows)]
fn open_native(app: &AppHandle, action: NativeAction) -> Result<ActionResult, String> {
    use std::{path::PathBuf, process::Command};

    // Absolute Windows paths avoid resolving an executable in the current directory.
    let windows_dir =
        PathBuf::from(std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into()));
    let explorer = windows_dir.join("explorer.exe");
    let (mut command, label) = match action {
        NativeAction::Notepad => (
            Command::new(windows_dir.join("System32\\notepad.exe")),
            "Notepad",
        ),
        NativeAction::Calculator => (
            Command::new(windows_dir.join("System32\\calc.exe")),
            "Calculator",
        ),
        NativeAction::Explorer => (Command::new(&explorer), "File Explorer"),
        NativeAction::WindowsSettings => {
            let mut command = Command::new(&explorer);
            command.arg("ms-settings:");
            (command, "Windows Settings")
        }
        NativeAction::Downloads | NativeAction::Documents | NativeAction::Desktop => {
            let (directory, label) = match action {
                NativeAction::Downloads => (app.path().download_dir(), "Downloads"),
                NativeAction::Documents => (app.path().document_dir(), "Documents"),
                _ => (app.path().desktop_dir(), "Desktop"),
            };
            let directory =
                directory.map_err(|error| format!("Could not locate {label}: {error}"))?;
            if !directory.is_dir() {
                return Err(format!("The {label} folder is unavailable."));
            }
            let mut command = Command::new(&explorer);
            command.arg(directory);
            (command, label)
        }
        _ => return Err("This action cannot open an application.".into()),
    };
    // Windows does not need a child reaper. Dropping its process handle allows
    // the launched application to continue without keeping a waiting thread.
    command
        .spawn()
        .map_err(|error| format!("Could not open {label}: {error}"))?;
    Ok(ActionResult {
        message: format!("Opened {label}"),
    })
}

#[cfg(not(windows))]
fn open_native(_app: &AppHandle, _action: NativeAction) -> Result<ActionResult, String> {
    Err("Opening applications and folders is supported on Windows only.".into())
}
