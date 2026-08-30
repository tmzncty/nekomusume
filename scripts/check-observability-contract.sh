#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
python3 - "$ROOT" <<'PY'
import json,re,sys
from pathlib import Path
root=Path(sys.argv[1])
files=[root/'schema/observability-event.v1.json',root/'schema/health.v1.json',root/'schema/diagnostic-bundle.v1.json']
for p in files:
    obj=json.loads(p.read_text(encoding='utf-8'))
    if obj.get('$schema')!='https://json-schema.org/draft/2020-12/schema': raise SystemExit(f'{p}: wrong JSON Schema draft')
    if obj.get('additionalProperties') is not False: raise SystemExit(f'{p}: top level must be closed')
event=json.loads(files[0].read_text())
health=json.loads(files[1].read_text())
bundle=json.loads(files[2].read_text())
required_events={'carrier.switch_completed','recovery.rtt_updated','recovery.loss_detected','recovery.pto_fired','recovery.frame_retransmitted','flow.blocked','scheduler.dequeued','migration.completed','crypto.key_update_completed','resource.limit_hit','diagnostic.events_dropped'}
events=set(event['properties']['event']['enum'])
missing=required_events-events
if missing: raise SystemExit('missing mandatory events: '+','.join(sorted(missing)))
props=event['$defs']['data']['properties']
for key in ('switch_reason','latest_rtt_us','loss_per_mille','pto_count','retransmit_frames','queued_bytes','wait_us','path_generation','new_key_phase','resource','dropped_total'):
    if key not in props: raise SystemExit(f'missing event field: {key}')
if health['properties']['event_buffer']['properties']['capacity']['maximum']>1024: raise SystemExit('event capacity too large')
if bundle['properties']['capture']['properties']['max_bundle_bytes']['maximum']>8388608: raise SystemExit('bundle bound too large')
text='\n'.join(p.read_text(encoding='utf-8').lower() for p in files)
for forbidden in ('private_key','private-key','e2e_secret','join_token','cookie','password','raw_address','socket_address','payload_body'):
    if re.search(r'"'+re.escape(forbidden)+r'"\s*:',text): raise SystemExit(f'forbidden secret/locator field: {forbidden}')
print('observability contract validation passed')
PY
