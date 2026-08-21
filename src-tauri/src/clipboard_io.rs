//! Unified clipboard I/O module.
//! Single entry point for all clipboard read/write operations.
//! Reuses clipboard connection to avoid X11 connection churn.

use arboard::Clipboard;
use std::cell::RefCell;
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::debug;

static LAST_WRITE_UNIX_MS: AtomicU64 = AtomicU64::new(0);

const HELPER_TIMEOUT: Duration = Duration::from_secs(3);
const HELPER_POLL: Duration = Duration::from_millis(2);

pub const MIME_URI_LIST: &str = "text/uri-list";

/// Unified payload type for clipboard write operations
pub enum Payload<'a> {
    Text(&'a str),
    Html { html: &'a str, plain: &'a str },
    Bytes { mime: &'a str, data: &'a [u8] },
    FileUri(&'a str),
}

impl Payload<'_> {
    fn as_bytes(&self) -> &[u8] {
        match self {
            Payload::Text(t) => t.as_bytes(),
            Payload::Html { html, .. } => html.as_bytes(),
            Payload::Bytes { data, .. } => data,
            Payload::FileUri(s) => s.as_bytes(),
        }
    }
}

#[derive(Debug)]
pub enum ClipError {
    Arboard(arboard::Error),
    External(String),
    VerificationFailed(String),
}

impl std::fmt::Display for ClipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipError::Arboard(e) => write!(f, "arboard: {e}"),
            ClipError::External(e) => write!(f, "external: {e}"),
            ClipError::VerificationFailed(e) => write!(f, "verify: {e}"),
        }
    }
}

impl std::error::Error for ClipError {}

impl From<arboard::Error> for ClipError {
    fn from(e: arboard::Error) -> Self {
        ClipError::Arboard(e)
    }
}

// ---------------------------------------------------------------------------
// Thread-local cached clipboard connection
// ---------------------------------------------------------------------------

thread_local! {
    static CLIPBOARD: RefCell<Option<Clipboard>> = const { RefCell::new(None) };
}

fn with_clipboard<F, T>(f: F) -> Result<T, arboard::Error>
where
    F: FnOnce(&mut Clipboard) -> Result<T, arboard::Error>,
{
    CLIPBOARD.with(|cell| {
        let mut borrow = cell.borrow_mut();
        if borrow.is_none() {
            *borrow = Some(Clipboard::new()?);
        }
        f(borrow.as_mut().unwrap())
    })
}

/// Read plain text (uses cached connection)
pub fn read_text() -> Result<String, ClipError> {
    with_clipboard(|c| c.get_text()).map_err(ClipError::Arboard)
}

/// Read HTML content
pub fn read_html() -> Option<String> {
    with_clipboard(|c| c.get().html()).ok()
}

/// Read image with caching
pub fn read_image() -> Result<Option<arboard::ImageData<'static>>, ClipError> {
    match with_clipboard(|c| c.get_image()) {
        Ok(img) => Ok(Some(arboard::ImageData {
            width: img.width,
            height: img.height,
            bytes: img.bytes.into_owned().into(),
        })),
        Err(arboard::Error::ContentNotAvailable) => Ok(None),
        Err(e) => Err(ClipError::Arboard(e)),
    }
}

fn stamp_write() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    LAST_WRITE_UNIX_MS.store(now, Ordering::SeqCst);
}

/// Record that the OS clipboard was just written (including via xclip/wl-copy).
pub fn notify_write() {
    stamp_write();
}

/// True when a clipboard write happened recently enough that Ctrl+V is intentional.
pub fn wrote_recently(window: Duration) -> bool {
    let last = LAST_WRITE_UNIX_MS.load(Ordering::SeqCst);
    if last == 0 {
        return false;
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(last);
    now.saturating_sub(last) <= window.as_millis() as u64
}

/// Write content to clipboard using best available method.
/// Tries external tool (xclip/wl-copy) first, falls back to arboard.
pub fn write(payload: &Payload<'_>) -> Result<(), ClipError> {
    #[cfg(target_os = "linux")]
    {
        if write_external(payload).is_ok() {
            stamp_write();
            return Ok(());
        }
        debug!("external clipboard tool failed, falling back to arboard");
    }

    let result = write_arboard(payload);
    if result.is_ok() {
        stamp_write();
    }
    result
}

// ---------------------------------------------------------------------------
// Linux external tools (xclip / wl-copy)
// ---------------------------------------------------------------------------

fn write_external(payload: &Payload<'_>) -> Result<(), ClipError> {
    let is_wayland = crate::session::is_wayland();
    let (cmd, args) = external_args(payload, is_wayland);

    debug!("{cmd} args: {args:?}");
    let mut child = Command::new(cmd)
        .args(args.into_iter())
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| ClipError::External(format!("spawn {cmd}: {e}")))?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin
            .write_all(payload.as_bytes())
            .map_err(|e| ClipError::External(format!("pipe write: {e}")))?;
    }

    if is_wayland {
        // wl-copy forks; wait for the forking parent to signal readiness
        wait_for_child(child, cmd)
    } else {
        // xclip stays alive; detach into background thread
        let _ = std::thread::spawn(move || {
            let _ = child.wait();
        });
        Ok(())
    }
}

fn external_args<'a>(payload: &Payload<'a>, wayland: bool) -> (&'a str, Vec<&'a str>) {
    match (wayland, payload) {
        (true, Payload::Text(_)) => ("wl-copy", vec!["--type", "text/plain;charset=utf-8"]),
        (true, Payload::Html { .. }) => ("wl-copy", vec!["--type", "text/html"]),
        (true, Payload::FileUri(_)) => ("wl-copy", vec!["--type", MIME_URI_LIST]),
        (true, Payload::Bytes { mime, .. }) => ("wl-copy", vec!["--type", mime]),
        (false, Payload::Text(_)) => ("xclip", vec!["-selection", "clipboard", "-t", "UTF8_STRING"]),
        (false, Payload::Html { .. }) => ("xclip", vec!["-selection", "clipboard", "-t", "text/html"]),
        (false, Payload::FileUri(_)) => {
            ("xclip", vec!["-selection", "clipboard", "-t", MIME_URI_LIST, "-loops", "0"])
        }
        (false, Payload::Bytes { mime, .. }) => {
            ("xclip", vec!["-selection", "clipboard", "-t", mime, "-loops", "0"])
        }
    }
}

fn wait_for_child(mut child: std::process::Child, cmd: &str) -> Result<(), ClipError> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(status)) => {
                let mut stderr = String::new();
                let _ = child.stderr.take().map(|mut p| p.read_to_string(&mut stderr));
                return Err(ClipError::External(format!("{cmd} exited {status}: {stderr}")));
            }
            Ok(None) if start.elapsed() < HELPER_TIMEOUT => {
                std::thread::sleep(HELPER_POLL);
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(ClipError::External(format!("{cmd} timed out")));
            }
            Err(e) => return Err(ClipError::External(format!("{cmd} wait: {e}"))),
        }
    }
}

// ---------------------------------------------------------------------------
// arboard fallback + verification
// ---------------------------------------------------------------------------

fn write_arboard(payload: &Payload<'_>) -> Result<(), ClipError> {
    with_clipboard(|c| {
        match payload {
            Payload::Text(t) => c.set_text(*t),
            Payload::Html { html, plain } => c.set_html(*html, Some(*plain)),
            Payload::FileUri(s) => c.set_text(*s),
            Payload::Bytes { data, .. } => {
                let s = std::str::from_utf8(data).unwrap_or_default();
                c.set_text(s)
            }
        }
    })
    .map_err(ClipError::Arboard)?;

    verify(payload)
}

fn verify(payload: &Payload<'_>) -> Result<(), ClipError> {
    let expected = payload.as_bytes();
    let observed = with_clipboard(|c| -> Result<Vec<u8>, arboard::Error> {
        Ok(c.get_text()?.into_bytes())
    })
    .map_err(|e| ClipError::VerificationFailed(format!("readback: {e}")))?;

    if observed != expected {
        return Err(ClipError::VerificationFailed("data mismatch".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payload_as_bytes() {
        let t = Payload::Text("hello");
        assert_eq!(t.as_bytes(), b"hello");

        let h = Payload::Html { html: "<b>hi</b>", plain: "hi" };
        assert_eq!(h.as_bytes(), b"<b>hi</b>");
    }

    #[test]
    fn test_payload_file_uri() {
        let p = Payload::FileUri("file:///tmp/test.gif\n");
        assert!(p.as_bytes().starts_with(b"file:///"));
    }
}
