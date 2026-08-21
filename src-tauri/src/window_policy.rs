//! Central deny-by-default authorization for Tauri command callers.
//! مجوزدهی متمرکز و «رد به‌صورت پیش‌فرض» برای فراخوان‌های فرمان Tauri.
//!
//! A webview compromise must not automatically grant access to commands owned
//! by another application window. Every state-changing command should call one
//! of the helpers in this module before touching files, settings, shortcuts,
//! the clipboard, or privileged system tools.
//! نفوذ به یک WebView نباید دسترسی فرمان‌های پنجره‌ای دیگر را فراهم کند. هر
//! فرمان تغییردهندهٔ وضعیت باید پیش از دسترسی به فایل، تنظیمات، میانبر،
//! کلیپ‌بورد یا ابزار سطح‌بالای سیستم از helperهای این ماژول استفاده کند.

use tauri::WebviewWindow;

use crate::error::AppError;

/// Trusted application-window identities.
/// هویت‌های مورد اعتماد پنجره‌های برنامه.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowRole {
    Main,
    Settings,
    Setup,
}

impl WindowRole {
    fn label(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::Settings => "settings",
            Self::Setup => "setup",
        }
    }
}

/// Require a command to originate from one of `allowed` window roles.
/// الزام می‌کند فرمان از یکی از نقش‌های مجاز `allowed` فراخوانی شده باشد.
fn is_authorized(label: &str, allowed: &[WindowRole]) -> bool {
    allowed.iter().any(|role| role.label() == label)
}

pub fn require(window: &WebviewWindow, allowed: &[WindowRole]) -> Result<(), AppError> {
    if is_authorized(window.label(), allowed) {
        return Ok(());
    }

    tracing::warn!(
        target: "security",
        command_caller = window.label(),
        allowed = ?allowed,
        "Rejected a Tauri command from an unauthorized window"
    );
    Err(AppError::PermissionDenied(format!(
        "window '{}' is not authorized for this action",
        window.label()
    )))
}

/// Require the main clipboard window.
/// الزام پنجرهٔ اصلی کلیپ‌بورد.
pub fn require_main(window: &WebviewWindow) -> Result<(), AppError> {
    require(window, &[WindowRole::Main])
}

/// Require the settings window.
/// الزام پنجرهٔ تنظیمات.
pub fn require_settings(window: &WebviewWindow) -> Result<(), AppError> {
    require(window, &[WindowRole::Settings])
}

/// Require the first-run setup window.
/// الزام پنجرهٔ راه‌اندازی اولیه.
pub fn require_setup(window: &WebviewWindow) -> Result<(), AppError> {
    require(window, &[WindowRole::Setup])
}

/// Permit settings and first-run setup flows.
/// مجازکردن جریان تنظیمات و راه‌اندازی اولیه.
pub fn require_configuration(window: &WebviewWindow) -> Result<(), AppError> {
    require(window, &[WindowRole::Settings, WindowRole::Setup])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorization_is_deny_by_default() {
        assert!(is_authorized("main", &[WindowRole::Main]));
        assert!(is_authorized(
            "setup",
            &[WindowRole::Settings, WindowRole::Setup]
        ));
        assert!(!is_authorized("main", &[WindowRole::Settings]));
        assert!(!is_authorized("unknown", &[]));
    }
}
