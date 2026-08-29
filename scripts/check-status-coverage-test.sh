#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TMP=$(mktemp -d "${TMPDIR:-/tmp}/neko-coverage.XXXXXX")
trap 'rm -rf "$TMP"' EXIT INT TERM
cp "$ROOT/docs/status.md" "$TMP/valid.md"
./scripts/check-status-coverage.sh "$TMP/valid.md" >/dev/null
cp "$ROOT/docs/status.md" "$TMP/missing.md"
sed -i '0,/`docs\/spec\/m5-release-readiness-gate.md`/s//`docs\/spec\/missing-gate.md`/' "$TMP/missing.md"
if ./scripts/check-status-coverage.sh "$TMP/missing.md" >/dev/null 2>&1; then exit 1; fi
cp "$ROOT/docs/status.md" "$TMP/duplicate.md"
python3 - "$TMP/duplicate.md" <<'PY2'
from pathlib import Path
import sys
p=Path(sys.argv[1]); lines=p.read_text().splitlines(); row=next(x for x in lines if x.startswith('| production |')); lines.insert(lines.index(row)+1,row); p.write_text('\n'.join(lines)+'\n')
PY2
if ./scripts/check-status-coverage.sh "$TMP/duplicate.md" >/dev/null 2>&1; then exit 1; fi
LINK="$ROOT/docs/spec/.status-coverage-link.md"
cleanup_link() { rm -f "$LINK"; }
trap 'cleanup_link; rm -rf "$TMP"' EXIT INT TERM
ln -s m5-release-readiness-gate.md "$LINK"
cp "$ROOT/docs/status.md" "$TMP/symlink.md"
sed -i '0,/`docs\/spec\/m5-release-readiness-gate.md`/s//`docs\/spec\/.status-coverage-link.md`/' "$TMP/symlink.md"
if ./scripts/check-status-coverage.sh "$TMP/symlink.md" >/dev/null 2>&1; then exit 1; fi
cleanup_link
printf '%s\n' 'status scoped coverage mutation regression passed'
