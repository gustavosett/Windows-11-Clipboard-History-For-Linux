#!/usr/bin/env bash
# Verify a built Debian artifact without installing it.
# راستی‌آزمایی artifact دبیان ساخته‌شده بدون نصب آن.
set -euo pipefail

package_path="${1:-}"
[[ -n "$package_path" && -f "$package_path" ]] || {
    echo "usage: $0 path/to/package.deb" >&2
    exit 2
}
command -v dpkg-deb >/dev/null || {
    echo "dpkg-deb is required / فرمان dpkg-deb لازم است" >&2
    exit 2
}

contents="$(dpkg-deb --contents "$package_path")"
control="$(dpkg-deb --field "$package_path")"

require_path() {
    grep -Eq "[.]/${1}$" <<<"$contents" || {
        echo "missing package path: /$1" >&2
        exit 1
    }
}

require_path 'usr/bin/windows-11-style-clipboard-history-manager'
grep -Eq '[.]/usr/(lib/windows-11-style-clipboard-history-manager/|bin/)windows-11-style-clipboard-history-manager-bin$' <<<"$contents" || {
    echo 'missing canonical binary' >&2
    exit 1
}
require_path 'usr/share/applications/io.github.mahdi-arts.clipboard-history.desktop'
require_path 'etc/udev/rules.d/99-windows-11-style-clipboard-history-input.rules'
grep -Fq 'Package: windows-11-style-clipboard-history-manager' <<<"$control"
grep -Fq 'Architecture:' <<<"$control"

printf 'Debian artifact verified: %s / بستهٔ دبیان تأیید شد: %s\n' "$package_path" "$package_path"
