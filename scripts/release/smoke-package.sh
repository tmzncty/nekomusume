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
  nekomusume-*-x86_64-unknown-linux-gnu|nekomusume-*-aarch64-unknown-linux-gnu) ;;
  *) echo "unsupported or unsafe package root" >&2; exit 1 ;;
esac
case "$ROOT_NAME" in
  */*|.|..|*'/./'*|*'/../'*) echo "unsafe package root" >&2; exit 1 ;;
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
# Validate archive kinds and exact builder modes before extraction; umask must not normalize bad metadata into acceptance.
python3 - "$ARCHIVE" "$ROOT_NAME" <<'PY'
import stat, sys, tarfile
archive, root = sys.argv[1:]
expected = {
    root: (stat.S_IFDIR, 0o755),
    f"{root}/bin": (stat.S_IFDIR, 0o755),
    f"{root}/bin/neko-cli": (stat.S_IFREG, 0o755),
    f"{root}/share": (stat.S_IFDIR, 0o755),
    f"{root}/share/doc": (stat.S_IFDIR, 0o755),
    f"{root}/share/doc/nekomusume": (stat.S_IFDIR, 0o755),
    f"{root}/share/doc/nekomusume/LICENSE-APACHE": (stat.S_IFREG, 0o644),
    f"{root}/share/doc/nekomusume/LICENSE-MIT": (stat.S_IFREG, 0o644),
    f"{root}/share/doc/nekomusume/README.txt": (stat.S_IFREG, 0o644),
    f"{root}/SHA256SUMS": (stat.S_IFREG, 0o644),
}
with tarfile.open(archive, "r:gz") as stream:
    members = stream.getmembers()
    if len(members) != len(expected) or {item.name for item in members} != set(expected):
        raise SystemExit("archive member set changed during metadata validation")
    for item in members:
        kind, mode = expected[item.name]
        expected_type = tarfile.DIRTYPE if kind == stat.S_IFDIR else tarfile.REGTYPE
        if item.mode != mode or item.type != expected_type:
            raise SystemExit(f"unexpected archive mode or type: {item.name}")
PY
tar -xzf "$ARCHIVE" -C "$TMP" --no-same-owner --no-same-permissions
ROOT="$TMP/$ROOT_NAME"
[ -d "$ROOT" ] && [ "$(find "$TMP" -mindepth 1 -maxdepth 1 | wc -l)" -eq 1 ]
[ "$(stat -c %a "$ROOT/bin/neko-cli")" = 755 ]
while IFS= read -r file; do
  [ "$(stat -c %a "$file")" = 644 ] || { echo "insecure document mode: $file" >&2; exit 1; }
done < <(find "$ROOT/share/doc/nekomusume" -type f -print)
EXPECTED_SUM_PATHS=$(printf '%s\n' './bin/neko-cli' './share/doc/nekomusume/LICENSE-APACHE' './share/doc/nekomusume/LICENSE-MIT' './share/doc/nekomusume/README.txt' | sort)
ACTUAL_SUM_PATHS=$(awk '
  length($1) != 64 || $1 !~ /^[0-9A-Fa-f]+$/ || $2 !~ /^\*?\.\// {exit 1}
  {path=$2; sub(/^\*/, "", path); print path}
' "$ROOT/SHA256SUMS" | sort) || { echo "malformed checksum manifest" >&2; exit 1; }
[ "$EXPECTED_SUM_PATHS" = "$ACTUAL_SUM_PATHS" ] || { echo "unexpected checksum manifest layout" >&2; exit 1; }
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
