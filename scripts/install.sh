#!/bin/bash

# Windows 11 Clipboard History Installer
# Author: Gustavo Sett
# License: MIT

set -e

# --- Configuration ---
REPO_OWNER="gustavosett"
REPO_NAME="Windows-11-Clipboard-History-For-Linux"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# --- 1. Cleanup & Prep ---

log "Preparing installation..."

# Kill existing instances to prevent conflicts during update
pkill -f "win11-clipboard-history" || true
sleep 1

command -v curl >/dev/null 2>&1 || error "curl is required."

# Detect Distro
if [ -f /etc/os-release ]; then
    . /etc/os-release
    DISTRO=$ID
else
    DISTRO="unknown"
fi

# Fetch Version
log "Fetching latest version info..."
LATEST_RELEASE_URL="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest"
RELEASE_TAG=$(curl -s "$LATEST_RELEASE_URL" | grep '"tag_name":' | head -n 1 | sed -E 's/.*"([^"]+)".*/\1/' | tr -cd '[:alnum:]._-')
[ -z "$RELEASE_TAG" ] && error "Failed to fetch version."
CLEAN_VERSION="${RELEASE_TAG#v}"

# Setup Temp Directory
# We explicitly set 755 permissions so the '_apt' user can read files inside it
TEMP_DIR=$(mktemp -d)
chmod 755 "$TEMP_DIR"
cd "$TEMP_DIR"
trap 'rm -rf "$TEMP_DIR"' EXIT

BASE_URL="https://github.com/$REPO_OWNER/$REPO_NAME/releases/download/$RELEASE_TAG"

# --- 2. Dependencies ---

install_deps() {
    log "Installing dependencies..."
    case "$DISTRO" in
        ubuntu|debian|linuxmint|pop|kali|neon)
            sudo apt-get update -qq
            sudo apt-get install -y xclip wl-clipboard acl
            ;;
        fedora|rhel|centos|almalinux|rocky)
            sudo dnf install -y xclip wl-clipboard acl
            ;;
        arch|manjaro|endeavouros)
            command -v pacman >/dev/null && sudo pacman -S --needed --noconfirm xclip wl-clipboard acl
            ;;
        opensuse*)
            sudo zypper install -y xclip wl-clipboard acl
            ;;
    esac
}

download_file() {
    log "Downloading $2..."
    curl -L -o "$2" "$1" --progress-bar
    # Fix permissions so apt/dnf can read the file without warnings
    chmod 644 "$2"
}

# --- 3. Installation ---

case "$DISTRO" in
    ubuntu|debian|linuxmint|pop|kali|neon)
        FILE="win11-clipboard-history_${CLEAN_VERSION}_amd64.deb"
        download_file "$BASE_URL/$FILE" "$FILE"
        install_deps
        
        log "Installing .deb package..."
        # 'yes' handles the prompt. 2>/dev/null hides the apt "download is unsandboxed" warning if it still appears
        yes | sudo apt-get install -y "./$FILE"
        ;;
        
    fedora|rhel|centos|almalinux|rocky)
        FILE="win11-clipboard-history-${CLEAN_VERSION}-1.x86_64.rpm"
        download_file "$BASE_URL/$FILE" "$FILE"
        install_deps
        sudo dnf install -y "./$FILE"
        ;;
        
    *)
        log "Installing AppImage..."
        FILE="win11-clipboard-history_${CLEAN_VERSION}_amd64.AppImage"
        download_file "$BASE_URL/$FILE" "$FILE"
        chmod +x "$FILE"
        install_deps
        
        mkdir -p "$HOME/.local/bin" "$HOME/.local/lib/win11-clipboard-history"
        mv "$FILE" "$HOME/.local/lib/win11-clipboard-history/win11-clipboard-history.AppImage"
        
        # Create Wrapper
        cat > "$HOME/.local/bin/win11-clipboard-history" << 'WRAPPER'
#!/bin/bash
exec env -i HOME="$HOME" USER="$USER" DISPLAY="${DISPLAY:-:0}" \
    WAYLAND_DISPLAY="$WAYLAND_DISPLAY" XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR" \
    GDK_BACKEND="x11" \
    "$HOME/.local/lib/win11-clipboard-history/win11-clipboard-history.AppImage" "$@"
WRAPPER
        chmod +x "$HOME/.local/bin/win11-clipboard-history"
        
        # Manual Permission Setup for AppImage
        if ! [ -f /etc/udev/rules.d/99-win11-clipboard-input.rules ]; then
            log "Configuring input permissions (with uaccess support)..."
            getent group input >/dev/null || sudo groupadd input
            groups "$USER" | grep -q input || sudo usermod -aG input "$USER"
            cat << 'EOF' | sudo tee /etc/udev/rules.d/99-win11-clipboard-input.rules > /dev/null
# udev rules for Windows 11 Clipboard History
# TAG+="uaccess" grants access to the active session user via systemd-logind
# GROUP="input" serves as fallback for non-systemd systems
KERNEL=="event*", SUBSYSTEM=="input", MODE="0660", GROUP="input", TAG+="uaccess"
KERNEL=="uinput", SUBSYSTEM=="misc", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput", TAG+="uaccess"
EOF
            sudo modprobe uinput 2>/dev/null || true
            sudo udevadm control --reload-rules && sudo udevadm trigger
        fi
        ;;
esac

# --- 4. Final Permissions ---

# Check if systemd-logind uaccess is working, otherwise use ACL fallback
if loginctl show-session $(loginctl --no-legend 2>/dev/null | grep "$USER" | head -1 | awk '{print $1}') -p Active 2>/dev/null | grep -q "Active=yes"; then
    log "Using systemd-logind uaccess for automatic permission management"
    # Trigger udev to apply uaccess tags
    sudo udevadm trigger --subsystem-match=input --action=change 2>/dev/null || true
    sudo udevadm trigger --subsystem-match=misc --action=change 2>/dev/null || true
else
    log "Applying ACL fallback for immediate access..."
    # ACL fallback for non-systemd systems or when uaccess isn't working
    if command -v setfacl &>/dev/null; then
        for dev in /dev/input/event*; do
            [ -e "$dev" ] && sudo setfacl -m "u:${USER}:rw" "$dev" 2>/dev/null || true
        done
        [ -e /dev/uinput ] && sudo setfacl -m "u:${USER}:rw" /dev/uinput 2>/dev/null || true
    fi
fi

# --- 5. Launch ---

log "Starting application..."

# Double check cleanup
pkill -f "win11-clipboard-history" || true
sleep 1

# Launch detached from terminal to prevent freezing
# Redirecting input/output to /dev/null is critical here
nohup win11-clipboard-history >/dev/null 2>&1 < /dev/null & disown

sleep 2

if pgrep -f "win11-clipboard-history" > /dev/null; then
    success "App is running! Press Super+V to open."
else
    # Fallback check: sometimes pgrep fails on specific distros immediately
    warn "App installed. If it didn't open, run 'win11-clipboard-history' or find it in your menu."
fi
