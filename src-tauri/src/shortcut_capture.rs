//! Intercept recorder input before Windows menus, hotkeys, and WebView accelerators.
//! The hook thread only tracks keys and queues events; IPC runs on a worker.
use std::{
    cell::RefCell,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};
use tauri::{AppHandle, Emitter, Manager};
use windows::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    UI::{
        Input::KeyboardAndMouse::GetAsyncKeyState,
        WindowsAndMessaging::{
            CallNextHookEx, DispatchMessageW, GetForegroundWindow, GetMessageW, KBDLLHOOKSTRUCT,
            MSG, SetWindowsHookExW, UnhookWindowsHookEx, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP,
            WM_SYSKEYDOWN, WM_SYSKEYUP,
        },
    },
};

enum CaptureEvent {
    Binding(String),
    Error(&'static str),
    Cancel,
}

struct Recorder {
    // Only key-downs consumed by this hook own their corresponding key-up.
    swallowed: [bool; 256],
    active: Arc<AtomicBool>,
    foreground: usize,
    events: mpsc::Sender<CaptureEvent>,
}

thread_local! { static RECORDER: RefCell<Option<Recorder>> = const { RefCell::new(None) }; }
static INITIALIZED: OnceLock<Result<(), String>> = OnceLock::new();

pub fn initialize(app: &AppHandle, active: Arc<AtomicBool>) -> Result<(), String> {
    INITIALIZED
        .get_or_init(|| {
            let foreground = app
                .get_webview_window("main")
                .ok_or("Mycast window is unavailable.")?
                .hwnd()
                .map_err(|e| e.to_string())?
                .0 as usize;
            let (events, received) = mpsc::channel();
            let (ready, started) = mpsc::sync_channel(1);
            std::thread::Builder::new()
                .name("shortcut-recorder".into())
                .spawn(move || unsafe {
                    RECORDER.with(|slot| {
                        *slot.borrow_mut() = Some(Recorder {
                            swallowed: [false; 256],
                            active,
                            foreground,
                            events,
                        })
                    });
                    let hook = match SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), None, 0)
                    {
                        Ok(hook) => hook,
                        Err(error) => {
                            let _ = ready
                                .send(Err(format!("Could not record keyboard input: {error}")));
                            return;
                        }
                    };
                    let _ = ready.send(Ok(()));
                    let mut message = MSG::default();
                    while GetMessageW(&mut message, None, 0, 0).0 > 0 {
                        DispatchMessageW(&message);
                    }
                    let _ = UnhookWindowsHookEx(hook);
                })
                .map_err(|e| e.to_string())?;
            started.recv().map_err(|e| e.to_string())??;
            let app = app.clone();
            std::thread::Builder::new()
                .name("shortcut-recorder-events".into())
                .spawn(move || {
                    for event in received {
                        match event {
                            CaptureEvent::Binding(binding) => {
                                let _ = app.emit("launcher-shortcut-captured", binding);
                                let _ = app.emit("launcher-shortcut-capture-ended", "completed");
                            }
                            CaptureEvent::Error(error) => {
                                let _ = app.emit("launcher-shortcut-capture-error", error);
                            }
                            CaptureEvent::Cancel => {
                                let _ = app.emit("launcher-shortcut-capture-ended", "cancelled");
                            }
                        }
                    }
                })
                .map_err(|e| e.to_string())?;
            Ok(())
        })
        .clone()
}

unsafe extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let message = wparam.0 as u32;
        if matches!(message, WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP) {
            let key = unsafe { (*(lparam.0 as *const KBDLLHOOKSTRUCT)).vkCode } as usize;
            let down = matches!(message, WM_KEYDOWN | WM_SYSKEYDOWN);
            let consumed = RECORDER.with(|slot| {
                let mut slot = slot.borrow_mut();
                let Some(recorder) = slot.as_mut() else {
                    return false;
                };
                let focused = unsafe { GetForegroundWindow().0 as usize == recorder.foreground };
                recorder.key(key, down, focused, |vk| unsafe {
                    GetAsyncKeyState(vk as i32) < 0
                })
            });
            if consumed {
                return LRESULT(1);
            }
        }
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

impl Recorder {
    fn key(&mut self, key: usize, down: bool, focused: bool, held: impl Fn(u16) -> bool) -> bool {
        if key >= self.swallowed.len() {
            return false;
        }
        if !down {
            return std::mem::take(&mut self.swallowed[key]);
        }
        if self.swallowed[key] {
            return true;
        } // Auto-repeat and releases after completion.
        if !self.active.load(Ordering::SeqCst) {
            return false;
        }
        if !focused {
            self.active.store(false, Ordering::SeqCst);
            let _ = self.events.send(CaptureEvent::Cancel);
            return false;
        }
        self.swallowed[key] = true;
        if matches!(key, 0x10..=0x12 | 0x5B..=0x5C | 0xA0..=0xA5) {
            return true;
        }
        if key == 0x1B {
            self.active.store(false, Ordering::SeqCst);
            let _ = self.events.send(CaptureEvent::Cancel);
            return true;
        }
        let pressed = |keys: &[u16]| {
            keys.iter()
                .any(|&vk| self.swallowed[vk as usize] || held(vk))
        };
        let ctrl = pressed(&[0x11, 0xA2, 0xA3]);
        let alt = pressed(&[0x12, 0xA4, 0xA5]);
        let shift = pressed(&[0x10, 0xA0, 0xA1]);
        let super_key = pressed(&[0x5B, 0x5C]);
        if !(ctrl || alt || super_key) {
            let _ = self.events.send(CaptureEvent::Error(
                "Include Ctrl, Alt, or Windows with a key.",
            ));
        } else if let Some(key) = key_name(key) {
            let mut parts = Vec::new();
            if ctrl {
                parts.push("Ctrl".into());
            }
            if alt {
                parts.push("Alt".into());
            }
            if shift {
                parts.push("Shift".into());
            }
            if super_key {
                parts.push("Super".into());
            }
            parts.push(key);
            self.active.store(false, Ordering::SeqCst);
            let _ = self.events.send(CaptureEvent::Binding(parts.join("+")));
        } else {
            let _ = self
                .events
                .send(CaptureEvent::Error("Choose a different key."));
        }
        true
    }
}

fn key_name(key: usize) -> Option<String> {
    Some(match key {
        0x30..=0x39 | 0x41..=0x5A => char::from_u32(key as u32)?.to_string(),
        0x70..=0x87 => format!("F{}", key - 0x70 + 1),
        0x60..=0x69 => format!("Numpad{}", key - 0x60),
        0x08 => "Backspace".into(),
        0x0D => "Enter".into(),
        0x20 => "Space".into(),
        0x21 => "PageUp".into(),
        0x22 => "PageDown".into(),
        0x23 => "End".into(),
        0x24 => "Home".into(),
        0x25 => "ArrowLeft".into(),
        0x26 => "ArrowUp".into(),
        0x27 => "ArrowRight".into(),
        0x28 => "ArrowDown".into(),
        0x2D => "Insert".into(),
        0x2E => "Delete".into(),
        0x6A => "NumpadMultiply".into(),
        0x6B => "NumpadAdd".into(),
        0x6D => "NumpadSubtract".into(),
        0x6E => "NumpadDecimal".into(),
        0x6F => "NumpadDivide".into(),
        0xBA => "Semicolon".into(),
        0xBB => "Equal".into(),
        0xBC => "Comma".into(),
        0xBD => "Minus".into(),
        0xBE => "Period".into(),
        0xBF => "Slash".into(),
        0xC0 => "Backquote".into(),
        0xDB => "BracketLeft".into(),
        0xDC => "Backslash".into(),
        0xDD => "BracketRight".into(),
        0xDE => "Quote".into(),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn windows_accepts_the_keyboard_hook() {
        // Exercise the actual OS registration without generating desktop input.
        unsafe {
            let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), None, 0).unwrap();
            UnhookWindowsHookEx(hook).unwrap();
        }
    }
    fn recorder() -> (Recorder, mpsc::Receiver<CaptureEvent>) {
        let (events, received) = mpsc::channel();
        (
            Recorder {
                swallowed: [false; 256],
                active: Arc::new(AtomicBool::new(true)),
                foreground: 0,
                events,
            },
            received,
        )
    }
    #[test]
    fn modifiers_and_key_releases_are_swallowed_after_completion() {
        for modifier in [0xA4, 0xA5, 0x5B, 0x5C] {
            let (mut recorder, events) = recorder();
            assert!(recorder.key(modifier, true, true, |_| false));
            assert!(events.try_recv().is_err());
            assert!(recorder.key(0x20, true, true, |_| false));
            let CaptureEvent::Binding(binding) = events.recv().unwrap() else {
                panic!("Expected binding")
            };
            assert_eq!(
                binding,
                if modifier >= 0xA4 {
                    "Alt+Space"
                } else {
                    "Super+Space"
                }
            );
            assert!(recorder.key(0x20, true, true, |_| false));
            assert!(recorder.key(modifier, false, true, |_| false));
            assert!(recorder.key(0x20, false, true, |_| false));
            assert!(!recorder.key(0x41, true, true, |_| false));
        }
    }
    #[test]
    fn escape_and_focus_loss_cancel_and_invalid_input_can_be_retried() {
        let (mut recorder, events) = recorder();
        assert!(recorder.key(0x41, true, true, |_| false));
        assert!(matches!(events.recv().unwrap(), CaptureEvent::Error(_)));
        assert!(recorder.active.load(Ordering::SeqCst));
        assert!(recorder.key(0x1B, true, true, |_| false));
        assert!(matches!(events.recv().unwrap(), CaptureEvent::Cancel));
        assert!(recorder.key(0x1B, false, true, |_| false));
        recorder.active.store(true, Ordering::SeqCst);
        assert!(!recorder.key(0x42, true, false, |_| false));
        assert!(matches!(events.recv().unwrap(), CaptureEvent::Cancel));
    }
    #[test]
    fn already_held_modifiers_keep_their_normal_release() {
        let (mut recorder, events) = recorder();
        assert!(recorder.key(0x4B, true, true, |vk| [0xA2, 0xA1, 0x5C].contains(&vk)));
        let CaptureEvent::Binding(binding) = events.recv().unwrap() else {
            panic!("Expected binding")
        };
        assert_eq!(binding, "Ctrl+Shift+Super+K");
        assert!(!recorder.key(0x5C, false, true, |_| false));
    }
    #[test]
    fn captured_keys_are_accepted_by_the_global_shortcut_parser() {
        for key in 0..256 {
            if let Some(name) = key_name(key) {
                for modifier in ["Alt", "Super", "Ctrl+Alt+Shift+Super"] {
                    let binding = format!("{modifier}+{name}");
                    assert!(
                        binding
                            .parse::<tauri_plugin_global_shortcut::Shortcut>()
                            .is_ok(),
                        "{binding}"
                    );
                }
            }
        }
    }
}
