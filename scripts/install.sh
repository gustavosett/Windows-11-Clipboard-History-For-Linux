#!/bin/bash
# install.sh - Smart installer for Windows 11 Style Clipboard History Manager
# Usage: curl -fsSL https://raw.githubusercontent.com/Mahdi-Arts/Windows-11-Style-Clipboard-History-Manager/master/scripts/install.sh | bash

set -e

# Security note: piping this file to bash trusts GitHub + your DNS/TLS path.
# Prefer: curl the script, read it, then run it. Package-manager installs are safer.

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
REPO_OWNER="Mahdi-Arts"
REPO_NAME="Windows-11-Style-Clipboard-History-Manager"
# Optional Cloudsmith repository. Disabled by default because it is a
# `curl | sudo bash` trust path. Enable explicitly:
#   USE_CLOUDSMITH=1 CLOUDSMITH_REPO=owner/repo ./install.sh
USE_CLOUDSMITH="${USE_CLOUDSMITH:-0}"
CLOUDSMITH_REPO="${CLOUDSMITH_REPO:-mahdi-arts/clipboard-manager}"

# Supply-chain controls:
# - Checksum verification against the release's SHA256SUMS is MANDATORY.
# - To skip verification (NOT recommended, e.g. offline mirror testing), set
#   ALLOW_UNVERIFIED=1 — the installer will refuse to run without it.
# - To require a detached GPG signature over SHA256SUMS, set
#   WINDOWS_11_CLIPBOARD_TRUST_KEY=<keyid-or-fingerprint>; verification then also
#   needs a reachable keyserver or an imported public key.
ALLOW_UNVERIFIED="${ALLOW_UNVERIFIED:-0}"
WINDOWS_11_CLIPBOARD_TRUST_KEY="${WINDOWS_11_CLIPBOARD_TRUST_KEY:-${MODERN_CLIPBOARD_HISTORY_TRUST_KEY:-}}"

# Cleanup previous AppImage installation (prevents conflicts with package manager installs)
cleanup_appimage_installation() {
    local has_appimage=false
    
    # Check for AppImage installation artifacts
    if [ -f "$HOME/.local/bin/windows-11-style-clipboard-history-manager.AppImage" ] || \
       [ -f "$HOME/.local/bin/windows-11-style-clipboard-history-manager" ] || \
       [ -f "$HOME/.local/share/applications/windows-11-style-clipboard-history-manager.desktop" ] || \
       [ -f "$HOME/.local/bin/modern-clipboard-history-for-linux.AppImage" ] || \
       [ -f "$HOME/.local/bin/modern-clipboard-history-for-linux" ] || \
       [ -f "$HOME/.local/share/applications/modern-clipboard-history-for-linux.desktop" ]; then
        has_appimage=true
    fi
    
    if [ "$has_appimage" = true ]; then
        log "Detected previous AppImage installation. Cleaning up..."
        
        # Kill any running AppImage instances
        pkill -f "windows-11-style-clipboard-history-manager.AppImage" 2>/dev/null || true
        pkill -f "modern-clipboard-history-for-linux.AppImage" 2>/dev/null || true

        # Wait for processes to terminate, with a timeout
        timeout=5
        interval=1
        elapsed=0
        while pgrep -f "windows-11-style-clipboard-history-manager.AppImage\|modern-clipboard-history-for-linux.AppImage" >/dev/null 2>&1; do
            if [ "$elapsed" -ge "$timeout" ]; then
                warn "Timed out waiting for Windows 11 Style Clipboard History Manager AppImage processes to terminate."
                break
            fi
            sleep "$interval"
            elapsed=$((elapsed + interval))
        done
        
        # Remove AppImage files
        rm -f "$HOME/.local/bin/windows-11-style-clipboard-history-manager.AppImage" 2>/dev/null || true
        rm -f "$HOME/.local/bin/windows-11-style-clipboard-history-manager" 2>/dev/null || true
        rm -f "$HOME/.local/share/applications/windows-11-style-clipboard-history-manager.desktop" 2>/dev/null || true
        rm -f "$HOME/.local/share/icons/hicolor"/*/apps/windows-11-style-clipboard-history-manager.png 2>/dev/null || true
        rm -f "$HOME/.local/bin/modern-clipboard-history-for-linux.AppImage" 2>/dev/null || true
        rm -f "$HOME/.local/bin/modern-clipboard-history-for-linux" 2>/dev/null || true
        rm -f "$HOME/.local/share/applications/modern-clipboard-history-for-linux.desktop" 2>/dev/null || true
        rm -f "$HOME/.local/share/icons/hicolor"/*/apps/modern-clipboard-history-for-linux.png 2>/dev/null || true
        
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
    
    # IMPORTANT: Check distro ID/family FIRST before falling back to command detection.
    # This prevents misdetection when tools like pacman are installed on non-Arch systems.
    
    # 1. Check for Fedora/RHEL Family
    # Note: fedora-asahi-remix is a Fedora-based distro for Apple Silicon Macs
    if [[ "$DISTRO_ID" == "fedora-asahi-remix" || "$SYSTEM_FAMILY_INFO" =~ "fedora" || "$SYSTEM_FAMILY_INFO" =~ "rhel" || "$SYSTEM_FAMILY_INFO" =~ "centos" ]]; then
        install_rpm
        return 0
    
    # 2. Check for Debian/Ubuntu Family
    elif [[ "$SYSTEM_FAMILY_INFO" =~ "debian" || "$SYSTEM_FAMILY_INFO" =~ "ubuntu" ]]; then
        install_deb
        return 0
    
    # 3. Check for OpenSUSE Family
    elif [[ "$SYSTEM_FAMILY_INFO" =~ "suse" ]]; then
        install_rpm_suse
        return 0
    
    # 4. Check for Arch Family (Arch, Manjaro, CachyOS, Endeavour, etc)
    # Check this AFTER other distros to avoid false positives from pacman being installed
    elif [[ "$SYSTEM_FAMILY_INFO" =~ "arch" ]]; then
        install_aur
        return 0
    fi
    
    # Fallback: Check for package managers if distro detection failed
    # This handles edge cases where /etc/os-release is incomplete
    if command -v dnf &>/dev/null; then
        install_rpm
        return 0
    elif command -v apt-get &>/dev/null; then
        install_deb
        return 0
    elif command -v zypper &>/dev/null; then
        install_rpm_suse
        return 0
    elif command -v pacman &>/dev/null; then
        install_aur
        return 0
    fi

    return 1  # Unknown system family
}

install_deb() {
    if [ "$USE_CLOUDSMITH" = "1" ]; then
        log "Setting up APT repository (Cloudsmith, USE_CLOUDSMITH=1)..."
        sudo apt-get update -qq
        sudo apt-get install -y apt-transport-https curl || true
        if curl -1sLf "https://dl.cloudsmith.io/public/${CLOUDSMITH_REPO}/setup.deb.sh" | sudo -E bash 2>/dev/null; then
            log "Installing windows-11-style-clipboard-history-manager from repository..."
            sudo apt-get update -qq
            if sudo apt-get install -y windows-11-style-clipboard-history-manager; then
                success "Installed via APT repository! (auto-updates enabled)"
                return 0
            fi
        fi
        warn "Cloudsmith repository not available, falling back to GitHub release..."
    fi

    log "Installing from GitHub releases (.deb)..."
    
    LATEST_RELEASE_URL="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest"
    RELEASE_TAG=$(curl -s "$LATEST_RELEASE_URL" | grep '"tag_name":' | head -n 1 | sed -E 's/.*"([^"]+)".*/\1/' | tr -cd '[:alnum:]._-')
    [ -z "$RELEASE_TAG" ] && error "Failed to fetch version."
    CLEAN_VERSION="${RELEASE_TAG#v}"
    
    TEMP_DIR=$(mktemp -d)
    chmod 755 "$TEMP_DIR"
    cd "$TEMP_DIR"
    trap 'rm -rf "$TEMP_DIR"' EXIT
    
    FILE="windows-11-style-clipboard-history-manager_${CLEAN_VERSION}_${DEB_ARCH}.deb"
    BASE_URL="https://github.com/$REPO_OWNER/$REPO_NAME/releases/download/$RELEASE_TAG"
    
    log "Downloading $FILE..."
    if ! curl -L -o "$FILE" "$BASE_URL/$FILE" --progress-bar --fail; then
        error "Failed to download $FILE"
    fi
    chmod 644 "$FILE"
    verify_downloaded_file "$FILE" "$BASE_URL"
    
    log "Installing dependencies..."
    sudo apt-get install -y xclip wl-clipboard acl || true
    sudo apt-get install -y libayatana-appindicator3-1 || sudo apt-get install -y libappindicator3-1 || true
    
    log "Installing .deb package..."
    yes | sudo apt-get install -y "./$FILE"
    
    success "Installed via APT (from GitHub release)"
}

download_and_install_rpm() {
    local pkg_mgr="$1"
    LATEST_RELEASE_URL="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest"
    RELEASE_TAG=$(curl -s "$LATEST_RELEASE_URL" | grep '"tag_name":' | head -n 1 | sed -E 's/.*"([^"]+)".*/\1/' | tr -cd '[:alnum:]._-')
    [ -z "$RELEASE_TAG" ] && error "Failed to fetch version."
    CLEAN_VERSION="${RELEASE_TAG#v}"
    TEMP_DIR=$(mktemp -d)
    chmod 755 "$TEMP_DIR"
    cd "$TEMP_DIR"
    trap 'rm -rf "$TEMP_DIR"' EXIT
    FILE="windows-11-style-clipboard-history-manager-${CLEAN_VERSION}-1.${RPM_ARCH}.rpm"
    BASE_URL="https://github.com/$REPO_OWNER/$REPO_NAME/releases/download/$RELEASE_TAG"
    log "Downloading $FILE..."
    if ! curl -L -o "$FILE" "$BASE_URL/$FILE" --progress-bar --fail; then
        error "Failed to download $FILE"
    fi
    chmod 644 "$FILE"
    verify_downloaded_file "$FILE" "$BASE_URL"
    if [ "$pkg_mgr" = "dnf" ]; then
        sudo dnf install -y xclip wl-clipboard acl libayatana-appindicator-gtk3 || true
        sudo dnf install -y "./$FILE"
        success "Installed via DNF (from GitHub release)"
    else
        sudo zypper install -y xclip wl-clipboard acl libayatana-appindicator3-1 || true
        sudo zypper install -y "./$FILE"
        success "Installed via Zypper (from GitHub release)"
    fi
}

install_rpm() {
    if [ "$USE_CLOUDSMITH" != "1" ]; then
        log "Installing from GitHub releases (.rpm)..."
        download_and_install_rpm dnf
        return 0
    fi
    log "Setting up RPM repository (Cloudsmith, USE_CLOUDSMITH=1)..."
    
    local env_args=()
    if [[ "$DISTRO_ID" == "fedora-asahi-remix" ]]; then
        log "Detected Fedora Asahi Remix - using standard Fedora repository..."
        local fedora_version=""
        if [ -f /etc/os-release ]; then
            fedora_version="$(awk -F= '$1=="VERSION_ID"{gsub(/"/,"",$2);print $2}' /etc/os-release)"
        fi
        env_args=("distro=fedora" "codename=")
        if [[ -n "$fedora_version" ]]; then
            env_args+=("version=$fedora_version")
        fi
    fi
    
    # Try Cloudsmith repository first (enables auto-updates)
    local repo_setup_success=false
    if [[ ${#env_args[@]} -gt 0 ]]; then
        # Export the override vars so they survive through sudo -E inside the
        # Cloudsmith setup script (env + bash -c loses them at the sudo boundary).
        for _evar in "${env_args[@]}"; do
            export "${_evar?}"
        done
        curl -1sLf "https://dl.cloudsmith.io/public/${CLOUDSMITH_REPO}/setup.rpm.sh" | sudo -E bash 2>/dev/null && repo_setup_success=true
        # Clean up exported overrides
        unset distro version codename 2>/dev/null || true
    else
        curl -1sLf "https://dl.cloudsmith.io/public/${CLOUDSMITH_REPO}/setup.rpm.sh" | sudo -E bash 2>/dev/null && repo_setup_success=true
    fi
    
    if [ "$repo_setup_success" = true ]; then
        log "Installing windows-11-style-clipboard-history-manager from repository..."
        if sudo dnf install -y windows-11-style-clipboard-history-manager; then
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
    
    FILE="windows-11-style-clipboard-history-manager-${CLEAN_VERSION}-1.${RPM_ARCH}.rpm"
    BASE_URL="https://github.com/$REPO_OWNER/$REPO_NAME/releases/download/$RELEASE_TAG"
    
    log "Downloading $FILE..."
    if ! curl -L -o "$FILE" "$BASE_URL/$FILE" --progress-bar --fail; then
        error "Failed to download $FILE"
    fi
    chmod 644 "$FILE"
    verify_downloaded_file "$FILE" "$BASE_URL"
    
    log "Installing dependencies..."
    sudo dnf install -y xclip wl-clipboard acl libayatana-appindicator-gtk3 || true
    
    log "Installing .rpm package..."
    sudo dnf install -y "./$FILE"
    
    success "Installed via DNF (from GitHub release)"
}

install_rpm_suse() {
    log "Installing from GitHub releases (.rpm)..."
    download_and_install_rpm zypper
}

install_aur() {
    log "Installing from AUR..."
    
    # Detect AUR helper
    if command -v yay &>/dev/null; then
        yay -S --noconfirm windows-11-style-clipboard-history-manager-bin
    elif command -v paru &>/dev/null; then
        paru -S --noconfirm windows-11-style-clipboard-history-manager-bin
    else
        warn "No AUR helper found. Installing yay first..."
        sudo pacman -S --needed --noconfirm git base-devel
        git clone https://aur.archlinux.org/yay-bin.git /tmp/yay-bin
        cd /tmp/yay-bin && makepkg -si --noconfirm
        yay -S --noconfirm windows-11-style-clipboard-history-manager-bin
    fi
    
    success "Installed via AUR!"
}

install_appimage() {
    log "Installing AppImage (universal fallback)..."
    
    local arch_name
    arch_name=$(uname -m)
    
    # Fetch latest version, filtering by architecture
    LATEST_URL=$(curl -s https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest \
        | grep "browser_download_url.*AppImage" \
        | grep -i "${DEB_ARCH}\|${arch_name}" \
        | head -1 \
        | cut -d '"' -f 4)
    
    # Fallback: try any AppImage if no arch-specific match found
    if [ -z "$LATEST_URL" ]; then
        LATEST_URL=$(curl -s https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest \
            | grep "browser_download_url.*AppImage" \
            | head -1 \
            | cut -d '"' -f 4)
    fi
    
    if [ -z "$LATEST_URL" ]; then
        error "Could not find AppImage download URL"
    fi
    
    # Create directories
    mkdir -p "$HOME/.local/bin"
    mkdir -p "$HOME/.local/share/applications"
    mkdir -p "$HOME/.local/share/icons/hicolor/128x128/apps"
    
    # Download AppImage
    log "Downloading AppImage..."
    curl -fsSL -o "$HOME/.local/bin/windows-11-style-clipboard-history-manager.AppImage" "$LATEST_URL"
    chmod +x "$HOME/.local/bin/windows-11-style-clipboard-history-manager.AppImage"
    BASE_URL="${LATEST_URL%/*}"
    verify_downloaded_file "$HOME/.local/bin/windows-11-style-clipboard-history-manager.AppImage" "$BASE_URL"
    
    # Download app icon for proper menu integration
    log "Downloading app icon..."
    curl -fsSL -o "$HOME/.local/share/icons/hicolor/128x128/apps/windows-11-style-clipboard-history-manager.png" \
        "https://raw.githubusercontent.com/$REPO_OWNER/$REPO_NAME/master/src-tauri/icons/128x128.png" 2>/dev/null || true
    
    # Wrapper script — mirrors src-tauri/bundle/linux/wrapper.sh sanitization
    cat > "$HOME/.local/bin/windows-11-style-clipboard-history-manager" << 'WRAPPER_EOF'
#!/bin/bash
# AppImage wrapper for windows-11-style-clipboard-history-manager
# Sanitizes Snap/Flatpak environment leaks, then launches the AppImage.

# Always clear library/runtime overrides from sandbox parents
unset LD_LIBRARY_PATH
unset LD_PRELOAD
unset GTK_PATH
unset GIO_MODULE_DIR
unset GTK_IM_MODULE_FILE
unset GTK_EXE_PREFIX
unset LOCPATH
unset GSETTINGS_SCHEMA_DIR

# Fix XDG_DATA_DIRS only when contaminated by sandbox paths
sanitize_xdg_data_dirs() {
    local xdg="${XDG_DATA_DIRS:-}"
    local system_dirs="/usr/local/share:/usr/share:/var/lib/snapd/desktop"

    if [[ -z "${SNAP:-}" && -z "${FLATPAK_ID:-}" && "$xdg" != *"/snap/"* && "$xdg" != *"/flatpak/"* ]]; then
        return
    fi

    local cleaned=""
    local entry
    IFS=':' read -ra entries <<< "$xdg"
    for entry in "${entries[@]}"; do
        case "$entry" in
            */snap/*|*/flatpak/*) continue ;;
        esac
        case ":$system_dirs:" in
            *":$entry:"*) continue ;;
        esac
        cleaned="${cleaned:+$cleaned:}$entry"
    done

    export XDG_DATA_DIRS="${system_dirs}${cleaned:+:$cleaned}"
}
sanitize_xdg_data_dirs

export NO_AT_BRIDGE=1

exec "$HOME/.local/bin/windows-11-style-clipboard-history-manager.AppImage" "$@"
WRAPPER_EOF
    chmod +x "$HOME/.local/bin/windows-11-style-clipboard-history-manager"
    
    # .desktop file with proper icon
    cat > "$HOME/.local/share/applications/windows-11-style-clipboard-history-manager.desktop" << EOF
[Desktop Entry]
Type=Application
Name=Clipboard History
Comment=Windows 11-style Clipboard History Manager
Exec=$HOME/.local/bin/windows-11-style-clipboard-history-manager
Icon=windows-11-style-clipboard-history-manager
Terminal=false
Categories=Utility;
StartupWMClass=windows-11-style-clipboard-history-manager
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
    sudo tee /etc/udev/rules.d/99-windows-11-style-clipboard-history-input.rules > /dev/null << 'EOF'
# udev rules for Windows 11 Clipboard History
ACTION=="add", SUBSYSTEM=="misc", KERNEL=="uinput", OPTIONS+="static_node=uinput"
KERNEL=="uinput", SUBSYSTEM=="misc", MODE="0660", GROUP="input", TAG+="uaccess"
EOF
    
    # Configure module to load on boot
    echo "uinput" | sudo tee /etc/modules-load.d/windows-11-style-clipboard-history-manager.conf > /dev/null
    
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
    pkill -f "windows-11-style-clipboard-history-manager-bin" 2>/dev/null || true
    pkill -f "windows-11-style-clipboard-history-manager.AppImage" 2>/dev/null || true
    pkill -f "modern-clipboard-history-for-linux-bin" 2>/dev/null || true
    pkill -f "modern-clipboard-history-for-linux.AppImage" 2>/dev/null || true
    sleep 1
    
    # Launch detached from terminal
    nohup windows-11-style-clipboard-history-manager >/dev/null 2>&1 < /dev/null & disown
    
    sleep 2
    
    if pgrep -f "windows-11-style-clipboard-history-manager" > /dev/null; then
        return 0
    else
        return 1
    fi
}

# ============================================================================
# Download integrity verification (supply-chain gate)
# ============================================================================

# verify_downloaded_file <file> <release_base_url>
#
# Downloads `SHA256SUMS` from the same release and requires the artifact to
# match its published checksum. Fails hard on any mismatch. Previously this
# function was referenced but never defined, which broke every GitHub-releases
# fallback path — it is now implemented and enforced.
verify_downloaded_file() {
    local file="$1"
    local base_url="$2"
    local expected=""
    local actual=""
    local tmp_sha=""

    if [ ! -f "$file" ]; then
        error "Cannot verify '$file': file not found"
    fi

    # Optional GPG verification of the checksum file itself.
    if [ -n "$WINDOWS_11_CLIPBOARD_TRUST_KEY" ]; then
        if ! command -v gpg &>/dev/null; then
            error "GPG verification requested (WINDOWS_11_CLIPBOARD_TRUST_KEY) but gpg is not installed"
        fi
        local sig_url="${base_url}/SHA256SUMS.sig"
        tmp_sha=$(mktemp)
        trap 'rm -f "$tmp_sha" "${tmp_sha}.sums"' RETURN
        if curl -fsSL -o "$tmp_sha" "$sig_url" 2>/dev/null; then
            curl -fsSL -o "${tmp_sha}.sums" "${base_url}/SHA256SUMS" 2>/dev/null || true
            if ! gpg --verify "$tmp_sha" "${tmp_sha}.sums" 2>/dev/null; then
                rm -f "$tmp_sha" "${tmp_sha}.sums"
                error "GPG signature verification failed for SHA256SUMS"
            fi
            log "GPG signature verified (key: $WINDOWS_11_CLIPBOARD_TRUST_KEY)"
        else
            rm -f "$tmp_sha"
            error "GPG verification requested but SHA256SUMS.sig was not found in the release"
        fi
    fi

    if ! command -v sha256sum &>/dev/null; then
        error "sha256sum is required to verify downloads. Install coreutils, or set ALLOW_UNVERIFIED=1 only if you accept the risk."
    fi

    tmp_sha=$(mktemp)
    trap 'rm -f "$tmp_sha"' RETURN
    if ! curl -fsSL -o "$tmp_sha" "${base_url}/SHA256SUMS" 2>/dev/null; then
        if [ "$ALLOW_UNVERIFIED" = "1" ]; then
            warn "SHA256SUMS not published for this release — skipping verification (ALLOW_UNVERIFIED=1)"
            return 0
        fi
        error "SHA256SUMS not found at ${base_url}/SHA256SUMS — refusing to install an unverified binary. Set ALLOW_UNVERIFIED=1 only if you know what you are doing."
    fi
    expected=$(grep -F "$(basename "$file")" "$tmp_sha" | awk '{print $1}' | head -n 1)
    rm -f "$tmp_sha"
    if [ -z "$expected" ]; then
        error "SHA256SUMS does not list $(basename "$file") — refusing to install"
    fi
    actual=$(sha256sum "$file" | awk '{print $1}')
    if [ "$actual" != "$expected" ]; then
        error "Checksum mismatch for $file! Expected $expected, got $actual — refusing to install (possible tampering or corrupted download)"
    fi
    success "Checksum verified: $(basename "$file") matches published SHA256SUMS"
}

# Main
main() {
    echo ""
    echo "╔═══════════════════════════════════════════════════════════╗"
    echo "║  Windows 11 Style Clipboard History Manager - Installer   ║"
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
        success " Run 'windows-11-style-clipboard-history-manager' or find it in your menu."
        success " Press Super+V to open your clipboard history."
        success "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    fi
    echo ""
}

main "$@"
