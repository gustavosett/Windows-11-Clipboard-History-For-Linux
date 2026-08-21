# 🌐 Bilingual Support Guide / راهنمای پشتیبانی دو زبانه

<div dir="rtl">

## 🗣️ معرفی

پنجره‌های **راه‌اندازی اولیه** و **تنظیمات** از دو زبان فارسی و انگلیسی پشتیبانی می‌کنند. سیستم ترجمه بر پایهٔ `react-i18next` است. طبق سیاست تجربهٔ کاربری پروژه، پنجرهٔ اصلی کلیپ‌بورد همیشه انگلیسی و LTR باقی می‌ماند تا جهت کلیدها، شبکه‌ها و گردش کار سریع هنگام تغییر زبان جابه‌جا نشود.

## ✨ قابلیت‌ها

| قابلیت | توضیح |
| --- | --- |
| **جابجایی آنی** | تغییر زبان بلافاصله اعمال می‌شود — بدون نیاز به ریستارت |
| **RTL محدود و خودکار** | با انتخاب فارسی، فقط راه‌اندازی و تنظیمات RTL می‌شوند؛ پنجرهٔ اصلی LTR است |
| **فونت فارسی** | فونت **Vazirmatn** برای زبان فارسی به طور خودکار فعال می‌شود |
| **همگام‌سازی بین پنجره‌ها** | تغییر زبان در تنظیمات، همه پنجره‌ها (اصلی، تنظیمات) را همگام به‌روز می‌کند |
| **ذخیره‌سازی دائمی** | انتخاب زبان در فایل تنظیمات ذخیره می‌شود و در اجرای بعدی حفظ می‌شود |

## 🎯 نحوه تغییر زبان

### از داخل برنامه (تنظیمات)
1. روی آیکون **تنظیمات** در سینی سیستم کلیک کنید
2. به بخش **زبان** بروید
3. زبان مورد نظر را انتخاب کنید: **English** یا **فارسی**

### به صورت خودکار
در اولین اجرا، زبان بر اساس زبان سیستم شما تشخیص داده می‌شود (با `localStorage`).

## 🛠️ ساختار فنی

```
src/
├── i18n/
│   ├── config.ts          # پیکربندی i18next + تشخیص زبان
│   └── useLanguage.ts     # هوک React برای زبان و RTL
├── locales/
│   ├── en.json            # ترجمه‌های انگلیسی
│   └── fa.json            # ترجمه‌های فارسی
└── (کامپوننت‌ها از useTranslation() استفاده می‌کنند)
```

## 👨‍💻 برای توسعه‌دهندگان

### اضافه کردن متن جدید

1. کلید را به هر دو فایل اضافه کنید:
   ```json
   // src/locales/en.json
   { "clipboard": { "new_text": "New text here" } }

   // src/locales/fa.json
   { "clipboard": { "new_text": "متن جدید اینجا" } }
   ```

2. در کامپوننت استفاده کنید:
   ```tsx
   import { useTranslation } from 'react-i18next'
   const { t } = useTranslation()
   <p>{t('clipboard.new_text')}</p>
   ```

### نکات RTL

- از `dir="rtl"` و CSS classes با `[dir="rtl"]` برای تنظیمات راست‌به‌چپ استفاده کنید
- از `useLanguage().isRTL` برای منطق شرطی استفاده کنید
- به یاد داشته باشید: انیمیشن‌ها و جهت آیکون‌ها در RTL باید آینه شوند

</div>

---

## 🗣️ Introduction

The **first-run Setup** and **Settings** windows support Persian and English through `react-i18next`. By deliberate UX policy, the main clipboard popup always remains English/LTR so keyboard navigation, grids, and rapid workflows do not shift direction when the configuration language changes.

## ✨ Features

| Feature | Description |
| --- | --- |
| **Instant Switching** | Language changes apply immediately — **no restart needed** |
| **Scoped automatic RTL** | Persian switches Setup/Settings to RTL; the main popup remains LTR |
| **Persian Font** | **Vazirmatn** is bundled locally (SIL OFL 1.1) and loads automatically for Persian — fully offline |
| **Cross-window Sync** | Language changes in Settings sync to all windows instantly |
| **Persistent** | Language choice is saved and restored on next launch |

## 🎯 How to Change Language

### From the App (Settings)
1. Click the **Settings** icon in the system tray
2. Navigate to **Language** section
3. Select **English** or **فارسی**

### Automatically
On first launch, the language is auto-detected from your system (via `localStorage`).

## 🛠️ Technical Structure

```
src/
├── i18n/
│   ├── config.ts          # i18next config + language detection
│   └── useLanguage.ts     # React hook for language & RTL
├── locales/
│   ├── en.json            # English translations
│   └── fa.json            # Persian translations
└── (components use useTranslation())
```

## 👨‍💻 For Developers

### Adding New Text

1. Add the key to both files:
   ```json
   // src/locales/en.json
   { "clipboard": { "new_text": "New text here" } }

   // src/locales/fa.json
   { "clipboard": { "new_text": "متن جدید اینجا" } }
   ```

2. Use it in a component:
   ```tsx
   import { useTranslation } from 'react-i18next'
   const { t } = useTranslation()
   <p>{t('clipboard.new_text')}</p>
   ```

### RTL Notes

- Use `dir="rtl"` and `[dir="rtl"]` CSS selectors for right-to-left adjustments
- Use `useLanguage().isRTL` for conditional logic
- Remember: animations and icon directions should be mirrored in RTL

## 📝 Bilingual code comments / کامنت‌های دو زبانه کد

Beyond the UI, every **documentation comment** in the Rust core and the
TypeScript frontend is written in both English and Persian — English first
(dominant for code reviewers), followed by the Persian equivalent. Module
headers, ADRs, the threat model, CI contracts, and packaging guides follow
the same rule. See for example `src-tauri/src/clipboard_events.rs`,
`src/utils/urlSafety.ts`, and `.github/workflows/ci.yml`.
فراتر از UI، تمام کامنت‌های مستندات در هستهٔ Rust و فرانت‌اند TypeScript
دو زبانه نوشته می‌شوند — ابتدا انگلیسی (زبان غالب بازبینی کد) و سپس
معادل فارسی. سرصفحهٔ ماژول‌ها، ADRها، مدل تهدید، قرارداد CI و راهنماهای
بسته‌بندی هم از همین قاعده پیروی می‌کنند (نمونه: `clipboard_events.rs` و
`urlSafety.ts`).