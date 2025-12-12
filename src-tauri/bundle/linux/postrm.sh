#!/bin/bash
# Post-removal script for Windows 11 Clipboard History

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m'

echo -e "${BLUE}Cleaning up Windows 11 Clipboard History...${NC}"

# Get the actual user (not root when using sudo)
ACTUAL_USER="${SUDO_USER:-$USER}"

# Remove autostart entry for the user
remove_autostart() {
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
    
    local desktop_file="$user_home/.config/autostart/win11-clipboard-history.desktop"
    
    if [ -f "$desktop_file" ]; then
        rm -f "$desktop_file"
        echo -e "${GREEN}✓${NC} Removed autostart entry"
    fi
}

# Kill any running instances
stop_running_instances() {
    if pgrep -x "win11-clipboard-history" > /dev/null 2>&1; then
        echo -e "${BLUE}Stopping running instances...${NC}"
        pkill -x "win11-clipboard-history" 2>/dev/null || true
        echo -e "${GREEN}✓${NC} Stopped running instances"
    fi
}

stop_running_instances
remove_autostart "$ACTUAL_USER"

# Also try to remove for all users who might have it
for user_home in /home/*; do
    if [ -d "$user_home" ]; then
        desktop_file="$user_home/.config/autostart/win11-clipboard-history.desktop"
        if [ -f "$desktop_file" ]; then
            rm -f "$desktop_file" 2>/dev/null || true
        fi
    fi
done

echo -e "${GREEN}✓${NC} Windows 11 Clipboard History has been removed"

exit 0
