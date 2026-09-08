#!/usr/bin/env bash
# Verify archive integrity, safe paths/modes and the current-host capabilities command.
set -eu
[ "$#" -eq 1 ] || { echo "usage: $0 PACKAGE.tar.gz" >&2; exit 2; }
ARCHIVE=$1
case "$ARCHIVE" in *.tar.gz) ;; *) echo "expected .tar.gz archive" >&2; exit 2;; esac
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT HUP INT TERM
# Validate names, member kinds, and the complete package shape before extraction.
mapfile -t MEMBERS < <(tar -tzf "$ARCHIVE" | sed 's:/$::')
[ "${#MEMBERS[@]}" -gt 0 ] || { echo "empty archive" >&2; exit 1; }
ROOT_NAME=${MEMBERS[0]}
case "$ROOT_NAME" in
  /*|..|../*|*/..|*/../*) echo "unsafe archive path" >&2; exit 1 ;;
esac
for member in "${MEMBERS[@]}"; do
  case "$member" in
    "$ROOT_NAME"|"$ROOT_NAME/bin"|"$ROOT_NAME/bin/neko-cli"|"$ROOT_NAME/share"|"$ROOT_NAME/share/doc"|"$ROOT_NAME/share/doc/nekomusume"|"$ROOT_NAME/share/doc/nekomusume/LICENSE-APACHE"|"$ROOT_NAME/share/doc/nekomusume/LICENSE-MIT"|"$ROOT_NAME/share/doc/nekomusume/README.txt"|"$ROOT_NAME/SHA256SUMS") ;;
    /*|*..*|*/*/*/*/*) echo "unexpected or unsafe archive path: $member" >&2; exit 1 ;;
    *) echo "unexpected archive member: $member" >&2; exit 1 ;;
  esac
done
EXPECTED=$(printf '%s\n' "$ROOT_NAME" "$ROOT_NAME/bin" "$ROOT_NAME/bin/neko-cli" "$ROOT_NAME/share" "$ROOT_NAME/share/doc" "$ROOT_NAME/share/doc/nekomusume" "$ROOT_NAME/share/doc/nekomusume/LICENSE-APACHE" "$ROOT_NAME/share/doc/nekomusume/LICENSE-MIT" "$ROOT_NAME/share/doc/nekomusume/README.txt" "$ROOT_NAME/SHA256SUMS" | sort)
ACTUAL=$(printf '%s\n' "${MEMBERS[@]}" | sort)
[ "$EXPECTED" = "$ACTUAL" ] || { echo "unexpected package layout" >&2; exit 1; }
# GNU tar's mode column starts with the member kind: only directories and regular files are allowed.
tar -tvzf "$ARCHIVE" | awk '$1 !~ /^[-d]/ {bad=1} END {exit bad}' || { echo "non-regular archive member" >&2; exit 1; }
tar -xzf "$ARCHIVE" -C "$TMP" --no-same-owner --no-same-permissions
ROOT="$TMP/$ROOT_NAME"
[ -d "$ROOT" ] && [ "$(find "$TMP" -mindepth 1 -maxdepth 1 | wc -l)" -eq 1 ]
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
