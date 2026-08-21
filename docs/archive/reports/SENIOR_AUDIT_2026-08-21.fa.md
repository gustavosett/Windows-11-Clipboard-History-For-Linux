# گزارش ممیزی ارشد فول‌استک/امنیت — Senior Full-Stack & Security Audit

**پروژه:** Windows 11 Style Clipboard History Manager (Tauri v2 + React 19 + Rust)
**نسخه:** 2.5.0 · **تاریخ ممیزی:** 2026-08-21 · **کامیت:** `4df4638` (شاخهٔ arena)
**ممیز:** Arena.ai Agent Mode — بازبینی ایستا + اجرای واقعی lint/test فرانت‌اند

---

## خلاصهٔ اجرایی

پروژه یک مدیر کلیپ‌بورد دسکتاپی برای لینوکس است که از نظر بلوغ مهندسی **بالاتر از میانگین پروژه‌های متن‌باز هم‌اندازه** قرار دارد: رمزنگاری at-rest با AEAD و رفتار fail-closed، مدل تهدید مکتوب، ۸ سند ADR، زنجیرهٔ تأمین سخت‌گیرانه (SHA-pinned actions، SBOM، SLSA provenance)، و مرز IPC با کنترل دسترسی پنجره‌ای. در اجرای واقعی این ممیزی: `tsc + ESLint` با **صفر خطا/هشدار**، **۸۹/۸۹ تست فرانت پاس** با پوشش ۸۷.۶٪ statements، و `scripts/check-packaging.sh` با موفقیت اجرا شد.

دو یافتهٔ عملیاتی مهم (هر دو باقی‌مانده از تغییر نام اخیر پروژه):
1. **`.github/workflows/release.yml` هنوز نام قدیمی مخزن/بسته‌ها را دارد** → یادداشت‌های انتشار با نام فایل اشتباه و شکستن کانال AUR در انتشار بعدی.
2. **بستهٔ RPM فاقد نگاشت `files`** است → قوانین udev، wrapper و پروفایل AppArmor وارد `.rpm` نمی‌شوند (در `.deb` هستند).

---

## ۱. معماری و ساختار — ۹/۱۰

### نقاط قوت
- **دو لایهٔ مشخص:** React 19 + TypeScript در WebView؛ هستهٔ Rust/Tauri v2 (~۱۲هزار خط، ۶۰+ ماژول تک‌مسئولیتی: `clipboard_manager/` با persistence/dedup/history_access/clipboard_write، `linux_shortcut_manager/` برای ۸ دسکتاپ، `shortcut_conflict_detector/`).
- **انتخاب تکنولوژی دقیق:** `arboard` با `wayland-data-control`، `x11rb` با `xfixes` برای بیدارباش رویدادمحور، SQLite/WAL با upsert تدریجی به‌جای بازنویسی JSON، `chacha20poly1305` با `zeroize`.
- **جداسازی قابلیت شبکه:** `reqwest` پشت feature `gif-search`؛ باینری پیش‌فرض هیچ کلاینت HTTP خروجی ندارد.
- **پروفایل release سخت‌گیرانه:** `lto=true`, `opt-level="z"`, `codegen-units=1`, `panic=abort`.
- ۸ سند ADR، Threat Model، بودجهٔ عملکرد، قرارداد CI مکتوب.

### weaknesses
- چند فایل بزرگ: `input_simulator.rs` (۷۵۱)، `history_crypto.rs` (۷۴۴)، `commands.rs` (۵۶۳)، `window_controller.rs` (۴۷۱) — قابل نگهداری اما در آستانهٔ شکست به adapter/policy.
- تست E2E واقعی X11/Wayland و تست نصب بسته در CI وجود ندارد (فقط smoke باینری زیر Xvfb).

## ۲. کیفیت کد — ۹/۱۰

- ESLint type-aware + `tsc --noEmit` با `--max-warnings 0` → **اجرای ممیزی: پاس، صفر هشدار**.
- ۸۹ تست فرانت (Vitest + Testing Library) → **۸۹/۸۹ پاس**؛ پوشش statements ۸۷.۶٪ (utils/services ~۹۵٪).
- ۶۲ تست Rust شامل تست‌های خصمانهٔ رمزنگاری (tamper bit-flip، تکرار nonce، کلید بیگانه، fail-closed marker).
- خطاهای ساختاریافتهٔ `AppError` با `thiserror`؛ کامنت‌های دوزبانهٔ «چرا» نه «چه»؛ صفر TODO/FIXME.
- SOLID رعایت‌شده: `net_policy.rs` منبع واحد سیاست IP برای `ssrf.rs` و `open_url.rs`؛ commands فقط thin wrapper روی دامنه.
- نقطه‌ضعف جزئی: پوشش توابع `icons.tsx` (۴۲٪) و برخی هوک‌ها (۷۵٪).

## ۳. امنیت و شبکه — ۹.۵/۱۰

### کنترل‌های برجسته
| حوزه | پیاده‌سازی |
|---|---|
| ذخیره‌سازی | ChaCha20-Poly1305 فیلدای (magic `W11E1` + nonce تصادفی)، کلید `0600` یا Secret Service، marker ی `history.key.check` با پذیرش fail-closed کلید، zeroize حتی در panic |
| IPC | قابلیت‌ها **به‌تفکیک پنجره** (paste/میانبر فقط `main`؛ تغییر تنظیمات فقط از `settings` با `require_settings_window`)، `withGlobalTauri: false`، CSP بدون `unsafe-eval`، بلیت یک‌بارمصرف ۵ ثانیه‌ای `PasteTicket` برای تزریق Ctrl+V، clamp سمت سرور `limit` به ۲۰۰ |
| SSRF | HTTPS-only، allowlist میزبان، رد IP مستقیم/credentials، رد رنج‌های خصوصی/CGNAT/metadata (v4+v6)، **DNS pinning** برای رفع TOCTOU/rebinding، بدون redirect |
| حریم خصوصی | فیلتر اسرار (PEM/JWT/`ghp_`/`AKIA…`/`password=`)، رد پنجره‌های پسوردمنیجر/incognito روی X11، تاریخچه ۰۶۰۰، لاگ ۰۷۰۰ |
| زنجیرهٔ تأمین | پین SHA کامل actionها، cargo audit/deny + npm audit مسدودکننده، SHA256SUMS اجباری در نصاب، SBOM SPDX لكل آرتیفکت، SLSA provenance، AUR fail-closed با `AUR_KNOWN_HOSTS` |
| تزریق کلید | دستگاه uinput ماندگار + verification وضعیت کلید XTest قبل از release؛ ابهام paste → عدم تکرار |

### ریسک‌های باقی‌مانده
- دسترسی `/dev/uinput` ذاتاً یک قابلیت پرامتیج است (به‌درستی مستند شده؛ AppArmor در حالت complain).
- Wayland هویت پنجرهٔ فوکوس نمی‌دهد → exclusion اسرار فقط X11 (محدودیت ذاتی، مستند).
- تهدید T1 (بدافزار هم‌کاربر) خارج از مدل کنترل اپ است و صادقانه در Threat Model آمده.

## ۴. مستندات — ۹/۱۰
- README دوزبانه با badges، جدول ویژگی‌ها، مسیر نصب چند دیسترو؛ `USER_GUIDE.fa/en`، `ARCHITECTURE.md`، `THREAT_MODEL.md`، `PERFORMANCE.md`، `CI.md`، `packaging/README.md` + `DEPLOYMENT.md`، ۸ ADR، CHANGELOG، CONTRIBUTING، SECURITY.md با مسیر گزارش آسیب‌پذیری، قالب issue/PR، dependabot، CODEOWNERS.
- کامنت‌های درون‌کدی دوزبانه و توضیح‌دهندهٔ «چرا».
- کسر امتیاز: ته‌مانده‌های نام قدیمی در متن release notes (تولید CI) و نیاز به ترجمهٔ مستقل برخی اسناد عمیق.

## ۵. امتیازدهی

| بخش | نمره / ۱۰ |
|---|---|
| کیفیت کد و معماری | **۹.۰** |
| امنیت | **۹.۵** |
| مستندات | **۹.۰** |
| قابلیت توسعه (Scalability) | **۸.۰** |
| **میانگین** | **۸.۹ / ۱۰** |

> Scalability برای اپ دسکتاپ یعنی: صفحه‌بندی IPC (ADR-0007)، سقف ۲۰۰۰ آیتم، upsert تدریجی، virtualization لیست با `react-window` — همه حاضرند؛ کسر امتیاز به‌خاطر نبود benchmark خودکار و نبود ماتریس تست نصب/اجرا روی دیستروهای واقعی.

## ۶. آمادگی انتشار `.deb` / `.rpm` / `.AppImage`

| قالب | وضعیت | توضیح |
|---|---|---|
| `.deb` | ✅ آماده | `files` کامل (wrapper، قوانین udev، desktop، آیکون‌ها، AppArmor)، postinst/postrm، وابستگی‌ها، ساخت x86_64+aarch64 در CI |
| `.AppImage` | ✅ آماده | ساخت در CI؛ محدودیت ذاتی (بدون udev/AppArmor سیستمی) با راهنمای install.sh پوشش داده شده |
| `.rpm` | ⚠️ تقریباً آماده | Tauri v2 از `files` در rpm پشتیبانی می‌کند اما در `tauri.conf.json` نگاشت نشده → قوانین udev، wrapper و AppArmor وارد بستهٔ Fedora نمی‌شود (کاربر NVIDIA بدون DMA-BUF workaround می‌ماند و `/dev/uinput` ACL خودکار ندارد) |

### الزامات پیش از تگ بعدی (ترتیب اولویت)
1. **رفع نام‌های قدیمی در `release.yml`:** `REPO` (استفاده‌نشده)، URLهای release notes (`win11-clipboard-history_*` ≠ نام واقعی آرتیفکت)، job AUR (`git clone win11-clipboard-history-bin` + `cp aur/win11-clipboard-history-bin.install` که دیگر وجود ندارد و با `|| true` بی‌صدا رد می‌شود، در حالی‌که `aur/PKGBUILD` اکنون `windows-11-style-clipboard-history-manager-bin` است → تعارض و شکست makepkg)، و `.SRCINFO` دست‌نویس.
2. **افزودن `files` به بخش rpm** (معادل بخش deb).
3. توصیه: اجرای یک انتشار آزمایشی `vX.Y.Z-rc.1` برای اعتبارسنجی end-to-end کانال‌ها.

---

### روش ممیزی
بازبینی ایستای کل درخت (`src-tauri/src`، `src/`، `.github/workflows`، `packaging/`، `scripts/`، `docs/`) + اجرای واقعی: `npm ci`، `npm run lint` (پاس)، `npm run test:coverage` (۸۹/۸۹، ۸۷.۶٪)، `scripts/check-packaging.sh` (پاس). تست‌های Rust (۶۲ مورد) در این sandbox فاقد toolchain زنجیرهٔ GTK/WebKit قابل اجرا نبودند و به CI محول‌اند.
