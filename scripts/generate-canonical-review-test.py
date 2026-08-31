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
text = generator.render(root)
assert text.count("| `") >= 42
# Every executable row must have an adapter; remove one mapping and require render failure.
key = (root["vectors"][0]["domain"], root["vectors"][0]["operation"])
old = generator.ADAPTERS.pop(key)
try:
    try: generator.render(root)
    except ValueError as e: assert root["vectors"][0]["id"] in str(e)
    else: raise AssertionError("render accepted missing executable adapter")
finally: generator.ADAPTERS[key] = old
# Corpus identity mutation must not be silently accepted by the generator contract.
mutated = dict(root); mutated["vectors"] = list(root["vectors"]); mutated["vectors"][0] = dict(root["vectors"][0]); mutated["vectors"][0]["id"] += ".changed"
assert generator.sha256_canonical(mutated) != root["corpus_sha256"]
print("canonical review generator mutation tests passed")
