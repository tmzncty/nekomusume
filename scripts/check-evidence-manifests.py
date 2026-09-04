#!/usr/bin/env python3
"""Verify canonical Git evidence blobs against non-self-referential manifests."""
import hashlib
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
manifests = ["artifacts/n8-20260831/sha256sums.txt"]
line_pattern = re.compile(r"^([0-9a-f]{64})  (.+)$")
tracked = set(
    subprocess.check_output(["git", "-C", str(ROOT), "ls-files"], text=True).splitlines()
)
checked = 0
for manifest_rel in manifests:
    manifest = ROOT / manifest_rel
    seen = set()
    for number, raw in enumerate(manifest.read_text(encoding="utf-8").splitlines(), 1):
        if not raw:
            continue
        match = line_pattern.fullmatch(raw)
        if not match:
            raise SystemExit(f"{manifest_rel}:{number}: malformed checksum line")
        expected, rel = match.groups()
        if rel == manifest_rel:
            raise SystemExit(f"{manifest_rel}:{number}: checksum manifest must not include itself")
        if rel in seen:
            raise SystemExit(f"{manifest_rel}:{number}: duplicate path: {rel}")
        seen.add(rel)
        if rel not in tracked:
            raise SystemExit(f"{manifest_rel}:{number}: evidence path is not Git-tracked: {rel}")
        try:
            canonical = subprocess.check_output(["git", "-C", str(ROOT), "show", f"HEAD:{rel}"])
        except subprocess.CalledProcessError as error:
            raise SystemExit(f"{manifest_rel}:{number}: cannot read canonical evidence blob: {rel}") from error
        actual = hashlib.sha256(canonical).hexdigest()
        if actual != expected:
            raise SystemExit(f"{manifest_rel}:{number}: checksum mismatch: {rel}")
        checked += 1
print(f"evidence checksum validation passed: {checked} files across {len(manifests)} manifest(s)")
