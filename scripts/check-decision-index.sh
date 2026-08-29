#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
DECISIONS="${1:-$ROOT/docs/decisions.md}"
python3 - "$DECISIONS" <<'PY'
from pathlib import Path
import re,sys
p=Path(sys.argv[1]); ids=[]
for n,line in enumerate(p.read_text(encoding='utf-8').splitlines(),1):
    if not line.startswith('## '):
        continue
    m=re.search(r'\b(D\d{3})\b\s*(?:[：:]|—)', line)
    if m:
        ids.append((m.group(1),n))
    elif re.search(r'\bD\d{3}\b', line):
        raise SystemExit(f'malformed numbered decision heading at line {n}: {line}')
if not ids: raise SystemExit('decision ledger has no numbered headings')
seen={}
for ident,line in ids:
    if ident in seen: raise SystemExit(f'duplicate decision ID: {ident} lines {seen[ident]},{line}')
    seen[ident]=line
print(f'decision index validation passed: {len(ids)} unique numbered decisions')
PY
