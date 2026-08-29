#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TMP=$(mktemp -d "$ROOT/.link-test.XXXXXX")
trap 'rm -rf "$TMP"' EXIT INT TERM
printf '%s\n' '[ok](README.md)' > "$TMP/valid.md"
cp "$ROOT/README.md" "$TMP/README.md"
./scripts/check-markdown-links.sh "$TMP/valid.md" >/dev/null
for target in missing.md /tmp/absolute.md ../../escape.md 'bad`tick.md'; do
 printf '[bad](%s)\n' "$target" > "$TMP/bad.md"
 if ./scripts/check-markdown-links.sh "$TMP/bad.md" >/dev/null 2>&1; then exit 1; fi
done
printf '%s\n' 'markdown link mutation regression passed'
