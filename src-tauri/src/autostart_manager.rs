// Custom autostart manager for Linux that uses the wrapper script instead of the binary directly.
// This is necessary because tauri-plugin-autostart uses current_exe() which points to the binary,
// but we need to use the wrapper script that sets up the correct environment variables
// (GDK_BACKEND, TAURI_TRAY, etc.) for proper tray icon functionality.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

const DESKTOP_ENTRY_TEMPLATE: &str = r#"[Desktop Entry]
Type=Application
Version=1.1
Name=Clipboard History
GenericName=Clipboard Manager
Comment=Windows 11-style Clipboard History Manager
Exec=sh -c "sleep 5 && 'EXEC_PATH' --background"
Icon=win11-clipboard-history
Terminal=false
Categories=Utility;
StartupNotify=false
X-GNOME-Autostart-enabled=true
"#;

/// Get the path to the autostart directory
fn get_autostart_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("autostart"))
}

/// Get the path to the autostart desktop file
fn get_autostart_file() -> Option<PathBuf> {
    get_autostart_dir().map(|p| p.join("win11-clipboard-history.desktop"))
}

/// Read the content of the autostart desktop file
fn read_autostart_content() -> Option<String> {
    get_autostart_file().and_then(|p| fs::read_to_string(p).ok())
}

/// Determines the correct executable path to use in the autostart entry.
/// Prioritizes the wrapper script over the direct binary.
fn get_exec_path() -> String {
    // Priority order for the wrapper/binary
    let possible_paths = [
        "/usr/bin/win11-clipboard-history", // Wrapper installed by .deb/.rpm
        "/usr/local/bin/win11-clipboard-history", // Manual install with PREFIX=/usr/local
        "/usr/bin/win11-clipboard-history-bin", // Direct binary (fallback)
        "/usr/local/bin/win11-clipboard-history-bin", // Direct binary local (fallback)
    ];

    for path in &possible_paths {
        if std::path::Path::new(path).exists() {
            return path.to_string();
        }
    }

    // Last resort: use current executable
    std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "win11-clipboard-history".to_string())
}

/// Enable autostart by creating a .desktop file in ~/.config/autostart/
#[tauri::command]
pub fn autostart_enable() -> Result<(), String> {
    let autostart_dir = get_autostart_dir().ok_or("Could not determine config directory")?;
    let autostart_file = get_autostart_file().ok_or("Could not determine autostart file path")?;

    // Create autostart directory if it doesn't exist
    fs::create_dir_all(&autostart_dir)
        .map_err(|e| format!("Failed to create autostart directory: {}", e))?;

    // Get the correct executable path (wrapper preferred)
    let exec_path = get_exec_path();

    // Generate desktop entry content
    let content = DESKTOP_ENTRY_TEMPLATE.replace("EXEC_PATH", &exec_path);

    // Write the desktop file
    let mut file = fs::File::create(&autostart_file)
        .map_err(|e| format!("Failed to create autostart file: {}", e))?;

    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write autostart file: {}", e))?;

    println!(
        "[Autostart] Enabled autostart with exec path: {}",
        exec_path
    );

    Ok(())
}

/// Disable autostart by removing the .desktop file
#[tauri::command]
pub fn autostart_disable() -> Result<(), String> {
    let autostart_file = get_autostart_file().ok_or("Could not determine autostart file path")?;

    if autostart_file.exists() {
        fs::remove_file(&autostart_file)
            .map_err(|e| format!("Failed to remove autostart file: {}", e))?;
        println!("[Autostart] Disabled autostart");
    }

    Ok(())
}

/// Check if autostart is enabled
#[tauri::command]
pub fn autostart_is_enabled() -> Result<bool, String> {
    let autostart_file = get_autostart_file().ok_or("Could not determine autostart file path")?;

    if !autostart_file.exists() {
        return Ok(false);
    }

    // Check if the file has X-GNOME-Autostart-enabled=false
    let content = read_autostart_content().unwrap_or_default();

    // If the file exists and doesn't explicitly disable itself, it's enabled
    let is_disabled = content
        .lines()
        .any(|line| line.trim() == "X-GNOME-Autostart-enabled=false");

    Ok(!is_disabled)
}

/// Migrate from the old tauri-plugin-autostart entry to the new custom one
/// This fixes existing installations where the autostart points to the wrong binary
/// or is missing the startup delay for proper tray initialization
#[tauri::command]
pub fn autostart_migrate() -> Result<bool, String> {
    let autostart_file = get_autostart_file().ok_or("Could not determine autostart file path")?;

    if !autostart_file.exists() {
        return Ok(false); // Nothing to migrate
    }

    let content = read_autostart_content().unwrap_or_default();

    // Check if the Exec= line is using the old binary path directly
    let uses_old_binary = content
        .lines()
        .find(|line| line.trim_start().starts_with("Exec="))
        .is_some_and(|line| line.contains("win11-clipboard-history-bin"));

    // Check if the Exec= line is missing the sleep (for multi-distro compatibility)
    // We use sleep in exec instead of X-GNOME-Autostart-Delay for better compatibility
    let missing_sleep = content
        .lines()
        .find(|line| line.trim_start().starts_with("Exec="))
        .is_some_and(|line| !line.contains("sleep"));

    // Check if the Exec= line is missing the --background flag
    let missing_background = content
        .lines()
        .find(|line| line.trim_start().starts_with("Exec="))
        .is_some_and(|line| !line.contains("--background"));

    // Check if using deprecated X-GNOME-Autostart-Delay (should use sleep in exec instead)
    let has_gnome_delay = content
        .lines()
        .any(|line| line.trim_start().starts_with("X-GNOME-Autostart-Delay="));

    let needs_migration = uses_old_binary || missing_sleep || missing_background || has_gnome_delay;

    if needs_migration {
        if uses_old_binary {
            println!("[Autostart] Migrating from old binary path to wrapper...");
        }
        if missing_sleep {
            println!("[Autostart] Adding sleep to exec for proper tray initialization...");
        }
        if missing_background {
            println!("[Autostart] Adding --background flag for minimized startup...");
        }
        if has_gnome_delay {
            println!("[Autostart] Replacing X-GNOME-Autostart-Delay with sleep in exec (multi-distro compatibility)...");
        }

        // Re-enable with correct path, sleep and --background
        autostart_enable()?;

        return Ok(true); // Migration performed
    }

    Ok(false) // No migration needed
}
