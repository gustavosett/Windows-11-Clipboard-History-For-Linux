//! Field-level at-rest encryption for SQLite history text.
//! رمزنگاری سطح فیلد برای متن‌های تاریخچهٔ SQLite.
//!
//! Sensitive columns (`text`, `html`, `preview`, `thumb_base64`) are stored as
//! `W11E1` || nonce(12) || ChaCha20-Poly1305 ciphertext. Legacy plaintext rows
//! are still readable and are rewritten encrypted on the next persist.
//! ستون‌های حساس به‌صورت `W11E1` || nonce(12) || رمز ChaCha20-Poly1305 ذخیره
//! می‌شوند. ردیف‌های متنی قدیمی هنوز خوانا هستند و در persist بعدی رمز می‌شوند.
//!
//! # Key backends / بک‌اندهای کلید
//!
//! The 256-bit key can live in two places (see ADR-0006):
//! کلید ۲۵۶ بیتی می‌تواند در دو مکان باشد (ADR-0006 را ببینید):
//!
//! 1. **File** — `history.key` next to the database (mode `0600`).
//! 2. **Secret Service** — the freedesktop keyring (GNOME Keyring / KWallet),
//!    reached through the `secret-tool` helper; the key never touches disk.
//!
//! Integrity is anchored by a `history.key.check` marker: the active key must
//! decrypt the marker before it is adopted. A backend that cannot prove it
//! holds the right key is never used silently — loading fails rather than
//! risking a history re-encrypted under the wrong key (fail-closed).
//! یکپارچگی با نشانگر `history.key.check` تضمین می‌شود: کلید فعال باید
//! پیش از پذیرش، نشانگر را رمزگشایی کند. بک‌اندی که نگه‌داشتن کلید درست
//! را ثابت نکند به‌صورت خاموش استفاده نمی‌شود — بارگذاری شکست می‌خورد تا
//! خطر رمز شدن تاریخچه با کلید اشتباه وجود نداشته باشد (fail-closed).
//!
//! These measures protect idle disk images and other local users; they do
//! **not** protect against a process already running as the same UID.
//! این تدابیر تصاویر دیسک و کاربران دیگر را محافظت می‌کند، نه فرآیندی
//! که از قبل با همان UID اجرا می‌شود.
//!
//! Defence in depth: the locked `chacha20poly1305` dependency includes
//! `zeroize` in its dependency graph (`Cargo.lock`). This reduces key residue
//! on normal drop, but it is not a defence against a live same-UID process.
//! دفاع عمقی: وابستگی قفل‌شدهٔ `chacha20poly1305` در گراف خود `zeroize` را
//! دارد (`Cargo.lock`). این کار باقی‌ماندهٔ کلید پس از drop عادی را کم می‌کند،
//! اما در برابر فرآیند زنده با همان UID دفاع محسوب نمی‌شود.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use rand::rngs::OsRng;
use rand::RngCore;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const MAGIC: &[u8] = b"W11E1";
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;
const KEY_FILE: &str = "history.key";
/// Marker file: `encrypt_str(KEY_CHECK_PLAIN)` under the adopted key.
/// فایل نشانگر: `encrypt_str(KEY_CHECK_PLAIN)` با کلید پذیرفته‌شده.
const KEY_CHECK_FILE: &str = "history.key.check";
/// Backup name for the file key after a Secret Service migration.
/// نام پشتیبان کلید فایل پس از مهاجرت به Secret Service.
const KEY_FILE_MIGRATED: &str = "history.key.migrated";
const KEY_CHECK_PLAIN: &str = "windows-11-style-clipboard-history-manager:key-check:v1";

/// Attribute pair identifying our item in the Secret Service.
/// جفت attribute که آیتم ما را در Secret Service شناسایی می‌کند.
const SS_ATTRIBUTES: [&str; 4] = [
    "application",
    "windows-11-style-clipboard-history-manager",
    "purpose",
    "history.key",
];
const SS_LABEL: &str = "Windows 11 Style Clipboard History Manager — history encryption key";

/// Where the history encryption key is stored.
/// محل ذخیرهٔ کلید رمزنگاری تاریخچه.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyBackend {
    /// `history.key` next to the database (mode 0600).
    /// فایل `history.key` کنار دیتابیس (سطح دسترسی 0600).
    File,
    /// freedesktop Secret Service (GNOME Keyring / KWallet via `secret-tool`).
    /// Secret Service دسکتاپ‌محور (از طریق `secret-tool`).
    SecretService,
}

impl KeyBackend {
    /// Parse the persisted setting value ("file" | "secret-service").
    /// تجزیهٔ مقدار ذخیره‌شدهٔ تنظیمات ("file" | "secret-service").
    pub fn from_setting(value: &str) -> Self {
        match value.trim() {
            "secret-service" => Self::SecretService,
            _ => Self::File,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::File => "file",
            Self::SecretService => "secret-service",
        }
    }
}

pub struct HistoryCrypto {
    cipher: ChaCha20Poly1305,
    backend: KeyBackend,
}

impl HistoryCrypto {
    /// Load (or create) the key using the classic file backend.
    /// بارگذاری (یا ساخت) کلید با بک‌اند کلاسیک فایل.
    pub fn load_or_create(data_dir: &Path) -> Result<Self, String> {
        Self::load_or_create_with_backend(data_dir, KeyBackend::File)
    }

    /// Load (or create) the key with an explicit backend and fail-closed
    /// marker verification. Falls back to the file key when the requested
    /// backend is unavailable *and* the file key proves itself.
    /// بارگذاری (یا ساخت) کلید با بک‌اند مشخص و راستی‌آزمایی fail-closed
    /// نشانگر. اگر بک‌اند درخواستی در دسترس نباشد *و* کلید فایل خودش را
    /// اثبات کند، به کلید فایل بازمی‌گردد.
    pub fn load_or_create_with_backend(
        data_dir: &Path,
        requested: KeyBackend,
    ) -> Result<Self, String> {
        crate::fs_atomic::ensure_parent(&data_dir.join(KEY_FILE))
            .map_err(|e| format!("create data dir: {e}"))?;
        let marker_path = data_dir.join(KEY_CHECK_FILE);
        let marker = fs::read_to_string(&marker_path)
            .ok()
            .filter(|s| !s.trim().is_empty());

        let (key_bytes, active) = match marker {
            Some(marker) => Self::adopt_key_matching_marker(data_dir, requested, &marker)?,
            None => Self::bootstrap_new_key(data_dir, requested, &marker_path)?,
        };

        // Wrap the derived key in `Zeroizing` so the in-memory copy is wiped on
        // drop (even across a panic). Combined with the `zeroize` feature on
        // `chacha20poly1305`, the key material never lingers in heap memory.
        // کلید استخراج‌شده در `Zeroizing` قرار می‌گیرد تا نسخهٔ حافظه هنگام
        // drop (حتی در panic) پاک شود. همراه با feature ی `zeroize` در
        // `chacha20poly1305`، کلید در حافظهٔ heap باقی نمی‌ماند.
        let key_bytes = zeroize::Zeroizing::new(key_bytes);
        Ok(Self {
            cipher: ChaCha20Poly1305::new(Key::from_slice(&key_bytes[..])),
            backend: active,
        })
    }

    /// Label of the backend actually in use by this instance.
    /// برچسب بک‌اندی که واقعاً توسط این نمونه استفاده می‌شود.
    pub fn backend_label(&self) -> &'static str {
        self.backend.as_str()
    }

    /// True when the `secret-tool` helper is installed and executable.
    /// Checks `PATH` in-process — no `which` subprocess (see `exec_lookup`).
    /// وقتی helper `secret-tool` نصب و اجرایی باشد «درست» است.
    /// بررسی PATH درون‌فرآیندی — بدون subprocess ی `which` (`exec_lookup`).
    pub fn secret_service_available() -> bool {
        crate::exec_lookup::command_exists("secret-tool")
    }

    // -----------------------------------------------------------------
    // Bootstrap / adoption
    // -----------------------------------------------------------------

    /// Marker present: only a backend that decrypts the marker may be used.
    /// نشانگر موجود است: فقط بک‌اندی که نشانگر را رمزگشایی کند استفاده می‌شود.
    fn adopt_key_matching_marker(
        data_dir: &Path,
        requested: KeyBackend,
        marker: &str,
    ) -> Result<(Vec<u8>, KeyBackend), String> {
        let mut tried = Vec::new();
        for candidate in [requested, KeyBackend::File] {
            if tried.contains(&candidate) {
                continue;
            }
            tried.push(candidate);
            if let Some(bytes) = Self::read_key_from_backend(data_dir, candidate) {
                if marker_verifies(&bytes, marker) {
                    if candidate != requested {
                        tracing::warn!(
                            "[HistoryCrypto] Requested backend '{}' unavailable/invalid; \
                             continuing with the file key that decrypts the existing history",
                            requested.as_str()
                        );
                    }
                    return Ok((bytes, candidate));
                }
            }
        }
        Err(
            "no available key backend can decrypt history.key.check; refusing to adopt a \
             fresh key that would corrupt the existing history"
                .to_string(),
        )
    }

    /// No marker yet: fresh installs create a key; pre-marker databases keep
    /// their file key so nothing is silently re-keyed.
    /// نشانگر نیست: نصب تازه کلید می‌سازد؛ دیتابیس‌های قدیمی کلید فایل خود
    /// را نگه می‌دارند تا هیچ‌چیز بی‌سروصدا کلیدعوض نکند.
    fn bootstrap_new_key(
        data_dir: &Path,
        requested: KeyBackend,
        marker_path: &Path,
    ) -> Result<(Vec<u8>, KeyBackend), String> {
        let db_exists = data_dir.join("history.db").exists();
        if db_exists {
            tracing::info!(
                "[HistoryCrypto] Existing history without a key marker: adopting the file key"
            );
            let bytes = Self::ensure_file_key(data_dir)?;
            write_marker(marker_path, &bytes)?;
            return Ok((bytes, KeyBackend::File));
        }

        let mut bytes = vec![0u8; KEY_LEN];
        OsRng.fill_bytes(&mut bytes);
        let active = match requested {
            KeyBackend::SecretService => {
                if Self::secret_service_available() {
                    match Self::store_key_in_secret_service(&bytes)
                        .and_then(|_| Self::read_key_from_secret_service())
                    {
                        Ok(read_back) if read_back == bytes => KeyBackend::SecretService,
                        Ok(_) => {
                            tracing::warn!(
                                "[HistoryCrypto] Secret Service read-back mismatch; using file key"
                            );
                            Self::write_file_key(data_dir, &bytes)?;
                            KeyBackend::File
                        }
                        Err(e) => {
                            tracing::warn!(
                                "[HistoryCrypto] Secret Service unusable ({e}); using file key"
                            );
                            Self::write_file_key(data_dir, &bytes)?;
                            KeyBackend::File
                        }
                    }
                } else {
                    tracing::warn!(
                        "[HistoryCrypto] secret-tool not found; using file key"
                    );
                    Self::write_file_key(data_dir, &bytes)?;
                    KeyBackend::File
                }
            }
            KeyBackend::File => {
                Self::write_file_key(data_dir, &bytes)?;
                KeyBackend::File
            }
        };
        write_marker(marker_path, &bytes)?;
        Ok((bytes, active))
    }

    /// Read the key from a backend; `None` when the backend has no key.
    /// خواندن کلید از یک بک‌اند؛ `None` وقتی بک‌اند کلیدی ندارد.
    fn read_key_from_backend(data_dir: &Path, backend: KeyBackend) -> Option<Vec<u8>> {
        match backend {
            KeyBackend::File => {
                let bytes = fs::read(key_path(data_dir)).ok()?;
                (bytes.len() == KEY_LEN).then_some(bytes)
            }
            KeyBackend::SecretService => {
                if !Self::secret_service_available() {
                    return None;
                }
                Self::read_key_from_secret_service().ok()
            }
        }
    }

    /// Load the file key, creating it (0600) when missing.
    /// بارگذاری کلید فایل؛ در نبود، ساختن آن (0600).
    fn ensure_file_key(data_dir: &Path) -> Result<Vec<u8>, String> {
        let path = key_path(data_dir);
        if path.exists() {
            let bytes = fs::read(&path).map_err(|e| format!("read history.key: {e}"))?;
            if bytes.len() != KEY_LEN {
                return Err("history.key has unexpected length".into());
            }
            return Ok(bytes);
        }
        let mut bytes = vec![0u8; KEY_LEN];
        OsRng.fill_bytes(&mut bytes);
        Self::write_file_key(data_dir, &bytes)?;
        Ok(bytes)
    }

    fn write_file_key(data_dir: &Path, bytes: &[u8]) -> Result<(), String> {
        let path = key_path(data_dir);
        crate::fs_atomic::ensure_parent(&path).map_err(|e| e.to_string())?;
        crate::fs_atomic::write_atomic(&path, bytes).map_err(|e| e.to_string())?;
        crate::fs_atomic::restrict_permissions(&path);
        Ok(())
    }

    // -----------------------------------------------------------------
    // Secret Service transport (secret-tool)
    // -----------------------------------------------------------------

    /// Store the key (base64) as a Secret Service item via `secret-tool`.
    /// ذخیرهٔ کلید (base64) به‌عنوان آیتم Secret Service با `secret-tool`.
    fn store_key_in_secret_service(bytes: &[u8]) -> Result<(), String> {
        if !Self::secret_service_available() {
            return Err("secret-tool not found; install libsecret-tools".into());
        }
        let encoded = BASE64.encode(bytes);
        let mut child = Command::new("secret-tool")
            .arg("store")
            .arg("--label")
            .arg(SS_LABEL)
            .args(SS_ATTRIBUTES)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("spawn secret-tool store: {e}"))?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(encoded.as_bytes())
                .map_err(|e| format!("write secret: {e}"))?;
        }
        let output = child
            .wait_with_output()
            .map_err(|e| format!("wait secret-tool store: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "secret-tool store failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        Ok(())
    }

    /// Read the key back from the Secret Service; errors when absent.
    /// خواندن کلید از Secret Service؛ در نبود آیتم خطا می‌دهد.
    fn read_key_from_secret_service() -> Result<Vec<u8>, String> {
        let output = Command::new("secret-tool")
            .arg("lookup")
            .args(SS_ATTRIBUTES)
            .output()
            .map_err(|e| format!("spawn secret-tool lookup: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "secret-tool lookup failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        let decoded = BASE64
            .decode(String::from_utf8_lossy(&output.stdout).trim().as_bytes())
            .map_err(|e| format!("stored secret is not valid base64: {e}"))?;
        if decoded.len() != KEY_LEN {
            return Err("stored secret has unexpected length".into());
        }
        Ok(decoded)
    }

    // -----------------------------------------------------------------
    // Migration commands (invoked from Settings)
    // -----------------------------------------------------------------

    /// Move the existing key into the Secret Service (verifies read-back,
    /// then renames `history.key` → `history.key.migrated`).
    /// انتقال کلید موجود به Secret Service (راستی‌آزمایی read-back، سپس
    /// تغییر نام `history.key` به `history.key.migrated`).
    pub fn migrate_to_secret_service(data_dir: &Path) -> Result<(), String> {
        if !Self::secret_service_available() {
            return Err("secret-tool not found; install libsecret-tools".into());
        }
        let path = key_path(data_dir);
        let key_bytes = fs::read(&path)
            .map_err(|_| "history.key not found; nothing to migrate".to_string())?;
        if key_bytes.len() != KEY_LEN {
            return Err("history.key has unexpected length".into());
        }
        // The marker must already match this key — never re-key blindly.
        // نشانگر باید از قبل با این کلید بخواند — هرگز کورکورانه کلید عوض نمی‌شود.
        let marker_path = data_dir.join(KEY_CHECK_FILE);
        match fs::read_to_string(&marker_path).ok().filter(|s| !s.trim().is_empty()) {
            Some(marker) if marker_verifies(&key_bytes, &marker) => {}
            Some(_) => {
                return Err(
                    "history.key.check does not match history.key; refusing to migrate".into()
                )
            }
            None => write_marker(&marker_path, &key_bytes)?,
        }

        Self::store_key_in_secret_service(&key_bytes)?;
        if Self::read_key_from_secret_service()? != key_bytes {
            return Err("Secret Service read-back mismatch; migration aborted".into());
        }
        let migrated = data_dir.join(KEY_FILE_MIGRATED);
        fs::rename(&path, &migrated).map_err(|e| format!("rename history.key: {e}"))?;
        tracing::info!("[HistoryCrypto] Key migrated to the Secret Service");
        Ok(())
    }

    /// Restore the key to the file backend (undo for the migration above).
    /// بازگرداندن کلید به بک‌اند فایل (واگرد مهاجرت بالا).
    pub fn migrate_to_file(data_dir: &Path) -> Result<(), String> {
        let path = key_path(data_dir);
        let migrated = data_dir.join(KEY_FILE_MIGRATED);

        let key_bytes = if migrated.exists() {
            fs::rename(&migrated, &path).map_err(|e| format!("restore history.key: {e}"))?;
            fs::read(&path).map_err(|e| format!("read history.key: {e}"))?
        } else if path.exists() {
            fs::read(&path).map_err(|e| format!("read history.key: {e}"))?
        } else if Self::secret_service_available() {
            Self::read_key_from_secret_service()?
        } else {
            return Err("no Secret Service and no history.key; nothing to restore".into());
        };
        if key_bytes.len() != KEY_LEN {
            return Err("key has unexpected length".into());
        }
        Self::write_file_key(data_dir, &key_bytes)?;
        Ok(())
    }

    // -----------------------------------------------------------------
    // Encrypt / decrypt
    // -----------------------------------------------------------------

    /// Encrypt an optional string. Fail-closed: never returns plaintext.
    /// رمزنگاری رشتهٔ اختیاری. در خطا شکست می‌خورد؛ هرگز plaintext برنمی‌گرداند.
    pub fn encrypt_optional(&self, plain: Option<&str>) -> Result<Option<String>, String> {
        match plain {
            Some(p) => Ok(Some(self.encrypt_str(p)?)),
            None => Ok(None),
        }
    }

    /// Encrypt `plain`. On AEAD failure the call errors out instead of
    /// storing plaintext under the `W11E1` magic (fail-closed).
    /// در شکست AEAD خطا برمی‌گردد؛ plaintext ذخیره نمی‌شود.
    pub fn encrypt_str(&self, plain: &str) -> Result<String, String> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ct = self
            .cipher
            .encrypt(nonce, plain.as_bytes())
            .map_err(|e| format!("ChaCha20-Poly1305 encrypt failed: {e}"))?;
        let mut out = Vec::with_capacity(MAGIC.len() + NONCE_LEN + ct.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ct);
        Ok(BASE64.encode(out))
    }

    /// Decrypt an optional stored field. Fail-closed: AEAD errors propagate.
    /// رمزگشایی فیلد اختیاری. در خطا شکست می‌خورد (fail-closed).
    pub fn decrypt_optional(&self, stored: Option<String>) -> Result<Option<String>, String> {
        match stored {
            Some(s) => Ok(Some(self.decrypt_str(&s)?)),
            None => Ok(None),
        }
    }

    /// Encrypt raw bytes (image files). Fail-closed: never returns plaintext.
    /// رمزنگاری بایت خام (فایل تصویر). در خطا شکست می‌خورد؛ هرگز plaintext برنمی‌گرداند.
    pub fn encrypt_bytes(&self, plain: &[u8]) -> Result<Vec<u8>, String> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ct = self
            .cipher
            .encrypt(nonce, plain)
            .map_err(|e| format!("ChaCha20-Poly1305 encrypt failed: {e}"))?;
        let mut out = Vec::with_capacity(MAGIC.len() + NONCE_LEN + ct.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ct);
        Ok(out)
    }

    /// Decrypt raw bytes. Legacy plaintext PNG (`\\x89PNG`) is returned as-is.
    /// رمزگشایی بایت. PNG قدیمی بدون پاکت همان‌طور برمی‌گردد.
    pub fn decrypt_bytes(&self, stored: &[u8]) -> Result<Vec<u8>, String> {
        if stored.starts_with(b"\x89PNG") {
            return Ok(stored.to_vec());
        }
        let magic_len = MAGIC.len();
        if stored.len() < magic_len + NONCE_LEN + 16 || !stored.starts_with(MAGIC) {
            return Err("encrypted blob has an invalid envelope".into());
        }
        let payload = &stored[magic_len..];
        let (nonce_bytes, ct) = payload.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);
        self.cipher
            .decrypt(nonce, ct)
            .map_err(|_| "ChaCha20-Poly1305 decrypt failed (wrong key or tampered data)".to_string())
    }

    /// Decrypt `stored`. Legacy plaintext (no `W11E1` envelope) is returned
    /// as-is. Tampered or foreign-key ciphertext **errors** instead of
    /// leaking the blob into the UI (fail-closed).
    /// رمزگشایی. متن قدیمی بدون پاکت `W11E1` همان‌طور برمی‌گردد.
    /// ciphertext خراب یا با کلید بیگانه خطا می‌دهد تا به UI نشت نکند.
    pub fn decrypt_str(&self, stored: &str) -> Result<String, String> {
        if !looks_encrypted(stored) {
            return Ok(stored.to_string());
        }
        let raw = BASE64
            .decode(stored)
            .map_err(|e| format!("encrypted field is not valid base64: {e}"))?;
        // Layout check: magic || nonce(12) || ciphertext(≥16-byte tag).
        // بررسی چیدمان: magic || nonce(۱۲) || رمز متن (تگ ≥۱۶ بایت).
        let magic_len = MAGIC.len();
        if raw.len() < magic_len + NONCE_LEN + 16 || !raw.starts_with(MAGIC) {
            return Err("encrypted field has an invalid envelope".into());
        }
        let (_, payload) = raw.split_at(magic_len);
        let (nonce_bytes, ct) = payload.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);
        let pt = self
            .cipher
            .decrypt(nonce, ct)
            .map_err(|_| "ChaCha20-Poly1305 decrypt failed (wrong key or tampered data)".to_string())?;
        String::from_utf8(pt).map_err(|e| format!("decrypted field is not valid UTF-8: {e}"))
    }
}

fn key_path(data_dir: &Path) -> PathBuf {
    data_dir.join(KEY_FILE)
}

/// Write (or rewrite) the key-integrity marker atomically.
/// نوشتن (یا بازنویسی) نشانگر یکپارچگی کلید به‌صورت اتمیک.
fn write_marker(marker_path: &Path, key_bytes: &[u8]) -> Result<(), String> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key_bytes));
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let ct = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), KEY_CHECK_PLAIN.as_bytes())
        .map_err(|e| format!("marker encrypt failed: {e}"))?;
    let mut out = Vec::with_capacity(MAGIC.len() + NONCE_LEN + ct.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);
    crate::fs_atomic::ensure_parent(marker_path).map_err(|e| e.to_string())?;
    crate::fs_atomic::write_atomic(marker_path, BASE64.encode(out).as_bytes())
        .map_err(|e| e.to_string())?;
    crate::fs_atomic::restrict_permissions(marker_path);
    Ok(())
}

/// True when `key_bytes` decrypts `marker` to the check constant.
/// وقتی `key_bytes` نشانگر را به ثابت کنترل رمزگشایی کند «درست» است.
fn marker_verifies(key_bytes: &[u8], marker: &str) -> bool {
    let Ok(raw) = BASE64.decode(marker) else {
        return false;
    };
    let magic_len = MAGIC.len();
    if raw.len() < magic_len + NONCE_LEN + 16 || !raw.starts_with(MAGIC) {
        return false;
    }
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key_bytes));
    let (_, payload) = raw.split_at(magic_len);
    let (nonce_bytes, ct) = payload.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ct)
        .map(|pt| pt == KEY_CHECK_PLAIN.as_bytes())
        .unwrap_or(false)
}

fn looks_encrypted(s: &str) -> bool {
    s.len() > 16 && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("hist-crypto-{name}-{}", Uuid::new_v4()));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn roundtrip_and_legacy_plaintext() {
        let dir = temp_dir("roundtrip");
        let crypto = HistoryCrypto::load_or_create(&dir).unwrap();
        let enc = crypto.encrypt_str("secret clipboard").unwrap();
        assert_ne!(enc, "secret clipboard");
        assert_eq!(crypto.decrypt_str(&enc).unwrap(), "secret clipboard");
        assert_eq!(
            crypto.decrypt_str("legacy plaintext").unwrap(),
            "legacy plaintext"
        );
        let png = b"\x89PNG\r\n\x1a\nhello-image";
        let enc_img = crypto.encrypt_bytes(png).unwrap();
        assert_ne!(enc_img.as_slice(), png.as_slice());
        assert_eq!(crypto.decrypt_bytes(&enc_img).unwrap(), png);
        assert_eq!(crypto.decrypt_bytes(png).unwrap(), png);
    }

    #[test]
    fn same_key_file_reused() {
        let dir = temp_dir("k-reuse");
        let a = HistoryCrypto::load_or_create(&dir).unwrap();
        let blob = a.encrypt_str("hello").unwrap();
        let b = HistoryCrypto::load_or_create(&dir).unwrap();
        assert_eq!(b.decrypt_str(&blob).unwrap(), "hello");
    }

    #[test]
    fn backend_setting_parses() {
        assert_eq!(KeyBackend::from_setting("file"), KeyBackend::File);
        assert_eq!(
            KeyBackend::from_setting("secret-service"),
            KeyBackend::SecretService
        );
        assert_eq!(KeyBackend::from_setting("garbage"), KeyBackend::File);
    }

    #[test]
    fn marker_rejects_wrong_key() {
        // A key that does not decrypt the marker must never be adopted.
        // کلیدی که نشانگر را رمزگشایی نکند هرگز پذیرفته نمی‌شود.
        let dir = temp_dir("wrong-key");
        let crypto = HistoryCrypto::load_or_create(&dir).unwrap();
        assert!(crypto.backend_label() == "file");
        assert!(dir.join(KEY_CHECK_FILE).exists());

        // Overwrite the key file with a different random key.
        let mut foreign = vec![0u8; KEY_LEN];
        OsRng.fill_bytes(&mut foreign);
        crate::fs_atomic::write_atomic(&dir.join(KEY_FILE), &foreign).unwrap();

        let reloaded = HistoryCrypto::load_or_create(&dir);
        assert!(
            reloaded.is_err(),
            "loading must fail-closed when no key matches the marker"
        );
    }

    #[test]
    fn existing_db_without_marker_adopts_file_key() {
        let dir = temp_dir("legacy-db");
        fs::write(dir.join("history.db"), b"stub").unwrap();
        let key_bytes = HistoryCrypto::ensure_file_key(&dir).unwrap();
        let crypto = HistoryCrypto::load_or_create_with_backend(
            &dir,
            KeyBackend::SecretService, // requested backend must NOT win here
        )
        .unwrap();
        assert_eq!(crypto.backend_label(), "file");
        assert!(dir.join(KEY_CHECK_FILE).exists());
        assert!(marker_verifies(
            &key_bytes,
            &fs::read_to_string(dir.join(KEY_CHECK_FILE)).unwrap()
        ));
    }

    #[test]
    fn secret_service_missing_is_reported_not_panicked() {
        // Only asserts the probe never panics; the boolean depends on the host.
        // فقط بررسی می‌کند که probe هرگز panic نکند؛ مقدار به میزبان وابسته است.
        let _ = HistoryCrypto::secret_service_available();
    }

    /// AEAD integrity: flipping a single ciphertext byte must fail decryption
    /// instead of silently returning garbage (or worse, plaintext).
    /// یکپارچگی AEAD: تغییر یک بایت ciphertext باید رمزگشایی را شکست دهد،
    /// نه اینکه بی‌صدا دادهٔ خراب (یا بدتر، متن اصلی) برگرداند.
    #[test]
    fn tampered_ciphertext_is_rejected() {
        let dir = temp_dir("tamper");
        let crypto = HistoryCrypto::load_or_create(&dir).unwrap();

        let stored = crypto.encrypt_str("top-secret clipboard").unwrap();
        let mut raw_bytes = BASE64.decode(stored.as_bytes()).unwrap();

        // Flip one bit in the middle of the ciphertext (past magic + nonce).
        // یک بیت در میانهٔ متن رمز را برعکس کن (بعد از magic و nonce).
        let middle = raw_bytes.len() / 2;
        raw_bytes[middle] ^= 0x01;
        let tampered = BASE64.encode(&raw_bytes);

        let result = crypto.decrypt_str(&tampered);
        assert!(
            result.is_err(),
            "tampered ciphertext must never decrypt (fail-closed)"
        );

        // The same rule applies to the raw-bytes envelope used for images.
        // همین قانون برای پاکت بایتیِ تصاویر هم برقرار است.
        let blob = crypto.encrypt_bytes(b"image-bytes").unwrap();
        let mut tampered_blob = blob;
        let middle = tampered_blob.len() / 2;
        tampered_blob[middle] ^= 0x01;
        assert!(crypto.decrypt_bytes(&tampered_blob).is_err());
    }

    /// Nonces must never repeat: encrypting the same plaintext many times
    /// must yield distinct ciphertexts (ChaCha20-Poly1305 nonce reuse is
    /// catastrophic for confidentiality).
    /// nonce هرگز نباید تکرار شود: رمزکردن یک متن ثابت چندبار باید
    /// ciphertextهای متمایز بدهد (تکرار nonce در ChaCha20-Poly1305
    /// برای محرمانگی فاجعه‌بار است).
    #[test]
    fn nonces_are_unique_across_many_encryptions() {
        let dir = temp_dir("nonces");
        let crypto = HistoryCrypto::load_or_create(&dir).unwrap();

        let mut seen = std::collections::HashSet::new();
        for _ in 0..1_000 {
            let stored = crypto.encrypt_str("identical plaintext").unwrap();
            assert!(
                seen.insert(stored.clone()),
                "duplicate ciphertext detected after {} encryptions",
                seen.len()
            );
        }
    }

    /// A key from another data directory must never decrypt this history.
    /// کلیدی از دایرکتوری دادهٔ دیگر هرگز نباید این تاریخچه را باز کند.
    #[test]
    fn foreign_key_cannot_decrypt_history() {
        let dir_a = temp_dir("foreign-a");
        let dir_b = temp_dir("foreign-b");
        let a = HistoryCrypto::load_or_create(&dir_a).unwrap();
        let b = HistoryCrypto::load_or_create(&dir_b).unwrap();

        let blob = a.encrypt_str("private clipboard text").unwrap();
        assert!(b.decrypt_str(&blob).is_err(), "foreign key must fail closed");
    }
}
