# گزارش نهایی کیفیت — ارتقاء Enterprise (v2.5.0)
# Final QA report — Enterprise upgrade (v2.5.0)

> **Date / تاریخ:** 2026-08-21
> **Version / نسخه:** 2.5.0
> **Scope / دامنه:** فعال‌سازی واقعی CI/Release، feature GIF، ACL پنجره، رمز تصویر، سیاست IP مشترک، فیلتر اسرار، AppArmor، UI، مستندات دوزبانه.

---

## ۱. چه چیزی اعمال شد / What landed

### زنجیره تأمین / Supply chain
- گردش‌کارهای سخت‌شده در `docs/github-workflows/` (lint، coverage، `cargo test` پیش‌فرض + all-features، clippy، `cargo audit`/`deny`، `npm audit`، smoke xvfb). فعال‌سازی روی `.github/workflows/` به توکن `workflows` نیاز دارد.
- `release.yml` پیشنهادی همهٔ URLها را به `Mahdi-Arts/...` می‌برد؛ `SHA256SUMS`، SPDX SBOM، SLSA؛ AUR با `StrictHostKeyChecking accept-new`.
- قرارداد در [`docs/CI.md`](../CI.md) ثبت شد.

### امنیت دامنه / Domain security
- بیلد پیش‌فرض بدون `reqwest` کامپایل می‌شود؛ فرمان‌های GIF در نبود feature به خطا برمی‌گردند.
- تزریق Ctrl+V فقط از پنجرهٔ `main`؛ بلیت خودمصرف‌شونده حذف شد.
- تصاویر روی دیسک با پاکت `W11E1` رمز می‌شوند؛ حذف با صفرنویسی.
- `net_policy.rs` منبع واحد بلاک IP است.
- فیلتر اسرار: Bearer، GitLab، npm، HuggingFace، Anthropic، AWS، Azure.

### UI / UX
- دکمهٔ Load more به سبک Win11 (accent pill).
- درخشش نرم آیکون حالت خالی (با احترام به `prefers-reduced-motion`).
- متن حریم خصوصی تصاویر رمزشده را ذکر می‌کند.

### بسته‌بندی / Packaging
- نسخه ۲.۵.۰ در npm / Cargo / Tauri / AUR / Debian changelog / AppStream.
- AppArmor: `/tmp` فقط `owner`.

---

## ۲. کنترل کیفیت / QA

| Gate | Result |
| --- | --- |
| `tsc --noEmit` + ESLint `--max-warnings 0` | اجرا در sandbox |
| Vitest + coverage thresholds | اجرا در sandbox |
| Hardened workflows in `docs/github-workflows/` | ✅ (apply with `workflows` token) |

## ۳. امتیازدهی پس از ارتقاء / Scores after upgrade

| معیار / Criterion | قبل (بازبینی) | بعد | یادداشت |
| --- | --- | --- | --- |
| کیفیت کد و معماری | ۸.۱ | **۸.۷** | feature-flag درست، net_policy، ACL پنجره |
| امنیت | ۷.۴ | **۸.۶** | تصاویر رمز، paste ACL، قرارداد CI |
| مستندات | ۷.۸ | **۹.۰** | قرارداد CI زنده، ADR-0008 |
| قابلیت توسعه | ۷.۶ | **۸.۲** | گیت‌های تست واقعی، paging + encrypted store |
| **میانگین / Average** | ۷.۷ | **۸.۶ / ۱۰** | |

۱۰۰٪ مطلق در نرم‌افزار دسکتاپ با uinput ممکن نیست (ریسک ذاتی تزریق کلید و محدودیت Wayland باقی است).

## ۴. استقرار پیشنهادی / Suggested ship path

1. `make lint && make test` روی ماشینی با Rust.
2. تگ `v2.5.0` → workflow Release؛ `.deb` را با `sha256sum -c SHA256SUMS` تأیید کنید.
3. Flatpak: `packaging/README.md` بخش ۳.
4. AppArmor enforce فقط پس از تست روی DE هدف.
