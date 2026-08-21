# Packaging and Release Guide / راهنمای بسته‌بندی و انتشار

> Canonical application ID: `io.github.mahdi-arts.clipboard-history`
> Canonical binary: `windows-11-style-clipboard-history-manager-bin`
> Canonical launcher: `windows-11-style-clipboard-history-manager`

---

<div dir="rtl">

## بخش فارسی

### هدف و ترتیب انتشار

مسیر رسمی انتشار به این ترتیب است:

1. کنترل کیفیت و امنیت؛
2. ساخت و آزمون بستهٔ Debian (`.deb`) برای GitHub Release؛
3. انتشار checksum، SBOM و provenance؛
4. آزمون بسته روی Ubuntu/Debian تمیز؛
5. ساخت Flatpak محلی؛
6. آماده‌سازی manifest برای ارسال جداگانه به Flathub.

### پیش‌نیازهای Debian/Ubuntu

```bash
sudo apt update
sudo apt install -y \
  build-essential debhelper devscripts fakeroot lintian \
  nodejs npm cargo rustc pkg-config \
  libwebkit2gtk-4.1-dev libssl-dev \
  libayatana-appindicator3-dev librsvg2-dev \
  libxdo-dev libgtk-3-dev libglib2.0-dev \
  desktop-file-utils appstream
```

### کنترل کیفیت پیش از ساخت

```bash
npm ci
npm run lint
npm run test:coverage
npm run build
scripts/check-packaging.sh
npm audit --audit-level=high
```

در محیط دارای Rust:

```bash
cd src-tauri
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --all-features
cargo audit
cargo deny check advisories bans licenses sources
```

### ساخت بستهٔ `.deb`

روش اصلی برای GitHub Release، bundler رسمی Tauri است:

```bash
npm run tauri:build
ls -lh src-tauri/target/release/bundle/deb/*.deb
```

روش کنترل مستقل با Debian tooling:

```bash
dpkg-buildpackage -us -uc -b
lintian --pedantic ../windows-11-style-clipboard-history-manager_*.changes
```

### آزمون نصب

آزمون را روی VM یا کانتینر تمیز اجرا کنید؛ نصب روی ماشین توسعه معیار انتشار نیست.

```bash
sudo apt install ./src-tauri/target/release/bundle/deb/*.deb
command -v windows-11-style-clipboard-history-manager
/usr/lib/windows-11-style-clipboard-history-manager/windows-11-style-clipboard-history-manager-bin --version
sudo apt remove windows-11-style-clipboard-history-manager
```

عمل paste به `/dev/uinput` نیاز دارد. اسکریپت post-install ماژول را بارگذاری می‌کند و udev rule دارای `TAG+="uaccess"` را نصب می‌کند. ACL گسترده یا mode برابر `0666` مجاز نیست.

### امضای خروجی انتشار

```bash
cd src-tauri/target/release/bundle
find . -type f \( -name '*.deb' -o -name '*.rpm' -o -name '*.AppImage' \) \
  -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS
sha256sum -c SHA256SUMS
```

GitHub Actions باید برای هر artifact، SPDX SBOM و attestation تولید کند. secretهای امضا یا توزیع در log چاپ نمی‌شوند.

### Flatpak

Flatpak را پس از تثبیت `.deb` بسازید:

```bash
sudo apt install flatpak flatpak-builder
flatpak remote-add --if-not-exists flathub \
  https://flathub.org/repo/flathub.flatpakrepo
cd packaging/flatpak
./build.sh
```

محدودیت مهم: سیاست Flathub دسترسی `/dev/uinput` را نمی‌دهد. بنابراین نسخهٔ Flatpak برای مشاهده و کپی تاریخچه مناسب است، اما paste شبیه‌سازی‌شده ممکن است در دسترس نباشد. دادن `--device=all` توصیهٔ پیش‌فرض پروژه نیست.

### ساختار فایل‌ها

```text
packaging/
├── debian/                  # metadata و scriptهای Debian
├── flatpak/                 # manifest، AppStream و build helper
├── apparmor/                # profile اختیاری دفاع عمقی
└── README.md
src-tauri/bundle/linux/
├── wrapper.sh               # launcher مشترک deb/rpm/Flatpak
├── windows-11-style-clipboard-history-manager.desktop
├── 99-windows-11-style-clipboard-history-input.rules
├── postinst.sh
└── postrm.sh
```

</div>

---

## English section

### Release order

The supported release path is:

1. quality and security gates;
2. build/test the Debian package for GitHub Releases;
3. publish checksums, SBOMs, and provenance;
4. install-test on clean Ubuntu/Debian systems;
5. build Flatpak locally;
6. prepare a separate immutable-source manifest for Flathub submission.

### Debian/Ubuntu prerequisites

```bash
sudo apt update
sudo apt install -y \
  build-essential debhelper devscripts fakeroot lintian \
  nodejs npm cargo rustc pkg-config \
  libwebkit2gtk-4.1-dev libssl-dev \
  libayatana-appindicator3-dev librsvg2-dev \
  libxdo-dev libgtk-3-dev libglib2.0-dev \
  desktop-file-utils appstream
```

### Pre-build gates

```bash
npm ci
npm run lint
npm run test:coverage
npm run build
scripts/check-packaging.sh
npm audit --audit-level=high

cd src-tauri
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --all-features
cargo audit
cargo deny check advisories bans licenses sources
```

### Build and test `.deb`

The primary GitHub Release path uses Tauri's bundler:

```bash
npm run tauri:build
ls -lh src-tauri/target/release/bundle/deb/*.deb
```

The independent Debian tooling path is:

```bash
dpkg-buildpackage -us -uc -b
lintian --pedantic ../windows-11-style-clipboard-history-manager_*.changes
```

Install-test on a clean VM/container:

```bash
sudo apt install ./src-tauri/target/release/bundle/deb/*.deb
command -v windows-11-style-clipboard-history-manager
/usr/lib/windows-11-style-clipboard-history-manager/windows-11-style-clipboard-history-manager-bin --version
sudo apt remove windows-11-style-clipboard-history-manager
```

Paste simulation requires `/dev/uinput`. Packaging installs a `uaccess` udev rule; world-writable `0666` device permissions are forbidden.

### Release integrity

```bash
cd src-tauri/target/release/bundle
find . -type f \( -name '*.deb' -o -name '*.rpm' -o -name '*.AppImage' \) \
  -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS
sha256sum -c SHA256SUMS
```

CI publishes an SPDX SBOM and provenance attestation for each artifact. Signing/distribution secrets must never appear in logs.

### Flatpak

Build Flatpak only after the Debian path is stable:

```bash
sudo apt install flatpak flatpak-builder
flatpak remote-add --if-not-exists flathub \
  https://flathub.org/repo/flathub.flatpakrepo
cd packaging/flatpak
./build.sh
```

Flathub policy does not expose `/dev/uinput`. The Flatpak remains useful for history and copy workflows, but simulated paste may be unavailable. The project does not recommend `--device=all` as a default.

### Contract validation

`scripts/check-packaging.sh` prevents binary/path drift across Cargo, Debian, Flatpak, and the launcher. Run it locally and in every release workflow.
