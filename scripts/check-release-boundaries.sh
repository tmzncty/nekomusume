#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
STATUS="${1:-$ROOT/docs/status.md}"
python3 - "$STATUS" <<'PY'
from pathlib import Path
import sys
p=Path(sys.argv[1]); text=p.read_text(encoding='utf-8')
rows={}
for line in text.splitlines():
    if line.startswith('|') and not line.startswith('|---') and not line.startswith('| ID '):
        c=[x.strip() for x in line.strip().strip('|').split('|')]
        if len(c)==5: rows[c[0]]=c
for ident in ('reachability','production'):
    if ident not in rows: raise SystemExit(f'missing mandatory boundary row: {ident}')
    if rows[ident][2] != 'blocked': raise SystemExit(f'{ident} boundary is not blocked: {rows[ident][2]}')
    boundary=rows[ident][4].lower()
    if 'no public' not in boundary and 'no production' not in boundary:
        raise SystemExit(f'{ident} boundary lacks explicit prohibition')
# Inspect only status-table cells, not the explanatory vocabulary below it.
for c in rows.values():
    table_text=' | '.join(c).lower()
    for forbidden in ('production-ready','production ready'):
        if forbidden in table_text:
            raise SystemExit(f'unsafe release claim in status table: {forbidden}')
print('release boundary validation passed: reachability and production remain blocked')
PY
