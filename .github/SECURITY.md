# 🔐 Security Policy / سیاست امنیتی

<div dir="rtl">

## امنیت برای ما اولویت است

ما امنیت این پروژه را جدی می‌گیریم. اگر آسیب‌پذیری امنیتی کشف کرده‌اید، لطفاً مراحل زیر را دنبال کنید.

## گزارش آسیب‌پذیری

**⚠️ لطفاً برای گزارش آسیب‌پذیری‌های امنیتی، issue عمومی باز نکنید.**

در عوض، از یکی از روش‌های زیر استفاده کنید:

1. **GitHub Security Advisory**: از [قابلیت گزارش خصوصی GitHub](https://github.com/Mahdi-Arts/Windows-11-Style-Clipboard-History-Manager/security/advisories/new) استفاده کنید
2. **ایمیل**: mahdi-arts@users.noreply.github.com (ترجیحاً از GitHub Advisory استفاده کنید)

### اطلاعات مورد نیاز

- **شرح** آسیب‌پذیری
- **مراحل بازتولید** مسئله
- **تأثیر بالقوه** آسیب‌پذیری
- **راه حل پیشنهادی** (اگر دارید)

### زمان‌بندی پاسخ

| سطح | زمان پاسخ |
| --- | --- |
| پاسخ اولیه | تا ۴۸ ساعت |
| به‌روزرسانی وضعیت | تا ۱ هفته |
| رفع بحرانی | ۲۴-۷۲ ساعت |
| رفع بالا | ۱ هفته |
| رفع متوسط | ۲ هفته |
| رفع پایین | انتشار بعدی |

</div>

---

## We Take Security Seriously

If you discover a security vulnerability, please follow these steps:

**⚠️ Do NOT open a public issue for security vulnerabilities.**

### Private Disclosure Methods

1. **GitHub Security Advisory**: Use [private vulnerability reporting](https://github.com/Mahdi-Arts/Windows-11-Style-Clipboard-History-Manager/security/advisories/new)
2. **Email**: mahdi-arts@users.noreply.github.com (prefer the GitHub Advisory flow)

### What to Include

- **Description** of the vulnerability
- **Steps to reproduce**
- **Potential impact**
- **Suggested fix** (if any)

### Response Timeline

| Severity | Response Time |
| --- | --- |
| Initial Response | Within 48 hours |
| Status Update | Within 1 week |
| Critical Fix | 24-72 hours |
| High | 1 week |
| Medium | 2 weeks |
| Low | Next release |

---

## Security Best Practices / بهترین روش‌های امنیتی

<div dir="rtl">

### هنگام استفاده

1. **همیشه به‌روز باشید**: از آخرین نسخه استفاده کنید
2. **از منبع معتبر نصب کنید**: از مخازن رسمی استفاده کنید
3. **مشکلات را گزارش دهید**: به ما در شناسایی رفتارهای مشکوک کمک کنید

### حریم خصوصی داده‌ها

- تاریخچه کلیپ‌بورد **به‌صورت محلی** در SQLite با مجوز `0600` ذخیره می‌شود
- فیلتر اسرار و نادیده‌گرفتن مدیر رمز عبور به‌صورت پیش‌فرض روشن است
- جستجوی GIF (اختیاری) فقط با `TENOR_API_KEY` به Tenor می‌رود
- شبیه‌سازی Ctrl+V از `/dev/uinput` استفاده می‌کند — باینری را مثل یک دستگاه ورودی قابل‌اعتماد در نظر بگیرید

### دسترسی‌های مورد نیاز

- **کلید میانبر سراسری**: برای `Super+V` و `Ctrl+Alt+V`
- **System Tray**: برای اجرای پس‌زمینه
- **دسترسی به کلیپ‌بورد**: عملکرد اصلی برنامه

### امنیت Wayland

در Wayland، دسترسی به کلیپ‌بورد تابع مدل امنیتی کامپوزیتور است که ممکن است دسترسی برنامه‌های پس‌زمینه را محدود کند.

</div>

### App Security Features / ویژگی‌های امنیتی برنامه

✔️ **CSP** — `script-src 'self'` + `font-src 'self'` (fonts bundled, app fully offline)  
✔️ `withGlobalTauri: false`  
✔️ **SSRF** — HTTPS + host allowlist + DNS pinning + no redirects  
✔️ **10 MB** GIF cap (streamed)  
✔️ **SQLite WAL** + chmod `0600`  
✔️ **Secret filter** and password-manager skip (defaults on)  
✔️ **Smart Actions** open via Rust `xdg-open` after allowlist (no `shell:allow-open`)  
✔️ **Opt-in** tiling WM config rewrite  
✔️ **Mandatory SHA256SUMS verification** in the installer (optional GPG)  
✔️ **Blocking** `cargo audit` + `npm audit` in CI  
✔️ **SLSA provenance + SPDX SBOM** published per release  

If you copy a password into an unsandboxed terminal, it can still land in history unless the secret filter matches. Review Settings → Privacy.

---

**Thank you for helping keep this project secure! 🔐**  
**از کمک شما برای امنیت این پروژه سپاسگزاریم! 🔐**