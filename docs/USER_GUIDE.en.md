# Complete English Guide — Windows 11 Style Clipboard History Manager

> Documentation version: 2.5.0 — reviewed 2026-08-21

This is the English entry point for installation, daily use, privacy, troubleshooting, and development. See [Architecture](ARCHITECTURE.md), [Threat Model](THREAT_MODEL.md), [CI](CI.md), and the [ADRs](adr/) for deeper engineering details.

## 1. Overview

Windows 11 Style Clipboard History Manager is a local Linux clipboard manager for X11 and Wayland, inspired by Windows 11 Win+V. It provides text, rich-text and image history; search and pinning; emoji, kaomoji and symbol pickers; bilingual English/Persian Setup and Settings with automatic RTL; a stable English/LTR main popup; SQLite persistence; and ChaCha20-Poly1305 encryption for stored text and images.

## 2. Installation

Download packages only from GitHub Releases and verify `SHA256SUMS` first.

```bash
sha256sum -c SHA256SUMS --ignore-missing
# Debian / Ubuntu
sudo apt install ./windows-11-style-clipboard-history-manager_2.5.0_amd64.deb
# Fedora
sudo dnf install ./windows-11-style-clipboard-history-manager-2.5.0-1.x86_64.rpm
# Paste simulation permission
sudo setfacl -m u:$USER:rw /dev/uinput
```

Arch users can install `windows-11-style-clipboard-history-manager-bin` from AUR. Flatpak does not expose `/dev/uinput` by default; review [Packaging](../packaging/README.md) before granting a device override.

## 3. Usage

| Shortcut | Action |
| --- | --- |
| `Super+V` | Open clipboard history |
| `Ctrl+Alt+V` | Alternative shortcut |
| `Super+.` | Open emoji picker |
| `Enter` | Paste selected item |
| `Esc` | Close |
| `Ctrl+F` | Search |

Settings cover theme, opacity, UI scale, language, retention, automatic deletion, secret filtering, image storage, and encryption-key storage. Language changes apply immediately and update text direction.

## 4. Privacy and security

- Database: `~/.local/share/windows-11-style-clipboard-history-manager/history.db`
- Images: `~/.local/share/windows-11-style-clipboard-history-manager/images/`
- Settings: `~/.config/windows-11-style-clipboard-history-manager/user_settings.json`
- Key: local `history.key` (`0600`) or desktop Secret Service
- Default history cap: 2,000 items
- Network: zero calls in the default build; optional GIF search requires its build feature and `TENOR_API_KEY`.

Secret detection and password-manager exclusion are enabled by default, but pattern matching cannot guarantee detection of every secret. Wayland does not reveal the focused application, so application exclusion is X11-only. `/dev/uinput` is a powerful permission: install only verified binaries.

## 5. Troubleshooting

- **GNOME shortcut conflict:** use `Ctrl+Alt+V` or rebind GNOME’s notification shortcut.
- **Paste does not work:** verify `/dev/uinput` ACL and log out/in.
- **Black NVIDIA window:** launch with `IS_NVIDIA=1 windows-11-style-clipboard-history-manager`.
- **Sensitive app captured on Wayland:** keep secret filtering enabled; focused-window exclusion is unavailable.
- **Logs:** inspect `~/.local/share/windows-11-style-clipboard-history-manager/logs/` and redact sensitive data before sharing.

## 6. Development

Requirements include Node.js 20+, stable Rust, WebKitGTK 4.1, GTK3, and the system libraries listed in README.

```bash
make deps
npm ci
npm run lint
npm run test:coverage
npm run build
cd src-tauri
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-features
```

Every new UI key must be added to both `src/locales/en.json` and `src/locales/fa.json`. `src/i18n/locales.test.ts` enforces key parity and rejects empty translations.

## 7. Security reports and contributions

Do not disclose vulnerabilities in public issues. Follow [SECURITY.md](../.github/SECURITY.md) and use GitHub Private Vulnerability Reporting. Contribution workflow, commit rules, and the PR checklist are in [CONTRIBUTING.md](../.github/CONTRIBUTING.md).

## 8. Documentation map

- [Bilingual root README](../README.md)
- [Complete Persian guide](USER_GUIDE.fa.md)
- [Architecture](ARCHITECTURE.md)
- [Bilingual support](BILINGUAL.md)
- [Threat model](THREAT_MODEL.md)
- [CI contract](CI.md)
- [Architecture decisions](adr/)
- [Packaging](../packaging/README.md)
- [2026 repository review (Persian)](reports/REPOSITORY_REVIEW_2026-08-21.fa.md)
