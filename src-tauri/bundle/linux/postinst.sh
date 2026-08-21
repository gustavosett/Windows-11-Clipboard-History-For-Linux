#!/bin/bash
# Post-installation script for windows-11-style-clipboard-history-manager
# Only system-level tasks. NON-INTERACTIVE.
set -e

log() { echo "[windows-11-style-clipboard-history-manager] $1"; }

# 1. Ensure uinput module loads on boot
if [ -f /etc/modules-load.d/windows-11-style-clipboard-history-manager.conf ]; then
    # File exists - ensure it contains uinput
    if ! grep -qx "uinput" /etc/modules-load.d/windows-11-style-clipboard-history-manager.conf 2>/dev/null; then
        echo "uinput" >> /etc/modules-load.d/windows-11-style-clipboard-history-manager.conf
        log "Appended uinput to existing config"
    fi
else
    # File doesn't exist - create it
    echo "uinput" > /etc/modules-load.d/windows-11-style-clipboard-history-manager.conf
    log "Configured uinput to load on boot"
fi

# 2. Load module now (ignore error if already loaded)
modprobe uinput 2>/dev/null || true

# 3. Reload udev rules
udevadm control --reload-rules 2>/dev/null || true
udevadm trigger --subsystem-match=misc --attr-match=name=uinput 2>/dev/null || true

# 4. Update system caches
update-desktop-database -q /usr/share/applications 2>/dev/null || true
gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor 2>/dev/null || true

log "Installation completed successfully."
log "Note: You may need to log out and back in for input permissions to take effect."

exit 0
