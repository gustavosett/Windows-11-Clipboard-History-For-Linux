//! Shared process/file utilities for shortcut handlers.
//!
//! All external commands are spawned with explicit argv (never a shell), and
//! config-file edits go through an atomic write (temp file + rename) with a
//! single `.bak` backup per file.

use super::shortcut_error::{Result, ShortcutError};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

pub struct Utils;

impl Utils {
    /// True when `cmd` is executable through `PATH` (in-process probe,
    /// no `which` subprocess). See `crate::exec_lookup`.
    /// وقتی `cmd` از طریق `PATH` اجرایی است «درست» است (بررسی
    /// درون‌فرآیندی، بدون subprocess ی `which`). `crate::exec_lookup` را ببینید.
    pub fn command_exists(cmd: &str) -> bool {
        crate::exec_lookup::command_exists(cmd)
    }

    pub fn run(cmd: &str, args: &[&str]) -> Result<String> {
        let output = Command::new(cmd).args(args).output()?;

        if !output.status.success() {
            return Err(ShortcutError::CommandFailed {
                cmd: cmd.to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Reads a file, creates a `.bak` copy, modifies content via callback,
    /// then writes back atomically using a temp file rename strategy.
    /// Returns `Ok(true)` if the file was modified, `Ok(false)` if no changes
    /// were needed.
    pub fn modify_file_atomic<F>(path: &Path, modifier: F) -> Result<bool>
    where
        F: FnOnce(String) -> Result<Option<String>>,
    {
        if !path.exists() {
            // Create directory structure if missing
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
        }

        let content = if path.exists() {
            // Create a single backup file (only if it doesn't exist yet)
            let bak_path = path.with_extension("bak");
            if !bak_path.exists() {
                fs::copy(path, &bak_path)?;
                info!("[Utils] Created backup: {:?}", bak_path);
            }

            fs::read_to_string(path)?
        } else {
            String::new()
        };

        // Run modifier logic
        let new_content = match modifier(content) {
            Ok(Some(s)) => s,
            Ok(None) => return Ok(false), // No changes needed
            Err(e) => return Err(e),
        };

        // Atomic Write Strategy: write to `.tmp.<millis>`, then rename
        let tmp_path = path.with_extension(format!(
            "tmp.{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_millis()
        ));

        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(new_content.as_bytes())?;
        file.sync_all()?; // Ensure flush to disk

        // Atomic rename
        fs::rename(&tmp_path, path)?;

        Ok(true) // File was modified
    }
}
