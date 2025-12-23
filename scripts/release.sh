#!/bin/bash
# release.sh - Automated release script
# Usage: ./scripts/release.sh 0.5.0
#    or: make release VERSION=0.5.0

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

# Get version from argument
VERSION="${1:-}"

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.5.0"
    echo ""
    echo "Current versions:"
    echo "  package.json:      $(grep '"version"' package.json | head -1 | sed 's/.*: "\(.*\)".*/\1/')"
    echo "  tauri.conf.json:   $(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*: "\(.*\)".*/\1/')"
    echo "  Cargo.toml:        $(grep '^version' src-tauri/Cargo.toml | head -1 | sed 's/.*= "\(.*\)"/\1/')"
    exit 1
fi

# Validate version format (semver)
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
    error "Invalid version format: $VERSION (expected: X.Y.Z or X.Y.Z-suffix)"
fi

# Ask about release type
echo ""
echo "Select release type:"
echo "  1) Stable      - Full release for all users (default)"
echo "                   → GitHub Release + Cloudsmith + AUR"
echo "  2) Pre-release - Testing version for early adopters"
echo "                   → GitHub Release only (won't affect apt upgrade)"
echo ""
read -p "Release type [1/2]: " -n 1 -r RELEASE_TYPE
echo ""

case "$RELEASE_TYPE" in
    2)
        IS_PRERELEASE=true
        # Ask for pre-release suffix type
        echo ""
        echo "Pre-release suffix:"
        echo "  1) beta   - e.g., v${VERSION}-beta.1"
        echo "  2) rc     - e.g., v${VERSION}-rc.1 (release candidate)"
        echo "  3) alpha  - e.g., v${VERSION}-alpha.1"
        echo "  4) dev    - e.g., v${VERSION}-dev"
        read -p "Suffix type [1/2/3/4]: " -n 1 -r SUFFIX_TYPE
        echo ""
        
        case "$SUFFIX_TYPE" in
            2) SUFFIX="rc" ;;
            3) SUFFIX="alpha" ;;
            4) SUFFIX="dev" ;;
            *) SUFFIX="beta" ;;
        esac
        
        if [[ "$SUFFIX" == "dev" ]]; then
            VERSION="${VERSION}-dev"
        else
            read -p "${SUFFIX^} number (e.g., 1 for v${VERSION}-${SUFFIX}.1): " SUFFIX_NUM
            VERSION="${VERSION}-${SUFFIX}.${SUFFIX_NUM:-1}"
        fi
        
        warn "Creating PRE-RELEASE v$VERSION"
        warn "→ Will be uploaded to GitHub Releases ONLY"
        warn "→ Will NOT be pushed to Cloudsmith (no apt upgrade)"
        warn "→ Will NOT update AUR"
        ;;
    *)
        IS_PRERELEASE=false
        # Stable releases should NOT have pre-release suffixes
        if [[ "$VERSION" =~ -(alpha|beta|dev|rc) ]]; then
            error "Stable releases cannot have pre-release suffixes. Use version X.Y.Z"
        fi
        success "Creating STABLE release v$VERSION"
        log "→ GitHub Release + Cloudsmith + AUR"
        ;;
esac

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    warn "You have uncommitted changes:"
    git status --short
    echo ""
    read -p "Commit these changes before release? [Y/n] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        git add -A
        git commit -m "chore: prepare for v$VERSION release"
        success "Changes committed"
    else
        error "Please commit or stash changes before releasing"
    fi
fi

# Check we're on main/master branch
BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [[ "$BRANCH" != "main" && "$BRANCH" != "master" ]]; then
    warn "You're on branch '$BRANCH', not main/master"
    read -p "Continue anyway? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

log "Releasing version $VERSION..."

# Update package.json
log "Updating package.json..."
npm version "$VERSION" --no-git-tag-version --allow-same-version
success "package.json updated"

# Update tauri.conf.json
log "Updating tauri.conf.json..."
sed -i 's/"version": "[^"]*"/"version": "'"$VERSION"'"/' src-tauri/tauri.conf.json
success "tauri.conf.json updated"

# Update Cargo.toml (first occurrence only)
log "Updating Cargo.toml..."
sed -i '0,/^version = "[^"]*"/s//version = "'"$VERSION"'"/' src-tauri/Cargo.toml
success "Cargo.toml updated"

# Update AUR PKGBUILD (only for stable releases)
if [ "$IS_PRERELEASE" = true ]; then
    log "Skipping aur/PKGBUILD (pre-release)"
else
    log "Updating aur/PKGBUILD..."
    sed -i "s/^pkgver=.*/pkgver=$VERSION/" aur/PKGBUILD
    success "aur/PKGBUILD updated"
fi

# Update Cargo.lock
log "Updating Cargo.lock..."
cd src-tauri && cargo update -p win11-clipboard-history-lib --precise "$VERSION" 2>/dev/null || cargo check 2>/dev/null || true
cd ..
success "Cargo.lock updated"

# Show what changed
echo ""
log "Version changes:"
echo "  package.json:      $(grep '"version"' package.json | head -1 | sed 's/.*: "\(.*\)".*/\1/')"
echo "  tauri.conf.json:   $(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*: "\(.*\)".*/\1/')"
echo "  Cargo.toml:        $(grep '^version' src-tauri/Cargo.toml | head -1 | sed 's/.*= "\(.*\)"/\1/')"
echo "  aur/PKGBUILD:      $(grep '^pkgver=' aur/PKGBUILD | sed 's/pkgver=//')"
echo ""

# Commit version bump
log "Committing version bump..."
git add package.json package-lock.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock aur/PKGBUILD 2>/dev/null || true
git add -A
git commit -m "chore: bump version to $VERSION" || warn "Nothing to commit (version already set?)"

# Create tag
log "Creating tag v$VERSION..."
if git rev-parse "v$VERSION" >/dev/null 2>&1; then
    warn "Tag v$VERSION already exists!"
    read -p "Delete and recreate? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git tag -d "v$VERSION"
        git push origin --delete "v$VERSION" 2>/dev/null || true
    else
        error "Tag already exists. Aborting."
    fi
fi
git tag -a "v$VERSION" -m "Release v$VERSION"
success "Tag v$VERSION created"

# Push
log "Pushing to origin..."
git push origin "$BRANCH"
git push origin "v$VERSION"
success "Pushed to origin"

echo ""
if [ "$IS_PRERELEASE" = true ]; then
    echo -e "${YELLOW}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${YELLOW}║  🧪 Pre-release v$VERSION initiated!${NC}"
    echo -e "${YELLOW}╠════════════════════════════════════════════════════════════════╣${NC}"
    echo -e "${YELLOW}║  GitHub Actions will now:                                      ${NC}"
    echo -e "${YELLOW}║  • Build .deb, .rpm, and .AppImage                            ${NC}"
    echo -e "${YELLOW}║  • Create GitHub Pre-release                                  ${NC}"
    echo -e "${YELLOW}║                                                                ${NC}"
    echo -e "${YELLOW}║  ⚠️  Cloudsmith and AUR will NOT be updated                    ${NC}"
    echo -e "${YELLOW}║  ⚠️  Users won't receive this via apt upgrade                  ${NC}"
    echo -e "${YELLOW}╚════════════════════════════════════════════════════════════════╝${NC}"
else
    echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║  🚀 Stable release v$VERSION initiated!${NC}"
    echo -e "${GREEN}╠════════════════════════════════════════════════════════════════╣${NC}"
    echo -e "${GREEN}║  GitHub Actions will now:                                      ${NC}"
    echo -e "${GREEN}║  • Build .deb, .rpm, and .AppImage                            ${NC}"
    echo -e "${GREEN}║  • Create GitHub Release                                       ${NC}"
    echo -e "${GREEN}║  • Upload to Cloudsmith (enables apt upgrade)                  ${NC}"
    echo -e "${GREEN}║  • Update AUR package                                          ${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
fi
echo ""
echo "Track progress: https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/actions"
