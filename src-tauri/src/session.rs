//! Session Detection Module
//! Detects whether we're running on Wayland or X11 session.
//! Evaluated lazily once and cached for performance.

use std::env;
use std::sync::OnceLock;

/// Cached session type singleton
static SESSION_TYPE: OnceLock<SessionType> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Wayland,
    X11,
    Unknown,
}

impl SessionType {
    /// heuristic detection of the current session
    fn detect() -> (Self, &'static str) {
        // 1. Check XDG_SESSION_TYPE (Most reliable source)
        if let Ok(val) = env::var("XDG_SESSION_TYPE") {
            match val.trim().to_lowercase().as_str() {
                "wayland" => return (Self::Wayland, "XDG_SESSION_TYPE"),
                "x11" => return (Self::X11, "XDG_SESSION_TYPE"),
                _ => {} // Continue to fallbacks for unknown values
            }
        }

        // 2. Check WAYLAND_DISPLAY (Standard Wayland indicator)
        if env::var_os("WAYLAND_DISPLAY").is_some() {
            return (Self::Wayland, "WAYLAND_DISPLAY");
        }

        // 3. Check DISPLAY (Standard X11 indicator)
        if env::var_os("DISPLAY").is_some() {
            return (Self::X11, "DISPLAY");
        }

        (Self::Unknown, "None")
    }
}

/// Get the cached session type.
/// Detects the session if it hasn't been initialized yet.
pub fn get_session_type() -> SessionType {
    *SESSION_TYPE.get_or_init(|| {
        let (session, source) = SessionType::detect();
        tracing::info!("[Session] Detected {:?} via {}", session, source);
        session
    })
}

/// Check if running on Wayland
#[inline]
pub fn is_wayland() -> bool {
    get_session_type() == SessionType::Wayland
}

/// Check if running on X11
#[inline]
pub fn is_x11() -> bool {
    get_session_type() == SessionType::X11
}

/// Explicitly initialize session detection.
/// Useful to ensure the log message appears early in the application startup.
pub fn init() {
    get_session_type();
}

/// Tauri command: return session info for the frontend privacy section.
/// فرمان Tauri: اطلاعات نشست را برای بخش حریم خصوصی فرانت‌اند برمی‌گرداند.
#[tauri::command]
pub fn get_session_info() -> crate::error::AppResult<SessionInfo> {
    let session = get_session_type();
    // Wayland compositors generally do not expose focused-app identity;
    // X11 can use WM_CLASS/title heuristics.
    let app_identity_available = session != SessionType::Wayland;
    Ok(SessionInfo {
        is_wayland: session == SessionType::Wayland,
        app_identity_available,
    })
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionInfo {
    pub is_wayland: bool,
    pub app_identity_available: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_cache() {
        // Ensure subsequent calls return the same value without re-evaluating
        let first = get_session_type();
        let second = get_session_type();
        assert_eq!(first, second);
    }
}
