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

# Add user to input group if running interactively
if [ -n "$ACTUAL_USER" ] && [ "$ACTUAL_USER" != "root" ]; then
    if ! groups "$ACTUAL_USER" 2>/dev/null | grep -q '\binput\b'; then
        usermod -aG input "$ACTUAL_USER"
        echo -e "${GREEN}✓${NC} Added $ACTUAL_USER to 'input' group"
    fi
fi

echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║          Windows 11 Clipboard History installed!              ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}IMPORTANT: Please log out and log back in for permissions to apply.${NC}"
echo ""
echo "After logging back in:"
echo "  • Press Super+V or Ctrl+Alt+V to open clipboard history"
echo "  • The app runs in the system tray"
echo "  • Run 'win11-clipboard-history --version' to check version"
echo ""

exit 0
