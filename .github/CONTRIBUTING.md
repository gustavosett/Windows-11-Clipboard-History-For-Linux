# 🤝 Contributing to Windows 11 Style Clipboard History Manager  /  مشارکت در پروژه

<div dir="rtl">

## 🌟 خوش آمدید

از اینکه به فکر مشارکت در این پروژه هستید، صمیمانه سپاسگزاریم! هر مشارکتی، چه کوچک و چه بزرگ، ارزشمند است.

## 📋 فهرست

- [قوانین رفتاری](#قوانین-رفتاری)
- [شروع کار](#شروع-کار)
- [تنظیم محیط توسعه](#تنظیم-محیط-توسعه)
- [ایجاد تغییرات](#ایجاد-تغییرات)
- [فرآیند درخواست Pull Request](#فرآیند-درخواست-pull-request)
- [راهنمای سبک کدنویسی](#راهنمای-سبک-کدنویسی)
- [گزارش باگ](#گزارش-باگ)
- [پیشنهاد ویژگی](#پیشنهاد-ویژگی)
- [مسائل ترجمه (i18n)](#مسائل-ترجمه-i18n)
- [سوالات](#سوالات)

## قوانین رفتاری

این پروژه و همه شرکت‌کنندگان در آن متعهد به ایجاد محیطی گرم و فراگیر هستند. لطفاً در همه تعاملات محترم و سازنده باشید.

## شروع کار

1. **مخزن را Fork کنید** در GitHub
2. **فورک خود را به صورت محلی Clone کنید**:
   ```bash
   git clone https://github.com/USERNAME/Windows-11-Style-Clipboard-History-Manager.git
   cd Windows-11-Style-Clipboard-History-Manager
   ```
3. **آدرس مخزن اصلی را به عنوان upstream اضافه کنید**:
   ```bash
   git remote add upstream https://github.com/Mahdi-Arts/Windows-11-Style-Clipboard-History-Manager.git
   ```

## تنظیم محیط توسعه

### پیش‌نیازها

- **Rust 1.77+** (نسخهٔ دقیق در `rust-toolchain.toml`)
- **Node.js ≥ 20.19** (کف Vite 7 — `.nvmrc` نسخهٔ ۲۰ را پین می‌کند)
- وابستگی‌های سیستمی

```bash
# نصب وابستگی‌های سیستمی
make deps

# نصب Rust و Node.js (در صورت نیاز)
make rust
make node
source ~/.cargo/env

# بررسی نصب
make check-deps
```

### اجرا در حالت توسعه

```bash
# نصب وابستگی‌های npm
npm install

# شروع سرور توسعه با Hot Reload
make dev
```

## ایجاد تغییرات

1. **یک شعبه جدید از `master` ایجاد کنید**:
   ```bash
   git checkout -b feature/your-feature-name
   # یا
   git checkout -b fix/your-bug-fix
   ```

2. **تغییرات خود را اعمال کنید** و محلی تست کنید

3. **لینترها و فرمترها را اجرا کنید**:
   ```bash
   make lint
   make format
   ```

4. **تغییرات خود را commit کنید** با پیام توصیفی:
   ```bash
   git commit -m "feat: add amazing new feature"
   # یا
   git commit -m "fix: resolve clipboard paste issue on Wayland"
   ```

### قرارداد پیام Commit

ما از [Conventional Commits](https://www.conventionalcommits.org/) پیروی می‌کنیم:

- `feat:` - ویژگی جدید
- `fix:` - رفع باگ
- `docs:` - مستندات
- `style:` - تغییرات ظاهری
- `refactor:` - بازنویسی کد
- `perf:` - بهبود کارایی
- `test:` - تست‌ها
- `chore:` - وظایف نگهداری
- `i18n:` - تغییرات ترجمه و زبان

## فرآیند درخواست Pull Request

1. **شعبه خود را به‌روز کنید**:
   ```bash
   git fetch upstream
   git rebase upstream/master
   ```

2. **شعبه خود را به فورک Push کنید**:
   ```bash
   git push origin feature/your-feature-name
   ```

3. **یک Pull Request در GitHub ایجاد کنید**

4. **منتظر بررسی باشید** - maintainerها ممکن است تغییراتی درخواست کنند

### الزامات PR

- [ ] کد مطابق با راهنمای سبک پروژه است
- [ ] همه لینترها پاس می‌شوند (`make lint`)
- [ ] `cargo test --lib` برای تست‌های persistence، رمزنگاری و paste ticket
- [ ] تغییرات محلی تست شده‌اند: `npm test` و `npm run test:coverage` (گیت پوشش)
- [ ] برای تغییرات Rust: `cargo fmt --check` و `cargo clippy -- -D warnings`
- [ ] مستندات به‌روز شده است (در صورت نیاز)
- [ ] برای تغییرات i18n، هر دو فایل `fa.json` و `en.json` به‌روز شده‌اند
- [ ] تغییرات روی X11 و Wayland تست شده (در صورت امکان)
- [ ] برای تغییرات بسته‌بندی/انتشار: `make packaging` سبز است (نام‌های رسمی،
      همگامی نسخه‌ها، برابری deb/rpm)

## راهنمای سبک کدنویسی

### TypeScript/React

- از کامپوننت‌های تابعی با hooks استفاده کنید
- از الگوهای موجود پیروی کنید
- از TypeScript types استفاده کنید (از `any` پرهیز کنید)
- از نام‌های معنادار استفاده کنید
- برای استخراج رشته‌ها از `useTranslation()` و `t()` استفاده کنید

### Rust

- از Rust idioms و best practices پیروی کنید
- از `cargo fmt` برای فرمت کردن استفاده کنید
- همه هشدارهای `clippy` را رفع کنید
- توابع و ماژول‌های عمومی را مستند کنید
- برای خطاها از `AppError` / `thiserror` استفاده کنید (نه `Result<(), String>`)
- از `tracing` به جای `eprintln!` استفاده کنید
- از `crate::fs_atomic::write_atomic` برای نوشتن فایل استفاده کنید
- از `crate::clipboard_io` برای عملیات کلیپ‌بورد استفاده کنید

### کامنت‌های دوزبانه (سیاست پروژه)

- کامنت‌های **سربرگ و مستندات (`///`) آیتم‌های عمومی** باید دوزبانه
  (انگلیسی + فارسی) باشند — الگوی موجود در `history_crypto.rs` را ببینید.
- کامنت‌های «چرا» ترجیح داده می‌شوند؛ کامنت‌های «چه» فقط وقتی کد
  خوداظهار نباشد.
- پیام‌های commit مطابق Conventional Commits باشند (hook `commit-msg`
  این را الزامی می‌کند).

### Git Hookها

```bash
make hooks   # نصب pre-commit و commit-msg
```

- `pre-commit`: روی فایل‌های stage شده ESLint + tsc و `cargo fmt --check` اجرا می‌کند.
- `commit-msg`: قالب Conventional Commits را الزامی می‌کند.

### سیاست مستندات و گزارش‌ها

- مستنداتِ زندهٔ کاربر (راهنماها، `ARCHITECTURE`، `ADR`ها،
  `THREAT_MODEL`، `PERFORMANCE`، `CI`) در سطح اول `docs/` می‌مانند.
- گزارش‌های بازبینی/QA مربوط به یک نشست، در `docs/archive/reports/`
  با پسوند تاریخ ISO قرار می‌گیرند (یا بهتر: در شرح خود PR).
  فهرست: [`docs/reports/README.md`](../docs/reports/README.md).
- ورک‌فلوها در `.github/workflows/` زندگی می‌کنند (منبع حقیقت واحد)؛ تا
  فعال‌سازی نهایی، پچ جاریِ معتبر `docs/patches/hardened-ci-workflows.patch`
  است و پچ‌های قدیمی در `docs/archive/patches/` آرشیو شده‌اند — آن‌ها را
  اعمال نکنید.

### وابستگی‌های جدید

- وابستگی Rust جدید باید مجوز آن در allow-list فایل `src-tauri/deny.toml`
  باشد وگرنه `cargo deny check` (گیت CI) شکست می‌خورد.

### CSS/Tailwind

- از Tailwind utility classes استفاده کنید
- از سیستم طراحی Windows 11 پیروی کنید
- از حالت تاریک و روشن پشتیبانی کنید
- برای RTL از ویژگی `dir` استفاده کنید

### i18n (ترجمه)

- همه رشته‌های UI باید در فایل‌های `src/locales/en.json` و `src/locales/fa.json` باشند
- رشته جدید را به هر دو زبان اضافه کنید
- از `t('key.path')` در کد استفاده کنید
- از `useLanguage()` hook برای RTL استفاده کنید

## مسائل ترجمه (i18n)

### اضافه کردن رشته جدید

1. به `src/locales/en.json` و `src/locales/fa.json` اضافه کنید
2. از کلیدهای تو در تو (nested) با ساختار مناسب استفاده کنید
3. از مقدار `t('key')` در کامپوننت‌ها استفاده کنید

### اضافه کردن زبان جدید

1. فایل `src/locales/{lang}.json` را ایجاد کنید
2. در `src/i18n/config.ts` آن را import کنید
3. در `UserSettings::set_language` در Rust (فایل `user_settings.rs`) آن را اضافه کنید

## گزارش باگ

قبل از گزارش باگ:

1. **مسائل موجود را جستجو کنید** تا از تکراری نبودن مطمئن شوید
2. **آخرین نسخه را امتحان کنید** - ممکن است باگ رفع شده باشد
3. **اطلاعات جمع‌آوری کنید**:
   - سیستم عامل و نسخه
   - محیط دسکتاپ
   - سرور نمایش (X11/Wayland)
   - مراحل بازتولید باگ
   - پیام‌های خطا

## پیشنهاد ویژگی

قبل از پیشنهاد ویژگی:

1. **بررسی کنید با هدف پروژه هماهنگ است**
2. **مسائل موجود را جستجو کنید**
3. **implementation را در نظر بگیرید**

## سوالات؟

می‌توانید [یک discussion باز کنید](https://github.com/Mahdi-Arts/Windows-11-Style-Clipboard-History-Manager/discussions) برای سوالات و ایده‌ها.

---

</div>

---

# English Version

## 🌟 Welcome

First off, thank you for considering contributing to Windows 11 Style Clipboard History Manager! 🎉 Every contribution, big or small, is valuable.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Pull Request Process](#pull-request-process)
- [Style Guidelines](#style-guidelines)
- [Reporting Bugs](#reporting-bugs)
- [Suggesting Features](#suggesting-features)
- [i18n / Translation](#i18n--translation)
- [Questions](#questions)

## Code of Conduct

This project and everyone participating in it is governed by our commitment to creating a welcoming and inclusive environment. Please be respectful and constructive in all interactions.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR-USERNAME/Windows-11-Style-Clipboard-History-Manager.git
   cd Windows-11-Style-Clipboard-History-Manager
   ```
3. **Add the upstream remote**:
   ```bash
   git remote add upstream https://github.com/Mahdi-Arts/Windows-11-Style-Clipboard-History-Manager.git
   ```

## Development Setup

### Prerequisites

- **Rust 1.77+** (نسخهٔ دقیق در `rust-toolchain.toml`)
- **Node.js ≥ 20.19** (کف Vite 7 — `.nvmrc` نسخهٔ ۲۰ را پین می‌کند)
- System build dependencies

```bash
# Install system dependencies
make deps
make rust
make node
source ~/.cargo/env
make check-deps
```

### Running in Development Mode

```bash
npm install
make dev
```

## Making Changes

1. **Create a new branch** from `master`
2. **Make your changes** and test them locally
3. **Run linters**: `make lint && make format`
4. **Commit** with descriptive message following [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat:` - New feature
   - `fix:` - Bug fix
   - `docs:` - Documentation
   - `i18n:` - Translation changes
   - `perf:` - Performance improvement
   - `refactor:` - Code refactoring

## Pull Request Process

1. Update your branch: `git rebase upstream/master`
2. Push: `git push origin feature/your-feature-name`
3. Create a Pull Request on GitHub
4. Wait for review

### PR Requirements

- [ ] Code follows style guidelines
- [ ] `make lint` passes
- [ ] `make test` passes
- [ ] Changes tested locally
- [ ] Documentation updated if needed
- [ ] i18n changes include both `fa.json` and `en.json`

## Style Guidelines

### TypeScript/React
- Use functional components with hooks
- Use TypeScript types (avoid `any`)
- Use `useTranslation()` for UI strings

### Rust
- Use `cargo fmt` and address `clippy` warnings
- Use `AppError` / `thiserror` (not `Result<(), String>`)
- Use `tracing` instead of `eprintln!`
- Use `crate::fs_atomic::write_atomic` for file writes
- Use `crate::clipboard_io` for clipboard operations

### Bilingual comments (project policy)
- Doc comments (`///`) on public items must be **bilingual (English +
  Persian)** — see `history_crypto.rs` for the pattern.
- Prefer "why" comments; add "what" comments only when code is not
  self-explanatory.
- Commit messages follow Conventional Commits (enforced by the
  `commit-msg` hook).

### Git hooks

```bash
make hooks   # install pre-commit and commit-msg
```

- `pre-commit`: ESLint + tsc on staged TS and `cargo fmt --check` on staged Rust.
- `commit-msg`: enforces the Conventional Commits format.

### New dependencies
- New Rust dependencies must have a license present in the
  `src-tauri/deny.toml` allow-list, or `cargo deny check` (a CI gate)
  will fail.

### CSS/Tailwind
- Use Tailwind utility classes
- Support both light and dark modes
- Support RTL via `dir` attribute

## i18n / Translation

### Adding a New String
1. Add to both `src/locales/en.json` and `src/locales/fa.json`
2. Use nested key structure
3. Reference with `t('key.path')` in components

### Adding a New Language
1. Create `src/locales/{lang}.json`
2. Import it in `src/i18n/config.ts`
3. Add it in `UserSettings::set_language` in `user_settings.rs`

## Reporting Bugs

Before reporting:
1. Search existing issues
2. Try the latest version
3. Collect: OS, DE, X11/Wayland, reproduction steps, logs

## Suggesting Features

Check alignment with project goals, search existing issues, consider implementation complexity.

## Questions?

Open a [discussion](https://github.com/Mahdi-Arts/Windows-11-Style-Clipboard-History-Manager/discussions).

---

**Thank you for contributing! 🙏 / از مشارکت شما سپاسگزاریم! 🙏**