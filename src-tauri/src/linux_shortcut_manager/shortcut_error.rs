//! Error type shared by every shortcut handler.
//!
//! Kept in its own module so per-DE handlers and utilities can reuse it
//! without coupling to the parent module.

use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ShortcutError {
    Io(io::Error),
    CommandFailed { cmd: String, stderr: String },
    DependencyMissing(String),
    ParseError(String),
    UnsupportedEnvironment(String),
}

impl From<io::Error> for ShortcutError {
    fn from(e: io::Error) -> Self {
        ShortcutError::Io(e)
    }
}

impl fmt::Display for ShortcutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO Error: {e}"),
            Self::CommandFailed { cmd, stderr } => {
                write!(f, "Command '{cmd}' failed: {stderr}")
            }
            Self::DependencyMissing(dep) => write!(f, "Missing dependency: {dep}"),
            Self::ParseError(s) => write!(f, "Config parse error: {s}"),
            Self::UnsupportedEnvironment(e) => write!(f, "Unsupported environment: {e}"),
        }
    }
}

impl std::error::Error for ShortcutError {}

/// Result alias used across all shortcut handlers.
pub type Result<T> = std::result::Result<T, ShortcutError>;
