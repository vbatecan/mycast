use serde::Deserialize;

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowMaterial {
    Solid,
    Transparent,
    Blur,
    Acrylic,
    Mica,
}

// Synchronous Tauri commands run on the main thread, where HWND effects belong.
#[tauri::command]
pub fn set_window_material(
    window: tauri::WebviewWindow,
    material: WindowMaterial,
    dark: bool,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        use window_vibrancy::{
            apply_acrylic, apply_blur, apply_mica, clear_acrylic, clear_blur, clear_mica,
        };
        // Remove the previous material before switching, including back to Solid.
        let _ = clear_blur(&window);
        let _ = clear_acrylic(&window);
        let _ = clear_mica(&window);
        let result = match material {
            WindowMaterial::Solid | WindowMaterial::Transparent => Ok(()),
            WindowMaterial::Blur => apply_blur(&window, Some((0, 0, 0, 1))),
            WindowMaterial::Acrylic => apply_acrylic(&window, Some((0, 0, 0, 1))),
            WindowMaterial::Mica => apply_mica(&window, Some(dark)),
        };
        result.map_err(|error| error.to_string())
    }
    #[cfg(not(windows))]
    {
        let _ = (window, dark);
        match material {
            WindowMaterial::Solid | WindowMaterial::Transparent => Ok(()),
            _ => Err("Native backdrop materials require Windows.".into()),
        }
    }
}
