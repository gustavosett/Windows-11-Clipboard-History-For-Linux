# 🛡️ Threat Model — Windows 11 Style Clipboard History Manager

> **Status:** Living document · **Applies to:** v2.5.0
> This document describes the assets, trust boundaries, threat agents, and
> the controls in place. It is the reference for security reviews and for
> any future hardening work (e.g. optional encryption).

---

## 1. Assets

| # | Asset | Location | Sensitivity |
| --- | --- | --- | --- |
| A1 | Clipboard history (text + rich text) | `~/.local/share/windows-11-style-clipboard-history-manager/history.db` (SQLite, WAL, `0600`) | **High** — may contain copied credentials, personal data |
| A2 | Clipboard images | `~/.local/share/windows-11-style-clipboard-history-manager/images/*.png` (`0600`, ChaCha20-Poly1305 envelope; legacy plaintext PNG still readable) | Medium-High |
| A3 | User settings | `~/.config/windows-11-style-clipboard-history-manager/user_settings.json` (`0600`) | Low-Medium |
| A4 | Emoji usage / custom kaomoji | `~/.local/share/windows-11-style-clipboard-history-manager/emoji_history.json` | Low |
| A5 | GIF cache | `~/.cache/windows-11-style-clipboard-history-manager/gifs/` | Low |
| A6 | Logs | `~/.local/share/windows-11-style-clipboard-history-manager/logs/` (`0700`) | Medium — may contain window titles, commands |
| A7 | Shortcut registrations (DE configs) | `~/.config/…` (gsettings, khotkeysrc, i3 config, …) | Medium |

## 2. Trust boundaries

```
┌─────────────────────────── user session (single user, local) ───────────────────────────┐
│                                                                                          │
│  Other local processes ──► clipboard (X11 selection / Wayland) ──► watcher ──► SQLite    │
│  (same user, unprivileged)        ▲                                                       │
│                                   │                                                       │
│  Web content in clipboard ────────┘  (URLs, HTML, images from any app)                   │
│                                                                                          │
│  App webview (CSP-restricted) ◄──IPC──► Rust core (privileged ops: uinput, pkexec)      │
│                                                                                          │
│  Network (only on user action): Tenor HTTPS (GIF search, env-key gated)                  │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

Key boundaries:
1. **Clipboard → persistence**: content from any app (potentially hostile) enters the history. Privacy filter + secret filter gate this.
2. **Webview → Rust**: the frontend can only call the registered Tauri commands (capability-gated **per window** — paste/injection is `main` only); `withGlobalTauri: false`; CSP `script-src 'self'`.
3. **Rust → system**: the binary can write `/dev/uinput` (keystroke injection) and run privilege helpers (`pkexec setfacl`). This is the highest-privilege capability; see §4.4.
4. **Rust → network**: outbound HTTP only through the SSRF-validated downloader (GIFs) and the Tenor API proxy.

## 3. Threat agents & capabilities

| Agent | Capability | Primary targets |
| --- | --- | --- |
| T1. Local malware (same user) | Full read of user files | A1–A7 — **not preventable by app controls**; rely on OS permissions + threat model note |
| T2. Other local users (multi-user system) | Read files they have permissions for | A1–A6 — mitigated by `0600/0700` + `owner` AppArmor rules |
| T3. Clipboard content (hostile text/URLs/HTML) | Injects strings the app processes | SSRF via smart actions, regex ReDoS in search, secret exfiltration |
| T4. Network adversary (MITM on HTTPS) | Read/modify TLS traffic (if certs compromised) | GIF downloads — mitigated by HTTPS + host allowlist + DNS pinning; no clipboard data ever leaves the machine |
| T5. Supply-chain (compromised release/installer) | Ship malicious binaries | All users — mitigated by checksums, provenance attestation, SBOM |

## 4. Controls per threat

### 4.1 Clipboard poisoning (T3)
- **Secret filter** (`privacy.rs`): drops PEM private keys, JWTs, known token prefixes (`ghp_`, `sk_live_`, `AKIA…`), and `password=`-style assignments of **any length** before they reach history. Default ON.
- **Sensitive-source skip** (X11 only): password managers and incognito/private windows are excluded via WM_CLASS/title matching (`window_identity.rs`). Wayland compositors do not expose focus identity — documented limitation.
- **Event-driven capture** (`clipboard_events.rs`): XFixes `SelectionNotify` on
  X11 and `wl-paste --watch` on Wayland wake the watcher instantly; any
  failure falls back to adaptive polling (200/800 ms), so capture latency
  shrinks without a correctness regression. The Wayland helper child is
  reaped via `PR_SET_PDEATHSIG` — no orphan processes survive the app.
- **Smart-action URL sanitizer** (`urlSafety.ts` + `open_url.rs`): HTTPS-only
  protocol allowlist (`https/mailto`); plain `http://` input is upgraded to
  `https://` in the frontend and rejected outright by Rust. Blocks
  credentials, control chars, localhost, private/loopback/CGNAT IPs. The
  webview calls `open_safe_url`; Rust re-validates and execs `xdg-open` (no
  Tauri `shell:allow-open`).
- **Regex search guard** (`historySearch.ts`): length cap (80), nested-quantifier detection, try/catch — prevents ReDoS on the UI thread.

### 4.2 Local exposure (T2)
- SQLite DB, images, settings, logs: `0600` files / `0700` dirs via `fs_atomic::restrict_permissions`.
- History text columns **and image files** are encrypted at rest (ChaCha20-Poly1305; see ADR 0004 and ADR 0008).
- **Key backends (ADR-0006):** the encryption key lives in `history.key`
  (`0600`) or, on request, in the freedesktop **Secret Service** keyring
  (`secret-tool`; key never touches disk). Adoption of any backend requires
  decrypting the `history.key.check` marker — **fail-closed**; a wrong or
  missing key can never silently re-encrypt history.
  بک‌اندهای کلید (ADR-0006): کلید در فایل یا Secret Service نگه‌داری
  می‌شود؛ پذیرش هر بک‌اند مستلزم رمزگشایی نشانگر `history.key.check` است
  (fail-closed) تا کلید اشتباه هرگز بی‌سروصدا تاریخچه را کلیدعوض نکند.
- AppArmor profile (complain by default) restricts file/socket access to the app's own XDG dirs + `/dev/uinput`.
- Atomic writes (`write_atomic`) prevent partial/corrupt state and .tmp disclosure.
- **Bounded IPC reads (ADR-0007):** `get_history_page` clamps `limit` to
  `1..=200` server-side; the webview cannot request unbounded payloads.

### 4.3 Outbound network (T4)
- **SSRF validator** (`ssrf.rs`): HTTPS-only; host allowlist (`tenor.com`, `giphy.com`, `media.tenor.co`); direct-IP URLs rejected; DNS resolved and **every** address checked against a private/loopback/CGNAT/metadata blocklist; the HTTP client **pins** the connection to the validated addresses (`resolve_to_addrs`) closing the DNS-rebinding window; redirects are refused; 10 MB streamed download cap; Content-Type sanity check.
- **Tenor API key** is read from the environment only — never bundled, never sent to the frontend.
- CSP: `connect-src 'self' ipc:` — the webview cannot open arbitrary network connections. Fonts are bundled; **the app makes zero network calls in normal operation** (v2.1.0+).

### 4.4 Keystroke injection capability (T1-adjacent risk)
- `/dev/uinput` (Wayland) or XTest (X11) can synthesize Ctrl+V into the focused window.
- Guards: paste commands accept **only the `main` window** (Rust `WebviewWindow::label` + per-window capabilities). `paste_item` / `paste_text` inject only after `wrote_recently(5s)` following their own clipboard write — they do **not** mint a self-consumed ticket. `finish_paste` (GIF path) still requires a one-shot ticket; paste text is capped at 1 MiB; the paste transaction is serialized (`paste_gate`); on X11 the previous focused window is restored and **verified** before injection (`focus_manager`/`paste_sync`); the popup window hides and releases focus first.
- udev rule grants access only to the logged-in session (`TAG+="uaccess"`); AppArmor profile restricts the device to the app binary.
- **Residual risk (documented):** a compromised binary could type into any focused window. Users should treat the binary like a trusted input device (README + SECURITY.md).

### 4.5 Supply chain (T5)
- Installer verifies artifacts against release `SHA256SUMS` (**mandatory**; `ALLOW_UNVERIFIED=1` required to skip) and optionally verifies a detached GPG signature.
- The hardened CI contract (`.github/workflows/` — the SHA-pinned,
  canonical-named pipelines) runs `cargo audit` + `cargo deny`
  (advisories/bans/licenses/sources) + `npm audit` as **blocking** gates
  (no `continue-on-error`); type-aware ESLint + `tsc`; `cargo clippy -D
  warnings`; frontend coverage thresholds; release-binary smoke
  (`--version`/`--help`, bare and under `xvfb-run`). Every workflow action
  is pinned to a full commit SHA (OpenSSF recommendation).
- Releases publish: SHA256SUMS (plus an optional GPG `SHA256SUMS.sig` when
  `RELEASE_GPG_PRIVATE_KEY` is configured), **per-artifact** SPDX SBOM
  (syft), SLSA build-provenance attestations.
- Every URL in the release pipeline points at
  `Mahdi-Arts/Windows-11-Style-Clipboard-History-Manager`; optional channels
  (Cloudsmith/AUR) activate only when repository secrets exist.
- AUR PKGBUILD checksums are populated by the release workflow. The AUR SSH
  connection is **fail-closed**: `StrictHostKeyChecking yes` with
  `known_hosts` pinned via the `AUR_KNOWN_HOSTS` secret — trust-on-first-use
  is never accepted.

## 5. Known limitations (accepted risk)

| Limitation | Risk | Mitigation / status |
| --- | --- | --- |
| Wayland: no focused-app detection | Secrets from password managers can land in history on Wayland | Secret filter still applies; UI warns; documented in README |
| History text is field-encrypted; key is a local `history.key` (default) | A local attacker with the same UID can still read the key | `0600` + ChaCha20-Poly1305 (fail-closed); **Secret Service keyring backend available** (ADR-0006) via Settings → Privacy; same-UID compromise stays out of scope |
| `style-src 'unsafe-inline'` in CSP | XSS would still be constrained (no `script-src` relaxation), but inline styles are allowed | Required by Tailwind/React inline styles; revisit with hashing if feasible |
| pkexec prompt surface | Social-engineering of the "Fix permissions" flow | Only triggered by explicit user action; command is argv-fixed (`setfacl -m u:<user>:rw /dev/uinput`) |
| AppImage/Flatpak cannot install udev rules | Paste unavailable until user grants `/dev/uinput` | Documented; deb/rpm are the recommended channels |

## 6. Security checklist for a release

- [ ] `cargo audit`, `cargo deny`, and `npm audit` pass (blocking in CI)
- [ ] `npm run lint`, `npm test`, coverage thresholds pass
- [ ] `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check` pass
- [ ] Release-binary smoke (`--version` / `--help`) passes
- [ ] SHA256SUMS generated and attached (GPG `.sig` when key configured)
- [ ] Per-artifact SBOM + provenance attestations attached
- [ ] AUR checksums updated by workflow (`AUR_SSH_KEY` + `AUR_KNOWN_HOSTS`)
- [ ] CHANGELOG entry added
