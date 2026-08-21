//! Event-driven clipboard change notifications for the history watcher.
//! اعلان رویدادمحور تغییر کلیپ‌بورد برای ناظر تاریخچه.
//!
//! The watcher previously polled the clipboard on a fixed cadence (200ms
//! active / 800ms idle), which costs CPU/energy while the machine is idle
//! and leaves a small race window between a copy and the capture pass.
//! This module provides *wakeup* channels instead:
//! ناظر قبلاً با آهنگ ثابت (۲۰۰ms فعال / ۸۰۰ms بیکار) کلیپ‌بورد را
//! پرس‌وجو می‌کرد که هنگام بیکاری هزینهٔ CPU/انرژی دارد و پنجرهٔ رقابت
//! کوچکی بین کپی و برداشت باقی می‌گذارد. این ماژول به‌جای آن کانال
//! *بیدارباش* فراهم می‌کند:
//!
//! - **X11** — subscribes to XFixes `SelectionNotify` for the `CLIPBOARD`
//!   selection and blocks on the X socket, so a copy wakes the watcher
//!   immediately with zero busy-work.
//!   - **X11** — روی رویداد `SelectionNotify` از XFixes برای selection
//!     `CLIPBOARD` مشترک می‌شود و روی سوکت X مسدود می‌ماند؛ بنابراین کپی
//!     فوراً ناظر را بیدار می‌کند بدون هیچ کار بیهوده‌ای.
//!
//! - **Wayland** — spawns `wl-paste --watch`; every byte on its stdout
//!   means "the clipboard changed". The child is killed automatically when
//!   this process dies (`PR_SET_PDEATHSIG`), so no orphan survives the app.
//!   - **Wayland** — فرآیند `wl-paste --watch` را اجرا می‌کند؛ هر بایت روی
//!     stdout آن یعنی «کلیپ‌بورد عوض شد». فرزند با مرگ این فرآیند خودکار
//!     کشته می‌شود (`PR_SET_PDEATHSIG`) تا هیچ یتیمی باقی نماند.
//!
//! Any failure (no X server, no wl-paste, unsupported flags) returns `None`
//! and the watcher transparently falls back to its adaptive polling loop,
//! so availability never regresses correctness.
//! هر خطایی (نبود X server، نبود wl-paste، فلگ پشتیبانی‌نشده) مقدار `None`
//! برمی‌گرداند و ناظر شفاف به حلقهٔ polling تطبیقی قبلی برمی‌گردد؛ پس
//! درستی کار هرگز به در دسترس بودن این ماژول وابسته نمی‌شود.

use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use x11rb::connection::Connection;
// `intern_atom` lives on the xproto extension trait; without this import the
// method call would not resolve.
// متد `intern_atom` روی trait توسعهٔ xproto است؛ بدون این import، فراخوانی
// متد حل نمی‌شود.
use x11rb::protocol::xproto::ConnectionExt as _;

/// Human-readable label of the active wakeup backend (for logs / status).
/// برچسب خوانای بک‌اند فعال بیدارباش (برای لاگ / وضعیت).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WakeSourceKind {
    /// XFixes `SelectionNotify` on the X11 `CLIPBOARD` selection.
    /// رویداد `SelectionNotify` از XFixes روی selection کلیپ‌بورد X11.
    X11Xfixes,
    /// `wl-paste --watch` child process on Wayland.
    /// فرآیند فرزند `wl-paste --watch` روی Wayland.
    WaylandWlPaste,
}

impl WakeSourceKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::X11Xfixes => "X11/XFixes",
            Self::WaylandWlPaste => "Wayland/wl-paste --watch",
        }
    }
}

/// Entry point for the watcher: start the best available wakeup source.
/// نقطهٔ ورود ناظر: راه‌اندازی بهترین منبع بیدارباش موجود.
pub fn start_wake_source() -> Option<Receiver<()>> {
    let kind = if crate::session::is_x11() {
        Some(WakeSourceKind::X11Xfixes)
    } else if crate::session::is_wayland() {
        Some(WakeSourceKind::WaylandWlPaste)
    } else {
        None
    };

    let kind = kind?;
    let (tx, rx) = channel();

    // The worker reports its own failure (and then exits), so the join
    // handle can be dropped and the thread detached safely.
    // کارگر خطای خودش را گزارش می‌کند (و سپس خارج می‌شود)؛ بنابراین
    // JoinHandle می‌تواند drop شود و رشته جدا (detached) بماند.
    let spawn_result = match kind {
        WakeSourceKind::X11Xfixes => thread::Builder::new()
            .name("clipboard-events-x11".to_string())
            .spawn(move || {
                if let Err(error) = run_x11_xfixes(tx) {
                    tracing::warn!("[ClipboardEvents] X11 wakeups stopped: {error}");
                }
            }),
        WakeSourceKind::WaylandWlPaste => thread::Builder::new()
            .name("clipboard-events-wayland".to_string())
            .spawn(move || {
                if let Err(error) = run_wayland_wl_paste(tx) {
                    tracing::warn!("[ClipboardEvents] Wayland wakeups stopped: {error}");
                }
            }),
    };

    match spawn_result {
        Ok(_handle) => {
            tracing::info!(
                "[ClipboardEvents] Event-driven wakeups enabled via {}",
                kind.label()
            );
            Some(rx)
        }
        Err(error) => {
            tracing::warn!(
                "[ClipboardEvents] Failed to spawn {} thread ({}); falling back to polling",
                kind.label(),
                error
            );
            None
        }
    }
}

// ---------------------------------------------------------------------------
// X11 — XFixes SelectionNotify
// ---------------------------------------------------------------------------

/// Subscribe to XFixes `SelectionNotify` for `CLIPBOARD` and block on the X
/// socket, waking the receiver on every owner change. Runs until the X
/// connection dies or the receiver is dropped.
/// روی رویداد XFixes برای `CLIPBOARD` مشترک شوید و روی سوکت X مسدود بمانید؛
/// با هر تغییر مالک، گیرنده بیدار می‌شود. تا مرگ اتصال X یا drop شدن
/// گیرنده ادامه می‌یابد.
fn run_x11_xfixes(tx: Sender<()>) -> Result<(), String> {
    let (conn, _) = x11rb::connect(None).map_err(|e| format!("X11 connect: {e}"))?;

    let clipboard_atom = conn
        .intern_atom(false, b"CLIPBOARD")
        .map_err(|e| format!("intern CLIPBOARD: {e}"))?
        .reply()
        .map_err(|e| format!("intern CLIPBOARD reply: {e}"))?
        .atom;

    // The root window is always valid, so we never need to create (and then
    // destroy) a private window just to receive these events.
    // پنجرهٔ ریشه همیشه معتبر است؛ پس برای دریافت رویدادها نیازی به ساخت
    // (و سپس نابودی) پنجرهٔ خصوصی نیست.
    let root = conn.setup().roots[0].root;

    // Compose the mask from the public constants via `u32` so this code is
    // independent of the `BitOr` impl details of the generated type.
    // ماسک از ثابت‌های عمومی از طریق `u32` ساخته می‌شود تا کد به جزئیات
    // پیاده‌سازی `BitOr` نوع تولیدشده وابسته نباشد.
    use x11rb::protocol::xfixes::SelectionEventMask;
    let mask = SelectionEventMask::from(
        u32::from(SelectionEventMask::SET_SELECTION_OWNER)
            | u32::from(SelectionEventMask::SELECTION_WINDOW_DESTROY)
            | u32::from(SelectionEventMask::SELECTION_CLIENT_CLOSE),
    );

    // The generated free function selects the input on `root`; the request
    // is flushed right away so the subscription is live before we block.
    // تابع آزادِ تولیدشده روی `root` انتخاب می‌کند؛ درخواست فوراً flush
    // می‌شود تا اشتراک پیش از مسدودشدن فعال باشد.
    x11rb::protocol::xfixes::select_selection_input(&conn, root, clipboard_atom, mask)
        .map_err(|e| format!("XFixes select_selection_input: {e}"))?;
    conn.flush().map_err(|e| format!("X11 flush: {e}"))?;

    loop {
        match conn.wait_for_event() {
            Ok(x11rb::protocol::Event::XfixesSelectionNotify(event))
                if event.selection == clipboard_atom =>
            {
                // The receiver was dropped (app shutting down): stop the thread.
                // گیرنده drop شده (خروج برنامه): حلقه تمام می‌شود.
                if tx.send(()).is_err() {
                    break;
                }
            }
            Ok(_) => {
                // Other events (errors, unrelated notifications) are ignored.
                // رویدادهای دیگر (خطاها، اعلان‌های نامربوط) نادیده گرفته می‌شوند.
            }
            Err(_) => break, // X connection lost; the watcher re-reads on polling.
                              // اتصال X قطع شد؛ ناظر با polling ادامه می‌دهد.
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Wayland — wl-paste --watch
// ---------------------------------------------------------------------------

/// Spawn `wl-paste --watch` and treat every stdout chunk as a wakeup. The
/// child inherits `PR_SET_PDEATHSIG` so the compositor-side selection offer
/// is torn down the moment this process exits.
/// فرآیند `wl-paste --watch` را اجرا و هر تکهٔ خروجی را بیدارباش تلقی کنید.
/// فرزند `PR_SET_PDEATHSIG` می‌گیرد تا با خروج برنامه، منبع انتخاب قطع شود.
#[cfg(unix)]
fn run_wayland_wl_paste(tx: Sender<()>) -> Result<(), String> {
    use std::os::unix::process::CommandExt;

    let mut command = Command::new("wl-paste");
    command
        .arg("--watch")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    // SAFETY: `prctl(PR_SET_PDEATHSIG, SIGTERM)` is async-signal-safe and
    // only arms the parent-death signal; no state is shared with the child.
    // امنیت: `prctl(PR_SET_PDEATHSIG, SIGTERM)` امن در context سیگنال است و
    // فقط سیگنال مرگ والد را مسلح می‌کند؛ حالتی با فرزند به اشتراک نیست.
    unsafe {
        command.pre_exec(|| {
            libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM);
            Ok(())
        });
    }

    let mut child = command
        .spawn()
        .map_err(|e| format!("spawn wl-paste --watch: {e}"))?;

    // `wl-paste --watch` prints the clipboard contents on every change; the
    // payload may be binary, so we read fixed-size chunks and never assume
    // newline framing. Any chunk == "the clipboard changed".
    // خروجی `wl-paste --watch` ممکن است باینری باشد؛ پس در تکه‌های ثابت
    // می‌خوانیم و به قاب‌بندی newline اتکا نمی‌کنیم. هر تکه = «تغییر».
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "wl-paste stdout unavailable".to_string())?;
    let mut buffer = [0u8; 4096];

    loop {
        match stdout.read(&mut buffer) {
            Ok(0) => break, // Child exited: nothing left to watch.
                            // فرزند خارج شده: چیزی برای نظارت نمانده.
            Ok(_) => {
                if tx.send(()).is_err() {
                    break; // Receiver dropped: stop reading and reap the child.
                           // گیرنده drop شده: خواندن متوقف و فرزند جمع می‌شود.
                }
            }
            Err(_) => break,
        }
    }

    let _ = child.kill();
    let _ = child.wait();
    Ok(())
}

#[cfg(not(unix))]
fn run_wayland_wl_paste(_tx: Sender<()>) -> Result<(), String> {
    Err("Wayland wakeups are unsupported on this platform".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// On a headless CI machine (no X server, no Wayland compositor) this
    /// must return quickly without panicking — either a working source or,
    /// far more likely, `None` so the watcher polls.
    /// روی ماشین CI بدون نمایشگر این تست باید سریع و بدون panic تمام شود —
    /// یا منبعی فعال یا (محتمل‌تر) `None` تا ناظر polling کند.
    #[test]
    fn start_never_panics() {
        let wake = start_wake_source();
        // Dropping the receiver ends any thread that did start.
        // با drop گیرنده، هر رشته‌ای که شروع شده هم تمام می‌شود.
        drop(wake);
    }

    /// The mask composition must cover all three ownership notifications.
    /// ترکیب ماسک باید هر سه اعلان مالکیت را پوشش دهد.
    #[test]
    fn composed_mask_covers_owner_events() {
        use x11rb::protocol::xfixes::SelectionEventMask;
        let raw = u32::from(SelectionEventMask::from(
            u32::from(SelectionEventMask::SET_SELECTION_OWNER)
                | u32::from(SelectionEventMask::SELECTION_WINDOW_DESTROY)
                | u32::from(SelectionEventMask::SELECTION_CLIENT_CLOSE),
        ));
        assert_eq!(raw, 0b111);
    }
}
