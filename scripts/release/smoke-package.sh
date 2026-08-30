#!/usr/bin/env bash
# Verify archive integrity, safe paths/modes and the current-host capabilities command.
set -eu
[ "$#" -eq 1 ] || { echo "usage: $0 PACKAGE.tar.gz" >&2; exit 2; }
ARCHIVE=$1
case "$ARCHIVE" in *.tar.gz) ;; *) echo "expected .tar.gz archive" >&2; exit 2;; esac
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT HUP INT TERM
# Reject absolute paths and traversal before extraction.
tar -tzf "$ARCHIVE" | grep -Eq '(^/|(^|/)\.\.(/|$))' && { echo "unsafe archive path" >&2; exit 1; } || :
tar -xzf "$ARCHIVE" -C "$TMP" --no-same-owner --no-same-permissions
ROOT=$(find "$TMP" -mindepth 1 -maxdepth 1 -type d | head -1)
[ -n "$ROOT" ] && [ "$(find "$TMP" -mindepth 1 -maxdepth 1 | wc -l)" -eq 1 ]
[ "$(stat -c %a "$ROOT/bin/neko-cli")" = 755 ]
while IFS= read -r file; do
  [ "$(stat -c %a "$file")" = 644 ] || { echo "insecure document mode: $file" >&2; exit 1; }
done < <(find "$ROOT/share/doc/nekomusume" -type f -print)
(cd "$ROOT" && sha256sum -c SHA256SUMS)
case $(basename "$ROOT") in
  *-x86_64-unknown-linux-gnu) TARGET=x86_64-unknown-linux-gnu ;;
  *-aarch64-unknown-linux-gnu) TARGET=aarch64-unknown-linux-gnu ;;
  *) echo "unsupported package root" >&2; exit 1 ;;
esac
HOST=$(rustc -vV | sed -n 's/^host: //p')
[ "$TARGET" = "$HOST" ] || { echo "integrity/mode smoke passed; execution skipped for target=$TARGET host=$HOST"; exit 0; }
CAP=$($ROOT/bin/neko-cli capabilities --json)
printf '%s\n' "$CAP" | grep -q '"schema":"nekomusume.capabilities.v1"'
printf '%s\n' "$CAP" | grep -q '"secret_free":true'
echo "package_smoke_ok target=$TARGET"
