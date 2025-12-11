#!/bin/bash

set -e

# Configuration
REPO_OWNER="gustavosett"
REPO_NAME="Windows-11-Clipboard-History-For-Linux"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Check for curl
command -v curl >/dev/null 2>&1 || error "curl is required but not installed."

# Get latest release tag
log "Fetching latest release version..."
LATEST_RELEASE_URL="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest"
RELEASE_TAG=$(curl -s "$LATEST_RELEASE_URL" | grep '"tag_name":' | head -n 1 | sed -E 's/.*"([^"]+)".*/\1/' | tr -cd '[:alnum:]._-')

if [ -z "$RELEASE_TAG" ]; then
    error "Failed to fetch latest version. Please check your internet connection."
fi

CLEAN_VERSION="${RELEASE_TAG#v}"
log "Latest version: $RELEASE_TAG"

# Detect Architecture
ARCH=$(uname -m)
if [ "$ARCH" != "x86_64" ]; then
    error "Unsupported architecture: $ARCH. Only x86_64 is supported."
fi

# Detect Distro
if [ -f /etc/os-release ]; then
    . /etc/os-release
    DISTRO=$ID
else
    DISTRO="unknown"
fi

# Prepare download
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"
trap 'rm -rf "$TEMP_DIR"' EXIT

download_and_install() {
    local url="$1"
    local filename="$2"
    local install_cmd="$3"

    log "Downloading $filename..."
    log "URL: $url"
    curl -L -o "$filename" "$url"
    
    if [ ! -f "$filename" ]; then
        error "Download failed."
    fi

    log "Installing..."
    eval "$install_cmd"
}

BASE_URL="https://github.com/$REPO_OWNER/$REPO_NAME/releases/download/$RELEASE_TAG"

case "$DISTRO" in
    ubuntu|debian|linuxmint|pop|kali|neon)
        FILE="win11-clipboard-history_${CLEAN_VERSION}_amd64.deb"
        URL="$BASE_URL/$FILE"
        CMD="sudo dpkg -i $FILE || sudo apt-get install -f -y"
        download_and_install "$URL" "$FILE" "$CMD"
        ;;
    fedora|rhel|centos|almalinux|rocky)
        FILE="win11-clipboard-history-${CLEAN_VERSION}-1.x86_64.rpm"
        URL="$BASE_URL/$FILE"
        CMD="sudo rpm -i $FILE || sudo dnf install -y ./$FILE"
        download_and_install "$URL" "$FILE" "$CMD"
        ;;
    *)
        log "Distribution '$DISTRO' not officially supported for native package. Installing AppImage..."
        FILE="win11-clipboard-history_${CLEAN_VERSION}_amd64.AppImage"
        URL="$BASE_URL/$FILE"
        
        log "Downloading $FILE..."
        curl -L -o "$FILE" "$URL"
        chmod +x "$FILE"
        
        INSTALL_DIR="$HOME/.local/bin"
        LIB_DIR="$HOME/.local/lib/win11-clipboard-history"
        mkdir -p "$INSTALL_DIR" "$LIB_DIR"
        
        # Move AppImage to lib directory
        mv "$FILE" "$LIB_DIR/win11-clipboard-history.AppImage"
        
        # Create wrapper script to handle Snap environment conflicts
        cat > "$INSTALL_DIR/win11-clipboard-history" << 'WRAPPER'
#!/bin/bash
# Wrapper script for win11-clipboard-history AppImage
# Cleans environment to avoid Snap/GTK library conflicts
# Forces X11/XWayland for better window positioning support

APPIMAGE="$HOME/.local/lib/win11-clipboard-history/win11-clipboard-history.AppImage"

# Always use clean environment to avoid library conflicts
# GDK_BACKEND=x11 forces XWayland on Wayland sessions for window positioning
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
    PATH="/usr/local/bin:/usr/bin:/bin:$HOME/.local/bin" \
    LANG="${LANG:-en_US.UTF-8}" \
    GDK_BACKEND="x11" \
    "$APPIMAGE" "$@"
WRAPPER
        chmod +x "$INSTALL_DIR/win11-clipboard-history"
        
        success "AppImage installed to $LIB_DIR"
        success "Wrapper script created at $INSTALL_DIR/win11-clipboard-history"
        echo "Please ensure $INSTALL_DIR is in your PATH."
        ;;
esac

# Setup input group permissions for global hotkeys and keyboard simulation
setup_input_permissions() {
    log "Setting up input device permissions for global hotkeys..."
    
    # Check if input group exists
    if ! getent group input > /dev/null 2>&1; then
        log "Creating 'input' group..."
        sudo groupadd input
    fi
    
    # Add current user to input group
    if ! groups "$USER" | grep -q '\binput\b'; then
        log "Adding $USER to 'input' group..."
        sudo usermod -aG input "$USER"
        echo -e "${GREEN}✓${NC} Added $USER to 'input' group"
    else
        log "User $USER is already in 'input' group"
    fi
    
    # Create comprehensive udev rules for input devices and uinput
    UDEV_RULE="/etc/udev/rules.d/99-win11-clipboard-input.rules"
    log "Creating udev rules for input devices..."
    sudo tee "$UDEV_RULE" > /dev/null << 'EOF'
# udev rules for Windows 11 Clipboard History
# Input devices (keyboards) - needed for evdev global hotkey detection
KERNEL=="event*", SUBSYSTEM=="input", MODE="0660", GROUP="input"
# uinput device - needed for kernel-level keyboard simulation (paste injection)
KERNEL=="uinput", SUBSYSTEM=="misc", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput"
EOF
    echo -e "${GREEN}✓${NC} Created udev rules for input devices"
    
    # Load uinput module if not loaded
    if ! lsmod | grep -q uinput; then
        log "Loading uinput kernel module..."
        sudo modprobe uinput 2>/dev/null || true
    fi
    
    # Ensure uinput is loaded on boot
    if [ ! -f /etc/modules-load.d/uinput.conf ]; then
        log "Configuring uinput module to load on boot..."
        echo "uinput" | sudo tee /etc/modules-load.d/uinput.conf > /dev/null
        echo -e "${GREEN}✓${NC} Configured uinput module to load on boot"
    fi
    
    # Reload udev rules and trigger for misc subsystem (for uinput)
    sudo udevadm control --reload-rules
    sudo udevadm trigger
    sudo udevadm trigger --subsystem-match=misc --action=change
}

setup_input_permissions

success "Installation completed successfully!"
echo ""
echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                    IMPORTANT: Please log out                   ║${NC}"
echo -e "${BLUE}║            and log back in for permissions to apply           ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "After logging back in:"
echo "  • Start the app with: win11-clipboard-history"
echo "  • Use keyboard shortcut: Super+V or Ctrl+Alt+V"
echo "  • Check version with: win11-clipboard-history --version"
