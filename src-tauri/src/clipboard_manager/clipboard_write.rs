//! Writing items back onto the OS clipboard (Wayland / X11 / arboard).
//! بازنویتن آیتم‌ها روی کلیپ‌بورد سیستم (Wayland / X11 / arboard).

use super::*;

impl ClipboardManager {
    /// Write `item` onto the OS clipboard without injecting Ctrl+V.
    /// نوشتن آیتم روی کلیپ‌بورد سیستم بدون تزریق Ctrl+V.
    ///
    /// Callers (`commands::paste_item`) then authorize keystroke injection
    /// via the paste ticket + `wrote_recently` gate.
    /// فراخواننده پس از آن تزریق را با بلیت paste و گیت `wrote_recently` مجاز می‌کند.
    pub fn write_item_to_clipboard(&mut self, item: &ClipboardItem) -> Result<(), String> {
        self.mark_as_pasted(item);
        match &item.content {
            ClipboardContent::Text(text) => self.set_text_robust(text)?,
            ClipboardContent::RichText { plain, html } => self.set_html_robust(html, plain)?,
            ClipboardContent::Image {
                base64,
                width,
                height,
            } => {
                if let Some(path) = self.image_paths.get(&item.id) {
                    if let Ok(png) = crate::image_store::read_png(path, &self.crypto) {
                        self.set_image_png_bytes(&png)?;
                    } else {
                        self.set_image_robust(base64, *width, *height)?;
                    }
                } else {
                    self.set_image_robust(base64, *width, *height)?;
                }
            }
        }
        // Stamp so inject_authorized_paste's wrote_recently gate can pass
        // for image payloads that go through xclip/wl-copy without clipboard_io::write.
        crate::clipboard_io::notify_write();
        self.move_item_to_top(&item.id);
        Ok(())
    }

    /// Legacy in-process paste (write + inject). New code should use
    /// `write_item_to_clipboard` plus the ticketed `finish_paste` command.
    /// paste درون‌فرآیندی قدیمی (نوشتن + تزریق). کدهای جدید باید از
    /// `write_item_to_clipboard` و فرمان بلیت‌دار `finish_paste` استفاده کنند.
    pub fn paste_item(&mut self, item: &ClipboardItem) -> Result<(), String> {
        self.write_item_to_clipboard(item)?;
        self.simulate_paste_action()?;
        Ok(())
    }

    fn set_image_png_bytes(&self, png: &[u8]) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        {
            if crate::session::is_wayland() {
                if self
                    .set_clipboard_external("wl-copy", &["--type", "image/png"], png)
                    .is_ok()
                {
                    return Ok(());
                }
            } else if self
                .set_clipboard_external(
                    "xclip",
                    &["-selection", "clipboard", "-t", "image/png"],
                    png,
                )
                .is_ok()
            {
                return Ok(());
            }
        }
        let img = image::load_from_memory(png).map_err(|e| format!("Image load failed: {e}"))?;
        let rgba = img.to_rgba8();
        let (width, height) = (rgba.width(), rgba.height());
        self.set_image_from_rgba(rgba.into_raw(), width, height)
    }

    fn set_image_from_rgba(&self, bytes: Vec<u8>, width: u32, height: u32) -> Result<(), String> {
        let mut clipboard = get_system_clipboard()?;
        let image_data = ImageData {
            width: width as usize,
            height: height as usize,
            bytes: bytes.clone().into(),
        };
        clipboard.set_image(image_data).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn set_image_robust(&self, base64_str: &str, width: u32, height: u32) -> Result<(), String> {
        let bytes = BASE64
            .decode(base64_str)
            .map_err(|e| format!("Base64 decode failed: {}", e))?;

        #[cfg(target_os = "linux")]
        {
            if crate::session::is_wayland() {
                if self
                    .set_clipboard_external("wl-copy", &["--type", "image/png"], &bytes)
                    .is_ok()
                {
                    return Ok(());
                }
            } else if self
                .set_clipboard_external(
                    "xclip",
                    &["-selection", "clipboard", "-t", "image/png"],
                    &bytes,
                )
                .is_ok()
            {
                return Ok(());
            }
        }

        let img =
            image::load_from_memory(&bytes).map_err(|e| format!("Image load failed: {e}"))?;
        let rgba = img.to_rgba8();
        self.set_image_from_rgba(rgba.into_raw(), width, height)
    }

    fn simulate_paste_action(&self) -> Result<(), String> {
        crate::input_simulator::simulate_paste_keystroke()
    }

    /// Write plain text with a verify-after-write check on the fallback path.
    /// نوشتن متن ساده با تأیید پس از نوشتن در مسیر جایگزین.
    pub fn set_text_robust(&self, text: &str) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        {
            if crate::session::is_wayland() {
                if let Ok(()) = self.set_clipboard_external(
                    "wl-copy",
                    &["--type", "text/plain;charset=utf-8"],
                    text.as_bytes(),
                ) {
                    crate::clipboard_io::write(&crate::clipboard_io::Payload::Text(text)).ok();
                    return Ok(());
                }
            } else if let Ok(()) = self.set_clipboard_external(
                "xclip",
                &["-selection", "clipboard", "-t", "UTF8_STRING"],
                text.as_bytes(),
            ) {
                crate::clipboard_io::notify_write();
                return Ok(());
            }
        }

        let mut clipboard = get_system_clipboard()?;
        clipboard.set_text(text).map_err(|e| e.to_string())?;
        let observed = clipboard.get_text().map_err(|e| e.to_string())?;
        if observed != text {
            return Err("Clipboard text verification returned different data".to_string());
        }
        crate::clipboard_io::notify_write();
        Ok(())
    }

    /// Write HTML + plain-text fallback, preferring native helpers.
    /// نوشتن HTML به‌همراه متن ساده، با اولویت helperهای بومی.
    pub fn set_html_robust(&self, html: &str, plain: &str) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        {
            if crate::session::is_wayland() {
                if let Ok(()) =
                    self.set_clipboard_external("wl-copy", &["--type", "text/html"], html.as_bytes())
                {
                    let _ = self.set_text_robust(plain);
                    return Ok(());
                }
            } else if let Ok(()) = self.set_clipboard_external(
                "xclip",
                &["-selection", "clipboard", "-t", "text/html"],
                html.as_bytes(),
            ) {
                let _ = self.set_text_robust(plain);
                return Ok(());
            }
        }

        let mut clipboard = get_system_clipboard()?;
        clipboard
            .set_html(html, Some(plain))
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Pipe `data` into a clipboard helper (wl-copy / xclip) and verify that
    /// the helper actually took ownership of the selection.
    /// ارسال `data` به helper کلیپ‌بورد (wl-copy / xclip) و تأیید این‌که
    /// helper واقعاً مالکیت selection را گرفته است.
    fn set_clipboard_external(&self, cmd: &str, args: &[&str], data: &[u8]) -> Result<(), String> {
        use std::io::{Read, Write};
        use std::process::{Command, Stdio};

        let owner_before = crate::paste_sync::clipboard_owner();

        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn {}: {}", cmd, e))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(data)
                .map_err(|e| format!("Pipe write error: {}", e))?;
        }

        if cmd == "wl-copy" {
            return wait_for_clipboard_helper_ready(&mut child, cmd);
        }

        let handoff_confirmed = crate::paste_sync::settle_clipboard_handoff(
            owner_before,
            CLIPBOARD_HELPER_READY_TIMEOUT,
        );
        if !handoff_confirmed {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "{} did not acquire the clipboard selection within {:?}",
                cmd, CLIPBOARD_HELPER_READY_TIMEOUT
            ));
        }

        match child.try_wait() {
            Ok(Some(status)) if !status.success() => {
                let mut stderr = String::new();
                if let Some(mut stderr_pipe) = child.stderr.take() {
                    let _ = stderr_pipe.read_to_string(&mut stderr);
                }
                Err(format!(
                    "{} exited with status {}. Stderr: {}",
                    cmd,
                    status,
                    stderr.trim()
                ))
            }
            Ok(_) => {
                thread::spawn(move || {
                    let _ = child.wait();
                });
                Ok(())
            }
            Err(e) => Err(format!("Process status check failed: {}", e)),
        }
    }
}

/// Poll a spawned clipboard helper until it reports readiness (wl-copy
/// stays alive while owning the selection, so exit is *not* expected).
/// نظارت بر helper تا اعلام آمادگی (wl-copy تا وقتی مالک است زنده می‌ماند،
/// پس خروج انتظار نمی‌رود).
fn wait_for_clipboard_helper_ready(
    child: &mut std::process::Child,
    command: &str,
) -> Result<(), String> {
    use std::io::Read;

    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(status)) => {
                let mut stderr = String::new();
                if let Some(mut pipe) = child.stderr.take() {
                    let _ = pipe.read_to_string(&mut stderr);
                }
                return Err(format!(
                    "{} exited with status {}. Stderr: {}",
                    command,
                    status,
                    stderr.trim()
                ));
            }
            Ok(None) if start.elapsed() < CLIPBOARD_HELPER_READY_TIMEOUT => {
                thread::sleep(CLIPBOARD_HELPER_POLL_INTERVAL);
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "{} did not confirm clipboard readiness within {:?}",
                    command, CLIPBOARD_HELPER_READY_TIMEOUT
                ));
            }
            Err(error) => {
                return Err(format!("Failed to inspect {} status: {}", command, error));
            }
        }
    }
}
