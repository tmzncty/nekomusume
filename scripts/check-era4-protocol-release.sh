#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
python3 - "${1:-$ROOT/docs/era4-capabilities.v1.json}" "${2:-$ROOT/docs/era4-protocol-release-v1.md}" <<'PY'
import json, pathlib, re, sys
m=json.loads(pathlib.Path(sys.argv[1]).read_text(encoding='utf-8'))
d=pathlib.Path(sys.argv[2]).read_text(encoding='utf-8')
assert m['schema']=='nekomusume.era4-capabilities.v1' and m['release_id'].startswith('era4-l-')
assert re.fullmatch(r'[0-9a-f]{40}',m['parent_commit']) and m['freeze'] is False and m['released'] is False
items=m['capabilities']; ids=[x['id'] for x in items]
assert len(ids)==len(set(ids)) and all(re.fullmatch(r'[a-z0-9-]+',i) for i in ids)
assert {x['state'] for x in items} >= {'supported','experimental','blocked'}
assert all(x['evidence_class'] in {'E0','E1','E2','E3','E4'} and x['limits'] for x in items)
assert all(v is False for v in m['release_gates'].values())
for phrase in ('not a protocol freeze','fail-closed','Mixed-feature conformance boundary','Remaining L work'):
    assert phrase.lower() in d.lower(), phrase
print('Era-4 L protocol release policy validation passed')
PY
