# ADR-0002: uinput/XTest paste injection architecture

- **Status:** Accepted (v1.x, hardened v2.0.0)
- **Date:** 2026-08-20

## Context

On Wayland, the classic "write clipboard + send fake Ctrl+V via X11 XTEST"
does not work because compositors isolate input synthesis. The app must paste
into the previously focused window from a background popup.

## Decision

Use a **persistent kernel uinput virtual keyboard** on Wayland and a
**persistent XTest connection** on X11, selected once at startup
(`session.rs`, `input_simulator.rs`):

- Devices are created/warmed once (not per paste) to avoid compositor
  attach races (a known cause of literal `v` keystrokes under load).
- Paste is a serialized transaction (`paste_gate`), gated on a real
  clipboard write within the last 5 seconds.
- On X11 the previous focus is saved, restored, and **verified stable**
  (two consecutive focus samples) before injecting.

## Consequences

- Requires `/dev/uinput` (udev rule ships with the package, `uaccess`-tagged)
  or XTest; the binary effectively is a trusted input device.
- The permission surface is documented in README/SECURITY/THREAT_MODEL and
  constrained by the optional AppArmor profile.
- Alternative approaches (portal-based RemoteDesktop, wtype) were rejected
  for lack of universal availability / reliability.
