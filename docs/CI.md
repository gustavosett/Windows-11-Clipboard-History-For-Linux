# CI contract / قرارداد CI

> **English** below the Persian section. / بخش انگلیسی پایین‌تر است.

<div dir="rtl">

این سند قرارداد گیت‌های کیفیت است. ورک‌فلوهای سخت‌شده و نام‌گذاری‌شدهٔ
رسمی **آمادهٔ فعال‌سازی با یک گام نگهدارنده‌اند** — پچ فعال‌سازی نهایی در
[`docs/archive/patches/hardened-ci-workflows.patch`](archive/patches/hardened-ci-workflows.patch)
بایگانی شده است:

```bash
git am docs/archive/patches/hardened-ci-workflows.patch && git push
```

پس از اعمال، [`.github/workflows/`](../.github/workflows/) **منبع حقیقت
واحد** می‌شود (`ci.yml`، `release.yml`، `e2e.yml` دستی، `stale.yml`)؛ کپی
آینه‌ای `docs/github-workflows/` برای همیشه حذف شده است.

همهٔ actionها به SHA کامل کامیت پین شده‌اند (توصیهٔ OpenSSF). نصب Rust با
rustup خودِ رانر و از `rust-toolchain.toml` مخزن انجام می‌شود (بدون اکشن
شخص ثالث، با تلاش مجدد روی خطای گذرای شبکه).

## گیت‌های مسدودکننده (`.github/workflows/ci.yml`)

| Job | بررسی | مسدودکننده؟ |
| --- | --- | --- |
| quality | `npm run lint` (tsc + ESLint، صفر هشدار) | بله |
| quality | `npm run test:coverage` (آستانه‌های Vitest) | بله |
| quality | `cargo fmt --all -- --check` | بله |
| quality | `cargo clippy --all-targets -- -D warnings` | بله |
| quality | `cargo test` (feature پیش‌فرض، بدون HTTP) | بله |
| quality | `cargo test --all-features` | بله |
| security | `cargo audit` | بله |
| security | `cargo deny check advisories bans licenses sources` | بله |
| security | `npm audit --audit-level=high` | بله |
| packaging | `scripts/check-packaging.sh` (نام‌های رسمی، همگامی نسخه‌ها، برابری deb/rpm، اعتبارسنجی desktop/metainfo) | بله |
| packaging | `flatpak-builder-lint` روی manifest و metainfo فلت‌پک | بله |
| build-linux | بیلد Tauri + نرمال‌سازی نام آرتیفکت‌ها (`scripts/normalize-artifacts.sh`) + `--version` / `--help` روی باینری (با و بدون xvfb) | بله |

`continue-on-error` روی هیچ گیت امنیتی نیست.

## تست‌های E2E (`.github/workflows/e2e.yml`)

ورک‌فلوی `E2E Tests` فقط با `workflow_dispatch` اجرا می‌شود (سنگین است:
برنامه را واقعاً می‌سازد و با Playwright در webkit/chromium/firefox
میماند). برای انتشار عمده، اجرای دستی آن روی حداقل webkit توصیه می‌شود.

## انتشار (`.github/workflows/release.yml`)

با تگ `v*` ساخته می‌شود:

- `.deb` / `.rpm` / AppImage برای x86_64 و aarch64
- `SHA256SUMS` (+ امضای اختیاری GPG با secret ی `RELEASE_GPG_PRIVATE_KEY`)
- SPDX SBOM به ازای هر آرتیفکت (`syft`)
- گواهی SLSA build-provenance
- همهٔ URLها به `Mahdi-Arts/Windows-11-Style-Clipboard-History-Manager`

Cloudsmith و AUR فقط وقتی secretهای مخزن تنظیم شده باشند فعال می‌شوند.
اتصال AUR با `StrictHostKeyChecking yes` و `known_hosts` پین‌شده از secret ی
`AUR_KNOWN_HOSTS` انجام می‌شود — هرگز trust-on-first-use.

</div>

---

This document is the quality-gate contract. The hardened, canonical-named
workflows are **ready to activate with one maintainer step** — the final
activation patch is archived at
[`docs/archive/patches/hardened-ci-workflows.patch`](archive/patches/hardened-ci-workflows.patch):

```bash
git am docs/archive/patches/hardened-ci-workflows.patch && git push
```

After that, [`.github/workflows/`](../.github/workflows/) is the **single
source of truth** (`ci.yml`, `release.yml`, a manual `e2e.yml` Playwright
workflow, `stale.yml`); the `docs/github-workflows/` mirror is gone for good.

All actions are pinned to full commit SHAs (OpenSSF recommendation).
Rust is installed with the runner's own rustup from the repository's
`rust-toolchain.toml` (no third-party action; transient network failures
are retried).

## Blocking gates (`.github/workflows/ci.yml`)

Every row in the table above is a hard failure. Audits do **not** use
`continue-on-error`. Default-feature `cargo test` proves the release
binary compiles **without** our optional `reqwest` / GIF search feature.
The `packaging` job additionally runs `flatpak-builder-lint` over the
Flatpak manifest and AppStream metainfo so the Flatpak story cannot drift.

## E2E tests (`.github/workflows/e2e.yml`)

The `E2E Tests` workflow runs on `workflow_dispatch` only (it is heavy:
it builds the real app and drives it with Playwright across
webkit/chromium/firefox). For major releases, run it manually on at least
webkit before tagging.

## Releases (`.github/workflows/release.yml`)

Tag `v*` publishes checksums (plus an optional GPG `SHA256SUMS.sig` driven
by `RELEASE_GPG_PRIVATE_KEY`), per-artifact SPDX SBOMs, and SLSA
attestations. Artifact filenames are normalized to the canonical lowercase
package name by `scripts/normalize-artifacts.sh` before upload, and the
version-sync step keeps `package.json`, `Cargo.toml`, `Cargo.lock`,
`tauri.conf.json`, the Debian changelog and the AppStream metainfo aligned. Optional channels (Cloudsmith, AUR) require repository
secrets; they never silently point at a third-party fork. The AUR SSH
connection is fail-closed: `StrictHostKeyChecking yes` with `known_hosts`
pinned via the `AUR_KNOWN_HOSTS` secret — trust-on-first-use is never
accepted.
