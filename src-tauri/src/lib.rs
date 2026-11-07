mod ai;
mod appearance;
mod applications;
mod calculator;
mod commands;
mod elevation;
mod files;
mod launcher;
mod modules;
mod search;
#[cfg(windows)]
mod shortcut_capture;

use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, WindowEvent,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_global_shortcut::ShortcutState;

fn position_launcher(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    if let Some(monitor) = window.primary_monitor()? {
        let area = monitor.work_area();
        let size = window.outer_size()?;
        let x = area.position.x + (area.size.width as i32 - size.width as i32).max(0) / 2;
        // Put the palette's center at the upper third, staying inside the work area.
        let offset = ((area.size.height as f64 / 3.0) - size.height as f64 / 2.0).max(24.0);
        let y = area.position.y + offset.round() as i32;
        window.set_position(PhysicalPosition::new(x, y))?;
    }
    Ok(())
}

fn show_launcher(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        position_launcher(&window)?;
        window.show()?;
        window.set_focus()?;
        window.emit("launcher-shown", ())?;
    }
    Ok(())
}

fn toggle_launcher(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let result = if window.is_visible().unwrap_or(false) {
            launcher::end_capture(app);
            let _ = window.emit("launcher-hidden", ());
            window.hide()
        } else {
            show_launcher(app)
        };
        if let Err(error) = result {
            eprintln!("Could not toggle launcher: {error}");
        }
    }
}

#[cfg(windows)]
fn has_owned_foreground(window: &tauri::Window) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{GA_ROOTOWNER, GetAncestor, GetForegroundWindow};

    let Ok(main) = window.hwnd() else {
        return false;
    };
    // A file/color chooser is an owned native dialog. Changing focus to it
    // should keep its parent visible; switching to another app still hides it.
    // Comparing the foreground window also ignores a stale queued blur event.
    unsafe {
        let foreground = GetForegroundWindow();
        !foreground.0.is_null()
            && (foreground.0 == main.0 || GetAncestor(foreground, GA_ROOTOWNER).0 == main.0)
    }
}

#[cfg(not(windows))]
fn has_owned_foreground(_window: &tauri::Window) -> bool {
    false
}

#[tauri::command]
fn updater_supported() -> bool {
    cfg!(windows) || (cfg!(target_os = "linux") && std::env::var_os("APPIMAGE").is_some())
}
pub fn run() {
    tauri::Builder::default()
        // Register first: a second launch activates the existing process and exits.
        .plugin(tauri_plugin_single_instance::init(|app, args, _| {
            if !args.iter().any(|arg| arg == "--background") {
                if let Err(error) = show_launcher(app) {
                    eprintln!("Could not activate launcher: {error}");
                }
            }
        }))
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--background"]),
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state == ShortcutState::Pressed
                        && !launcher::capture_press(app, *shortcut)
                    {
                        toggle_launcher(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            app.manage(ai::AiState::default());
            app.manage(modules::Modules::load(app.handle()));
            app.manage(applications::ApplicationCatalog::default());
            app.manage(files::FileIndex::new(app.handle()));
            app.manage(launcher::LauncherControls::default());
            launcher::initialize(app.handle());

            let show = MenuItem::with_id(app, "show", "Open Mycast", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit Mycast", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            let mut tray = TrayIconBuilder::with_id("mycast")
                .tooltip("Mycast — command launcher")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        let _ = show_launcher(app);
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if matches!(
                        event,
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        }
                    ) {
                        let _ = show_launcher(tray.app_handle());
                    }
                });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;
            if !std::env::args().any(|arg| arg == "--background") {
                show_launcher(app.handle())?;
            }
            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::Focused(false) => {
                if elevation::is_pending() {
                    return;
                }
                launcher::end_capture(window.app_handle());
                if !has_owned_foreground(window) {
                    let _ = window.emit("launcher-hidden", ());
                    let _ = window.hide();
                }
            }
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                launcher::end_capture(window.app_handle());
                let _ = window.emit("launcher-hidden", ());
                let _ = window.hide();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            ai::open_ai_link,
            ai::get_ai_config,
            ai::save_ai_config,
            ai::ask_ai,
            ai::cancel_ai,
            search::list_windows_commands,
            search::open_windows_command,
            search::open_search_url,
            modules::get_modules,
            modules::set_module,
            calculator::evaluate_math,
            updater_supported,
            appearance::set_window_material,
            launcher::get_launcher_status,
            launcher::set_launcher_shortcut,
            launcher::set_shortcut_capture,
            launcher::set_launch_at_startup,
            commands::execute_native,
            commands::copy_text,
            commands::hide_launcher,
            commands::quit_launcher,
            applications::list_applications,
            applications::launch_application,
            applications::get_application_icon,
            files::get_file_status,
            files::search_files,
            files::resolve_files,
            files::rebuild_file_index,
            files::set_file_preferences,
            files::remember_file,
            files::file_action,
            files::choose_file_folder,
            files::run_shell_command,
        ])
        .run(tauri::generate_context!())
        .expect("Could not start Mycast");
}
