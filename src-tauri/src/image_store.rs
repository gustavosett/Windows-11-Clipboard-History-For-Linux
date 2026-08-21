//! On-disk image storage for clipboard history.
//! / ذخیره‌سازی تصویر روی دیسک برای تاریخچهٔ کلیپ‌بورد.
//!
//! Full-resolution PNG is encrypted at rest with the same ChaCha20-Poly1305
//! envelope as history text (`W11E1`). Only a small thumbnail is sent over IPC.
//! PNG تمام‌وضوح با همان پاکت ChaCha20-Poly1305 رمز می‌شود؛ فقط بندانگشتی از IPC می‌گذرد.

use arboard::ImageData;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use image::{DynamicImage, ImageFormat};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use crate::history_crypto::HistoryCrypto;

const MAX_THUMB_DIM: u32 = 256;

#[derive(Debug, Clone)]
pub struct StoredImage {
    pub full_path: PathBuf,
    pub thumb_base64: String,
    pub width: u32,
    pub height: u32,
}

/// Persist a clipboard image as an encrypted PNG and return a compact thumbnail.
/// ذخیرهٔ تصویر کلیپ‌بورد به‌صورت PNG رمزشده و بازگرداندن بندانگشتی فشرده.
pub fn store_rgba(
    images_dir: &Path,
    id: &str,
    image: &ImageData<'_>,
    crypto: &HistoryCrypto,
) -> Result<StoredImage, String> {
    fs::create_dir_all(images_dir).map_err(|e| format!("Failed to create images dir: {e}"))?;
    crate::fs_atomic::restrict_permissions(images_dir);

    let width = image.width as u32;
    let height = image.height as u32;
    let rgba = image::RgbaImage::from_raw(width, height, image.bytes.to_vec())
        .ok_or_else(|| "Image dimensions do not match pixel buffer".to_string())?;
    let dyn_img = DynamicImage::ImageRgba8(rgba);

    let mut png_buf = Cursor::new(Vec::new());
    dyn_img
        .write_to(&mut png_buf, ImageFormat::Png)
        .map_err(|e| format!("Failed to encode PNG: {e}"))?;
    write_encrypted_png(images_dir, id, png_buf.get_ref(), crypto)?;

    let thumb_base64 = encode_thumbnail(&dyn_img)?;
    Ok(StoredImage {
        full_path: image_path(images_dir, id),
        thumb_base64,
        width,
        height,
    })
}

/// Store an already-encoded PNG (e.g. during JSON → SQLite migration).
/// ذخیرهٔ PNG از پیش‌کدشده (مثلاً هنگام مهاجرت JSON → SQLite).
pub fn store_png_bytes(
    images_dir: &Path,
    id: &str,
    png: &[u8],
    crypto: &HistoryCrypto,
) -> Result<StoredImage, String> {
    fs::create_dir_all(images_dir).map_err(|e| format!("Failed to create images dir: {e}"))?;
    let dyn_img = image::load_from_memory(png).map_err(|e| format!("Invalid PNG: {e}"))?;
    let width = dyn_img.width();
    let height = dyn_img.height();

    write_encrypted_png(images_dir, id, png, crypto)?;

    Ok(StoredImage {
        full_path: image_path(images_dir, id),
        thumb_base64: encode_thumbnail(&dyn_img)?,
        width,
        height,
    })
}

fn image_path(images_dir: &Path, id: &str) -> PathBuf {
    images_dir.join(format!("{id}.png"))
}

fn write_encrypted_png(
    images_dir: &Path,
    id: &str,
    png: &[u8],
    crypto: &HistoryCrypto,
) -> Result<(), String> {
    let full_path = image_path(images_dir, id);
    let encrypted = crypto.encrypt_bytes(png)?;
    crate::fs_atomic::write_atomic(&full_path, &encrypted).map_err(|e| e.to_string())?;
    crate::fs_atomic::restrict_permissions(&full_path);
    Ok(())
}

pub fn encode_thumbnail(img: &DynamicImage) -> Result<String, String> {
    let thumb = img.thumbnail(MAX_THUMB_DIM, MAX_THUMB_DIM);
    let mut buffer = Cursor::new(Vec::new());
    thumb
        .write_to(&mut buffer, ImageFormat::Png)
        .map_err(|e| format!("Failed to encode thumbnail: {e}"))?;
    Ok(BASE64.encode(buffer.get_ref()))
}

/// Read a stored image, decrypting the `W11E1` envelope (legacy plaintext PNG is accepted).
/// خواندن تصویر ذخیره‌شده با رمزگشایی پاکت `W11E1` (PNG متنی قدیمی پذیرفته می‌شود).
pub fn read_png(path: &Path, crypto: &HistoryCrypto) -> Result<Vec<u8>, String> {
    let raw = fs::read(path)
        .map_err(|e| format!("Failed to read stored image {}: {e}", path.display()))?;
    let plain = crypto.decrypt_bytes(raw.as_slice())?;
    Ok(plain)
}

/// Overwrite then unlink so residual plaintext/ciphertext is not left in a hole.
/// صفرنویسی سپس حذف تا باقی‌مانده روی دیسک نماند.
pub fn remove_image(path: &Path) {
    if let Ok(meta) = fs::metadata(path) {
        let len = meta.len() as usize;
        if len > 0 && len < 64 * 1024 * 1024 {
            let _ = crate::fs_atomic::write_atomic(path, &vec![0u8; len]);
        }
    }
    let _ = fs::remove_file(path);
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn thumbnail_is_smaller_than_source() {
        let img = DynamicImage::new_rgba8(800, 600);
        let encoded = encode_thumbnail(&img).unwrap();
        assert!(!encoded.is_empty());
        // 256px PNG thumbnail should be well under a megabyte of base64.
        assert!(encoded.len() < 400_000);
    }

    #[test]
    fn roundtrip_encrypted_png_and_legacy_plaintext() {
        let dir = std::env::temp_dir().join(format!("img-enc-{}", Uuid::new_v4()));
        let _ = fs::create_dir_all(&dir);
        let crypto = HistoryCrypto::load_or_create(&dir).unwrap();

        let png = {
            let img = DynamicImage::new_rgba8(16, 16);
            let mut buf = Cursor::new(Vec::new());
            img.write_to(&mut buf, ImageFormat::Png).unwrap();
            buf.into_inner()
        };
        let stored = store_png_bytes(&dir, "item-1", &png, &crypto).unwrap();
        let on_disk = fs::read(&stored.full_path).unwrap();
        assert!(!on_disk.starts_with(b"\x89PNG"), "disk image must be encrypted");
        let decoded = read_png(&stored.full_path, &crypto).unwrap();
        assert!(decoded.starts_with(b"\x89PNG"));

        // Legacy plaintext PNG still loads. / PNG قدیمی هنوز بارگذاری می‌شود.
        let legacy = dir.join("legacy.png");
        fs::write(&legacy, &png).unwrap();
        assert_eq!(read_png(&legacy, &crypto).unwrap(), png);
    }
}
