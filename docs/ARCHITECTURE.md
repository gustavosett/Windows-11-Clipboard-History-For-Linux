# Architecture / معماری

> **English** below the Persian section. / بخش انگلیسی پایین‌تر است.

<div dir="rtl">

## نمای کلی

برنامه یک مدیر تاریخچه کلیپ‌بورد دسکتاپ است که تجربهٔ Win+V ویندوز ۱۱ را روی لینوکس (Wayland و X11) بازسازی می‌کند.

```
┌──────────── Webview (React 19) ────────────┐
│  main / setup / settings                   │
│  CSP: script-src 'self'                    │
└─────────────── IPC (Tauri) ────────────────┘
                    │
┌──────────── Rust core ─────────────────────┐
│  Watcher → Privacy → SQLite (encrypted)    │
│  Paste ticket → uinput / XTest             │
│  SSRF + DNS pin (Tenor, optional)          │
└────────────────────────────────────────────┘
```

## مرزهای اعتماد

1. **کلیپ‌بورد → persistence:** محتوای هر برنامه وارد تاریخچه می‌شود؛ فیلتر اسرار و skip مدیر رمز عبور جلوی آن را می‌گیرند.
2. **Webview → Rust:** فقط فرمان‌های ثبت‌شده؛ `withGlobalTauri: false`.
3. **Rust → سیستم:** `/dev/uinput` و `pkexec`؛ بلیت paste و گیت `wrote_recently`.
4. **Rust → شبکه:** فقط دانلود GIF با allowlist + DNS pinning.

## نقشهٔ ماژول‌های هسته

ماژول `clipboard_manager/` به زیرماژول‌های متمرکز تفکیک شده است:
`types` (مدل دامنه)، `deduplication` (خط لولهٔ ثبت)، `persistence`
(SQLite رمزنگاری‌شده)، `history_access` (خواندن، صفحه‌بندی، سنجاق،
نگهداشت) و `clipboard_write` (I/O کلیپ‌بورد سیستم). ارجاعات داخلی بین
زیرماژول‌ها `pub(super)` هستند و فراخوان‌های بیرونی فقط `pub` می‌بینند.

جزئیات تهدید در [`THREAT_MODEL.md`](THREAT_MODEL.md).

</div>

---

## Overview

The app recreates the Windows 11 Win+V clipboard history on Linux (Wayland and X11).

```mermaid
flowchart LR
  subgraph UI["Webview — React 19"]
    Main[main window]
    Setup[setup wizard]
    Settings[settings]
  end
  subgraph Core["Rust / Tauri v2"]
    Watcher[clipboard watcher]
    Privacy[secret + app filters]
    Store[SQLite WAL + ChaCha20-Poly1305]
    Paste[paste ticket + uinput/XTest]
  end
  Clipboard[(X11 / Wayland clipboard)] --> Watcher
  Watcher --> Privacy --> Store
  Main -->|IPC paste_item| Paste
  Paste --> Clipboard
```

## Layers

| Layer | Stack | Role |
| --- | --- | --- |
| UI | React 19, TypeScript strict, Tailwind 4 | Three windows; lazy pickers; virtualized list |
| Backend | Rust, Tauri v2 | Domain modules, typed `AppError` |
| Clipboard I/O | arboard + `wl-copy` / `xclip` | Read/write with fallback |
| Persistence | SQLite WAL, field + image encryption, PNG thumbs | Cap 2000 items; bounded paged reads |
| Key storage | `history.key` (0600) or Secret Service keyring | Marker-verified, fail-closed |
| Input | Persistent uinput / XTest | Ctrl+V after an authorized write |
| Packaging | deb / rpm / AppImage / AUR / Flatpak | Multi-distro |

## Core module map / نقشهٔ ماژول‌های هسته

`clipboard_manager/` is a package of focused submodules (split from a single
1250-line file): `types` (domain model), `deduplication` (capture pipeline),
`persistence` (encrypted SQLite), `history_access` (reads, paging, pin,
retention), `clipboard_write` (OS clipboard I/O). Cross-module internals use
`pub(super)`; external callers see only `pub` items.

## Paste authorization

1. Write the payload to the OS clipboard.
2. Issue a one-shot paste ticket (5 s TTL).
3. Consume the ticket and require `wrote_recently(5s)`.
4. Hide the popup, restore the previous focus (X11), inject Ctrl+V.

## Related ADRs

- [0001 SQLite persistence](adr/0001-sqlite-persistence.md)
- [0002 Paste injection](adr/0002-paste-injection-architecture.md)
- [0003 SSRF DNS pinning](adr/0003-ssrf-dns-pinning.md)
- [0004 Field encryption](adr/0004-field-encryption.md)
- [0005 CI & supply chain](adr/0005-ci-supply-chain.md)
- [0006 Secret Service key storage](adr/0006-secret-service-key-storage.md)
- [0007 IPC pagination](adr/0007-ipc-pagination.md)
- [0008 Window ACL + image encryption](adr/0008-window-acl-and-image-encryption.md)
