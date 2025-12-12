#!/bin/bash
# Post-installation script for Windows 11 Clipboard History

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m'

echo -e "${BLUE}Setting up Windows 11 Clipboard History...${NC}"

# Install clipboard tools for GIF paste support
install_clipboard_tools() {
    echo -e "${BLUE}Installing clipboard tools for GIF support...${NC}"
    local installed=false

    if command -v apt-get &> /dev/null; then
        apt-get install -y xclip wl-clipboard 2>/dev/null && installed=true || true
    elif command -v dnf &> /dev/null; then
        dnf install -y xclip wl-clipboard 2>/dev/null && installed=true || true
    elif command -v pacman &> /dev/null; then
        pacman -S --needed --noconfirm xclip wl-clipboard 2>/dev/null && installed=true || true
    elif command -v zypper &> /dev/null; then
        zypper install -y xclip wl-clipboard 2>/dev/null && installed=true || true
    fi

    if [ "$installed" = true ]; then
        echo -e "${GREEN}✓${NC} Clipboard tools installed"
    else
        echo -e "${YELLOW}!${NC} Could not install clipboard tools automatically. Please install 'xclip' and 'wl-clipboard' manually."
    fi
}

install_clipboard_tools

# Create wrapper script to handle Snap environment conflicts
BINARY_PATH="/usr/bin/win11-clipboard-history"
LIB_DIR="/usr/lib/win11-clipboard-history"

if [ -f "$BINARY_PATH" ] && [ ! -L "$BINARY_PATH" ]; then
    # Move binary to lib directory and create wrapper
    mkdir -p "$LIB_DIR"
    mv "$BINARY_PATH" "$LIB_DIR/win11-clipboard-history-bin"
    
    # Create wrapper script
    cat > "$BINARY_PATH" << 'WRAPPER'
#!/bin/bash
# Wrapper script for win11-clipboard-history
# Cleans environment to avoid Snap library conflicts
# Forces X11/XWayland for better window positioning support

BINARY="/usr/lib/win11-clipboard-history/win11-clipboard-history-bin"

# Always use clean environment to avoid library conflicts
# GDK_BACKEND=x11 forces XWayland on Wayland sessions for window positioning
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
WRAPPER
    chmod +x "$BINARY_PATH"
    echo -e "${GREEN}✓${NC} Created wrapper script for Snap compatibility"
fi

# Ensure input group exists
if ! getent group input > /dev/null 2>&1; then
    echo -e "${BLUE}Creating 'input' group...${NC}"
    groupadd input
fi

# Create udev rules for input devices and uinput
UDEV_RULE="/etc/udev/rules.d/99-win11-clipboard-input.rules"
cat > "$UDEV_RULE" << 'EOF'
# udev rules for Windows 11 Clipboard History
# Input devices (keyboards) - needed for evdev global hotkey detection
KERNEL=="event*", SUBSYSTEM=="input", MODE="0660", GROUP="input"
# uinput device - needed for kernel-level keyboard simulation (paste injection)
KERNEL=="uinput", SUBSYSTEM=="misc", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput"
EOF
echo -e "${GREEN}✓${NC} Created udev rules for input devices"

# Load uinput module if not loaded
if ! lsmod | grep -q uinput; then
    modprobe uinput 2>/dev/null || true
fi

# Ensure uinput is loaded on boot
if [ ! -f /etc/modules-load.d/uinput.conf ]; then
    echo "uinput" > /etc/modules-load.d/uinput.conf
    echo -e "${GREEN}✓${NC} Configured uinput module to load on boot"
fi

# Reload udev rules and trigger for misc subsystem (for uinput)
udevadm control --reload-rules 2>/dev/null || true
udevadm trigger 2>/dev/null || true
udevadm trigger --subsystem-match=misc --action=change 2>/dev/null || true

# Get the actual user (not root when using sudo)
ACTUAL_USER="${SUDO_USER:-$USER}"

# Check if user already has input group membership (from previous install or manual setup)
USER_ALREADY_IN_INPUT_GROUP=false
if [ -n "$ACTUAL_USER" ] && [ "$ACTUAL_USER" != "root" ]; then
    if groups "$ACTUAL_USER" 2>/dev/null | grep -q '\binput\b'; then
        USER_ALREADY_IN_INPUT_GROUP=true
        echo -e "${GREEN}✓${NC} User $ACTUAL_USER is already in 'input' group"
    else
        usermod -aG input "$ACTUAL_USER"
        echo -e "${GREEN}✓${NC} Added $ACTUAL_USER to 'input' group"
    fi
fi

# Grant IMMEDIATE access using ACLs (no logout required!)
# This allows the user to start using the app right away
# The group membership above ensures it works after reboot too
grant_immediate_access() {
    local user="$1"
    
    if [ -z "$user" ] || [ "$user" = "root" ]; then
        return
    fi
    
    # Check if setfacl is available
    if ! command -v setfacl &> /dev/null; then
        echo -e "${YELLOW}!${NC} 'acl' package not installed. Installing..."
        if command -v apt-get &> /dev/null; then
            apt-get install -y acl 2>/dev/null || true
        elif command -v dnf &> /dev/null; then
            dnf install -y acl 2>/dev/null || true
        elif command -v pacman &> /dev/null; then
            pacman -S --needed --noconfirm acl 2>/dev/null || true
        elif command -v zypper &> /dev/null; then
            zypper install -y acl 2>/dev/null || true
        fi
    fi
    
    if command -v setfacl &> /dev/null; then
        echo -e "${BLUE}Granting immediate input device access (no logout needed)...${NC}"
        
        # Grant ACL access to keyboard input devices
        for dev in /dev/input/event*; do
            if [ -e "$dev" ]; then
                setfacl -m "u:${user}:rw" "$dev" 2>/dev/null || true
            fi
        done
        
        # Grant ACL access to uinput
        if [ -e /dev/uinput ]; then
            setfacl -m "u:${user}:rw" /dev/uinput 2>/dev/null || true
        fi
        
        echo -e "${GREEN}✓${NC} Granted immediate access to input devices"
        return 0
    else
        echo -e "${YELLOW}!${NC} Could not install 'acl' package for immediate access"
        return 1
    fi
}

# Determine if logout is needed:
# - If user was already in input group, no logout needed
# - If ACLs were granted successfully, no logout needed
# - Otherwise, logout is needed for new group membership to take effect
NEEDS_LOGOUT=false
if [ "$USER_ALREADY_IN_INPUT_GROUP" = true ]; then
    # User already has permissions from previous session
    NEEDS_LOGOUT=false
elif grant_immediate_access "$ACTUAL_USER"; then
    # ACLs granted immediate access
    NEEDS_LOGOUT=false
else
    # Neither condition met, logout required
    NEEDS_LOGOUT=true
fi

# Setup autostart for the application (only if user agrees)
setup_autostart() {
    local user="$1"
    
    if [ -z "$user" ] || [ "$user" = "root" ]; then
        return
    fi
    
    # Get user's home directory
    local user_home
    user_home=$(getent passwd "$user" | cut -d: -f6)
    
    if [ -z "$user_home" ]; then
        return
    fi
    
    local autostart_dir="$user_home/.config/autostart"
    local desktop_file="$autostart_dir/win11-clipboard-history.desktop"
    
    # Check if autostart is already configured
    if [ -f "$desktop_file" ]; then
        echo -e "${GREEN}✓${NC} Autostart already configured"
        return
    fi
    
    # Ask user if they want to enable autostart
    echo ""
    echo -e "${BLUE}Would you like the app to start automatically on login? [Y/n]${NC}"
    
    # Read user input with a timeout (default to Yes if no response or non-interactive)
    local response="y"
    if [ -t 0 ]; then
        read -r -t 30 response 2>/dev/null || response="y"
    fi
    
    case "$response" in
        [nN]|[nN][oO])
            echo -e "${YELLOW}!${NC} Autostart skipped. You can enable it later in your desktop's Startup Applications."
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
    
    # Fix ownership to the actual user
    chown "$user:$user" "$autostart_dir" 2>/dev/null || true
    chown "$user:$user" "$desktop_file" 2>/dev/null || true
    
    echo -e "${GREEN}✓${NC} Autostart enabled - app will start automatically on login"
}

setup_autostart "$ACTUAL_USER"

# Launch the app for the user (in background, as the actual user)
launch_app() {
    local user="$1"
    
    if [ -z "$user" ] || [ "$user" = "root" ]; then
        return
    fi
    
    # Only launch if we have permissions ready (no logout needed)
    if [ "$NEEDS_LOGOUT" = true ]; then
        return
    fi
    
    # Get user's home directory and display info
    local user_home
    user_home=$(getent passwd "$user" | cut -d: -f6)
    
    if [ -z "$user_home" ]; then
        return
    fi
    
    echo -e "${BLUE}Starting Clipboard History...${NC}"
    
    # Launch the app as the actual user with proper environment
    if [ -n "$DISPLAY" ] || [ -n "$WAYLAND_DISPLAY" ]; then
        # We have a display, try to launch
        su - "$user" -c "nohup win11-clipboard-history > /dev/null 2>&1 &" 2>/dev/null || true
        sleep 1
        
        if pgrep -u "$user" -x "win11-clipboard-history" > /dev/null 2>&1; then
            echo -e "${GREEN}✓${NC} App started successfully"
        else
            echo -e "${YELLOW}!${NC} Could not start app automatically. Please run 'win11-clipboard-history' manually."
        fi
    else
        echo -e "${YELLOW}!${NC} No display detected. The app will start on next login."
    fi
}

launch_app "$ACTUAL_USER"

echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║          Windows 11 Clipboard History installed!              ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

if [ "$NEEDS_LOGOUT" = true ]; then
    echo -e "${YELLOW}IMPORTANT: Please log out and log back in for permissions to apply.${NC}"
    echo ""
    echo "After logging back in, the app will start automatically."
else
    echo -e "${GREEN}✓ Ready to use! Press Super+V or Ctrl+Alt+V to open clipboard history.${NC}"
fi

echo ""
echo "Tips:"
echo "  • The app runs in the system tray"
echo "  • Use Super+V or Ctrl+Alt+V to open clipboard history"
echo "  • Run 'win11-clipboard-history --version' to check version"
echo ""

exit 0
