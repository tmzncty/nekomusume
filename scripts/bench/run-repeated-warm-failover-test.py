#!/usr/bin/env python3
from __future__ import annotations

import copy
import os
import importlib.util
import json
import pathlib
import subprocess
import tempfile

SCRIPT = pathlib.Path(__file__).with_name("run-repeated-warm-failover.py")
spec = importlib.util.spec_from_file_location("repeated_warm", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

def row(index: int, passed: bool = True) -> dict:
    return {
        "cycle_index": index,
        "git_commit": "a" * 40,
        "binary_sha256": "b" * 64,
        "binary_bytes": 1234,
        "endpoint_provenance": [{"role": role, "execution": "local", "underlying_binary_path": "/nonsecret/neko-cli", "binary_sha256": "b" * 64, "binary_bytes": 1234, "git_commit": "a" * 40} for role in ("server", "client")],
        "parameters": {"record_count": 3, "record_payload_bytes": 16, "client_max_seconds": 12, "server_max_seconds": 15, "concurrency": 1},
        "semantic": {"selected_version": 0, "udp_negotiated": True, "udp_authenticated": True, "tcp_negotiated": True, "tcp_authenticated": True, "resume_validated": True, "readiness_proofs": 3, "readiness_completed": True, "udp_confirmed_before_failure": True},
        "accounting": {"udp_confirmed_records": 1, "udp_confirmed_bytes": 16, "uncertain_records": 2, "uncertain_bytes": 32, "replayed_records": 2, "replayed_bytes": 32, "confirmed_records": 3, "confirmed_bytes": 48, "duplicate_records": 0, "duplicate_bytes": 0, "lost_records": 0, "lost_bytes": 0, "conflicting_records": 0, "conflicting_bytes": 0},
        "timing": {"failure_decided_at_us": 100, "first_resumed_data_at_us": 130, "first_resumed_ack_at_us": 140, "recovery_latency_us": 30},
        "classification": {"failure_seam": "controlled_application_reply_cessation", "natural_blackhole": False, "pto_blackhole": False},
        "result": {"status": "passed", "client_exit_code": 0, "server_exit_code": 0, "failure_stage": None, "failure_reason": None},
        "cleanup": {"status": "verified", "client_process_reaped": True, "server_process_reaped": True, "listeners_remaining": 0, "temporary_files_removed": True},
    } if passed else failed_row(index)

def failed_row(index: int) -> dict:
    value = row(index)
    value["semantic"]["readiness_proofs"] = 2
    value["semantic"]["readiness_completed"] = False
    value["timing"] = {key: None for key in value["timing"]}
    value["result"] = {"status": "failed", "client_exit_code": 2, "server_exit_code": 0, "failure_stage": "readiness", "failure_reason": "bounded fixture failure"}
    return value

class Clock:
    def __init__(self) -> None:
        self.now = 0.0
    def __call__(self) -> float:
        self.now += 0.01
        return self.now

def execute(rows: list[dict]) -> tuple[dict, int, list[int]]:
    called: list[int] = []
    def invoke(command, **kwargs):
        index = int(kwargs["env"]["NEKO_FAILOVER_CYCLE_INDEX"])
        called.append(index)
        value = rows[index - 1]
        return subprocess.CompletedProcess(command, 0, json.dumps(value), "")
    with tempfile.TemporaryDirectory() as directory:
        output = pathlib.Path(directory) / "result.json"
        batch, code = module.run(["--output", str(output), "--", "fixture-cycle"], invoke=invoke, clock=Clock())
        assert json.loads(output.read_text()) == batch
        assert not pathlib.Path(str(output) + ".tmp").exists()
        return batch, code, called

def expect_invalid(value: dict, text: str) -> None:
    try:
        module.validate_cycle(value, value.get("cycle_index", 1))
    except module.EvidenceError as error:
        assert text in str(error), str(error)
    else:
        raise AssertionError("invalid evidence accepted")

def test_six_success() -> None:
    batch, code, called = execute([row(i) for i in range(1, 7)])
    assert code == 0
    assert batch["status"] == "passed"
    assert batch["completed_cycles"] == 6
    assert batch["first_failure"] is None
    assert called == [1, 2, 3, 4, 5, 6]

def test_six_cycles_may_use_distinct_ports() -> None:
    rows = [row(i) for i in range(1, 7)]
    for index, value in enumerate(rows):
        value["parameters"].update(udp_port=40081 + index * 2, tcp_port=40080 + index * 2)
    batch, code, called = execute(rows)
    assert code == 0 and called == [1, 2, 3, 4, 5, 6]
    assert [item["parameters"]["tcp_port"] for item in batch["cycles"]] == [40080, 40082, 40084, 40086, 40088, 40090]

def test_middle_failure_preserves_prefix() -> None:
    rows = [row(1), row(2), failed_row(3), row(4), row(5), row(6)]
    batch, code, called = execute(rows)
    assert code == 1
    assert called == [1, 2, 3]
    assert [item["cycle_index"] for item in batch["cycles"]] == [1, 2, 3]
    assert batch["first_failure"] == {"cycle_index": 3, "kind": "cycle_failed", "detail": "readiness"}

def test_nonzero_stderr_is_bounded_private_and_validate_only():
    with tempfile.TemporaryDirectory() as td:
        private = pathlib.Path(td) / "private"
        def failing(*_args, **_kwargs):
            return subprocess.CompletedProcess([], 7, "", "password=supersecret ssh://user@192.0.2.9 /home/user/private.log\n" * 1000)
        output = private.parent / "result.json"
        old = os.environ.get("NEKO_PRIVATE_DIAGNOSTICS_DIR")
        os.environ["NEKO_PRIVATE_DIAGNOSTICS_DIR"] = str(private)
        try:
            batch, code = module.run(["--output", str(output), "--", "fake"], invoke=failing)
        finally:
            if old is None: os.environ.pop("NEKO_PRIVATE_DIAGNOSTICS_DIR", None)
            else: os.environ["NEKO_PRIVATE_DIAGNOSTICS_DIR"] = old
        assert code == 1 and batch["completed_cycles"] == 0
        diagnostic = batch["first_failure"]["diagnostic"]
        assert diagnostic["bytes"] <= module.MAX_DIAGNOSTIC_BYTES and diagnostic["truncated"]
        assert "supersecret" not in pathlib.Path(private / "cycle-1.stderr.txt").read_text()
        assert "192.0.2.9" not in pathlib.Path(private / "cycle-1.stderr.txt").read_text()
        assert "supersecret" not in output.read_text()
        assert oct((private.stat().st_mode & 0o777)) == "0o700"
        assert oct((private / "cycle-1.stderr.txt").stat().st_mode & 0o777) == "0o600"

def test_required_fields_fail_closed() -> None:
    value = row(1); del value["endpoint_provenance"]; expect_invalid(value, "endpoint_provenance")
    required_paths = [
        ("semantic", "udp_negotiated"), ("semantic", "udp_authenticated"), ("semantic", "tcp_negotiated"), ("semantic", "resume_validated"),
        ("timing", "first_resumed_ack_at_us"), ("cleanup", "listeners_remaining"),
    ] + [("accounting", key) for key in module.COUNT_FIELDS]
    for parent, key in required_paths:
        value = row(1)
        del value[parent][key]
        expect_invalid(value, "missing")

def test_cycle_identity_must_be_stable() -> None:
    rows = [row(i) for i in range(1, 7)]
    rows[1]["binary_sha256"] = "c" * 64
    for endpoint in rows[1]["endpoint_provenance"]: endpoint["binary_sha256"] = "c" * 64
    batch, code, called = execute(rows)
    assert code == 1
    assert called == [1, 2]
    assert [item["cycle_index"] for item in batch["cycles"]] == [1]
    assert batch["first_failure"]["cycle_index"] == 2
    assert "identity or parameters" in batch["first_failure"]["detail"]

def test_cleanup_failure_cannot_pass() -> None:
    value = row(1)
    value["cleanup"]["listeners_remaining"] = 1
    expect_invalid(value, "contradicts")
    value["cleanup"]["status"] = "failed"
    expect_invalid(value, "result.status contradicts")

def test_semantic_classification_is_controlled_only() -> None:
    for mutation in (
        {"failure_seam": "natural_udp_blackhole", "natural_blackhole": True, "pto_blackhole": False},
        {"failure_seam": "controlled_application_reply_cessation", "natural_blackhole": False, "pto_blackhole": True},
    ):
        value = row(1)
        value["classification"] = mutation
        expect_invalid(value, "controlled application-level")

def test_privacy_and_unknown_fields() -> None:
    value = row(1)
    serialized = json.dumps(value)
    for forbidden in ("address", "hostname", "username", "password", "private_key", "auth_token", "secret"):
        assert f'"{forbidden}"' not in serialized
    value["server_address"] = "192.0.2.1"
    expect_invalid(value, "unknown")
    schema = json.loads(SCRIPT.parent.parent.parent.joinpath("schema/repeated-warm-failover.v1.json").read_text())
    cycle_required = schema["$defs"]["cycle"]["required"]
    assert not any(any(word in field for word in ("address", "host", "secret", "key", "token")) for field in cycle_required)

def main() -> None:
    tests = [value for name, value in sorted(globals().items()) if name.startswith("test_")]
    for test in tests:
        test()

    # Collector contract: a valid failed experiment row is retained only on wrapper exit 0.
    with tempfile.TemporaryDirectory() as td:
        failed = row(1, False)
        def valid_failed(*_args, **_kwargs):
            return subprocess.CompletedProcess([], 0, json.dumps(failed), "")
        batch, code = module.run(["--output", str(pathlib.Path(td) / "result.json"), "--", "fake"], invoke=valid_failed)
        assert code == 1 and batch["completed_cycles"] == 1
        assert batch["cycles"][0]["result"]["client_exit_code"] == 2
        assert batch["first_failure"]["kind"] == "cycle_failed"

    # Nonzero collector + row is contradictory and the row is not retained.
    with tempfile.TemporaryDirectory() as td:
        def nonzero_with_row(*_args, **_kwargs):
            return subprocess.CompletedProcess([], 1, json.dumps(row(1, False)), "")
        batch, code = module.run(["--output", str(pathlib.Path(td) / "result.json"), "--", "fake"], invoke=nonzero_with_row)
        assert code == 1 and batch["completed_cycles"] == 0
        assert batch["first_failure"]["kind"] == "invalid_cycle_evidence"
        assert "nonzero with a row" in batch["first_failure"]["detail"]

    # Exit 0 + malformed output is also a collector contradiction.
    with tempfile.TemporaryDirectory() as td:
        def zero_malformed(*_args, **_kwargs):
            return subprocess.CompletedProcess([], 0, "not-json", "")
        batch, code = module.run(["--output", str(pathlib.Path(td) / "result.json"), "--", "fake"], invoke=zero_malformed)
        assert code == 1 and batch["completed_cycles"] == 0
        assert batch["first_failure"]["kind"] == "invalid_cycle_evidence"
    print(f"repeated warm failover tests passed: {len(tests)}")

if __name__ == "__main__":
    main()
