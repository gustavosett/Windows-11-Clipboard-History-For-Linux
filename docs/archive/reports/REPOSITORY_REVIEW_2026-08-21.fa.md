# گزارش بازبینی فنی، امنیتی و دوزبانگی مخزن

**پروژه:** Windows 11 Style Clipboard History Manager  
**نسخه:** 2.5.0  
**تاریخ بازبینی:** 2026-08-21  
**دامنه:** کد React/TypeScript، هستهٔ Rust/Tauri، بسته‌بندی، CI/CD، امنیت، شبکه، رابط دوزبانه و مستندات

## خلاصهٔ مدیریتی

پروژه از نظر طراحی یک برنامهٔ دسکتاپ لینوکسی، بالاتر از میانگین پروژه‌های متن‌باز هم‌اندازه است. جداسازی React و Rust، SQLite صفحه‌بندی‌شده، رمزنگاری محتوای ذخیره‌شده، CSP، کنترل پنجرهٔ فراخواننده برای بخشی از فرمان‌های حساس، فیلتر اسرار، DNS pinning برای قابلیت اختیاری GIF و CI امنیتی از نقاط قوت مهم‌اند.

مهم‌ترین فاصله‌ها تا سطح Enterprise عبارت‌اند از: پوشش ناکامل کنترل دسترسی روی همهٔ فرمان‌های Tauri تغییردهندهٔ سیستم، نبود تست واقعی X11/Wayland و تست نصب بسته‌ها، ناسازگاری نام باینری در چند مسیر بسته‌بندی/CI، پوشش تست انتخابی و محدود، و کامل‌نبودن ترجمهٔ سندهای عمیق مانند ADRها و Threat Model به‌صورت متن مستقل فارسی.

در جریان این بازبینی، رشته‌های ثابت اصلی رابط تنظیمات، Setup Wizard، انتخابگرها، برچسب‌های دسترس‌پذیری و دسته‌بندی‌ها به کاتالوگ i18n منتقل شدند؛ برابری کاتالوگ‌های فارسی و انگلیسی با تست خودکار کنترل شد؛ خطای TypeScript 5.9 در `ignoreDeprecations` رفع شد؛ و راهنمای جامع مستقل فارسی و انگلیسی افزوده شد.

---

## ۱. تحلیل معماری و ساختار

### نقاط قوت

- معماری دو لایه روشن است: React 19 + TypeScript برای WebView و Rust + Tauri v2 برای دسترسی سیستم.
- منطق کلیپ‌بورد به زیرماژول‌های `clipboard_manager/` شامل persistence، history access، deduplication و clipboard write شکسته شده است.
- مسئولیت‌های سیستمی در فایل‌های جدا مانند `input_simulator.rs`، `privacy.rs`، `history_crypto.rs`، `ssrf.rs`، `window_controller.rs` و `theme_manager.rs` قرار دارند.
- persistence از SQLite/WAL، صفحه‌بندی IPC و سقف تاریخچه استفاده می‌کند؛ این انتخاب از بازنویسی کامل فایل JSON جلوگیری می‌کند.
- قابلیت شبکه‌ای GIF با feature خاموش پیش‌فرض جدا شده است؛ build عادی کلاینت HTTP خروجی ندارد.
- تنظیمات رابط به کامپوننت‌های کوچک‌تر تقسیم شده‌اند و فایل `SettingsApp.tsx` بیشتر نقش orchestration دارد.
- ADR، Threat Model، CI contract و راهنمای بسته‌بندی وجود دارد.

### ضعف‌ها و ریسک‌ها

1. **ناسازگاری نام باینری — زیاد:** `Cargo.toml` باینری `windows-11-style-clipboard-history-manager-bin` می‌سازد، و CI، Debian rules، Flatpak و بخش‌هایی از AppArmor با نام `windows-11-style-clipboard-history-manager-bin` یکپارچه شده‌اند.
2. فایل‌های بزرگ باقی‌مانده‌اند: `input_simulator.rs`، `history_crypto.rs`، `SetupWizard.tsx` و `commands.rs` هر کدام چندصد خط‌اند. شکستن آن‌ها به adapter/service/policy/test module نگهداری را بهتر می‌کند.
3. دو پیاده‌سازی SymbolPicker در `components/SymbolPicker.tsx` و `components/common/SymbolPicker.tsx` دیده می‌شود؛ نسخهٔ بلااستفاده خطر drift دارد.
4. نام‌های تاریخی پروژه در مسیرها و بسته‌ها به `windows-11-style-clipboard-history-manager` یکنواخت شدند.
5. محصول دسکتاپ است و «مقیاس‌پذیری» آن بیشتر به حجم تاریخچه، مصرف حافظه و latency IPC مربوط است، نه horizontal scaling. صفحه‌بندی خوب است ولی benchmark و budget عملکردی ثبت نشده است.

### جمع‌بندی معماری

معماری پایه مناسب و قابل توسعه است، اما لازم است قرارداد نام‌گذاری artifactها و boundaryهای فرمان‌های Tauri یکپارچه و با تست end-to-end تثبیت شوند.

---

## ۲. بررسی کیفیت کد

### نقاط قوت

- TypeScript در حالت strict است و ESLint type-aware با zero warnings اجرا می‌شود.
- Rust از `thiserror` و `AppError` استفاده می‌کند؛ بیشتر عملیات حساس error propagation دارند.
- SQL پارامتری است و نشانه‌ای از SQL injection مشاهده نشد.
- نوشتن تنظیمات JSON اتمیک و مجوز فایل‌های حساس محدود می‌شود.
- هوک‌ها و سرویس‌های جستجو از UI جدا شده‌اند.
- تست‌های واحد برای جستجو، pagination، URL safety، smart actions، clipboard hook و چند کامپوننت وجود دارد.
- پس از اصلاح، `npm run lint`، `npm run build` و ۸۷ تست frontend موفق‌اند.

### ضعف‌ها

- `load_rows` از `rows.flatten()` استفاده می‌کند و خطای decode یک ردیف را بی‌صدا حذف می‌کند؛ بهتر است خطا گزارش یا ردیف quarantine شود.
- کامنت `history_crypto.rs` ادعا می‌کند feature مربوط به zeroize در `Cargo.toml` فعال است، ولی dependency به‌صورت صریح این feature را نشان نمی‌دهد؛ ادعا باید با گراف واقعی dependency یا تست/پیکربندی اثبات شود.
- چند پیام خطای backend به‌صورت `String` به UI می‌رسد. برای ترجمه و telemetry بهتر است error code ساخت‌یافته همراه با جزئیات فنی برگردد.
- پوشش تست روی مجموعه‌ای صریح و محدود اعمال می‌شود و بخش زیادی از Setup، Settings، watcher، input simulation و packaging را اندازه نمی‌گیرد.
- تست Rust در این محیط اجرا نشد، چون toolchain Rust نصب نبود؛ بنابراین نتیجهٔ Rust بر اساس بازبینی ایستا و قرارداد CI است، نه اجرای محلی این گزارش.
- چند `expect` در مسیر runtime وجود دارد. بعضی در bootstrap قابل قبول‌اند، ولی watcher thread و device lifecycle باید graceful failure داشته باشند.

### SOLID و Clean Code

SRP و separation of concerns در بخش عمده رعایت شده است. dependency inversion در adapterهای سیستم‌عامل کامل نیست و بسیاری از ماژول‌ها مستقیماً `Command`، فایل‌سیستم یا global state را صدا می‌زنند؛ تزریق interface برای command runner، clipboard backend و clock، تست‌پذیری را افزایش می‌دهد.

---

## ۳. بررسی امنیت و شبکه

### کنترل‌های مثبت

- CSP با `script-src 'self'` و `withGlobalTauri: false` فعال است.
- تاریخچهٔ متن و تصویر با ChaCha20-Poly1305 رمز می‌شود و nonce تصادفی دارد.
- فایل‌های DB، WAL، SHM، کلید و تنظیمات به مجوزهای محدود منتقل می‌شوند.
- Secret Service به‌عنوان backend اختیاری کلید وجود دارد و marker از تعویض خاموش کلید جلوگیری می‌کند.
- فیلتر private key، API token، JWT و password pattern وجود دارد.
- paste injection به پنجرهٔ `main`، نوشتن اخیر کلیپ‌بورد و serialize شدن عملیات paste محدود شده است.
- URL opener فقط HTTPS/mailto را می‌پذیرد و schemeهای خطرناک، credential و IPهای خصوصی literal را رد می‌کند.
- دانلود GIF اختیاری دارای host allowlist، HTTPS-only، no redirect، محدودیت اندازه و DNS pinning است.
- CI شامل npm audit، cargo audit، cargo deny، actionهای SHA-pinned و SBOM/provenance است.

### یافته‌های مهم

1. **ACL ناقص فرمان‌های Tauri — زیاد:** کنترل `require_main_window` و `require_settings_window` برای تعدادی فرمان حساس وجود دارد، اما فرمان‌هایی مانند `fix_permissions_now`، reset first run و تعدادی عملیات shortcut/system مستقیماً window identity را بررسی نمی‌کنند. هر فرمانی که سیستم، تنظیمات یا فایل‌های DE را تغییر می‌دهد باید policy مرکزی و deny-by-default داشته باشد.
2. **مجوز قدرتمند `/dev/uinput` — ذاتی/زیاد:** compromise برنامه می‌تواند تزریق ورودی انجام دهد. paste gate دامنهٔ آسیب را کم می‌کند، ولی sandbox و AppArmor باید در بسته‌های رسمی آزموده و در صورت امکان enforce شوند.
3. **کلید فایل کنار DB — متوسط:** رمزنگاری با key-file از خواندن توسط کاربر دیگر و بعضی snapshotهای ناقص محافظت می‌کند، اما در سرقت کامل home directory که DB و key هر دو موجودند کافی نیست. Secret Service باید گزینهٔ توصیه‌شده باشد.
4. **عدم تشخیص برنامهٔ فعال در Wayland — متوسط/ذاتی:** فیلتر مدیر رمز عبور روی Wayland قابل اتکا نیست. UI این محدودیت را توضیح می‌دهد، اما باید در onboarding نیز هشدار برجسته باشد.
5. **Release fail-closed ناکامل — متوسط:** مرحله‌های Cloudsmith/AUR دارای `continue-on-error: true` هستند؛ در صورت تنظیم secret و شکست واقعی هم release سبز می‌ماند. این رفتار با ادعای fail-closed کاملاً منطبق نیست.
6. **CSP شامل `style-src 'unsafe-inline'` — کم تا متوسط:** برای styling فعلی قابل توضیح است، ولی بهتر است nonce/hash یا حذف styleهای inline بررسی شود.
7. **لاگ و clipboard — متوسط:** threat model حساسیت log را ذکر می‌کند. باید تست شود هیچ محتوای clipboard، token، URL حساس یا window title بدون redaction ثبت نمی‌شود.

### شبکه

در build پیش‌فرض سطح شبکه بسیار کم است. در feature GIF، طراحی SSRF مناسب است. برای `open_safe_url`، مرورگر خارجی مقصد را باز می‌کند و خود برنامه درخواست HTTP نمی‌فرستد؛ با این حال سیاست block برای hostnameهایی که بعداً به private IP resolve می‌شوند در این مسیر اعمال نمی‌شود. این بیشتر policy مرورگر/کاربر است تا SSRF backend، ولی باید در مدل تهدید صریح باشد.

---

## ۴. ارزیابی مستندات

### نقاط قوت

- README شامل معرفی، نصب، حریم خصوصی، معماری، میانبرها، رفع اشکال و توسعه است.
- Threat Model، CI contract، هشت ADR، SECURITY و CONTRIBUTING موجودند.
- راهنمای دوزبانگی و فونت محلی وجود دارد.
- راهنمای جامع مستقل [فارسی](../USER_GUIDE.fa.md) و [انگلیسی](../USER_GUIDE.en.md) در این بازبینی اضافه شد.

### شکاف‌ها

- تمام ADRهای قدیمی ترجمهٔ کامل فارسی ندارند؛ برخی فقط انگلیسی و برخی ترکیبی‌اند.
- Threat Model عمدتاً انگلیسی است و باید نسخهٔ کامل فارسی مستقل داشته باشد.
- `CHANGELOG.md` برای همهٔ ورودی‌ها ترجمهٔ کامل دوطرفه ندارد.
- بعضی ادعاهای README باید در CI به‌صورت machine-verifiable کنترل شوند؛ نمونه: نام باینری، zero network و fail-closed channelها.
- بهتر است لینک انتخاب زبان در ابتدای README قرار گیرد: «راهنمای فارسی» و «English Guide».

### نتیجهٔ بررسی بند ۸

مستندات اصلی برای هر دو گروه اکنون entry point جامع دارند، ولی شرط سخت‌گیرانهٔ «تمام اسناد، خط‌به‌خط در هر دو زبان» هنوز محقق نشده است. برای تحقق کامل باید ADRها، Threat Model، packaging details و CHANGELOG نسخهٔ mirrored فارسی/انگلیسی داشته باشند و parity آن‌ها در CI کنترل شود.

---

## ۵. امتیازدهی

| معیار | امتیاز از ۱۰ | دلیل کوتاه |
| --- | ---: | --- |
| کیفیت کد و معماری | **8.2** | معماری روشن و ابزار کیفیت خوب؛ نام‌گذاری artifact و فایل‌های بزرگ نیازمند اصلاح |
| امنیت | **8.0** | رمزنگاری، CSP، paste gate، SSRF و supply-chain قوی؛ ACL فرمان‌ها و uinput ریسک باقی‌مانده |
| مستندات | **8.1** | پوشش موضوعی عالی و راهنمای جامع دو زبان؛ ترجمهٔ خط‌به‌خط همهٔ اسناد کامل نیست |
| قابلیت توسعه | **7.6** | ماژولار و صفحه‌بندی‌شده؛ تست integration/benchmark و abstraction سیستم‌عامل ناکافی |
| **میانگین کل** | **7.98 ≈ 8.0** | میانگین حسابی چهار معیار |

---

## ۶. پیشنهادهای کلیدی Enterprise

### اولویت P0

1. یک نام canonical برای binary/package/service انتخاب و در Cargo، Tauri، CI، Debian، RPM، Flatpak، AUR، AppArmor، wrapper و README یکسان کنید؛ یک تست CI تمام pathها را validate کند.
2. تمام فرمان‌های Tauri را inventory کنید و policy مرکزی deny-by-default بسازید: `main-only`، `settings-only`، `setup-only` و `read-only`.
3. CI را روی PR واقعی اجرا و build/package smoke را برای `.deb`، `.rpm` و AppImage نصب/اجرا کنید.
4. مراحل distribution دارای secret را در صورت فعال بودن fail-closed کنید؛ نبود secret می‌تواند skip شود، ولی شکست upload نباید نادیده گرفته شود.

### اولویت P1

5. تست integration برای Xvfb/X11 و یک ماتریس Wayland compositor اضافه کنید؛ shortcut، watcher، paste، focus restore و cleanup را بسنجید.
6. error contract ساخت‌یافته با `code`, `message_key`, `debug_detail` طراحی کنید تا backend diagnosticها قابل ترجمه و redaction باشند.
7. Secret Service را در دسکتاپ‌های پشتیبانی‌شده گزینهٔ پیشنهادی کنید و recovery/export امن کلید را مستند سازید.
8. fuzz/property test برای URL parser، secret detector، legacy migration، encryption envelope و IPC pagination اضافه کنید.
9. benchmark و SLO تعریف کنید: startup، زمان بازشدن popup، مصرف RAM، latency جستجو و DB در ۲٬۰۰۰ مورد.

### اولویت P2

10. `SetupWizard.tsx`، `commands.rs`، `input_simulator.rs` و `history_crypto.rs` را به واحدهای کوچک‌تر تقسیم کنید.
11. SymbolPicker تکراری و dead code را حذف و dependency boundaries را با lint enforce کنید.
12. reproducible build، امضای release اجباری، rotation key، CODEOWNERS حساس و branch protection را تکمیل کنید.
13. نسخهٔ کامل فارسی Threat Model و ADRها را بسازید و parity docs را در CI بررسی کنید.

---

## ۷. نتیجهٔ بررسی منوها و رابط فارسی/انگلیسی

### وضعیت پس از اصلاح این بازبینی

- کاتالوگ‌های `en.json` و `fa.json` هر کدام **۳۰۳ کلید** همسان و غیرخالی دارند.
- تنظیمات Appearance، auto-delete، transparency، UI scale، history، custom kaomoji، features، shortcut و reset از i18n استفاده می‌کنند.
- Setup Wizard، دسته‌بندی‌های ایموجی/کائوموجی/نماد، SearchBar و برچسب‌های accessibility ترجمه شدند.
- RTL از `document.documentElement.dir` اعمال می‌شود.
- منوی tray در startup بر اساس زبان ذخیره‌شده فارسی یا انگلیسی ساخته می‌شود.
- تست خودکار parity کاتالوگ‌ها اضافه شد.

### محدودیت باقی‌مانده

- پیام‌های diagnostic که مستقیم از Rust می‌آیند (مانند دستورالعمل دستی DE یا خطای permission) عمدتاً انگلیسی‌اند؛ تبدیل آن‌ها به error/status code برای پوشش صددرصد لازم است.
- نام emoji/symbol برگرفته از dataset انگلیسی است؛ عنوان‌های داده‌ای هنوز ترجمهٔ کامل ندارند، هرچند منوها و دسته‌ها ترجمه شده‌اند.
- منوی tray بعد از تغییر زبان احتمالاً تا restart بازسازی نمی‌شود؛ باید update زندهٔ MenuItemها اضافه و تست شود.

بنابراین «تمام منوهای اصلی» پوشش دوزبانهٔ بسیار خوبی دارند، اما برای ادعای ۱۰۰٪ باید سه مورد بالا نیز بسته شوند.

---

## ۸. نتیجهٔ بررسی دسترسی مستندات فارسی و انگلیسی

- راهنمای جامع مستقل برای هر دو زبان افزوده شده است.
- README و اسناد مهم متعددی دو زبانه‌اند.
- CONTRIBUTING و SECURITY پوشش فارسی مناسبی دارند.
- همهٔ اسناد تخصصی هنوز ترجمهٔ کامل یک‌به‌یک ندارند؛ این مورد به‌عنوان backlog مستندسازی Enterprise باقی می‌ماند.

---

## شواهد آزمون

| فرمان/بررسی | نتیجه |
| --- | --- |
| `npm audit --audit-level=moderate` | ۰ آسیب‌پذیری گزارش‌شده |
| `npm run lint` | موفق پس از اصلاح TypeScript 5.9 |
| `npm test -- --run` | ۱۲ فایل / ۸۷ تست موفق |
| `npm run build` | موفق |
| برابری کلیدهای locale | موفق؛ بدون کلید گمشده یا مقدار خالی |
| `cargo test` / `cargo clippy` | در محیط بازبینی اجرا نشد؛ Rust toolchain نصب نبود |

> این گزارش جایگزین penetration test روی باینری بسته‌بندی‌شده، تست واقعی چند compositor یا ممیزی رمزنگاری مستقل نیست.
