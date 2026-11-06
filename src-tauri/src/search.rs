use serde_json::{Value, json};

struct Entry {
    id: &'static str,
    label: &'static str,
    executable: &'static str,
    argument: &'static str,
    keywords: &'static str,
}
const ENTRIES: &[Entry] = &[
    Entry {
        id: "installed-apps",
        label: "Installed apps",
        executable: "",
        argument: "ms-settings:appsfeatures",
        keywords: "add or remove programs|uninstall apps|apps and features",
    },
    Entry {
        id: "default-apps",
        label: "Default apps",
        executable: "",
        argument: "ms-settings:defaultapps",
        keywords: "file associations|browser",
    },
    Entry {
        id: "startup-apps",
        label: "Startup apps",
        executable: "",
        argument: "ms-settings:startupapps",
        keywords: "startup programs",
    },
    Entry {
        id: "display",
        label: "Display",
        executable: "",
        argument: "ms-settings:display",
        keywords: "screen|resolution|monitor",
    },
    Entry {
        id: "sound",
        label: "Sound",
        executable: "",
        argument: "ms-settings:sound",
        keywords: "volume|audio|microphone",
    },
    Entry {
        id: "bluetooth",
        label: "Bluetooth",
        executable: "",
        argument: "ms-settings:bluetooth",
        keywords: "devices|pair",
    },
    Entry {
        id: "wifi",
        label: "Wi-Fi",
        executable: "",
        argument: "ms-settings:network-wifi",
        keywords: "wireless|internet",
    },
    Entry {
        id: "ethernet",
        label: "Ethernet",
        executable: "",
        argument: "ms-settings:network-ethernet",
        keywords: "wired network",
    },
    Entry {
        id: "vpn",
        label: "VPN",
        executable: "",
        argument: "ms-settings:network-vpn",
        keywords: "virtual private network",
    },
    Entry {
        id: "proxy",
        label: "Network proxy",
        executable: "",
        argument: "ms-settings:network-proxy",
        keywords: "proxy server",
    },
    Entry {
        id: "windows-update",
        label: "Windows Update",
        executable: "",
        argument: "ms-settings:windowsupdate",
        keywords: "updates",
    },
    Entry {
        id: "storage",
        label: "Storage",
        executable: "",
        argument: "ms-settings:storagesense",
        keywords: "disk space|cleanup",
    },
    Entry {
        id: "power",
        label: "Power and battery",
        executable: "",
        argument: "ms-settings:powersleep",
        keywords: "sleep|battery",
    },
    Entry {
        id: "notifications",
        label: "Notifications",
        executable: "",
        argument: "ms-settings:notifications",
        keywords: "alerts",
    },
    Entry {
        id: "personalization",
        label: "Personalization",
        executable: "",
        argument: "ms-settings:personalization",
        keywords: "wallpaper|theme",
    },
    Entry {
        id: "taskbar",
        label: "Taskbar",
        executable: "",
        argument: "ms-settings:taskbar",
        keywords: "task bar",
    },
    Entry {
        id: "date-time",
        label: "Date and time",
        executable: "",
        argument: "ms-settings:dateandtime",
        keywords: "clock|timezone",
    },
    Entry {
        id: "language",
        label: "Language and region",
        executable: "",
        argument: "ms-settings:regionlanguage",
        keywords: "language|region",
    },
    Entry {
        id: "accessibility",
        label: "Accessibility",
        executable: "",
        argument: "ms-settings:easeofaccess",
        keywords: "ease of access",
    },
    Entry {
        id: "privacy",
        label: "Privacy and security",
        executable: "",
        argument: "ms-settings:privacy",
        keywords: "permissions",
    },
    Entry {
        id: "security",
        label: "Windows Security",
        executable: "",
        argument: "ms-settings:windowsdefender",
        keywords: "antivirus|defender",
    },
    Entry {
        id: "signin",
        label: "Sign-in options",
        executable: "",
        argument: "ms-settings:signinoptions",
        keywords: "password|pin|hello",
    },
    Entry {
        id: "about",
        label: "About your PC",
        executable: "",
        argument: "ms-settings:about",
        keywords: "specifications|system info",
    },
    Entry {
        id: "control-panel",
        label: "Control Panel",
        executable: "control.exe",
        argument: "",
        keywords: "control panel",
    },
    Entry {
        id: "programs-features",
        label: "Programs and Features",
        executable: "control.exe",
        argument: "appwiz.cpl",
        keywords: "classic uninstall",
    },
    Entry {
        id: "task-manager",
        label: "Task Manager",
        executable: "Taskmgr.exe",
        argument: "",
        keywords: "processes",
    },
    Entry {
        id: "device-manager",
        label: "Device Manager",
        executable: "mmc.exe",
        argument: "devmgmt.msc",
        keywords: "drivers|hardware",
    },
    Entry {
        id: "services",
        label: "Services",
        executable: "mmc.exe",
        argument: "services.msc",
        keywords: "windows services",
    },
    Entry {
        id: "disk-management",
        label: "Disk Management",
        executable: "mmc.exe",
        argument: "diskmgmt.msc",
        keywords: "partitions|disks",
    },
    Entry {
        id: "system-information",
        label: "System Information",
        executable: "msinfo32.exe",
        argument: "",
        keywords: "system specs",
    },
    Entry {
        id: "environment-variables",
        label: "Environment Variables",
        executable: "rundll32.exe",
        argument: "sysdm.cpl,EditEnvironmentVariables",
        keywords: "path|environment",
    },
];

#[tauri::command]
pub fn list_windows_commands() -> Vec<Value> {
    if !cfg!(windows) {
        return vec![];
    }
    ENTRIES.iter().map(|entry| json!({
        "id": format!("windows-{}", entry.id),
        "personalizationKey": format!("windows:{}", entry.id),
        "module": "windows", "label": entry.label,
        "description": if entry.executable.is_empty() { "Windows Settings" } else { "Windows system tool" },
        "category": "Windows", "icon": "settings-2",
        "keywords": entry.keywords.split('|').collect::<Vec<_>>(),
        "action": { "type": "windows", "id": entry.id }
    })).collect()
}

#[tauri::command]
pub fn open_windows_command(id: String) -> Result<String, String> {
    crate::modules::require("windows")?;
    let entry = ENTRIES
        .iter()
        .find(|entry| entry.id == id)
        .ok_or("Unknown Windows command")?;
    launch(entry)?;
    Ok(format!("Opened {}", entry.label))
}

#[cfg(windows)]
fn launch(entry: &Entry) -> Result<(), String> {
    use std::{path::PathBuf, process::Command};
    let root =
        PathBuf::from(std::env::var_os("SystemRoot").unwrap_or_else(|| r"C:\Windows".into()));
    if entry.executable.is_empty() {
        return open_uri(entry.argument);
    }
    let mut command = Command::new(root.join("System32").join(entry.executable));
    if !entry.argument.is_empty() {
        command.arg(entry.argument);
    }
    command
        .spawn()
        .map_err(|error| format!("Could not open {}: {error}", entry.label))?;
    Ok(())
}
#[cfg(not(windows))]
fn launch(_entry: &Entry) -> Result<(), String> {
    Err("Windows settings and tools are available on Windows only.".into())
}

pub(crate) fn validate_url(value: &str) -> Result<(), String> {
    let url = tauri::Url::parse(value).map_err(|_| "Invalid search URL")?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("Search URLs must use HTTP or HTTPS without credentials.".into());
    }
    Ok(())
}
#[tauri::command]
pub fn open_search_url(url: String) -> Result<String, String> {
    crate::modules::require("web-search")?;
    validate_url(&url)?;
    open_uri(&url)?;
    Ok("Opened search in your default browser".into())
}
#[cfg(windows)]
pub(crate) fn open_uri(uri: &str) -> Result<(), String> {
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    use windows::core::PCWSTR;
    let verb: Vec<u16> = "open".encode_utf16().chain(Some(0)).collect();
    let target: Vec<u16> = uri.encode_utf16().chain(Some(0)).collect();
    let result = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(verb.as_ptr()),
            PCWSTR(target.as_ptr()),
            None,
            None,
            SW_SHOWNORMAL,
        )
    };
    if result.0 as isize <= 32 {
        return Err("Windows could not open this destination.".into());
    }
    Ok(())
}
#[cfg(not(windows))]
pub(crate) fn open_uri(url: &str) -> Result<(), String> {
    std::process::Command::new("xdg-open")
        .arg(url)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalogue_ids_are_unique() {
        assert!(open_windows_command("unknown-command".into()).is_err());
        let mut ids = std::collections::HashSet::new();
        for entry in ENTRIES {
            assert!(ids.insert(entry.id));
        }
        assert!(
            ENTRIES
                .iter()
                .any(|entry| entry.keywords.contains("add or remove programs"))
        );
    }
    #[test]
    fn only_web_urls_are_accepted() {
        for url in ["https://example.com/?q=test", "http://localhost/search"] {
            assert!(validate_url(url).is_ok());
        }
        for url in [
            "file:///C:/Windows",
            "javascript:alert(1)",
            "https://user:pass@example.com",
            "invalid",
        ] {
            assert!(validate_url(url).is_err());
        }
    }
}
