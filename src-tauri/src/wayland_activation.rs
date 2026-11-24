//! Wayland focus activation (xdg-activation-v1)
//!
//! On Wayland a window may only take keyboard focus when it presents a valid
//! activation token. Compositors such as KWin hand that token to the process
//! launched by a global shortcut via `XDG_ACTIVATION_TOKEN`. When the shortcut
//! launches a secondary instance (or the wrapper forwards the request over
//! D-Bus), the token must be forwarded to the running instance, otherwise the
//! compositor's focus stealing prevention keeps the popup unfocused: it then
//! never receives a focus-out event and does not close when clicking outside.
//!
//! The launcher forwards the token as `--activation-token=<token>`; the running
//! instance stores it and hands it to GDK right before presenting the popup.

use parking_lot::Mutex;

/// Command line flag used to forward the activation token to the running instance.
pub const ACTIVATION_TOKEN_ARG: &str = "--activation-token=";

static PENDING_TOKEN: Mutex<Option<String>> = Mutex::new(None);

/// Extracts the activation token from an argv list, if present and non-empty.
pub fn token_from_args(args: &[String]) -> Option<String> {
    args.iter()
        .find_map(|arg| arg.strip_prefix(ACTIVATION_TOKEN_ARG))
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_string)
}

/// Stores the activation token for the next popup show (replaces any older token).
pub fn store_token(token: Option<String>) {
    if token.is_some() {
        *PENDING_TOKEN.lock() = token;
    }
}

/// Takes the pending activation token. Tokens are single-use.
pub fn take_token() -> Option<String> {
    PENDING_TOKEN.lock().take()
}

/// Hands the token to GDK so the next `gdk_window_focus` on this display
/// activates the window through xdg-activation instead of being ignored.
///
/// Must be called on the GTK main thread. No-op on non-Wayland displays.
#[cfg(target_os = "linux")]
pub fn apply_token_to_gdk(token: &str) -> bool {
    use gtk::gdk;
    use gtk::glib::translate::ToGlibPtr;
    use gtk::prelude::*;
    use std::ffi::CString;

    extern "C" {
        fn gdk_wayland_display_set_startup_notification_id(
            display: *mut gdk::ffi::GdkDisplay,
            startup_id: *const std::os::raw::c_char,
        );
    }

    let Some(display) = gdk::Display::default() else {
        return false;
    };
    if display.type_().name() != "GdkWaylandDisplay" {
        return false;
    }
    let Ok(token) = CString::new(token) else {
        return false;
    };

    // SAFETY: `display` is a live GdkWaylandDisplay (type checked above) and
    // `token` is a valid NUL-terminated string that GDK copies.
    unsafe {
        gdk_wayland_display_set_startup_notification_id(display.to_glib_none().0, token.as_ptr());
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn extracts_token() {
        let a = args(&["win11-clipboard-history", "--activation-token=kwin-42"]);
        assert_eq!(token_from_args(&a).as_deref(), Some("kwin-42"));
    }

    #[test]
    fn ignores_missing_or_empty_token() {
        assert_eq!(token_from_args(&args(&["win11-clipboard-history"])), None);
        assert_eq!(
            token_from_args(&args(&["win11-clipboard-history", "--activation-token="])),
            None
        );
    }

    #[test]
    fn keeps_other_flags_working() {
        let a = args(&["x", "--emoji", "--activation-token=abc"]);
        assert!(a.iter().any(|s| s == "--emoji"));
        assert_eq!(token_from_args(&a).as_deref(), Some("abc"));
    }

    #[test]
    fn token_is_single_use() {
        store_token(Some("t1".into()));
        store_token(None); // must not clear a pending token
        assert_eq!(take_token().as_deref(), Some("t1"));
        assert_eq!(take_token(), None);
    }
}
