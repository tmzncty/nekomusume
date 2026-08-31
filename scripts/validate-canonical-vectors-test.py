#!/usr/bin/env python3
"""Mutation tests for canonical corpus identity and required coverage."""
import copy
import importlib.util
import json
import sys
import tempfile
from pathlib import Path

sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parent.parent
SPEC = importlib.util.spec_from_file_location(
    "canonical_validator", ROOT / "scripts/validate-canonical-vectors.py"
)
validator = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(validator)
source = json.loads((ROOT / "fixtures/canonical-vectors.v1.json").read_text())


def must_reject(name, mutate, expected_error):
    candidate = copy.deepcopy(source)
    mutate(candidate)
    with tempfile.NamedTemporaryFile("w", suffix=".json", encoding="utf-8") as f:
        json.dump(candidate, f)
        f.flush()
        try:
            validator.check(f.name)
        except ValueError as error:
            if expected_error not in str(error):
                raise AssertionError(f"{name}: rejected for {error!s}, expected {expected_error!r}")
            return
    raise AssertionError(f"validator accepted {name}")


validator.check(ROOT / "fixtures/canonical-vectors.v1.json")
must_reject(
    "wire-byte mutation with stale identity",
    lambda corpus: corpus["vectors"][0].__setitem__("bytes_hex", "4e31010200000003"),
    "corpus_sha256 mismatch",
)
must_reject(
    "semantic mutation with stale identity",
    lambda corpus: corpus["vectors"][0]["input"]["versions"].__setitem__(1, 3),
    "corpus_sha256 mismatch",
)
def remove_close_and_recompute(corpus):
    corpus["vectors"] = [v for v in corpus["vectors"] if v["domain"] != "close"]
    corpus["corpus_sha256"] = validator.canonical_content_sha256(corpus)


must_reject(
    "missing required close domain",
    remove_close_and_recompute,
    "missing domains: close",
)
print("canonical vector mutation tests passed")
