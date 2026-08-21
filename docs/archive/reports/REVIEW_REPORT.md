# 🔍 گزارش بررسی فنی و امنیتی — Windows 11 Style Clipboard History Manager

> **مخزن:** `github.com/Mahdi-Arts/Windows-11-Style-Clipboard-History-Manager`
> **نسخه:** 2.0.0 | **تاریخ بررسی:** 2026-08-20
> **روش بررسی:** تحلیل استاتیک کامل کد (Rust + TypeScript/React)، اجرای تست‌ها و لینت‌ها، بررسی CI/CD، پیکربندی بسته‌بندی و مستندات.

---

## ۱. تحلیل معماری و ساختار

### ۱.۱ نمای کلی

این پروژه یک **مدیر تاریخچه کلیپ‌بورد** برای لینوکس است که تجربه Win+V ویندوز ۱۱ را با **Tauri v2** شبیه‌سازی می‌کند:

| لایه | فناوری | نقش |
| --- | --- | --- |
| **UI** | React 19 + TypeScript (strict) + Tailwind 4 + Vite 7 | سه پنجره: اصلی، Setup Wizard، تنظیمات |
| **Backend** | Rust + Tauri v2 | ۲۷ ماژول تک‌وظیفه‌ای (single-concern) |
| **Clipboard I/O** | arboard + `wl-copy`/`xclip` | خواندن/نوشتن کلیپ‌بورد با fallback |
| **Persistence** | SQLite (WAL) + فایل PNG + JSON اتمیک | تاریخچه، تصاویر، تنظیمات |
| **Input Injection** | uinput (Wayland) / XTest (X11) | شبیه‌سازی Ctrl+V برای Paste |
| **بسته‌بندی** | deb / rpm / AppImage / AUR / Flatpak + Dockerfile | توزیع چند-توزیعی |

### ۱.۲ نقاط قوت معماری

- **لایه‌بندی تمیز و انضباط‌شده:** `main.rs` فقط bootstrap است؛ منطق دامنه در ماژول‌های `lib` قرار دارد؛ `commands.rs` صرفاً wrapperهای باریک Tauri است؛ state مشترک (`AppState`) به‌صورت تزریق‌شده (DI) به commandها می‌رسد؛ خطاها از طریق `AppError` (thiserror) یکپارچه‌اند.
- **پایداری ذخیره‌سازی:** SQLite با حالت WAL و upsert افزایشی (به‌جای بازنویسی کامل)، نوشتن اتمیک JSON (`fs_atomic`)، اعمال مجوز `0600/0700` بر همه فایل‌های حساس، و تصاویر با thumbnail جداگانه (انتقال فقط thumb از طریق IPC — طراحی درست برای ۲۰۰۰ آیتم).
- **مدیریت منابع:** polling تطبیقی کلیپ‌بورد (200ms فعال / 800ms بیکار)، اتصال X11 کش‌شده، warmup در threadهای جداگانه، `paste_gate` برای سریال‌سازی تراکنش paste، single-instance.
- **مجازی‌سازی لیست** با `react-window` که برای سقف ۲۰۰۰ آیتم ضروری است.
- **بسته‌بندی کامل:** CI ماتریسی x86_64 + aarch64، خودکارسازی Cloudsmith و AUR، manifest فلت‌پک محافظه‌کارانه (بدون `--device=all` پیش‌فرض).

### ۱.۳ نقاط ضعف معماری

- **ماژول‌های غول‌پیکر:** `linux_shortcut_manager.rs` (~۱۶۲۰ خط)، `clipboard_manager.rs` (~۱۳۰۰ خط)، `shortcut_conflict_detector.rs` (~۷۹۰ خط) و `SettingsApp.tsx` (~۱۱۰۰ خط) — منسجم هستند اما از مرز خوانایی/قابلیت تست عبور کرده‌اند.
- **تاریخچه گیت تک-کامیتی** — امکان ارزیابی روند توسعه و بازبینی (reviewability) وجود ندارد.
- ناسازگاری نام‌گذاری: شناسه برنامه `dev.gustavosett.*` است در حالی که مخزن `Mahdi-Arts/...` است (با fork شدن پروژه به‌روز نشده).
- `window_identity.rs` در هر poll یک اتصال X11 جدید می‌سازد (هزینه اضافه کوچک).

---

## ۲. بررسی کیفیت کد (Code Quality)

### ۲.۱ نتایج اجرای عملی (این بررسی)

| بررسی | نتیجه |
| --- | --- |
| `npm run lint` (tsc + ESLint، `--max-warnings 0`) | ✅ پاس |
| `npx tsc --noEmit` | ✅ پاس (strict: true، noUnusedLocals/Parameters) |
| `npm test` (Vitest) | ✅ ۴ فایل، **۴۲ تست پاس** |
| تست‌های Rust | ۳۲ تابع `#[test]` (اجرا در CI: `cargo test`) |
| `cargo fmt --check` + `cargo clippy -D warnings` | در CI الزامی و پاس |

### ۲.۲ نقاط قوت

- **پروژه عملاً Clean Code است:** ماژول‌ها تک‌مسئولیت، نام‌گذاری گویا، کامنت‌های doc (دوزبانه) تقریباً روی همه فایل‌ها، خطاهای typed به‌جای String، عدم استفاده از `unwrap()` در مسیرهای production (بیشتر unwrapها داخل تست‌هاست).
- **امنیت دفاعی در لایه کد:** `looks_like_secret` با تست، regex safety برای جستجوی کاربر (`historySearch.ts` با محدودیت طول و تو در تویی)، URL sanitizer با ۱۸ تست.
- **پارگی i18n صفر:** ۱۳۶ کلید en و ۱۳۶ کلید fa با تطابق کامل.
- ESLint با react-hooks/recommended و strict TS — استاندارد enterprise.
- پروفایل release بهینه (`lto`, `opt-level z`, `codegen-units 1`, `panic=abort`).

### ۲.۳ نقاط ضعف

- **پوشش تست نامتوازن:** تست‌ها عمدتاً unit-level روی ابزارهاست؛ هیچ تست کامپوننت (Testing Library) یا تست E2E وجود ندارد. بحرانی‌ترین ماژول‌ها (`clipboard_manager`، `input_simulator`، `linux_shortcut_manager`) تست‌های محدودی دارند.
- **بدون سنجش پوشش (coverage):** هیچ گیت درصد پوشش در CI نیست؛ پوشش می‌تواند بدون اطلاع افت کند.
- باقی‌مانده `eprintln!` در `focus_manager.rs` به‌جای `tracing` (ناسازگاری کوچک).
- نسخه Rust در CI با `stable` شناور است — برای reproducibility بهتر است `rust-toolchain.toml` پین شود.

---

## ۳. بررسی امنیت و شبکه

### ۳.۱ نقاط قوت (قابل توجه — بالاتر از میانگین پروژه‌های مشابه)

- **CSP محکم:** `script-src 'self'`، `withGlobalTauri: false`، capabilityها محدود به پنجره‌های مشخص و پلتفرم linux؛ `shell:allow-open` فقط scoped به `https/http/mailto`.
- **SSRF با استاندارد بالا** (`ssrf.rs`): HTTPS-only، allowlist میزبان، **DNS pinning** با `resolve_to_addrs` (بستن پنجره DNS-rebinding بین validate و connect)، رد مستقیم IP، بلاک‌لیست کامل آدرس‌های خصوصی/loopback/CGNAT/مستند، **عدم دنبال کردن redirect**، سقف ۱۰MB با stream و بررسی Content-Type.
- **فیلتر اسرار:** شناسایی PEM/کلید خصوصی، JWT، توکن‌های شناخته‌شده (GitHub, Slack, Stripe, AWS...) و `password=`؛ حذف منبع حساس (password managerها و پنجره‌های ناشناس) در X11 — همگی با تست.
- **نگهبان Paste:** تزریق Ctrl+V فقط ۵ ثانیه پس از یک نوشتن واقعی کلیپ‌بورد (`finish_paste`)، قفل سراسری paste، بازگردانی و **تأیید focus پنجره مقصد** در X11 (جلوگیری از چسباندن به پنجره اشتباه).
- **اجرای فرمان‌ها بدون shell:** همه فراخوانی‌های سیستمی با `Command::new` + argv هستند (پس از رفع باگ `sh -c` در نسخه ۲.۰) — بدون ریسک command injection.
- **بازنویسی کانفیگ WM ها به‌صورت opt-in** (`allow_wm_config_rewrite` پیش‌فرض off) + escape امن XML/INI.
- **محیط اجرای پاک:** `wrapper.sh` متغیرهای `LD_PRELOAD`/`LD_LIBRARY_PATH`/`GTK_PATH` را پاک می‌کند (جلوگیری از تزریق کتابخانه از Snap/Flatpak).
- مجوز فایل‌ها `0600` و دایرکتوری‌ها `0700`، سیاست انتشار مسئولانه (SECURITY.md + GitHub private advisory).

### ۳.۲ آسیب‌پذیری‌ها و ریسک‌های شناسایی‌شده

| # | ریسک | شدت | توضیح |
| --- | --- | --- | --- |
| 1 | **Auditهای CI غیرالزامی** — `cargo audit` و `npm audit` با `continue-on-error: true` اجرا می‌شوند؛ یعنی CI حتی با vulnerability معلوم سبز می‌ماند | **متوسط** | باید به گیت الزامی (gate) تبدیل شوند |
| 2 | **ناسازگاری زنجیره تأمین در `release.yml`** — متن Release، دستورات نصب، مخزن Cloudsmith و URL بسته AUR همگی به مخزن بالادستی `gustavosett/Windows-11-Clipboard-History-For-Linux` اشاره می‌کنند | **متوسط-بالا** | کاربرانی که از Releaseهای این fork نصب می‌کنند به دانلود از مخزن دیگر هدایت می‌شوند (ریسک اعتماد/تأمین) |
| 3 | **AUR PKGBUILD با checksum `SKIP`** در مخزن | متوسط | باید checksum واقعی داشته باشد (workflow فقط هنگام release آن را می‌نویسد) |
| 4 | **`curl \| bash` installer** — حتی با یادداشت «قبل از اجرا بخوانید»، SHA256SUMS فقط «در صورت انتشار» بررسی می‌شود | متوسط | احراز امضا/checksum باید اجباری باشد |
| 5 | **Google Fonts در runtime** — اتصال شبکه به `fonts.googleapis.com/gstatic` در هر اجرا (افشای IP + وابستگی به شبکه + نمی‌توان در حالت آفلاین فونت فارسی گرفت) | کم-متوسط | فونت Vazirmatn را به‌صورت محلی باندل کنید |
| 6 | **`style-src 'unsafe-inline'`** در CSP | کم | لازمه Tailwind است ولی در حالت ایده‌آل با hash/nonce محدود می‌شود |
| 7 | **قدرت ذاتی uinput** — برنامه توانایی تزریق کیبورد به هر پنجره‌ای دارد | ذاتی (مستند) | با udev rule از `uaccess` استفاده می‌شود (فقط کاربر لاگین‌شده)؛ قابل قبول اما باید AppArmor/seccomp هم فکر شود |
| 8 | فایل‌های `-wal`/`-shm` SQLite | کم | مجوز 0600 روی db اعمال می‌شود؛ بررسی شود companionها هم 0600 بمانند |

> نکته: این برنامه **مکانیزم احراز هویت ندارد** — و درست هم هست؛ مدل تهدید آن تک‌کاربره/محلی است. «امنیت» در این پروژه یعنی: حفاظت از داده محلی در برابر سایر کاربران سیستم (با 0600 ✓)، جلوگیری از خروج داده (بدون آپلود ✓)، و دفاع در برابر محتوای مخرب کلیپ‌بورد/URL (با sanitizer و SSRF ✓).

---

## ۴. ارزیابی مستندات (Documentation)

| سند | وضعیت |
| --- | --- |
| `README.md` | ✅ عالی: دوزبانه، ویژگی‌ها، حریم خصوصی، نصب برای هر توزیع، جدول میانبرها، جدول معماری، عیب‌یابی، راهنمای توسعه، جدول متغیرهای محیطی |
| `SECURITY.md` | ✅ دوزبانه با زمان‌بندی پاسخ و کانال‌های گزارش خصوصی |
| `CONTRIBUTING.md` + قالب‌های Issue/PR + FUNDING | ✅ کامل |
| `CHANGELOG.md` | ✅ نگه‌داری‌شده با دسته‌بندی (امنیت/پایداری/بسته‌بندی/تست) |
| `docs/BILINGUAL.md` | ✅ راهنمای فنی i18n |
| `packaging/README.md` + Flatpak/Debian/AUR | ✅ |
| `OPTIMIZATION_REPORT.md` / `UPGRADE_REPORT.md` | یادداشت‌های تصمیم — خوب اما ناقص |

**کمبودها:** لینک CONTRIBUTING در README وجود ندارد؛ تصاویر `docs/img/` در README استفاده نشده‌اند؛ سند مدل تهدید (Threat Model) رسمی وجود ندارد؛ هیچ ADR (سوابق تصمیم معماری) برای تغییرات بزرگ مثل مهاجرت JSON→SQLite ثبت نشده.

---

## ۵. امتیازدهی

| معیار | امتیاز از ۱۰ | دلیل |
| --- | --- | --- |
| **کیفیت کد و معماری** | **۸.۵** | لایه‌بندی عالی، Rust مدرن، خطاهای typed، strict TS؛ کسر برای ماژول‌های بزرگ و نبود تست کامپوننت/E2E |
| **امنیت** | **۸.۵** | SSRF با DNS pinning، CSP، فیلتر اسرار، paste guard — در سطح حرفه‌ای؛ کسر برای auditهای غیرالزامی CI و ناسازگاری URL در release |
| **مستندات** | **۸.۵** | README دوزبانه نمونه، SECURITY/CONTRIBUTING/CHANGELOG کامل؛ کسر برای نبود Threat Model و ADR |
| **قابلیت توسعه (Scalability)** | **۷.۵** | معماری ماژولار و CI چندمعماری، پایگاه داده افزایشی، ۲۰۰۰ آیتم با مجازی‌سازی؛ کسر برای ماژول‌های غول‌پیکر، نبود پوشش تست و وابستگی تک-نگهدارنده |
| **میانگین کلی** | **۸.۲۵ / ۱۰** | ✅ |

---

## ۶. پیشنهادات کلیدی برای ارتقاء به سطح Enterprise

### پیشنهاد ۱ — بهداشت زنجیره تأمین (Supply Chain) — *اولویت: بحرانی*
- `release.yml` را کامل بازنویسی کنید تا همه URLها به `Mahdi-Arts/Windows-11-Style-Clipboard-History-Manager` اشاره کنند (متن Release، Cloudsmith، `.SRCINFO`). انتشار از مخزن اشتباه برای کاربران = ریسک اعتماد جدی.
- `cargo audit` و `npm audit` را **الزامی** کنید (حذف `continue-on-error` یا ساخت job جداگانه با آستانه fail)، و `cargo-audit` را در CI کش کنید.
- به هر Release **مهر Provenance** اضافه کنید: `actions/attest-build-provenance` (یا حداقل همیشه-الزامی `SHA256SUMS` + امضای GPG)، و در `install.sh` بررسی checksum را اجباری کنید نه مشروط.
- در `aur/PKGBUILD` به‌جای `SKIP` از checksum واقعی استفاده کنید و `rust-toolchain.toml` برای پین نسخه Rust اضافه کنید.

### پیشنهاد ۲ — حریم خصوصی شبکه: باندل کردن فونت
- فایل‌های Vazirmatn را به `public/fonts/` اضافه کنید و لینک Google Fonts را از `index.html` حذف کنید.
- نتیجه: **بدون هیچ اتصال شبکه‌ای در اجرای عادی** (امکان ادعای «کاملاً آفلاین»)، CSP سخت‌تر (`font-src 'self'`)، کارکرد آفلاین و حذف یکی از معدود نشت‌های داده.

### پیشنهاد ۳ — پلتفرم تست Enterprise
- پوشش کد را با آستانه الزامی اضافه کنید (`vitest --coverage` + `cargo llvm-cov`).
- تست کامپوننت با Testing Library برای `HistoryItem`/`SearchBar`/`SettingsApp` و یک تست **E2E اسمُک** در CI روی Xvfb/Wayland headless (باز کردن برنامه، کپی یک متن، تأیید ورود به تاریخچه، paste).
- برای ماژول‌های بحرانی (`privacy`، `ssrf`، `historySearch`) تست property-based (`proptest`) اضافه کنید.

### پیشنهاد ۴ — شکستن ماژول‌های بزرگ
- `linux_shortcut_manager.rs` (۱۶۲۰ خط) را به ماژول‌های per-DE تقسیم کنید: `de/gnome.rs`, `de/kde.rs`, `de/xfce.rs`, `de/tiling.rs` با یک trait مشترک — تست‌پذیری و نگه‌داری چند برابر می‌شود.
- `SettingsApp.tsx` را به بخش‌های (General/Privacy/Shortcuts/About) با hookهای اختصاصی تقسیم کنید.
- مستندات تصمیم‌ها را با **ADR** در `docs/adr/` شروع کنید (شروع با ADR برای مهاجرت SQLite و معماری input).

### پیشنهاد ۵ — سخت‌سازی استقرار (Enterprise Hardening)
- **SBOM در هر Release**: `syft` یا `cargo sbom` + `npm sbom` را به workflow اضافه کنید و به Release ضمیمه کنید.
- **پروفایل AppArmor/seccomp** برای deb/rpm و مستندسازی فلت‌پک به‌عنوان کانال پیشنهادی sandbox (با ذکر دقیق overrideهای لازم).
- سند **Threat Model** رسمی (`docs/THREAT_MODEL.md`) بنویسید: دارایی‌ها (تاریخچه کلیپ‌بورد، تصاویر، تنظیمات)، مرزهای اعتماد (کاربر محلی، X11 vs Wayland، محتوای مخرب کلیپ‌بورد) و کنترل‌های موجود — این سند برای ممیزی‌های امنیتی سازمانی ضروری است.
- گزینه رمزنگاری اختیاری تاریخچه (SQLCipher) برای کاربران حساس — با توجه به ماهیت «تاریخچه کلیپ‌بورد»، این یک تمایز رقابتی امنیتی واقعی است.

---

## جمع‌بندی

این پروژه در سطح **بالاتر از میانگین پروژه‌های اوپن‌سورس دسکتاپ** قرار دارد: معماری تمیز Rust/Tauri، امنیت دفاعی عمیق (SSRF با DNS pinning، فیلتر اسرار، paste guard)، مستندات دوزبانه مثال‌زدنی و CI چندمرحله‌ای. فاصله تا سطح Enterprise عمدتاً در **بهداشت زنجیره تأمین** (ناسازگاری URLهای release، auditهای غیرالزامی)، **پوشش تست** و **شکستن ماژول‌های بزرگ** است. اجرای پیشنهادهای ۱ تا ۳ بیشترین بازده را با کمترین هزینه دارد.

**امتیاز نهایی: ۸.۲۵ از ۱۰** ✅
