#!/bin/bash
# Optional end-to-end smoke via tauri-driver + WebDriver.
# اسموک تست انتها-به-انتها با tauri-driver و WebDriver (اختیاری).
#
# Usage / استفاده:
#   ./scripts/e2e.sh          # installs deps, runs the E2E smoke
#
# Requirements / پیش‌نیازها:
#   - Rust toolchain + system deps of the app (see Makefile `deps`)
#   - tauri-driver:  cargo install tauri-driver
#   - WebDriver client: a WebdriverIO test file in tests/e2e/ (see below)
#
# CI already runs a release-binary smoke (--version/--help under xvfb) on
# every commit; this script is for local, browser-level checks of the real
# UI (window opens, history renders, ESC hides) before a release.
# CI روی هر کامیت اسموک باینری انتشار را اجرا می‌کند؛ این اسکریپت برای
# بررسی‌های سطح مرورگرِ UI واقعی پیش از انتشار است.

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

if ! command -v tauri-driver >/dev/null 2>&1; then
  echo "[!] tauri-driver not found; installing with cargo (first run only)..." >&2
  cargo install tauri-driver
fi

if ! command -v xvfb-run >/dev/null 2>&1; then
  echo "[!] xvfb-run not found; install xvfb (needed for headless WebKitGTK)" >&2
  exit 1
fi

echo "[*] Building the debug app for the driver ..."
npm run tauri:build -- --debug --no-bundle >/dev/null 2>&1 || \
  (cd src-tauri && cargo build)

echo "[*] Starting tauri-driver (port 4444) under Xvfb ..."
xvfb-run --auto-servernum --server-args="-screen 0 1280x800x24" \
  tauri-driver --port 4444 &
DRIVER_PID=$!
trap 'kill "$DRIVER_PID" 2>/dev/null || true' EXIT
sleep 3

echo "[*] Running E2E checks ..."
node tests/e2e/smoke.mjs

echo "[✓] E2E smoke passed."
