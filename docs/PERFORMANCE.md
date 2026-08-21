# Performance Budget / بودجهٔ عملکرد

<div dir="rtl">

## فارسی

این سند SLOهای نسخهٔ دسکتاپ را تعریف می‌کند. اندازه‌گیری release باید روی سخت‌افزار مرجع (۴ هسته، ۸ گیگابایت RAM، SSD، Ubuntu LTS) و جداگانه روی X11 و Wayland انجام شود.

| معیار | هدف p95 | حد توقف انتشار |
| --- | ---: | ---: |
| آغاز سرد تا آماده‌شدن tray | کمتر از ۱٫۵ ثانیه | ۳ ثانیه |
| بازشدن popup پس از میانبر | کمتر از ۱۲۰ میلی‌ثانیه | ۲۵۰ میلی‌ثانیه |
| صفحهٔ نخست تاریخچه (۲۰۰ مورد) | کمتر از ۸۰ میلی‌ثانیه | ۱۵۰ میلی‌ثانیه |
| جستجو در ۲۰۰۰ مورد | کمتر از ۵۰ میلی‌ثانیه | ۱۰۰ میلی‌ثانیه |
| paste متن تا تزریق Ctrl+V | کمتر از ۱۵۰ میلی‌ثانیه | ۳۰۰ میلی‌ثانیه |
| حافظهٔ پایدار پس از ۱۰ دقیقه | کمتر از ۱۵۰ MiB | ۲۵۰ MiB |
| اندازهٔ payload هر صفحهٔ IPC | کمتر از ۲ MiB | ۴ MiB |

### روش بازبینی

1. release build را اجرا کنید، نه dev build.
2. تاریخچه‌های ۰، ۲۰۰ و ۲۰۰۰ موردی شامل متن و تصویر بسازید.
3. هر سناریو را ۳۰ بار اجرا و p50/p95 را ثبت کنید.
4. log محتوای clipboard را ثبت نکند؛ فقط span و مدت مجاز است.
5. regression بیشتر از ۱۵٪ باید در PR توضیح و تأیید شود.

</div>

---

## English

These SLOs apply to release builds on the reference host (4 CPU cores, 8 GiB RAM, SSD, Ubuntu LTS), measured independently on X11 and Wayland.

| Metric | p95 target | Release blocker |
| --- | ---: | ---: |
| Cold start to tray ready | < 1.5 s | 3 s |
| Shortcut to visible popup | < 120 ms | 250 ms |
| First 200-item history page | < 80 ms | 150 ms |
| Search across 2,000 items | < 50 ms | 100 ms |
| Text paste to Ctrl+V injection | < 150 ms | 300 ms |
| Steady memory after 10 minutes | < 150 MiB | 250 MiB |
| IPC payload per page | < 2 MiB | 4 MiB |

### Review protocol

1. Measure a release build, never a development build.
2. Use 0-, 200-, and 2,000-item datasets with text and images.
3. Run every scenario 30 times and record p50/p95.
4. Traces may contain spans and durations, never clipboard content.
5. A regression above 15% requires an explicit PR explanation and approval.
