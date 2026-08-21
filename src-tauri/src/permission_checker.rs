//! `/dev/uinput` permission checks and first-run state management.
//! بررسی مجوز `/dev/uinput` و مدیریت وضعیت اجرای نخست.

use crate::error::AppError;
use crate::window_policy;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::process::Command;
use tauri::WebviewWindow;

#[derive(serde::Serialize, Clone)]
pub struct PermissionStatus {
    pub uinput_accessible: bool,
    pub uinput_path: String,
    pub user_in_input_group: bool,
    /// Stable UI translation key; never expose a localized backend string.
    /// کلید پایدار ترجمهٔ رابط؛ متن محلی‌سازی‌شده از backend ارسال نمی‌شود.
    pub status_code: String,
}

/// Return the setup-state directory under the XDG config home.
/// بازگرداندن پوشهٔ وضعیت راه‌اندازی در مسیر تنظیمات XDG.
fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string())).join(".config")
        })
        .join("windows-11-style-clipboard-history-manager")
}

/// Return the first-run marker path.
/// بازگرداندن مسیر نشانگر اجرای نخست.
fn get_config_path() -> PathBuf {
    get_config_dir().join("setup.json")
}

/// Read permission state without exposing a Tauri command boundary.
/// خواندن وضعیت مجوز بدون در معرض قراردادن مرز فرمان Tauri.
fn permission_status() -> PermissionStatus {
    let uinput_path = "/dev/uinput";
    let uinput_accessible = OpenOptions::new().write(true).open(uinput_path).is_ok();
    let user_in_input_group = Command::new("groups")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).contains("input"))
        .unwrap_or(false);

    let status_code = if uinput_accessible {
        "permissions_ok"
    } else if user_in_input_group {
        "relogin_required"
    } else {
        "permissions_missing"
    };

    PermissionStatus {
        uinput_accessible,
        uinput_path: uinput_path.to_string(),
        user_in_input_group,
        status_code: status_code.to_string(),
    }
}

/// Verify `/dev/uinput` access for setup/settings callers only.
/// بررسی دسترسی `/dev/uinput` فقط برای فراخوان تنظیمات/راه‌اندازی.
#[tauri::command]
pub fn check_permissions(window: WebviewWindow) -> Result<PermissionStatus, AppError> {
    window_policy::require_configuration(&window)?;
    Ok(permission_status())
}

/// Test whether a trusted executable is available through `PATH`.
/// بررسی وجود ابزار مورد اعتماد در `PATH`.
///
/// Delegates to the in-process `PATH` probe — no `which` subprocess.
/// به بررسی درون‌فرآیندی PATH واگذار می‌شود — بدون subprocess ی `which`.
fn command_exists(command: &str) -> bool {
    crate::exec_lookup::command_exists(command)
}

/// Apply a user-specific ACL through Polkit after explicit UI action.
/// اعمال ACL مختص کاربر از طریق Polkit پس از اقدام صریح رابط.
#[tauri::command]
pub fn fix_permissions_now(window: WebviewWindow) -> Result<String, AppError> {
    window_policy::require_configuration(&window)?;

    if !command_exists("pkexec") {
        return Err(AppError::PermissionDenied("pkexec_missing".into()));
    }
    if !command_exists("setfacl") {
        return Err(AppError::PermissionDenied("setfacl_missing".into()));
    }

    let username =
        whoami::username().map_err(|error| AppError::Other(format!("username_lookup:{error}")))?;
    let acl = format!("u:{username}:rw");
    let status = Command::new("pkexec")
        .args(["setfacl", "-m", &acl, "/dev/uinput"])
        .status()
        .map_err(|error| AppError::Other(format!("pkexec_launch:{error}")))?;

    if status.success() {
        Ok("permission_granted".to_string())
    } else {
        Err(AppError::PermissionDenied("permission_fix_failed".into()))
    }
}

/// Internal first-run query used by native startup code.
/// پرس‌وجوی داخلی اجرای نخست برای کد native آغاز برنامه.
pub fn first_run_pending() -> bool {
    !get_config_path().exists()
}

/// Query first-run state from the main or setup window.
/// پرس‌وجوی وضعیت اجرای نخست از پنجرهٔ اصلی یا راه‌اندازی.
#[tauri::command]
pub fn is_first_run(window: WebviewWindow) -> Result<bool, AppError> {
    window_policy::require(
        &window,
        &[
            window_policy::WindowRole::Main,
            window_policy::WindowRole::Setup,
        ],
    )?;
    Ok(first_run_pending())
}

/// Persist the first-run completion marker (native/internal API).
/// ذخیرهٔ نشانگر تکمیل اجرای نخست (API داخلی/native).
pub fn complete_first_run() -> Result<(), AppError> {
    let config_path = get_config_path();
    crate::fs_atomic::ensure_parent(&config_path)?;

    let marker = serde_json::json!({
        "setupComplete": true,
        "setupDate": chrono::Utc::now().to_rfc3339()
    });
    crate::fs_atomic::write_json_atomic(&config_path, &marker)?;
    Ok(())
}

/// Reset onboarding from the settings window only.
/// بازنشانی راه‌اندازی فقط از پنجرهٔ تنظیمات.
#[tauri::command]
pub fn reset_first_run(window: WebviewWindow) -> Result<(), AppError> {
    window_policy::require_settings(&window)?;
    let config_path = get_config_path();
    if config_path.exists() {
        std::fs::remove_file(&config_path)?;
    }
    Ok(())
}
