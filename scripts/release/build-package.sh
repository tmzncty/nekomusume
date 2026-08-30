#!/usr/bin/env bash
# Build a deterministic, bounded Linux binary archive and emit machine-readable evidence.
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$ROOT"
TARGET=${TARGET:-$(rustc -vV | sed -n 's/^host: //p')}
OUT=${OUT:-$ROOT/dist}
SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH:-$(git show -s --format=%ct HEAD)}
VERSION=$(cargo metadata --locked --format-version 1 --no-deps | sed -n 's/.*"name":"neko-cli","version":"\([^"]*\)".*/\1/p')
case "$TARGET" in
  x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu) ;;
  *) echo "unsupported release target: $TARGET" >&2; exit 2 ;;
esac
case "$SOURCE_DATE_EPOCH" in *[!0-9]*|'') echo "SOURCE_DATE_EPOCH must be an integer" >&2; exit 2;; esac
command -v sha256sum >/dev/null
command -v tar >/dev/null
command -v gzip >/dev/null
mkdir -p "$OUT"
OUT=$(CDPATH= cd -- "$OUT" && pwd)
NAME="nekomusume-${VERSION}-${TARGET}"
STAGE=$(mktemp -d)
trap 'rm -rf "$STAGE"' EXIT HUP INT TERM
export SOURCE_DATE_EPOCH
export CARGO_INCREMENTAL=0
cargo build --release --locked --target "$TARGET" --package neko-cli
install -d -m 0755 "$STAGE/$NAME/bin" "$STAGE/$NAME/share/doc/nekomusume"
install -m 0755 "target/$TARGET/release/neko-cli" "$STAGE/$NAME/bin/neko-cli"
install -m 0644 LICENSE-MIT LICENSE-APACHE docs/release/README.txt "$STAGE/$NAME/share/doc/nekomusume/"
(
  cd "$STAGE/$NAME"
  find . -type f ! -name SHA256SUMS -print0 | LC_ALL=C sort -z | xargs -0 sha256sum > SHA256SUMS
  chmod 0644 SHA256SUMS
)
ARCHIVE="$OUT/$NAME.tar.gz"
# GNU tar fixes ordering, ownership, permissions metadata and mtimes; gzip -n omits its timestamp/name.
tar --sort=name --format=gnu --owner=0 --group=0 --numeric-owner \
  --mtime="@$SOURCE_DATE_EPOCH" -C "$STAGE" -cf - "$NAME" | gzip -n -9 > "$ARCHIVE.tmp"
mv -f "$ARCHIVE.tmp" "$ARCHIVE"
ARCHIVE_SHA256=$(sha256sum "$ARCHIVE" | cut -d' ' -f1)
BINARY_SHA256=$(sha256sum "$STAGE/$NAME/bin/neko-cli" | cut -d' ' -f1)
printf '{"schema":"nekomusume.release-build-evidence.v1","git_commit":"%s","version":"%s","target":"%s","source_date_epoch":%s,"archive":"%s","archive_sha256":"%s","binary_sha256":"%s","claim":"bounded research package; not an RC; production remains blocked"}\n' \
  "$(git rev-parse HEAD)" "$VERSION" "$TARGET" "$SOURCE_DATE_EPOCH" "$(basename "$ARCHIVE")" "$ARCHIVE_SHA256" "$BINARY_SHA256" > "$OUT/$NAME.build.json"
printf '%s  %s\n' "$ARCHIVE_SHA256" "$(basename "$ARCHIVE")"
