#!/usr/bin/env python3
"""Collect exactly one real failover-server/client cycle as one JSON row.

COLLECTOR CONTRACT: exit 0 means that one schema-valid evidence row was emitted.
It does not mean the experiment passed.  Client/server exits and experiment
success are reported only in ``result``.  A nonzero adapter exit means no valid
row was collected, and stdout is empty.

Secret/config input is supplied only through the environment:
NEKO_FAILOVER_CLEANUP_COMMAND_JSON is a JSON argv array.  Server/client may use
the legacy local command variables, or NEKO_FAILOVER_ENDPOINTS_JSON: exactly
one structured descriptor per role with execution local/ssh, underlying binary
path/hash/size/commit, argv, and (for ssh) a bounded transport argv.  Local
execution retains same-file verification.  SSH receives one JSON request on
stdin and must run remote-endpoint-exec.py, which verifies the remote underlying
file immediately before direct shell-free spawn.  The server must select
--cease-udp-replies-after 1 and the client --automatic-health-failover. CLEANUP
must print exactly {"listeners_remaining":N}. NEKO_FAILOVER_BINARY identifies
the locally staged bytes whose SHA-256/size every endpoint must declare.
NEKO_FAILOVER_GIT_COMMIT must be the exact HEAD of the checkout containing
this adapter.  Logs and sampler output live in a mode-0700 temporary directory
and are deleted before the sole stdout row is emitted.
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
REMOTE_EXEC_PROTOCOL = "nekomusume.remote-exec.v1"
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

def executable_path(value: str) -> pathlib.Path:
    candidate = pathlib.Path(value).expanduser()
    if not candidate.is_absolute() and candidate.parent == pathlib.Path("."):
        found = shutil.which(value)
        if found is None:
            raise CollectionError("command executable is unavailable")
        candidate = pathlib.Path(found)
    try:
        return candidate.resolve(strict=True)
    except OSError as exc:
        raise CollectionError("command executable is unavailable") from exc

def require_same_executable(argv: list[str], binary: pathlib.Path) -> None:
    try:
        same = os.path.samefile(executable_path(argv[0]), binary)
    except OSError as exc:
        raise CollectionError("cannot compare command executable") from exc
    if not same:
        raise CollectionError("command executable differs from declared binary")

def valid_argv(value: Any, name: str) -> list[str]:
    if (not isinstance(value, list) or not 1 <= len(value) <= MAX_ARGV or
            any(not isinstance(arg, str) or not arg or "\0" in arg or len(arg) > 4096 for arg in value)):
        raise CollectionError(f"invalid {name}")
    return value

def endpoints(binary: pathlib.Path, binary_sha: str, binary_bytes: int, commit: str) -> tuple[dict[str, Any], dict[str, Any]]:
    raw = os.environ.get("NEKO_FAILOVER_ENDPOINTS_JSON")
    if raw is None:
        server, client = (command(name) for name in ("NEKO_FAILOVER_SERVER_COMMAND_JSON", "NEKO_FAILOVER_CLIENT_COMMAND_JSON"))
        require_same_executable(server, binary); require_same_executable(client, binary)
        def legacy(role: str, argv: list[str]) -> dict[str, Any]:
            return {"role": role, "execution": "local", "binary": {"path": str(binary.resolve()), "sha256": binary_sha, "bytes": binary_bytes, "git_commit": commit}, "argv": argv}
        return legacy("server", server), legacy("client", client)
    try:
        value = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise CollectionError("invalid NEKO_FAILOVER_ENDPOINTS_JSON") from exc
    if not isinstance(value, list) or len(value) != 2:
        raise CollectionError("invalid endpoint descriptors")
    result: dict[str, dict[str, Any]] = {}
    for item in value:
        role=item.get("role") if isinstance(item, dict) else None
        execution=item.get("execution") if isinstance(item, dict) else None
        expected_keys = {"role", "execution", "binary", "argv"} | ({"transport_argv", "ssh_executable"} if execution == "ssh" else set())
        if not isinstance(item, dict) or set(item) != expected_keys:
            raise CollectionError("invalid endpoint descriptor")
        declared=item.get("binary")
        if role not in ("server", "client") or role in result or execution not in ("local", "ssh"):
            raise CollectionError("invalid endpoint role or execution")
        if (not isinstance(declared, dict) or set(declared) != {"path", "sha256", "bytes", "git_commit"} or
                not isinstance(declared["path"], str) or not declared["path"] or "\0" in declared["path"] or
                declared["sha256"] != binary_sha or declared["bytes"] != binary_bytes or declared["git_commit"] != commit):
            raise CollectionError("endpoint binary differs from staged binary identity")
        argv=valid_argv(item.get("argv"), "endpoint argv")
        if argv[0] != declared["path"]:
            raise CollectionError("endpoint argv executable differs from underlying binary")
        normalized={"role":role,"execution":execution,"binary":declared,"argv":argv}
        if execution == "local":
            if "transport_argv" in item: raise CollectionError("local endpoint has transport")
            require_same_executable(argv,binary)
        else:
            transport=valid_argv(item.get("transport_argv"),"SSH transport argv")
            declared_ssh=item.get("ssh_executable")
            if not isinstance(declared_ssh, str) or not declared_ssh:
                raise CollectionError("invalid SSH executable")
            try:
                if not os.path.samefile(executable_path(transport[0]), executable_path(declared_ssh)):
                    raise CollectionError("SSH transport executable differs from declared SSH executable")
            except OSError as exc:
                raise CollectionError("cannot compare SSH transport executable") from exc
            normalized["transport_argv"]=transport
        result[role]=normalized
    if set(result) != {"server","client"}: raise CollectionError("missing endpoint role")
    return result["server"], result["client"]

def execution_argv(endpoint: dict[str, Any]) -> tuple[list[str], bytes | None]:
    if endpoint["execution"] == "local": return endpoint["argv"], None
    request={"protocol":REMOTE_EXEC_PROTOCOL,"role":endpoint["role"],"binary":endpoint["binary"],"argv":endpoint["argv"]}
    return endpoint["transport_argv"], json.dumps(request,separators=(",",":"),sort_keys=True).encode()

def checkout_head() -> str:
    try:
        root = subprocess.run(
            ["git", "-C", str(HERE), "rev-parse", "--show-toplevel"],
            capture_output=True, text=True, timeout=5, check=True,
        ).stdout.strip()
        if not root:
            raise CollectionError("adapter checkout is unavailable")
        head = subprocess.run(
            ["git", "-C", root, "rev-parse", "HEAD"],
            capture_output=True, text=True, timeout=5, check=True,
        ).stdout.strip()
    except (OSError, subprocess.SubprocessError) as exc:
        raise CollectionError("adapter checkout is unavailable") from exc
    if not HEX40.fullmatch(head):
        raise CollectionError("adapter checkout HEAD is invalid")
    return head

def requires(argv: list[str], token: str, value: str | None = None) -> None:
    if token not in argv:
        raise CollectionError(f"command missing {token}")
    if value is not None:
        index = argv.index(token)
        if index + 1 >= len(argv) or argv[index + 1] != value:
            raise CollectionError(f"command requires {token} {value}")

def event_objects(text: str) -> list[dict[str, Any]]:
    values = []
    for line_number, line in enumerate(text.splitlines(), 1):
        candidate = line.lstrip()
        json_looking = (candidate.startswith(("{", "[", '"', "-")) or
                        (candidate[:1].isdigit()) or
                        candidate.startswith(("true", "false", "null")))
        if not json_looking:
            continue
        try:
            value = json.loads(candidate)
        except json.JSONDecodeError as exc:
            raise CollectionError(f"malformed JSON event on line {line_number}") from exc
        if not isinstance(value, dict) or not isinstance(value.get("event"), str):
            raise CollectionError(f"invalid JSON event on line {line_number}")
        values.append(value)
    return values

def carrier(text: str, name: str) -> list[str]:
    return [line for line in text.splitlines() if line.startswith("carrier_event ") and f"name={name} " in line]

def singleton_carrier(text: str, name: str) -> str | None:
    rows = carrier(text, name)
    if len(rows) > 1:
        raise CollectionError(f"duplicate carrier event: {name}")
    return rows[0] if rows else None

def version(text: str, name: str) -> int | None:
    row = singleton_carrier(text, name)
    match = re.search(r"(?:^| )version=([0-9]+)(?: |$)", row) if row else None
    return int(match.group(1)) if match else None

def one_event(events: list[dict[str, Any]], name: str, required: bool = False) -> dict[str, Any] | None:
    matches = [item for item in events if item.get("event") == name]
    if len(matches) > 1:
        raise CollectionError(f"duplicate JSON event: {name}")
    if required and not matches:
        raise CollectionError(f"missing JSON event: {name}")
    return matches[0] if matches else None

def exact_nonnegative_int(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0

def validate_event_stream(events: list[dict[str, Any]], role: str, identity: str) -> None:
    for item in events:
        if item.get("role") != role or item.get("experiment_id") != identity:
            raise CollectionError(f"{role} event identity mismatch")

def event_position(text: str, name: str) -> int:
    for index, line in enumerate(text.splitlines()):
        candidate = line.lstrip()
        if candidate.startswith("{") and json.loads(candidate).get("event") == name:
            return index
    raise CollectionError(f"missing JSON event: {name}")

def carrier_position(text: str, name: str) -> int:
    for index, line in enumerate(text.splitlines()):
        if line.startswith("carrier_event ") and f"name={name} " in line:
            return index
    raise CollectionError(f"missing carrier event: {name}")

def strictly_ordered(*positions: int) -> bool:
    return all(left < right for left, right in zip(positions, positions[1:]))

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
        if checkout_head() != commit:
            raise CollectionError("git commit differs from adapter checkout HEAD")
        binary = pathlib.Path(os.environ.get("NEKO_FAILOVER_BINARY", ""))
        if not binary.is_file():
            raise CollectionError("binary is not a file")
        binary_bytes = binary.stat().st_size
        if binary_bytes <= 0:
            raise CollectionError("binary is empty")
        binary_sha = hashlib.sha256(binary.read_bytes()).hexdigest()
        server_endpoint, client_endpoint = endpoints(binary, binary_sha, binary_bytes, commit)
        server_argv, client_argv = server_endpoint["argv"], client_endpoint["argv"]
        cleanup_argv = command("NEKO_FAILOVER_CLEANUP_COMMAND_JSON")
        remote_server = server_endpoint["execution"] == "ssh"
        requires(server_argv, "failover-server")
        requires(server_argv, "--diagnostic")
        requires(server_argv, "--cease-udp-replies-after", "1")
        requires(client_argv, "failover-client")
        requires(client_argv, "--diagnostic")
        requires(client_argv, "--automatic-health-failover")
        udp_port = int(os.environ.get("NEKO_FAILOVER_UDP_PORT", "40081"))
        tcp_port = int(os.environ.get("NEKO_FAILOVER_TCP_PORT", "40080"))
        if not all(40080 <= value <= 40100 for value in (udp_port, tcp_port)) or udp_port == tcp_port:
            raise CollectionError("ports outside CLI bounded range")
        parameters = dict(PARAMETERS, udp_port=udp_port, tcp_port=tcp_port)
        tmp_path = pathlib.Path(tempfile.mkdtemp(prefix=f"neko-warm-cycle-{index}-"))
        os.chmod(tmp_path, 0o700)
        identity = f"git:{commit}"
        server_exec, server_input = execution_argv(server_endpoint)
        client_exec, client_input = execution_argv(client_endpoint)
        server_resource_path = tmp_path / "server-resource.json"
        if remote_server:
            server_cmd = server_exec
        else:
            server_cmd, server_resource_path = sampled_command(tmp_path, "server", identity, [udp_port, tcp_port], server_exec)
        client_cmd, client_resource_path = sampled_command(tmp_path, "client", identity, [], client_exec)
        server_log = open(tmp_path / "server.log", "xb")
        client_log = open(tmp_path / "client.log", "xb")
        try:
            server = subprocess.Popen(server_cmd, stdin=subprocess.PIPE if server_input is not None else subprocess.DEVNULL, stdout=server_log, stderr=subprocess.STDOUT, start_new_session=True, shell=False)
            if server_input is not None:
                assert server.stdin is not None; server.stdin.write(server_input); server.stdin.close()
            time.sleep(float(os.environ.get("NEKO_FAILOVER_SERVER_STARTUP_SECONDS", "0.2")))
            client = subprocess.Popen(client_cmd, stdin=subprocess.PIPE if client_input is not None else subprocess.DEVNULL, stdout=client_log, stderr=subprocess.STDOUT, start_new_session=True, shell=False)
            if client_input is not None:
                assert client.stdin is not None; client.stdin.write(client_input); client.stdin.close()
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
            allowed_cleanup = {"listeners_remaining", "processes_remaining"}
            if (not set(cleanup_value) <= allowed_cleanup or isinstance(listeners, bool) or
                    not isinstance(listeners, int) or listeners < 0):
                raise ValueError
            processes = cleanup_value.get("processes_remaining")
            if remote_server and (isinstance(processes, bool) or not isinstance(processes, int) or processes < 0):
                raise ValueError
        except (json.JSONDecodeError, KeyError, TypeError, ValueError):
            raise CollectionError("cleanup command did not return bounded evidence")

        server_text = (tmp_path / "server.log").read_text(errors="replace")
        client_text = (tmp_path / "client.log").read_text(errors="replace")
        se, ce = event_objects(server_text), event_objects(client_text)
        sr, cr = load_json(server_resource_path), load_json(client_resource_path)
        server_exit = exit_code(sr, server.returncode if server and server.returncode is not None else 124)
        client_exit = exit_code(cr, client.returncode if client and client.returncode is not None else 124)
        server_identity = f"warm-cycle-{index}-server"
        client_identity = f"warm-cycle-{index}-client"
        requires(server_argv, "--experiment-id", server_identity)
        requires(client_argv, "--experiment-id", client_identity)
        validate_event_stream(se, "server", server_identity)
        validate_event_stream(ce, "client", client_identity)
        client_start = one_event(ce, "start", required=True)
        server_start = one_event(se, "start", required=True)
        timing_event = one_event(ce, "failover_timing")
        accounting_event = one_event(ce, "failover_accounting")
        summary_client = one_event(ce, "summary")
        summary_server = one_event(se, "summary")
        expected_client = {"count": PARAMETERS["record_count"], "record_payload_bytes": PARAMETERS["record_payload_bytes"],
                           "application_bytes_total": PARAMETERS["record_count"] * PARAMETERS["record_payload_bytes"],
                           "udp_port": udp_port, "tcp_port": tcp_port, "max_seconds": PARAMETERS["client_max_seconds"]}
        expected_server = dict(expected_client, max_seconds=PARAMETERS["server_max_seconds"])
        if any(not exact_nonnegative_int(client_start.get(key)) or client_start.get(key) != value
               for key, value in expected_client.items()) or any(
                not exact_nonnegative_int(server_start.get(key)) or server_start.get(key) != value
                for key, value in expected_server.items()):
            raise CollectionError("start parameters mismatch")
        if any(client_start.get(key) != server_start.get(key) for key in
               ("count", "record_payload_bytes", "application_bytes_total", "udp_port", "tcp_port")):
            raise CollectionError("server/client start parameters mismatch")
        readiness_events = [item for item in ce if item.get("event") == "tcp_warm_readiness"]
        readiness = {item.get("seq") for item in readiness_events}
        event_cardinality = {
            "client UDP delivery acknowledgements": (count_events(ce, "udp_delivery_ack_validated"), 1),
            "client uncertain range": (count_events(ce, "udp_uncertain_range_sent"), 1),
            "client TCP readiness proofs": (len(readiness_events), 3),
            "client TCP delivery acknowledgements": (count_events(ce, "tcp_delivery_ack_validated"), 2),
            "server TCP readiness responses": (count_events(se, "tcp_readiness_response"), 3),
            "server TCP delivery acknowledgements": (count_events(se, "tcp_delivery_ack_sent"), 2),
        }
        if all(item is not None for item in (timing_event, accounting_event, summary_client)):
            for label, (actual, expected) in event_cardinality.items():
                if actual != expected:
                    raise CollectionError(f"invalid {label} cardinality")
        if len(readiness_events) != len(readiness):
            raise CollectionError("duplicate TCP readiness evidence")
        terminal_events = [name for name, item in (("failover_timing", timing_event), ("failover_accounting", accounting_event),
                                             ("summary", summary_client)) if item is not None]
        if terminal_events == ["failover_timing", "failover_accounting", "summary"] and not strictly_ordered(
                event_position(client_text, "start"), event_position(client_text, "udp_delivery_ack_validated"),
                event_position(client_text, "tcp_warm_readiness"), carrier_position(client_text, "tcp_warm"),
                event_position(client_text, "failover_timing"), event_position(client_text, "failover_accounting"),
                carrier_position(client_text, "ordered_records_complete"), event_position(client_text, "summary")):
            raise CollectionError("client evidence out of order")
        if summary_server is not None and not strictly_ordered(
                event_position(server_text, "start"), carrier_position(server_text, "udp_negotiated"),
                carrier_position(server_text, "udp_authenticated"), carrier_position(server_text, "tcp_negotiated"),
                carrier_position(server_text, "tcp_authenticated"), carrier_position(server_text, "tcp_resume_validated"),
                carrier_position(server_text, "tcp_resumed"), event_position(server_text, "summary")):
            raise CollectionError("server evidence out of order")
        udp_acks = count_events(ce, "udp_delivery_ack_validated")
        ordered = singleton_carrier(client_text, "ordered_records_complete") is not None
        server_resumed = singleton_carrier(server_text, "tcp_resumed") is not None
        complete = (ordered and server_resumed and summary_client is not None and summary_server is not None and
                    summary_client.get("records") == summary_server.get("records") == 3 and
                    summary_client.get("application_bytes_total") == summary_server.get("application_bytes_total") == 48)
        selected = version(server_text, "udp_negotiated")
        tcp_selected = version(server_text, "tcp_negotiated")
        semantic = {
            "selected_version": selected if selected is not None else 0,
            "udp_negotiated": selected is not None,
            "udp_authenticated": singleton_carrier(server_text, "udp_authenticated") is not None and singleton_carrier(client_text, "udp_authenticated") is not None,
            "tcp_negotiated": tcp_selected is not None and tcp_selected == selected,
            "tcp_authenticated": singleton_carrier(server_text, "tcp_authenticated") is not None,
            "resume_validated": singleton_carrier(server_text, "tcp_resume_validated") is not None,
            "readiness_proofs": len(readiness),
            "readiness_completed": readiness == {1, 2, 3} and (warm := singleton_carrier(client_text, "tcp_warm")) is not None and "application_data=0" in warm,
            "udp_confirmed_before_failure": udp_acks >= 1,
        }
        # The CLI emits the first uncertain send as a representative runtime event;
        # the real start/count and validated acknowledgements bound the whole range.
        accounting = {key: accounting_event.get(key, 0) if accounting_event else 0 for key in (
            "udp_confirmed_records", "udp_confirmed_bytes", "uncertain_records", "uncertain_bytes",
            "replayed_records", "replayed_bytes", "confirmed_records", "confirmed_bytes",
            "duplicate_records", "duplicate_bytes", "lost_records", "lost_bytes",
            "conflicting_records", "conflicting_bytes")}
        timing_fields = ("failure_decided_at_us", "first_resumed_data_accepted_us",
                         "first_resumed_ack_at_us", "recovery_latency_us")
        raw_timing = {key: timing_event.get(key) if timing_event else None for key in timing_fields}
        timing_valid = (timing_event is not None and all(exact_nonnegative_int(value) for value in raw_timing.values()) and
                        raw_timing["failure_decided_at_us"] <= raw_timing["first_resumed_data_accepted_us"] <= raw_timing["first_resumed_ack_at_us"] and
                        raw_timing["recovery_latency_us"] == raw_timing["first_resumed_data_accepted_us"] - raw_timing["failure_decided_at_us"])
        if timing_event is not None and not timing_valid:
            raise CollectionError("invalid failover timing")
        timing = {"failure_decided_at_us": raw_timing["failure_decided_at_us"],
                  "first_resumed_data_at_us": raw_timing["first_resumed_data_accepted_us"],
                  "first_resumed_ack_at_us": raw_timing["first_resumed_ack_at_us"],
                  "recovery_latency_us": raw_timing["recovery_latency_us"]}
        payload = parameters["record_payload_bytes"]
        count = parameters["record_count"]
        accounting_valid = (accounting_event is not None and
                            all(exact_nonnegative_int(value) for value in accounting.values()) and
                            accounting["udp_confirmed_records"] == 1 and accounting["udp_confirmed_bytes"] == payload and
                            accounting["uncertain_records"] == accounting["replayed_records"] == count - 1 and
                            accounting["uncertain_bytes"] == accounting["replayed_bytes"] == (count - 1) * payload and
                            accounting["confirmed_records"] == accounting["udp_confirmed_records"] + accounting["replayed_records"] == count and
                            accounting["confirmed_bytes"] == accounting["udp_confirmed_bytes"] + accounting["replayed_bytes"] == count * payload and
                            all(accounting[bytes_key] == accounting[records_key] * payload for records_key, bytes_key in (
                                ("udp_confirmed_records", "udp_confirmed_bytes"), ("uncertain_records", "uncertain_bytes"),
                                ("replayed_records", "replayed_bytes"), ("confirmed_records", "confirmed_bytes"),
                                ("duplicate_records", "duplicate_bytes"), ("lost_records", "lost_bytes"),
                                ("conflicting_records", "conflicting_bytes"))) and
                            accounting["confirmed_records"] + accounting["lost_records"] == count and
                            all(accounting[key] == 0 for key in ("duplicate_records", "duplicate_bytes", "lost_records", "lost_bytes", "conflicting_records", "conflicting_bytes")))
        if accounting_event is not None and not accounting_valid:
            raise CollectionError("invalid failover accounting")
        client_reaped = bool(cr and cr.get("cleanup", {}).get("complete"))
        server_reaped = (cleanup.returncode == 0 and listeners == 0 and processes == 0) if remote_server else bool(sr and sr.get("cleanup", {}).get("complete"))
        cleanup_verified = cleanup.returncode == 0 and listeners == 0 and client_reaped and server_reaped
        evidence_pass = (complete and all(semantic[key] for key in ("udp_negotiated", "udp_authenticated", "tcp_negotiated", "tcp_authenticated", "resume_validated", "readiness_completed", "udp_confirmed_before_failure")) and
                         semantic["readiness_proofs"] == 3 and accounting_valid and timing_valid and
                         client_exit == server_exit == 0 and cleanup_verified)
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
            "endpoint_provenance": [{"role": endpoint["role"], "execution": endpoint["execution"], "underlying_binary_path": endpoint["binary"]["path"], "binary_sha256": endpoint["binary"]["sha256"], "binary_bytes": endpoint["binary"]["bytes"], "git_commit": endpoint["binary"]["git_commit"]} for endpoint in (server_endpoint, client_endpoint)],
            "parameters": parameters, "semantic": semantic, "accounting": accounting, "timing": timing,
            "classification": {"failure_seam": "controlled_application_reply_cessation", "natural_blackhole": False, "pto_blackhole": False},
            "result": {"status": "passed" if evidence_pass else "failed", "client_exit_code": client_exit, "server_exit_code": server_exit,
                       "failure_stage": failure_stage, "failure_reason": failure_reason},
            "cleanup": {"status": "verified" if cleanup_verified else "failed", "client_process_reaped": client_reaped,
                        "server_process_reaped": server_reaped, "listeners_remaining": listeners, "temporary_files_removed": True},
            "resources": {"client": cr, "server": {"status": "not_collected_remote"} if remote_server else sr},
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
