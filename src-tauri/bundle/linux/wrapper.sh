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

# ---------------------------------------------------------------------------
# Runtime environment sanitization
# ---------------------------------------------------------------------------
# When launched from a Snap terminal (e.g. VS Code Snap) or a Flatpak host,
# the parent process may inject library/schema paths from its confined runtime
# into child processes. These paths point to sandbox-internal libraries that
# are incompatible with the host GTK/WebKit stack this app links against,
# causing crashes, missing schemas, or wrong icon themes.
# ---------------------------------------------------------------------------

# Always clear library/runtime overrides — they must never leak from a sandbox
# into the host-linked Tauri binary.
unset LD_LIBRARY_PATH
unset LD_PRELOAD
unset GTK_PATH
unset GIO_MODULE_DIR
unset GTK_IM_MODULE_FILE
unset GTK_EXE_PREFIX
unset LOCPATH
unset GSETTINGS_SCHEMA_DIR

# Fix XDG_DATA_DIRS only when contaminated by sandbox paths.
# Snap terminals inject entries like /snap/code/*/usr/share which cause the
# app to resolve wrong GSettings schemas, icons, or .desktop files.
# When contaminated we strip sandbox entries and place system dirs first
# (matching run-dev.sh) so host resources always win.
sanitize_xdg_data_dirs() {
    local xdg="${XDG_DATA_DIRS:-}"
    local system_dirs="/usr/local/share:/usr/share:/var/lib/snapd/desktop"

    # Detect contamination: $SNAP/$FLATPAK_ID set, or paths contain sandbox dirs
    if [[ -z "${SNAP:-}" && -z "${FLATPAK_ID:-}" && "$xdg" != *"/snap/"* && "$xdg" != *"/flatpak/"* ]]; then
        return  # Environment is clean — leave XDG_DATA_DIRS untouched
    fi

    # Rebuild: keep only non-sandbox entries that aren't already in system_dirs
    local cleaned=""
    local entry
    IFS=':' read -ra entries <<< "$xdg"
    for entry in "${entries[@]}"; do
        # Skip sandbox-injected paths
        case "$entry" in
            */snap/*|*/flatpak/*) continue ;;
        esac
        # Skip if already covered by system_dirs (avoid duplicates)
        case ":$system_dirs:" in
            *":$entry:"*) continue ;;
        esac
        cleaned="${cleaned:+$cleaned:}$entry"
    done

    # System dirs first (highest precedence), then remaining clean dirs
    export XDG_DATA_DIRS="${system_dirs}${cleaned:+:$cleaned}"
}
sanitize_xdg_data_dirs

# ---------------------------------------------------------------------------
# Display & rendering defaults
# ---------------------------------------------------------------------------
export GDK_SCALE="${GDK_SCALE:-1}"
export GDK_DPI_SCALE="${GDK_DPI_SCALE:-1}"

export TAURI_TRAY="${TAURI_TRAY:-libayatana-appindicator3}"

# Disable AT-SPI to prevent accessibility bus warnings/delays
export NO_AT_BRIDGE=1

# Force software rendering in virtualized environments to avoid GPU issues
if systemd-detect-virt -q 2>/dev/null; then
    export LIBGL_ALWAYS_SOFTWARE=1
fi

exec "$BINARY" "$@"
