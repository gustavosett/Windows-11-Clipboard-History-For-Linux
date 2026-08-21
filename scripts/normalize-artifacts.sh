#!/usr/bin/env bash
# Normalize bundle artifact filenames to the canonical lowercase package
# name so every distribution channel (GitHub Release, AUR PKGBUILD, the
# convenience installer, SHA256SUMS, docs) references identical names —
# regardless of how the Tauri bundler rendered `productName` per format.
# / نام فایل آرتیفکت‌ها را به نام بستهٔ رسمی و کوچک نرمال می‌کند تا همهٔ
# کانال‌های توزیع دقیقاً به یک نام اشاره کنند.
#
# Usage: scripts/normalize-artifacts.sh [bundle-dir] [version]
#   bundle-dir  defaults to src-tauri/target/release/bundle
#   version     defaults to the version in src-tauri/Cargo.toml
#
# Canonical names:
#   deb:      windows-11-style-clipboard-history-manager_<ver>_<amd64|arm64>.deb
#   AppImage: windows-11-style-clipboard-history-manager_<ver>_<amd64|arm64>.AppImage
#   rpm:      windows-11-style-clipboard-history-manager-<ver>-1.<x86_64|aarch64>.rpm
set -euo pipefail

readonly PKG="windows-11-style-clipboard-history-manager"
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
BUNDLE_DIR="${1:-$ROOT/src-tauri/target/release/bundle}"
VERSION="${2:-$(sed -n '0,/^version = "\([^"]*\)"/s//\1/p' "$ROOT/src-tauri/Cargo.toml")}"

if [[ -z "$VERSION" ]]; then
    echo "normalize-artifacts: could not determine version" >&2
    exit 1
fi
if [[ ! -d "$BUNDLE_DIR" ]]; then
    echo "normalize-artifacts: bundle dir not found: $BUNDLE_DIR" >&2
    exit 1
fi

map_deb_arch() {
    case "$1" in
        x86_64|amd64) echo amd64 ;;
        aarch64|arm64) echo arm64 ;;
        *) return 1 ;;
    esac
}

# Canonical target filename for a produced bundle file, or empty when the
# architecture cannot be derived. / نام هدفِ رسمی برای فایل تولیدشده.
canonical_name() {
    local lower="$1" ext="$2" arch deb_arch
    # Lowercase + spaces to hyphens: how the canonical name looks.
    lower="$(printf '%s' "$lower" | tr 'A-Z' 'a-z' | tr ' ' '-')"
    arch="$(printf '%s' "$lower" | sed -n 's/.*[._-]\(x86_64\|amd64\|aarch64\|arm64\)[._-].*/\1/p')"
    [[ -z "$arch" ]] && arch="$(printf '%s' "$lower" | sed -n 's/.*[._-]\(x86_64\|amd64\|aarch64\|arm64\)$/\1/p')"
    [[ -z "$arch" ]] && return 0

    case "$ext" in
        deb)
            deb_arch="$(map_deb_arch "$arch")" || return 0
            printf '%s\n' "${PKG}_${VERSION}_${deb_arch}.deb" ;;
        AppImage)
            deb_arch="$(map_deb_arch "$arch")" || return 0
            printf '%s\n' "${PKG}_${VERSION}_${deb_arch}.AppImage" ;;
        rpm)
            printf '%s\n' "${PKG}-${VERSION}-1.${arch}.rpm" ;;
    esac
}

renamed=0
for f in "$BUNDLE_DIR"/deb/*.deb \
         "$BUNDLE_DIR"/appimage/*.AppImage \
         "$BUNDLE_DIR"/rpm/*.rpm; do
    [[ -f "$f" ]] || continue
    base="$(basename "$f")"
    dir="$(dirname "$f")"
    ext="${base##*.}"
    target_name="$(canonical_name "$base" "$ext")"
    # Skip when nothing to do (idempotent on already-canonical names).
    # / اگر نام از قبل رسمی است رد می‌شود.
    if [[ -z "$target_name" || "$base" == "$target_name" ]]; then
        [[ -z "$target_name" ]] && \
            echo "normalize-artifacts: cannot derive arch from '$base'; leaving untouched" >&2
        continue
    fi
    mv -- "$f" "$dir/$target_name"
    echo "normalize-artifacts: $base  ->  $target_name"
    renamed=$((renamed + 1))
done

echo "normalize-artifacts: $renamed file(s) renamed to canonical '$PKG' names (version $VERSION)."
