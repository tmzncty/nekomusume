#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TMP=$(mktemp -d "${TMPDIR:-/tmp}/governance-status-test.XXXXXX")
trap 'rm -rf "$TMP"' EXIT
cp -a "$ROOT/." "$TMP/"
run_checker() { (cd "$TMP" && bash scripts/check-governance-status.sh); }
run_checker >/dev/null

# Missing evidence preceded by whitespace must fail, not be skipped.
sed -i '0,/| `docs\/adr\/m1-g0-noise-ik-candidate.md`/s//|   `does-not-exist.md`/' "$TMP/docs/status.md"
if run_checker >/dev/null 2>&1; then echo 'mutation unexpectedly passed: evidence path'; exit 1; fi
cp "$ROOT/docs/status.md" "$TMP/docs/status.md"

# An unchecked high-signal roadmap item must contradict implemented status.
sed -i 's/^- \[x\] CLI skeleton：/- [ ] CLI skeleton：/' "$TMP/ROADMAP.md"
if run_checker >/dev/null 2>&1; then echo 'mutation unexpectedly passed: roadmap drift'; exit 1; fi

echo 'governance checker regression tests passed'
