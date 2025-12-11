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

# Ensure input group exists
if ! getent group input > /dev/null 2>&1; then
    echo -e "${BLUE}Creating 'input' group...${NC}"
    groupadd input
fi

# Create udev rules for input devices and uinput
UDEV_RULE="/etc/udev/rules.d/99-win11-clipboard-input.rules"
cat > "$UDEV_RULE" << 'EOF'
# udev rules for Windows 11 Clipboard History
# Input devices (keyboards, mice) - needed for rdev global hotkeys
KERNEL=="event*", SUBSYSTEM=="input", MODE="0660", GROUP="input"
# uinput device - needed for enigo keyboard simulation (paste injection)
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
echo ""

exit 0
