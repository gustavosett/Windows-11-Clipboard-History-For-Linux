# 📦 Deployment Guide / راهنمای استقرار
## Windows 11 Style Clipboard History Manager

<div dir="rtl">

## راهنمای جامع بسته‌بندی و انتشار

این سند راهنمای کامل برای بسته‌بندی و انتشار برنامه در قالب‌های مختلف است.

</div>

---

## Table of Contents / فهرست مطالب

0. [Packaging architecture / معماری بسته‌بندی](#packaging-architecture)
1. [Prerequisites / پیش‌نیازها](#prerequisites)
2. [Building / ساخت](#building)
3. [Debian/Ubuntu (.deb)](#debianubuntu-deb)
4. [Fedora/RHEL (.rpm)](#fedorarhel-rpm)
5. [AppImage (.AppImage)](#appimage-appimage)
6. [Flatpak (.flatpak)](#flatpak-flatpak)
7. [Arch Linux (AUR)](#arch-linux-aur)
8. [Release Checklist / چک‌لیست انتشار](#release-checklist)

---

## Packaging architecture

There are deliberately **two** native packaging paths. They serve different
audiences and must stay in lockstep — `scripts/check-packaging.sh`
(structurally enforced in CI) guarantees they install the same system files.

| Path / مسیر | Audience | Output | Owner |
| --- | --- | --- | --- |
| **Tauri bundle** (`src-tauri/tauri.conf.json` → `bundle.linux`) | GitHub Releases | `.deb`, `.rpm`, `.AppImage` (amd64 + arm64) | `release.yml` workflow |
| **distro packaging** (`packaging/debian/`, `packaging/rpm/`) | Debian archives / PPAs / distro maintainers | source-built `.deb`/`.rpm` via `debhelper`/`rpmbuild` | maintainers |

**Rule of thumb / قاعدهٔ کلی:** user-facing GitHub Releases always come from
the Tauri bundle path; the `packaging/` tree exists for distribution
integration. Never let the two drift — the packaging gate fails the build if
the installed file sets diverge.

**قاعده:** انتشار‌های گیت‌هاب همیشه از مسیر Tauri bundle ساخته می‌شوند؛
درخت `packaging/` برای یکپارچه‌سازی با توزیع‌هاست. این دو نباید از هم
فاصله بگیرند — اگر مجموعهٔ فایل‌های نصبی واگرا شود، گیت بسته‌بندی بیلد
را متوقف می‌کند.

---


## Prerequisites / پیش‌نیازها

<div dir="rtl">

### پیش‌نیازهای عمومی

قبل از شروع بسته‌بندی، مطمئن شوید که موارد زیر نصب هستند:

</div>

```bash
# Common build dependencies
sudo apt install -y \
  build-essential \
  cargo \
  rustc \
  nodejs \
  npm \
  pkg-config \
  libwebkit2gtk-4.1-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libxdo-dev \
  libgtk-3-dev \
  libglib2.0-dev \
  desktop-file-utils \
  appstream
```

<div dir="rtl">

### پیش‌نیازهای Debian/Ubuntu

</div>

```bash
# Debian-specific
sudo apt install -y \
  debhelper \
  devscripts \
  fakeroot \
  lintian
```

<div dir="rtl">

### پیش‌نیازهای Fedora/RHEL

</div>

```bash
# Fedora/RHEL-specific
sudo dnf install -y \
  rpm-build \
  make \
  gcc \
  gcc-c++ \
  rustfmt
```

<div dir="rtl">

### پیش‌نیازهای Flatpak

</div>

```bash
# Flatpak
sudo apt install -y \
  flatpak \
  flatpak-builder
```

---

## Building / ساخت

<div dir="rtl">

### ساخت باینری

</div>

```bash
# Clone and setup
git clone https://github.com/Mahdi-Arts/Windows-11-Style-Clipboard-History-Manager.git
cd Windows-11-Style-Clipboard-History-Manager

# Install Node.js dependencies
npm ci

# Build Tauri application
npm run tauri:build

# Verify build
ls -lh src-tauri/target/release/bundle/
```

<div dir="rtl">

### ساخت برای پلتفرم‌های مختلف

</div>

```bash
# Release binary path
BINARY=src-tauri/target/release/windows-11-style-clipboard-history-manager-bin

# Verify binary
file "$BINARY"
"$BINARY" --version
"$BINARY" --help
```

---

## Debian/Ubuntu (.deb)

<div dir="rtl">

### روش ۱: Tauri Bundler (توصیه‌شده)

</div>

```bash
# Build with Tauri bundler
npm run tauri:build

# Package is automatically created
ls -lh src-tauri/target/release/bundle/deb/*.deb
```

<div dir="rtl">

### روش ۲: مستقیم با dpkg-buildpackage

</div>

```bash
# Build the .deb directly
dpkg-buildpackage -us -uc -b

# Check the package
ls -lh ../*.deb
lintian --pedantic ../*.changes
```

<div dir="rtl">

### نصب و آزمون

</div>

```bash
# Install
sudo apt install ./path/to/windows-11-style-clipboard-history-manager_*.deb

# Verify installation
which windows-11-style-clipboard-history-manager
windows-11-style-clipboard-history-manager --version

# Test on a clean system (VM recommended)
sudo apt install ./windows-11-style-clipboard-history-manager_*.deb

# Cleanup
sudo apt remove windows-11-style-clipboard-history-manager
```

<div dir="rtl">

### نکات مهم Debian

</div>

| موضوع | توضیح |
|-------|-------|
| **udev rules** | قوانین udev با `TAG+="uaccess"` نصب می‌شوند |
| **ACL permissions** | اسکریپت postinst مجوزها را تنظیم می‌کند |
| **AppArmor** | پروفایل در حالت complain نصب می‌شود |

---

## Fedora/RHEL (.rpm)

<div dir="rtl">

### ساخت RPM

</div>

```bash
# Method 1: Using the spec file
rpmbuild -bb packaging/rpm/windows-11-style-clipboard-history-manager.spec

# Method 2: Using Tauri bundler (if supported)
npm run tauri:build -- --bundles rpm

# Find the package
ls -lh ~/rpmbuild/RPMS/x86_64/*.rpm
```

<div dir="rtl">

### نصب

</div>

```bash
# Install
sudo dnf install ./path/to/windows-11-style-clipboard-history-manager-*.rpm

# Verify
rpm -qi windows-11-style-clipboard-history-manager
```

---

## AppImage (.AppImage)

<div dir="rtl">

### Tauri Bundler

</div>

```bash
# Build AppImage
npm run tauri:build -- --bundles appimage

# Find the package
ls -lh src-tauri/target/release/bundle/appimage/*.AppImage

# Make executable
chmod +x *.AppImage

# Run
./windows-11-style-clipboard-history-manager_2.5.0_amd64.AppImage
```

<div dir="rtl">

### نکات AppImage

</div>

| موضوع | توضیح |
|-------|-------|
| **NVIDIA workaround** | `IS_APPIMAGE=1` برای GPU انویدیا |
| **Portable** | نیازی به نصب ندارد |
| **udev** | قوانین udev نصب نمی‌شوند |

---

## Flatpak (.flatpak)

<div dir="rtl">

### ساخت Flatpak

</div>

```bash
# Add Flathub repository (if not added)
flatpak remote-add --if-not-exists flathub \
  https://flathub.org/repo/flathub.flatpakrepo

# Build from manifest
cd packaging/flatpak
./build.sh

# Or manually
flatpak-builder --force-clean --user --install build-dir \
  io.github.mahdi-arts.clipboard-history.yml
```

<div dir="rtl">

### محدودیت‌های Flatpak

</div>

<div dir="rtl">

⚠️ **مهم**: سیاست Flatpak دسترسی به `/dev/uinput` را نمی‌دهد. بنابراین عملیات paste شبیه‌سازی‌شده ممکن است کار نکند. برای عملکرد کامل، از بسته‌های `.deb` یا `.rpm` استفاده کنید.

</div>

```bash
# For history viewing (works)
flatpak run io.github.mahdi-arts.clipboard-history

# For paste simulation (may not work)
# Enable network for GIF search
flatpak override --user --share=network io.github.mahdi-arts.clipboard-history

# For full paste (NOT recommended - security risk)
flatpak override --user --device=all io.github.mahdi-arts.clipboard-history
```

<div dir="rtl">

### لینت و ارسال به Flathub

</div>

The release pipeline lints both the manifest and the AppStream metainfo with
`flatpak-builder-lint` (Flathub's own rules) before publishing. Run the same
checks locally before any Flathub submission:

```bash
# Flathub-rule linting of the manifest and AppStream metadata
flatpak-builder-lint manifest packaging/flatpak/io.github.mahdi-arts.clipboard-history.yml
flatpak-builder-lint appstream packaging/flatpak/io.github.mahdi-arts.clipboard-history.metainfo.xml

# End-to-end export + validation of the built bundle
flatpak-builder --force-clean --repo=repo build-dir \
  packaging/flatpak/io.github.mahdi-arts.clipboard-history.yml
flatpak build-bundle repo \
  io.github.mahdi-arts.clipboard-history.flatpak \
  io.github.mahdi-arts.clipboard-history
flatpak install --user io.github.mahdi-arts.clipboard-history.flatpak
```

To submit to Flathub, open a pull request against
`flathub/flathub` (or `flathub/io.github.mahdi-arts.clipboard-history` when
the app is accepted) containing the manifest, the metainfo, and the icon set
from `packaging/flatpak/`. The Flathub bot runs the same
`flatpak-builder-lint` gates in CI.

<div dir="rtl">

خط لولهٔ انتشار پیش از انتشار، manifest و متادیتای AppStream را با
`flatpak-builder-lint` (قوانین خود Flathub) لینت می‌کند. همین بررسی‌ها را
پیش از هر ارسال به Flathub به‌صورت محلی اجرا کنید (دستورهای بالا).
برای ارسال به Flathub، یک PR شامل manifest، متادیتا و آیکون‌ها از
`packaging/flatpak/` به مخزن `flathub/flathub` بزنید؛ ربات Flathub همان
گیت‌های `flatpak-builder-lint` را در CI اجرا می‌کند.

</div>

---

## Arch Linux (AUR)

<div dir="rtl">

### نصب از AUR

</div>

```bash
# Using yay
yay -S windows-11-style-clipboard-history-manager-bin

# Or manually
git clone https://aur.archlinux.org/windows-11-style-clipboard-history-manager-bin.git
cd windows-11-style-clipboard-history-manager-bin
makepkg -si
```

---

## Release Checklist / چک‌لیست انتشار

<div dir="rtl">

قبل از انتشار هر نسخه، موارد زیر را بررسی کنید:

</div>

### Pre-Release / پیش از انتشار

```bash
# Quality gates
npm run lint
npm run test:coverage
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --all-features
cargo audit
cargo deny check advisories bans licenses sources
npm audit --audit-level=high

# Build all targets
npm run tauri:build
```

### Integrity / یکپارچگی

```bash
# Generate checksums
cd src-tauri/target/release/bundle
sha256sum deb/*.deb rpm/*.rpm appimage/*.AppImage > SHA256SUMS

# Verify checksums
sha256sum -c SHA256SUMS

# Optional GPG signing
gpg --armor --detach-sign SHA256SUMS
```

### GitHub Release / انتشار GitHub

```bash
# Create and push tag
git tag -a v2.5.0 -m "Release v2.5.0"
git push origin v2.5.0

# Upload artifacts to GitHub Release
# - .deb
# - .rpm  
# - .AppImage
# - SHA256SUMS
# - SHA256SUMS.sig (if signed)
# - SBOM per artifact
# - Provenance attestations
```

### Post-Release / پس از انتشار

```bash
# Verify release
# - Check all download links work
# - Verify signatures
# - Update AUR PKGBUILD checksums
# - Update Flatpak manifest version
# - Update CHANGELOG.md
# - Announce on social media/discussion
```

---

## Troubleshooting / عیب‌یابی

<div dir="rtl">

### مشکلات رایج

</div>

| مشکل | راه‌حل |
|------|--------|
| `Permission denied` on `/dev/uinput` | Log out and log back in |
| `AppImage` not running | `chmod +x` and run with `IS_APPIMAGE=1` |
| Flatpak paste not working | Use `.deb`/`.rpm` instead |
| Symbol fonts missing | Bundled fonts should be used |

---

## Security Notes / نکات امنیتی

<div dir="rtl">

### تأیید بسته‌ها

</div>

```bash
# Always verify checksums
sha256sum -c SHA256SUMS

# Verify GPG signature (if available)
gpg --verify SHA256SUMS.sig SHA256SUMS

# Check package signatures
dpkg-sig --verify *.deb
rpm --checksig *.rpm
```

<div dir="rtl">

### توصیه‌های امنیتی

</div>

- ✅ Always download from official GitHub releases
- ✅ Always verify checksums
- ✅ Prefer signed releases
- ✅ Use official repositories when available
- ❌ Never run `curl | bash` installers without review

---

## License / مجوز

MIT License - See [LICENSE](../LICENSE)

---

**Version / نسخه**: 2.5.0  
**Last Updated / آخرین به‌روزرسانی**: 2026-08-21
