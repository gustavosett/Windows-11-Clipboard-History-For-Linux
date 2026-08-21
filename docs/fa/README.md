# درگاه مستندات فنی فارسی

این صفحه، معادل فارسی محتوای عملیاتی و تصمیم‌های اصلی پروژه را یکجا در اختیار توسعه‌دهندگان و مدیران سیستم قرار می‌دهد. متن مرجع انگلیسی هر موضوع نیز پیوند شده است.

## سیاست زبان رابط

- پنجرهٔ راه‌اندازی نخست و تنظیمات: فارسی/انگلیسی و RTL/LTR پویا؛
- پنجرهٔ اصلی کلیپ‌بورد و tray: همیشه انگلیسی و LTR؛
- ترجیح زبان در تنظیمات ذخیره می‌شود، اما به سطح اصلی تحمیل نمی‌شود.

جزئیات: [راهنمای دوزبانگی](../BILINGUAL.md)

## معماری

React/TypeScript در WebView فقط رابط و تعامل کاربر را مدیریت می‌کند. Rust/Tauri مالک کلیپ‌بورد، SQLite، رمزنگاری، میانبر، تزریق paste و دسترسی سیستم است. فرمان‌های Tauri با `window_policy` به نقش‌های `main`، `settings` و `setup` محدود می‌شوند. تاریخچه با صفحه‌بندی محدود منتقل و متن/تصویر روی دیسک رمز می‌شود.

متن مرجع: [ARCHITECTURE.md](../ARCHITECTURE.md)

## مدل تهدید فارسی

### دارایی‌ها

1. متن، HTML و تصویر کلیپ‌بورد؛
2. کلید رمزنگاری فایل یا Secret Service؛
3. تنظیمات حریم خصوصی و میانبر؛
4. مجوز `/dev/uinput`؛
5. زنجیرهٔ ساخت و artifactهای انتشار.

### مرزهای اعتماد

- دادهٔ کلیپ‌بورد ورودی غیرقابل‌اعتماد است؛
- هر WebView فقط فرمان‌های نقش خودش را فراخوانی می‌کند؛
- `/dev/uinput` و Polkit مرز سطح‌بالای سیستم هستند؛
- شبکه در build پیش‌فرض غیرفعال است؛ GIF اختیاری HTTPS، allowlist و DNS pinning دارد؛
- artifact انتشار باید checksum، SBOM و provenance داشته باشد.

### کنترل‌ها و محدودیت‌ها

- ChaCha20-Poly1305، مجوز فایل `0600` و Secret Service؛
- فیلتر اسرار و نادیده‌گرفتن برنامهٔ حساس روی X11؛
- paste gate، window ACL و محدودیت اندازهٔ payload؛
- CSP و URL allowlist؛
- Wayland نام برنامهٔ فعال را ارائه نمی‌کند؛
- کلید فایل کنار DB در برابر سرقت کامل home کافی نیست؛ Secret Service توصیه می‌شود؛
- Flatpak به‌طور پیش‌فرض `/dev/uinput` ندارد.

متن مرجع: [THREAT_MODEL.md](../THREAT_MODEL.md)

## تصمیم‌های معماری (ADR)

| ADR | تصمیم فارسی | متن مرجع |
| --- | --- | --- |
| 0001 | SQLite/WAL جایگزین بازنویسی کامل JSON؛ persistence افزایشی و محدود | [لینک](../adr/0001-sqlite-persistence.md) |
| 0002 | تزریق paste فقط پس از نوشتن معتبر کلیپ‌بورد و از پنجرهٔ اصلی | [لینک](../adr/0002-paste-injection-architecture.md) |
| 0003 | دانلود اختیاری با HTTPS، allowlist، no-redirect و DNS pinning | [لینک](../adr/0003-ssrf-dns-pinning.md) |
| 0004 | رمزنگاری سطح فیلد با nonce تصادفی و مهاجرت legacy | [لینک](../adr/0004-field-encryption.md) |
| 0005 | CI مسدودکننده، action پین‌شده، checksum، SBOM و provenance | [لینک](../adr/0005-ci-supply-chain.md) |
| 0006 | کلید در فایل یا Secret Service با marker و مهاجرت fail-closed | [لینک](../adr/0006-secret-service-key-storage.md) |
| 0007 | خواندن تاریخچه با صفحه‌بندی IPC و سقف سمت سرور | [لینک](../adr/0007-ipc-pagination.md) |
| 0008 | ACL پنجره، رمزنگاری تصویر و تفکیک capabilityها | [لینک](../adr/0008-window-acl-and-image-encryption.md) |

## CI و انتشار

گیت اجباری شامل TypeScript/ESLint، پوشش تست، Rust syntax، `fmt`، `clippy -D warnings`، تست Rust عادی و all-features، ممیزی npm/Rust و ساخت release است. انتشار Debian پیش از Flatpak انجام می‌شود. Cloudsmith/AUR فقط در نبود credential رد می‌شوند؛ پس از پیکربندی، خطای واقعی انتشار را متوقف می‌کند.

- [قرارداد CI](../CI.md)
- [راهنمای بسته‌بندی](../../packaging/README.md)
- [بودجهٔ عملکرد](../PERFORMANCE.md)
- [راهنمای جامع کاربر فارسی](../USER_GUIDE.fa.md)

## گزارش امنیتی

آسیب‌پذیری را در issue عمومی ثبت نکنید. از GitHub Private Vulnerability Reporting مطابق [SECURITY.md](../../.github/SECURITY.md) استفاده کنید.
