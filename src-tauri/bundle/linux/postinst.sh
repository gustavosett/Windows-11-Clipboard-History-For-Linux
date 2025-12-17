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

# Setup Permissions for uinput (needed for paste simulation)
setup_uinput_permissions() {
    echo -e "${BLUE}Configuring permissions for paste simulation...${NC}"
    
    # Create input group if missing
    getent group input >/dev/null || groupadd input
    
    # Create udev rule for uinput (needed for paste simulation)
    UDEV_RULE="/etc/udev/rules.d/99-win11-clipboard-input.rules"
    cat > "$UDEV_RULE" << 'EOF'
# udev rules for Windows 11 Clipboard History
# uinput device - needed for kernel-level keyboard simulation (paste injection)
# TAG+="uaccess" grants access to the active session user via systemd-logind
# GROUP="input" serves as fallback for non-systemd systems
KERNEL=="uinput", SUBSYSTEM=="misc", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput", TAG+="uaccess"
EOF
    echo -e "${GREEN}✓${NC} Created udev rules for uinput (with uaccess support)"
    
    # Load uinput module if not loaded
    if ! lsmod | grep -q uinput; then
        modprobe uinput 2>/dev/null || true
    fi
    
    # Ensure uinput is loaded on boot
    if [ ! -f /etc/modules-load.d/uinput.conf ]; then
        echo "uinput" > /etc/modules-load.d/uinput.conf
        echo -e "${GREEN}✓${NC} Configured uinput module to load on boot"
    fi
    
    # Reload udev rules
    udevadm control --reload-rules 2>/dev/null || true
    udevadm trigger --subsystem-match=misc --action=change 2>/dev/null || true
    
    # Add user to input group for uinput access
    REAL_USER="${SUDO_USER:-$USER}"
    if [ -n "$REAL_USER" ] && [ "$REAL_USER" != "root" ]; then
        if ! id -nG "$REAL_USER" | grep -qw input; then
            usermod -aG input "$REAL_USER"
            echo -e "${GREEN}✓${NC} User '$REAL_USER' added to 'input' group for paste simulation"
        else
            echo -e "${GREEN}✓${NC} User '$REAL_USER' is already in 'input' group"
        fi
    fi
}

setup_uinput_permissions

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

# Get the actual user (not root when using sudo)
ACTUAL_USER="${SUDO_USER:-$USER}"

# Grant immediate access to uinput using ACLs (no logout required for paste)
grant_uinput_access() {
    local user="$1"
    
    if [ -z "$user" ] || [ "$user" = "root" ]; then
        return 1
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
        # Grant ACL access to uinput only (for paste simulation)
        if [ -e /dev/uinput ]; then
            setfacl -m "u:${user}:rw" /dev/uinput 2>/dev/null || true
            echo -e "${GREEN}✓${NC} Granted immediate access to uinput for paste simulation"
        fi
        return 0
    else
        echo -e "${YELLOW}!${NC} Could not install 'acl' package for immediate access"
        return 1
    fi
}

# ACL fallback: Only needed for non-systemd systems or if uaccess doesn't work
# On systemd systems with TAG+="uaccess", permissions are granted automatically
# Extract the session ID for the actual user
SESSION_ID=$(loginctl --no-legend | grep "$ACTUAL_USER" | head -1 | awk '{print $1}')
if ! loginctl show-session "$SESSION_ID" -p Active 2>/dev/null | grep -q "Active=yes"; then
    # User doesn't have an active systemd session, use ACL fallback
    grant_uinput_access "$ACTUAL_USER"
else
    echo -e "${GREEN}✓${NC} Using systemd-logind uaccess for automatic permission management"
    # Still try ACL as extra safety for immediate access
    grant_uinput_access "$ACTUAL_USER" 2>/dev/null || true
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
Exec=win11-clipboard-history --quiet
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

# Try to launch the app for the user via desktop environment
launch_app() {
    local user="$1"
    
    if [ -z "$user" ] || [ "$user" = "root" ]; then
        return 1
    fi
    
    # Get user's runtime dir for proper D-Bus access
    local user_id
    user_id=$(id -u "$user" 2>/dev/null) || return 1
    local runtime_dir="/run/user/$user_id"
    
    # Check if we have a display
    if [ -z "$DISPLAY" ] && [ -z "$WAYLAND_DISPLAY" ]; then
        return 1
    fi
    
    # Try to launch via gtk-launch (works properly with user context)
    if command -v gtk-launch &> /dev/null; then
        sudo -u "$user" \
        DISPLAY="${DISPLAY:-:0}" \
        WAYLAND_DISPLAY="$WAYLAND_DISPLAY" \
        XDG_RUNTIME_DIR="$runtime_dir" \
        DBUS_SESSION_BUS_ADDRESS="unix:path=$runtime_dir/bus" \
        gtk-launch win11-clipboard-history 2>/dev/null &
        return 0
    fi
    
    return 1
}

APP_LAUNCHED=false
if launch_app "$ACTUAL_USER"; then
    sleep 1
    if pgrep -u "$ACTUAL_USER" -x "win11-clipboard-history" > /dev/null 2>&1; then
        APP_LAUNCHED=true
    fi
fi

echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║          Windows 11 Clipboard History installed!              ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

if [ "$APP_LAUNCHED" = true ]; then
    echo -e "${GREEN}✓ App is now running! Press Super+V to open.${NC}"
else
    echo -e "${GREEN}✓ Ready to use!${NC}"
    echo ""
    echo "The app will start automatically on your next login."
    echo "To start now, find 'Clipboard History' in your application menu."
fi

echo ""
echo -e "${BLUE}Note:${NC} If the Super+V shortcut doesn't work automatically,"
echo "please configure it manually in your desktop's keyboard settings:"
echo ""
echo "  GNOME/Pantheon: Settings → Keyboard → Shortcuts → Custom"
echo "  KDE Plasma:     System Settings → Shortcuts → Custom Shortcuts"
echo "  XFCE:           Settings → Keyboard → Application Shortcuts"
echo "  Cinnamon:       System Settings → Keyboard → Shortcuts → Custom"
echo "  COSMIC:         Settings → Keyboard → Custom Shortcuts"
echo ""
echo "  Command:  win11-clipboard-history"
echo "  Shortcut: Super+V"
echo ""

exit 0
