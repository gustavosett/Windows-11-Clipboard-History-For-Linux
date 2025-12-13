//! Focus Manager Module
//! Tracks and restores window focus for proper paste injection on X11.

#[cfg(target_os = "linux")]
use std::sync::atomic::{AtomicU32, Ordering};
#[cfg(target_os = "linux")]
use std::thread;
#[cfg(target_os = "linux")]
use std::time::Duration;
#[cfg(target_os = "linux")]
use x11rb::connection::Connection;
#[cfg(target_os = "linux")]
use x11rb::protocol::xproto::{ConnectionExt, InputFocus};

/// Time to wait after restoring focus before allowing the paste to proceed
#[cfg(target_os = "linux")]
const FOCUS_RESTORE_DELAY: Duration = Duration::from_millis(100);

/// Stores the ID of the window that had focus before we opened
#[cfg(target_os = "linux")]
static LAST_FOCUSED_WINDOW: AtomicU32 = AtomicU32::new(0);

// --- Linux Implementation ---

#[cfg(target_os = "linux")]
pub fn save_focused_window() {
    match get_x11_connection() {
        Ok(conn) => match conn.get_input_focus() {
            Ok(cookie) => match cookie.reply() {
                Ok(reply) => {
                    let window_id = reply.focus;
                    LAST_FOCUSED_WINDOW.store(window_id, Ordering::SeqCst);
                    eprintln!("[FocusManager] Saved focused window: {}", window_id);
                }
                Err(e) => eprintln!("[FocusManager] Failed to get focus reply: {}", e),
            },
            Err(e) => eprintln!("[FocusManager] Failed to request input focus: {}", e),
        },
        Err(e) => eprintln!("[FocusManager] X11 Connection failed: {}", e),
    }
}

#[cfg(target_os = "linux")]
pub fn restore_focused_window() -> Result<(), String> {
    let window_id = LAST_FOCUSED_WINDOW.load(Ordering::SeqCst);

    if window_id == 0 {
        return Err("No previous window saved".to_string());
    }

    eprintln!("[FocusManager] Restoring focus to window: {}", window_id);

    let conn = get_x11_connection()?;

    conn.set_input_focus(InputFocus::PARENT, window_id, x11rb::CURRENT_TIME)
        .map_err(|e| format!("Set focus failed: {}", e))?;

    conn.flush().map_err(|e| format!("Flush failed: {}", e))?;

    // Small delay to ensure the Window Manager processes the focus change
    // before we attempt to simulate keystrokes
    thread::sleep(FOCUS_RESTORE_DELAY);

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn get_focused_window() -> Option<u32> {
    let conn = get_x11_connection().ok()?;

    // Split the chain to satisfy the borrow checker (fix for E0597)
    let cookie = conn.get_input_focus().ok()?;
    let reply = cookie.reply().ok()?;

    Some(reply.focus)
}

/// Helper to establish X11 connection
#[cfg(target_os = "linux")]
fn get_x11_connection() -> Result<impl Connection, String> {
    x11rb::connect(None)
        .map(|(conn, _)| conn)
        .map_err(|e| format!("X11 connect failed: {}", e))
}

// --- Non-Linux Fallbacks ---

#[cfg(not(target_os = "linux"))]
pub fn save_focused_window() {}

#[cfg(not(target_os = "linux"))]
pub fn restore_focused_window() -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn get_focused_window() -> Option<u32> {
    None
}
