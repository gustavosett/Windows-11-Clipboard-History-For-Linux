#!/bin/bash
# Wrapper for win11-clipboard-history
# Purpose: Clean environment to avoid Snap/Flatpak library conflicts
#          and force X11/XWayland for window positioning on Wayland

set -e

BINARY_LOCATIONS=(
    "/usr/bin/win11-clipboard-history-bin"
    "/usr/lib/win11-clipboard-history/win11-clipboard-history-bin"
    "/usr/local/lib/win11-clipboard-history/win11-clipboard-history-bin"
)

# Find the binary
BINARY=""
for loc in "${BINARY_LOCATIONS[@]}"; do
    if [ -x "$loc" ]; then
        BINARY="$loc"
        break
    fi
done

# Verify binary was found
if [ -z "$BINARY" ]; then
    echo "Error: win11-clipboard-history binary not found." >&2
    echo "The wrapper searched for an executable in the following locations (in order):" >&2
    for loc in "${BINARY_LOCATIONS[@]}"; do
        echo "  - $loc" >&2
    done
    echo "" >&2
    echo "If you installed via package manager, try reinstalling the package." >&2
    echo "If you installed manually with a custom PREFIX, ensure the binary is in one of the locations above." >&2
    exit 1
fi

# Clean up environment to avoid Snap/Flatpak library conflicts
unset LD_LIBRARY_PATH
unset LD_PRELOAD
unset GTK_PATH
unset GIO_MODULE_DIR
unset WEBKIT_DISABLE_COMPOSITING_MODE

export GDK_SCALE="${GDK_SCALE:-1}"
export GDK_DPI_SCALE="${GDK_DPI_SCALE:-1}"

export GDK_BACKEND="x11"
export WEBKIT_DISABLE_COMPOSITING_MODE=1
export TAURI_TRAY="${TAURI_TRAY:-libayatana-appindicator3}"

# Disable AT-SPI to prevent accessibility bus warnings/delays
export NO_AT_BRIDGE=1

exec "$BINARY" "$@"
