//! Paste synchronization barriers for X11.
//!
//! A single cached X11 connection is used for focus and clipboard checks.
//! Besides avoiding connection setup on every paste, keeping the atom cache
//! and polling on one connection makes the hot path both cheaper and easier
//! to reason about.

use crate::session;
use parking_lot::Mutex;
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{Atom, ConnectionExt, InputFocus};
use x11rb::rust_connection::RustConnection;

/// Polling interval for all waiters. X11 round trips are local and cheap.
const POLL_INTERVAL: Duration = Duration::from_millis(3);

/// One matching focus sample can precede a pending WM transition. Requiring
/// two consecutive samples closes that transient-match race.
const FOCUS_STABLE_SAMPLES: u8 = 2;

struct X11Context {
    connection: RustConnection,
    clipboard_atom: Atom,
}

impl X11Context {
    fn connect() -> Result<Self, String> {
        let (connection, _) =
            x11rb::connect(None).map_err(|error| format!("X11 connection failed: {}", error))?;
        let clipboard_atom = connection
            .intern_atom(false, b"CLIPBOARD")
            .map_err(|error| format!("Failed to intern CLIPBOARD: {}", error))?
            .reply()
            .map_err(|error| format!("Failed to intern CLIPBOARD: {}", error))?
            .atom;

        Ok(Self {
            connection,
            clipboard_atom,
        })
    }

    fn clipboard_owner(&self) -> Result<u32, String> {
        self.connection
            .get_selection_owner(self.clipboard_atom)
            .map_err(|error| format!("Failed to query clipboard owner: {}", error))?
            .reply()
            .map_err(|error| format!("Failed to query clipboard owner: {}", error))
            .map(|reply| reply.owner)
    }

    fn focused_window(&self) -> Result<u32, String> {
        self.connection
            .get_input_focus()
            .map_err(|error| format!("Failed to query X11 focus: {}", error))?
            .reply()
            .map_err(|error| format!("Failed to query X11 focus: {}", error))
            .map(|reply| reply.focus)
    }
}

static X11_CONTEXT: OnceLock<Mutex<Option<X11Context>>> = OnceLock::new();

fn x11_context() -> &'static Mutex<Option<X11Context>> {
    X11_CONTEXT.get_or_init(|| Mutex::new(None))
}

fn with_x11_context<T>(
    operation: impl FnOnce(&mut X11Context) -> Result<T, String>,
) -> Result<T, String> {
    if !session::is_x11() {
        return Err("X11 synchronization requested outside an X11 session".to_string());
    }

    let mut slot = x11_context().lock();
    if slot.is_none() {
        *slot = Some(X11Context::connect()?);
    }

    let result = operation(slot.as_mut().expect("X11 context was initialized"));
    if result.is_err() {
        // A broken server connection should not poison every later paste.
        // The next operation reconnects lazily.
        *slot = None;
    }
    result
}

/// Warms the cached X11 synchronization connection outside the paste path.
pub fn init() {
    if !session::is_x11() {
        return;
    }

    if let Err(error) = thread::Builder::new()
        .name("x11-paste-sync-warmup".to_string())
        .spawn(|| match with_x11_context(|_| Ok(())) {
            Ok(()) => tracing::info!("[PasteSync] X11 synchronization connection is ready"),
            Err(error) => tracing::info!(
                "[PasteSync] X11 warmup failed; first paste will retry: {}",
                error
            ),
        })
    {
        tracing::info!("[PasteSync] Failed to start warmup thread: {}", error);
    }
}

/// Returns the current owner window of the CLIPBOARD selection.
/// `Some(0)` (`x11rb::NONE`) means that the selection has no owner.
pub fn clipboard_owner() -> Option<u32> {
    with_x11_context(|context| context.clipboard_owner()).ok()
}

/// Returns the window that currently has X11 input focus.
pub fn focused_window() -> Option<u32> {
    with_x11_context(|context| context.focused_window()).ok()
}

/// Requests focus for `target_window` and waits until it is observed in two
/// consecutive samples. The request and verification share one connection,
/// so request ordering is guaranteed without a fixed settle delay.
pub fn restore_and_settle_focus(target_window: u32, timeout: Duration) -> Result<bool, String> {
    if target_window == 0 {
        return Err("Cannot restore focus to X11 window 0".to_string());
    }

    with_x11_context(|context| {
        context
            .connection
            .set_input_focus(InputFocus::PARENT, target_window, x11rb::CURRENT_TIME)
            .map_err(|error| format!("Set focus failed: {}", error))?;
        context
            .connection
            .flush()
            .map_err(|error| format!("X11 focus flush failed: {}", error))?;

        let mut stable_samples = 0;
        poll_until(timeout, || {
            if context.focused_window()? == target_window {
                stable_samples += 1;
                Ok(stable_samples >= FOCUS_STABLE_SAMPLES)
            } else {
                stable_samples = 0;
                Ok(false)
            }
        })
    })
}

/// Waits for the CLIPBOARD owner to change to a non-empty owner.
pub fn settle_clipboard_handoff(owner_before: Option<u32>, timeout: Duration) -> bool {
    let Some(owner_before) = owner_before else {
        // The caller can immediately use its fallback instead of paying the
        // full timeout for a condition that cannot be verified.
        return false;
    };

    with_x11_context(|context| {
        poll_until(timeout, || {
            let owner = context.clipboard_owner()?;
            Ok(owner != x11rb::NONE && owner != owner_before)
        })
    })
    .unwrap_or(false)
}

fn poll_until(
    timeout: Duration,
    mut check: impl FnMut() -> Result<bool, String>,
) -> Result<bool, String> {
    let deadline = Instant::now() + timeout;
    loop {
        if check()? {
            return Ok(true);
        }
        if Instant::now() >= deadline {
            return Ok(false);
        }
        thread::sleep(POLL_INTERVAL);
    }
}
