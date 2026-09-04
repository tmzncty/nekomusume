#!/usr/bin/env python3
"""Validate the Era-4 evidence-opportunity taxonomy without making release claims."""
import json
import sys
from pathlib import Path

path = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("docs/era4-ledger-2026-08-30.json")
data = json.loads(path.read_text(encoding="utf-8"))
tracks = data["tracks"]
allowed = set(data["closure"]["allowed_classifications"])
actual = {t["classification"] for t in tracks}
if not actual <= allowed:
    raise SystemExit(f"unknown classifications: {sorted(actual - allowed)}")
open_rows = [t["id"] for t in tracks if t["classification"] == "OPEN_READY"]
if open_rows != data["closure"]["open_ready_rows"]:
    raise SystemExit("open_ready_rows disagrees with track classifications")
sufficient = [t["id"] for t in tracks if t["classification"] == "ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION"]
if sufficient != data["closure"].get("already_sufficient_rows", []):
    raise SystemExit("already_sufficient_rows disagrees with track classifications")
for track in tracks:
    cls = track["classification"]
    if cls == "OPEN_READY":
        for field in ("evidence_needed", "next_action", "requires", "execution_scope"):
            if not isinstance(track.get(field), str) or not track[field].strip():
                raise SystemExit(f"OPEN_READY {track['id']} lacks concrete {field}")
        if track["execution_scope"] not in {"local", "vps"}:
            raise SystemExit(f"OPEN_READY {track['id']} has invalid execution_scope")
    elif cls in {"BLOCKED_IMPLEMENTATION", "BLOCKED_ENVIRONMENT", "BLOCKED_ORCHESTRATION_CURRENT_LINE", "GOVERNANCE_GATE"}:
        if not (track.get("classification_basis") or track.get("gate")):
            raise SystemExit(f"blocked {track['id']} lacks explanation")
    elif cls == "ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION" and track["status"] not in {"ready", "era3-complete", "in-progress"}:
        raise SystemExit(f"sufficient {track['id']} has unexpected status")
print(f"Era-4 closure taxonomy validation passed: {len(open_rows)} open-ready, {len(sufficient)} already-sufficient")
