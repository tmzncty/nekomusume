#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
STATUS="${1:-$ROOT/docs/status.md}"
python3 - "$ROOT" "$STATUS" <<'PY'
import pathlib, re, sys
root=pathlib.Path(sys.argv[1]).resolve(); status=pathlib.Path(sys.argv[2]).resolve()
allowed={'implemented','candidate','provisional','absent','blocked'}
text=status.read_text(encoding='utf-8')
rows=[]
seen=set()
for line in text.splitlines():
    if not line.startswith('|') or line.startswith('|---') or line.startswith('| ID '): continue
    cells=[x.strip() for x in line.strip().strip('|').split('|')]
    if len(cells)!=5: raise SystemExit(f'invalid status row: {line}')
    ident,area,state,evidence,boundary=cells
    if not ident or ident in seen: raise SystemExit(f'duplicate or empty status ID: {ident}')
    seen.add(ident)
    if state not in allowed: raise SystemExit(f'invalid status: {ident}: {state}')
    evidence=evidence.strip('`')
    if not evidence or evidence.startswith('/') or re.match(r'^[A-Za-z]:[\\/]',evidence) or '\\' in evidence:
        raise SystemExit(f'evidence must be repository-relative POSIX file: {ident}: {evidence}')
    candidate=(root/evidence).resolve()
    try: candidate.relative_to(root)
    except ValueError: raise SystemExit(f'evidence escapes repository: {ident}: {evidence}')
    if not candidate.is_file(): raise SystemExit(f'evidence is missing or not a file: {ident}: {evidence}')
    rows.append(ident)
if not rows: raise SystemExit('status table has no rows')
print(f'status evidence validation passed: {len(rows)} rows')
PY
