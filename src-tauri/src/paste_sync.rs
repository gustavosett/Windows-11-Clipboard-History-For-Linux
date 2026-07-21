//! Paste Synchronization Module
//!
//! Adaptive X11 barriers for the paste pipeline. Each function polls an
//! observable server condition and returns as soon as it is confirmed. The
//! caller chooses a timeout as a failure ceiling, so constrained systems get
//! more time without imposing that delay on fast systems.
//!
//! These helpers return whether the condition was actually confirmed. Paste
//! code must treat `false` as a failed precondition rather than continuing
//! with an unverified input sequence.

use crate::session;
use std::thread;
use std::time::{Duration, Instant};
use x11rb::protocol::xproto::ConnectionExt;

/// Polling interval for all waiters. Cheap X11 round-trips (~0.1ms each).
const POLL_INTERVAL: Duration = Duration::from_millis(3);

/// A single matching focus sample can occur before a pending WM transition
/// (such as our popup's UnmapNotify) is processed. Requiring stability closes
/// that transient-match race at a cost of one short poll on fast systems.
const FOCUS_STABLE_SAMPLES: u8 = 2;

/// Returns the current owner window of the CLIPBOARD selection,
/// or `None` when it cannot be determined (non-X11, connection failure).
/// `Some(0)` (x11rb::NONE) means "no owner".
pub fn clipboard_owner() -> Option<u32> {
    if !session::is_x11() {
        return None;
    }
    let (conn, _) = x11rb::connect(None).ok()?;
    let atom = conn
        .intern_atom(false, b"CLIPBOARD")
        .ok()?
        .reply()
        .ok()?
        .atom;
    let owner = conn.get_selection_owner(atom).ok()?.reply().ok()?.owner;
    Some(owner)
}

/// Wait (at most `budget`) for the X11 input focus to land on
/// `target_window`. Returns `true` if the focus was confirmed early.
///
/// Replaces a fixed focus sleep: instead of assuming the
/// window manager needs N ms to process `set_input_focus`, we watch the
/// actual focus and return the moment it settles. When verification is
/// impossible (non-X11, connection failure), sleeps the full budget.
pub fn settle_focus(target_window: u32, budget: Duration) -> bool {
    let start = Instant::now();
    let mut stable_samples = 0;
    let confirmed = poll_until(budget, || {
        if !session::is_x11() || target_window == 0 {
            return PollState::Unverifiable;
        }
        match focused_window() {
            Some(focus) if focus == target_window => {
                stable_samples += 1;
                if stable_samples >= FOCUS_STABLE_SAMPLES {
                    PollState::Confirmed
                } else {
                    PollState::Pending
                }
            }
            Some(_) => {
                stable_samples = 0;
                PollState::Pending
            }
            None => PollState::Unverifiable,
        }
    });

    if !confirmed {
        sleep_remaining(start, budget);
    }
    confirmed
}

/// Wait (at most `budget`) for the CLIPBOARD selection owner to *change*
/// from `owner_before` (to a non-NONE owner). Returns `true` if the handoff
/// was confirmed early.
///
/// Replaces the fixed "clipboard settle" sleep after spawning xclip/wl-copy:
/// the instant the X server reports the new selection owner, any target app
/// will read the new content — there is nothing further to wait for.
/// Checking for a *change* (not just presence) avoids a false positive when
/// the previous owner is still holding the selection.
pub fn settle_clipboard_handoff(owner_before: Option<u32>, budget: Duration) -> bool {
    let start = Instant::now();
    let confirmed = poll_until(budget, || {
        let before = match owner_before {
            Some(owner) => owner,
            None => return PollState::Unverifiable,
        };
        match clipboard_owner() {
            Some(owner) if owner != x11rb::NONE && owner != before => PollState::Confirmed,
            Some(_) => PollState::Pending,
            None => PollState::Unverifiable,
        }
    });

    if !confirmed {
        sleep_remaining(start, budget);
    }
    confirmed
}

/// Wait (at most `budget`) for the CLIPBOARD selection to have *some* owner.
/// Weaker check than [`settle_clipboard_handoff`], used as a final settle
/// confirmation right before the paste keystroke, after the write has
/// already been verified upstream.
pub fn settle_clipboard_owned(budget: Duration) -> bool {
    let start = Instant::now();
    let confirmed = poll_until(budget, || match clipboard_owner() {
        Some(owner) if owner != x11rb::NONE => PollState::Confirmed,
        Some(_) => PollState::Pending,
        None => PollState::Unverifiable,
    });

    if !confirmed {
        sleep_remaining(start, budget);
    }
    confirmed
}

// --- internals ---

enum PollState {
    Confirmed,
    Pending,
    Unverifiable,
}

/// Polls `check` every POLL_INTERVAL until it confirms, becomes
/// unverifiable, or `budget` elapses. Returns `true` only on confirmation.
fn poll_until(budget: Duration, mut check: impl FnMut() -> PollState) -> bool {
    let deadline = Instant::now() + budget;
    loop {
        match check() {
            PollState::Confirmed => return true,
            PollState::Unverifiable => return false,
            PollState::Pending => {}
        }
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(POLL_INTERVAL);
    }
}

/// Sleeps whatever is left of `budget`, so an unconfirmed settle takes
/// exactly as long as the old fixed sleep did — never longer.
fn sleep_remaining(start: Instant, budget: Duration) {
    let remaining = budget.saturating_sub(start.elapsed());
    if !remaining.is_zero() {
        thread::sleep(remaining);
    }
}

fn focused_window() -> Option<u32> {
    let (conn, _) = x11rb::connect(None).ok()?;
    let reply = conn.get_input_focus().ok()?.reply().ok()?;
    Some(reply.focus)
}
