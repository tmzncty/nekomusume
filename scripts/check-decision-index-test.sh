#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TMP=$(mktemp -d "${TMPDIR:-/tmp}/neko-decisions.XXXXXX")
trap 'rm -rf "$TMP"' EXIT INT TERM
cp "$ROOT/docs/decisions.md" "$TMP/valid.md"
./scripts/check-decision-index.sh "$TMP/valid.md" >/dev/null
cp "$ROOT/docs/decisions.md" "$TMP/duplicate.md"
python3 - "$TMP/duplicate.md" <<'PY2'
from pathlib import Path
import sys
p=Path(sys.argv[1]); s=p.read_text(); line=next(x for x in s.splitlines() if x.startswith('## ') and 'D031' in x); p.write_text(s+'\n'+line+'\n')
PY2
if ./scripts/check-decision-index.sh "$TMP/duplicate.md" >/dev/null 2>&1; then exit 1; fi
cp "$ROOT/docs/decisions.md" "$TMP/malformed.md"
printf '%s\n' '## D999 malformed without body' >> "$TMP/malformed.md"
# A malformed numbered heading must be rejected.
if ./scripts/check-decision-index.sh "$TMP/malformed.md" >/dev/null 2>&1; then exit 1; fi
printf '%s\n' 'decision index mutation regression passed'
