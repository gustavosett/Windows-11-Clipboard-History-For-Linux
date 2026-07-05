#!/bin/bash
# uninstall.sh - Uninstaller for Win11 Clipboard History
# Reverses every action performed by install.sh
# Usage: curl -fsSL https://raw.githubusercontent.com/gustavosett/Windows-11-Clipboard-History-For-Linux/master/scripts/uninstall.sh | bash

set -euo pipefail

# ---------------------------------------------------------------------------
# Colors / logging helpers
# ---------------------------------------------------------------------------
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log()     { echo -e "${BLUE}[*]${NC} $1"; }
success() { echo -e "${GREEN}[✓]${NC} $1"; }
warn()    { echo -e "${YELLOW}[!]${NC} $1"; }
error()   { echo -e "${RED}[✗]${NC} $1"; }

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
REPO_OWNER="gustavosett"
REPO_NAME="Windows-11-Clipboard-History-For-Linux"
CLOUDSMITH_REPO="gustavosett/clipboard-manager"

DISTRO_ID=""
SYSTEM_FAMILY_INFO=""

# ---------------------------------------------------------------------------
# Step 0: Stop any running instances
# ---------------------------------------------------------------------------
stop_running_processes() {
    log "Stopping any running instances..."

    pkill -f "win11-clipboard-history-bin" 2>/dev/null || true
    pkill -f "win11-clipboard-history.AppImage" 2>/dev/null || true
    pkill -f "win11-clipboard-history" 2>/dev/null || true

    # Give processes a moment to exit, same pattern install.sh uses for AppImage
    local timeout=5 interval=1 elapsed=0
    while pgrep -f "win11-clipboard-history" >/dev/null 2>&1; do
        if [ "$elapsed" -ge "$timeout" ]; then
            warn "Timed out waiting for win11-clipboard-history processes to terminate."
            break
        fi
        sleep "$interval"
        elapsed=$((elapsed + interval))
    done

    success "Running instances stopped (if any were running)"
}

# ---------------------------------------------------------------------------
# Distro / arch detection (needed to pick the right package manager path)
# ---------------------------------------------------------------------------
detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        DISTRO_ID="${ID:-}"
        local id_like="${ID_LIKE:-}"
        SYSTEM_FAMILY_INFO=$(echo "$DISTRO_ID $id_like" | tr '[:upper:]' '[:lower:]')
    else
        warn "Cannot detect distribution (/etc/os-release not found). Will rely on command detection only."
    fi
}

# ---------------------------------------------------------------------------
# Step 1: Remove the installed package (APT/DNF/Zypper/AUR)
# Only removes the package that install.sh installs; does not touch unrelated packages.
# ---------------------------------------------------------------------------
remove_package() {
    log "Checking for installed package 'win11-clipboard-history'..."

    if command -v dpkg &>/dev/null && dpkg -s "win11-clipboard-history" &>/dev/null; then
        log "Removing APT package 'win11-clipboard-history'..."
        sudo apt-get remove -y "win11-clipboard-history" || warn "Failed to remove APT package (may already be gone)."
        sudo apt-get autoremove -y || true
        success "APT package removed"
        return 0
    fi

    if command -v rpm &>/dev/null && rpm -q "win11-clipboard-history" &>/dev/null; then
        if command -v dnf &>/dev/null; then
            log "Removing DNF package 'win11-clipboard-history'..."
            sudo dnf remove -y "win11-clipboard-history" || warn "Failed to remove DNF package (may already be gone)."
        elif command -v zypper &>/dev/null; then
            log "Removing Zypper package 'win11-clipboard-history'..."
            sudo zypper remove -y "win11-clipboard-history" || warn "Failed to remove Zypper package (may already be gone)."
        else
            log "Removing RPM package 'win11-clipboard-history' directly..."
            sudo rpm -e "win11-clipboard-history" || warn "Failed to remove RPM package (may already be gone)."
        fi
        success "RPM-based package removed"
        return 0
    fi

    if command -v pacman &>/dev/null && pacman -Qi "win11-clipboard-history-bin" &>/dev/null; then
        log "Removing AUR package 'win11-clipboard-history-bin'..."
        if command -v yay &>/dev/null; then
            yay -Rns --noconfirm "win11-clipboard-history-bin" || warn "Failed to remove AUR package."
        elif command -v paru &>/dev/null; then
            paru -Rns --noconfirm "win11-clipboard-history-bin" || warn "Failed to remove AUR package."
        else
            sudo pacman -Rns --noconfirm "win11-clipboard-history-bin" || warn "Failed to remove AUR package."
        fi
        success "AUR package removed"
        return 0
    fi

    log "No package-manager installation of 'win11-clipboard-history' found (may have been an AppImage install)."
}

# NOTE ON DEPENDENCIES:
# install.sh also installs xclip, wl-clipboard, acl, and
# libayatana-appindicator3-1 / libappindicator3-1 / libayatana-appindicator-gtk3.
# These are shared system libraries/utilities that may be relied upon by other
# applications. Since install.sh does not track whether they were already
# present before installation, they are intentionally NOT removed here to
# avoid breaking unrelated software. Remove them manually if you are certain
# they are unused elsewhere, e.g.:
#   sudo apt-get remove xclip wl-clipboard acl libayatana-appindicator3-1

# ---------------------------------------------------------------------------
# Step 2: Remove the Cloudsmith repository configuration
# ---------------------------------------------------------------------------
remove_cloudsmith_repo() {
    log "Removing Cloudsmith repository configuration (if present)..."

    # APT (Debian/Ubuntu) - Cloudsmith's setup.deb.sh typically drops a
    # sources.list.d file and a keyring file named after the repo.
    local apt_list="/etc/apt/sources.list.d/${CLOUDSMITH_REPO//\//-}.list"
    local apt_list_alt="/etc/apt/sources.list.d/gustavosett-clipboard-manager.list"
    local apt_keyring="/usr/share/keyrings/${CLOUDSMITH_REPO//\//-}-archive-keyring.gpg"
    local apt_keyring_alt="/usr/share/keyrings/gustavosett-clipboard-manager-archive-keyring.gpg"

    local apt_changed=false
    for f in "$apt_list" "$apt_list_alt" "$apt_keyring" "$apt_keyring_alt"; do
        if [ -f "$f" ]; then
            sudo rm -f "$f"
            apt_changed=true
        fi
    done
    if [ "$apt_changed" = true ]; then
        sudo apt-get update -qq || true
        success "APT Cloudsmith repository removed"
    fi

    # DNF/YUM (Fedora/RHEL/CentOS)
    local yum_repo="/etc/yum.repos.d/${CLOUDSMITH_REPO//\//-}.repo"
    local yum_repo_alt="/etc/yum.repos.d/gustavosett-clipboard-manager.repo"
    for f in "$yum_repo" "$yum_repo_alt"; do
        if [ -f "$f" ]; then
            sudo rm -f "$f"
            success "DNF/YUM Cloudsmith repository removed ($f)"
        fi
    done

    # Zypper (openSUSE)
    if command -v zypper &>/dev/null; then
        local zypper_repo_alias="gustavosett-clipboard-manager"
        if zypper lr 2>/dev/null | grep -qi "$zypper_repo_alias"; then
            sudo zypper removerepo "$zypper_repo_alias" || warn "Failed to remove Zypper repo (check 'zypper lr')."
            success "Zypper Cloudsmith repository removed"
        fi
    fi

    log "Cloudsmith repository cleanup complete (nothing found is also normal for AppImage installs)."
}

# ---------------------------------------------------------------------------
# Step 3: Remove AppImage-based installation artifacts
# ---------------------------------------------------------------------------
remove_appimage_installation() {
    log "Removing AppImage installation artifacts (if present)..."

    local found=false

    if [ -f "$HOME/.local/bin/win11-clipboard-history.AppImage" ]; then
        rm -f "$HOME/.local/bin/win11-clipboard-history.AppImage"
        found=true
    fi

    if [ -f "$HOME/.local/bin/win11-clipboard-history" ]; then
        rm -f "$HOME/.local/bin/win11-clipboard-history"
        found=true
    fi

    if [ -f "$HOME/.local/share/applications/win11-clipboard-history.desktop" ]; then
        rm -f "$HOME/.local/share/applications/win11-clipboard-history.desktop"
        found=true
    fi

    # Icon(s) installed under any icon size directory
    shopt -s nullglob
    local icon_files=("$HOME/.local/share/icons/hicolor"/*/apps/"win11-clipboard-history.png")
    shopt -u nullglob
    if [ "${#icon_files[@]}" -gt 0 ]; then
        rm -f "${icon_files[@]}"
        found=true
    fi

    if [ "$found" = true ]; then
        if command -v update-desktop-database &>/dev/null; then
            update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
        fi
        success "AppImage installation artifacts removed"
    else
        log "No AppImage installation artifacts found."
    fi
}

# ---------------------------------------------------------------------------
# Step 4: Remove udev rules and modules-load config for /dev/uinput (only the entries created by install.sh's setup_udev_appimage function)
# ---------------------------------------------------------------------------
remove_udev_config() {
    log "Removing uinput udev/module configuration (if present)..."

    local udev_rule="/etc/udev/rules.d/99-win11-clipboard-input.rules"
    local modules_conf="/etc/modules-load.d/win11-clipboard.conf"
    local changed=false

    if [ -f "$udev_rule" ]; then
        sudo rm -f "$udev_rule"
        changed=true
    fi

    if [ -f "$modules_conf" ]; then
        sudo rm -f "$modules_conf"
        changed=true
    fi

    if [ "$changed" = true ]; then
        sudo udevadm control --reload-rules 2>/dev/null || true
        sudo udevadm trigger --subsystem-match=misc 2>/dev/null || true
        success "udev rules and module-load configuration removed"
    else
        log "No udev/module configuration found."
    fi

    # NOTE: install.sh loads the uinput kernel module via `modprobe uinput`.
    # uinput is a common kernel module that may be used by other
    # applications (e.g. input remapping tools), so it is intentionally left
    # loaded rather than being unloaded here. Run `sudo modprobe -r uinput`
    # manually if you are certain nothing else depends on it.
}

# ---------------------------------------------------------------------------
# Step 5: Remove the ACL grant on /dev/uinput
# ---------------------------------------------------------------------------
remove_acl_permissions() {
    log "Removing ACL permissions on /dev/uinput (if present)..."

    if command -v setfacl &>/dev/null && [ -e /dev/uinput ]; then
        if getfacl /dev/uinput 2>/dev/null | grep -q "user:${USER}:rw-"; then
            sudo setfacl -x "u:${USER}" /dev/uinput 2>/dev/null || warn "Failed to remove ACL entry for /dev/uinput."
            success "ACL entry removed from /dev/uinput"
        else
            log "No matching ACL entry found on /dev/uinput."
        fi
    else
        log "setfacl not available or /dev/uinput does not exist; skipping."
    fi
}

# ---------------------------------------------------------------------------
# Step 6: Remove the application's config file
# ---------------------------------------------------------------------------
remove_application_folder() {
    log "Removing configuration file (if present)..."

    local config_folder="$HOME/.config/win11-clipboard-history"

    if [ -f "$config_folder" ]; then
        rm -rf "$config_folder"
        success "Configuration file removed ($config_folder)"
    else
        log "No configuration file found."
    fi
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
main() {
    echo ""
    echo "╔═══════════════════════════════════════════════════════════╗"
    echo "║     Win11 Clipboard History - Linux Uninstaller           ║"
    echo "╚═══════════════════════════════════════════════════════════╝"
    echo ""

    detect_distro
    log "Detected: ${DISTRO_ID:-unknown} (Family: ${SYSTEM_FAMILY_INFO:-unknown})"

    stop_running_processes
    remove_package
    remove_cloudsmith_repo
    remove_appimage_installation
    remove_udev_config
    remove_acl_permissions
    remove_application_folder

    echo ""
    success "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    success " Uninstallation complete."
    success " Shared dependencies (xclip, wl-clipboard, acl,"
    success " libayatana-appindicator*) were left installed since they"
    success " may be used by other applications."
    success "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
}

main "$@"
