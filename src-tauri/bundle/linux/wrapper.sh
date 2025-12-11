#!/bin/bash
# Wrapper script for win11-clipboard-history
# Cleans environment to avoid Snap library conflicts

# Binary location
BINARY="/usr/lib/win11-clipboard-history/win11-clipboard-history-bin"

# If running from a Snap-polluted environment, clean it
if [[ -n "$SNAP" ]] || [[ "$LD_LIBRARY_PATH" == */snap/* ]]; then
    # Run with clean environment, preserving only essential variables
    exec env -i \
        HOME="$HOME" \
        USER="$USER" \
        DISPLAY="${DISPLAY:-:0}" \
        XAUTHORITY="$XAUTHORITY" \
        WAYLAND_DISPLAY="$WAYLAND_DISPLAY" \
        XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR" \
        XDG_SESSION_TYPE="$XDG_SESSION_TYPE" \
        XDG_CURRENT_DESKTOP="$XDG_CURRENT_DESKTOP" \
        DBUS_SESSION_BUS_ADDRESS="$DBUS_SESSION_BUS_ADDRESS" \
        PATH="/usr/local/bin:/usr/bin:/bin" \
        LANG="${LANG:-en_US.UTF-8}" \
        "$BINARY" "$@"
else
    exec "$BINARY" "$@"
fi
