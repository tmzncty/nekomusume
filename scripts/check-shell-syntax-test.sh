#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TMP=$(mktemp -d "${TMPDIR:-/tmp}/neko-shell.XXXXXX")
trap 'rm -rf "$TMP"' EXIT INT TERM
cp -a "$ROOT/scripts/." "$TMP/"
# checker resolves its own directory, so the copied fixture is self-contained
printf '%s\n' 'if (' > "$TMP/broken.sh"
chmod +x "$TMP/broken.sh"
before=$(git status --porcelain=v1 --untracked-files=all)
if (cd "$TMP" && ./check-shell-syntax.sh >/dev/null 2>&1); then
    printf '%s\n' 'malformed shell unexpectedly accepted' >&2
    exit 1
fi
after=$(git status --porcelain=v1 --untracked-files=all)
[ "$before" = "$after" ]
printf '%s\n' 'shell syntax mutation regression passed'
