#!/bin/bash
# Install the AppArmor profile for windows-11-style-clipboard-history-manager.
#
# Default mode is COMPLAIN (logs violations, blocks nothing) so the profile
# can be validated on your desktop environment without breaking the app.
# To enforce it, run: sudo aa-enforce /etc/apparmor.d/windows-11-style-clipboard-history-manager
#
# Usage: sudo ./install.sh [--enforce]

set -e

PROFILE_SRC="$(dirname "$0")/windows-11-style-clipboard-history-manager"
PROFILE_DEST="/etc/apparmor.d/windows-11-style-clipboard-history-manager"

if [ "$(id -u)" -ne 0 ]; then
    echo "Please run as root: sudo $0 [--enforce]" >&2
    exit 1
fi

if ! command -v apparmor_parser >/dev/null 2>&1; then
    echo "AppArmor tools not found (install the 'apparmor-utils' package)." >&2
    exit 1
fi

install -m 644 "$PROFILE_SRC" "$PROFILE_DEST"

if [ "${1:-}" = "--enforce" ]; then
    apparmor_parser -r -W "$PROFILE_DEST"
    echo "AppArmor profile installed and ENFORCED for windows-11-style-clipboard-history-manager."
    echo "If the app misbehaves, check: sudo aa-status | grep clipboard"
    echo "and the log: sudo journalctl -k | grep -i apparmor"
else
    # complain mode: log violations without blocking
    apparmor_parser -r -C -W "$PROFILE_DEST"
    echo "AppArmor profile installed in COMPLAIN mode (logs only, no blocking)."
    echo "To enforce: sudo aa-enforce /etc/apparmor.d/windows-11-style-clipboard-history-manager"
    echo "To remove:  sudo aa-remove /etc/apparmor.d/windows-11-style-clipboard-history-manager"
fi
