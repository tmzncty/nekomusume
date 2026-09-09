#!/usr/bin/env bash
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT HUP INT TERM
git -C "$ROOT" archive HEAD | tar -x -C "$TMP"
# Exercise the current slice rather than the pre-change HEAD archived above.
cp "$ROOT/.gitignore" "$TMP/.gitignore"
cp "$ROOT/scripts/release/check-clean-source.sh" "$TMP/scripts/release/check-clean-source.sh"
git -C "$TMP" init -q
git -C "$TMP" config user.email test@example.invalid
git -C "$TMP" config user.name test
git -C "$TMP" add .
git -C "$TMP" commit -qm base
CHECK="$ROOT/scripts/release/check-clean-source.sh"
"$CHECK" "$TMP"
mkdir -p "$TMP/target/existing" "$TMP/dist"
: > "$TMP/target/existing/output"
: > "$TMP/dist/old.tar.gz"
"$CHECK" "$TMP"
printf '\n# dirty\n' >> "$TMP/README.md"
if "$CHECK" "$TMP" >/dev/null 2>&1; then echo 'accepted unstaged tracked change' >&2; exit 1; fi
git -C "$TMP" checkout -q -- README.md
printf '\n# staged\n' >> "$TMP/README.md"
git -C "$TMP" add README.md
if "$CHECK" "$TMP" >/dev/null 2>&1; then echo 'accepted staged change' >&2; exit 1; fi
git -C "$TMP" reset -q --hard HEAD
: > "$TMP/untracked-source"
if "$CHECK" "$TMP" >/dev/null 2>&1; then echo 'accepted non-ignored untracked file' >&2; exit 1; fi
rm "$TMP/untracked-source"
"$CHECK" "$TMP"
echo package-clean-source-regressions-ok
