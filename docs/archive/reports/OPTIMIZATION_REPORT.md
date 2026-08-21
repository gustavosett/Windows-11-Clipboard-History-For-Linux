# Status — 2.0.0

The 2026-08-20 production hardening pass is implemented. See `CHANGELOG.md`.

Still intentionally deferred (not 2.0 blockers):

- Event-driven X11 clipboard via XFixes (adaptive 200/800ms polling remains)
- Splitting every DE handler in `linux_shortcut_manager.rs` into per-DE crates (tiling WM rewrite is now gated)
- xdg-desktop-portal input instead of raw uinput (documented; Flatpak stays sandboxed)
