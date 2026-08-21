//! Privacy filters for clipboard history.
//! Secrets, password-manager windows, and optional image capture.

use serde::{Deserialize, Serialize};

/// Runtime privacy policy loaded from user settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrivacyPolicy {
    /// Drop clipboard text that looks like a credential or key.
    pub filter_secrets: bool,
    /// Persist captured images (full PNG on disk + thumbnail in history).
    pub save_images: bool,
    /// Skip capture when the focused app looks like a password manager / incognito window.
    pub exclude_sensitive_apps: bool,
    /// Extra WM_CLASS / title fragments supplied by the user.
    pub extra_excluded_apps: Vec<String>,
}

impl Default for PrivacyPolicy {
    fn default() -> Self {
        Self {
            filter_secrets: true,
            save_images: true,
            exclude_sensitive_apps: true,
            extra_excluded_apps: Vec::new(),
        }
    }
}

const DEFAULT_EXCLUDED_APPS: &[&str] = &[
    "keepass",
    "keepassxc",
    "keepassx",
    "1password",
    "1password-linux",
    "bitwarden",
    "bitwarden-desktop",
    "vaultwarden",
    "lastpass",
    "enpass",
    "protonpass",
    "proton-pass",
    "protonmail",
    "authy",
    "seahorse",
    "gnome-keyring",
    "kwallet",
    "kwalletmanager",
    "secret-service",
    "snap.1password",
    "org.keepassxc.keepassxc",
    "com.bitwarden.desktop",
    "org.gnome.seahorse",
];

const SENSITIVE_TITLE_FRAGMENTS: &[&str] = &[
    "private browsing",
    "incognito",
    "inprivate",
    "password",
    "passkey",
    "unlock vault",
    "master password",
];

/// True when `text` looks like a secret that must never be stored.
pub fn looks_like_secret(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }

    if looks_like_private_key(trimmed) {
        return true;
    }
    if looks_like_known_token(trimmed) {
        return true;
    }
    if looks_like_telegram_bot_token(trimmed) {
        return true;
    }
    if looks_like_jwt(trimmed) {
        return true;
    }
    if looks_like_password_assignment(trimmed) {
        return true;
    }
    false
}

fn looks_like_private_key(text: &str) -> bool {
    text.contains("BEGIN ") && text.contains("PRIVATE KEY")
}

fn looks_like_known_token(text: &str) -> bool {
    let compact = text.trim();
    const PREFIXES: &[&str] = &[
        "ghp_",
        "gho_",
        "ghu_",
        "ghs_",
        "ghr_",
        "github_pat_",
        "glpat-",
        "glptt-",
        "gldt-",
        "npm_",
        "hf_",
        "xoxb-",
        "xoxp-",
        "xoxa-",
        "xoxr-",
        "xoxe-",
        "xoxc-",
        "sk_live_",
        "rk_live_",
        "sk_test_",
        "sk-ant-",
        "AIza",
        "ya29.",
        "EAAC", // Facebook Graph API tokens / توکن‌های Graph فیسبوک
        "SG.", // SendGrid API keys / کلیدهای API سندگریل
        "xoxs-", // Slack short-lived tokens / توکن‌های کوتاه‌عمر اسلک
        "whsec_", // Stripe webhook secrets / اسرار وبهوک استرایپ
    ];
    if PREFIXES.iter().any(|p| compact.starts_with(p)) {
        return compact.len() >= 20;
    }
    if compact.starts_with("sk-") && compact.len() >= 24 && compact.is_ascii() {
        return compact
            .bytes()
            .skip(3)
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
    }
    let lower = compact.to_ascii_lowercase();
    if (lower.starts_with("bearer ") || lower.contains("authorization: bearer "))
        && compact.len() >= 20
    {
        return true;
    }
    if let Some(idx) = compact.find("AKIA") {
        let slice = compact.get(idx..idx + 20).unwrap_or("");
        return slice.len() == 20
            && slice.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit());
    }
    false
}

/// True for Telegram bot tokens: `<bot_id>:<secret>` where the id is 6–12
/// digits and the secret is at least 30 base62 chars (`-`/`_` allowed).
/// Since 2025 Bot API tokens start with the bot id followed by `:`; the
/// length gate keeps ordinary strings like `2026:notes` unclassified.
/// درست برای توکن‌های ربات تلگرام: `<bot_id>:<secret>` — شناسهٔ ۶ تا ۱۲
/// رقم و secret دست‌کم ۳۰ نویسهٔ base62 (با `-`/`_`). دروازهٔ طول مانع
/// از شناسایی متن‌های عادی مانند `2026:notes` می‌شود.
fn looks_like_telegram_bot_token(text: &str) -> bool {
    let compact = text.trim();
    let Some((id, secret)) = compact.split_once(':') else {
        return false;
    };
    (6..=12).contains(&id.len())
        && id.bytes().all(|b| b.is_ascii_digit())
        && secret.len() >= 30
        && secret.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn looks_like_jwt(text: &str) -> bool {
    let compact = text.trim();
    if !compact.starts_with("eyJ") {
        return false;
    }
    let mut parts = compact.split('.');
    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some(h), Some(p), Some(s), None)
            if h.len() > 10 && p.len() > 10 && s.len() > 10
    )
}

fn looks_like_password_assignment(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    const KEYS: &[&str] = &[
        "password=",
        "passwd=",
        "pwd=",
        "secret=",
        "client_secret=",
        "api_secret=",
        "api_key=",
        "access_token=",
        "refresh_token=",
        "auth_token=",
        "aws_secret_access_key=",
        "accountkey=",
        "sharedaccesssignature=",
        "private_key=",
    ];
    KEYS.iter().any(|k| lower.contains(k))
}

/// True when the focused window should not contribute clipboard history.
pub fn is_sensitive_source(class: &str, title: &str, extra: &[String]) -> bool {
    let class_l = class.to_ascii_lowercase();
    let title_l = title.to_ascii_lowercase();

    if DEFAULT_EXCLUDED_APPS
        .iter()
        .any(|frag| class_l.contains(frag) || title_l.contains(frag))
    {
        return true;
    }
    if SENSITIVE_TITLE_FRAGMENTS
        .iter()
        .any(|frag| title_l.contains(frag))
    {
        return true;
    }
    extra.iter().any(|frag| {
        let f = frag.trim().to_ascii_lowercase();
        !f.is_empty() && (class_l.contains(&f) || title_l.contains(&f))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_pem_and_tokens() {
        assert!(looks_like_secret(
            "-----BEGIN OPENSSH PRIVATE KEY-----\nabc\n-----END OPENSSH PRIVATE KEY-----"
        ));
        assert!(looks_like_secret("ghp_abcdefghijklmnopqrstuvwxyz0123456789"));
        assert!(looks_like_secret("sk-abcdefghijklmnopqrstuvwxyz012345"));
        assert!(looks_like_secret(
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4ifQ.s3cr3tSignatureHere"
        ));
        assert!(looks_like_secret("https://example.com/login?password=hunter2"));
        // Long dumps must still be filtered. / فایل‌های بلند هم باید فیلتر شوند.
        let long_env = format!("{}password=supersecret", "x".repeat(5000));
        assert!(looks_like_secret(&long_env));
        assert!(!looks_like_secret("hello world"));
        assert!(!looks_like_secret("https://example.com/docs"));
    }

    #[test]
    fn detects_password_managers() {
        assert!(is_sensitive_source("keepassxc", "KeePassXC", &[]));
        assert!(is_sensitive_source("firefox", "Private Browsing", &[]));
        assert!(is_sensitive_source("Code", "main.rs", &["code".into()]));
        assert!(!is_sensitive_source("firefox", "Example Domain", &[]));
    }

    // Property-style edge cases: token length gates must not be bypassable by
    // surrounding whitespace or a too-short value.
    // موارد لبه به سبک property: شرط طول token نباید با فضای خالی یا مقدار
    // خیلی کوتاه دور زده شود.
    #[test]
    fn detects_telegram_bot_tokens_and_new_prefixes() {
        // Telegram bot tokens: numeric id + colon + long base62 secret.
        assert!(looks_like_secret("1234567890:AAFcdefghijklmnopqrstuvwxyzABCDEFGHIJKL"));
        // Well-known service prefixes with the ≥20 length gate.
        assert!(looks_like_secret("whsec_abcdefghijklmnopqrstuvwxyz123456"));
        assert!(looks_like_secret("EAACEdEose0cBAabcdefghijklmnopqrstuvwxyz123456"));
        assert!(looks_like_secret(
            "SG.abcdefghijklmnopqrstuvwxyz1234567890.ABCDEFGHIJKLMNOPQRST"
        ));
        // Short ids, non-numeric ids, or short secrets must NOT be flagged.
        assert!(!looks_like_secret("12345:shortsecret"));
        assert!(!looks_like_secret("abcdefg:hijklmnopqrstuvwxyzABCDEFGHIJKLMNOP"));
        assert!(!looks_like_secret("1234567890:short"));
        // Ordinary strings must stay unclassified.
        assert!(!looks_like_secret("2026:notes"));
    }

    #[test]
    fn token_length_gate_is_enforced() {
        // Too short to be a real token → not a secret (avoid over-blocking).
        assert!(!looks_like_secret("ghp_short"));
        // With a real length, whitespace trimming still counts it as a secret.
        assert!(looks_like_secret("  ghp_abcdefghijklmnopqrstuvwxyz0123456789  "));
        // Bearer token below the length gate is not flagged.
        assert!(!looks_like_secret("bearer ab"));
    }

    #[test]
    fn private_key_without_full_marker_is_not_flag() {
        // Fragments like "PRIVATE KEY" alone (without "BEGIN") are ambiguous.
        assert!(!looks_like_secret("PRIVATE KEY"));
        assert!(!looks_like_secret("BEGIN OPENSSH"));
    }

    #[test]
    fn sensitive_titles_and_extra_are_or_combined() {
        assert!(is_sensitive_source("app", "My Master Password", &[]));
        // An extra fragment matches when it appears in the class OR the title.
        assert!(is_sensitive_source("vault", "app", &["vault".into()]));
        assert!(is_sensitive_source("app", "VaultApp", &["vault".into()]));
        // Empty extra fragment never matches.
        assert!(!is_sensitive_source("app", "app", &["   ".into()]));
    }
}
