#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TMP=$(mktemp -d "${TMPDIR:-/tmp}/neko-plan.XXXXXX")
trap 'rm -rf "$TMP"' EXIT INT TERM
cp "$ROOT/ROADMAP.md" "$TMP/roadmap.md"
cp "$ROOT/IMPLEMENTATION_PLAN.md" "$TMP/plan.md"
./scripts/check-plan-sync.sh "$TMP/roadmap.md" "$TMP/plan.md" >/dev/null
sed -i '0,/^- \[x\].*0-RTT/s//- [ ] 0-RTT/' "$TMP/plan.md"
if ./scripts/check-plan-sync.sh "$TMP/roadmap.md" "$TMP/plan.md" >/dev/null 2>&1; then exit 1; fi
printf '%s\n' 'roadmap/plan mutation regression passed'
