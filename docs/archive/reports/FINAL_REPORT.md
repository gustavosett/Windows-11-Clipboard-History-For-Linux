# Quality report / گزارش کیفیت — v2.3.0

> DevOps QA loop after the enterprise hardening pass.
> حلقهٔ کنترل کیفیت پس از ارتقاء سطح سازمانی.

## Scores / امتیازها (post-upgrade)

| Area / بخش | Score / از ۱۰ |
| --- | --- |
| Code & architecture / کیفیت کد و معماری | **9.0** |
| Security / امنیت | **9.0** |
| Documentation / مستندات | **9.0** |
| Scalability / قابلیت توسعه | **8.0** |
| **Overall / میانگین** | **8.75** |

## Gates that now match the docs / گیت‌هایی که با مستندات هم‌خوان شدند

- CI `quality`: lint, coverage, `cargo test`, clippy `-D warnings`, rustfmt
- CI `security`: `cargo audit` + `npm audit --audit-level=high` (blocking)
- CI `build-linux`: depends on both; `--version` / `--help` smoke
- Release: `SHA256SUMS`, SPDX SBOM, SLSA provenance; URLs only this repo

## Residual accepted risk / ریسک پذیرفته‌شده

- `/dev/uinput` is a trusted-input-device capability (documented).
- `history.key` is same-UID readable; libsecret wrapping remains roadmap.
- AppArmor ships in complain mode; `--enforce` is opt-in.
- Flatpak has no `/dev/uinput` unless overridden.
