#!/bin/bash
# install.sh - Smart installer for Win11 Clipboard History
# Usage: curl -fsSL https://raw.githubusercontent.com/gustavosett/Windows-11-Clipboard-History-For-Linux/master/scripts/install.sh | bash

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log()     { echo -e "${BLUE}[*]${NC} $1"; }
success() { echo -e "${GREEN}[✓]${NC} $1"; }
warn()    { echo -e "${YELLOW}[!]${NC} $1"; }
error()   { echo -e "${RED}[✗]${NC} $1"; exit 1; }

# Configuration
REPO_OWNER="gustavosett"
REPO_NAME="Windows-11-Clipboard-History-For-Linux"
CLOUDSMITH_REPO="gustavosett/clipboard-manager"

# Cleanup previous AppImage installation (prevents conflicts with package manager installs)
cleanup_appimage_installation() {
    local has_appimage=false
    
    # Check for AppImage installation artifacts
    if [ -f "$HOME/.local/bin/win11-clipboard-history.AppImage" ] || \
       [ -f "$HOME/.local/bin/win11-clipboard-history" ] || \
       [ -f "$HOME/.local/share/applications/win11-clipboard-history.desktop" ]; then
        has_appimage=true
    fi
    
    if [ "$has_appimage" = true ]; then
        log "Detected previous AppImage installation. Cleaning up..."
        
        # Kill any running instances
        pkill -f "win11-clipboard-history.AppImage" 2>/dev/null || true
        pkill -f "win11-clipboard-history-bin" 2>/dev/null || true

        # Wait for processes to terminate, with a timeout
        timeout=5
        interval=1
        elapsed=0
        while pgrep -f "win11-clipboard-history.AppImage" >/dev/null 2>&1 || \
              pgrep -f "win11-clipboard-history-bin" >/dev/null 2>&1; do
            if [ "$elapsed" -ge "$timeout" ]; then
                warn "Timed out waiting for Win11 Clipboard History AppImage processes to terminate."
                break
            fi
            sleep "$interval"
            elapsed=$((elapsed + interval))
        done
        
        # Remove AppImage files
        rm -f "$HOME/.local/bin/win11-clipboard-history.AppImage" 2>/dev/null || true
        rm -f "$HOME/.local/bin/win11-clipboard-history" 2>/dev/null || true
        rm -f "$HOME/.local/share/applications/win11-clipboard-history.desktop" 2>/dev/null || true
        rm -f "$HOME/.local/share/icons/hicolor"/*/apps/win11-clipboard-history.png 2>/dev/null || true
        
        # Update desktop database if available
        if command -v update-desktop-database &>/dev/null; then
            update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
        fi
        
        success "Previous AppImage installation cleaned up"
    fi
}

# Detect the distribution
detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        DISTRO_ID="${ID}"
        DISTRO_ID_LIKE="${ID_LIKE:-}"
        SYSTEM_FAMILY_INFO=$(echo "$ID $ID_LIKE" | tr '[:upper:]' '[:lower:]')
    else
        error "Cannot detect distribution. /etc/os-release not found."
    fi
}

# Detect system architecture and set DEB_ARCH/RPM_ARCH
detect_arch() {
    local arch
    arch=$(uname -m)
    
    case "$arch" in
        x86_64|amd64)
            DEB_ARCH="amd64"
            RPM_ARCH="x86_64"
            ;;
        aarch64|arm64)
            DEB_ARCH="arm64"
            RPM_ARCH="aarch64"
            ;;
        armv7l|armhf)
            DEB_ARCH="armhf"
            RPM_ARCH="armv7hl"
            ;;
        *)
            warn "Unknown architecture: $arch. Defaulting to x86_64."
            DEB_ARCH="amd64"
            RPM_ARCH="x86_64"
            ;;
    esac
    
    log "Architecture: $arch (DEB: $DEB_ARCH, RPM: $RPM_ARCH)"
}

# Check WebKitGTK compatibility
check_webkit_compatibility() {
    log "Checking WebKitGTK compatibility..."
    
    if ldconfig -p 2>/dev/null | grep -q "libwebkit2gtk-4.1"; then
        success "WebKitGTK 4.1 found (modern)"
        return 0
    elif ldconfig -p 2>/dev/null | grep -q "libwebkit2gtk-4.0"; then
        warn "WebKitGTK 4.0 found (legacy)"
        warn "Package manager installation may fail. AppImage fallback available."
        return 1
    else
        warn "WebKitGTK not found. Will be installed with the package."
        return 0
    fi
}

# Installation via package manager
install_via_package_manager() {
    # Clean up any previous AppImage installation to prevent PATH conflicts
    cleanup_appimage_installation
    
    # 1. Check for Arch Family (Arch, Manjaro, CachyOS, Endeavour, etc)
    if [[ "$SYSTEM_FAMILY_INFO" =~ "arch" ]] || command -v pacman &>/dev/null; then
        install_aur
        return 0
    
    # 2. Check for Debian/Ubuntu Family
    elif [[ "$SYSTEM_FAMILY_INFO" =~ "debian" || "$SYSTEM_FAMILY_INFO" =~ "ubuntu" ]] || command -v apt-get &>/dev/null; then
        install_deb
        return 0
        
    # 3. Check for Fedora/RHEL Family
    elif [[ "$SYSTEM_FAMILY_INFO" =~ "fedora" || "$SYSTEM_FAMILY_INFO" =~ "rhel" || "$SYSTEM_FAMILY_INFO" =~ "centos" ]] || command -v dnf &>/dev/null; then
        install_rpm
        return 0
        
    # 4. Check for OpenSUSE Family
    elif [[ "$SYSTEM_FAMILY_INFO" =~ "suse" ]] || command -v zypper &>/dev/null; then
        install_rpm_suse
        return 0
    fi

    return 1  # Unknown system family
}

install_deb() {
    log "Setting up APT repository (Cloudsmith)..."
    
    # Install prerequisites for HTTPS repos
    sudo apt-get update -qq
    sudo apt-get install -y apt-transport-https curl || true
    
    # Try Cloudsmith repository first (enables auto-updates)
    if curl -1sLf "https://dl.cloudsmith.io/public/${CLOUDSMITH_REPO}/setup.deb.sh" | sudo -E bash 2>/dev/null; then
        log "Installing win11-clipboard-history from repository..."
        sudo apt-get update -qq
        if sudo apt-get install -y win11-clipboard-history; then
            success "Installed via APT repository! (auto-updates enabled)"
            return 0
        fi
    fi
    
    # Fallback: download from GitHub releases
    warn "Repository not available, falling back to GitHub release..."
    log "Installing from GitHub releases (.deb)..."
    
    LATEST_RELEASE_URL="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest"
    RELEASE_TAG=$(curl -s "$LATEST_RELEASE_URL" | grep '"tag_name":' | head -n 1 | sed -E 's/.*"([^"]+)".*/\1/' | tr -cd '[:alnum:]._-')
    [ -z "$RELEASE_TAG" ] && error "Failed to fetch version."
    CLEAN_VERSION="${RELEASE_TAG#v}"
    
    TEMP_DIR=$(mktemp -d)
    chmod 755 "$TEMP_DIR"
    cd "$TEMP_DIR"
    trap 'rm -rf "$TEMP_DIR"' EXIT
    
    FILE="win11-clipboard-history_${CLEAN_VERSION}_${DEB_ARCH}.deb"
    BASE_URL="https://github.com/$REPO_OWNER/$REPO_NAME/releases/download/$RELEASE_TAG"
    
    log "Downloading $FILE..."
    curl -L -o "$FILE" "$BASE_URL/$FILE" --progress-bar
    chmod 644 "$FILE"
    
    log "Installing dependencies..."
    sudo apt-get install -y xclip wl-clipboard acl || true
    sudo apt-get install -y libayatana-appindicator3-1 || sudo apt-get install -y libappindicator3-1 || true
    
    log "Installing .deb package..."
    yes | sudo apt-get install -y "./$FILE"
    
    success "Installed via APT (from GitHub release)"
}

install_rpm() {
    log "Setting up RPM repository (Cloudsmith)..."
    
    # Try Cloudsmith repository first (enables auto-updates)
    if curl -1sLf "https://dl.cloudsmith.io/public/${CLOUDSMITH_REPO}/setup.rpm.sh" | sudo -E bash 2>/dev/null; then
        log "Installing win11-clipboard-history from repository..."
        if sudo dnf install -y win11-clipboard-history; then
            success "Installed via DNF repository! (auto-updates enabled)"
            return 0
        fi
    fi
    
    # Fallback: download from GitHub releases
    warn "Repository not available, falling back to GitHub release..."
    log "Installing from GitHub releases (.rpm)..."
    
    LATEST_RELEASE_URL="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest"
    RELEASE_TAG=$(curl -s "$LATEST_RELEASE_URL" | grep '"tag_name":' | head -n 1 | sed -E 's/.*"([^"]+)".*/\1/' | tr -cd '[:alnum:]._-')
    [ -z "$RELEASE_TAG" ] && error "Failed to fetch version."
    CLEAN_VERSION="${RELEASE_TAG#v}"
    
    TEMP_DIR=$(mktemp -d)
    chmod 755 "$TEMP_DIR"
    cd "$TEMP_DIR"
    trap 'rm -rf "$TEMP_DIR"' EXIT
    
    FILE="win11-clipboard-history-${CLEAN_VERSION}-1.${RPM_ARCH}.rpm"
    BASE_URL="https://github.com/$REPO_OWNER/$REPO_NAME/releases/download/$RELEASE_TAG"
    
    log "Downloading $FILE..."
    curl -L -o "$FILE" "$BASE_URL/$FILE" --progress-bar
    chmod 644 "$FILE"
    
    log "Installing dependencies..."
    sudo dnf install -y xclip wl-clipboard acl libayatana-appindicator-gtk3 || true
    
    log "Installing .rpm package..."
    sudo dnf install -y "./$FILE"
    
    success "Installed via DNF (from GitHub release)"
}

install_rpm_suse() {
    log "Setting up RPM repository (Cloudsmith)..."
    
    # Try Cloudsmith repository first (enables auto-updates)
    if curl -1sLf "https://dl.cloudsmith.io/public/${CLOUDSMITH_REPO}/setup.rpm.sh" | sudo -E bash 2>/dev/null; then
        log "Installing win11-clipboard-history from repository..."
        if sudo zypper install -y win11-clipboard-history; then
            success "Installed via Zypper repository! (auto-updates enabled)"
            return 0
        fi
    fi
    
    # Fallback: download from GitHub releases
    warn "Repository not available, falling back to GitHub release..."
    log "Installing from GitHub releases (.rpm)..."
    
    LATEST_RELEASE_URL="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest"
    RELEASE_TAG=$(curl -s "$LATEST_RELEASE_URL" | grep '"tag_name":' | head -n 1 | sed -E 's/.*"([^"]+)".*/\1/' | tr -cd '[:alnum:]._-')
    [ -z "$RELEASE_TAG" ] && error "Failed to fetch version."
    CLEAN_VERSION="${RELEASE_TAG#v}"
    
    TEMP_DIR=$(mktemp -d)
    chmod 755 "$TEMP_DIR"
    cd "$TEMP_DIR"
    trap 'rm -rf "$TEMP_DIR"' EXIT
    
    FILE="win11-clipboard-history-${CLEAN_VERSION}-1.${RPM_ARCH}.rpm"
    BASE_URL="https://github.com/$REPO_OWNER/$REPO_NAME/releases/download/$RELEASE_TAG"
    
    log "Downloading $FILE..."
    curl -L -o "$FILE" "$BASE_URL/$FILE" --progress-bar
    chmod 644 "$FILE"
    
    log "Installing dependencies..."
    sudo zypper install -y xclip wl-clipboard acl libayatana-appindicator3-1 || true
    
    log "Installing .rpm package..."
    sudo zypper install -y "./$FILE"
    
    success "Installed via Zypper (from GitHub release)"
}

install_aur() {
    log "Installing from AUR..."
    
    # Detect AUR helper
    if command -v yay &>/dev/null; then
        yay -S --noconfirm win11-clipboard-history-bin
    elif command -v paru &>/dev/null; then
        paru -S --noconfirm win11-clipboard-history-bin
    else
        warn "No AUR helper found. Installing yay first..."
        sudo pacman -S --needed --noconfirm git base-devel
        git clone https://aur.archlinux.org/yay-bin.git /tmp/yay-bin
        cd /tmp/yay-bin && makepkg -si --noconfirm
        yay -S --noconfirm win11-clipboard-history-bin
    fi
    
    success "Installed via AUR!"
}

install_appimage() {
    log "Installing AppImage (universal fallback)..."
    
    # Fetch latest version
    LATEST_URL=$(curl -s https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest \
        | grep "browser_download_url.*AppImage" \
        | cut -d '"' -f 4)
    
    if [ -z "$LATEST_URL" ]; then
        error "Could not find AppImage download URL"
    fi
    
    # Create directories
    mkdir -p "$HOME/.local/bin"
    mkdir -p "$HOME/.local/share/applications"
    mkdir -p "$HOME/.local/share/icons/hicolor/128x128/apps"
    
    # Download AppImage
    log "Downloading AppImage..."
    curl -fsSL -o "$HOME/.local/bin/win11-clipboard-history.AppImage" "$LATEST_URL"
    chmod +x "$HOME/.local/bin/win11-clipboard-history.AppImage"
    
    # Download app icon for proper menu integration
    log "Downloading app icon..."
    curl -fsSL -o "$HOME/.local/share/icons/hicolor/128x128/apps/win11-clipboard-history.png" \
        "https://raw.githubusercontent.com/$REPO_OWNER/$REPO_NAME/master/src-tauri/icons/128x128.png" 2>/dev/null || true
    
    # Wrapper script
    cat > "$HOME/.local/bin/win11-clipboard-history" << 'EOF'
#!/bin/bash
exec env -i \
    HOME="$HOME" USER="$USER" SHELL="$SHELL" \
    DISPLAY="${DISPLAY:-:0}" XAUTHORITY="$XAUTHORITY" \
    WAYLAND_DISPLAY="$WAYLAND_DISPLAY" XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR" \
    XDG_SESSION_TYPE="$XDG_SESSION_TYPE" XDG_CURRENT_DESKTOP="$XDG_CURRENT_DESKTOP" \
    XDG_DATA_DIRS="${XDG_DATA_DIRS:-/usr/local/share:/usr/share}" \
    DBUS_SESSION_BUS_ADDRESS="$DBUS_SESSION_BUS_ADDRESS" \
    PATH="$HOME/.local/bin:/usr/local/bin:/usr/bin:/bin" \
    LANG="${LANG:-en_US.UTF-8}" \
    GDK_BACKEND="x11" \
    GDK_SCALE="${GDK_SCALE:-1}" GDK_DPI_SCALE="${GDK_DPI_SCALE:-1}" \
    NO_AT_BRIDGE=1 \
    "$HOME/.local/bin/win11-clipboard-history.AppImage" "$@"
EOF
    chmod +x "$HOME/.local/bin/win11-clipboard-history"
    
    # .desktop file with proper icon
    cat > "$HOME/.local/share/applications/win11-clipboard-history.desktop" << EOF
[Desktop Entry]
Type=Application
Name=Clipboard History
Comment=Windows 11-style Clipboard History Manager
Exec=$HOME/.local/bin/win11-clipboard-history
Icon=win11-clipboard-history
Terminal=false
Categories=Utility;
StartupWMClass=win11-clipboard-history
EOF
    
    # Ask about udev rules for AppImage (optional - maintains portability)
    setup_udev_appimage_optional
    
    success "AppImage installed to ~/.local/bin/"
    warn "Add ~/.local/bin to your PATH if not already there"
}

setup_udev_appimage_optional() {
    echo ""
    warn "For paste simulation to work, the app needs access to /dev/uinput."
    echo ""
    echo "You have two options:"
    echo "  1. Quick fix (no logout required): Run this command:"
    echo "     sudo setfacl -m u:$USER:rw /dev/uinput"
    echo ""
    echo "  2. Permanent fix (requires sudo, then logout/login):"
    echo "     The installer can set up udev rules for you."
    echo ""
    
    # Check if running interactively
    if [ -t 0 ]; then
        read -p "Set up permanent udev rules now? [y/N] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            setup_udev_appimage
        else
            log "Skipping udev setup. You can run this later if paste doesn't work:"
            echo "  sudo setfacl -m u:$USER:rw /dev/uinput"
        fi
    else
        # Non-interactive: just use ACL for immediate access
        log "Non-interactive mode: Using ACL for immediate access..."
        if command -v setfacl &>/dev/null && [ -e /dev/uinput ]; then
            sudo setfacl -m "u:${USER}:rw" /dev/uinput 2>/dev/null || true
            success "Permissions configured via ACL"
        else
            warn "Run 'sudo setfacl -m u:$USER:rw /dev/uinput' if paste doesn't work"
        fi
    fi
}

setup_udev_appimage() {
    log "Setting up permanent uinput permissions (requires sudo)..."
    
    # Create udev rule
    sudo tee /etc/udev/rules.d/99-win11-clipboard-input.rules > /dev/null << 'EOF'
# udev rules for Windows 11 Clipboard History
ACTION=="add", SUBSYSTEM=="misc", KERNEL=="uinput", OPTIONS+="static_node=uinput"
KERNEL=="uinput", SUBSYSTEM=="misc", MODE="0660", GROUP="input", TAG+="uaccess"
EOF
    
    # Configure module to load on boot
    echo "uinput" | sudo tee /etc/modules-load.d/win11-clipboard.conf > /dev/null
    
    # Load now
    sudo modprobe uinput 2>/dev/null || true
    sudo udevadm control --reload-rules 2>/dev/null || true
    sudo udevadm trigger --subsystem-match=misc 2>/dev/null || true
    
    # ACL for immediate access
    if command -v setfacl &>/dev/null && [ -e /dev/uinput ]; then
        sudo setfacl -m "u:${USER}:rw" /dev/uinput 2>/dev/null || true
        success "Permissions configured (immediate access via ACL)"
    else
        warn "You may need to log out and back in for permissions to take effect"
    fi
}

launch_app() {
    log "Starting application..."
    
    # Kill any existing instances (matches both wrapper and -bin binary)
    pkill -f "win11-clipboard-history-bin" 2>/dev/null || true
    pkill -f "win11-clipboard-history.AppImage" 2>/dev/null || true
    sleep 1
    
    # Launch detached from terminal
    nohup win11-clipboard-history >/dev/null 2>&1 < /dev/null & disown
    
    sleep 2
    
    if pgrep -f "win11-clipboard-history" > /dev/null; then
        return 0
    else
        return 1
    fi
}

# Main
main() {
    echo ""
    echo "╔═══════════════════════════════════════════════════════════╗"
    echo "║     Win11 Clipboard History - Linux Installer             ║"
    echo "╚═══════════════════════════════════════════════════════════╝"
    echo ""
    
    command -v curl >/dev/null 2>&1 || error "curl is required."
    
    detect_distro
    detect_arch
    log "Detected: $DISTRO_ID (Family: $SYSTEM_FAMILY_INFO)"
    
    # Check WebKitGTK compatibility
    check_webkit_compatibility
    webkit_status=$?
    
    # Prefer AppImage if only legacy WebKitGTK 4.0 is available
    if [ "$webkit_status" -eq 1 ]; then
        warn "Legacy WebKitGTK detected. Preferring AppImage for better compatibility."
        install_appimage
    # Try package manager first
    elif install_via_package_manager; then
        success "Package installation complete!"
    else
        warn "No native package found for your system family. Using AppImage."
        install_appimage
    fi
    
    # Try to launch
    if launch_app; then
        echo ""
        success "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        success " Installation complete! App is running."
        success " Press Super+V to open your clipboard history."
        success "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    else
        echo ""
        success "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        success " Installation complete!"
        success " Run 'win11-clipboard-history' or find it in your menu."
        success " Press Super+V to open your clipboard history."
        success "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    fi
    echo ""
}

main "$@"
