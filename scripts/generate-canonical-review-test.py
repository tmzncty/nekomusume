#!/usr/bin/env python3
"""Mutation tests for generated review coverage and corpus identity."""
import importlib.util, json, tempfile
from pathlib import Path
ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("generator", ROOT / "scripts/generate-canonical-review.py")
generator = importlib.util.module_from_spec(spec); spec.loader.exec_module(generator)
root = json.loads((ROOT / "fixtures/canonical-vectors.v1.json").read_text(encoding="utf-8"))
assert len(root["vectors"]) == 42
assert generator.sha256_canonical(root) == root["corpus_sha256"]
assert root["freeze"] is True
reverted = dict(root); reverted["freeze"] = False; reverted["corpus_sha256"] = generator.sha256_canonical(reverted)
try:
    generator.render(reverted)
except ValueError as e:
    assert "freeze must be true" in str(e)
else:
    raise AssertionError("generator accepted freeze=false with recomputed identity")
text = generator.render(root)
assert text.count("| `") >= 42
# Every executable row must have an adapter; remove one mapping and require render failure.
key = (root["vectors"][0]["domain"], root["vectors"][0]["operation"])
old = generator.ORACLE_PATHS.pop(key)
try:
    try: generator.render(root)
    except ValueError as e: assert root["vectors"][0]["id"] in str(e)
    else: raise AssertionError("render accepted missing executable adapter")
finally: generator.ORACLE_PATHS[key] = old
# Missing and mislabeled enabled oracle paths must be rejected.
neg = root["vectors"][0]
neg_key = (neg["domain"], neg["operation"])
missing_path = generator.ORACLE_PATHS[neg_key].pop("decode_bytes_equals_expected")
try:
    try: generator.render(root)
    except ValueError as e: assert "missing enabled oracle path" in str(e)
    else: raise AssertionError("render accepted missing enabled oracle path")
finally:
    generator.ORACLE_PATHS[neg_key]["decode_bytes_equals_expected"] = missing_path
old_path = generator.ORACLE_PATHS[neg_key]["decode_bytes_equals_expected"]
generator.ORACLE_PATHS[neg_key]["decode_bytes_equals_expected"] = "VersionNegotiator::client_hello"
try:
    try: generator.render(root)
    except ValueError as e: assert "negotiation.hello.v0-v2" in str(e) or "mislabeled" in str(e)
    else: raise AssertionError("render accepted mislabeled negotiation decode path")
finally:
    generator.ORACLE_PATHS[neg_key]["decode_bytes_equals_expected"] = old_path
# The legacy one-string coarse adapter shape must be rejected by render.
paths = generator.ORACLE_PATHS[neg_key]
generator.ORACLE_PATHS[neg_key] = "VersionNegotiator::client_hello"
try:
    try: generator.render(root)
    except ValueError as e: assert "coarse oracle path mapping rejected" in str(e)
    else: raise AssertionError("render accepted old coarse negotiation shape")
finally:
    generator.ORACLE_PATHS[neg_key] = paths
assert not hasattr(generator, "ADAPTERS")
# Corpus identity mutation must not be silently accepted by the generator contract.
mutated = dict(root); mutated["vectors"] = list(root["vectors"]); mutated["vectors"][0] = dict(root["vectors"][0]); mutated["vectors"][0]["id"] += ".changed"
assert generator.sha256_canonical(mutated) != root["corpus_sha256"]
print("canonical review generator mutation tests passed")
