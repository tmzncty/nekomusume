#!/usr/bin/env python3
"""Generate/check the non-normative canonical-vector review artifact."""
import argparse, hashlib, json, pathlib
ROOT = pathlib.Path(__file__).resolve().parent.parent
CORPUS = ROOT / "fixtures/canonical-vectors.v1.json"
OUT = ROOT / "docs/spec/canonical-vector-review.v1.md"
ADAPTERS = {
    ("negotiation", "client_hello"): "crates/neko-wire/tests/canonical_vectors.rs::every_claimed_oracle_executes_real_implementation_code (VersionNegotiator::client_hello)",
    ("negotiation", "server_accept_hello"): "crates/neko-wire/tests/canonical_vectors.rs::every_claimed_oracle_executes_real_implementation_code (VersionNegotiator::server_accept_hello)",
    ("negotiation", "server_response"): "crates/neko-wire/tests/canonical_vectors.rs::every_claimed_oracle_executes_real_implementation_code (VersionNegotiator::client_accept_response)",
    ("wire", "record"): "crates/neko-wire/tests/canonical_vectors.rs::every_claimed_oracle_executes_real_implementation_code (neko_wire::{encode,decode})",
    ("wire", "varint"): "crates/neko-wire/tests/canonical_vectors.rs::every_claimed_oracle_executes_real_implementation_code (neko_wire::{encode_varint,decode_varint})",
    ("error", "decode"): "crates/neko-wire/tests/canonical_vectors.rs::every_claimed_oracle_executes_real_implementation_code (neko_wire::decode)",
    ("frame", "frames"): "crates/neko-wire/tests/canonical_vectors.rs::every_claimed_oracle_executes_real_implementation_code (neko_wire::{encode_frames,decode_frames})",
    ("close", "frames"): "crates/neko-wire/tests/canonical_vectors.rs::every_claimed_oracle_executes_real_implementation_code (neko_wire::{encode_frames,decode_frames})",
}

def sha256_canonical(root):
    value = dict(root); value.pop("corpus_sha256", None)
    raw = json.dumps(value, ensure_ascii=False, allow_nan=False, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(raw).hexdigest()

def render(root):
    vectors = root["vectors"]
    lines = ["# Canonical vector review coverage (generated)", "", "> **Non-normative review artifact.** Generated from `fixtures/canonical-vectors.v1.json`; it records review coverage and implementation evidence only. It does not add, replace, or freeze protocol requirements. `freeze=false` remains authoritative.", "", f"- Corpus schema: `{root['schema']}` revision `{root['schema_revision']}`", f"- Corpus identity: `{root['corpus_sha256']}`", f"- Vector count: **{len(vectors)}**", "- Generator: `scripts/generate-canonical-review.py`", "", "| # | id | domain / operation | classification | bytes | oracles | expected semantic fields / error | adapter / coverage path |", "|---:|---|---|---|:---:|---|---|---|"]
    for i, v in enumerate(vectors, 1):
        state = "state-only (no wire bytes)" if "state_only" in v["classification"] else "present"
        oracle = ", ".join(k.removesuffix("_equals_bytes").replace("_", " ") for k,x in v["oracle"].items() if x) or "none (state-only)"
        exp = "error: `" + v["expected"]["error"] + "`" if not v["expected"]["ok"] else "fields: `" + ", ".join(v["expected"]["value"].keys()) + "`"
        adapter = "state-only exceptional row; no executable wire adapter claimed" if "state_only" in v["classification"] else ADAPTERS.get((v["domain"], v["operation"]))
        if adapter is None: raise ValueError(f"missing adapter mapping for {v['id']}")
        cls = ", ".join(v["classification"])
        lines.append(f"| {i} | `{v['id']}` | `{v['domain']}` / `{v['operation']}` | {cls} | {state} | {oracle} | {exp} | `{adapter}` |")
    lines += ["", "## Gate contract", "", "The generator is also the consistency gate: `python3 scripts/generate-canonical-review.py --check` regenerates this exact byte content and fails on drift. It validates that the corpus identity is embedded, every vector has one row, executable vectors have a known adapter mapping, and state-only exceptional rows explicitly have no wire adapter. The corpus validator and Rust canonical-vector test remain separate gates for schema, bytes, expected values, and real adapter execution.", ""]
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
