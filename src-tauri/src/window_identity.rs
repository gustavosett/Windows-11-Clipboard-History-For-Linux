//! Identify the currently focused application (X11 WM_CLASS / title).
//! Used by the privacy filter. Wayland compositors do not expose this.

use crate::session;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt};

#[derive(Debug, Clone, Default)]
pub struct SourceApp {
    pub class: String,
    pub title: String,
}

/// Best-effort identity of the focused window. `None` on Wayland or on error.
pub fn focused_source() -> Option<SourceApp> {
    if !session::is_x11() {
        return None;
    }
    let window = crate::paste_sync::focused_window()?;
    if window == 0 {
        return None;
    }
    read_identity(window).ok()
}

fn read_identity(window: u32) -> Result<SourceApp, String> {
    let (conn, _) = x11rb::connect(None).map_err(|e| e.to_string())?;

    let class = read_wm_class(&conn, window).unwrap_or_default();
    let title = read_title(&conn, window).unwrap_or_default();
    Ok(SourceApp { class, title })
}

fn read_wm_class<C: Connection>(conn: &C, window: u32) -> Option<String> {
    let reply = conn
        .get_property(false, window, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 256)
        .ok()?
        .reply()
        .ok()?;
    // WM_CLASS is instance\0class\0
    let raw = reply.value;
    let parts: Vec<&[u8]> = raw.split(|b| *b == 0).filter(|p| !p.is_empty()).collect();
    let joined = parts
        .iter()
        .filter_map(|p| std::str::from_utf8(p).ok())
        .collect::<Vec<_>>()
        .join(" ");
    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}

fn read_title<C: Connection>(conn: &C, window: u32) -> Option<String> {
    let net_wm_name = conn.intern_atom(false, b"_NET_WM_NAME").ok()?.reply().ok()?.atom;
    let utf8 = conn.intern_atom(false, b"UTF8_STRING").ok()?.reply().ok()?.atom;
    if let Ok(cookie) = conn.get_property(false, window, net_wm_name, utf8, 0, 256) {
        if let Ok(reply) = cookie.reply() {
            if let Ok(name) = String::from_utf8(reply.value) {
                let name = name.trim_matches('\0').trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
    }
    let reply = conn
        .get_property(false, window, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 256)
        .ok()?
        .reply()
        .ok()?;
    String::from_utf8(reply.value)
        .ok()
        .map(|s| s.trim_matches('\0').trim().to_string())
        .filter(|s| !s.is_empty())
}
