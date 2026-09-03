#!/usr/bin/env python3
"""Run exactly six fresh, sequential warm-failover cycle commands.

The cycle command receives NEKO_FAILOVER_CYCLE_INDEX=1..6 and follows a
collector contract: exit 0 means exactly one valid evidence row was emitted,
including a valid failed experiment row; any nonzero exit means no row was
collected. Client/server exits live only inside the row. No address or secret is
part of the result contract. Execution stops at the first failed or malformed
cycle and retains the preceding valid prefix.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import time
from typing import Any, Callable

CYCLES = 6
MAX_BATCH_SECONDS = 570
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
STABLE_PARAMETER_FIELDS = ("record_count", "record_payload_bytes", "client_max_seconds", "server_max_seconds", "concurrency")
COUNT_FIELDS = (
    "udp_confirmed_records", "udp_confirmed_bytes", "uncertain_records",
    "uncertain_bytes", "replayed_records", "replayed_bytes",
    "confirmed_records", "confirmed_bytes", "duplicate_records",
    "duplicate_bytes", "lost_records", "lost_bytes", "conflicting_records",
    "conflicting_bytes",
)

class EvidenceError(ValueError):
    pass

def obj(value: Any, name: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise EvidenceError(f"{name} must be an object")
    return value

def exact_keys(value: dict[str, Any], required: set[str], optional: set[str], name: str) -> None:
    missing = required - value.keys()
    extra = value.keys() - required - optional
    if missing:
        raise EvidenceError(f"{name} missing: {','.join(sorted(missing))}")
    if extra:
        raise EvidenceError(f"{name} unknown: {','.join(sorted(extra))}")

def nonnegative_int(value: Any, name: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise EvidenceError(f"{name} must be a non-negative integer")
    return value

def validate_cycle(raw: Any, expected_index: int) -> dict[str, Any]:
    row = obj(raw, "cycle")
    required = {
        "cycle_index", "git_commit", "binary_sha256", "binary_bytes", "parameters",
        "semantic", "accounting", "timing", "classification", "result", "cleanup",
    }
    exact_keys(row, required, {"resources", "endpoint_provenance"}, "cycle")
    if row["cycle_index"] != expected_index:
        raise EvidenceError("cycle_index does not match sequential invocation")
    if not isinstance(row["git_commit"], str) or not HEX40.fullmatch(row["git_commit"]):
        raise EvidenceError("git_commit must be an exact lowercase 40-hex commit")
    if not isinstance(row["binary_sha256"], str) or not HEX64.fullmatch(row["binary_sha256"]):
        raise EvidenceError("binary_sha256 must be lowercase SHA-256")
    if nonnegative_int(row["binary_bytes"], "binary_bytes") == 0:
        raise EvidenceError("binary_bytes must be positive")
    if "endpoint_provenance" in row:
        provenance = row["endpoint_provenance"]
        if not isinstance(provenance, list) or len(provenance) != 2:
            raise EvidenceError("endpoint_provenance must contain server and client")
        for expected_role, endpoint in zip(("server", "client"), provenance):
            endpoint = obj(endpoint, "endpoint provenance")
            exact_keys(endpoint, {"role", "execution", "underlying_binary_path", "binary_sha256", "binary_bytes", "git_commit"}, set(), "endpoint provenance")
            if endpoint["role"] != expected_role or endpoint["execution"] not in ("local", "ssh"):
                raise EvidenceError("endpoint provenance role/execution is invalid")
            if not isinstance(endpoint["underlying_binary_path"], str) or not endpoint["underlying_binary_path"]:
                raise EvidenceError("endpoint underlying binary path is invalid")
            if any(endpoint[key] != row[key] for key in ("binary_sha256", "binary_bytes", "git_commit")):
                raise EvidenceError("endpoint provenance differs from cycle binary identity")

    parameters = obj(row["parameters"], "parameters")
    exact_keys(parameters, {"record_count", "record_payload_bytes", "client_max_seconds", "server_max_seconds", "concurrency"}, {"udp_port", "tcp_port"}, "parameters")
    for key in ("record_count", "record_payload_bytes", "client_max_seconds", "server_max_seconds"):
        if nonnegative_int(parameters[key], key) == 0:
            raise EvidenceError(f"{key} must be positive")
    if parameters["concurrency"] != 1:
        raise EvidenceError("concurrency must equal 1")
    for key in ("udp_port", "tcp_port"):
        if key in parameters and not 1024 <= nonnegative_int(parameters[key], key) <= 65535:
            raise EvidenceError(f"{key} must be an unprivileged port")

    semantic = obj(row["semantic"], "semantic")
    exact_keys(semantic, {"selected_version", "udp_negotiated", "udp_authenticated", "tcp_negotiated", "tcp_authenticated", "resume_validated", "readiness_proofs", "readiness_completed", "udp_confirmed_before_failure"}, set(), "semantic")
    if not isinstance(semantic["selected_version"], int) or semantic["selected_version"] < 0:
        raise EvidenceError("selected_version must be a non-negative integer")
    for key in ("udp_negotiated", "udp_authenticated", "tcp_negotiated", "tcp_authenticated", "resume_validated", "readiness_completed", "udp_confirmed_before_failure"):
        if not isinstance(semantic[key], bool):
            raise EvidenceError(f"{key} must be boolean")
    if nonnegative_int(semantic["readiness_proofs"], "readiness_proofs") > 3:
        raise EvidenceError("readiness_proofs exceeds the D064 gate")

    accounting = obj(row["accounting"], "accounting")
    exact_keys(accounting, set(COUNT_FIELDS), set(), "accounting")
    for key in COUNT_FIELDS:
        nonnegative_int(accounting[key], key)

    timing = obj(row["timing"], "timing")
    exact_keys(timing, {"failure_decided_at_us", "first_resumed_data_at_us", "first_resumed_ack_at_us", "recovery_latency_us"}, set(), "timing")
    for key, value in timing.items():
        if value is not None:
            nonnegative_int(value, key)
    if timing["failure_decided_at_us"] is not None and timing["first_resumed_data_at_us"] is not None:
        if timing["first_resumed_data_at_us"] < timing["failure_decided_at_us"]:
            raise EvidenceError("first resumed data predates failure decision")

    classification = obj(row["classification"], "classification")
    exact_keys(classification, {"failure_seam", "natural_blackhole", "pto_blackhole"}, set(), "classification")
    if classification != {"failure_seam": "controlled_application_reply_cessation", "natural_blackhole": False, "pto_blackhole": False}:
        raise EvidenceError("classification must remain controlled application-level reply cessation")

    result = obj(row["result"], "result")
    exact_keys(result, {"status", "client_exit_code", "server_exit_code", "failure_stage", "failure_reason"}, set(), "result")
    if result["status"] not in ("passed", "failed"):
        raise EvidenceError("result.status must be passed or failed")
    for key in ("client_exit_code", "server_exit_code"):
        if not isinstance(result[key], int):
            raise EvidenceError(f"{key} must be integer")
    for key in ("failure_stage", "failure_reason"):
        if result[key] is not None and not isinstance(result[key], str):
            raise EvidenceError(f"{key} must be string or null")

    cleanup = obj(row["cleanup"], "cleanup")
    exact_keys(cleanup, {"status", "client_process_reaped", "server_process_reaped", "listeners_remaining", "temporary_files_removed"}, set(), "cleanup")
    for key in ("client_process_reaped", "server_process_reaped", "temporary_files_removed"):
        if not isinstance(cleanup[key], bool):
            raise EvidenceError(f"{key} must be boolean")
    nonnegative_int(cleanup["listeners_remaining"], "listeners_remaining")
    verified = all((cleanup["client_process_reaped"], cleanup["server_process_reaped"], cleanup["temporary_files_removed"])) and cleanup["listeners_remaining"] == 0
    if cleanup["status"] not in ("verified", "failed") or (cleanup["status"] == "verified") != verified:
        raise EvidenceError("cleanup status contradicts required observations")

    passed = (all(semantic[key] for key in ("udp_negotiated", "udp_authenticated", "tcp_negotiated", "tcp_authenticated", "resume_validated", "readiness_completed", "udp_confirmed_before_failure"))
              and semantic["readiness_proofs"] == 3
              and accounting["confirmed_records"] == parameters["record_count"]
              and accounting["confirmed_bytes"] == parameters["record_count"] * parameters["record_payload_bytes"]
              and accounting["duplicate_records"] == accounting["duplicate_bytes"] == 0
              and accounting["lost_records"] == accounting["lost_bytes"] == 0
              and accounting["conflicting_records"] == accounting["conflicting_bytes"] == 0
              and accounting["uncertain_records"] == accounting["replayed_records"]
              and accounting["uncertain_bytes"] == accounting["replayed_bytes"]
              and all(timing[key] is not None for key in timing)
              and result["client_exit_code"] == result["server_exit_code"] == 0
              and cleanup["status"] == "verified")
    if (result["status"] == "passed") != passed:
        raise EvidenceError("result.status contradicts semantic/accounting/timing/cleanup evidence")
    return row

def run(argv: list[str], invoke: Callable[..., subprocess.CompletedProcess[str]] = subprocess.run, clock: Callable[[], float] = time.monotonic) -> tuple[dict[str, Any], int]:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", required=True)
    parser.add_argument("--max-batch-seconds", type=int, default=MAX_BATCH_SECONDS)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    if args.command[:1] == ["--"]:
        args.command = args.command[1:]
    if not args.command:
        parser.error("cycle command is required after --")
    if not 1 <= args.max_batch_seconds < 600:
        parser.error("--max-batch-seconds must be 1..599")

    started = clock()
    rows: list[dict[str, Any]] = []
    first_failure: dict[str, Any] | None = None
    for index in range(1, CYCLES + 1):
        # The command must create and reap a fresh server/client process pair.
        remaining = args.max_batch_seconds - (clock() - started)
        if remaining <= 0:
            first_failure = {"cycle_index": index, "kind": "batch_timeout", "detail": "batch deadline elapsed"}
            break
        env = os.environ.copy()
        env["NEKO_FAILOVER_CYCLE_INDEX"] = str(index)
        try:
            completed = invoke(args.command, text=True, capture_output=True, timeout=remaining, env=env, check=False)
            if completed.returncode != 0:
                if completed.stdout.strip():
                    raise EvidenceError("collector returned nonzero with a row")
                raise EvidenceError("collector returned nonzero without a valid row")
            raw = json.loads(completed.stdout)
            row = validate_cycle(raw, index)
            if rows and any(row[key] != rows[0][key] for key in ("git_commit", "binary_sha256", "binary_bytes")):
                raise EvidenceError("cycle identity or parameters differ from cycle 1")
            if rows and any(row["parameters"][key] != rows[0]["parameters"][key] for key in STABLE_PARAMETER_FIELDS):
                raise EvidenceError("cycle identity or parameters differ from cycle 1")
            rows.append(row)
            if row["result"]["status"] != "passed":
                first_failure = {"cycle_index": index, "kind": "cycle_failed", "detail": row["result"]["failure_stage"] or "reported failure"}
                break
        except (subprocess.TimeoutExpired, json.JSONDecodeError, EvidenceError) as error:
            first_failure = {"cycle_index": index, "kind": "invalid_cycle_evidence", "detail": str(error)[:240]}
            break

    elapsed_ms = int((clock() - started) * 1000)
    status = "passed" if len(rows) == CYCLES and first_failure is None else "failed"
    batch = {"schema": "nekomusume.repeated-warm-failover.v1", "expected_cycles": CYCLES, "concurrency": 1, "status": status, "completed_cycles": len(rows), "elapsed_ms": elapsed_ms, "cycles": rows, "first_failure": first_failure}
    # Atomic replace prevents a stale six-pass artifact surviving interruption.
    output = os.path.abspath(args.output)
    temporary = output + ".tmp"
    with open(temporary, "w", encoding="utf-8") as handle:
        json.dump(batch, handle, sort_keys=True, separators=(",", ":"))
        handle.write("\n")
    os.replace(temporary, output)
    return batch, 0 if status == "passed" else 1

def main() -> None:
    _, code = run(sys.argv[1:])
    raise SystemExit(code)

if __name__ == "__main__":
    main()
