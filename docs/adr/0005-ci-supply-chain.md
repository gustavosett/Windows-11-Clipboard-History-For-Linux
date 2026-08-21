# ADR-0005: Blocking CI quality gates and signed releases

- **Status:** Implemented (v2.3.0) — the hardened workflows now live in
  `.github/workflows/` (the staging copies under `docs/github-workflows/`
  were removed; a single source of truth).
- **Date:** 2026-08-20

## Context / زمینه

README and CHANGELOG previously claimed blocking `cargo audit` / `npm audit`,
published `SHA256SUMS`, SPDX SBOMs and SLSA provenance. The workflows still
had `continue-on-error: true` on audits, ran no tests, and the release notes
pointed at the upstream fork.

مستندات ادعا می‌کرد گیت‌های امنیتی الزامی‌اند، اما workflowها audit را
با `continue-on-error` اجرا می‌کردند، تست نداشتند و Release به فورک بالادستی
اشاره می‌کرد.

## Decision / تصمیم

1. CI job `quality` runs `npm run lint`, `npm run test:coverage`,
   `cargo fmt --check`, `cargo clippy -D warnings`, and `cargo test`.
2. CI job `security` runs `cargo audit`, `cargo deny check advisories bans
   licenses sources` (see `src-tauri/deny.toml`), and
   `npm audit --audit-level=high` — all **without** `continue-on-error`.
3. `build-linux` depends on both jobs and smokes `--version` / `--help`
   (bare and under `xvfb-run`).
4. Tagged releases publish `SHA256SUMS`, a **per-artifact** SPDX SBOM
   (syft scan of each `.deb` / `.rpm` / `.AppImage`), and SLSA provenance
   (`actions/attest-build-provenance`). All URLs target
   `Mahdi-Arts/Windows-11-Style-Clipboard-History-Manager`.
5. CI declares least-privilege `permissions: contents: read`,
   `persist-credentials: false` on checkouts, and concurrency groups that
   cancel stale runs.

## Consequences / پیامدها

- A known high-severity advisory turns the PR red.
- The installer can require checksums because releases actually attach
  `SHA256SUMS`.
- Cloudsmith upload remains best-effort (`continue-on-error`) because it
  needs a secret that forks may not have.
- `cargo-deny` licenses use an explicit OSI allow-list
  (`src-tauri/deny.toml`); adding a dependency under a new license is a
  visible, reviewed change.
