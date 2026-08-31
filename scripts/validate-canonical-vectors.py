#!/usr/bin/env python3
"""Strict structural gate for the candidate canonical vector corpus.

The three oracle bits are deliberately mandatory: an adapter that executes a
vector must set them only after encode(value)==bytes, decode(bytes)==expected,
and encode(decode(bytes))==bytes. This gate rejects unverifiable vectors rather
than silently treating them as evidence.
"""
import hashlib, json, re, sys
from pathlib import Path

REQUIRED_DOMAINS = frozenset({
    "negotiation", "wire", "frame", "ack", "reliable_udp", "datagram",
    "key_update", "carrier_transition", "close", "error",
})
DOMAINS = REQUIRED_DOMAINS
CLASSES = {"valid","malformed","truncated","trailing","oversized","unknown_enum","unknown_version","unauthenticated","out_of_range","integer_min","integer_max","integer_overflow","noncanonical_integer","duplicate","late","expected_failure","conceptual","state_only"}
HEX = re.compile(r"^(?:[0-9a-f]{2})*$")
ID = re.compile(r"^[a-z0-9][a-z0-9._-]{2,95}$")
ERR = re.compile(r"^[A-Z][A-Za-z0-9_.-]{1,95}$")
SHA256 = re.compile(r"^[0-9a-f]{64}$")

def fail(msg):
    raise ValueError(msg)

def canonical_content_sha256(root):
    content = dict(root)
    content.pop("corpus_sha256", None)
    canonical = json.dumps(
        content, ensure_ascii=False, allow_nan=False, sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")
    return hashlib.sha256(canonical).hexdigest()

def check(path):
    root = json.loads(Path(path).read_text(encoding="utf-8"))
    required_root={"schema","schema_version","schema_revision","corpus_sha256","freeze","vectors"}
    if set(root) != required_root: fail("root properties")
    if root.get("schema") != "nekomusume.canonical-vector.v1" or root.get("schema_version") != 1: fail("schema/version")
    if root.get("schema_revision") != 1: fail("schema_revision")
    if root.get("freeze") is not False: fail("freeze must remain false until N9")
    claimed_hash = root.get("corpus_sha256", "")
    if not SHA256.fullmatch(claimed_hash): fail("corpus_sha256")
    actual_hash = canonical_content_sha256(root)
    if claimed_hash != actual_hash: fail("corpus_sha256 mismatch")
    vectors = root.get("vectors")
    if not isinstance(vectors, list) or not 1 <= len(vectors) <= 4096: fail("vectors bounds")
    ids = set(); domains = set()
    for i, v in enumerate(vectors):
        p=f"vectors[{i}]"
        if not isinstance(v, dict): fail(p)
        required={"id","domain","operation","input","bytes_hex","expected","oracle","classification"}
        if set(v) != required: fail(p+" properties")
        if not ID.fullmatch(v["id"]) or v["id"] in ids: fail(p+" id")
        ids.add(v["id"])
        if v["domain"] not in DOMAINS: fail(p+" domain")
        domains.add(v["domain"])
        if not re.fullmatch(r"^[a-z0-9][a-z0-9._-]{1,63}$", v["operation"]): fail(p+" operation")
        if not isinstance(v["input"], dict): fail(p+" input")
        b=v["bytes_hex"]
        state_only = "state_only" in v["classification"]
        if state_only:
            if b is not None: fail(p+" state_only bytes_hex must be null")
        elif not isinstance(b,str) or len(b)>32768 or not HEX.fullmatch(b): fail(p+" bytes_hex")
        e=v["expected"]
        if not isinstance(e,dict) or set(e)-{"ok","value","error"} or not isinstance(e.get("ok"),bool): fail(p+" expected")
        if e["ok"] != ("value" in e and "error" not in e): fail(p+" expected polarity")
        if not e["ok"] and ("error" not in e or not ERR.fullmatch(e["error"])): fail(p+" error")
        o=v["oracle"]
        if set(o) != {"encode_equals_bytes","decode_bytes_equals_expected","roundtrip_equals_bytes"} or any(not isinstance(o[k], bool) for k in o): fail(p+" oracle booleans")
        if state_only and any(o.values()): fail(p+" state_only oracle")
        if not state_only and not any(o.values()): fail(p+" executable wire row has no oracle")
        if o["roundtrip_equals_bytes"] and (not e["ok"] or not o["encode_equals_bytes"] or not o["decode_bytes_equals_expected"]): fail(p+" roundtrip prerequisites")
        c=v["classification"]
        if not isinstance(c,list) or not c or len(set(c)) != len(c) or any(x not in CLASSES for x in c): fail(p+" classification")
    missing=REQUIRED_DOMAINS-domains
    if missing: fail("missing domains: "+",".join(sorted(missing)))
    print(f"canonical vector validation passed: {len(vectors)} vectors; domains={len(domains)}; freeze=false")

if __name__ == "__main__":
    check(sys.argv[1] if len(sys.argv)>1 else "fixtures/canonical-vectors.v1.json")
