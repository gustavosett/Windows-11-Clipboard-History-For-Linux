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
# Fast path: forward to the running instance over D-Bus
# ---------------------------------------------------------------------------
# Global shortcuts launch this wrapper on every key press. Starting the full
# binary just to hand the request to the running instance (via the
# single-instance plugin) costs a few hundred milliseconds, plus the GPU
# detection below. Talk to the running instance directly instead.
#
# On Wayland the compositor passes an xdg-activation token in
# XDG_ACTIVATION_TOKEN; forward it so the running instance may take focus.
FORWARD_ARGS=("$@")
if [[ -n "${XDG_ACTIVATION_TOKEN:-}" ]]; then
    FORWARD_ARGS+=("--activation-token=${XDG_ACTIVATION_TOKEN}")
fi

SINGLE_INSTANCE_DEST="dev.gustavosett.clipboard-history.SingleInstance"
SINGLE_INSTANCE_PATH="/dev/gustavosett/clipboard_history/SingleInstance"

forward_to_running_instance() {
    # Only plain toggles are forwarded; anything else (--help, --version, ...)
    # must run the binary itself.
    local arg
    for arg in "$@"; do
        case "$arg" in
            --emoji|--settings) ;;
            *) return 1 ;;
        esac
    done

    local argv=("win11-clipboard-history" "${FORWARD_ARGS[@]}")

    if command -v busctl &>/dev/null; then
        busctl --user call "$SINGLE_INSTANCE_DEST" "$SINGLE_INSTANCE_PATH" \
            org.SingleInstance.DBus ExecuteCallback ass \
            "${#argv[@]}" "${argv[@]}" "$PWD" &>/dev/null && return 0
    fi

    if command -v gdbus &>/dev/null; then
        local list="" item
        for item in "${argv[@]}"; do
            item=${item//\\/\\\\}
            item=${item//\'/\\\'}
            list+="${list:+, }'${item}'"
        done
        local cwd=${PWD//\\/\\\\}
        cwd=${cwd//\'/\\\'}
        gdbus call --session --dest "$SINGLE_INSTANCE_DEST" \
            --object-path "$SINGLE_INSTANCE_PATH" \
            --method org.SingleInstance.DBus.ExecuteCallback \
            "[${list}]" "'${cwd}'" &>/dev/null && return 0
    fi

    return 1
}

if forward_to_running_instance "$@"; then
    exit 0
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
# NVIDIA GPU detection
# ---------------------------------------------------------------------------
# NVIDIA GPUs with proprietary drivers have known issues with WebKit's
# DMA-BUF renderer causing opacity/transparency rendering artifacts.
# Detect via lspci or the nvidia kernel module presence.
detect_nvidia() {
    # Check if user has already forced the flag
    if [[ -n "${IS_NVIDIA:-}" ]]; then
        return 0
    fi

    # Method 1: Check for loaded nvidia kernel module
    if lsmod 2>/dev/null | grep -qi '^nvidia'; then
        export IS_NVIDIA=1
        return 0
    fi

    # Method 2: Check lspci for NVIDIA VGA controller
    if command -v lspci &>/dev/null && lspci 2>/dev/null | grep -qi 'vga.*nvidia'; then
        export IS_NVIDIA=1
        return 0
    fi

    return 1
}
detect_nvidia || true

# ---------------------------------------------------------------------------
# AppImage detection
# ---------------------------------------------------------------------------
# AppImage bundles may ship incompatible Mesa/GL libraries that conflict
# with the host GPU driver, causing similar opacity rendering issues.
if [[ -n "${APPIMAGE:-}" ]]; then
    export IS_APPIMAGE=1
fi

# ---------------------------------------------------------------------------
# WebKit DMA-BUF workaround for NVIDIA / AppImage
# ---------------------------------------------------------------------------
# When running on NVIDIA or inside an AppImage, disable the DMA-BUF
# renderer in WebKitGTK to prevent opacity/transparency glitches.
if [[ "${IS_NVIDIA:-}" == "1" || "${IS_APPIMAGE:-}" == "1" ]]; then
    echo "Info: Disabling WebKit DMA-BUF renderer due to NVIDIA GPU or AppImage environment."
    export WEBKIT_DISABLE_DMABUF_RENDERER=1
fi

# ---------------------------------------------------------------------------
# Display & rendering defaults
# ---------------------------------------------------------------------------

export TAURI_TRAY="${TAURI_TRAY:-libayatana-appindicator3}"

# Disable AT-SPI to prevent accessibility bus warnings/delays
export NO_AT_BRIDGE=1

# Force software rendering in virtualized environments to avoid GPU issues
if systemd-detect-virt -q 2>/dev/null; then
    export LIBGL_ALWAYS_SOFTWARE=1
fi

exec "$BINARY" "${FORWARD_ARGS[@]}"
