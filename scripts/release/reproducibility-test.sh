#!/usr/bin/env bash
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT HUP INT TERM
EPOCH=$(git -C "$ROOT" show -s --format=%ct HEAD)
SOURCE_DATE_EPOCH="$EPOCH" OUT="$TMP/dist-a" "$ROOT/scripts/release/build-package.sh" >/dev/null
SOURCE_DATE_EPOCH="$EPOCH" OUT="$TMP/dist-b" "$ROOT/scripts/release/build-package.sh" >/dev/null
A=$(sha256sum "$TMP"/dist-a/*.tar.gz | cut -d' ' -f1)
B=$(sha256sum "$TMP"/dist-b/*.tar.gz | cut -d' ' -f1)
test "$A" = "$B"
test -z "$(git -C "$ROOT" status --porcelain)"
echo "package-reproducibility-ok archive_sha256=$A"
