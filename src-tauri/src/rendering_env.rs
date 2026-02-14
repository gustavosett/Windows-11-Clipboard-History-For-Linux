//! Rendering Environment Detection Module
//!
//! Centralised detection of environments where transparency and rounded corners
//! must be disabled to avoid opacity rendering glitches (NVIDIA GPUs, AppImage builds).
//!
//! Detection is done **programmatically** so it works even when the wrapper
//! script is not in the execution path (e.g. AppImage launches the binary
//! directly).  The wrapper may *also* set `IS_NVIDIA` / `IS_APPIMAGE` — those
//! are respected as overrides.
//!
//! **IMPORTANT**: [`init()`] must be called very early in `main()`, *before*
//! any Tauri / WebKit initialisation, because it sets
//! `WEBKIT_DISABLE_DMABUF_RENDERER=1` when needed.

use serde::Serialize;
use std::process::Command;
use std::sync::OnceLock;

/// Immutable snapshot of the rendering environment, computed once at startup.
#[derive(Debug, Clone, Serialize)]
pub struct RenderingEnv {
    /// `true` when an NVIDIA GPU is detected.
    pub is_nvidia: bool,
    /// `true` when the app is running from an AppImage.
    pub is_appimage: bool,
    /// `true` when **either** flag is set – the frontend uses this as a single
    /// gate to disable transparency & rounded corners.
    pub transparency_disabled: bool,
    /// Human-readable reason string shown in the Settings UI.
    /// Empty when transparency is supported.
    pub reason: String,
}

/// Singleton – computed once by [`init()`] and read thereafter.
static RENDERING_ENV: OnceLock<RenderingEnv> = OnceLock::new();

// ---------------------------------------------------------------------------
// Detection helpers
// ---------------------------------------------------------------------------

/// Detect NVIDIA GPU presence.
///
/// Order of checks:
/// 1. `IS_NVIDIA=1` env var (set by wrapper or user).
/// 2. `/proc/modules` contains a loaded `nvidia` kernel module.
/// 3. `lspci` output mentions an NVIDIA VGA controller.
fn detect_nvidia() -> bool {
    // 1. Explicit env override
    if std::env::var("IS_NVIDIA")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        return true;
    }

    // 2. Check loaded kernel modules (fast, no subprocess)
    if let Ok(modules) = std::fs::read_to_string("/proc/modules") {
        // Each line starts with the module name followed by a space.
        // The NVIDIA driver suite loads multiple modules: nvidia, nvidia_drm,
        // nvidia_modeset, nvidia_uvm — match any of them.
        for line in modules.lines() {
            if let Some(name) = line.split_whitespace().next() {
                if name.to_ascii_lowercase().starts_with("nvidia") {
                    return true;
                }
            }
        }
    }

    // 3. Fall back to lspci
    if let Ok(output) = Command::new("lspci").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.lines().any(|l| {
                l.to_ascii_lowercase().contains("vga") && l.to_ascii_lowercase().contains("nvidia")
            }) {
                return true;
            }
        }
    }

    false
}

/// Detect whether we are running inside an AppImage.
///
/// The AppImage runtime always sets the `APPIMAGE` env var pointing to the
/// `.AppImage` file path.  We also accept the explicit `IS_APPIMAGE=1`
/// override that the wrapper may set.
fn detect_appimage() -> bool {
    if std::env::var("IS_APPIMAGE")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        return true;
    }
    // The standard AppImage runtime sets $APPIMAGE
    std::env::var("APPIMAGE")
        .map(|v| !v.is_empty())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// **Must be called at the very start of `main()`** before Tauri / WebKit init.
///
/// Performs detection, caches the result, sets `WEBKIT_DISABLE_DMABUF_RENDERER`
/// if needed, and logs the outcome.
pub fn init() {
    let env = RENDERING_ENV.get_or_init(|| {
        let is_nvidia = detect_nvidia();
        let is_appimage = detect_appimage();
        let transparency_disabled = is_nvidia || is_appimage;

        let reason = if is_nvidia && is_appimage {
            "Transparency is not supported on NVIDIA GPUs running via AppImage.".to_string()
        } else if is_nvidia {
            "Transparency is not supported on NVIDIA GPUs due to rendering issues.".to_string()
        } else if is_appimage {
            "Transparency is not supported when running as an AppImage.".to_string()
        } else {
            String::new()
        };

        RenderingEnv {
            is_nvidia,
            is_appimage,
            transparency_disabled,
            reason,
        }
    });

    // Set the WebKit env var *before* any WebView is created.
    if env.transparency_disabled {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        println!(
            "[RenderingEnv] WEBKIT_DISABLE_DMABUF_RENDERER=1 (NVIDIA={}, AppImage={})",
            env.is_nvidia, env.is_appimage
        );
    } else {
        println!("[RenderingEnv] Transparency enabled (no NVIDIA/AppImage detected)");
    }
}

/// Return the cached rendering environment (panics if [`init()`] was not called).
pub fn get_rendering_env() -> &'static RenderingEnv {
    RENDERING_ENV
        .get()
        .expect("rendering_env::init() must be called before get_rendering_env()")
}

/// Tauri command – returns the rendering environment to the frontend.
#[tauri::command]
pub fn get_rendering_environment() -> RenderingEnv {
    get_rendering_env().clone()
}
