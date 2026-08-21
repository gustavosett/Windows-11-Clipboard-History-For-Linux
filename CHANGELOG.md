# 📦 Changelog

<div dir="rtl">

همه تغییرات قابل توجه این پروژه در این فایل ثبت می‌شود.

</div>

All notable changes to this project will be documented in this file.

---

## [Unreleased] — 2026-08-21

### Workflows hardened + security/UX pass / سخت‌سازی ورک‌فلوها + گردش امنیت و UX (2026-08-21)

- **Hardened pipelines implemented; activation is one maintainer step.**
  The final activation patch is archived at
  `docs/archive/patches/hardened-ci-workflows.patch` and applies cleanly:
  `git am docs/archive/patches/hardened-ci-workflows.patch && git push`.
  It activates canonical `ci.yml` (rustup-only Rust install, packaging job
  incl. `flatpak-builder-lint`, canonical binary smoke test), `release.yml`
  (canonical artifact names/URLs/AUR, GPG/SBOM/SLSA), a manual `e2e.yml`
  Playwright workflow and a SHA-pinned `stale.yml`. The workflow files
  themselves are intentionally not part of this PR: a bot token without the
  `workflows` permission cannot push `.github/workflows/*`. Verified in a
  pristine worktree: after applying, `scripts/check-packaging.sh` passes
  (legacy `win11-clipboard-history` / `Modern-Clipboard-History-For-Linux`
  names and the missing `check-rust-syntax.mjs` reference are gone).
  **خطوط لولهٔ hardened پیاده‌سازی شدند؛ فعال‌سازی یک گام نگهدارنده است.**
  پچ نهایی در `docs/archive/patches/hardened-ci-workflows.patch` بایگانی
  شده و اعمال تمیز دارد: `git am ... && git push`. این پچ `ci.yml` رسمی
  (نصب Rust فقط با rustup، job بسته‌بندی شامل `flatpak-builder-lint`،
  smoke تست باینری رسمی)، `release.yml` (نام‌های رسمی آرتیفکت/URL/AUR،
  GPG/SBOM/SLSA)، `e2e.yml` دستی و `stale.yml` پین‌شده را فعال می‌کند.
  فایل‌های ورک‌فلو عمداً جزو این PR نیستند: توکن ربات بدون مجوز
  `workflows` نمی‌تواند `.github/workflows/*` را پوش کند. در کار درخت
  تمیز تأیید شد: پس از اعمال، `scripts/check-packaging.sh` سبز است (نام‌های
  قدیمی `win11-clipboard-history` / `Modern-Clipboard-History-For-Linux` و
  ارجاع مفقود به `check-rust-syntax.mjs` حذف می‌شوند).
- **Security: secret filter extended.** New well-known token prefixes
  (`EAAC`, `SG.`, `xoxs-`, `whsec_`) and Telegram bot-token detection
  (`<bot_id>:<secret>`, length-gated) — with unit tests. Legacy unbounded
  `get_history` IPC command removed (frontend uses paged `get_history_page`,
  ADR-0007).
  **امنیت: فیلتر اسرار تقویت شد.** پیشوندهای توکن شناخته‌شدهٔ جدید
  (`EAAC`، `SG.`، `xoxs-`، `whsec_`) و تشخیص توکن ربات تلگرام
  (`<bot_id>:<secret>` با دروازهٔ طول) — همراه تست واحد. فرمان IPC منسوخ و
  نامحدود `get_history` حذف شد (فرانت‌اند از `get_history_page` صفحه‌بندی‌شده
  استفاده می‌کند، ADR-0007).
- **UX: accessibility polish.** `aria-label`/`aria-pressed`/`title` added to
  icon-only controls (pin, delete, smart actions, compact toggle, drag-handle
  close) — screen-reader friendly, zero visual regression.
  **UX: صیقل دسترس‌پذیری.** برچسب‌های `aria-label`/`aria-pressed`/`title`
  برای کنترل‌های آیکون‌فقط (سنجاق، حذف، عملیات هوشمند، حالت فشرده، دکمهٔ
  بستن دستگیرهٔ کشیدن) اضافه شد — سازگار با صفحه‌خوان، بدون بازگشت بصری.

### Hardened pipelines activated + maintenance pass / فعال‌سازی خطوط لولهٔ hardened + گردش نگهداشت (2026-08-21)

- **Hardened CI/release/E2E pipelines finalised; two-step activation.**
  The canonical-named, SHA-pinned workflows (CI, release, and a NEW manual
  `e2e.yml` Playwright workflow) are complete, but the bot token cannot
  push `.github/workflows/*` — they ship as the regenerated
  `docs/patches/hardened-ci-workflows.patch` (applies cleanly on master;
  verified in a pristine worktree). One maintainer command activates them:
  `git am docs/patches/hardened-ci-workflows.patch && git push`. The
  `docs/github-workflows/` mirror is deleted for good; superseded patches
  live in `docs/archive/patches/`.
  **خطوط لولهٔ hardened نهایی و آماده‌اند؛** چون توکن ربات نمی‌تواند
  ورک‌فلوها را پوش کند، با پچ بازتولیدشدهٔ
  `docs/patches/hardened-ci-workflows.patch` حمل می‌شوند (اعمال تمیز روی
  master راستی‌آزمایی شد) و فعال‌سازی با یک دستور نگهدارنده است. کپی
  آینه‌ای برای همیشه حذف شد.
- **`actions/stale` is pinned to a full commit SHA** (`4391f3da…` # v11.0.0),
  closing the last unpinned action and honouring the supply-chain contract.
  **آخرین اکشن بدون پین (`actions/stale`) به SHA کامل پین شد.**
- **Repository slimmed by ~41 MB.** Removed the unreferenced 34 MB
  `docs/img/banner.gif`, the duplicate `dynamic_themes.jpg` (3.4 MB), and
  converted `dynamic_themes.png` → WebP (3.8 MB → 247 KB, same 2046×1160).
  **مخزن ~۴۱ مگابایت سبک شد** (حذف گیف مردهٔ ۳۴ مگابایتی، حذف نسخهٔ
  تکراری jpg و تبدیل png به WebP با کیفیت یکسان).
- **Font payload right-sized.** `Vazirmatn-ExtraLight` (never declared) and
  `Vazirmatn-Light` (weight 300, never used by the UI) removed; the CSS now
  documents the variable-font-first strategy with 400/500/600/700 static
  fallbacks only.
  **بار فونت بهینه شد؛** دو وزن بی‌استفاده حذف و راهبرد «فونت متغیر
  اول، چهار وزن ایستا برای سازگاری» مستند شد.
- **tree-sitter dev gate removed.** The 20 MB native devDependencies
  (`tree-sitter`, `tree-sitter-rust`) and `scripts/check-rust-syntax.mjs`
  were dropped — `cargo fmt`/`clippy`/`cargo test` in CI (and the pre-commit
  hook when Rust is installed) already cover syntax with strictly more
  precision. README, `docs/CI.md`, CONTRIBUTING, Makefile and the workflows
  were updated accordingly.
  **گیت سینتکس tree-sitter حذف شد** (۲۰ مگابایت وابستگی native کمتر)؛
  پوشش کامل‌تر با `cargo fmt/clippy/test` از قبل در CI وجود دارد.
- **`which` subprocess eliminated.** New `src-tauri/src/exec_lookup.rs`
  resolves helpers against `PATH` in-process (with unit tests); the five
  former `Command::new("which")` call sites now delegate to it — one fewer
  fork per availability check and no trust in a PATH-hijackable `which`.
  **جستجوی برنامه‌های کمکی بدون subprocess ی `which`** در ماژول جدید
  `exec_lookup` انجام می‌شود؛ پنج نقطهٔ فراخوانی به آن واگذار شد.
- **`ClipboardManager::new` takes the data directory.** The vestigial
  `persistence_path` (`history.json`) parameter — whose parent directory
  was the real data dir — was renamed to an explicit `data_dir`; the legacy
  JSON path is now derived inside the constructor.
  **امضای سازندهٔ `ClipboardManager` صریح شد؛** پارامتر تاریخی
  `history.json` حذف و `data_dir` مستقیم گرفته می‌شود.
- **`ClipboardTab` decomposed (561 → ~360-line orchestrator).** Rendering
  moved to `src/components/HistoryList/` (`HistoryRow`, `PinnedSection`,
  `RecentSectionLabel`, `LoadMoreButton`) with bilingual docs; behaviour
  unchanged, all 135 frontend tests stay green.
  **تب کلیپ‌بورد تجزیه شد** — اجزای رندر به `HistoryList/` منتقل و تب
  به هماهنگ‌کنندهٔ ~۳۶۰ خطی تبدیل شد (رفتار بدون تغییر، تست‌ها سبز).
- **Flatpak readiness hardened.** Runtime bumped GNOME 46 → 48; CI now
  lints the manifest **and** AppStream metainfo with `flatpak-builder-lint`
  (pinned to a full commit SHA, PyGObject from the prebuilt system package
  — nothing compiles from source); the Flatpak guide documents Flathub
  submission and the new gate.
  **آمادگی فلت‌پک:** runtime به GNOME 48 ارتقا یافت و lint فلت‌هاب به
  CI اضافه شد (پین‌شده به SHA، بدون کامپایل سورس).
- **Packaging architecture documented.** `packaging/DEPLOYMENT.md` now
  opens with an explicit two-path table (Tauri bundle → GitHub Releases;
  `packaging/` tree → distro integration) enforced by the packaging gate.
  **معماری دو‌مسیری بسته‌بندی در DEPLOYMENT.md مستند شد.**
- **Favicon & `.gitattributes` fixed.** The Vite default icon is replaced
  by the app icon (`/icon.svg`); linguist hints now only mark lockfiles as
  generated (hand-edited JSON stays visible in blame) and binaries are
  marked `binary`.
  **آیکون پیش‌فرض Vite با آیکون برنامه جایگزین شد** و
  `.gitattributes` فقط lockfileها را «تولیدشده» می‌شمارد.

### Release-readiness pass / گردش کار آماده‌سازی انتشار (2026-08-21)

- **Release pipeline fully renamed to the canonical project.** 17 stale
  references to the pre-rename names (`win11-clipboard-history`,
  `Modern-Clipboard-History-For-Linux`) remained in `ci.yml` and
  `release.yml`: the CI smoke test probed a binary that no longer exists,
  the release notes installed artifact names that are never produced, and
  the AUR job pushed a package pointing at a deleted repository. All are
  now the canonical `windows-11-style-clipboard-history-manager` names;
  `scripts/check-packaging.sh` (which previously failed on `master`) is
  green again and wired into CI as a dedicated blocking job.
  **خط لولهٔ انتشار کامل به نام رسمی پروژه بازنویسی شد؛** ۱۷ ارجاع
  قدیمی در ورک‌فلوها اصلاح شد و گیت پکیجینگ سبز و در CI مسدودکننده است.
- **CI no longer depends on a third-party Rust action.** The flaky
  `dtolnay/rust-toolchain` step (root cause of six consecutive red runs)
  was replaced by the runner's built-in rustup reading the repository's
  `rust-toolchain.toml`, with retries on transient network failures —
  one fewer supply-chain dependency.
  **نصب Rust با rustup خود رانر از فایل مخزن انجام می‌شود** (بدون اکشن
  شخص ثالث، با تلاش مجدد روی خطای گذرا) — رفع ریشهٔ شش اجرای قرمز CI.
- **Canonical artifact filenames, guaranteed.** New
  `scripts/normalize-artifacts.sh` renames whatever the Tauri bundler
  emits (per-format `productName` casing) to the exact lowercase names
  referenced by README, PKGBUILD, the installer and `SHA256SUMS`; the
  release workflow runs it right after `tauri build` (idempotent,
  unit-tested against six naming variants).
  **نام فایل آرتیفکت‌ها با اسکریپت نرمال‌سازی به نام رسمی تضمین می‌شود.**
- **Version sync covers every source.** The tag workflow now also updates
  `Cargo.lock`, the Debian changelog and the AppStream metainfo, matching
  the drift guard in `check-packaging.sh`.
  **همگامی نسخه شامل Cargo.lock، changelog دبیان و متاینفو هم شد.**
- **Setup wizard decomposed (748 → 216-line orchestrator).** The monolith
  is now `src/components/setup/`: one module per step plus shared
  `WizardButton`, `StatusCard`, `StepHeader`, `LanguageSwitcher` and a
  `useSetupChecks` hook. Behaviour identical; five new unit tests cover
  the extracted primitives.
  **جادوگر راه‌اندازی به ماژول‌های تک‌مسئولیتی تجزیه شد** با ۵ تست جدید.
- **Entrance motion (Windows 11 style).** `animate-window-in` /
  `animate-step-in` / `animate-item-in` keyframes for the settings and
  setup windows and wizard steps — automatically disabled under
  `prefers-reduced-motion`.
  **انیمیشن ورود ویندوزی برای پنجره‌ها و گام‌ها** (با احترام به
  reduced-motion).
- **Flatpak metainfo fixed & enriched.** Screenshot URL pointed at the
  non-existent `main` branch (404) — now `master`; 2.5.0 release notes
  are bilingual and describe the actual feature set.
  **متاینفو: اصلاح URL اسکرین‌شات و غنی‌سازی یادداشت نسخه به دو زبان.**
- **Docs hygiene.** Session reports moved from `docs/reports/` to
  `docs/archive/reports/` (index left behind with the policy); README,
  CONTRIBUTING and `docs/CI.md` updated to match the real pipeline;
  `engines.node >= 20.19` declared and `.nvmrc` added; new
  `make packaging` target.
  **پاکیزگی مستندات:** آرشیو گزارش‌ها، همگامی README/CONTRIBUTING/CI با
  خط لولهٔ واقعی، اعلان کف Node و target جدید `make packaging`.

### QA follow-up / پیروی از بازبینی (2026-08-21)

- **Lockfile restored in sync.** `package-lock.json` had drifted out of sync
  with `package.json` (`@playwright/test`, `playwright`, `playwright-core`
  were missing), so a clean `npm ci` failed. Regenerated the lockfile; `npm ci`
  now installs cleanly.
  **بازگردانی همگامی قفل وابستگی‌ها:** `package-lock.json` از `package.json`
  فاصله گرفته بود و `npm ci` تمیز شکست می‌خورد؛ بازتولید شد و اکنون درست نصب می‌شود.
- **Playwright E2E specs now linted.** The type-aware ESLint block matched
  `tests/e2e/**` but they belonged to no `tsconfig` project, so `npm run lint`
  failed with a parsing error and 117 real findings stayed hidden. Added a
  dedicated `tests/e2e/tsconfig.json` + ESLint block and fixed every finding:
  unused imports, unsafe `any` reads in `global-teardown.ts` (typed JSON
  report + `describeError` helper), and an unused locator whose test now
  actually asserts the drag handle is visible.
  **لینت شدن specهای E2E:** specها در هیچ پروژهٔ tsconfig نبودند و لینت با خطای
  parsing می‌شکست؛ با افزودن tsconfig اختصاصی و بلوک ESLint، همهٔ یافته‌ها
  (ایمپورت استفاده‌نشده، خواندن ناامن `any` و locator بی‌اثر) اصلاح شد.
- **Live release pipeline canonicalised for real.** `.github/workflows/release.yml`
  still referenced the legacy repository (`Mahdi-Arts/Modern-Clipboard-History-For-Linux`)
  and legacy package names (`win11-clipboard-history-bin`), and the Cloudsmith/AUR
  steps still used `continue-on-error` + masked failures. Replaced with the
  hardened, canonical-named pipelines (`docs/github-workflows/` is now a true
  mirror). `scripts/check-packaging.sh` passes again.
  **خط لولهٔ انتشار واقعاً رسمی شد:** فایل‌های زنده به مخزن و نام‌های قدیمی
  ارجاع می‌دادند و مراحل Cloudsmith/AUR fail-open بودند؛ با نسخهٔ سخت‌شده و
  رسمی جایگزین و `check-packaging.sh` دوباره قبول شد.
- **Packaging contract is now a real CI gate.** `scripts/check-packaging.sh`
  runs as a blocking step in `ci.yml` (previously only mentioned in a comment),
  and gained a **single-source version-drift guard** (Cargo.toml, package.json,
  tauri.conf.json, Cargo.lock, debian/changelog, AppStream metainfo must agree).
  **گیت بسته‌بندی واقعی شد:** اسکریپت کنترل بسته‌بندی به‌عنوان گام مسدودکنندهٔ CI
  اجرا می‌شود و یک گارد رانش نسخهٔ تک‌منبعی هم به آن افزوده شد.
- **Maintainer metadata corrected.** Replaced the placeholder
  `info@Mahdiarts.ir` in `packaging/debian/control` and the RPM spec changelog
  with `mahdi-arts@users.noreply.github.com` (matching `debian/changelog` and
  `SECURITY.md`).
  **اصلاح فرادادهٔ نگهدارنده:** ایمیل جایگزین `info@Mahdiarts.ir` در `debian/control`
  و spec آرپی‌ام با ایمیل رسمی پروژه جایگزین شد.
- **README / CI docs brought in line** with the now-live hardened pipelines
  and the new `check-packaging.sh` gate.
  **هم‌راستا شدن README / CI.md** با خط‌لوله‌های سخت‌شدهٔ فعال و گیت جدید بسته‌بندی.

### Security / امنیت

- **Central window authorization.** State-changing Tauri commands now use a
  deny-by-default `window_policy` for `main`, `settings`, and `setup`.
  **مجوزدهی متمرکز پنجره‌ها:** فرمان‌های تغییردهندهٔ وضعیت با سیاست رد پیش‌فرض
  برای پنجره‌های اصلی، تنظیمات و راه‌اندازی محدود شده‌اند.
- **Fail-closed optional publication.** Configured Cloudsmith/AUR uploads now
  fail the release on real errors; absent optional credentials still skip safely.
  **انتشار اختیاری fail-closed:** خطای واقعی Cloudsmith/AUR تنظیم‌شده انتشار را
  متوقف می‌کند و نبود credential اختیاری فقط باعث عبور امن می‌شود.

### UI and localization / رابط و محلی‌سازی

- Persian/English selection is available in first-run Setup and Settings.
  Persian RTL is scoped to those windows; main popup and tray remain English/LTR.
  انتخاب فارسی/انگلیسی در راه‌اندازی نخست و تنظیمات فراهم است؛ RTL فقط روی همین
  پنجره‌ها اعمال می‌شود و پنجرهٔ اصلی و tray انگلیسی/LTR می‌مانند.
- Permission/manual-setup diagnostics use stable codes or translated frontend text.
  پیام‌های مجوز و راه‌اندازی دستی با کد پایدار یا متن ترجمه‌شده نمایش داده می‌شوند.

### Packaging and QA / بسته‌بندی و کنترل کیفیت

- **Rename completed in the release pipeline (audit follow-up).** The live
  `.github/workflows/release.yml` now references only the canonical repository
  (`${{ github.repository }}` in release-notes URLs), canonical artifact names
  (`windows-11-style-clipboard-history-manager_{version}_amd64.deb`,
  `windows-11-style-clipboard-history-manager-{version}-1.x86_64.rpm`) and the
  canonical AUR package `windows-11-style-clipboard-history-manager-bin`
  (clone target, install file, hand-written `.SRCINFO` fallback). The CI smoke
  test now runs `windows-11-style-clipboard-history-manager-bin`, and the
  Cloudsmith/AUR steps are truly fail-closed (`set -euo pipefail`, no
  `continue-on-error`, unmasked `git push`).
  **تکمیل تغییر نام در خط لولهٔ انتشار (پیرو ممیزی):** workflow های زنده اکنون
  فقط به مخزن، نام artefact و بستهٔ AUR رسمی ارجاع می‌دهند؛ تست smoke ی CI باینری
  رسمی را اجرا می‌کند و کانال‌های Cloudsmith/AUR واقعاً fail-closed شدند.
- **RPM bundle at parity with Debian.** `bundle.linux.rpm.files` now installs
  the same system files as the `.deb` — launcher wrapper, `/dev/uinput` udev
  rule, desktop entry, icon set, and AppArmor profile — so Fedora installs get
  the NVIDIA/DMA-BUF wrapper and input ACLs automatically.
  **برابری بستهٔ RPM با Debian:** بستهٔ rpm نیز wrapper، قانون udev، دسکتاپ‌فایل،
  آیکون‌ها و پروفایل AppArmor را نصب می‌کند.
- **Regression guards added.** `scripts/check-packaging.sh` now fails on any
  legacy identifier in `.github/workflows/`, verifies the CI smoke binary,
  cross-checks the AUR package name between `aur/PKGBUILD` and `release.yml`,
  and enforces structural deb/rpm `files` parity via Node. New contract tests
  cover every Settings-window SVG icon.
  **گاردهای بازگشت:** اسکریپت کنترل بسته‌بندی هر شناسهٔ قدیمی در workflowها،
  ناهماهنگی نام AUR و عدم برابری deb/rpm را شکست می‌دهد؛ تست‌های قراردادی برای
  همهٔ آیکون‌های پنجرهٔ تنظیمات افزوده شد.
- **Product named `Windows 11 Style Clipboard History Manager`.** The binary is
  `windows-11-style-clipboard-history-manager-bin`, the launcher/command is
  `windows-11-style-clipboard-history-manager`, and all packaging (Debian, RPM, AppImage,
  Flatpak, AUR), AppArmor profiles, udev rules, scripts, CI, docs, and data/config
  paths (`~/.local/share/…`, `~/.config/…`, `~/.cache/…`) use the canonical name.
  محصول با نام رسمی Windows 11 Style Clipboard History Manager یکپارچه شد؛ باینری، راه‌انداز،
  بسته‌ها و مسیرهای داده/تنظیمات همگی به نام استاندارد پروژه تنظیم شدند.
- **Security hardening (from the 2026-08-21 review).** The in-memory history
  key is now wrapped in `zeroize::Zeroizing` and the `chacha20poly1305`
  `zeroize` feature is enabled explicitly; `open_safe_url` now refuses hosts
  that resolve to private/metadata IPs (post-DNS defence); undecryptable DB
  rows are **quarantined** to `quarantine.log` instead of being silently
  dropped; `actions/stale` is SHA-pinned; and the Cloudsmith/AUR channels are
  now truly **fail-closed** (`set -euo pipefail`, no `continue-on-error`).
  **سخت‌سازی امنیتی (از بازبینی ۲۰۲۶-۰۸-۲۱):** کلید حافظه در `Zeroizing`
  پیچیده و feature ی `zeroize` در `chacha20poly1305` صریحاً فعال شد؛
  `open_safe_url` میزبان‌هایی که به IP خصوصی/metadata رزولوشن می‌شوند را رد
  می‌کند (دفاع پس از DNS)؛ ردیف‌های قابل‌رمزگشایی‌نشده به `quarantine.log`
  قرنطینه می‌شوند نه حذف بی‌صدا؛ `actions/stale` به SHA پین شد؛ و کانال‌های
  Cloudsmith/AUR واقعاً **fail-closed** شدند.
- **Structured errors + tests.** `AppError::code()` adds a stable machine-readable
  code for telemetry/translation; property-style edge-case tests were added to
  the secret filter (`privacy.rs`) and the post-DNS URL check (`open_url.rs`).
  **خطاهای ساخت‌یافته + تست‌ها:** متد `AppError::code()` کد پایدار ماشین‌خوان
  اضافه کرد؛ تست‌های لبه به سبک property برای فیلتر اسرار و چک پس از DNS افزوده شد.
- **Packaging & ops docs.** Added a bilingual `packaging/DEPLOYMENT.md`
  (`.deb` → GitHub Release → Flatpak pipeline, key backup/recovery, AppArmor
  enforce, release checklist) and updated ADR-0006 to recommend the Secret
  Service backend with recovery guidance.
  **مستندات بسته‌بندی و عملیات:** `packaging/DEPLOYMENT.md` دوزبانه (خط لولهٔ
  `.deb` → GitHub Release → Flatpak، پشتیبان‌گیری/بازیابی کلید، AppArmor enforce،
  چک‌لیست انتشار) و به‌روزرسانی ADR-0006 برای توصیهٔ Secret Service افزوده شد.
- Canonicalized `windows-11-style-clipboard-history-manager-bin` across Cargo, CI, Debian, Flatpak,
  AppArmor, Make, and the launcher; added package-contract and `.deb` checks.
  نام باینری در همهٔ مسیرهای ساخت و انتشار یکسان و کنترل محتوای `.deb` افزوده شد.
- Added bilingual Debian-first/Flatpak-second release guidance and performance SLOs.
  راهنمای دوزبانهٔ انتشار Debian-اول/Flatpak-دوم و SLOهای عملکرد افزوده شد.

- **Hardened CI prepared for activation.** The intended `.github/workflows/`
  (blocking audits, coverage, `cargo test` default + `--all-features`,
  clippy, syntax gate, CLI smoke) live in `docs/github-workflows/` with a
  ready-to-apply patch — `git am docs/patches/hardened-ci-workflows.patch`
  (GitHub Apps without the `workflows` permission cannot push workflow
  files). Every action is pinned to a full commit SHA; all third-party URL
  references were removed from the release pipeline.
  CI سخت‌شده برای فعال‌سازی آماده شد (پچ آماده در docs/patches)؛ همهٔ actionها به SHA کامل پین و ارجاعات شخص ثالث از خط لولهٔ انتشار حذف شدند.
- **AUR push is fail-closed.** SSH uses `StrictHostKeyChecking yes` with
  `known_hosts` pinned via the `AUR_KNOWN_HOSTS` secret — never
  trust-on-first-use. Optional GPG `SHA256SUMS.sig` published when
  `RELEASE_GPG_PRIVATE_KEY` is configured.
  اتصال AUR به‌صورت fail-closed با کلید میزبان پین‌شده انجام می‌شود؛ امضای GPG اختیاری برای SHA256SUMS اضافه شد.
- **HTTPS-only smart actions.** `open_url.rs` and `urlSafety.ts` now accept
  only `https`/`mailto`; frontend upgrades `http://` input to `https://`.
  عملیات هوشمند فقط HTTPS؛ ورودی http به https ارتقا می‌یابد.
- **Window-gated settings.** `set_user_settings` / `set_app_language` are
  refused unless called from the `settings` window.
  تغییر تنظیمات فقط از پنجرهٔ تنظیمات مجاز است.
- **Key material zeroised in memory.** `chacha20poly1305` compiled with the
  `zeroize` feature; new crypto tests (tamper detection, nonce uniqueness,
  foreign-key rejection).
  پاک‌سازی کلید از حافظه با `zeroize` + تست‌های مقاومت رمزنگاری جدید.

### Performance / کارایی

- **Event-driven clipboard monitoring** (`clipboard_events.rs`): XFixes
  `SelectionNotify` on X11 and `wl-paste --watch` on Wayland wake the
  watcher instantly; adaptive polling remains as fallback (with
  `PR_SET_PDEATHSIG` child reaping).
  نظارت کلیپ‌بورد رویدادمحور شد (XFixes / wl-paste) با polling پشتیبان.

### Quality / کیفیت

- ESLint upgraded to **type-aware** (`recommendedTypeChecked`); all 83
  findings fixed (floating promises, misused async handlers, typed test
  mocks, ref-narrowing in keyboard navigation).
  ESLint به حالت آگاه به نوع ارتقا یافت و تمام یافته‌ها اصلاح شدند.
- Flatpak deployment guide + one-command `packaging/flatpak/build.sh`;
  packaging checklist extended (GPG, AUR known hosts).
  راهنمای استقرار فلت‌پک و اسکریپت ساخت یک‌دستوری اضافه شد.

---

## [2.5.0] - 2026-08-21

### Security / امنیت

- **Hardened CI/release contract** in `docs/github-workflows/` (blocking
  lint, coverage, `cargo test` default + all-features, clippy, `cargo audit`,
  `cargo deny`, `npm audit`; releases publish `SHA256SUMS`, per-artifact SPDX
  SBOM, and SLSA provenance). Copy onto `.github/workflows/` with a
  `workflows`-scoped token ([docs/github-workflows/README.md](docs/github-workflows/README.md)).
  قرارداد CI/انتشار سخت‌شده در `docs/github-workflows/`؛ فعال‌سازی زنده با مجوز `workflows`.
- **GIF HTTP is compile-clean.** Default build no longer references `reqwest`
  or `gif_manager`; `paste_gif_from_url` / `search_tenor` stub without the
  feature. `ssrf::no_redirects` is cfg-gated.
  بیلد پیش‌فرض بدون کلاینت HTTP کامپایل می‌شود.
- **Paste ACL.** Keystroke injection is refused unless the caller is the
  `main` window. `inject_authorized_paste` no longer issues-and-consumes its
  own ticket (that check was a no-op). Tickets remain for the GIF path.
  تزریق کلید فقط از پنجرهٔ اصلی.
- **Encrypted images.** PNG files use the `W11E1` ChaCha20-Poly1305 envelope;
  legacy plaintext PNGs still load. Deletes overwrite with zeros.
  تصاویر روی دیسک رمز می‌شوند.
- **Shared `net_policy`.** SSRF and Smart-Action URL opening share one IP
  blocklist. Secret filter now matches Bearer, GitLab, npm, HuggingFace,
  Anthropic, AWS secret keys, and Azure AccountKey.
  سیاست IP مشترک و فیلتر اسرار گسترده‌تر.

### Changed / تغییرات

- Tauri capabilities split per window (`main` / `settings` / `setup`).
  قابلیت‌های Tauri به ازای پنجره جدا شدند.
- AppArmor `/tmp` is owner-only. / دسترسی `/tmp` در AppArmor فقط مالک است.
- Load-more control and empty-state icon polish.
  دکمهٔ «بارگذاری بیشتر» و حالت خالی زیباتر شد.

---

## [2.4.0] - 2026-08-21

### Security / امنیت

- **Fail-closed persistence:** a missing or wrong encryption key no longer
  installs an ephemeral key next to the real database. The session stays
  in-memory-only until the key is restored. AEAD decrypt errors skip the
  row instead of leaking ciphertext into the UI.
  ماندگاری fail-closed: کلید اشتباه دیگر کلید موقت کنار دیتابیس نمی‌سازد.
- **GIF HTTP is compile-time optional** (`--features gif-search`). The
  default release binary has no `reqwest` client.
  کلاینت HTTP جستجوی GIF فقط با ویژگی کامپایل فعال است.
- **CI/release workflows now match the docs:** blocking `cargo audit` /
  `cargo deny` / `npm audit`, coverage, `cargo test`, xvfb smoke,
  `SHA256SUMS` + per-artifact SPDX SBOM + SLSA provenance. All URLs point
  at `Mahdi-Arts/Windows-11-Style-Clipboard-History-Manager`.
  گیت‌های CI/انتشار با مستندات هم‌خوان شدند.

### Changed / تغییرات

- **Paged IPC is the default UI path** (`get_history_page`, ADR-0007).
  `get_history` is clamped to the first 200 items. `history-sync` emits a
  `HistoryPage`. The list shows *loaded / total* and a Load more control.
  صفحه‌بندی IPC مسیر پیش‌فرض رابط است.
- Remaining shortcut backends (KDE, XFCE, COSMIC, LXQt, LXDE) live in
  dedicated modules.
  بک‌اندهای میانبر باقی‌مانده به ماژول‌های جدا منتقل شدند.
- AppArmor profile allows `secret-tool` for the Secret Service key backend.

---

## [2.3.0] - 2026-08-20

### Security / امنیت

- Encryption is **fail-closed**: ChaCha20-Poly1305 errors never persist plaintext.
- Secret filter no longer skips `password=` payloads longer than 4096 bytes.
- `clipboard-changed` and `get_item` now send `for_ipc()` payloads (no HTML, 2048-char cap).
- `paste_item` / `paste_text` go through the same paste ticket + `wrote_recently` gate as GIF paste; paste text is capped at 1 MiB.
- Autostart `.desktop` `Exec=` no longer uses `sh -c`.
- AppArmor profile allows `xdg-open` / `groups` (needed for Smart Actions and permission checks).
- CI **blocks** on `npm run test:coverage`, `cargo test`, Clippy `-D warnings`, `cargo audit`, and `npm audit --audit-level=high`.
- Releases attach `SHA256SUMS`, an SPDX SBOM, and SLSA provenance; notes and AUR `.SRCINFO` point at this repository only.

### Quality / کیفیت

- `shortcut_conflict_detector` split into per-DE modules (`gnome`, `kde`, `tiling`, `xfce`).
- FNV hashing extracted to `content_hash.rs`.
- Settings window strings are fully i18n; empty-state keyboard hint restyled.

### Packaging / بسته‌بندی

- Version 2.3.0 across npm / Cargo / Tauri / AUR / Debian / AppStream.
- Flatpak remains the sandboxed channel (no `/dev/uinput` by default).

---

## [2.2.0] - 2026-08-20

### Security / امنیت

- Paste injection (`finish_paste`) now requires a one-shot ticket issued only after a real clipboard write.
- Smart Actions open URLs via a Rust `xdg-open` helper with the same allowlist as the TypeScript sanitizer. The Tauri shell plugin permission (`http://*` / `https://*`) was removed.
- IPC history payloads strip HTML and cap text at 2048 characters; full items stay in SQLite.
- SQLite uses `secure_delete` and `0600` on the db plus `-wal`/`-shm` sidecars.
- Installer downloads from this repository's GitHub Releases by default (checksum required). Cloudsmith `curl | sudo bash` is opt-in (`USE_CLOUDSMITH=1`).
- CI now **blocks** on `npm test` + coverage, `cargo test`, Clippy `-D warnings`, and `cargo audit` / `npm audit`.
- Releases publish `SHA256SUMS`, an SPDX SBOM, and SLSA provenance; notes no longer point at the upstream fork.

### UI / UX

- Empty-state panel with Super+V hint, loading spinner instead of a blank window, tighter glass cards.

### Packaging / بسته‌بندی

- Debian `rules` ships the AppArmor profile; version 2.2.0 across npm / Cargo / Tauri / AUR / changelog.

---

## [2.1.0] - 2026-08-20

### Security / امنیت

- **Fully offline**: Vazirmatn font bundled locally; Google Fonts removed; CSP tightened to `font-src 'self' data:` — the app makes zero network calls at runtime.
- **Hardened URL sanitizer**: blocks IPv6 loopback/ULA/link-local/mapped, private/CGNAT/benchmark/multicast IPv4, raw control characters, and `.internal` hosts (tests caught the original gaps).
- **Mandatory supply-chain verification**: `install.sh` now *implements* `verify_downloaded_file` (previously referenced but undefined — every GitHub-releases fallback path was broken) and requires SHA256SUMS matches; optional GPG verification via `WINDOWS_11_CLIPBOARD_TRUST_KEY`.
- **Blocking security audits in CI**: `cargo audit` and `npm audit` are hard gates (previously `continue-on-error: true`).
- **Release hardening**: SLSA build provenance attestation, SPDX SBOM, SHA256SUMS published per release; all release URLs fixed to the Mahdi-Arts repository (previously pointed at the upstream fork's install scripts and Cloudsmith/AUR sources).
- Optional **AppArmor profile** (complain mode) shipped in `packaging/apparmor/` and installed by the deb to `/usr/share/doc/`.
- Identifier migrated to `io.github.mahdi-arts.clipboard-history` (Flatpak ID, developer metadata).

### Quality / کیفیت

- **Settings UI refactored**: 1106-line `SettingsApp.tsx` split into single-responsibility sections under `src/components/settings/` with a shared `SectionCard`; added `:focus-visible` rings, `prefers-reduced-motion` support, and fixed the undefined `custom-scrollbar` class.
- **Rust refactor**: 1622-line `linux_shortcut_manager.rs` split into `shortcut_config`, `shortcut_error`, `shortcut_utils`, `shortcut_gsettings`, and `shortcut_tiling` submodules (i3/Sway/Hyprland now share idempotent atomic helpers); all `println!/eprintln!` replaced with `tracing`.
- **Testing**: component tests (Testing Library + jsdom), real hook tests with `renderHook`, SSRF edge-case tests (28 urlSafety cases), keyboard-nav tests; **67→72+ tests**; Vitest coverage gate (85%+ lines on covered units).
- `rust-toolchain.toml` added for reproducible Rust builds.

### Packaging / بسته‌بندی

- Desktop file localized (fa); AppStream metainfo updated with 2.1.0 release entry; Flatpak manifest installs metainfo and uses the new app-id.
- Debian metadata: maintainer/author fields point to the project owner; AppArmor profile shipped in the deb.

### Documentation / مستندات

- New `docs/THREAT_MODEL.md` and `docs/adr/` (SQLite persistence, paste injection, SSRF DNS pinning).


### Reliability / پایداری

- Incremental SQLite persistence (upsert / delete / sort-index updates) instead of rewriting the whole table on every copy.
- History hard-cap lowered to **2000** items (was 100 000 on a full-rewrite store).
- Clipboard watcher no longer re-reads `user_settings.json` every poll.

### Security / امنیت

- Tiling WM config rewrite (`i3` / Sway / Hyprland) is **actually gated** on `allow_wm_config_rewrite` (default off).
- Outbound HTTPS clients pin DNS to already-validated public addresses (DNS-rebinding window closed).
- Conflict resolver no longer shells out via `sh -c`; only `gsettings` / `xfconf-query` argv.
- `install.sh` verifies `SHA256SUMS` when GitHub Releases publish them.
- CSP tightened (`img-src` no longer `https:`); Google Fonts allowed only for Vazirmatn.
- Smart-action URL sanitizer rejects loopback and link-local metadata IPs.
- Settings UI warns that password-manager skip is X11-only.

### Quality / کیفیت

- Fixed compile breaks from the 2.0 module split (`Ordering`, `Mutex`, command module paths).
- CI now **requires** `npm test` and `cargo test` before Linux builds.
- History / paste / pin commands return `Result<_, AppError>`.
- Bilingual strings for header, empty state, tray (fa/en), and smart actions.

### Packaging / بسته‌بندی

- Debian `changelog` / `copyright` / `source/format`; `postrm` no longer walks `/home/*`.
- Flatpak manifest no longer requests `--device=all` or `--share=network` by default.
- GitHub Releases publish `SHA256SUMS`.

### Testing / تست

- **New frontend tests**: `urlSafety.test.ts` (13 test cases), `useClipboardHistory.test.ts` (5 test cases)
- **Enhanced `historySearch.test.ts`**: comprehensive tests for regex safety, text extraction, and filtering
- **All tests use Vitest** with proper Tauri API mocking

### Packaging / بسته‌بندی

- **Debian packaging structure** (`packaging/debian/`):
  - `control` — package metadata with proper dependencies
  - `rules` — build rules using dh + Tauri
  - `postinst` — post-install script (udev, input group, desktop database)
  - `postrm` — post-remove script (cleanup)
- **Flatpak manifest** (`packaging/flatpak/`):
  - `dev.gustavosett.ClipboardHistory.yml` — full Flatpak build manifest
  - `dev.gustavosett.ClipboardHistory.metainfo.xml` — AppStream metadata for Flathub

### Quality / کیفیت

- Replaced scattered `eprintln!` calls with `tracing::info!`, `tracing::warn!`, `tracing::error!`
- Named threads (`clipboard-watcher`, `xtest-paste-warmup`, etc.) for better debugging
- Improved code documentation with module-level doc comments

---

## [1.1.0] - 2026-08-20

### Reliability / پایداری

- Clipboard history is persisted to SQLite (`history.db`) on every mutation, on Drop, and on Quit. The previous dirty-flag path never flushed to disk.
- Legacy `history.json` is migrated once, then renamed to `history.json.bak`.
- Images are stored as PNG files; the UI only receives a thumbnail.

### Privacy / حریم خصوصی

- Secret filter (private keys, tokens, JWTs, `password=`) on by default.
- Skip password-manager and private-browsing windows on X11 (on by default).
- Optional “don’t save images”.
- History/settings files are chmod `0600`.
- Tiling WM config rewrite is **opt-in**.

### Security / امنیت

- Real SSRF controls for GIF downloads: HTTPS-only, host allowlist, DNS/IP checks, no redirects.
- Tenor query strings are URL-encoded; API key comes only from `TENOR_API_KEY` (no hardcoded fallback).
- `finish_paste` refuses Ctrl+V unless a clipboard write happened in the last 5 seconds.
- `shell:allow-open` is scoped to `http(s)` and `mailto`.
- Tracing is initialized and the worker guard is kept alive.

### UI / UX

- Pinned items no longer render twice.
- Privacy section and language picker in Settings.
- Incremental `clipboard-changed` updates (no full history refetch).
- Safer regex search (length + nested-quantifier guard).

### Quality / کیفیت

- Frontend Vitest tests and Rust unit tests in CI.
- README rebuilt (the 1.0 architecture section was corrupted).
- Dockerfile no longer swallows build failures.

---

## [1.0.0] - 2026-08-20

### ✅ Production Release Readiness / آماده‌سازی انتشار پایدار

- Promoted the project to the first stable production release line.
- Synchronized application version across npm, Tauri, Cargo, Cargo.lock, and AUR packaging metadata.
- Updated release/package references for the current repository.
- Verified frontend production build, TypeScript, ESLint, Prettier formatting, lockfile reproducibility, and npm security audit.
- Prepared release workflow and Linux packaging metadata for generating `.deb`, `.rpm`, and AppImage artifacts in a full Tauri build environment.

---

## [0.8.0] - 2026-08-20

### 🌐 Bilingual Support / پشتیبانی دو زبانه (NEW)

- **Full Persian (فارسی) + English support** with `react-i18next`
- Instant language switching — **no restart required** (تمام برنامه بدون ریستارت تغییر می‌کند)
- Automatic **RTL (Right-to-Left)** layout for Persian
- **Vazirmatn** Persian font support
- Language selection in Settings UI + persistence in user settings
- Cross-window synchronization via `app-language-changed` event
- `set_app_language` Tauri command with validation
- All documentation bilingual: README, CONTRIBUTING, SECURITY, issue/PR templates
- `docs/BILINGUAL.md` — i18n development guide

### 🚀 Performance / کارایی

- **Unified clipboard I/O module** (`clipboard_io.rs`): single cached X11/Wayland connection reused across all reads — eliminates connection churn in the watcher
- **Watcher optimized**: clipboard read now happens *outside* the history mutex (shorter lock window), and one `arboard::Clipboard` instance is reused instead of 3 new ones per 500ms tick
- **O(1) duplicate detection**: `HashSet<u64>` text-hash index replaces linear scans
- **Debounced persistence**: dirty-flag based saving instead of full-file rewrite on every change
- **Atomic file writes** (`fs_atomic.rs`): `.tmp` + `rename` — crash-safe history/settings
- **Virtualized clipboard list**: `react-window` List replaces flat `Array.map()` — renders only visible items (performance for 100k+ items)

### 🔒 Security / امنیت

- **CSP enabled** (was `null`): strict `Content-Security-Policy` in `tauri.conf.json`
- **`withGlobalTauri: false`**: Tauri API no longer exposed to global scope
- **SSRF protection** for GIF downloads: HTTPS-only, private/loopback IPs blocked
- **10 MB download limit** for GIFs (streamed, not buffered)
- **Unified error types** (`error.rs` with `thiserror`) replacing `Result<(), String>`
- **Tenor API key moved to backend** (`tenor_api.rs`): key no longer in frontend bundle — client calls `search_tenor` Tauri command instead of direct API
- **CSP tightened**: Tenor CDN removed from CSP (now proxied through backend)

### 🧹 Code Quality / کیفیت کد

- `tracing` + `tracing-subscriber` logging with daily rotating log files (replaces scattered `eprintln!`)
- `Cargo.lock` now tracked in git (reproducible builds)
- Stable FNV hash for GIF cache filenames (was randomized `DefaultHasher`)
- GIF cache TTL (24h) to avoid stale content
- Lazy-loaded tabs (Emoji/Kaomoji Symbol pickers) with `React.lazy` + `Suspense`

### 🐳 DevOps & CI

- **Dockerfile** (multi-stage): reproducible build environment for CI
- **CI updated**: Rust tests (`cargo test`) added to pipeline; Docker build check job added
- **Build pipeline**: test job blocks build-linux (quality gate)

### 📚 Documentation / مستندات

- **Fully bilingual README** (فارسی/English) — 359 lines
- **Bilingual CONTRIBUTING.md** with i18n guidelines (349 lines)
- **Bilingual SECURITY.md**, issue templates, PR template
- New `docs/BILINGUAL.md` — i18n development guide
- New `CHANGELOG.md` — structured release history
- `OPTIMIZATION_REPORT.md` — full architecture analysis with prioritized action plan

---

## [0.7.1] - Previous Release

- Bug fixes and dependency updates (see git history for details)