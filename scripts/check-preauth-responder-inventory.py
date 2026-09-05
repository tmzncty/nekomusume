#!/usr/bin/env python3
"""Fail closed when an inventoried pre-auth responder loses required ordering."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "docs/preauth-responder-inventory.v1.json"
data = json.loads(MANIFEST.read_text(encoding="utf-8"))
if data.get("schema") != "nekomusume.preauth-responder-inventory.v1":
    raise SystemExit("preauth responder inventory: wrong schema")

seen = set()
for responder in data.get("responders", []):
    rid = responder.get("id")
    if not rid or rid in seen:
        raise SystemExit(f"preauth responder inventory: missing/duplicate id {rid!r}")
    seen.add(rid)
    path = ROOT / responder["file"]
    text = path.read_text(encoding="utf-8")
    begin = responder["begin"]
    end = responder["end"]
    start = text.find(begin)
    if start < 0:
        raise SystemExit(f"{rid}: missing begin anchor {begin!r}")
    stop = text.find(end, start + len(begin))
    if stop < 0:
        raise SystemExit(f"{rid}: missing end anchor {end!r}")
    region = text[start:stop]
    cursor = 0
    for anchor in responder["ordered"]:
        position = region.find(anchor, cursor)
        if position < 0:
            raise SystemExit(f"{rid}: missing/out-of-order anchor {anchor!r}")
        cursor = position + len(anchor)
    for key in (
        "admission_owner_anchor",
        "success_cleanup_anchor",
        "expiry_cleanup_anchor",
        "rejection_cleanup_anchor",
    ):
        anchor = responder.get(key)
        if anchor and anchor not in text:
            raise SystemExit(f"{rid}: missing {key} {anchor!r}")

expected = {
    "ordinary_tcp_probe",
    "ordinary_udp_probe",
    "periodic_tcp",
    "multistream_tcp",
    "failover_udp_pending",
    "failover_udp_new",
    "failover_tcp",
}
if seen != expected:
    raise SystemExit(f"preauth responder inventory: expected {sorted(expected)}, got {sorted(seen)}")

source = (ROOT / "crates/neko-cli/src").glob("*.rs")
admits = 0
for path in source:
    text = path.read_text(encoding="utf-8")
    if path.name != "preauth.rs":
        admits += text.count(".admit(peer)")
if admits != 6:
    raise SystemExit(f"preauth responder inventory: admission surface count changed: {admits} != 6")

print(f"preauth responder inventory passed: {len(seen)} surfaces, {admits} admission sites")
