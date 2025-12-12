//! Focus Manager Module
//! Tracks and restores window focus for proper paste injection on X11

#[cfg(target_os = "linux")]
use std::sync::atomic::{AtomicU32, Ordering};

#[cfg(target_os = "linux")]
use x11rb::connection::Connection;
#[cfg(target_os = "linux")]
use x11rb::protocol::xproto::{ConnectionExt, InputFocus};

/// Stores the previously focused window ID
#[cfg(target_os = "linux")]
static PREVIOUS_WINDOW: AtomicU32 = AtomicU32::new(0);

/// Save the currently focused window before showing our clipboard window
#[cfg(target_os = "linux")]
pub fn save_focused_window() {
    if let Ok((conn, _)) = x11rb::connect(None) {
        if let Ok(reply) = conn.get_input_focus() {
            if let Ok(focus) = reply.reply() {
                let window_id = focus.focus;
                PREVIOUS_WINDOW.store(window_id, Ordering::SeqCst);
                eprintln!("[FocusManager] Saved focused window: {}", window_id);
            }
        }
    }
}

/// Restore focus to the previously saved window
#[cfg(target_os = "linux")]
pub fn restore_focused_window() -> Result<(), String> {
    let window_id = PREVIOUS_WINDOW.load(Ordering::SeqCst);

    if window_id == 0 {
        eprintln!("[FocusManager] No previous window saved");
        return Err("No previous window saved".to_string());
    }

    eprintln!("[FocusManager] Restoring focus to window: {}", window_id);

    let (conn, _) = x11rb::connect(None).map_err(|e| format!("X11 connect failed: {}", e))?;

    conn.set_input_focus(InputFocus::PARENT, window_id, x11rb::CURRENT_TIME)
        .map_err(|e| format!("Set focus failed: {}", e))?;

    conn.flush().map_err(|e| format!("Flush failed: {}", e))?;

    // Small delay to ensure focus is set
    std::thread::sleep(std::time::Duration::from_millis(10));

    Ok(())
}

/// Get the currently focused window ID (for debugging)
#[cfg(target_os = "linux")]
pub fn get_focused_window() -> Option<u32> {
    if let Ok((conn, _)) = x11rb::connect(None) {
        if let Ok(reply) = conn.get_input_focus() {
            if let Ok(focus) = reply.reply() {
                return Some(focus.focus);
            }
        }
    }
    None
}

// Fallback implementations for non-Linux platforms
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
