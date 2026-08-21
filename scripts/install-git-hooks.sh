#!/usr/bin/env bash
# Install this repository's git hooks (zero dependencies).
# نصب git hookهای این مخزن (بدون وابستگی).
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

mkdir -p .git/hooks
for hook in pre-commit commit-msg; do
  src="scripts/git-hooks/$hook"
  dst=".git/hooks/$hook"
  install -m 0755 "$src" "$dst"
  echo "✓ installed $dst / نصب شد"
done

cat <<'NOTE'

Hooks active / hookها فعال شدند:
  • pre-commit: ESLint + tsc on staged TS, cargo fmt --check on staged Rust
  • commit-msg: Conventional Commits (<type>(scope)?: <subject>)

NOTE
