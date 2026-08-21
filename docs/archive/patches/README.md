# Archived workflow patches / پچ‌های آرشیوشدهٔ ورک‌فلو

> **Status:** the earlier patches in this directory are historical/superseded
> — do not apply. The **active** activation patch is
> [`docs/archive/patches/hardened-ci-workflows.patch`](hardened-ci-workflows.patch);
> apply it with:
>
> ```bash
> git am docs/archive/patches/hardened-ci-workflows.patch && git push
> ```
>
> (A bot token without the `workflows` permission cannot push
> `.github/workflows/*`, so the patch travels instead — the maintainer step
> above activates `ci.yml` / `release.yml` / `e2e.yml` / `stale.yml`, after
> which `.github/workflows/` becomes the single source of truth.)
>
> **وضعیت:** پچ‌های پیشین این پوشه تاریخی/منسوخ‌اند — اعمال نکنید. پچ
> فعال‌سازیِ **جاری**
> [`docs/archive/patches/hardened-ci-workflows.patch`](hardened-ci-workflows.patch)
> است؛ آن را با دستور بالا اعمال کنید (توکن ربات بدون مجوز `workflows`
> نمی‌تواند `.github/workflows/*` را پوش کند، پس پچ به‌جای آن حمل می‌شود؛
> پس از اعمال، `.github/workflows/` منبع حقیقت واحد می‌شود).

## Contents / محتوا

- `workflows-rename-*.patch`, `enterprise-workflow-upgrade.patch` — earlier,
  superseded iterations. **Do not apply.**
  تکرارهای پیشین و منسوخ. **اعمال نکنید.**
- `hardened-ci-workflows.patch` — the **active final iteration** (2026-08-21):
  canonical `ci.yml` (rustup-only, packaging job incl. `flatpak-builder-lint`),
  `release.yml` (canonical artifact names/URLs/AUR, GPG/SBOM/SLSA), a new
  manual `e2e.yml`, and `actions/stale` pinned to a full commit SHA.
  Applies cleanly; verified in a pristine worktree (after applying,
  `scripts/check-packaging.sh` passes and the CI smoke test targets the
  canonical binary).
  تکرار نهاییِ **جاری** (۲۰۲۶-۰۸-۲۱): `ci.yml` رسمی (فقط rustup، job
  بسته‌بندی شامل `flatpak-builder-lint`)، `release.yml` (نام‌های رسمی
  آرتیفکت/URL/AUR، GPG/SBOM/SLSA)، `e2e.yml` دستی جدید و پین‌شدن
  `actions/stale` به SHA کامل. اعمال تمیز دارد؛ در کار درخت تمیز
  راستی‌آزمایی شد (پس از اعمال، `scripts/check-packaging.sh` سبز و smoke
  تست CI باینری رسمی را هدف می‌گیرد).
