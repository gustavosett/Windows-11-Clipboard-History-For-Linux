# Reports — index / گزارش‌ها — فهرست

Session reports and QA/audit write-ups are **archived**, not kept in the
living documentation tree. This keeps user-facing docs focused on the
product and its contracts.
گزارش‌های نشست و بازبینی به **آشیانه** منتقل می‌شوند تا مستندات
کاربر بر محصول و قراردادهای آن متمرکز بماند.

- [`docs/archive/reports/`](../archive/reports/) — all session reports
  (code reviews, QA passes, upgrade reports), newest first by date suffix.

Policy / سیاست:

1. New review/QA reports belong in `docs/archive/reports/` with an
   ISO-date suffix (`SOMETHING_YYYY-MM-DD.md`), or better, in the
   PR description itself.
2. `docs/` at the top level holds only living documents (guides,
   architecture, ADRs, threat model, performance budget, CI contract).
