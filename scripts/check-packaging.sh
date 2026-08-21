#!/usr/bin/env bash
# Validate release-package contracts without building a GUI bundle.
# اعتبارسنجی قراردادهای بستهٔ انتشار بدون ساخت bundle گرافیکی.
set -euo pipefail

readonly ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

readonly BINARY="windows-11-style-clipboard-history-manager-bin"
readonly WRAPPER="windows-11-style-clipboard-history-manager"
readonly APP_ID="io.github.mahdi-arts.clipboard-history"
# Exported so the version-drift guard (node) can resolve the metainfo path.
# Export می‌شود تا گارد رانش نسخه (node) بتواند مسیر metainfo را بیابد.
export APP_ID

fail() {
    printf 'packaging check failed: %s\n' "$*" >&2
    exit 1
}

[[ "$(sed -n '/^\[\[bin\]\]/,/^$/s/^name = "\([^"]*\)"/\1/p' src-tauri/Cargo.toml)" == "$BINARY" ]] \
    || fail "Cargo binary must be $BINARY"

grep -Fq "src-tauri/target/release/$BINARY" packaging/debian/rules \
    || fail "Debian rules do not install the canonical binary"
grep -Fq "src-tauri/target/release/$BINARY" packaging/flatpak/$APP_ID.yml \
    || fail "Flatpak manifest does not install the canonical binary"
grep -Fq "/usr/lib/$WRAPPER/$BINARY" src-tauri/bundle/linux/wrapper.sh \
    || fail "wrapper does not search the Debian/RPM library path"
grep -Fq "/app/lib/$WRAPPER/$BINARY" src-tauri/bundle/linux/wrapper.sh \
    || fail "wrapper does not search the Flatpak library path"
grep -Fq 'Icon=io.github.mahdi-arts.clipboard-history' \
    src-tauri/bundle/linux/windows-11-style-clipboard-history-manager.desktop \
    || fail "desktop entry and installed icon ID have drifted"
grep -Fq 'windows-11-style-clipboard-history-manager.desktop' Makefile \
    || fail "Makefile references a missing desktop source"

for required in \
    src-tauri/bundle/linux/wrapper.sh \
    src-tauri/bundle/linux/windows-11-style-clipboard-history-manager.desktop \
    src-tauri/bundle/linux/99-windows-11-style-clipboard-history-input.rules \
    packaging/flatpak/$APP_ID.yml \
    packaging/flatpak/$APP_ID.metainfo.xml \
    packaging/debian/control \
    packaging/debian/rules; do
    [[ -f "$required" ]] || fail "missing $required"
done

# ---------------------------------------------------------------------------
# Release-engineering regression guards / گاردهای بازگشتِ مهندسی انتشار
# ---------------------------------------------------------------------------
# The 2026 rename left stale identifiers behind once; these guards make any
# future drift a hard build failure instead of a broken release.
# تغییر نام ۲۰۲۶ یک‌بار شناسه‌های قدیمی جا ماند؛ این گاردها هر drift آینده را
# به‌جای انتشار خراب، به شکست سخت build تبدیل می‌کنند.

# 1. No legacy project/package names may survive in CI/release pipelines.
#    هیچ نام قدیمی پروژه/بسته نباید در خطوط لولهٔ CI/انتشار باقی بماند.
if grep -rIn --include='*.yml' -e 'win11-clipboard-history' -e 'Modern-Clipboard-History' .github/workflows; then
    fail "legacy project name found in .github/workflows"
fi

# 2. The CI smoke test must exercise the canonical binary.
#    تست smoke ی CI باید باینری رسمی را اجرا کند.
grep -Fq "src-tauri/target/release/$BINARY" .github/workflows/ci.yml \
    || fail "ci.yml smoke test does not reference $BINARY"

# 3. AUR package name must be consistent across PKGBUILD and the release job.
#    نام بستهٔ AUR باید در PKGBUILD و job انتشار یکسان باشد.
grep -Fq "pkgname=$WRAPPER-bin" aur/PKGBUILD \
    || fail "aur/PKGBUILD pkgname is not $WRAPPER-bin"
grep -Fq "aur.archlinux.org/$WRAPPER-bin.git" .github/workflows/release.yml \
    || fail "release.yml does not clone the $WRAPPER-bin AUR package"

# 4. deb and rpm bundles must install the same system files (wrapper, udev
#    rules, desktop entry, icons, AppArmor profile). Implemented in node so
#    the JSON comparison is structural, not textual.
#    بسته‌های deb و rpm باید فایل‌های سیستمی یکسانی نصب کنند؛ مقایسه با node
#    به‌صورت ساختاری انجام می‌شود، نه متنی.
node --input-type=module - <<'PKG_NODE_EOF'
import { readFileSync } from 'node:fs'

const conf = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8'))
const debFiles = Object.keys(conf.bundle?.linux?.deb?.files ?? {}).sort()
const rpmFiles = Object.keys(conf.bundle?.linux?.rpm?.files ?? {}).sort()

if (debFiles.length === 0) {
  console.error('packaging check failed: deb bundle has no custom files')
  process.exit(1)
}
if (JSON.stringify(debFiles) !== JSON.stringify(rpmFiles)) {
  console.error('packaging check failed: rpm bundle files are not at parity with deb')
  console.error('  deb:', debFiles)
  console.error('  rpm:', rpmFiles)
  process.exit(1)
}

const targets = conf.bundle?.targets ?? []
for (const required of ['deb', 'rpm', 'appimage']) {
  if (!targets.includes(required)) {
    console.error(`packaging check failed: bundle.targets is missing "${required}"`)
    process.exit(1)
  }
}
PKG_NODE_EOF

# ---------------------------------------------------------------------------
# Single-source version drift guard / گارد رانش نسخه از منبع واحد
# ---------------------------------------------------------------------------
# The version must be identical across the Cargo manifest, the frontend
# package manifest, the Tauri bundle config, the Debian changelog, the Cargo
# lockfile, and the AppStream metainfo `<releases>` tag. A mismatch here means
# a release would publish with an internally inconsistent version.
# نسخه باید در manifest Cargo، manifest فرانت‌اند، کانفیگ bundle تائوری،
# changelog دبیان، lockfile کارگو و تگ `<releases>` متادیتای AppStream یکسان
# باشد. ناهماهنگی یعنی انتشار با نسخهٔ ناسازگار درونی منتشر می‌شود.
node --input-type=module - <<'VER_NODE_EOF'
import { readFileSync } from 'node:fs'

const pkg = JSON.parse(readFileSync('package.json', 'utf8')).version
const cargo = /^version = "([^"]+)"/m.exec(readFileSync('src-tauri/Cargo.toml', 'utf8'))?.[1]
const tauri = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8')).version
const lock = (() => {
  const text = readFileSync('src-tauri/Cargo.lock', 'utf8')
  const m = text.match(
    /name = "windows-11-style-clipboard-history-manager"\nversion = "([^"]+)"/
  )
  return m?.[1]
})()
// The Debian version carries an upstream revision suffix (`2.5.0-1`);
// compare only the upstream portion for consistency.
// نسخهٔ دبیان سافیکس تجدیدنظر دارد (`2.5.0-1`)؛ فقط بخش بالادستی مقایسه می‌شود.
const changelog = /\(([^)]+)\)/
  .exec(readFileSync('packaging/debian/changelog', 'utf8'))?.[1]
  ?.split('-')[0]
const metainfo = /<release version="([^"]+)"/.exec(
  readFileSync(`packaging/flatpak/${process.env.APP_ID}.metainfo.xml`, 'utf8')
)?.[1]

const versionSources = { 'package.json': pkg, 'Cargo.toml': cargo, 'tauri.conf.json': tauri, 'Cargo.lock': lock, 'debian/changelog': changelog, 'metainfo': metainfo }
const values = Object.entries(versionSources)
const canonical = values[0][1]
for (const [label, value] of values) {
  if (!value) {
    console.error(`packaging check failed: version not found in ${label}`)
    process.exit(1)
  }
  if (value !== canonical) {
    console.error(`packaging check failed: version drift — ${canonical} in ${values[0][0]} but ${value} in ${label}`)
    process.exit(1)
  }
}
console.log(`Version consistent at ${canonical}. / نسخه در ${canonical} سازگار است.`)
VER_NODE_EOF

if command -v desktop-file-validate >/dev/null 2>&1; then
    desktop-file-validate src-tauri/bundle/linux/windows-11-style-clipboard-history-manager.desktop
fi
if command -v appstreamcli >/dev/null 2>&1; then
    appstreamcli validate --no-net packaging/flatpak/$APP_ID.metainfo.xml
fi

printf 'Packaging contracts are consistent. / قراردادهای بسته‌بندی سازگارند.\n'
