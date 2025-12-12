//! GIF Manager
//! Handles downloading GIFs and preparing them for clipboard paste
//!
//! IMPORTANT: Even though the app runs with GDK_BACKEND=x11 for window positioning,
//! the target apps (Discord, Chrome, etc.) run as native Wayland apps.
//! Therefore, we MUST use wl-copy (Wayland clipboard) for GIF paste to work.
//! For X11 sessions, we fall back to xclip.

use crate::session;
use arboard::Clipboard;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Get the temp directory for storing downloaded GIFs
fn get_gif_cache_dir() -> Result<PathBuf, String> {
    let cache_dir = dirs::cache_dir()
        .ok_or("Failed to get cache directory")?
        .join("win11-clipboard-history")
        .join("gifs");

    fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache directory: {}", e))?;

    Ok(cache_dir)
}

/// Download a GIF from URL and save to a temp file
/// Returns the path to the downloaded GIF file
pub fn download_gif_to_file(url: &str) -> Result<PathBuf, String> {
    eprintln!("[GifManager] Downloading GIF from: {}", url);

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(url)
        .send()
        .map_err(|e| format!("Failed to download GIF: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let bytes = response
        .bytes()
        .map_err(|e| format!("Failed to read response: {}", e))?;

    eprintln!("[GifManager] Downloaded {} bytes", bytes.len());

    // Generate a unique filename based on URL hash
    let url_hash = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        hasher.finish()
    };

    let cache_dir = get_gif_cache_dir()?;
    let gif_path = cache_dir.join(format!("{}.gif", url_hash));

    let mut file =
        fs::File::create(&gif_path).map_err(|e| format!("Failed to create GIF file: {}", e))?;

    file.write_all(&bytes)
        .map_err(|e| format!("Failed to write GIF file: {}", e))?;

    eprintln!("[GifManager] Saved GIF to: {:?}", gif_path);

    Ok(gif_path)
}

/// Copy GIF to clipboard using wl-copy (Wayland) with text/uri-list format
fn copy_gif_to_clipboard_wayland(gif_path: &Path) -> Result<(), String> {
    eprintln!("[GifManager] Copying GIF using wl-copy (Wayland) with text/uri-list...");

    let wayland_display = std::env::var("WAYLAND_DISPLAY")
        .map_err(|_| "WAYLAND_DISPLAY not set; Wayland clipboard not available".to_string())?;

    let xdg_runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .map_err(|_| "XDG_RUNTIME_DIR not set; Wayland clipboard not available".to_string())?;

    eprintln!(
        "[GifManager] Using WAYLAND_DISPLAY={}, XDG_RUNTIME_DIR={}",
        wayland_display, xdg_runtime_dir
    );

    let file_uri = format!("file://{}\n", gif_path.to_string_lossy());

    // Use wl-copy to set clipboard
    // Note: wl-copy forks to background by default to serve paste requests
    // We write to stdin and then let it run in background
    let mut child = Command::new("wl-copy")
        .env("WAYLAND_DISPLAY", &wayland_display)
        .env("XDG_RUNTIME_DIR", &xdg_runtime_dir)
        .arg("--type")
        .arg("text/uri-list")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            format!("Failed to spawn wl-copy: {e}. Make sure wl-clipboard is installed.")
        })?;

    // Write to stdin and close it
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(file_uri.as_bytes())
            .map_err(|e| format!("Failed to write to wl-copy: {e}"))?;
        // stdin is dropped here, closing it
    }

    // Give wl-copy time to read stdin and set up the clipboard
    // We don't wait for it to finish because it stays running to serve paste requests
    std::thread::sleep(std::time::Duration::from_millis(150));

    // Check if the process is still running (good) or exited with error (bad)
    match child.try_wait() {
        Ok(Some(status)) => {
            if !status.success() {
                // Process exited with error
                if let Ok(output) = child.wait_with_output() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(format!("wl-copy failed: {stderr}"));
                }
                return Err("wl-copy failed with unknown error".to_string());
            }
            // Process exited successfully (unusual but ok)
        }
        Ok(None) => {
            // Process still running - this is expected, wl-copy stays alive to serve paste requests
            eprintln!("[GifManager] wl-copy running in background to serve paste requests");
        }
        Err(e) => {
            return Err(format!("Failed to check wl-copy status: {e}"));
        }
    }

    eprintln!("[GifManager] Successfully set Wayland clipboard to text/uri-list");
    Ok(())
}

/// Copy GIF to clipboard using xclip (X11) with text/uri-list format
fn copy_gif_to_clipboard_x11(gif_path: &Path) -> Result<(), String> {
    eprintln!("[GifManager] Copying GIF using xclip (X11) with text/uri-list...");

    let display = std::env::var("DISPLAY")
        .map_err(|_| "DISPLAY not set; X11 clipboard not available".to_string())?;

    let file_uri = format!("file://{}", gif_path.to_string_lossy());

    let mut child = Command::new("xclip")
        .env("DISPLAY", &display)
        .arg("-selection")
        .arg("clipboard")
        .arg("-t")
        .arg("text/uri-list")
        .arg("-loops")
        .arg("0")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn xclip: {e}. Make sure xclip is installed."))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(file_uri.as_bytes())
            .map_err(|e| format!("Failed to write to xclip: {e}"))?;
    }

    // Detach xclip process so it can serve clipboard requests
    std::thread::spawn(move || {
        let _ = child.wait();
    });

    eprintln!("[GifManager] xclip started with text/uri-list");
    Ok(())
}

/// Copy a URL to clipboard as fallback
pub fn copy_url_to_clipboard(url: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| format!("Failed to open clipboard: {}", e))?;

    clipboard
        .set_text(url)
        .map_err(|e| format!("Failed to set clipboard: {}", e))?;

    eprintln!("[GifManager] Set clipboard to URL (fallback): {}", url);

    Ok(())
}

/// Set clipboard from a local GIF file path
/// Tries Wayland first if detected, then falls back to X11
fn set_gif_clipboard_from_file(path: &Path, is_wayland: bool) -> Result<(), String> {
    if is_wayland {
        // Try Wayland first, fall back to X11 if it fails
        match copy_gif_to_clipboard_wayland(path) {
            Ok(()) => Ok(()),
            Err(wayland_err) => {
                eprintln!(
                    "[GifManager] Wayland clipboard failed ({wayland_err}), trying X11 fallback..."
                );
                copy_gif_to_clipboard_x11(path)
            }
        }
    } else {
        copy_gif_to_clipboard_x11(path)
    }
}

/// Main function: Download GIF and prepare for pasting
pub fn paste_gif_to_clipboard(url: &str) -> Result<(), String> {
    paste_gif_to_clipboard_with_uri(url).map(|_| ())
}

/// Main function: Download GIF and prepare for pasting
/// Returns the file URI that was set to clipboard (for marking as pasted)
pub fn paste_gif_to_clipboard_with_uri(url: &str) -> Result<Option<String>, String> {
    let is_wayland = session::is_wayland();
    eprintln!(
        "[GifManager] Session type: {}",
        if is_wayland { "Wayland" } else { "X11" }
    );

    // Try to download and set clipboard
    let result = download_gif_to_file(url).and_then(|gif_path| {
        let file_uri = format!("file://{}", gif_path.to_string_lossy());
        let res = set_gif_clipboard_from_file(&gif_path, is_wayland);
        if res.is_ok() {
            eprintln!("[GifManager] Successfully set clipboard to GIF");
        }
        res.map(|_| file_uri)
    });

    match result {
        Ok(uri) => Ok(Some(uri)),
        Err(e) => {
            eprintln!("[GifManager] GIF clipboard failed ({e}), falling back to URL");
            copy_url_to_clipboard(url)?;
            // URL fallback - mark the URL as well
            Ok(Some(url.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_gif_cache_dir() {
        let result = get_gif_cache_dir();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.ends_with("win11-clipboard-history/gifs"));
    }

    #[test]
    fn test_session_detection() {
        // This test just ensures the function doesn't panic
        let _ = session::is_wayland();
    }
}
