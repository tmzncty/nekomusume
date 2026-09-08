#!/usr/bin/env python3
"""Fail closed when an inventoried pre-auth responder loses required ordering."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "docs/preauth-responder-inventory.v1.json"
data = json.loads(MANIFEST.read_text(encoding="utf-8"))
if data.get("schema") != "nekomusume.preauth-responder-inventory.v1":
    raise SystemExit("preauth responder inventory: wrong schema")


def bounded_region(text, begin, end, rid, label):
    start = text.find(begin)
    if start < 0:
        raise ValueError(f"{rid}: missing {label} begin anchor {begin!r}")
    stop = text.find(end, start + len(begin))
    if stop < 0:
        raise ValueError(f"{rid}: missing {label} end anchor {end!r}")
    return text[start:stop]


def validate_pending_lifecycle(responder, text):
    rid = responder["id"]
    required = (
        "pending_producer_begin", "pending_producer_end",
        "pending_consumer_begin", "pending_consumer_end",
        "pending_expiry_begin", "pending_expiry_end",
        "pending_reserve_anchor", "pending_store_anchor",
        "pending_cancel_anchor", "expiry_cleanup_anchor",
    )
    if not all(responder.get(key) for key in required):
        raise ValueError(f"{rid}: pending owner lacks scoped lifecycle anchors")
    producer = bounded_region(text, responder["pending_producer_begin"], responder["pending_producer_end"], rid, "producer")
    consumer = bounded_region(text, responder["pending_consumer_begin"], responder["pending_consumer_end"], rid, "consumer")
    expiry = bounded_region(text, responder["pending_expiry_begin"], responder["pending_expiry_end"], rid, "expiry")
    reserve = producer.find(responder["pending_reserve_anchor"])
    store = producer.find(responder["pending_store_anchor"], reserve + len(responder["pending_reserve_anchor"]))
    if reserve < 0 or store < 0:
        raise ValueError(f"{rid}: producer does not prove reserve -> persisted store ordering")
    if responder["pending_cancel_anchor"] not in consumer:
        raise ValueError(f"{rid}: consumer lacks scoped terminal cancellation")
    if responder["expiry_cleanup_anchor"] not in expiry:
        raise ValueError(f"{rid}: expiry region lacks scoped cleanup")


seen = set()
responders = data.get("responders", [])
for responder in responders:
    rid = responder.get("id")
    if not rid or rid in seen:
        raise SystemExit(f"preauth responder inventory: missing/duplicate id {rid!r}")
    seen.add(rid)
    path = ROOT / responder["file"]
    text = path.read_text(encoding="utf-8")
    begin = responder["begin"]
    end = responder["end"]
    try:
        region = bounded_region(text, begin, end, rid, "responder")
    except ValueError as error:
        raise SystemExit(error) from error
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
    if responder.get("pending_owner"):
        try:
            validate_pending_lifecycle(responder, text)
        except ValueError as error:
            raise SystemExit(error) from error

# Regression: an identical store anchor outside the scoped producer must not
# rescue a broken persisted-owner lifecycle.
pending_fixture = next(r for r in responders if r.get("id") == "failover_udp_pending")
fixture = "\n".join((
    pending_fixture["pending_expiry_begin"], pending_fixture["expiry_cleanup_anchor"], pending_fixture["pending_expiry_end"],
    pending_fixture["pending_consumer_begin"], pending_fixture["pending_cancel_anchor"], pending_fixture["pending_consumer_end"],
    pending_fixture["pending_producer_begin"], pending_fixture["pending_reserve_anchor"], pending_fixture["pending_producer_end"],
    pending_fixture["pending_store_anchor"],
))
try:
    validate_pending_lifecycle(pending_fixture, fixture)
except ValueError:
    pass
else:
    raise SystemExit("failover_udp_pending: unrelated store anchor satisfied scoped producer mutation")

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
        admits += text.count(".admit(peer)") + text.count(".admit_carrier(")
if admits != 6:
    raise SystemExit(f"preauth responder inventory: admission surface count changed: {admits} != 6")

print(f"preauth responder inventory passed: {len(seen)} surfaces, {admits} admission sites")
