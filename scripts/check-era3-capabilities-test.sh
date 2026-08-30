#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TMP=$(mktemp -d "${TMPDIR:-/tmp}/neko-era3-capabilities.XXXXXX")
trap 'rm -rf "$TMP"' EXIT INT TERM
cp "$ROOT/docs/era3-capabilities.v1.json" "$TMP/valid.json"
"$ROOT/scripts/check-era3-capabilities.sh" "$TMP/valid.json" >/dev/null
python3 - "$TMP/valid.json" "$TMP/promoted.json" <<'PY'
import json, sys
with open(sys.argv[1], encoding="utf-8") as f: data=json.load(f)
for item in data["capabilities"]:
    if item["id"] == "zero-rtt": item["supported"] = True; item["evidence_class"] = "deterministic_local"
with open(sys.argv[2], "w", encoding="utf-8") as f: json.dump(data, f)
PY
if "$ROOT/scripts/check-era3-capabilities.sh" "$TMP/promoted.json" >/dev/null 2>&1; then
    echo 'unsupported capability promotion was accepted'; exit 1
fi
printf '%s\n' 'Era 3 unsupported-capability mutation regression passed'
