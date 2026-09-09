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
        # Persisted pending-owner anchors are checked in their declared,
        # occurrence-stable lifecycle regions below; whole-file membership
        # would let a sibling responder satisfy the wrong owner.
        if anchor and not responder.get("pending_owner") and anchor not in text:
            raise SystemExit(f"{rid}: missing {key} {anchor!r}")
    if responder.get("pending_owner"):
        try:
            validate_pending_lifecycle(responder, text)
        except ValueError as error:
            raise SystemExit(error) from error

# The two inventoried UDP entries intentionally share one persisted pending
# lifecycle. Make that sharing explicit rather than allowing either entry to
# silently borrow a different sibling occurrence.
pending_owners = [r for r in responders if r.get("pending_owner")]
if len(pending_owners) != 2:
    raise SystemExit("preauth responder inventory: expected two pending owners")
lifecycle_keys = (
    "pending_producer_begin", "pending_producer_end", "pending_consumer_begin",
    "pending_consumer_end", "pending_expiry_begin", "pending_expiry_end",
    "pending_reserve_anchor", "pending_store_anchor", "pending_cancel_anchor",
    "expiry_cleanup_anchor",
)
if any(tuple(r.get(key) for key in lifecycle_keys) != tuple(pending_owners[0].get(key) for key in lifecycle_keys) for r in pending_owners[1:]):
    raise SystemExit("preauth responder inventory: sibling pending owners do not reference one shared lifecycle")

# Mutation regressions: an identical anchor outside the intended region must not
# rescue producer/store, consumer/cancel, or expiry cleanup ownership.
pending_fixture = pending_owners[0]
fixture_parts = (
    pending_fixture["pending_expiry_begin"], pending_fixture["expiry_cleanup_anchor"], pending_fixture["pending_expiry_end"],
    pending_fixture["pending_consumer_begin"], pending_fixture["pending_cancel_anchor"], pending_fixture["pending_consumer_end"],
    pending_fixture["pending_producer_begin"], pending_fixture["pending_reserve_anchor"], pending_fixture["pending_producer_end"],
    pending_fixture["pending_store_anchor"],
)
for label, fixture in (
    ("store", "\n".join(fixture_parts)),
    ("cancel", "\n".join(fixture_parts).replace(pending_fixture["pending_cancel_anchor"], "missing-cancel")),
    ("expiry", "\n".join(fixture_parts).replace(pending_fixture["expiry_cleanup_anchor"], "missing-expiry")),
):
    try:
        validate_pending_lifecycle(pending_fixture, fixture)
    except ValueError:
        continue
    raise SystemExit(f"failover_udp_pending: unrelated {label} anchor satisfied scoped lifecycle mutation")

expected = {
    "ordinary_tcp_probe",
    "ordinary_udp_probe",
    "periodic_tcp",
    "multistream_tcp",
    "failover_udp_pending",
    "failover_udp_new",
    "failover_tcp",
    "endpoint_rebind_udp",
}
if seen != expected:
    raise SystemExit(f"preauth responder inventory: expected {sorted(expected)}, got {sorted(seen)}")

source = (ROOT / "crates/neko-cli/src").glob("*.rs")
admits = 0
for path in source:
    text = path.read_text(encoding="utf-8")
    if path.name != "preauth.rs":
        admits += text.count(".admit(peer)") + text.count(".admit_carrier(")
if admits != 7:
    raise SystemExit(f"preauth responder inventory: admission surface count changed: {admits} != 7")

print(f"preauth responder inventory passed: {len(seen)} surfaces, {admits} admission sites")
