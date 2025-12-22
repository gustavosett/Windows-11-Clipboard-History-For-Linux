//! Permission checker module for Windows 11 Clipboard History
//! Handles uinput permission verification and fixing

use std::fs::OpenOptions;
use std::path::PathBuf;
use std::process::Command;

#[derive(serde::Serialize, Clone)]
pub struct PermissionStatus {
    pub uinput_accessible: bool,
    pub uinput_path: String,
    pub user_in_input_group: bool,
    pub suggestion: String,
}

/// Get the config directory path following XDG spec
fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string())).join(".config")
        })
        .join("win11-clipboard-history")
}

/// Get the config file path
fn get_config_path() -> PathBuf {
    get_config_dir().join("setup.json")
}

/// Verify if the user has access to /dev/uinput
#[tauri::command]
pub fn check_permissions() -> PermissionStatus {
    let uinput_path = "/dev/uinput";

    // Try to open for writing
    let uinput_accessible = OpenOptions::new().write(true).open(uinput_path).is_ok();

    // Check if user is in input group
    let user_in_input_group = Command::new("groups")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("input"))
        .unwrap_or(false);

    let suggestion = if uinput_accessible {
        "Permissions OK! Paste simulation will work.".to_string()
    } else if user_in_input_group {
        "You're in the 'input' group but /dev/uinput is not accessible. Try logging out and back in.".to_string()
    } else {
        "Missing permissions. Click 'Fix Permissions' or run: sudo usermod -aG input $USER && logout".to_string()
    };

    PermissionStatus {
        uinput_accessible,
        uinput_path: uinput_path.to_string(),
        user_in_input_group,
        suggestion,
    }
}

/// Check if a command exists in PATH
fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Apply ACL for immediate access (requires pkexec/sudo)
#[tauri::command]
pub fn fix_permissions_now() -> Result<String, String> {
    // Check required commands exist
    if !command_exists("pkexec") {
        return Err("pkexec not found. Install polkit or run manually: sudo setfacl -m u:$USER:rw /dev/uinput".to_string());
    }
    if !command_exists("setfacl") {
        return Err("setfacl not found. Install acl package (e.g., 'sudo apt install acl') or add yourself to the input group: sudo usermod -aG input $USER".to_string());
    }

    let username = whoami::username();

    // Use pkexec for graphical password prompt
    let status = Command::new("pkexec")
        .args([
            "setfacl",
            "-m",
            &format!("u:{}:rw", username),
            "/dev/uinput",
        ])
        .status()
        .map_err(|e| format!("Failed to run pkexec: {}", e))?;

    if status.success() {
        Ok("Permission granted! Paste should work now.".to_string())
    } else {
        Err("Failed to set permissions. Try running manually: sudo setfacl -m u:$USER:rw /dev/uinput".to_string())
    }
}

/// Check if this is the first run of the application
#[tauri::command]
pub fn is_first_run() -> bool {
    !get_config_path().exists()
}

/// Mark the first run as complete
#[tauri::command]
pub fn mark_first_run_complete() -> Result<(), String> {
    let config_path = get_config_path();

    // Create directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
    }

    // Create initial setup config file
    let config_content = serde_json::json!({
        "setupComplete": true,
        "setupDate": chrono::Utc::now().to_rfc3339()
    });

    std::fs::write(&config_path, config_content.to_string())
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(())
}
