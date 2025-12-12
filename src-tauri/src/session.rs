//! Session Detection Module
//! Detects whether we're running on Wayland or X11 session
//! This is evaluated once at startup and cached for performance

use std::sync::OnceLock;

/// Cached session type - evaluated once at first access
static SESSION_TYPE: OnceLock<SessionType> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Wayland,
    X11,
    Unknown,
}

/// Detect the session type from environment variables
fn detect_session_type() -> SessionType {
    // Check XDG_SESSION_TYPE first (most reliable)
    if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
        match session_type.to_lowercase().as_str() {
            "wayland" => {
                eprintln!("[Session] Detected Wayland session via XDG_SESSION_TYPE");
                return SessionType::Wayland;
            }
            "x11" => {
                eprintln!("[Session] Detected X11 session via XDG_SESSION_TYPE");
                return SessionType::X11;
            }
            _ => {}
        }
    }

    // Fallback: Check for WAYLAND_DISPLAY
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        eprintln!("[Session] Detected Wayland session via WAYLAND_DISPLAY");
        return SessionType::Wayland;
    }

    // Fallback: Check for DISPLAY (X11)
    if std::env::var("DISPLAY").is_ok() {
        eprintln!("[Session] Detected X11 session via DISPLAY");
        return SessionType::X11;
    }

    eprintln!("[Session] Could not detect session type");
    SessionType::Unknown
}

/// Get the cached session type (detected once on first call)
pub fn get_session_type() -> SessionType {
    *SESSION_TYPE.get_or_init(detect_session_type)
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

/// Initialize session detection (call this early in main to cache the value)
pub fn init() {
    let session = get_session_type();
    eprintln!("[Session] Initialized: {:?}", session);
}
