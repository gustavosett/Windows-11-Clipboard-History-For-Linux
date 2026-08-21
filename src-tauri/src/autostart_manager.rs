//! Shell-free Linux autostart entries that launch the sanitized wrapper.
//! ورودی‌های بدون shell اجرای خودکار لینوکس برای اجرای wrapper پاک‌سازی‌شده.


use std::fs;
use std::path::PathBuf;
use tauri::WebviewWindow;

use crate::error::AppError;
use crate::window_policy;

/// Autostart desktop entry. Exec never goes through a shell.
/// / ورودی autostart. Exec هرگز از شل عبور نمی‌کند.
///
/// Delay is expressed with `X-GNOME-Autostart-Delay` (GNOME) rather than
/// `sh -c "sleep …"` so the command line cannot be injected into.
/// تأخیر با کلید دسکتاپ بیان می‌شود، نه با `sh -c`.
const DESKTOP_ENTRY_TEMPLATE: &str = r#"[Desktop Entry]
Type=Application
Version=1.1
Name=Clipboard History
GenericName=Clipboard Manager
Comment=Windows 11-style Clipboard History Manager
Exec="EXEC_PATH" --background
Icon=io.github.mahdi-arts.clipboard-history
Terminal=false
Categories=Utility;
StartupNotify=false
X-GNOME-Autostart-enabled=true
X-GNOME-Autostart-Delay=5
"#;

/// Get the path to the autostart directory
fn get_autostart_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("autostart"))
}

/// Get the path to the autostart desktop file
fn get_autostart_file() -> Option<PathBuf> {
    get_autostart_dir().map(|p| p.join("windows-11-style-clipboard-history-manager.desktop"))
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
        "/usr/bin/windows-11-style-clipboard-history-manager", // Wrapper installed by .deb/.rpm
        "/usr/local/bin/windows-11-style-clipboard-history-manager", // Manual install with PREFIX=/usr/local
        "/usr/bin/windows-11-style-clipboard-history-manager-bin", // Direct binary (fallback)
        "/usr/local/bin/windows-11-style-clipboard-history-manager-bin", // Direct binary local (fallback)
    ];

    for path in &possible_paths {
        if std::path::Path::new(path).exists() {
            return path.to_string();
        }
    }

    // Last resort: use current executable
    std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "windows-11-style-clipboard-history-manager".to_string())
}

/// Enable autostart by creating a .desktop file in ~/.config/autostart/
fn enable() -> Result<(), String> {
    let autostart_dir = get_autostart_dir().ok_or("Could not determine config directory")?;
    let autostart_file = get_autostart_file().ok_or("Could not determine autostart file path")?;

    // Create autostart directory if it doesn't exist
    fs::create_dir_all(&autostart_dir)
        .map_err(|e| format!("Failed to create autostart directory: {}", e))?;

    // Get the correct executable path (wrapper preferred). Reject control
    // characters so they cannot break out of the quoted Exec= line.
    let exec_path = get_exec_path();
    if exec_path.chars().any(|c| c.is_control() || c == '"') {
        return Err("Refusing to write autostart entry with an unsafe executable path".into());
    }

    // Generate desktop entry content (path is quoted in the template).
    let content = DESKTOP_ENTRY_TEMPLATE.replace("EXEC_PATH", &exec_path);

    // Atomic owner-only write; never leave a partially written Exec entry.
    // نوشتن اتمیک فقط برای مالک؛ ورودی Exec نیمه‌نوشته باقی نمی‌ماند.
    crate::fs_atomic::write_atomic(&autostart_file, content.as_bytes())
        .map_err(|error| format!("Failed to write autostart file: {error}"))?;

    tracing::info!(
        "[Autostart] Enabled autostart with exec path: {}",
        exec_path
    );

    Ok(())
}

/// Disable autostart by removing the .desktop file
fn disable() -> Result<(), String> {
    let autostart_file = get_autostart_file().ok_or("Could not determine autostart file path")?;

    if autostart_file.exists() {
        fs::remove_file(&autostart_file)
            .map_err(|e| format!("Failed to remove autostart file: {}", e))?;
        tracing::info!("[Autostart] Disabled autostart");
    }

    Ok(())
}

/// Check if autostart is enabled
fn is_enabled() -> Result<bool, String> {
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
pub fn migrate_native() -> Result<bool, String> {
    let autostart_file = get_autostart_file().ok_or("Could not determine autostart file path")?;

    if !autostart_file.exists() {
        return Ok(false); // Nothing to migrate
    }

    let content = read_autostart_content().unwrap_or_default();

    // Check if the Exec= line is using the old binary path directly
    let uses_old_binary = content
        .lines()
        .find(|line| line.trim_start().starts_with("Exec="))
        .is_some_and(|line| {
            line.contains("windows-11-style-clipboard-history-manager-bin")
                || line.contains("modern-clipboard-history-for-linux-bin")
                || line.contains("win11-clipboard-history-bin")
        });

    // Legacy entries used `sh -c "sleep 5 && …"` — migrate them off the shell.
    let uses_shell = content
        .lines()
        .find(|line| line.trim_start().starts_with("Exec="))
        .is_some_and(|line| line.contains("sh -c") || line.contains("sleep"));

    // Check if the Exec= line is missing the --background flag
    let missing_background = content
        .lines()
        .find(|line| line.trim_start().starts_with("Exec="))
        .is_some_and(|line| !line.contains("--background"));

    let needs_migration = uses_old_binary || uses_shell || missing_background;

    if needs_migration {
        if uses_old_binary {
            tracing::info!("[Autostart] Migrating from old binary path to wrapper...");
        }
        if uses_shell {
            tracing::info!("[Autostart] Removing shell wrapper from Exec= line...");
        }
        if missing_background {
            tracing::info!("[Autostart] Adding --background flag for minimized startup...");
        }
        // Re-enable with a quoted Exec= line (no shell) and --background
        enable()?;

        return Ok(true); // Migration performed
    }

    Ok(false) // No migration needed
}

/// Enable login startup from Settings or Setup only.
/// فعال‌سازی اجرای هنگام ورود فقط از تنظیمات یا راه‌اندازی.
#[tauri::command]
pub fn autostart_enable(window: WebviewWindow) -> Result<(), AppError> {
    window_policy::require_configuration(&window)?;
    enable().map_err(AppError::Other)
}

/// Disable login startup from Settings or Setup only.
/// غیرفعال‌سازی اجرای هنگام ورود فقط از تنظیمات یا راه‌اندازی.
#[tauri::command]
pub fn autostart_disable(window: WebviewWindow) -> Result<(), AppError> {
    window_policy::require_configuration(&window)?;
    disable().map_err(AppError::Other)
}

/// Read login-startup state from configuration windows.
/// خواندن وضعیت اجرای هنگام ورود از پنجره‌های پیکربندی.
#[tauri::command]
pub fn autostart_is_enabled(window: WebviewWindow) -> Result<bool, AppError> {
    window_policy::require_configuration(&window)?;
    is_enabled().map_err(AppError::Other)
}

/// Migrate a legacy autostart entry from configuration windows.
/// مهاجرت ورودی قدیمی autostart از پنجره‌های پیکربندی.
#[tauri::command]
pub fn autostart_migrate(window: WebviewWindow) -> Result<bool, AppError> {
    window_policy::require_configuration(&window)?;
    migrate_native().map_err(AppError::Other)
}
