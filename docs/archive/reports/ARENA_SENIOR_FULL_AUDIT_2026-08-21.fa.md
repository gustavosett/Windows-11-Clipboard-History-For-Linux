# 📋 بازبینی ارشد کامل مخزن — ۲۰۲۶-۰۸-۲۱

> تحلیل جامع معماری، کیفیت کد، امنیت، مستندات، فایل‌های زائد و آمادگی انتشار
> بررسی‌شده روی کامیت `ad2e96a` (master / PR #21)
> **خلاصهٔ اجرایی: پروژه از نظر طراحی در سطح بالغ (Enterprise-grade) است، اما خط لولهٔ CI/انتشار روی master قرمز و دچار رانش هویت (identity drift) است و تا فعال‌سازی نسخه‌های hardened، هیچ Release‌ای نباید منتشر شود.**

---

## ۱. تحلیل معماری و ساختار

### پشتهٔ فناوری
| لایه | فناوری | حجم |
| --- | --- | --- |
| هسته | Rust 2021 + Tauri 2.11 | ~۱۱,۹۰۰ خط / ۵۶ فایل |
| رابط کاربری | React 19 + TypeScript 5.6 + Vite 7 + Tailwind 4 | ~۶,۶۰۰ خط |
| ذخیره‌سازی | SQLite (rusqlite bundled) + ChaCha20-Poly1305 | — |
| ورودی/تزریق paste | uinput / XTest (x11rb) / wl-clipboard / arboard | — |
| i18n | i18next (fa/en) + فونت محلی Vazirmatn | — |
| تست | Vitest (۱۳۵) + cargo test (۶۲) + Playwright E2E + smoke | — |

### نقاط قوت
- **جداسازی مراقبتی تمیز (SoC):** `main.rs` فقط bootstrap است؛ `commands.rs` پوشش نازک روی ماژول‌های دامنه؛ هر ماژول یک نگرانی واحد (clipboard_io، privacy، input_simulator، window_policy و…). این دقیقاً الگوی Clean Architecture برای اپ دسکتاپ است.
- **AppState متمرکز** با `paste_gate` (mutex تراکنشی) و `PasteTicket` (nonce یک‌بارمصرف با TTL پنج‌ثانیه‌ای) — طراحی ضد تزریق آزاد Ctrl+V از WebView.
- **زیرماژول‌های per-DE** برای میانبرها (GNOME/KDE/XFCE/LXDE/LXQt/COSMIC/tiling) با الگوی یکسان — قابل توسعه.
- **مجوزدهی پنجره‌ها Deny-by-default** (`window_policy.rs`) + سه Capability مجزا برای main/settings/setup در `src-tauri/capabilities/`.
- **۸ سند ADR + THREAT_MODEL + ARCHITECTURE + PERFORMANCE budget** — نادر حتی در پروژه‌های تجاری.
- فرانت‌اند: تفکیک components/hooks/services/utils، لیست مجازی‌شده (react-window)، تقسیم Setup Wizard به مراحل کوچک.

### نقاط ضعف
- `ClipboardTab.tsx` با ۵۶۰ خط بزرگ‌ترین فایل UI است و ظرف تقسیم به زیرکامپوننت‌ها را دارد.
- API تاریخی: `ClipboardManager::new(dir.join("history.json"))` — پارامتر `persistence_path` امروز فقط برای migrate قدیمی استفاده می‌شود و دیتابیس واقعی `history.db` است؛ نام‌گذاری گمراه‌کننده است (بوی کد Legacy، نه باگ).
- دو مسیر بسته‌بندی موازی (deb/rpm داخلی Tauri + `packaging/debian` با debhelper) — گارد `check-packaging.sh` هماهنگی را تضمین می‌کند ولی بار نگهداشت دوبرابر است.

**ارزیابی: ۹/۱۰**

---

## ۲. کیفیت کد (Code Quality)

### نتایج آزمایش واقعی (اجراشده در این بازبینی)
| گیت | نتیجه |
| --- | --- |
| `tsc --noEmit` | ✅ بدون خطا |
| `eslint --max-warnings 0` | ✅ صفر هشدار |
| `vitest run` | ✅ ۱۳۵/۱۳۵ تست سبز (۱۴ فایل) |
| `node scripts/check-rust-syntax.mjs` | ✅ ۵۶ فایل Rust سالم |
| `bash scripts/check-packaging.sh` | ❌ **شکست می‌خورد** — «legacy project name found in .github/workflows» |

### نقاط قوت
- کامنت‌گذاری دوزبانه که «چرا» را توضیح می‌دهد نه «چه»؛ خطای‌گیری ساخت‌یافته با `thiserror`؛ لاگ `tracing` با رولینگ روزانه و `release_max_level_info`.
- SOLID: Open/Closed در ماژول میانبرها (افزودن DE جدید = فایل جدید)، Dependency Inversion در `KeyBackend` (فایل ↔ Secret Service)، Single Responsibility در تمام ماژول‌ها.
- کد FAIL-CLOSED در مسیرهای حساس (رمزنگاری، کلید، بلیت paste، AUR push).
- SQL کاملاً پارامتری (`params![]`) — بدون ریسک SQL Injection.
- `unsafe` فقط در مرز FFI (uinput/libc/X11) و محدود.
- تست‌های واحد امنیتی داخل خود ماژول‌ها (SSRF، IP policy، paste ticket).

### نقاط ضعف
- پوشش تست Rust روی GUI/یکپارچگی نازک‌تر از پوشش منطق است (طبیعی برای Tauri، ولی E2E فقط با فعال‌شدن workflow `e2e.yml` اجرا می‌شود که هنوز فعال نیست).
- کشودن `Command::new("which")` در چند ماژول؛ `which` POSIX نیست و PATH-محور است (خطر کوچک hijack از طریق PATH کاربر — برای helperهای سیستمی مثل `pkexec` بهتر است مسیر مطلق `/usr/bin` بررسی شود).
- `ClipboardTab.tsx` بزرگ (گفتیم).

**ارزیابی: ۹/۱۰**

---

## ۳. امنیت و شبکه

### لایه‌های دفاعی پیاده‌شده (خیلی قوی)
1. **صفر شبکه در پیش‌فرض:** کلاینت HTTP (`reqwest`) فقط با feature اختیاری `gif-search` کامپایل می‌شود؛ باینری انتشار کلاینت خروجی ندارد.
2. **SSRF چندلایه** (`ssrf.rs` + `net_policy.rs`): فقط HTTPS، رد IP مستقیم، allowlist دامنه (tenor/giphy)، رد IPهای خصوصی/loopback/link-local/CGNAT/metadata (169.254.169.254) و IPv6 معادل‌ها، **DNS pinning** به آدرس‌های اعتبارسنجی‌شده، سیاست «هرگز redirect دنبال نکن». منبع واحد سیاست در `net_policy` تا ssrf و open_url هم‌راستا بمانند.
3. **رمزنگاری at-rest:** ChaCha20-Poly1305 روی ستون‌های متنی + تصاویر، قالب نسخه‌دار `W11E1`، zeroize کلید، fail-closed (خطای decrypt هرگز به متن خام نمی‌انجامد)، نشانگر یکپارچگی کلید `history.key.check`، پشتیبانی Secret Service (GNOME Keyring/KWallet).
4. **حریم خصوصی:** فیلتر خودکار اسرار (JWT، کلید خصوصی، `password=`)، حذف capture روی پنجره‌های مدیر گذرواژه/incognito، سقف ۲۰۰۰ آیتم، فایل‌ها با مجوز 0600.
5. **IPC سخت‌گیرانه:** CSP بدون `unsafe-eval`، `withGlobalTauri: false`، سه Capability مجزا، `window_policy` deny-by-default، paste فقط با بلیت nonce یک‌بارمصرف.
6. **سیستم:** پروفایل AppArmor (complain پیش‌فرض، enforce اختیاری)، udev با `uaccess` (نه world-writable)، محیط wrapper پاک‌سازی‌شده (`unset LD_PRELOAD` و…).
7. **زنجیرهٔ تأمین:** اکشن‌های پین‌شده به SHA، `cargo audit`/`cargo deny`/`npm audit` مسدودکننده، SBOM (SPDX per-artifact)، گواهی SLSA provenance، SHA256SUMS اجباری در نصاب، امضای اختیاری GPG، AUR fail-closed با known_hosts تأییدشده.

### یافته‌های امنیتی (به ترتیب شدت)
| # | شدت | یافته |
| --- | --- | --- |
| S1 | 🔴 بالا | **ورک‌فلوهای زندهٔ `.github/workflows/` هنوز هویت پیش از rename دارند.** `release.yml` به مخزن حذف‌شدهٔ `Modern-Clipboard-History-For-Linux` اشاره می‌کند (`REPO` در خط ۳۵ + لینک‌های raw.githubusercontent در متن Release). اگر آن نام‌فضا در GitHub آزاد شود، لینک‌های نصاب داخل متن Releaseهای آینده به مخزن قابل‌ربایش اشاره می‌کنند. همچنین job ی AUR بستهٔ `win11-clipboard-history-bin` را push می‌کند که PKGBUILD واقعی نام دیگری دارد. |
| S2 | 🟠 متوسط | `stale.yml` از `actions/stale@v11` **بدون پین SHA** استفاده می‌کند — نقض صریح قرارداد زنجیرهٔ تأمین خود پروژه (README: «Every GitHub Action is pinned to a full commit SHA»). |
| S3 | 🟠 متوسط | CI روی master برای ۵ اجرای اخیر **قرمز** است (شکست در مرحلهٔ Install Rust از اکشن شخص ثالث `dtolnay/rust-toolchain`). نسخهٔ hardened در docs همین وابستگی را حذف و با rustup + retry جایگزین کرده — ولی فعال نشده. CI قرمز یعنی گیت‌های امنیتی (audit/deny/clippy) عملاً اجرا نمی‌شوند. |
| S4 | 🟡 کم | `Command::new("which")` PATH-محور برای یافتن helperها (از جمله پیش از `pkexec setfacl`)؛ توصیه: probing مسیر مطلق یا `command -v` در شل ثابت. |
| S5 | 🟡 کم | `style-src 'unsafe-inline'` در CSP (برای Tailwind inline style رایج/قابل‌قبول است؛ ثبت به‌عنوان تصمیم آگاهانه در ADR خوب است). |
| S6 | ℹ️ یادداشت | Flatpak به‌طور ذاتی `/dev/uinput` نمی‌دهد؛ مستندات صادقانه است ولی `--device=all` کاربر عملاً sandbox را برای device باز می‌کند — این trade-off باید در متن Release دیده شود. |

**ارزیابی: ۸/۱۰** (طراحی ۹.۵، اجرا/عملیاتی‌سازی ۶ — به‌خاطر S1 تا S3)

---

## ۴. مستندات

### نقاط قوت
- README دوزبانه با جدول ویژگی‌ها، مسیرهای داده، سیاست حریم خصوصی شفاف.
- `docs/USER_GUIDE.fa.md` / `en.md`، `docs/fa/README.md` درگاه فارسی.
- ۸ ADR واقعی (SQLite، تزریق paste، SSRF pinning، رمزنگاری فیلدی، زنجیرهٔ تأمین CI، Secret Service، صفحه‌بندی IPC، ACL پنجره‌ها).
- THREAT_MODEL، ARCHITECTURE، PERFORMANCE budget، CI contract — همه زنده و دوزبانه.
- قالب‌های Issue/PR، SECURITY.md، CONTRIBUTING، CODEOWNERS با مسیرهای حساس، dependabot.
- سیاست آرشیو گزارش‌ها (`docs/reports/README.md`) — تصمیم مهندسی درست.

### نقاط ضعف (Drift حقیقت)
| # | یافته |
| --- | --- |
| D1 | README می‌گوید «The live `.github/workflows/` **already run** the hardened, canonical-named pipelines» — **نادرست است**؛ نسخه‌های زنده قدیمی‌اند و ۹۴ خط با نسخهٔ docs فاصله دارند. |
| D2 | CHANGELOG زیر `[Unreleased]` ادعا می‌کند رفع ۱۷ ارجاع legacy انجام و `check-packaging.sh` سبز و در CI سیم‌کشی شده است — **روی master رخ نداده** (من اجرا کردم: شکست می‌خورد؛ و در ci.yml زنده هیچ traceای از check-packaging نیست). اصلاحات فقط در `docs/github-workflows/` و پچ آماده exist دارند. |
| D3 | متن بدنهٔ Release در `release.yml` نام آرتیفکت‌ها را `win11-clipboard-history_*` می‌نویسد در حالی که Tauri واقعاً `windows-11-style-clipboard-history-manager_*` تولید می‌کند → دستور نصب داخل Releaseهای آینده ۴۰۴ می‌شود. |

**ارزیابی: ۸/۱۰** (غنا ۹.۵، صحت ۶)

---

## ۵. امتیازدهی

| محور | نمره | توضیح |
| --- | --- | --- |
| کیفیت کد و معماری | **۹/۱۰** | Separation of concerns نمونه‌ای؛ تسته؛ فقط چند بوی legacy |
| امنیت | **۸/۱۰** | طراحی چندلایهٔ کم‌نظیر؛ اجرا: CI قرمز + ورک‌فلو drift |
| مستندات | **۸/۱۰** | دوزبانه و غنی با ADR؛ اما ادعاهای ناسازگار با واقعیت مخزن |
| قابلیت توسعه (Scalability) | **۸/۱۰** | برای اپ تک‌کاربر دسکتاپ؛ matrix چند-DE/چند-آرش، feature-flag GIF، صفحهبندی IPC؛ ریسک: نگهداشت موازی دو مسیر بسته‌بندی و دو نسخهٔ workflow |
| **میانگین کلی** | **۸.۲۵ / ۱۰** | |

---

## ۶. فایل‌های اضافی، زائد و فرصت‌های بهینه‌سازی

### 🔴 بحرانی — پیش از هر Release
1. **فعال‌سازی ورک‌فلوهای hardened (اصلی‌ترین یافتهٔ این بازبینی):**
   ```bash
   git am docs/patches/hardened-ci-workflows.patch   # پچ تمیز اعمال می‌شود (بررسی شد)
   git push   # نیاز به توکن با scope: workflows
   ```
   این کار هم‌زمان S1، S3، D1، D2، D3 را می‌بندد و `check-packaging.sh` را سبز می‌کند (گیت شمارهٔ ۲ در اسکریپت دقیقاً همین drift را می‌گیرد).
2. **پین‌کردن `actions/stale` به SHA** در `stale.yml`.

### 🟠 حجم مخزن — ۴۱ مگابایت `.git`، عمدتاً یک فایل مرده
| فایل | حجم | وضعیت | اقدام |
| --- | --- | --- | --- |
| `docs/img/banner.gif` | **۳۴ MB** | **هیچ‌جا ارجاع ندارد** | حذف (+ در آینده `git filter-repo` برای کوچک‌کردن تاریخچه) |
| `docs/img/dynamic_themes.jpg` | ۳.۴ MB | ارجاع فقط به نسخهٔ png | حذف |
| `docs/img/dynamic_themes.png` | ۳.۷ MB | در README استفاده می‌شود | تبدیل به WebP (~۲۰۰KB) |
| `public/fonts/Vazirmatn-ExtraLight.woff2` | ۵۱ KB | هیچ `@font-face`ای استفاده نمی‌کند | حذف |
| فونت متغیر + ۵ وزن ایستا هم‌زمان | ~۱۵۰ KB اضافی | CSS هر دو را declare کرده | فقط فونت متغیر کافی است |

### 🟡 نگهداشت / جلوگیری از Drift
3. **`docs/github-workflows/` (کپی آینه‌ای) و `docs/patches/`:** پس از فعال‌سازی، دو نسخه دوباره drift می‌کنند (همین حالا ۹۴ خط فاصله دارند). پیشنهاد: یک step در CI که `diff .github/workflows/ci.yml docs/github-workflows/ci.yml` را بلاک کند، یا کپی‌ها و پچ را پس از فعال‌سازی حذف کنید و منبع حقیقت فقط `.github/workflows/` باشد.
4. **`Dockerfile`:** در هیچ CI/Makefile/سندی ارجاع ندارد. یا به‌عنوان build-env در CI استفاده کنید یا حذفش کنید.
5. **`tree-sitter` + `tree-sitter-rust` در devDependencies (~۲۰MB ماژول native):** فقط برای گیت سینتکس Rust که `cargo check/fmt` (که در CI همین‌طور اجرا می‌شود) با کیفیت بهتر انجامش می‌دهد. حذف = نصب سریع‌تر CI و توسعه‌دهنده. (پس از حذف، `npm run lint` و docs/CI.md را هم به‌روز کنید.)
6. **`docs/archive/reports/` (۱۶ فایل، ~۱۶۸KB):** سیاست آرشیو درست است؛ حجم مشکل نیست. اختیاری: به یک branch جدا یا خارج از مخزن منتقل شود.
7. **`public/vite.svg` + `<link rel="icon" href="/vite.svg">`:** آیکون پیش‌فرض Vite برای اپ دسکتاپ بی‌معناست؛ به آیکون خود اپ تغییر یابد.
8. **`.gitattributes`:** `package.json linguist-generated` و `*.json linguist-generated` همهٔ JSONهای دست‌نویس (مثل `tauri.conf.json`، `src/locales/*.json`) را از آمار/blame GitHub مخفی می‌کند — پیشنهاد: محدود به `package-lock.json` و `Cargo.lock`.
9. **API تاریخی `history.json`:** تغییر نام پارامتر `persistence_path` به `legacy_json_path` یا انتقال منطق migrate به سازندهٔ صریح، خوانایی را بالا می‌برد.
10. **دو مسیر بسته‌بندی:** تصمیم صریح بگیرید (Tauri-bundle = Release، debhelper = PPA/Debian رسمی) و در `packaging/DEPLOYMENT.md` بنویسید — الان مبهم است.

---

## ۷. آمادگی انتشار (.deb / .rpm / .AppImage / Flatpak)

### وضعیت فعلی
- هنوز **هیچ Release‌ای** در GitHub منتشر نشده (`gh release list` خالی است) و CI قرمز است.

### ارزیابی تک‌به‌تک
| هدف | وضعیت | جزئیات |
| --- | --- | --- |
| **.deb** | ✅ آماده (پس از اصلاح ورک‌فلو) | هدف `deb` در tauri.conf با wrapper، udev rule، desktop entry، آیکون‌ها، postinst، وابستگی‌های درست (webkit2gtk4.1/gtk3/appindicator)؛ مسیر موازی debhelper هم موجود؛ `scripts/verify-deb.sh` برای QA؛ معماری amd64+arm64 در ماتریکس CI |
| **.rpm** | ✅ آماده (پس از اصلاح ورک‌فلو) | هدف `rpm` با depends معتبر Fedora (webkit2gtk4.1, gtk3, polkit, libayatana)؛ همان فایل‌های سیستمی deb (گارد برابری ساختاری در check-packaging) |
| **.AppImage** | ✅ آماده (پس از اصلاح ورک‌فلو) | هدف فعال؛ wrapper به‌طور خاص محیط AppImage را مدیریت می‌کند (`WEBKIT_DISABLE_DMABUF_RENDERER` روی NVIDIA/AppImage) — نکتهٔ بالغ و درست |
| **Flatpak** | ⚠️ شرطی | manifest + metainfo + build.sh موجود و با گارد QA هم‌گام است، **اما**: (۱) uinput در سندباکس Flathub ممنوع است → paste کامل کار نمی‌کند مگر `--device=all` کاربر؛ (۲) انتشار روی Flathub نیازمند submit جداگانه و بازبینی است؛ (۳) `runtime-version 46` را به 48 ارتقا دهید؛ (۴) هیچ job ی CI manifest فلت‌پک را build/اعتبارسنجی نمی‌کند — یک workflow جدا اضافه شود |

### چک‌لیست پیشنهادی انتشار (ترتیبی)
1. `git am docs/patches/hardened-ci-workflows.patch` و push (توکن با scope `workflows`) → CI سبز شود.
2. پین SHA برای `actions/stale`.
3. حذف `banner.gif`، `dynamic_themes.jpg`، فونت ExtraLight؛ بهینه‌سازی `dynamic_themes.png`.
4. اجرای `npm run qa:packaging` (باید سبز شود) و `make qa` محلی.
5. تأیید متن بدنهٔ Release در workflow جدید: نام آرتیفکت‌ها = نام واقعی تولیدی Tauri؛ لینک نصاب = مخزن فعلی.
6. تنظیم secretهای اختیاری: `AUR_KNOWN_HOSTS` (وگرنه AUR skip می‌شود — fail-closed درست)، `RELEASE_GPG_PRIVATE_KEY` (اختیاری).
7. تگ `v2.5.0` → ساخت خودکار deb/rpm/AppImage دو-معماری + SHA256SUMS + SBOM + provenance.
8. برای Flatpak: ارسال به Flathub با مستندات شفاف محدودیت uinput (یا انتشار فقط لینک دستی build).

**نتیجه: بله — زیرساخت انتشار هر چهار قالب موجود و باکیفیت است، اما تا زمان اعمال آیتم‌های ۱ و ۲ بالا (ورک‌فلوهای قدیمی با نام‌های اشتباه و لینک‌های شکسته)، انتشار باعث تولید Release خراب می‌شود. ابتدا CI را سبز کنید، بعد تگ بزنید.**

---

## جمع‌بندی نهایی

این پروژه از نظر **طراحی امنیتی و مهندسی مستندات در بالاترین جیمی از پروژه‌های متن‌باز دسکتاپ** قرار می‌گیرد (SSRF چندلایه، رمزنگاری at-rest با zeroize، ACL پنجره‌ها، بلیت paste، ADR، مدل تهدید). مشکل اصلی نه در طراحی، بلکه در **آخرین گام عملیاتی‌سازی** است: نسخهٔ hardened ورک‌فلوها ساخته شده ولی هرگز روی `.github/workflows/` فعال نشده و CI به همین دلیل قرمز است؛ CHANGELOG و README هم این واقعیت را به‌درستی منعکس نمی‌کنند. با اعمال یک پچ آماده + کمی پاک‌سازی (~۴۱MB فایل مرده)، این مخزن یک انتشار درجه‌یک خواهد داشت.

*گزارش تولیدشده توسط Arena Agent — ۲۰۲۶-۰۸-21*
