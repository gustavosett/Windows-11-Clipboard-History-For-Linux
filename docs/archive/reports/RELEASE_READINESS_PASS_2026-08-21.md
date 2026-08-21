# Release-Readiness Pass — Final Report / گزارش نهایی گردش آماده‌سازی انتشار

**Date / تاریخ:** 2026-08-21 · **Base / پایه:** `eec68cf` · **PR:** #21
**Scope:** every blocker + recommendation from the senior review
(SENIOR_AUDIT_2026-08-21.fa.md, REPOSITORY_REVIEW_2026-08-21.fa.md).

---

## 1. Blockers resolved / بلاکرهای رفع‌شده

| # | Blocker (before) | Fix | Verification |
|---|---|---|---|
| 1 | CI red ×6 — `Install Rust` step (dtolnay action) | rustup from `rust-toolchain.toml`, retry ×3, no third-party action | step logic reviewed; bash-validated; shipped in the workflow patch |
| 2 | Smoke test probed non-existent `win11-clipboard-history-bin` | canonical binary name | `check-packaging.sh` guard #2 |
| 3 | 17 legacy names in CI/release workflows (AUR → dead repo, release notes → wrong files) | full canonical rename (`windows-11-style-clipboard-history-manager*`) | guard #1 + repo-wide scan |
| 4 | `check-packaging.sh` failed on master | now green + wired as dedicated blocking CI job | local run: exit 0 |
| 5 | No artifact-name contract | `scripts/normalize-artifacts.sh` (idempotent) | unit-tested on 6 naming variants |
| 6 | Version sync covered 3 of 6 sources | all six incl. Cargo.lock/changelog/metainfo | simulated tag 2.6.0 end-to-end ✔ |
| 7 | Metainfo screenshot 404 (`main` branch) | → `master` + bilingual release notes | XML well-formed; guard #5 |

## 2. Quality upgrades / ارتقاءهای کیفی

- **Setup wizard:** 748-line monolith → 216-line orchestrator +
  9 single-purpose modules (`src/components/setup/`); +5 unit tests
  (135/135 passing); behaviour identical.
- **UI motion:** Windows 11-style `animate-window-in` / `animate-step-in`
  entrances, auto-disabled under `prefers-reduced-motion`.
- **Docs:** reports archived with policy index; fallback patch
  regenerated from fixed pipelines (`git apply --check` ✔ on base);
  obsolete patches archived; README/CONTRIBUTING/CI.md synced;
  `engines.node>=20.19` + `.nvmrc`; `make packaging` target.

## 3. Gate results (local, this pass) / نتایج گیت‌ها

| Gate | Result |
|---|---|
| `npm run lint` (tsc + type-aware ESLint, 0 warnings) | ✅ |
| `npm run test:coverage` (135/135 + thresholds) | ✅ |
| `npm run build` (vite production) | ✅ |
| `node scripts/check-rust-syntax.mjs` (56 files) | ✅ |
| `bash scripts/check-packaging.sh` (5 contract guards) | ✅ |
| Workflow YAML parse (live + mirrors) | ✅ |
| `bash -n` all shell scripts (10) | ✅ |
| Version-sync simulation (tag 2.6.0) | ✅ 6/6 sources |
| Artifact rename variants | ✅ 6/6 |
| Patch applicability on base commit | ✅ |

Rust compile gates (`cargo fmt/clippy/test/audit/deny`) run on GitHub
runners — unchanged Rust sources, identical pinned deps.

## 4. Release checklist / چک‌لیست انتشار

1. Merge PR #21.
2. As maintainer (workflows scope):
   `git am docs/patches/hardened-ci-workflows.patch && git push`
   — activates the fixed CI/release pipelines.
3. Watch CI go green on master.
4. `git tag v2.5.0 && git push origin v2.5.0` → artifacts (deb/rpm/
   AppImage, SHA256SUMS, SBOM, SLSA) publish with canonical names.
5. Flatpak: `packaging/flatpak/build.sh` (paste simulation limited by
   the sandbox — documented; `--device=all` override available).
