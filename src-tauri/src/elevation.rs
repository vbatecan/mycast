use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Manager;
static PENDING: AtomicBool = AtomicBool::new(false);
#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LaunchMode {
    Normal,
    Administrator,
}
pub struct Pending;
impl Pending {
    pub fn new() -> Result<Self, String> {
        PENDING
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .map(|_| Self)
            .map_err(|_| "An administrator request is already pending.".into())
    }
}
impl Drop for Pending {
    fn drop(&mut self) {
        PENDING.store(false, Ordering::SeqCst);
    }
}
pub fn is_pending() -> bool {
    PENDING.load(Ordering::SeqCst)
}
pub fn restore(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
#[cfg(windows)]
pub fn run(executable: &str, arguments: &str, directory: &str) -> Result<bool, String> {
    use windows::{
        Win32::{
            System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize},
            UI::{
                Shell::{
                    SEE_MASK_FLAG_NO_UI, SEE_MASK_NOASYNC, SHELLEXECUTEINFOW, ShellExecuteExW,
                },
                WindowsAndMessaging::SW_SHOWNORMAL,
            },
        },
        core::{PCWSTR, w},
    };
    struct Apartment;
    impl Drop for Apartment {
        fn drop(&mut self) {
            unsafe {
                CoUninitialize();
            }
        }
    }
    unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }
        .ok()
        .map_err(|e| e.to_string())?;
    let _apartment = Apartment;
    let wide = |s: &str| s.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    let exe = wide(executable);
    let args = wide(arguments);
    let dir = wide(directory);
    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOASYNC | SEE_MASK_FLAG_NO_UI,
        lpVerb: w!("runas"),
        lpFile: PCWSTR(exe.as_ptr()),
        lpParameters: PCWSTR(args.as_ptr()),
        lpDirectory: if directory.is_empty() {
            PCWSTR::null()
        } else {
            PCWSTR(dir.as_ptr())
        },
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };
    match unsafe { ShellExecuteExW(&mut info) } {
        Ok(()) => Ok(true),
        Err(e) if e.code().0 as u32 == 0x800704c7 => Ok(false),
        Err(e) => Err(format!("Could not launch as administrator: {e}")),
    }
}
#[cfg(not(windows))]
pub fn run(_: &str, _: &str, _: &str) -> Result<bool, String> {
    Err("Administrator launching requires Windows.".into())
}
pub fn notepad() -> Result<bool, String> {
    let path = std::path::PathBuf::from(
        std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into()),
    )
    .join("System32/notepad.exe");
    run(&path.to_string_lossy(), "", "")
}
