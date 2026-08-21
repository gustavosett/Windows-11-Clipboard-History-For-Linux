# بررسی فنی، امنیتی و معماری مخزن Windows-11-Style-Clipboard-History-Manager

**بسم الله الرحمن الرحیم**

| | |
|---|---|
| **پروژه** | Windows 11 Style Clipboard History Manager |
| **نسخه** | 2.5.0 |
| **تاریخ بازبینی** | 2026-08-21 |
| **روش** | بازبینی ایستا (Static Review) از روی کد، پیکربندی، بسته‌بندی و مستندات |
| **محدودهٔ تحلیل** | ~۱۱٬۷۰۰ خط Rust، ~۹٬۳۰۰ خط TypeScript/TSX، بسته‌بندی Debian/RPM/AppImage/Flatpak/AUR، CI/CD و ۸ سند ADR |

> نکته: این گزارش به‌صورت مستقل و از روی **کد واقعی** تهیه شده، نه فقط از گزارش‌های قبلی. در همین نشست، نام‌گذاری باینری و بسته‌ها به `windows-11-style-clipboard-history-manager-bin` و `windows-11-style-clipboard-history-manager` یکپارچه شد که یافته‌های گزارش پیشین را برطرف می‌کند.

---

## ۱. تحلیل معماری و ساختار

### نمای کلی

معماری **دو لایهٔ روشن و درست** دارد:

- **Backend (Rust + Tauri v2):** تمام منطق سیستم‌محور (کلیپ‌بورد، میانبرهای سراسری، تزریق paste، رمزنگاری، مجوزها، تشخیص دسکتاپ، persistence) در ماژول‌های Rust زیر `src-tauri/src/` قرار دارد.
- **Frontend (React 19 + TypeScript):** رابط کاربری WebView با `withGlobalTauri: false` (بدون expose کردن API به global) و جداسازی هوک‌ها / سرویس‌ها / کامپوننت‌ها / ابزارها.

### نقاط قوت معماری

1. **جداسازی مسئولیت‌ها (SRP):** زیرماژول‌های `clipboard_manager/` (persistence، history_access، deduplication، clipboard_write، types) به‌درستی تفکیک شده‌اند. جدا از آن، ماژول‌های مستقلی برای SSRF، شبکه، رمزنگاری، فایل اتمی، امنیت، مدیر کلید، مدیر ایموجی/GIF و هویت پنجره وجود دارد.
2. **مهاجرت داده:** گذار از `history.json` به SQLite (`history.db`) با `persistence.rs` و یک بارمهاجرتیِ امن انجام شده است (فایل JSON قدیمی حفظ می‌شود).
3. **قابلیت «feature-gating»:** دانلود GIF (شبکه‌ای) پشت feature `gif-search` قرار دارد؛ build پیش‌فرض تقریباً هیچ کلاینت HTTP ندارد — یعنی سطح حملهٔ شبکه در نصب عادی حداقلی است. این یک تصمیم معماری بسیار خوب است.
4. **پایداری داده:** استفاده از SQLite/WAL با صفحه‌بندی IPC (ADR-0007) به‌جای بازنویسی کامل یک فایل JSON.
5. **تزریق وابستگی برای تست‌پذیری** در بخش‌هایی (clock، clipboard backend) رعایت شده، هرچند کامل نیست.

### ضعف‌های معماری

1. **فایل‌های بزرگ (God objects):** `input_simulator.rs` (~۷۵۰ خط)، `history_crypto.rs` (~۷۴۰ خط)، `SetupWizard.tsx` (~۷۵۰ خط)، `commands.rs` (~۵۶۰ خط). نگهداری و تست این فایل‌ها را دشوار می‌کند.
2. **تکرار کد:** دو پیاده‌سازی `SymbolPicker` در `components/SymbolPicker.tsx` و `components/common/SymbolPicker.tsx` وجود دارد؛ نسخهٔ بلااستفاده ریسک drift دارد.
3. **ناسازگاری تاریخی نام‌ها** — که در همین نشست اصلاح شد (باینری، بسته‌ها، مسیرهای داده و اسکریپت‌ها به نام یکسان `windows-11-style-clipboard-history-manager` رسیدند).
4. **وابستگی مستقیم به فرمان‌های سیستم** (Command, fs, global state) در بسیاری از ماژول‌ها؛ تست integration را سخت می‌کند.

---

## ۲. بررسی کیفیت کد (Code Quality)

### نقاط قوت

- **TypeScript strict** + ESLint type-aware با `--max-warnings 0`؛ اجرای `tsc --noEmit` و `vite build` بدون خطا.
- **Rust** از `thiserror` و `AppError` یکپارچه استفاده می‌کند؛ بیشتر عملیات حساس error propagation دارند و به صورت `Result` برگردانده می‌شوند.
- **SQL پارامتری** — هیچ نشانه‌ای از SQL injection دیده نشد.
- **نوشتن اتمیک تنظیمات** با محدودسازی مجوز فایل‌های حساس.
- کامنت‌های دوزبانه (فارسی/انگلیسی) در جاهای حساس، خوانایی را بالا برده‌اند.
- تست‌های واحد برای URL safety، جستجو، صفحه‌بندی، smart actions، کلیپ‌بورد و چند کامپوننت.

### ضعف‌ها

1. **حذف بی‌صدأ خطای decode در بارگذاری تاریخچه:** در `persistence.rs`، ردیف‌هایی که decode نمی‌شوند با `continue` ساکت رها می‌شوند (بدون گزارش یا quarantine). در صورت خرابی رمزنگاری یا فساد ردیف، داده بی‌صدا گم می‌شود.
2. **برخی `expect` در مسیر runtime** (مثل `Connection::open_in_memory().expect(...)`) که در bootstrap شاید قابل قبول‌اند اما در watcher/device lifecycle باید graceful باشد.
3. **پیام‌های خطای backend به‌صورت `String`** به UI می‌رسد؛ برای ترجمه و telemetry بهتر است error code ساخت‌یافته برگردد.
4. **پوشش تست انتخابی و محدود:** بخش بزرگی از Setup، Settings، watcher، input simulation و packaging اندازه‌گیری نمی‌شود.
5. **ادعای `zeroize`:** کامنت `history_crypto.rs` ادعای فعال بودن `zeroize` در گراف وابستگی را دارد اما dependency به‌صورت صریح feature را فعال نمی‌کند؛ باید با تست/پیکربندی اثبات شود.

---

## ۳. بررسی امنیت و شبکه

### نقاط قوت (قابل توجه)

1. **CSP سخت:** `script-src 'self'` و `withGlobalTauri: false` — هیچ `eval` / `dangerouslySetInnerHTML` در فرانت‌اند نیست (بررسی شد، نتیجه خالی بود).
2. **رمزنگاری در حالت استراحت:** ستون‌های حساس با **ChaCha20-Poly1305** و nonce تصادفی (قالب `W11E1 || nonce(12) || ciphertext`). محدودیت مجوز ۰۶۰۰ برای DB/WAL/SHM/key/settings.
3. **بک‌اند کلید Secret Service:** آپشن اختیاری که کلید را روی دیسک نمی‌نویسد (فقط در GNOME Keyring/KWallet). با marker از تعویض کلید جلوگیری می‌شود.
4. **دفاع SSRF واقعاً خوب:** در `ssrf.rs` — فقط HTTPS، **allowlist میزبان** (tenor/giphy)، رد IP مستقیم و dotted، **پین‌کردن DNS به آدرس‌های تأییدشده**، `redirect: none`، رد خصوصی/لوپ‌بک/link-local/metadata/CGNAT. آزمون‌های واحد این موارد را پوشش می‌دهند.
5. **ACL مرکزی پنجره (deny-by-default):** `window_policy.rs` با نقش‌های `Main/Settings/Setup`. بررسی شد که تقریباً **همه**ٔ فرمان‌های تغییردهندهٔ وضعیت (clear، delete، paste، autostart، تنظیمات، setup، resolve_conflicts، reset_first_run، fix_permissions) با `require_*` محافظت می‌شوند.
6. **فیلتر اسرار:** `privacy.rs` با الگوهای private key، JWT، token، password assignment و تشخیص پنجرهٔ password-manager؛ در Wayland با توضیح UI محدودیت ذکر شده.
7. **Safe URL opener:** محدود به `https`/`mailto`، رد credential، کنترل کاراکتر، طول ۲۰۴۸، رد میزبان‌های داخلی — هم در Rust هم در TS.
8. **تزریق paste محافظت‌شده:** پنجرهٔ `main`، تأیید نوشتن اخیر کلیپ‌بورد در ۵ ثانیه اخیر، serialize با paste_gate و مکانیزم ticket برای GIF.
9. **Supply-chain:** اکشن‌های CI به **full commit SHA پین** شده‌اند، `npm audit`/`cargo audit`/`cargo deny`، SBOM (syft) و attest-build-provenance (SLSA) در release فعال است.
10. **Lint قراردادی اسرار در بسته** و بررسی دستورالعمل (README آپشنال GPG با `WINDOWS_11_CLIPBOARD_TRUST_KEY`).

### یافته‌های مهم امنیتی

1. **شکاف «resolution بعد از DNS» در `open_url.rs`:** این مسیر فقط **متنِ host** را بررسی می‌کند (IP مستقیم)، نه آدرس‌های **بعد از DNS resolution**. یک hostname معمولی که بعداً به IP خصوصی resolve شود (مثلاً رکوردی عمومی که به شبکهٔ داخلی اشاره دارد) از این validation عبور می‌کند. با این‌که `xdg-open` مرورگر خارجی را باز می‌کند و خود برنامه درخواست HTTP نمی‌فرستد، ولی مدل تهدید باید این را صریح کند یا مانند `ssrf.rs` آدرس‌های resolve شده را هم بررسی کند. (خطر: کم — مرورگر/سیاست کاربر.)
2. **`stale.yml` پین نشده:** از `actions/stale@v11` (تگ متغیر) استفاده می‌کند، در حالی که بقیهٔ اکشن‌ها به SHA پین شده‌اند — ناسازگار با سیاست بقیهٔ مخزن.
3. **مجوز قدرتمند `/dev/uinput`:** ذاتاً بالاست (قابلیت تزریق ورودی). با paste-gate و AppArmor complain-mode محدود شده اما AppArmor در حالت enforce در بسته‌های رسمی **آزموده/اجباری نشده** است.
4. **Release fail-closed ناقص:** مرحله‌های Cloudsmith/AUR به‌صورت opt-in با `continue-on-error` / skip روی نبود secret انجام می‌شود — طبق گزارش خودِ پروژه هنوز کاملاً با ادعای fail-closed هم‌خوانی ندارد.
5. **`style-src 'unsafe-inline'`:** برای styling فعلی قابل دفاع است ولی بهتر است nonce/hash بررسی شود.
6. **برخی فرمان‌های read-only** (مثل `get_system_theme`) بدون ACL هستند — ریسک پایین ولی برای policy یکنواخت بهتر است همهٔ فرمان‌ها نقش داشته باشند.

---

## ۴. ارزیابی مستندات (Documentation)

### نقاط قوت

- **README جامع:** معرفی، نصب (Deb/RPM/Flatpak/AUR/AppImage)، حریم خصوصی، معماری، میانبرها، رفع اشکال و توسعه.
- **معماری و امنیت:** `ARCHITECTURE.md`، `THREAT_MODEL.md`، `CI.md`، `PERFORMANCE.md`.
- **۸ سند ADR** برای تصمیمات کلیدی (SQLite، معماری تزریق paste، SSRF/DNS-pinning، رمزنگاری فیلد، CI supply-chain، Secret Service، صفحه‌بندی IPC، ACL پنجره).
- **راهنمای دوزبانهٔ مستقل** `USER_GUIDE.en.md` و `USER_GUIDE.fa.md` + مستندات BILINGUAL و فونت محلی Vazirmatn.
- **مستندات امنیتی:** `SECURITY.md`، `CODEOWNERS`، قالب‌های Issue/PR.
- مستندات گزارش قبلی (`REPOSITORY_REVIEW`، QA، ENTERPRISE_UPGRADE) نشان‌دهندهٔ بلوغ فرایند.

### شکاف‌ها

1. **تمام ADRها و Threat Model به‌صورت خط‌به‌خط دوزبانه نیستند** — برخی انگلیسی، برخی ترکیبی.
2. **CHANGELOG برای همهٔ ورودی‌ها ترجمهٔ کامل دوسویه ندارد.**
3. برخی ادعاهای README (نام باینری، zero-network، fail-closed) به‌صورت machine-verifiable در CI چک نمی‌شوند (پس از اصلاح نام، contract باینری اکنون در `check-packaging.sh` چک می‌شود).
4. پیشنهاد: لینک انتخاب زبان در ابتدای README قرار گیرد.

---

## ۵. امتیازدهی

| معیار | امتیاز از ۱۰ | خلاصهٔ دلیل |
| --- | ---: | --- |
| کیفیت کد و معماری | **8.5** | معماری دو لایهٔ روشن، ماژولار و feature-gated؛ اما فایل‌های بزرگ، حذف بی‌صدأ خطا در load_rows و تکراری در SymbolPicker |
| امنیت | **8.5** | SSRF با DNS pinning، رمزنگاری ChaCha20-Poly1305 + Secret Service، ACL deny-by-default، CSP سخت و CI پین‌شده؛ اما شکاف post-DNS در open_url، stale.yml پین‌نشده و uinput/AppArmor |
| مستندات | **8.5** | README/THREAT_MODEL/۸ ADR/راهنمای دوزبانه؛ اما ترجمهٔ کامل همهٔ اسناد و CHANGELOG ناقص |
| قابلیت توسعه (Scalability) | **8.0** | صفحه‌بندی IPC، SQLite/WAL و جداسازی ماژول خوب؛ اما تست integration/benchmark ناقص و abstraction سیستم‌عامل کامل نیست |
| **میانگین کل** | **8.4** | میانگین حسابی چهار معیار |

---

## ۶. پیشنهادهای کلیدی برای رسیدن به سطح Enterprise

### اولویت P0 (فوری)

1. **اثبات/اصلاح ادعای `zeroize`** در `history_crypto.rs` — یا feature را صریحاً در `Cargo.toml` فعال کنید یا ادعا را بردارید؛ امنیت ادعایی نباید بدون شواهد بماند.
2. **بستن شکاف `open_url.rs`** — آدرس‌های resolve شده را هم (مثل `ssrf.rs`) با `is_disallowed_ip` چک کنید، تا hostnameهای سوییچ‌شونده به IP خصوصی رد شوند.
3. **پین کردن `stale.yml`** به SHA کامل برای انطباق با سیاست بقیهٔ مخزن.
4. **quarantine/گزارش ردیف‌های خراب** در `load_rows` به‌جای `continue` ساکت، و افزودن تست برای آن.

### اولویت P1

5. **error contract ساخت‌یافته** (`code`, `message_key`, `debug_detail`) برای پیام‌های backend تا قابل ترجمه و redaction باشند.
6. **تست integration واقعی** روی Xvfb/X11 و ماتریس چند Wayland compositor (میانبر، watcher، paste، focus restore، cleanup).
7. **Secret Service را backend پیشنهادی** کنید و recovery/export امن کلید را مستند کنید.
8. **fuzz/property test** برای URL parser، secret detector، legacy migration و encryption envelope.
9. **AppArmor را در حالت enforce** روی بستهٔ رسمی آزموده و به‌صورت اختیاری فعال کنید؛ `/dev/uinput` را در مدل تهدید دقیق مدیریت کنید.

### اولویت P2

10. **شکستن فایل‌های بزرگ** (`input_simulator.rs`، `history_crypto.rs`، `SetupWizard.tsx`، `commands.rs`) به adapter/service/policy/test module.
11. **حذف `SymbolPicker` تکراری** و dead code؛ enforce کردن boundary وابستگی با lint.
12. **reproducible build + امضای اجباری release + rotation کلید + branch protection** کامل.
13. **نسخهٔ کامل فارسی Threat Model و ADRها** و کنترل parity در CI.

---

## شواهد اجرا در این نشست

| بررسی | نتیجه |
| --- | --- |
| `npm run lint` | موفق (zero warnings) |
| `npm run build` | موفق |
| `npm test` | ۱۲ فایل / ۸۹ تست موفق |
| `bash scripts/check-packaging.sh` | موفق — قرارداد بسته‌بندی سازگار |
| `cargo test` / `cargo clippy` | در این محیط اجرا نشد (Rust toolchain نصب نبود) — بر اساس بازبینی ایستا |

> این گزارش جایگزین penetration test روی باینری بسته‌بندی‌شده، تست واقعی چند compositor یا ممیزی مستقل رمزنگاری نیست.

**یا علی مدد.**
