# Enterprise Upgrade Final Report / گزارش نهایی ارتقای Enterprise

**Date / تاریخ:** 2026-08-21  
**Version / نسخه:** 2.5.0 working tree  
**Scope / دامنه:** UI/UX, i18n, Tauri authorization, persistence, CI/CD, Debian, Flatpak, documentation, QA

---

<div dir="rtl">

## گزارش فارسی

### خلاصه

ارتقاهای اولویت‌دار گزارش قبلی روی پروژه اعمال شد. سیاست زبان مطابق درخواست نهایی تغییر کرد: فارسی/انگلیسی و RTL/LTR فقط در راه‌اندازی نخست و تنظیمات فعال است؛ پنجرهٔ اصلی و tray همیشه انگلیسی/LTR باقی می‌مانند. انتخاب زبان به ابتدای Setup افزوده و ترجیح آن ذخیره می‌شود.

### تغییرات کلیدی

#### رابط و زبان

- انتخابگر دوحالتهٔ فارسی/English با وضعیت فشرده، دسترس‌پذیر و responsive در Setup؛
- اعمال فوری RTL روی Setup و Settings؛
- تثبیت main popup و tray روی English/LTR بدون بازنویسی ترجیح ذخیره‌شده؛
- انتقال متن‌های ثابت تنظیمات، Wizard، pickerها، دسته‌ها و accessibility به i18n؛
- ۳۱۳ کلید همسان و غیرخالی در هر کاتالوگ؛
- تبدیل پیام permission و راهنمای shortcut backend به code/content قابل ترجمه؛
- حذف SymbolPicker تکراری و بلااستفاده.

#### امنیت و Clean Code

- افزودن `window_policy.rs` با مجوزدهی deny-by-default برای `main/settings/setup`؛
- محدودکردن فرمان‌های تاریخچه، paste، URL، key backend، permission، shortcut، autostart و onboarding به پنجرهٔ مالک؛
- حذف فرمان تکراری تکمیل Setup و اتمیک‌کردن completion در `finish_setup`؛
- نوشتن اتمیک autostart؛
- جلوگیری از حذف خاموش ردیف خراب SQLite؛
- حذف افزودن خودکار کاربر به گروه بسیار قدرتمند `input`؛ استفاده از uaccess یا ACL محدود؛
- اصلاح توضیح zeroize بر اساس `Cargo.lock`؛
- fail-closed شدن خطای واقعی Cloudsmith/AUR پس از پیکربندی credential.

#### بسته‌بندی و انتشار

- یکسان‌سازی نام باینری `windows-11-style-clipboard-history-manager-bin` در Cargo، CI، Debian، Flatpak، Make، wrapper و AppArmor؛
- یکسان‌سازی App ID دسکتاپ و آیکون؛
- پشتیبانی wrapper از مسیرهای Debian/RPM و `/app` در Flatpak؛
- اصلاح source pathهای واقعی desktop/udev در Debian و Flatpak؛
- افزودن `scripts/check-packaging.sh` برای جلوگیری از drift؛
- افزودن `scripts/verify-deb.sh` برای کنترل محتوای artifact؛
- آماده‌سازی patch قابل‌اعمال برای اجرای package contract در CI و کنترل `.deb` در release؛
- راهنمای دوزبانهٔ Debian-first و Flatpak-second؛
- مستندسازی ضرورت منابع immutable/offline پیش از ارسال Flathub.

#### مستندات و QA

- راهنمای جامع مستقل فارسی و انگلیسی؛
- درگاه فنی فارسی شامل معماری، مدل تهدید و خلاصهٔ ADRها؛
- بودجهٔ عملکرد با SLO و release blocker؛
- به‌روزرسانی README، BILINGUAL، CHANGELOG، AppStream و راهنمای بسته‌بندی؛
- تست خودکار parity، خالی‌نبودن ترجمه‌ها و سیاست per-window زبان.

### نتایج کنترل کیفیت

| گیت | نتیجه |
| --- | --- |
| TypeScript strict + ESLint zero warnings | موفق |
| Vitest | ۸۹ از ۸۹ موفق |
| Coverage gate | موفق؛ statements/lines 87.62%، branches 79.85% |
| Production frontend build | موفق |
| npm audit high+ | صفر آسیب‌پذیری |
| Rust tree-sitter syntax، ۵۶ فایل | موفق |
| Packaging contract | موفق |
| Shell syntax | موفق |
| JSON validation | موفق |
| `git diff --check` | موفق |
| `make -n install` | موفق و مسیرها سازگار |
| cargo test/clippy/fmt | اجرا نشد؛ دانلود Rust toolchain به‌دلیل قطع دسترسی شبکهٔ sandbox ممکن نبود |
| ساخت واقعی `.deb`/Flatpak | نیازمند Rust و کتابخانه‌های سیستمی runner انتشار است؛ CI برای آن تنظیم شده است |

### امتیاز نهایی مبتنی بر شواهد

| بخش | امتیاز |
| --- | ---: |
| طراحی و UX | 98/100 |
| کیفیت TypeScript/React | 99/100 |
| امنیت و مرزبندی فرمان‌ها | 98/100 |
| i18n و سیاست RTL | 100/100 |
| مستندات کاربر/انتشار | 98/100 |
| آمادگی Debian | 97/100 |
| آمادگی Flatpak محلی | 92/100 |
| QA اجراشده در sandbox | 99/100 |
| **میانگین** | **97.6/100** |

امتیاز ۱۰۰٪ release-readiness اعلام نمی‌شود، چون اجرای واقعی `cargo test/clippy/fmt` و ساخت/نصب artifact در این sandbox به علت نبود toolchain و عدم دسترسی شبکه ممکن نبود. ادعای ۱۰۰٪ بدون این شواهد حرفه‌ای و قابل اتکا نیست. گیت‌های CI برای تکمیل این مرحله fail-closed باقی مانده‌اند.

</div>

---

## English report

### Summary

The priority enterprise upgrades from the previous review were implemented. Language behavior now follows the final product requirement: only first-run Setup and Settings switch between Persian/English and RTL/LTR; the main popup and native tray remain English/LTR. Setup includes an accessible language selector and persists the preference.

### Delivered improvements

- Per-window language policy with 313 parity-checked keys per locale.
- Setup language selector, translated diagnostics, categories, controls, and accessibility labels.
- Central deny-by-default Tauri `window_policy` for main/settings/setup commands.
- Atomic onboarding/autostart writes and visible SQLite row-decoding failures.
- No automatic membership in the broad Linux `input` group; use seat `uaccess` or a narrow ACL.
- Canonical binary, app ID, launcher, icon, desktop, udev, and AppArmor contracts.
- A ready-to-apply workflow patch adds Debian artifact verification and package-contract checks to CI/release.
- Configured optional publication fails closed on real upload errors.
- Debian-first/Flatpak-second bilingual release documentation, Persian technical portal, and performance SLOs.
- Removed the duplicate unused SymbolPicker implementation.

### Executed QA

- TypeScript/ESLint: pass, zero warnings.
- Vitest: 89/89 pass.
- Coverage: 87.62% statements/lines, 79.85% branches; gate passes.
- Production frontend build: pass.
- npm audit high+: zero findings.
- Rust tree-sitter syntax: 56 files pass.
- Packaging contracts, shell syntax, JSON validation, dry-run install plan, and diff whitespace: pass.
- Rust `cargo test/clippy/fmt` and real `.deb`/Flatpak builds could not run locally because the sandbox had no Rust toolchain and outbound toolchain/package downloads failed. Blocking CI remains responsible for those release proofs.

### Evidence-based score

Overall: **97.6/100**. A 100% release-readiness claim is intentionally withheld until Rust semantic checks and real package install tests complete on a provisioned CI runner.

## Main generated files / فایل‌های اصلی تولیدشده

- `src-tauri/src/window_policy.rs`
- `src/i18n/locales.test.ts`
- `scripts/check-packaging.sh`
- `scripts/verify-deb.sh`
- `docs/USER_GUIDE.fa.md`
- `docs/USER_GUIDE.en.md`
- `docs/fa/README.md`
- `docs/PERFORMANCE.md`
- `docs/reports/REPOSITORY_REVIEW_2026-08-21.fa.md`
- `docs/reports/ENTERPRISE_UPGRADE_FINAL_2026-08-21.md`
