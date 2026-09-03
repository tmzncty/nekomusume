#!/usr/bin/env python3
"""Collect exactly one real failover-server/client cycle as one JSON row.

COLLECTOR CONTRACT: exit 0 means that one schema-valid evidence row was emitted.
It does not mean the experiment passed.  Client/server exits and experiment
success are reported only in ``result``.  A nonzero adapter exit means no valid
row was collected, and stdout is empty.

Secret/config input is supplied only through the environment:
NEKO_FAILOVER_{SERVER,CLIENT,CLEANUP}_COMMAND_JSON are JSON argv arrays.  The
server/client arrays must invoke the existing failover-server/failover-client
CLI modes; the server must select --cease-udp-replies-after 1 and the client
must select --automatic-health-failover.  CLEANUP must print exactly
{"listeners_remaining":N}.  NEKO_FAILOVER_BINARY identifies the local binary
whose SHA-256/size are recorded, and NEKO_FAILOVER_GIT_COMMIT is its exact
commit.  Logs and sampler output live in a mode-0700 temporary directory and
are deleted before the sole stdout row is emitted.
"""
from __future__ import annotations

import hashlib
import json
import os
import pathlib
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from typing import Any

HERE = pathlib.Path(__file__).resolve().parent
SAMPLER = HERE / "process-resource-sampler.py"
HEX40 = re.compile(r"^[0-9a-f]{40}$")
MAX_ARGV = 64
PARAMETERS = {
    "record_count": 3,
    "record_payload_bytes": 16,
    "client_max_seconds": 12,
    "server_max_seconds": 15,
    "concurrency": 1,
}

class CollectionError(ValueError):
    pass

def command(name: str) -> list[str]:
    try:
        value = json.loads(os.environ[name])
    except (KeyError, json.JSONDecodeError) as exc:
        raise CollectionError(f"invalid {name}") from exc
    if (not isinstance(value, list) or not 1 <= len(value) <= MAX_ARGV or
            any(not isinstance(arg, str) or not arg or "\0" in arg or len(arg) > 4096 for arg in value)):
        raise CollectionError(f"invalid {name}")
    return value

def requires(argv: list[str], token: str, value: str | None = None) -> None:
    if token not in argv:
        raise CollectionError(f"command missing {token}")
    if value is not None:
        index = argv.index(token)
        if index + 1 >= len(argv) or argv[index + 1] != value:
            raise CollectionError(f"command requires {token} {value}")

def event_objects(text: str) -> list[dict[str, Any]]:
    values = []
    for line in text.splitlines():
        if not line.startswith("{"):
            continue
        try:
            value = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(value, dict) and isinstance(value.get("event"), str):
            values.append(value)
    return values

def carrier(text: str, name: str) -> list[str]:
    return [line for line in text.splitlines() if line.startswith("carrier_event ") and f"name={name} " in line]

def version(text: str, name: str) -> int | None:
    rows = carrier(text, name)
    match = re.search(r"(?:^| )version=([0-9]+)(?: |$)", rows[-1]) if rows else None
    return int(match.group(1)) if match else None

def one_event(events: list[dict[str, Any]], name: str) -> dict[str, Any] | None:
    matches = [item for item in events if item.get("event") == name]
    return matches[-1] if matches else None

def count_events(events: list[dict[str, Any]], name: str) -> int:
    return sum(item.get("event") == name for item in events)

def exit_code(resource: dict[str, Any] | None, fallback: int) -> int:
    if not resource:
        return fallback
    result = resource.get("exit", {})
    if isinstance(result.get("code"), int):
        return result["code"]
    if isinstance(result.get("signal"), int):
        return 128 + result["signal"]
    return fallback

def sampled_command(tmp: pathlib.Path, role: str, identity: str, ports: list[int], argv: list[str]) -> tuple[list[str], pathlib.Path]:
    output = tmp / f"{role}-resource.json"
    wrapper = [sys.executable, str(SAMPLER), "--experiment-id", f"warm-cycle-{os.environ['NEKO_FAILOVER_CYCLE_INDEX']}-{role}",
               "--implementation", "nekomusume", "--role", role, "--identity", identity,
               "--application-bytes", "48", "--interval-ms", "20", "--max-seconds", "20", "--output", str(output)]
    for port in ports:
        wrapper += ["--owned-port", str(port)]
    return wrapper + ["--"] + argv, output

def load_json(path: pathlib.Path) -> dict[str, Any] | None:
    try:
        value = json.loads(path.read_text())
        return value if isinstance(value, dict) else None
    except (OSError, json.JSONDecodeError):
        return None

def terminate(process: subprocess.Popen[bytes] | None) -> None:
    if process is None or process.poll() is not None:
        return
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        return
    try:
        process.wait(timeout=1)
    except subprocess.TimeoutExpired:
        try: os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError: pass
        process.wait(timeout=1)

def main() -> int:
    emitted = False
    tmp_path: pathlib.Path | None = None
    server: subprocess.Popen[bytes] | None = None
    client: subprocess.Popen[bytes] | None = None
    try:
        index_text = os.environ.get("NEKO_FAILOVER_CYCLE_INDEX", "")
        if not index_text.isdigit() or not 1 <= int(index_text) <= 6:
            raise CollectionError("invalid cycle index")
        index = int(index_text)
        commit = os.environ.get("NEKO_FAILOVER_GIT_COMMIT", "")
        if not HEX40.fullmatch(commit):
            raise CollectionError("invalid git commit")
        binary = pathlib.Path(os.environ.get("NEKO_FAILOVER_BINARY", ""))
        if not binary.is_file():
            raise CollectionError("binary is not a file")
        binary_bytes = binary.stat().st_size
        if binary_bytes <= 0:
            raise CollectionError("binary is empty")
        binary_sha = hashlib.sha256(binary.read_bytes()).hexdigest()
        server_argv, client_argv, cleanup_argv = (command(name) for name in (
            "NEKO_FAILOVER_SERVER_COMMAND_JSON", "NEKO_FAILOVER_CLIENT_COMMAND_JSON", "NEKO_FAILOVER_CLEANUP_COMMAND_JSON"))
        requires(server_argv, "failover-server")
        requires(server_argv, "--diagnostic-id")
        requires(server_argv, "--cease-udp-replies-after", "1")
        requires(client_argv, "failover-client")
        requires(client_argv, "--diagnostic-id")
        requires(client_argv, "--automatic-health-failover")
        udp_port = int(os.environ.get("NEKO_FAILOVER_UDP_PORT", "40081"))
        tcp_port = int(os.environ.get("NEKO_FAILOVER_TCP_PORT", "40080"))
        if not all(40080 <= value <= 40100 for value in (udp_port, tcp_port)) or udp_port == tcp_port:
            raise CollectionError("ports outside CLI bounded range")
        parameters = dict(PARAMETERS, udp_port=udp_port, tcp_port=tcp_port)
        tmp_path = pathlib.Path(tempfile.mkdtemp(prefix=f"neko-warm-cycle-{index}-"))
        os.chmod(tmp_path, 0o700)
        identity = f"git:{commit}"
        server_cmd, server_resource_path = sampled_command(tmp_path, "server", identity, [udp_port, tcp_port], server_argv)
        client_cmd, client_resource_path = sampled_command(tmp_path, "client", identity, [], client_argv)
        server_log = open(tmp_path / "server.log", "xb")
        client_log = open(tmp_path / "client.log", "xb")
        try:
            server = subprocess.Popen(server_cmd, stdout=server_log, stderr=subprocess.STDOUT, start_new_session=True)
            time.sleep(float(os.environ.get("NEKO_FAILOVER_SERVER_STARTUP_SECONDS", "0.2")))
            client = subprocess.Popen(client_cmd, stdout=client_log, stderr=subprocess.STDOUT, start_new_session=True)
            try: client.wait(timeout=22)
            except subprocess.TimeoutExpired: terminate(client)
            try: server.wait(timeout=3)
            except subprocess.TimeoutExpired: terminate(server)
        finally:
            terminate(client); terminate(server)
            server_log.close(); client_log.close()
        cleanup = subprocess.run(cleanup_argv, capture_output=True, text=True, timeout=5, check=False, start_new_session=True)
        try:
            cleanup_value = json.loads(cleanup.stdout)
            listeners = cleanup_value["listeners_remaining"]
            if set(cleanup_value) != {"listeners_remaining"} or isinstance(listeners, bool) or not isinstance(listeners, int) or listeners < 0:
                raise ValueError
        except (json.JSONDecodeError, KeyError, TypeError, ValueError):
            raise CollectionError("cleanup command did not return bounded evidence")

        server_text = (tmp_path / "server.log").read_text(errors="replace")
        client_text = (tmp_path / "client.log").read_text(errors="replace")
        se, ce = event_objects(server_text), event_objects(client_text)
        sr, cr = load_json(server_resource_path), load_json(client_resource_path)
        server_exit = exit_code(sr, server.returncode if server and server.returncode is not None else 124)
        client_exit = exit_code(cr, client.returncode if client and client.returncode is not None else 124)
        readiness = {item.get("seq") for item in ce if item.get("event") == "tcp_warm_readiness" and item.get("seq") in (1, 2, 3)}
        timing_event = one_event(ce, "failover_timing")
        client_start, server_start = one_event(ce, "start"), one_event(se, "start")
        summary_client, summary_server = one_event(ce, "summary"), one_event(se, "summary")
        if client_start and server_start:
            starts_match = all(client_start.get(key) == server_start.get(key) for key in ("count", "record_payload_bytes"))
            if starts_match and isinstance(client_start.get("count"), int) and isinstance(client_start.get("record_payload_bytes"), int):
                parameters["record_count"] = client_start["count"]
                parameters["record_payload_bytes"] = client_start["record_payload_bytes"]
            parameters["client_max_seconds"] = client_start.get("max_seconds", PARAMETERS["client_max_seconds"])
            parameters["server_max_seconds"] = server_start.get("max_seconds", PARAMETERS["server_max_seconds"])
        udp_acks = count_events(ce, "udp_delivery_ack_validated")
        uncertain = [item for item in ce if item.get("event") == "udp_uncertain_range_sent" and isinstance(item.get("len"), int)]
        tcp_acks = count_events(ce, "tcp_delivery_ack_validated")
        ordered = bool(carrier(client_text, "ordered_records_complete"))
        server_resumed = bool(carrier(server_text, "tcp_resumed"))
        complete = (ordered and server_resumed and summary_client is not None and summary_server is not None and
                    summary_client.get("records") == summary_server.get("records") == 3 and
                    summary_client.get("application_bytes_total") == summary_server.get("application_bytes_total") == 48)
        selected = version(server_text, "udp_negotiated")
        tcp_selected = version(server_text, "tcp_negotiated")
        semantic = {
            "selected_version": selected if selected is not None else 0,
            "udp_negotiated": selected is not None,
            "udp_authenticated": bool(carrier(server_text, "udp_authenticated")) and bool(carrier(client_text, "udp_authenticated")),
            "tcp_negotiated": tcp_selected is not None and tcp_selected == selected,
            "tcp_authenticated": bool(carrier(server_text, "tcp_authenticated")),
            "resume_validated": bool(carrier(server_text, "tcp_resume_validated")),
            "readiness_proofs": len(readiness),
            "readiness_completed": readiness == {1, 2, 3} and any("application_data=0" in line for line in carrier(client_text, "tcp_warm")),
            "udp_confirmed_before_failure": udp_acks >= 1,
        }
        # The CLI emits the first uncertain send as a representative runtime event;
        # the real start/count and validated acknowledgements bound the whole range.
        accounting_event = one_event(ce, "failover_accounting")
        accounting = {key: accounting_event.get(key, 0) if accounting_event else 0 for key in (
            "udp_confirmed_records", "udp_confirmed_bytes", "uncertain_records", "uncertain_bytes",
            "replayed_records", "replayed_bytes", "confirmed_records", "confirmed_bytes",
            "duplicate_records", "duplicate_bytes", "lost_records", "lost_bytes",
            "conflicting_records", "conflicting_bytes")}
        failure_us = timing_event.get("failure_decided_at_us") if timing_event else None
        data_us = timing_event.get("first_resumed_data_accepted_us") if timing_event else None
        latency_us = timing_event.get("recovery_latency_us") if timing_event else None
        timing = {"failure_decided_at_us": failure_us, "first_resumed_data_at_us": data_us,
                  "first_resumed_ack_at_us": timing_event.get("first_resumed_ack_at_us") if timing_event else None, "recovery_latency_us": latency_us}
        client_reaped = bool(cr and cr.get("cleanup", {}).get("complete"))
        server_reaped = bool(sr and sr.get("cleanup", {}).get("complete"))
        cleanup_verified = cleanup.returncode == 0 and listeners == 0 and client_reaped and server_reaped
        evidence_pass = (all(semantic[key] for key in ("udp_negotiated", "udp_authenticated", "tcp_negotiated", "tcp_authenticated", "resume_validated", "readiness_completed", "udp_confirmed_before_failure")) and
                         semantic["readiness_proofs"] == 3 and accounting["confirmed_records"] == parameters["record_count"] and accounting["confirmed_bytes"] == parameters["record_count"] * parameters["record_payload_bytes"] and
                         accounting["uncertain_records"] == accounting["replayed_records"] and accounting["uncertain_bytes"] == accounting["replayed_bytes"] and
                         all(accounting[key] == 0 for key in ("duplicate_records", "duplicate_bytes", "lost_records", "lost_bytes", "conflicting_records", "conflicting_bytes")) and
                         all(value is not None for value in timing.values()) and client_exit == server_exit == 0 and cleanup_verified)
        if evidence_pass:
            failure_stage = failure_reason = None
        elif client_exit != 0:
            failure_stage, failure_reason = "client", "client_timeout" if client_exit == 124 else "client_nonzero"
        elif server_exit != 0:
            failure_stage, failure_reason = "server", "server_nonzero"
        elif not cleanup_verified:
            failure_stage, failure_reason = "cleanup", "cleanup_not_verified"
        else:
            failure_stage, failure_reason = "evidence", "required_runtime_evidence_missing"
        row = {
            "cycle_index": index, "git_commit": commit, "binary_sha256": binary_sha, "binary_bytes": binary_bytes,
            "parameters": parameters, "semantic": semantic, "accounting": accounting, "timing": timing,
            "classification": {"failure_seam": "controlled_application_reply_cessation", "natural_blackhole": False, "pto_blackhole": False},
            "result": {"status": "passed" if evidence_pass else "failed", "client_exit_code": client_exit, "server_exit_code": server_exit,
                       "failure_stage": failure_stage, "failure_reason": failure_reason},
            "cleanup": {"status": "verified" if cleanup_verified else "failed", "client_process_reaped": client_reaped,
                        "server_process_reaped": server_reaped, "listeners_remaining": listeners, "temporary_files_removed": True},
            "resources": {"client": cr, "server": sr} if cr and sr else None,
        }
        shutil.rmtree(tmp_path); tmp_path = None
        print(json.dumps(row, sort_keys=True, separators=(",", ":")))
        emitted = True
        return 0
    except (CollectionError, OSError, subprocess.SubprocessError, ValueError) as exc:
        print(f"live failover collector: {exc}", file=sys.stderr)
        return 2
    finally:
        terminate(client); terminate(server)
        if tmp_path is not None:
            shutil.rmtree(tmp_path, ignore_errors=True)
        if not emitted:
            # Collector failures never leave a partial or diagnostic stdout row.
            pass

if __name__ == "__main__":
    raise SystemExit(main())
