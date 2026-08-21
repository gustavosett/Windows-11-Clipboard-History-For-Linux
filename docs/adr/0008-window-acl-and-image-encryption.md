# ADR 0008 — Window ACL for paste + encrypted image files

- **Status:** Accepted
- **Date:** 2026-08-21
- **Deciders:** Maintainers

## Context / زمینه

<div dir="rtl">

دو شکاف در مدل تهدید باقی مانده بود:

1. فرمان‌های `paste_item` / `paste_text` / `finish_paste` از هر webview
   (از جمله Settings و Setup) قابل فراخوانی بودند. بلیت paste در مسیر
   `inject_authorized_paste` توسط همان تابع صادر و مصرف می‌شد (چک ظاهری).
2. ستون‌های متنی SQLite رمز می‌شدند اما فایل‌های PNG در
   `images/` به‌صورت خام روی دیسک می‌ماندند.

</div>

Two gaps remained in the threat model:

1. Paste commands were callable from every webview (including Settings and
   Setup). The paste ticket inside `inject_authorized_paste` was issued and
   consumed by the same function (a no-op check).
2. SQLite text columns were encrypted, but PNG files under `images/` sat
   on disk in the clear.

## Decision / تصمیم

<div dir="rtl">

- قابلیت‌های Tauri به ازای پنجره جدا شدند (`default` = main،
  `settings`، `setup`). پنجره‌های تنظیمات/نصب میانبر سراسری ثبت نمی‌کنند.
- فرمان‌های تزریق کلیدstroke فقط از پنجرهٔ `main` پذیرفته می‌شوند
  (بررسی `WebviewWindow::label` در Rust).
- `inject_authorized_paste` دیگر بلیت صادر نمی‌کند؛ فقط
  `wrote_recently(5s)` را پس از یک نوشتن واقعی کلیپ‌بورد چک می‌کند.
  بلیت یک‌بارمصرف فقط برای مسیر GIF (`paste_gif_from_url` → `finish_paste`)
  باقی می‌ماند.
- فایل تصویر با همان پاکت `W11E1` + ChaCha20-Poly1305 ذخیره می‌شود؛
  PNGهای قدیمی بدون پاکت همچنان خوانده می‌شوند (سازگاری). حذف فایل یک
  گذر صفرنویسی دارد.

</div>

- Tauri capabilities are split per window (`default` = main, `settings`,
  `setup`). Settings/setup cannot register global shortcuts.
- Keystroke-injection commands accept only the `main` window
  (`WebviewWindow::label` check in Rust).
- `inject_authorized_paste` no longer issues a ticket; it only checks
  `wrote_recently(5s)` after a real clipboard write. The one-shot ticket
  remains for the GIF path (`paste_gif_from_url` → `finish_paste`).
- Image files use the same `W11E1` + ChaCha20-Poly1305 envelope; legacy
  plaintext PNGs still decrypt as a pass-through. Deletes overwrite with
  zeros once before unlink.

## Consequences / پیامدها

- XSS in the settings webview can no longer inject Ctrl+V.
- Same-UID attackers still see the key (file or Secret Service); image
  ciphertext without the key is useless.
- Migration of existing image directories is lazy (plaintext PNG remains
  readable until the next rewrite).
