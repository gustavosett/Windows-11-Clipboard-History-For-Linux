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

# Install clipboard tools based on distro
install_clipboard_tools() {
    log "Installing clipboard tools (xclip, wl-clipboard)..."
    local installed=false
    case "$DISTRO" in
        ubuntu|debian|linuxmint|pop|kali|neon)
            sudo apt-get install -y xclip wl-clipboard 2>/dev/null && installed=true || true
            ;;
        fedora|rhel|centos|almalinux|rocky)
            sudo dnf install -y xclip wl-clipboard 2>/dev/null && installed=true || true
            ;;
        arch|manjaro|endeavouros)
            sudo pacman -S --needed --noconfirm xclip wl-clipboard 2>/dev/null && installed=true || true
            ;;
        opensuse*)
            sudo zypper install -y xclip wl-clipboard 2>/dev/null && installed=true || true
            ;;
        *)
            log "Please install xclip and wl-clipboard manually for GIF paste support"
            return
            ;;
    esac

    if [ "$installed" = true ]; then
        success "Clipboard tools installed"
    else
        log "Could not install clipboard tools automatically. Please install 'xclip' and 'wl-clipboard' manually."
    fi
}

case "$DISTRO" in
    ubuntu|debian|linuxmint|pop|kali|neon)
        FILE="win11-clipboard-history_${CLEAN_VERSION}_amd64.deb"
        URL="$BASE_URL/$FILE"
        CMD="sudo dpkg -i $FILE || sudo apt-get install -f -y"
        download_and_install "$URL" "$FILE" "$CMD"
        install_clipboard_tools
        ;;
    fedora|rhel|centos|almalinux|rocky)
        FILE="win11-clipboard-history-${CLEAN_VERSION}-1.x86_64.rpm"
        URL="$BASE_URL/$FILE"
        CMD="sudo rpm -i $FILE || sudo dnf install -y ./$FILE"
        download_and_install "$URL" "$FILE" "$CMD"
        install_clipboard_tools
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
        
        # Try to install clipboard tools for AppImage users
        install_clipboard_tools
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
    
    # Check if user already has input group membership (from previous install)
    if groups "$USER" | grep -q '\binput\b'; then
        log "User $USER is already in 'input' group"
        USER_ALREADY_IN_INPUT_GROUP=true
    else
        log "Adding $USER to 'input' group..."
        sudo usermod -aG input "$USER"
        echo -e "${GREEN}✓${NC} Added $USER to 'input' group"
        USER_ALREADY_IN_INPUT_GROUP=false
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

# Grant IMMEDIATE access using ACLs (no logout required!)
# This allows the user to start using the app right away
# The group membership ensures it works after reboot too
grant_immediate_access() {
    log "Granting immediate input device access (no logout needed)..."
    
    # Check if setfacl is available, install if not
    if ! command -v setfacl &> /dev/null; then
        log "Installing 'acl' package for immediate access..."
        case "$DISTRO" in
            ubuntu|debian|linuxmint|pop|kali|neon)
                sudo apt-get install -y acl 2>/dev/null || true
                ;;
            fedora|rhel|centos|almalinux|rocky)
                sudo dnf install -y acl 2>/dev/null || true
                ;;
            arch|manjaro|endeavouros)
                sudo pacman -S --needed --noconfirm acl 2>/dev/null || true
                ;;
            opensuse*)
                sudo zypper install -y acl 2>/dev/null || true
                ;;
        esac
    fi
    
    if command -v setfacl &> /dev/null; then
        # Grant ACL access to keyboard input devices
        for dev in /dev/input/event*; do
            if [ -e "$dev" ]; then
                sudo setfacl -m "u:${USER}:rw" "$dev" 2>/dev/null || true
            fi
        done
        
        # Grant ACL access to uinput
        if [ -e /dev/uinput ]; then
            sudo setfacl -m "u:${USER}:rw" /dev/uinput 2>/dev/null || true
        fi
        
        echo -e "${GREEN}✓${NC} Granted immediate access to input devices"
        return 0
    else
        log "Could not install 'acl' package for immediate access"
        return 1
    fi
}

# Initialize the variable before setup_input_permissions
USER_ALREADY_IN_INPUT_GROUP=false

setup_input_permissions

# Determine if logout is needed:
# - If user was already in input group, no logout needed
# - If ACLs were granted successfully, no logout needed
# - Otherwise, logout is needed for new group membership to take effect
NEEDS_LOGOUT=false
if [ "$USER_ALREADY_IN_INPUT_GROUP" = true ]; then
    # User already has permissions from previous session
    NEEDS_LOGOUT=false
elif grant_immediate_access; then
    # ACLs granted immediate access
    NEEDS_LOGOUT=false
else
    # Neither condition met, logout required
    NEEDS_LOGOUT=true
fi

# Setup autostart for the application (only if user agrees)
setup_autostart() {
    local autostart_dir="$HOME/.config/autostart"
    local desktop_file="$autostart_dir/win11-clipboard-history.desktop"
    
    # Check if autostart is already configured
    if [ -f "$desktop_file" ]; then
        echo -e "${GREEN}✓${NC} Autostart already configured"
        return
    fi
    
    # Ask user if they want to enable autostart
    echo ""
    echo -e "${BLUE}Would you like the app to start automatically on login? [Y/n]${NC}"
    read -r response
    
    case "$response" in
        [nN]|[nN][oO])
            log "Autostart skipped. You can enable it later in your desktop's Startup Applications."
            return
            ;;
    esac
    
    # Create autostart directory if it doesn't exist
    mkdir -p "$autostart_dir"
    
    # Create the autostart desktop entry
    cat > "$desktop_file" << 'EOF'
[Desktop Entry]
Name=Clipboard History
Comment=Windows 11-style Clipboard History Manager
GenericName=Clipboard Manager
Exec=win11-clipboard-history
Icon=win11-clipboard-history
Terminal=false
Type=Application
Categories=Utility;
Keywords=clipboard;history;paste;copy;
StartupWMClass=win11-clipboard-history
X-GNOME-Autostart-enabled=true
X-GNOME-Autostart-Delay=2
EOF
    
    echo -e "${GREEN}✓${NC} Autostart enabled - app will start automatically on login"
}

setup_autostart

# Launch the app if permissions are ready
launch_app() {
    # Only launch if we have permissions ready (no logout needed)
    if [ "$NEEDS_LOGOUT" = true ]; then
        return
    fi
    
    log "Starting Clipboard History..."
    
    # Launch the app in background
    if [ -n "$DISPLAY" ] || [ -n "$WAYLAND_DISPLAY" ]; then
        nohup win11-clipboard-history > /dev/null 2>&1 &
        sleep 1
        
        if pgrep -x "win11-clipboard-history" > /dev/null 2>&1; then
            echo -e "${GREEN}✓${NC} App started successfully"
        else
            log "Could not start app automatically. Please run 'win11-clipboard-history' manually."
        fi
    else
        log "No display detected. The app will start on next login."
    fi
}

launch_app

success "Installation completed successfully!"
echo ""

if [ "$NEEDS_LOGOUT" = true ]; then
    echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║                    IMPORTANT: Please log out                   ║${NC}"
    echo -e "${BLUE}║            and log back in for permissions to apply           ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "After logging back in, the app will start automatically."
else
    echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║            ✓ Ready to use! Press Super+V to open.             ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
fi

echo ""
echo "Tips:"
echo "  • The app runs in the system tray"
echo "  • Use Super+V or Ctrl+Alt+V to open clipboard history"
echo "  • Run 'win11-clipboard-history --version' to check version"
