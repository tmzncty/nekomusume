#!/usr/bin/env python3
"""Launch the repeated failover runner from a validated JSON argv plan.

No command is interpreted by a shell.  ``--preflight`` runs six harmless
synthetic collector dispatches through the real repeated runner and emits a
command-plan report explicitly marked as non-live evidence.
"""
from __future__ import annotations

import argparse
import json
import os
import pathlib
import subprocess
import sys
import tempfile
from typing import Any

HERE = pathlib.Path(__file__).resolve().parent
RUNNER = HERE / "run-repeated-warm-failover.py"
ADAPTER = HERE / "run-live-warm-failover-cycle.py"
CYCLES = 6
MAX_ARGV = 64
HEX40 = __import__("re").compile(r"^[0-9a-f]{40}$")

class PlanError(ValueError):
    pass

def argv(value: Any, name: str) -> list[str]:
    if (not isinstance(value, list) or not 1 <= len(value) <= MAX_ARGV or
            any(not isinstance(arg, str) or not arg or "\0" in arg or len(arg) > 4096 for arg in value)):
        raise PlanError(f"invalid {name}")
    return list(value)

def load_plan(path: str) -> dict[str, Any]:
    try:
        raw = json.loads(pathlib.Path(path).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise PlanError("invalid plan JSON") from exc
    if not isinstance(raw, dict) or set(raw) != {"git_commit", "binary", "cycles"}:
        raise PlanError("plan must contain exactly git_commit, binary, cycles")
    if not isinstance(raw["git_commit"], str) or not HEX40.fullmatch(raw["git_commit"]):
        raise PlanError("invalid git_commit")
    if not isinstance(raw["binary"], str) or not raw["binary"] or "\0" in raw["binary"]:
        raise PlanError("invalid binary")
    if not isinstance(raw["cycles"], list) or len(raw["cycles"]) != CYCLES:
        raise PlanError("plan must contain exactly six cycles")
    cycles = []
    for index, value in enumerate(raw["cycles"], 1):
        if not isinstance(value, dict) or set(value) != {"udp_port", "tcp_port", "server_command", "client_command", "cleanup_command"}:
            raise PlanError(f"invalid cycle {index}")
        udp, tcp = value["udp_port"], value["tcp_port"]
        if (isinstance(udp, bool) or isinstance(tcp, bool) or not isinstance(udp, int) or
                not isinstance(tcp, int) or not 40080 <= udp <= 40100 or
                not 40080 <= tcp <= 40100 or udp == tcp):
            raise PlanError(f"invalid cycle {index} ports")
        cycles.append({
            "udp_port": udp, "tcp_port": tcp,
            "server_command": argv(value["server_command"], f"cycle {index} server_command"),
            "client_command": argv(value["client_command"], f"cycle {index} client_command"),
            "cleanup_command": argv(value["cleanup_command"], f"cycle {index} cleanup_command"),
        })
    return {"git_commit": raw["git_commit"], "binary": raw["binary"], "cycles": cycles}

def adapter_env(plan: dict[str, Any], index: int) -> dict[str, str]:
    cycle = plan["cycles"][index - 1]
    return {
        "NEKO_FAILOVER_GIT_COMMIT": plan["git_commit"],
        "NEKO_FAILOVER_BINARY": plan["binary"],
        "NEKO_FAILOVER_UDP_PORT": str(cycle["udp_port"]),
        "NEKO_FAILOVER_TCP_PORT": str(cycle["tcp_port"]),
        "NEKO_FAILOVER_SERVER_COMMAND_JSON": json.dumps(cycle["server_command"], separators=(",", ":")),
        "NEKO_FAILOVER_CLIENT_COMMAND_JSON": json.dumps(cycle["client_command"], separators=(",", ":")),
        "NEKO_FAILOVER_CLEANUP_COMMAND_JSON": json.dumps(cycle["cleanup_command"], separators=(",", ":")),
    }

def preflight_row(plan: dict[str, Any], index: int) -> dict[str, Any]:
    cycle = plan["cycles"][index - 1]
    return {
        "cycle_index": index, "git_commit": plan["git_commit"],
        "binary_sha256": "0" * 64, "binary_bytes": 1,
        "parameters": {"record_count": 3, "record_payload_bytes": 16, "client_max_seconds": 12, "server_max_seconds": 15, "concurrency": 1, "udp_port": cycle["udp_port"], "tcp_port": cycle["tcp_port"]},
        "semantic": {"selected_version": 0, "udp_negotiated": True, "udp_authenticated": True, "tcp_negotiated": True, "tcp_authenticated": True, "resume_validated": True, "readiness_proofs": 3, "readiness_completed": True, "udp_confirmed_before_failure": True},
        "accounting": {"udp_confirmed_records": 1, "udp_confirmed_bytes": 16, "uncertain_records": 2, "uncertain_bytes": 32, "replayed_records": 2, "replayed_bytes": 32, "confirmed_records": 3, "confirmed_bytes": 48, "duplicate_records": 0, "duplicate_bytes": 0, "lost_records": 0, "lost_bytes": 0, "conflicting_records": 0, "conflicting_bytes": 0},
        "timing": {"failure_decided_at_us": 1, "first_resumed_data_at_us": 2, "first_resumed_ack_at_us": 3, "recovery_latency_us": 1},
        "classification": {"failure_seam": "controlled_application_reply_cessation", "natural_blackhole": False, "pto_blackhole": False},
        "result": {"status": "passed", "client_exit_code": 0, "server_exit_code": 0, "failure_stage": None, "failure_reason": None},
        "cleanup": {"status": "verified", "client_process_reaped": True, "server_process_reaped": True, "listeners_remaining": 0, "temporary_files_removed": True},
    }

def dispatch(plan_path: str, preflight: bool) -> int:
    plan = load_plan(plan_path)
    text = os.environ.get("NEKO_FAILOVER_CYCLE_INDEX", "")
    if not text.isdigit() or not 1 <= int(text) <= CYCLES:
        raise PlanError("invalid cycle index")
    index = int(text)
    if preflight:
        print(json.dumps(preflight_row(plan, index), separators=(",", ":")))
        return 0
    env = os.environ.copy()
    env.update(adapter_env(plan, index))
    return subprocess.run([sys.executable, str(ADAPTER)], env=env, check=False, shell=False).returncode

def run_outer(plan_path: str, output: str, preflight: bool) -> int:
    plan = load_plan(plan_path)
    command = [sys.executable, str(RUNNER), "--output", output, "--", sys.executable,
               str(pathlib.Path(__file__).resolve()), "--dispatch", plan_path]
    if preflight:
        command.append("--preflight-dispatch")
    completed = subprocess.run(command, check=False, shell=False)
    if not preflight:
        return completed.returncode
    try:
        batch = json.loads(pathlib.Path(output).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise PlanError("preflight runner did not produce a batch") from exc
    report = {
        "schema": "nekomusume.repeated-warm-failover-command-preflight.v1",
        "dry_run": True, "live_evidence": False,
        "runner_entered": batch.get("expected_cycles") == CYCLES,
        "completed_dispatches": batch.get("completed_cycles"),
        "runner_argv": command,
        "dispatches": [
            {"cycle_index": i, "adapter_argv": [sys.executable, str(ADAPTER)],
             "environment": adapter_env(plan, i)} for i in range(1, CYCLES + 1)
        ],
    }
    pathlib.Path(output).write_text(json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n", encoding="utf-8")
    return 0 if completed.returncode == 0 and report["runner_entered"] and report["completed_dispatches"] == CYCLES else 1

def fake_plan() -> dict[str, Any]:
    cycles = []
    for index in range(CYCLES):
        cycles.append({
            "udp_port": 40081 + index * 2, "tcp_port": 40080 + index * 2,
            "server_command": ["/harmless/fake binary", "failover-server", "failover-server,", "--label", "comma,quote\"kept"],
            "client_command": ["/harmless/fake binary", "failover-client", "--label", "client,'quoted'"],
            "cleanup_command": ["/harmless/fake cleanup", "--ports", f"{40080 + index * 2},{40081 + index * 2}"],
        })
    return {"git_commit": "0" * 40, "binary": "/harmless/fake binary", "cycles": cycles}

def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--plan")
    parser.add_argument("--output")
    parser.add_argument("--preflight", action="store_true")
    parser.add_argument("--dispatch")
    parser.add_argument("--preflight-dispatch", action="store_true", help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.dispatch:
        if args.plan or args.output or args.preflight:
            parser.error("--dispatch cannot be combined with outer options")
        return dispatch(args.dispatch, args.preflight_dispatch)
    if not args.output:
        parser.error("--output is required")
    if args.preflight:
        if args.plan:
            parser.error("--preflight does not accept --plan")
        with tempfile.TemporaryDirectory() as directory:
            plan_path = pathlib.Path(directory) / "plan.json"
            plan_path.write_text(json.dumps(fake_plan()), encoding="utf-8")
            return run_outer(str(plan_path), args.output, True)
    if not args.plan:
        parser.error("--plan is required unless --preflight is used")
    return run_outer(args.plan, args.output, False)

if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except PlanError as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(2)
