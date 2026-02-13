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

sanitize_runtime_env() {
    # Snap/Flatpak sandboxes may inject GTK/GIO/runtime paths from confined runtimes.
    # Clear them so the app uses host system libraries consistently.
    unset LD_LIBRARY_PATH
    unset LD_PRELOAD
    unset GTK_PATH
    unset GIO_MODULE_DIR
    unset GTK_IM_MODULE_FILE
    unset GTK_EXE_PREFIX
    unset LOCPATH
    unset GSETTINGS_SCHEMA_DIR

    # Keep user/system XDG_DATA_DIRS when valid.
    # Only merge in defaults if empty or clearly snap-injected.
    local xdg_data_dirs="${XDG_DATA_DIRS:-}"
    local system_dirs=("/usr/local/share" "/usr/share" "/var/lib/snapd/desktop")

    if [ -z "$xdg_data_dirs" ]; then
        xdg_data_dirs="${system_dirs[0]}:${system_dirs[1]}:${system_dirs[2]}"
    elif [[ "$xdg_data_dirs" == *"/snap/"* || "$xdg_data_dirs" == *"snap/code"* || -n "${SNAP:-}" ]]; then
        local dir
        for dir in "${system_dirs[@]}"; do
            case ":$xdg_data_dirs:" in
                *":$dir:"*) ;;
                *) xdg_data_dirs="$xdg_data_dirs:$dir" ;;
            esac
        done
    fi

    export XDG_DATA_DIRS="$xdg_data_dirs"
}

sanitize_runtime_env

export GDK_SCALE="${GDK_SCALE:-1}"
export GDK_DPI_SCALE="${GDK_DPI_SCALE:-1}"

export TAURI_TRAY="${TAURI_TRAY:-libayatana-appindicator3}"

# Disable AT-SPI to prevent accessibility bus warnings/delays
export NO_AT_BRIDGE=1

# Force software rendering in virtualized environments to avoid GPU issues
if systemd-detect-virt -q; then
    export LIBGL_ALWAYS_SOFTWARE=1
fi

exec "$BINARY" "$@"
