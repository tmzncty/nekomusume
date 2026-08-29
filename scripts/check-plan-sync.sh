#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
ROADMAP="${1:-$ROOT/ROADMAP.md}"
PLAN="${2:-$ROOT/IMPLEMENTATION_PLAN.md}"
python3 - "$ROADMAP" "$PLAN" <<'PY'
from pathlib import Path
import re, sys
roadmap, plan = map(lambda x: Path(x).read_text(encoding='utf-8'), sys.argv[1:])
# Shared gates whose status must not drift between the roadmap and executable plan.
items = {
    'plpmtud': r'(?:PMTUD|PLPMTUD)',
    'fec': r'FEC',
    'unreliable': r'unreliable datagram',
    'key-update': r'key update',
    '0rtt': r'0-RTT',
    'concurrent': r'concurrent UDP \+ TCP',
    'multipath': r'heterogeneous multipath aggregation',
}
def marker(text, pattern):
    in_track=False; lines=[]
    for line in text.splitlines():
        if re.match(r'^## Experimental Track C', line): in_track=True; continue
        if in_track and line.startswith('## ') and not line.startswith('### '): break
        if in_track and re.match(r'^- \[[ xX]\]', line) and re.search(pattern, line, re.I) and ('bounded' in line.lower() or 'gate closed' in line.lower() or 'api' in line.lower()): lines.append(line)
    if not lines: raise SystemExit(f'missing plan item: {pattern}')
    # Prefer the experimental checklist line; reject ambiguity rather than guessing.
    checks=[line for line in lines if re.match(r'^- \[[ xX]\]', line.strip())]
    if len(checks)!=1: raise SystemExit(f'ambiguous plan item ({len(checks)}): {pattern}')
    return checks[0].strip()[3:6].lower(), checks[0]
for name, pattern in items.items():
    rline = marker(roadmap, pattern); pline = marker(plan, pattern)
    if rline[0] != pline[0]:
        raise SystemExit(f'plan marker drift: {name}: ROADMAP={rline[0]} PLAN={pline[0]}')
print(f'roadmap/implementation-plan sync passed: {len(items)} shared gates')
PY
