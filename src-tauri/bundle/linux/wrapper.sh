#!/bin/bash
# Wrapper script for win11-clipboard-history
# Forces X11/XWayland for better window positioning support

# Binary location
BINARY="/usr/lib/win11-clipboard-history/win11-clipboard-history-bin"

# Set GDK_BACKEND=x11 for window positioning support, but inherit full environment
# to avoid issues with DBus, display variables, etc.
# Wayland restricts cursor_position() and set_position() for security
# XWayland allows these operations while still running on Wayland session
export GDK_BACKEND="x11"
exec "$BINARY" "$@"
