#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TMP=$(mktemp -d "${TMPDIR:-/tmp}/neko-boundary.XXXXXX")
trap 'rm -rf "$TMP"' EXIT INT TERM
cp "$ROOT/docs/status.md" "$TMP/valid.md"
./scripts/check-release-boundaries.sh "$TMP/valid.md" >/dev/null
cp "$ROOT/docs/status.md" "$TMP/unblocked.md"
sed -i 's/^| reachability |/| reachability |/' "$TMP/unblocked.md"
sed -i '/^| reachability |/s/| blocked |/| candidate |/' "$TMP/unblocked.md"
if ./scripts/check-release-boundaries.sh "$TMP/unblocked.md" >/dev/null 2>&1; then exit 1; fi
cp "$ROOT/docs/status.md" "$TMP/claim.md"
sed -i 's/| production |/| production-ready |/' "$TMP/claim.md"
if ./scripts/check-release-boundaries.sh "$TMP/claim.md" >/dev/null 2>&1; then exit 1; fi
printf '%s\n' 'release boundary mutation regression passed'
