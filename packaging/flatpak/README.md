# 📦 Flatpak build & deployment guide / راهنمای ساخت و استقرار فلت‌پک

> This directory holds everything needed to build and publish the Flatpak of
> **Windows 11 Style Clipboard History Manager** (`io.github.mahdi-arts.clipboard-history`).
> این پوشه شامل همهٔ آنچه برای ساخت و انتشار فلت‌پک پروژه لازم است می‌باشد.

---

## 1. Files / فایل‌ها

| File | Purpose / هدف |
| --- | --- |
| `io.github.mahdi-arts.clipboard-history.yml` | Local bootstrap manifest; generate immutable npm/Cargo sources before Flathub submission / مانیفست محلی؛ پیش از Flathub منابع immutable تولید شوند |
| `io.github.mahdi-arts.clipboard-history.metainfo.xml` | AppStream metadata (name, screenshots, `<releases>`) |
| `build.sh` | One-command local build helper / راه‌انداز ساخت محلی |

---

## 2. Local build / ساخت محلی

```bash
# 1) Install the runtime + extensions (once)
#     نصب runtime و افزونه‌ها (یک‌بار)
flatpak install --user flathub \
  org.gnome.Sdk//48 org.gnome.Platform//48 \
  org.freedesktop.Sdk.Extension.rust-stable \
  org.freedesktop.Sdk.Extension.node20

# 2) Build + install + launch (from the repo root)
#     ساخت + نصب + اجرا (از ریشهٔ مخزن)
./packaging/flatpak/build.sh
```

The helper wraps `flatpak-builder` with the official manifest and a
`.flatpak-builder` cache directory (git-ignored, see repo root `.gitignore`).
اسکریپت کمکی، `flatpak-builder` را با مانیفست رسمی و پوشهٔ کش
`.flatpak-builder` اجرا می‌کند.

### What the sandbox allows / سندباکس چه اجازه‌هایی دارد

| Permission | Granted? | Why / چرا |
| --- | --- | --- |
| `--socket=wayland` + `--socket=fallback-x11` | ✅ | windowing / نمایش پنجره |
| `--share=ipc` | ✅ | X11 shared memory / حافظهٔ مشترک X11 |
| Desktop + Settings portals | ✅ | theme detection / تشخیص تم |
| `org.kde.StatusNotifierWatcher` | ✅ | system tray / تری سیستم |
| XDG data/config dirs (`create`) | ✅ | history DB + settings / دیتابیس و تنظیمات |
| `--device=all` (`/dev/uinput`) | ❌ default | paste simulation needs a user override |
| `--share=network` | ❌ default | optional GIF search only |

```bash
# Diagnostic-only, broad device override — NOT recommended for normal use.
# فقط برای عیب‌یابی؛ دسترسی گسترده به دستگاه‌ها و برای استفادهٔ عادی توصیه نمی‌شود.
flatpak override --user --device=all io.github.mahdi-arts.clipboard-history

# Optional GIF search
# جستجوی اختیاری GIF
flatpak override --user --share=network io.github.mahdi-arts.clipboard-history
```

---

## 3. Flathub publication / انتشار در Flathub

1. Replace the local `type: dir` source with a pinned Git commit/tag and generate offline npm/Cargo source modules (for example with `flatpak-node-generator` and `flatpak-cargo-generator`). Never submit a network-dependent build. / منبع محلی `type: dir` را با commit/tag پین‌شده جایگزین و ماژول‌های آفلاین npm/Cargo را با generatorهای Flatpak تولید کنید؛ build وابسته به شبکه را ارسال نکنید.
2. Fork/PR the `flathub/io.github.mahdi-arts.clipboard-history` repository with that immutable manifest. / مانیفست immutable را به مخزن Flathub پروژه ارسال کنید.
3. Bump the `<releases>` entry in the metainfo for every version (date +
   version). / در هر نسخه رکورد `<releases>` متادیتا را به‌روز کنید.
4. Attach screenshots (1280×800 or larger) and a 128×128 icon.
   / اسکرین‌شات (۱۲۸۰×۸۰۰ یا بزرگ‌تر) و آیکون ۱۲۸×۱۲۸ پیوست کنید.
5. Flathub's CI will build against the pinned runtime (GNOME 48); the
   manifest must stay reproducible (no network in the sandbox at runtime).
   / CI فلت‌هاب با runtime پین‌شده (GNOME 48) بیلد می‌گیرد؛ مانیفست باید
   قابل بازتولید بماند (بدون شبکه در زمان اجرا).
6. CI in this repository already validates the manifest and metainfo with
   `flatpak-builder-lint` (Flathub rules) on every push — keep it green
   before submitting. / CI همین مخزن مانیفست و متادیتا را با
   `flatpak-builder-lint` (قواعد فلت‌هاب) در هر پوش اعتبارسنجی می‌کند —
   پیش از ارسال، سبزش نگه دارید.

### Version bump checklist / چک‌لیست نسخه

- [ ] `app-id` unchanged / بدون تغییر
- [ ] `runtime-version` supported by Flathub / پشتیبانی‌شده در فلت‌هاب
- [ ] metainfo `<releases>` entry added / رکورد انتشار اضافه شود
- [ ] screenshots current / اسکرین‌شات‌ها به‌روز

---

## 4. Known limitations / محدودیت‌های شناخته‌شده

1. Paste (Ctrl+V injection) requires `--device=all` — Flathub's policy does
   not grant `/dev/uinput` by default. / paste تا قبل از `--device=all`
   غیرفعال است — سیاست فلت‌هاب `/dev/uinput` را پیش‌فرض نمی‌دهد.
2. Global shortcuts (`Super+V`) cannot be registered from inside the sandbox.
   Use `Ctrl+Alt+V` or the native packages. / میانبر سراسری (`Super+V`) از
   داخل سندباکس ثبت نمی‌شود؛ از `Ctrl+Alt+V` یا بسته‌های بومی استفاده کنید.
3. udev rules are not installed (native channels only).
   / قوانین udev نصب نمی‌شود (فقط کانال‌های بومی).
