#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd); t=$(mktemp -d); trap 'rm -rf "$t"' EXIT
cp "$ROOT/docs/era4-capabilities.v1.json" "$t/m.json"
python3 - "$t/m.json" <<'PY'
import json,sys
p=sys.argv[1]; d=json.load(open(p)); d['freeze']=True; json.dump(d,open(p,'w'))
PY
if "$ROOT/scripts/check-era4-protocol-release.sh" "$t/m.json" "$ROOT/docs/era4-protocol-release-v1.md" >/dev/null 2>&1; then echo 'mutation unexpectedly passed'; exit 1; fi
echo 'Era-4 L protocol release mutation regression passed'
