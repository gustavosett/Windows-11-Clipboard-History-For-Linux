#!/bin/bash
# Post-removal script for win11-clipboard-history
set -e

case "$1" in
    purge)
        # Remove module configuration (handle both possible filenames for compatibility)
        rm -f /etc/modules-load.d/win11-clipboard.conf
        rm -f /etc/modules-load.d/uinput.conf
        
        # Update caches
        update-desktop-database -q /usr/share/applications 2>/dev/null || true
        gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor 2>/dev/null || true
        ;;
esac

exit 0
