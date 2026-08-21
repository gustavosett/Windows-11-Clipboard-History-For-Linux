# راهنمای جامع فارسی — Windows 11 Style Clipboard History Manager

> نسخهٔ مستندات: ۲٫۵٫۰ — آخرین بازبینی: ۱۴۰۵/۰۵/۳۰ (2026-08-21)

این سند، راهنمای فارسی نصب، استفاده، حریم خصوصی، رفع اشکال و مشارکت در پروژه است. برای جزئیات فنی عمیق‌تر، [معماری](ARCHITECTURE.md)، [مدل تهدید](THREAT_MODEL.md)، [CI](CI.md) و [ADRها](adr/) را ببینید.

## ۱. معرفی و قابلیت‌ها

برنامه یک مدیر تاریخچهٔ کلیپ‌بورد محلی برای لینوکس است که روی X11 و Wayland اجرا می‌شود و تجربه‌ای شبیه `Win+V` ویندوز ۱۱ فراهم می‌کند:

- تاریخچهٔ متن، متن غنی و تصویر؛
- جستجو، سنجاق‌کردن و حذف خودکار؛
- انتخابگر ایموجی، کائوموجی و نماد؛
- راه‌اندازی اولیه و تنظیمات فارسی/انگلیسی با RTL خودکار و فونت محلی Vazirmatn؛ پنجرهٔ اصلی برای پایداری تعامل، انگلیسی و LTR می‌ماند؛
- ذخیرهٔ SQLite و رمزنگاری متن و تصویر با ChaCha20-Poly1305؛
- عملکرد آفلاین در پیکربندی پیش‌فرض.

## ۲. نصب

### Debian و Ubuntu

بستهٔ `.deb` را از صفحهٔ Releases بگیرید، مقدار آن را با `SHA256SUMS` تطبیق دهید و اجرا کنید:

```bash
sha256sum -c SHA256SUMS --ignore-missing
sudo apt install ./windows-11-style-clipboard-history-manager_2.5.0_amd64.deb
sudo setfacl -m u:$USER:rw /dev/uinput
```

### Fedora

```bash
sha256sum -c SHA256SUMS --ignore-missing
sudo dnf install ./windows-11-style-clipboard-history-manager-2.5.0-1.x86_64.rpm
sudo setfacl -m u:$USER:rw /dev/uinput
```

### Arch Linux

```bash
yay -S windows-11-style-clipboard-history-manager-bin
```

### Flatpak

محدودهٔ Flatpak به‌طور پیش‌فرض `/dev/uinput` را ارائه نمی‌کند. جزئیات و پیامدهای امنیتی override در [راهنمای بسته‌بندی](../packaging/README.md) آمده است.

## ۳. استفاده

| میانبر | عملکرد |
| --- | --- |
| `Super+V` | نمایش تاریخچه |
| `Ctrl+Alt+V` | میانبر جایگزین |
| `Super+.` | انتخابگر ایموجی |
| `Enter` | چسباندن مورد انتخابی |
| `Esc` | بستن پنجره |
| `Ctrl+F` | جستجو |

از تنظیمات می‌توانید پوسته، شفافیت، مقیاس رابط، زبان، اندازهٔ تاریخچه، حذف خودکار، فیلتر اسرار، ذخیرهٔ تصویر و محل کلید رمزنگاری را تغییر دهید. تغییر زبان بلافاصله جهت صفحه و متن‌های رابط را عوض می‌کند.

## ۴. داده، حریم خصوصی و امنیت

- پایگاه داده: `~/.local/share/windows-11-style-clipboard-history-manager/history.db`
- تصاویر: `~/.local/share/windows-11-style-clipboard-history-manager/images/`
- تنظیمات: `~/.config/windows-11-style-clipboard-history-manager/user_settings.json`
- کلید پیش‌فرض: `history.key` با مجوز `0600`، یا Secret Service دسکتاپ
- سقف پیش‌فرض تاریخچه: ۲۰۰۰ مورد
- شبکه: در حالت پیش‌فرض هیچ درخواست شبکه‌ای انجام نمی‌شود. قابلیت اختیاری GIF به `TENOR_API_KEY` و build feature مربوط نیاز دارد.

فیلتر اسرار و نادیده‌گرفتن مدیرهای رمز عبور به‌صورت پیش‌فرض فعال‌اند، ولی هیچ تشخیص الگویی کامل نیست. در Wayland نام پنجرهٔ فعال در اختیار برنامه نیست؛ بنابراین فیلتر برنامه‌های حساس فقط روی X11 عمل می‌کند. دسترسی `/dev/uinput` قدرتمند است؛ فقط باینری معتبر و checksum-شده را اجرا کنید.

## ۵. رفع اشکال

- **میانبر روی GNOME کار نمی‌کند:** GNOME ممکن است `Super+V` را رزرو کرده باشد؛ `Ctrl+Alt+V` را امتحان یا میانبر اعلان را تغییر دهید.
- **چسباندن انجام نمی‌شود:** دسترسی `/dev/uinput` را با `setfacl` بررسی و یک‌بار logout/login کنید.
- **پنجره روی NVIDIA سیاه است:** برنامه را با `IS_NVIDIA=1` اجرا کنید.
- **مدیر رمز عبور در Wayland ثبت می‌شود:** «نادیده گرفتن اسرار» را روشن نگه دارید؛ تشخیص پنجرهٔ فعال در Wayland ممکن نیست.
- **گزارش‌ها:** مسیر `~/.local/share/windows-11-style-clipboard-history-manager/logs/` را بررسی کنید و پیش از ارسال، اطلاعات حساس را حذف کنید.

## ۶. توسعه و آزمون

پیش‌نیازها: Node.js 20+، Rust پایدار، WebKitGTK 4.1، GTK3 و کتابخانه‌های نوشته‌شده در README.

```bash
make deps
npm ci
npm run lint
npm run test:coverage
npm run build
cd src-tauri
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-features
```

کلید ترجمهٔ جدید باید هم‌زمان در `src/locales/en.json` و `src/locales/fa.json` اضافه شود. آزمون `src/i18n/locales.test.ts` برابری و خالی‌نبودن کلیدها را کنترل می‌کند.

## ۷. گزارش امنیتی و مشارکت

آسیب‌پذیری را عمومی ثبت نکنید. از GitHub Private Vulnerability Reporting طبق [SECURITY.md](../.github/SECURITY.md) استفاده کنید. قواعد مشارکت، قالب commit و چک‌لیست PR در [CONTRIBUTING.md](../.github/CONTRIBUTING.md) آمده است.

## ۸. نقشهٔ مستندات

- [README اصلی دوزبانه](../README.md)
- [راهنمای کامل انگلیسی](USER_GUIDE.en.md)
- [معماری](ARCHITECTURE.md)
- [راهنمای دوزبانگی](BILINGUAL.md)
- [مدل تهدید](THREAT_MODEL.md)
- [قرارداد CI](CI.md)
- [تصمیم‌های معماری](adr/)
- [بسته‌بندی](../packaging/README.md)
- [گزارش بازبینی ۲۰۲۶](reports/REPOSITORY_REVIEW_2026-08-21.fa.md)
