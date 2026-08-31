#!/usr/bin/env python3
"""Generate/check the non-normative canonical-vector review artifact."""
import argparse, hashlib, json, pathlib
ROOT = pathlib.Path(__file__).resolve().parent.parent
CORPUS = ROOT / "fixtures/canonical-vectors.v1.json"
OUT = ROOT / "docs/spec/canonical-vector-review.v1.md"
ORACLE_PATHS = {
    ("negotiation", "client_hello"): {
        "encode_equals_bytes": "VersionNegotiator::client_hello",
        "decode_bytes_equals_expected": "VersionNegotiator::server_accept_hello",
        "roundtrip_equals_bytes": "VersionNegotiator::client_hello -> VersionNegotiator::server_accept_hello",
    },
    ("negotiation", "server_accept_hello"): {
        "decode_bytes_equals_expected": "VersionNegotiator::server_accept_hello",
    },
    ("negotiation", "server_response"): {
        "encode_equals_bytes": "VersionNegotiator::server_accept_hello",
        "decode_bytes_equals_expected": "VersionNegotiator::client_accept_response",
        "roundtrip_equals_bytes": "VersionNegotiator::server_accept_hello -> VersionNegotiator::client_accept_response",
    },
    ("wire", "record"): {
        "encode_equals_bytes": "neko_wire::encode",
        "decode_bytes_equals_expected": "neko_wire::decode",
        "roundtrip_equals_bytes": "neko_wire::encode -> neko_wire::decode",
    },
    ("wire", "varint"): {
        "encode_equals_bytes": "neko_wire::encode_varint",
        "decode_bytes_equals_expected": "neko_wire::decode_varint",
        "roundtrip_equals_bytes": "neko_wire::encode_varint -> neko_wire::decode_varint",
    },
    ("error", "decode"): {"decode_bytes_equals_expected": "neko_wire::decode"},
    ("frame", "frames"): {
        "encode_equals_bytes": "neko_wire::encode_frames",
        "decode_bytes_equals_expected": "neko_wire::decode_frames",
        "roundtrip_equals_bytes": "neko_wire::encode_frames -> neko_wire::decode_frames",
    },
    ("close", "frames"): {
        "encode_equals_bytes": "neko_wire::encode_frames",
        "decode_bytes_equals_expected": "neko_wire::decode_frames",
        "roundtrip_equals_bytes": "neko_wire::encode_frames -> neko_wire::decode_frames",
    },
}
ORACLE_KEYS = ("encode_equals_bytes", "decode_bytes_equals_expected", "roundtrip_equals_bytes")
ORACLE_LABELS = {"encode_equals_bytes": "encode", "decode_bytes_equals_expected": "decode", "roundtrip_equals_bytes": "roundtrip"}
EXPECTED_ORACLE_PATHS = {key: dict(value) for key, value in ORACLE_PATHS.items()}


def oracle_paths(v):
    paths = ORACLE_PATHS.get((v["domain"], v["operation"]))
    if paths is None:
        raise ValueError(f"missing oracle path mapping for {v['id']}")
    enabled = [key for key in ORACLE_KEYS if v["oracle"].get(key)]
    if not isinstance(paths, dict):
        raise ValueError(f"coarse oracle path mapping rejected for {v['id']}")
    missing = [key for key in enabled if not paths.get(key)]
    if missing:
        raise ValueError(f"missing enabled oracle path for {v['id']}: {', '.join(missing)}")
    expected = EXPECTED_ORACLE_PATHS[(v["domain"], v["operation"])]
    mislabeled = [key for key in enabled if paths[key] != expected.get(key)]
    if mislabeled:
        raise ValueError(f"mislabeled enabled oracle path for {v['id']}: {', '.join(mislabeled)}")
    return paths


def sha256_canonical(root):
    value = dict(root); value.pop("corpus_sha256", None)
    raw = json.dumps(value, ensure_ascii=False, allow_nan=False, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(raw).hexdigest()

def render(root):
    if root.get("freeze") is not True:
        raise ValueError("freeze must be true for canonical corpus v1")
    vectors = root["vectors"]
    lines = ["# Canonical vector review coverage (generated)", "", "> **Non-normative review artifact.** Generated from `fixtures/canonical-vectors.v1.json`; it records review coverage and implementation evidence only. It does not add or replace protocol requirements. `freeze=true` records only the corpus-specific v1 compatibility freeze; repository-wide protocol/release `FREEZE=false` remains authoritative.", "", f"- Corpus schema: `{root['schema']}` revision `{root['schema_revision']}`", f"- Corpus identity: `{root['corpus_sha256']}`", f"- Vector count: **{len(vectors)}**", "- Generator: `scripts/generate-canonical-review.py`", "", "| # | id | domain / operation | classification | bytes | oracles | expected semantic fields / error | adapter / coverage path |", "|---:|---|---|---|:---:|---|---|---|"]
    for i, v in enumerate(vectors, 1):
        state = "state-only (no wire bytes)" if "state_only" in v["classification"] else "present"
        oracle = ", ".join(k.removesuffix("_equals_bytes").replace("_", " ") for k,x in v["oracle"].items() if x) or "none (state-only)"
        exp = "error: `" + v["expected"]["error"] + "`" if not v["expected"]["ok"] else "fields: `" + ", ".join(v["expected"]["value"].keys()) + "`"
        if "state_only" in v["classification"]:
            adapter = "state-only exceptional row; no executable wire adapter claimed"
        else:
            paths = oracle_paths(v)
            adapter = "; ".join(f"{ORACLE_LABELS[key]}={paths[key]}" for key in ORACLE_KEYS if v["oracle"].get(key))
        cls = ", ".join(v["classification"])
        lines.append(f"| {i} | `{v['id']}` | `{v['domain']}` / `{v['operation']}` | {cls} | {state} | {oracle} | {exp} | `{adapter}` |")
    lines += ["", "## Gate contract", "", "The generator is also the consistency gate: `python3 scripts/generate-canonical-review.py --check` regenerates this exact byte content and fails on drift. It validates that the corpus identity is embedded, every vector has one row, each enabled oracle bit has its exact encode/decode/roundtrip implementation path (including distinct negotiation producer/consumer paths), and state-only exceptional rows explicitly have no wire adapter. The corpus validator and Rust canonical-vector test remain separate gates for schema, bytes, expected values, and real adapter execution.", ""]
    return "\n".join(lines)

def main():
    p = argparse.ArgumentParser(); p.add_argument("--check", action="store_true"); args = p.parse_args()
    root = json.loads(CORPUS.read_text(encoding="utf-8"))
    if root.get("corpus_sha256") != sha256_canonical(root): raise SystemExit("corpus identity mismatch")
    text = render(root)
    if args.check:
        if not OUT.exists() or OUT.read_text(encoding="utf-8") != text: raise SystemExit("canonical review artifact drift: regenerate it")
        print(f"canonical review artifact check passed: {len(root['vectors'])} vectors")
    else:
        OUT.write_text(text, encoding="utf-8", newline="\n"); print(f"generated {OUT}")
if __name__ == "__main__": main()
