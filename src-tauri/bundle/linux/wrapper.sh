#!/bin/bash
# Wrapper script for win11-clipboard-history
# Cleans environment to avoid Snap library conflicts

# Binary location
BINARY="/usr/lib/win11-clipboard-history/win11-clipboard-history-bin"

# Force X11/XWayland for better window positioning support
# Wayland restricts cursor_position() and set_position() for security
# XWayland allows these operations while still running on Wayland session
WEBKIT_DISABLE_COMPOSITING_MODE=1

# Always use clean environment to avoid library conflicts from Snap, Flatpak, etc.
exec env -i \
    HOME="$HOME" \
    USER="$USER" \
    SHELL="$SHELL" \
    TERM="$TERM" \
    DISPLAY="${DISPLAY:-:0}" \
    XAUTHORITY="$XAUTHORITY" \
    WAYLAND_DISPLAY="$WAYLAND_DISPLAY" \
    XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR" \
    XDG_SESSION_TYPE="$XDG_SESSION_TYPE" \
    XDG_CURRENT_DESKTOP="$XDG_CURRENT_DESKTOP" \
    XDG_SESSION_CLASS="$XDG_SESSION_CLASS" \
    DBUS_SESSION_BUS_ADDRESS="$DBUS_SESSION_BUS_ADDRESS" \
    PATH="/usr/local/bin:/usr/bin:/bin" \
    LANG="${LANG:-en_US.UTF-8}" \
    LC_ALL="${LC_ALL:-}" \
    GDK_BACKEND="x11" \
    "$BINARY" "$@"
