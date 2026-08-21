//! GIF Manager
//! Downloads GIFs (allowlisted HTTPS hosts only) and prepares them for clipboard paste.

use crate::clipboard_io::{self, Payload};
use crate::ssrf;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{debug, info, warn};

const APP_CACHE_DIR: &str = "windows-11-style-clipboard-history-manager/gifs";
const DOWNLOAD_TIMEOUT: u64 = 10;

struct GifCache;

impl GifCache {
    fn get_dir() -> Result<PathBuf, String> {
        let cache_dir = dirs::cache_dir()
            .ok_or("Failed to resolve system cache directory")?
            .join(APP_CACHE_DIR);

        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)
                .map_err(|e| format!("Failed to create cache dir: {}", e))?;
            crate::fs_atomic::restrict_permissions(&cache_dir);
        }

        Ok(cache_dir)
    }

    fn get_path_for_url(url: &str) -> Result<PathBuf, String> {
        let hash = crate::clipboard_manager::calculate_hash(&url);
        Ok(Self::get_dir()?.join(format!("{:016x}.gif", hash)))
    }
}

struct Downloader;

impl Downloader {
    const MAX_GIF_BYTES: u64 = 10 * 1024 * 1024;

    pub fn download(url: &str, destination: &Path) -> Result<(), String> {
        let validated = ssrf::validate_and_pin(url)?;

        const CACHE_TTL: Duration = Duration::from_secs(24 * 3600);
        if let Ok(meta) = fs::metadata(destination) {
            if meta.len() > 0 {
                if let Ok(modified) = meta.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        if elapsed < CACHE_TTL {
                            debug!("[GifManager] Cache hit for {url}");
                            return Ok(());
                        }
                    }
                }
            }
        }

        debug!("[GifManager] Downloading: {url}");
        let client = ssrf::pinned_blocking_client(
            &validated,
            Duration::from_secs(DOWNLOAD_TIMEOUT),
        )?;

        let response = client
            .get(validated.url.clone())
            .send()
            .map_err(|e| format!("Network request failed: {e}"))?;

        if response.status().is_redirection() {
            return Err("Refusing to follow HTTP redirects for GIF downloads".into());
        }
        if !response.status().is_success() {
            return Err(format!("HTTP Error: {}", response.status()));
        }

        if let Some(ct) = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
        {
            let ct = ct.to_ascii_lowercase();
            if !ct.contains("gif") && !ct.contains("octet-stream") && !ct.contains("image/") {
                return Err(format!("Unexpected content type: {ct}"));
            }
        }

        let mut file =
            fs::File::create(destination).map_err(|e| format!("File creation failed: {e}"))?;

        let mut downloaded: u64 = 0;
        let mut reader = response.take(Self::MAX_GIF_BYTES + 1);
        let mut buffer = [0u8; 4096];

        loop {
            let n = reader
                .read(&mut buffer)
                .map_err(|e| format!("Read error: {e}"))?;
            if n == 0 {
                break;
            }
            downloaded += n as u64;
            if downloaded > Self::MAX_GIF_BYTES {
                let _ = fs::remove_file(destination);
                return Err(format!(
                    "GIF download exceeds {} MB limit",
                    Self::MAX_GIF_BYTES / (1024 * 1024)
                ));
            }
            file.write_all(&buffer[..n])
                .map_err(|e| format!("File write failed: {e}"))?;
        }

        crate::fs_atomic::restrict_permissions(destination);
        info!("[GifManager] Saved {downloaded} bytes to {:?}", destination);
        Ok(())
    }
}

struct ClipboardHandler;

impl ClipboardHandler {
    fn make_file_uri(path: &Path) -> String {
        format!("file://{}\n", path.to_string_lossy())
    }

    fn copy_uri(path: &Path) -> Result<(), String> {
        let uri = Self::make_file_uri(path);
        clipboard_io::write(&Payload::FileUri(&uri))
            .map_err(|e| format!("GIF clipboard copy failed: {e}"))
    }
}

pub fn download_gif_to_file(url: &str) -> Result<PathBuf, String> {
    let target_path = GifCache::get_path_for_url(url)?;
    Downloader::download(url, &target_path)?;
    Ok(target_path)
}

pub fn paste_gif_to_clipboard_with_uri(url: &str) -> Result<Option<String>, String> {
    debug!("[GifManager] paste_gif_to_clipboard_with_uri: {url}");

    let gif_path = download_gif_to_file(url)?;

    match ClipboardHandler::copy_uri(&gif_path) {
        Ok(_) => {
            let uri = format!("file://{}", gif_path.to_string_lossy());
            info!("[GifManager] GIF ready: {uri}");
            Ok(Some(uri))
        }
        Err(e) => {
            warn!("[GifManager] File copy failed ({e})");
            Err(e)
        }
    }
}

pub fn paste_gif_to_clipboard(url: &str) -> Result<(), String> {
    paste_gif_to_clipboard_with_uri(url).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_generation() {
        let path = GifCache::get_path_for_url("https://media.tenor.com/cat.gif");
        assert!(path.is_ok());
        assert_eq!(path.unwrap().extension().unwrap(), "gif");
    }

    #[test]
    fn rejects_ssrf_targets() {
        let dir = std::env::temp_dir().join("gif-ssrf-test");
        let dest = dir.join("x.gif");
        assert!(Downloader::download("http://127.0.0.1/x.gif", &dest).is_err());
        assert!(Downloader::download("https://127.0.0.1/x.gif", &dest).is_err());
        assert!(Downloader::download("https://evil.example/x.gif", &dest).is_err());
    }
}
