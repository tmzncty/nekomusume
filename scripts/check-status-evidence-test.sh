#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TMP=$(mktemp -d "${TMPDIR:-/tmp}/neko-status.XXXXXX")
trap 'rm -rf "$TMP"' EXIT INT TERM
mutate() { python3 - "$1" "$2" <<'PY'
from pathlib import Path
import sys
p=Path(sys.argv[1]); replacement=sys.argv[2]
s=p.read_text(); old='`docs/adr/m1-g0-research-authorization.md`'; assert old in s
p.write_text(s.replace(old, replacement, 1))
PY
}
cp "$ROOT/docs/status.md" "$TMP/valid.md"
./scripts/check-status-evidence.sh "$TMP/valid.md" >/dev/null
cp "$ROOT/docs/status.md" "$TMP/missing.md"; mutate "$TMP/missing.md" '`docs/missing-evidence.md`'
if ./scripts/check-status-evidence.sh "$TMP/missing.md" >/dev/null 2>&1; then exit 1; fi
cp "$ROOT/docs/status.md" "$TMP/absolute.md"; mutate "$TMP/absolute.md" '`/tmp/evidence.md`'
if ./scripts/check-status-evidence.sh "$TMP/absolute.md" >/dev/null 2>&1; then exit 1; fi
cp "$ROOT/docs/status.md" "$TMP/directory.md"; mutate "$TMP/directory.md" '`docs`'
if ./scripts/check-status-evidence.sh "$TMP/directory.md" >/dev/null 2>&1; then exit 1; fi
cp "$ROOT/docs/status.md" "$TMP/invalid.md"; python3 - "$TMP/invalid.md" <<'PY'
from pathlib import Path
p=Path(__import__('sys').argv[1]); s=p.read_text(); s=s.replace('| candidate |', '| invalid |', 1); p.write_text(s)
PY
if ./scripts/check-status-evidence.sh "$TMP/invalid.md" >/dev/null 2>&1; then exit 1; fi
printf '%s\n' 'status evidence mutation tests passed'
