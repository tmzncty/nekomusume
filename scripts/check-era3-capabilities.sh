#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
MANIFEST="${1:-$ROOT/docs/era3-capabilities.v1.json}"
NOTE="${2:-$ROOT/docs/era3-closure-2026-08-30.md}"
python3 - "$ROOT" "$MANIFEST" "$NOTE" <<'PY'
import json, pathlib, re, subprocess, sys
root = pathlib.Path(sys.argv[1]).resolve()
manifest = pathlib.Path(sys.argv[2]).resolve()
note = pathlib.Path(sys.argv[3]).resolve()
data = json.loads(manifest.read_text(encoding="utf-8"))
if data.get("schema") != "nekomusume.era3-capabilities.v1":
    raise SystemExit("invalid Era 3 capability schema")
if not re.fullmatch(r"[0-9a-f]{40}", data.get("parent_commit", "")):
    raise SystemExit("invalid Era 3 parent commit")
items = data.get("capabilities")
if not isinstance(items, list) or not items:
    raise SystemExit("Era 3 capability list is empty")
seen = set(); note_text = note.read_text(encoding="utf-8")
unsupported = {"zero-rtt", "concurrent-heterogeneous-multipath", "production-readiness", "nat-endpoint-change", "ipv6-surviving-session-failover", "long-soak", "performance-superiority"}
for item in items:
    ident = item.get("id")
    if not isinstance(ident, str) or not re.fullmatch(r"[a-z0-9-]+", ident) or ident in seen:
        raise SystemExit(f"invalid or duplicate capability id: {ident}")
    seen.add(ident)
    supported = item.get("supported")
    klass = item.get("evidence_class")
    if type(supported) is not bool or klass not in {"live_vps_verified", "deterministic_local", "negative_or_inconclusive"}:
        raise SystemExit(f"invalid capability classification: {ident}")
    if supported == (klass == "negative_or_inconclusive"):
        raise SystemExit(f"support/classification mismatch: {ident}")
    evidence = item.get("evidence", "")
    if not evidence or evidence.startswith("/") or "\\" in evidence or ".." in pathlib.PurePosixPath(evidence).parts:
        raise SystemExit(f"unsafe evidence path: {ident}: {evidence}")
    candidate = (root / evidence).resolve()
    try: candidate.relative_to(root)
    except ValueError: raise SystemExit(f"evidence escapes repository: {ident}")
    if not candidate.is_file(): raise SystemExit(f"missing evidence: {ident}: {evidence}")
    rel = candidate.relative_to(root).as_posix()
    if subprocess.run(["git", "-C", str(root), "ls-files", "--error-unmatch", "--", rel], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode:
        raise SystemExit(f"untracked evidence: {ident}: {evidence}")
    if ident not in note_text:
        raise SystemExit(f"closure note omits capability: {ident}")
missing = unsupported - seen
if missing: raise SystemExit("missing unsupported capabilities: " + ", ".join(sorted(missing)))
for ident in unsupported:
    item = next(x for x in items if x["id"] == ident)
    if item["supported"] or item["evidence_class"] != "negative_or_inconclusive":
        raise SystemExit(f"unsupported capability promoted: {ident}")
print(f"Era 3 capability validation passed: {len(items)} entries")
PY
