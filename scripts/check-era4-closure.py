#!/usr/bin/env python3
"""Validate the Era-4 evidence-opportunity taxonomy without making release claims."""
import json
import sys
from pathlib import Path

path = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("docs/era4-ledger-2026-08-30.json")
data = json.loads(path.read_text(encoding="utf-8"))
tracks = data["tracks"]
by_id = {track["id"]: track for track in tracks}
if len(by_id) != len(tracks):
    raise SystemExit("duplicate track id")
allowed = set(data["closure"]["allowed_classifications"])
actual = {track["classification"] for track in tracks}
if not actual <= allowed:
    raise SystemExit(f"unknown classifications: {sorted(actual - allowed)}")

open_rows = [track["id"] for track in tracks if track["classification"] == "OPEN_READY"]
if open_rows != data["closure"]["open_ready_rows"]:
    raise SystemExit("open_ready_rows disagrees with track classifications")
sufficient = [track["id"] for track in tracks if track["classification"] == "ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION"]
if sufficient != data["closure"].get("already_sufficient_rows", []):
    raise SystemExit("already_sufficient_rows disagrees with track classifications")
blocked_dependencies = [track["id"] for track in tracks if track["classification"] == "BLOCKED_DEPENDENCY"]
if blocked_dependencies != data["closure"].get("blocked_dependency_rows", []):
    raise SystemExit("blocked_dependency_rows disagrees with track classifications")

blocking = {
    "BLOCKED_DEPENDENCY",
    "BLOCKED_ENVIRONMENT",
    "BLOCKED_IMPLEMENTATION",
    "BLOCKED_ORCHESTRATION_CURRENT_LINE",
    "GOVERNANCE_GATE",
}
for track in tracks:
    unknown_dependencies = sorted(set(track["depends_on"]) - by_id.keys())
    if unknown_dependencies:
        raise SystemExit(f"{track['id']} has unknown dependencies: {unknown_dependencies}")
    cls = track["classification"]
    if cls == "OPEN_READY":
        for field in ("evidence_needed", "next_action", "requires", "execution_scope"):
            if not isinstance(track.get(field), str) or not track[field].strip():
                raise SystemExit(f"OPEN_READY {track['id']} lacks concrete {field}")
        if track["execution_scope"] not in {"local", "vps"}:
            raise SystemExit(f"OPEN_READY {track['id']} has invalid execution_scope")
        blocked = [dependency for dependency in track["depends_on"] if by_id[dependency]["classification"] in blocking]
        if blocked:
            raise SystemExit(f"OPEN_READY {track['id']} has blocked dependencies: {blocked}")
    elif cls in blocking:
        if not (track.get("classification_basis") or track.get("gate")):
            raise SystemExit(f"blocked {track['id']} lacks explanation")
        if cls == "BLOCKED_DEPENDENCY":
            blocked = [dependency for dependency in track["depends_on"] if by_id[dependency]["classification"] in blocking]
            if not blocked:
                raise SystemExit(f"BLOCKED_DEPENDENCY {track['id']} has no blocked dependency")
            for field in ("evidence_needed", "requires"):
                if not isinstance(track.get(field), str) or not track[field].strip():
                    raise SystemExit(f"BLOCKED_DEPENDENCY {track['id']} lacks concrete {field}")
    elif cls == "ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION" and track["status"] not in {"ready", "era3-complete", "in-progress"}:
        raise SystemExit(f"sufficient {track['id']} has unexpected status")

print(
    "Era-4 closure taxonomy validation passed: "
    f"{len(open_rows)} open-ready, {len(sufficient)} already-sufficient, "
    f"{len(blocked_dependencies)} dependency-blocked"
)
