//! Unified error types for the application.
//!
//! All Tauri commands return `Result<T, AppError>` for structured,
//! serialisable error reporting to the frontend.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Clipboard operation failed: {0}")]
    Clipboard(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Persistence error: {0}")]
    Persistence(String),

    #[error("Input simulation failed: {0}")]
    InputSimulation(String),

    #[error("Item '{id}' not found in history")]
    NotFound { id: String },

    #[error("Network error: {0}")]
    Network(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Download too large (max {max} bytes)")]
    DownloadTooLarge { max: u64 },

    #[error("Session error: {0}")]
    Session(String),

    #[error("X11 error: {0}")]
    X11(String),

    #[error("Privacy policy blocked this clipboard item")]
    PrivacyBlocked,

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("{0}")]
    Other(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    /// Stable, machine-readable error code (unaffected by message translation).
    /// Frontends/telemetry key on this; human text stays in `Display`.
    /// کد خطای پایدار و ماشین‌خوان (تحت تأثیر ترجمهٔ پیام نیست).
    /// فرانت‌اند/telemetry روی این کد کلید می‌زند؛ متن انسانی در `Display` می‌ماند.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Clipboard(_) => "clipboard_error",
            AppError::Serialization(_) => "serialization_error",
            AppError::Io(_) => "io_error",
            AppError::Persistence(_) => "persistence_error",
            AppError::InputSimulation(_) => "input_simulation_error",
            AppError::NotFound { .. } => "not_found",
            AppError::Network(_) => "network_error",
            AppError::InvalidUrl(_) => "invalid_url",
            AppError::DownloadTooLarge { .. } => "download_too_large",
            AppError::Session(_) => "session_error",
            AppError::X11(_) => "x11_error",
            AppError::PrivacyBlocked => "privacy_blocked",
            AppError::PermissionDenied(_) => "permission_denied",
            AppError::Other(_) => "other",
        }
    }
}

// ---------------------------------------------------------------------------
// From implementations for seamless ? operator usage
// ---------------------------------------------------------------------------

impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::Other(msg)
    }
}

impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        AppError::Other(msg.to_string())
    }
}

impl From<arboard::Error> for AppError {
    fn from(err: arboard::Error) -> Self {
        AppError::Clipboard(err.to_string())
    }
}

impl From<crate::clipboard_io::ClipError> for AppError {
    fn from(err: crate::clipboard_io::ClipError) -> Self {
        match err {
            crate::clipboard_io::ClipError::Arboard(e) => AppError::Clipboard(e.to_string()),
            crate::clipboard_io::ClipError::External(msg) => AppError::Clipboard(msg),
            crate::clipboard_io::ClipError::VerificationFailed(msg) => {
                AppError::Clipboard(format!("Verification failed: {msg}"))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tauri serialization (commands return AppError as a string to the frontend)
// ---------------------------------------------------------------------------

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
