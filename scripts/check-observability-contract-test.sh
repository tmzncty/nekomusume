#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TMP=$(mktemp -d "${TMPDIR:-/tmp}/neko-observability.XXXXXX")
trap 'rm -rf "$TMP"' EXIT INT TERM
mkdir -p "$TMP/schema" "$TMP/scripts"
cp "$ROOT"/schema/{observability-event.v1.json,health.v1.json,diagnostic-bundle.v1.json} "$TMP/schema/"
cp "$ROOT/scripts/check-observability-contract.sh" "$TMP/scripts/"
"$TMP/scripts/check-observability-contract.sh" >/dev/null
python3 - "$TMP/schema/observability-event.v1.json" <<'PY'
import json,sys
p=sys.argv[1]; x=json.load(open(p)); x['$defs']['data']['properties']['private_key']={'type':'string'}; open(p,'w').write(json.dumps(x))
PY
if "$TMP/scripts/check-observability-contract.sh" >/dev/null 2>&1; then exit 1; fi
cp "$ROOT/schema/observability-event.v1.json" "$TMP/schema/observability-event.v1.json"
python3 - "$TMP/schema/observability-event.v1.json" <<'PY'
import json,sys
p=sys.argv[1]; x=json.load(open(p)); x['properties']['event']['enum'].remove('recovery.pto_fired'); open(p,'w').write(json.dumps(x))
PY
if "$TMP/scripts/check-observability-contract.sh" >/dev/null 2>&1; then exit 1; fi
printf '%s\n' 'observability contract mutation regression passed'
