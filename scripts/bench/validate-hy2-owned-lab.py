#!/usr/bin/env python3
import argparse, datetime, hashlib, json, math, os, pathlib, re, tempfile

SENTINEL = "nekomusume.gnu-time.v1"
SHA256 = re.compile(r"^[0-9a-f]{64}$")
DIAGNOSTIC_INPUT_LIMIT = 4096
DIAGNOSTIC_BUNDLE_LIMIT = 2048
DIAGNOSTIC_CATEGORIES = {
    "tls": re.compile(r"\b(tls|certificate|x509|pin(?:sha256)?|handshake)\b", re.I),
    "auth": re.compile(r"\b(auth(?:entication|orization)?|unauthorized|forbidden|password|credential)\b", re.I),
    "config": re.compile(r"\b(config(?:uration)?|yaml|unknown (?:field|option)|invalid (?:field|option|value))\b", re.I),
    "path": re.compile(r"\b(connection refused|no route|network is unreachable|timeout|timed out|no such file|path)\b", re.I),
    "readiness": re.compile(r"\b(not ready|readiness|listener|failed to listen|address already in use)\b", re.I),
}
LIFECYCLE_STAGES = ("server_bound", "client_started", "quic_udp", "tls_authenticated")
LIFECYCLE_PATTERNS = (
    ("tls_authenticated", re.compile(r"\b(authenticated|authentication succeeded|connected to server|login succeeded)\b", re.I)),
    ("quic_udp", re.compile(r"\b(quic|udp|initial packet|connection established|handshake response)\b", re.I)),
)
SECRET_PATTERNS = (
    re.compile(r"(?i)\b(password|credential|token|secret|authorization|pinsha256|private[_ -]?key)\s*[:=]\s*\S+"),
    re.compile(r"(?i)\b(?:https?|quic|udp|tcp)://\S+"),
    re.compile(r"(?i)\b(?:[a-z0-9-]+\.)+[a-z]{2,63}\b"),
    re.compile(r"(?<![0-9A-Fa-f])(?:\d{1,3}\.){3}\d{1,3}(?::\d+)?"),
    re.compile(r"(?<![0-9A-Fa-f:])(?:[0-9A-Fa-f]{0,4}:){2,}[0-9A-Fa-f]{0,4}(?::\d+)?"),
    re.compile(r"(?<!\w)/(?:[^\s/:]+/)+[^\s:]*"),
)


def _iso8601(value):
    try:
        parsed = datetime.datetime.fromisoformat(value.replace("Z", "+00:00"))
    except (AttributeError, ValueError):
        raise ValueError("diagnostic timestamp must be ISO-8601")
    if parsed.tzinfo is None:
        raise ValueError("diagnostic timestamp must include timezone")
    return parsed.astimezone(datetime.timezone.utc).isoformat().replace("+00:00", "Z")


def _sanitize_diagnostic(text):
    text = "".join(char if char in "\n\t" or ord(char) >= 32 else "�" for char in text)
    for pattern in SECRET_PATTERNS:
        text = pattern.sub("[REDACTED]", text)
    return text


def client_diagnostic(path, bundle_path, started_at, ended_at, baseline_stage):
    if baseline_stage not in LIFECYCLE_STAGES:
        raise ValueError("invalid diagnostic lifecycle stage")
    try:
        source = pathlib.Path(path).read_bytes()[:DIAGNOSTIC_INPUT_LIMIT + 1]
        input_truncated = len(source) > DIAGNOSTIC_INPUT_LIMIT
        raw = source[:DIAGNOSTIC_INPUT_LIMIT]
    except OSError:
        return None
    if not raw:
        return None
    text = raw.decode("utf-8", "replace")
    category = next((name for name, pattern in DIAGNOSTIC_CATEGORIES.items() if pattern.search(text)), "unknown")
    baseline = LIFECYCLE_STAGES.index(baseline_stage)
    observed = [LIFECYCLE_STAGES.index(stage) for stage, pattern in LIFECYCLE_PATTERNS if pattern.search(text)]
    last_stage = LIFECYCLE_STAGES[max([baseline] + observed)]
    sanitized_full = _sanitize_diagnostic(text).encode("utf-8")
    sanitized = sanitized_full[:DIAGNOSTIC_BUNDLE_LIMIT]
    started = _iso8601(started_at)
    ended = _iso8601(ended_at)
    if datetime.datetime.fromisoformat(ended.replace("Z", "+00:00")) < datetime.datetime.fromisoformat(started.replace("Z", "+00:00")):
        raise ValueError("diagnostic timestamps are reversed")
    bundle = {
        "schema": "nekomusume.hy2-private-diagnostic.v1",
        "started_at": started,
        "ended_at": ended,
        "category": category,
        "last_success_stage": last_stage,
        "truncated": input_truncated or len(sanitized) < len(sanitized_full),
        "sanitized_text": sanitized.decode("utf-8", "ignore"),
    }
    encoded = (json.dumps(bundle, sort_keys=True, separators=(",", ":")) + "\n").encode()
    if len(encoded) > DIAGNOSTIC_BUNDLE_LIMIT + 512:
        raise ValueError("diagnostic bundle exceeds bound")
    target = pathlib.Path(bundle_path)
    target.parent.mkdir(parents=True, exist_ok=True)
    fd, temporary = tempfile.mkstemp(prefix=target.name + ".", dir=target.parent)
    try:
        os.fchmod(fd, 0o600)
        with os.fdopen(fd, "wb") as handle:
            handle.write(encoded)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, target)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)
    return {
        "category": category,
        "last_success_stage": last_stage,
        "bundle_sha256": hashlib.sha256(encoded).hexdigest(),
        "bundle_bytes": len(encoded),
        "started_at": started,
        "ended_at": ended,
    }


def finite_nonnegative(value, integer=False):
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        return False
    if not math.isfinite(value) or value < 0:
        return False
    return not integer or isinstance(value, int)


def parse_time(path):
    matches = []
    for line in pathlib.Path(path).read_text(errors="replace").splitlines():
        try:
            value = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(value, dict) and value.get("sentinel") == SENTINEL:
            matches.append(value)
    if len(matches) != 1:
        raise ValueError("GNU time output must contain exactly one sentinel record")
    value = matches[0]
    if set(value) != {"sentinel", "elapsed_seconds", "cpu_user_seconds", "cpu_system_seconds", "rss_kib", "exit_code"}:
        raise ValueError("GNU time sentinel has an unexpected shape")
    for key in ("elapsed_seconds", "cpu_user_seconds", "cpu_system_seconds"):
        if not finite_nonnegative(value[key]):
            raise ValueError(f"invalid {key}")
    for key in ("rss_kib", "exit_code"):
        if not finite_nonnegative(value[key], integer=True):
            raise ValueError(f"invalid {key}")
    if value["exit_code"] > 255:
        raise ValueError("invalid exit_code")
    return value


def load_json(path):
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


def load_jsonl(path):
    rows = []
    if not os.path.exists(path):
        return rows
    with open(path, encoding="utf-8") as handle:
        for number, line in enumerate(handle, 1):
            if not line.endswith("\n"):
                raise ValueError(f"record {number} is not newline terminated")
            try:
                row = json.loads(line)
            except json.JSONDecodeError as error:
                raise ValueError(f"malformed record {number}: {error}") from error
            if not isinstance(row, dict):
                raise ValueError(f"record {number} is not an object")
            rows.append(row)
    return rows


def atomic_write(path, value):
    target = pathlib.Path(path)
    target.parent.mkdir(parents=True, exist_ok=True)
    fd, temporary = tempfile.mkstemp(prefix=target.name + ".", dir=target.parent)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as handle:
            json.dump(value, handle, sort_keys=True, separators=(",", ":"))
            handle.write("\n")
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, target)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def _nullable_nonnegative(value, integer=False):
    return value is None or finite_nonnegative(value, integer)


def read_client_output(path):
    try:
        value = json.loads(pathlib.Path(path).read_text(errors="replace"))
    except (OSError, json.JSONDecodeError):
        return {}
    return value if isinstance(value, dict) else {}


def make_sample(implementation, run, return_code, time_path, resource_path, output_path,
                payload_bytes, payload_hash, expected_identity=None, diagnostic_path=None,
                diagnostic_bundle=None, diagnostic_started_at=None, diagnostic_ended_at=None, diagnostic_stage=None):
    if implementation not in ("nekomusume", "hy2") or not isinstance(run, int) or run < 1:
        raise ValueError("invalid sample identity")
    exit_code = return_code if isinstance(return_code, int) and 0 <= return_code <= 255 else None
    try:
        timing = parse_time(time_path)
    except (OSError, ValueError, json.JSONDecodeError):
        timing = None
    output = read_client_output(output_path)
    try:
        resource = load_json(resource_path)
    except (OSError, ValueError, json.JSONDecodeError):
        resource = {}
    cpu = resource.get("cpu", {}) if isinstance(resource, dict) else {}
    rss = resource.get("rss", {}) if isinstance(resource, dict) else {}
    fd_count = resource.get("fd", {}).get("peak_count") if isinstance(resource, dict) else None
    cpu_user, cpu_system, rss_kib = cpu.get("user_seconds"), cpu.get("system_seconds"), rss.get("max_kib")
    expected_identity = expected_identity or ("sha256:" + ("66dbdb0608f25f3057b433afe975a9fc1af2ca8e512479e294988b3ef363d6c1" if implementation == "hy2" else resource.get("identity", "").removeprefix("sha256:")))
    resource_ok = (validate_resource(resource)
                   and resource.get("implementation") == implementation
                   and resource.get("role") == "client"
                   and resource.get("identity") == expected_identity
                   and resource.get("experiment_id") == f"{implementation}-owned-lab-{run}"
                   and resource.get("sampling", {}).get("scope") == "sampler-created process group"
                   and all(finite_nonnegative(v) for v in (cpu_user, cpu_system))
                   and finite_nonnegative(rss_kib, True)
                   and resource.get("exit", {}).get("code") == exit_code
                   and resource.get("exit", {}).get("timed_out") is (exit_code == 124)
                   and timing is not None and timing.get("exit_code") == exit_code)
    if not finite_nonnegative(fd_count, True):
        fd_count = None
    application_bytes = output.get("application_bytes")
    if not finite_nonnegative(application_bytes, True):
        application_bytes = 0
    observed_hash = output.get("payload_sha256")
    if not isinstance(observed_hash, str) or not SHA256.fullmatch(observed_hash):
        observed_hash = None
    stage = None
    if exit_code != 0:
        stage = "client_exit"
    elif timing is None:
        stage = "time_parse"
    elif application_bytes != payload_bytes:
        stage = "application_bytes"
    elif observed_hash != payload_hash:
        stage = "payload_hash"
    elif fd_count is None or not resource_ok:
        stage = "resource_evidence"
    failure = int(stage is not None)
    diagnostic = None
    if exit_code != 0 and diagnostic_path and diagnostic_bundle and diagnostic_started_at and diagnostic_ended_at and diagnostic_stage:
        diagnostic = client_diagnostic(diagnostic_path, diagnostic_bundle, diagnostic_started_at, diagnostic_ended_at, diagnostic_stage)
    return {
        "name": f"{implementation}-{run}", "implementation": implementation,
        "run": run, "failures": failure,
        "elapsed_seconds": timing["elapsed_seconds"] if timing else None,
        "cpu_user_seconds": cpu_user if resource_ok else None,
        "cpu_system_seconds": cpu_system if resource_ok else None,
        "rss_kib": rss_kib if resource_ok else None,
        "fd_count": fd_count, "application_bytes": application_bytes,
        "payload_sha256": observed_hash, "wire_bytes": None,
        "exit_code": exit_code, "failure_stage": stage, "client_diagnostic": diagnostic,
    }


def validate_samples(samples, contract, require_complete):
    runs = contract.get("runs_per_implementation")
    payload_bytes = contract.get("payload_bytes")
    payload_hash = contract.get("payload_sha256")
    payload_prepared = contract.get("payload_prepared")
    if not isinstance(runs, int) or not 3 <= runs <= 10:
        raise ValueError("invalid runs_per_implementation")
    if not isinstance(payload_bytes, int) or payload_bytes <= 0:
        raise ValueError("invalid payload_bytes")
    if not isinstance(payload_prepared, bool): raise ValueError("invalid payload_prepared")
    if payload_prepared != (isinstance(payload_hash, str) and SHA256.fullmatch(payload_hash) is not None): raise ValueError("contradictory payload evidence")
    if require_complete and not payload_prepared: raise ValueError("complete result lacks payload")
    expected = [(implementation, run) for run in range(1, runs + 1) for implementation in ("nekomusume", "hy2")]
    seen = []
    for index, row in enumerate(samples):
        required = {"name", "implementation", "run", "failures", "elapsed_seconds", "cpu_user_seconds", "cpu_system_seconds", "rss_kib", "fd_count", "application_bytes", "payload_sha256", "wire_bytes", "exit_code", "failure_stage"}
        if not required.issubset(row):
            raise ValueError(f"sample {index + 1} missing fields")
        identity = (row["implementation"], row["run"])
        seen.append(identity)
        if row["name"] != f"{identity[0]}-{identity[1]}":
            raise ValueError("sample name mismatch")
        if row["failures"] not in (0, 1):
            raise ValueError("sample failure count must be zero or one")
        if row["exit_code"] is not None and (not isinstance(row["exit_code"], int) or not 0 <= row["exit_code"] <= 255):
            raise ValueError("invalid sample exit code")
        success = row["failures"] == 0
        numeric = ("elapsed_seconds", "cpu_user_seconds", "cpu_system_seconds", "rss_kib", "fd_count")
        if any(not _nullable_nonnegative(row[key], key in ("rss_kib", "fd_count")) for key in numeric):
            raise ValueError("sample has invalid timing/resource evidence")
        if not finite_nonnegative(row["application_bytes"], True):
            raise ValueError("sample has invalid application byte evidence")
        if row["payload_sha256"] is not None and (not isinstance(row["payload_sha256"], str) or not SHA256.fullmatch(row["payload_sha256"])):
            raise ValueError("sample has invalid payload hash")
        if success and any(row[key] is None for key in numeric):
            raise ValueError("successful sample lacks valid timing/resource evidence")
        if success and (row["exit_code"] != 0 or row["application_bytes"] != payload_bytes or row["payload_sha256"] != payload_hash or row["failure_stage"] is not None):
            raise ValueError("successful sample violates exit/app-byte/hash evidence")
        if not success and (not isinstance(row["failure_stage"], str) or not row["failure_stage"]):
            raise ValueError("failed sample lacks failure stage")
        diagnostic = row.get("client_diagnostic")
        if diagnostic is not None:
            if (not isinstance(diagnostic, dict)
                    or set(diagnostic) != {"category", "last_success_stage", "bundle_sha256", "bundle_bytes", "started_at", "ended_at"}
                    or diagnostic.get("category") not in set(DIAGNOSTIC_CATEGORIES) | {"unknown"}
                    or diagnostic.get("last_success_stage") not in LIFECYCLE_STAGES
                    or not isinstance(diagnostic.get("bundle_sha256"), str) or not SHA256.fullmatch(diagnostic["bundle_sha256"])
                    or not isinstance(diagnostic.get("bundle_bytes"), int) or not 0 < diagnostic["bundle_bytes"] <= DIAGNOSTIC_BUNDLE_LIMIT + 512):
                raise ValueError("sample has unsafe client diagnostic")
            started = _iso8601(diagnostic.get("started_at"))
            ended = _iso8601(diagnostic.get("ended_at"))
            reversed_time = datetime.datetime.fromisoformat(ended.replace("Z", "+00:00")) < datetime.datetime.fromisoformat(started.replace("Z", "+00:00"))
            if started != diagnostic["started_at"] or ended != diagnostic["ended_at"] or reversed_time:
                raise ValueError("sample has unsafe client diagnostic timestamps")
            if success:
                raise ValueError("successful sample has client diagnostic")
        if row["wire_bytes"] is not None and not finite_nonnegative(row["wire_bytes"], True):
            raise ValueError("invalid wire_bytes")
    if len(set(seen)) != len(seen):
        raise ValueError("duplicate sample")
    if seen != expected[:len(seen)]:
        raise ValueError("sample order is malformed")
    if require_complete and seen != expected:
        raise ValueError("sample set is incomplete")
    if require_complete and any(row["failures"] != 0 for row in samples):
        raise ValueError("complete result contains a failed sample")


def percentile(values, fraction):
    values = sorted(values)
    return None if not values else values[math.ceil((len(values) - 1) * fraction)] * 1000


def expected_summary(samples):
    result = []
    for implementation in ("hy2", "nekomusume"):
        rows = [row for row in samples if row["implementation"] == implementation]
        success = [row for row in rows if row["failures"] == 0]
        values = [row["elapsed_seconds"] for row in success]
        result.append({"implementation": implementation, "failures": sum(row["failures"] for row in rows), "median_exchange_latency_ms": percentile(values, .5), "p95_exchange_latency_ms": percentile(values, .95), "application_bytes": sum(row["application_bytes"] for row in success), "wire_bytes": None})
    return result


def validate_resource(resource):
    if not isinstance(resource, dict):
        return False
    cleanup = resource.get("cleanup")
    cpu = resource.get("cpu", {}); rss = resource.get("rss", {}); fd = resource.get("fd", {})
    return (isinstance(cleanup, dict) and cleanup.get("process_reaped") is True and cleanup.get("process_group_empty") is True and cleanup.get("owned_sockets_after_exit") == 0 and cleanup.get("complete") is True
            and all(finite_nonnegative(cpu.get(k)) for k in ("user_seconds", "system_seconds"))
            and finite_nonnegative(rss.get("max_kib"), True) and finite_nonnegative(fd.get("peak_count"), True))


def validate_result(path):
    doc = load_json(path)
    schema = doc.get("schema")
    if schema == "nekomusume.benchmark-blocked-harness.v1":
        required = {"schema", "experiment_id", "git_commit", "status", "failure_stage", "samples", "cleanup_status", "cleanup_evidence"}
        if not required.issubset(doc) or doc["status"] != "BLOCKED_HARNESS" or not doc["failure_stage"]:
            raise ValueError("malformed BLOCKED_HARNESS artifact")
        contract = doc.get("contract")
        if not isinstance(contract, dict):
            raise ValueError("blocked artifact lacks contract")
        validate_samples(doc["samples"], contract, False)
        if "summary" in doc:
            raise ValueError("blocked artifact must not contain a partial summary")
        cleanup = doc["cleanup_evidence"]
        cleanup_required = {"local_processes_reaped", "local_listeners_remaining", "remote_process_groups_reaped", "remote_listeners_remaining", "remote_temp_path_removed"}
        if not isinstance(cleanup, dict) or set(cleanup) != cleanup_required:
            raise ValueError("blocked artifact lacks cleanup evidence")
        for key in ("local_processes_reaped", "remote_process_groups_reaped", "remote_temp_path_removed"):
            if cleanup[key] is not None and not isinstance(cleanup[key], bool):
                raise ValueError("blocked artifact has invalid cleanup evidence")
        for key in ("local_listeners_remaining", "remote_listeners_remaining"):
            if cleanup[key] is not None and not finite_nonnegative(cleanup[key], True):
                raise ValueError("blocked artifact has invalid cleanup evidence")
        verified = cleanup == {"local_processes_reaped": True, "local_listeners_remaining": 0, "remote_process_groups_reaped": True, "remote_listeners_remaining": 0, "remote_temp_path_removed": True}
        if doc["cleanup_status"] != ("verified" if verified else "failed"):
            raise ValueError("blocked artifact cleanup status is untruthful")
        return
    if schema != "nekomusume.benchmark-result.v1":
        raise ValueError("unexpected result schema")
    required = {"schema", "experiment_id", "git_commit", "contract", "samples", "summary", "resources", "cleanup_status", "cleanup_evidence"}
    if not required.issubset(doc):
        raise ValueError("result missing required evidence")
    validate_samples(doc["samples"], doc["contract"], True)
    if doc["summary"] != expected_summary(doc["samples"]):
        raise ValueError("summary does not match complete retained sample set")
    resources = doc["resources"]
    if not isinstance(resources, list) or not resources or not all(validate_resource(item) for item in resources):
        raise ValueError("resource evidence is missing or incomplete")
    contract = doc["contract"]
    for key in ("nekomusume_binary_sha256", "hy2_binary_sha256"):
        value = contract.get(key)
        if not isinstance(value, str) or not SHA256.fullmatch(value):
            raise ValueError("contract binary identity is not an exact SHA-256")
    bounds = doc.get("bounds")
    if not isinstance(bounds, dict) or set(bounds) != {"maximum_duration_ms", "application_bytes_max"}:
        raise ValueError("result bounds are missing")
    maximum = bounds["maximum_duration_ms"]
    expected_maximum = contract.get("enforced_global_deadline_ms")
    if not finite_nonnegative(maximum, True) or maximum <= 0 or maximum > 600_000:
        raise ValueError("invalid result duration bound")
    if expected_maximum != maximum:
        raise ValueError("result bound does not match enforced global deadline")
    work = contract.get("work_deadline_ms"); reserve = contract.get("cleanup_reserve_ms"); whole = contract.get("whole_lab_deadline_ms")
    if not all(finite_nonnegative(v, True) for v in (work, reserve, whole)) or work + reserve != whole or whole != maximum or whole > 600_000:
        raise ValueError("deadline bounds are inconsistent")
    if not finite_nonnegative(bounds["application_bytes_max"], True) or bounds["application_bytes_max"] != contract["payload_bytes"] * contract["runs_per_implementation"] * 2:
        raise ValueError("invalid application bounds")
    if contract.get("client_lifecycle") != "fresh transport per timed sample" or contract.get("client_resource_scope") != "sampler-created process group":
        raise ValueError("client lifecycle/resource contract is missing")
    expected_clients = {(implementation, f"{implementation}-owned-lab-{run}")
                        for implementation in ("nekomusume", "hy2")
                        for run in range(1, contract["runs_per_implementation"] + 1)}
    client_resources = [item for item in resources if item.get("role") == "client"
                        and item.get("sampling", {}).get("scope") == "sampler-created process group"]
    observed_clients = {(item.get("implementation"), item.get("experiment_id"))
                        for item in client_resources}
    if observed_clients != expected_clients or len(client_resources) != len(expected_clients):
        raise ValueError("per-sample client transport resource evidence is incomplete")
    if any(item.get("exit") != {"code": 0, "timed_out": False} for item in client_resources):
        raise ValueError("complete result contains unsuccessful client resource evidence")
    pinned = {"nekomusume": "sha256:" + contract.get("nekomusume_binary_sha256", ""), "hy2": "sha256:" + contract.get("hy2_binary_sha256", "")}
    if any(item.get("implementation") in pinned and item.get("identity") != pinned[item["implementation"]] for item in resources if item.get("role") == "client"):
        raise ValueError("client resource identity is not pinned to contract")
    server_pinned = {"nekomusume": pinned["nekomusume"], "hy2-v2.9.3": pinned["hy2"]}
    if any(item.get("implementation") in server_pinned and item.get("identity") != server_pinned[item["implementation"]] for item in resources if item.get("role") == "server"):
        raise ValueError("server resource identity is not pinned to contract")
    if doc["cleanup_status"] != "verified" or doc["cleanup_evidence"] != {"local_processes_reaped": True, "local_listeners_remaining": 0, "remote_process_groups_reaped": True, "remote_listeners_remaining": 0, "remote_temp_path_removed": True}:
        raise ValueError("cleanup evidence is missing")


def main():
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="command", required=True)
    parse = sub.add_parser("parse-time"); parse.add_argument("path")
    validate = sub.add_parser("validate-result"); validate.add_argument("path")
    sample = sub.add_parser("make-sample")
    sample.add_argument("--implementation", required=True); sample.add_argument("--run", required=True, type=int)
    sample.add_argument("--return-code", required=True, type=int); sample.add_argument("--time", required=True); sample.add_argument("--resource", required=True)
    sample.add_argument("--client-output", required=True); sample.add_argument("--bytes", required=True, type=int)
    sample.add_argument("--payload-hash", required=True); sample.add_argument("--expected-identity"); sample.add_argument("--client-diagnostics")
    sample.add_argument("--diagnostic-bundle"); sample.add_argument("--diagnostic-started-at"); sample.add_argument("--diagnostic-ended-at"); sample.add_argument("--diagnostic-stage", choices=LIFECYCLE_STAGES)
    blocked = sub.add_parser("blocked")
    blocked.add_argument("--records", required=True); blocked.add_argument("--output", required=True)
    blocked.add_argument("--stage", required=True); blocked.add_argument("--commit", required=True)
    blocked.add_argument("--runs", required=True, type=int); blocked.add_argument("--bytes", required=True, type=int)
    blocked.add_argument("--payload-prepared", choices=("true", "false"), required=True); blocked.add_argument("--payload-hash")
    blocked.add_argument("--local-reaped", required=True); blocked.add_argument("--local-listeners", required=True)
    blocked.add_argument("--remote-reaped", required=True); blocked.add_argument("--remote-listeners", required=True)
    blocked.add_argument("--remote-path-removed", required=True)
    args = parser.parse_args()
    if args.command == "parse-time":
        print(json.dumps(parse_time(args.path), sort_keys=True, separators=(",", ":")))
    elif args.command == "make-sample":
        value = make_sample(args.implementation, args.run, args.return_code, args.time, args.resource,
                            args.client_output, args.bytes, args.payload_hash, args.expected_identity, args.client_diagnostics,
                            args.diagnostic_bundle, args.diagnostic_started_at, args.diagnostic_ended_at, args.diagnostic_stage)
        print(json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False))
    elif args.command == "blocked":
        truth=lambda x: None if x=="unknown" else x=="true"
        count=lambda x: None if x=="unknown" else int(x)
        cleanup = {"local_processes_reaped":truth(args.local_reaped),"local_listeners_remaining":count(args.local_listeners),"remote_process_groups_reaped":truth(args.remote_reaped),"remote_listeners_remaining":count(args.remote_listeners),"remote_temp_path_removed":truth(args.remote_path_removed)}
        ok = cleanup == {"local_processes_reaped":True,"local_listeners_remaining":0,"remote_process_groups_reaped":True,"remote_listeners_remaining":0,"remote_temp_path_removed":True}
        doc = {"schema":"nekomusume.benchmark-blocked-harness.v1","experiment_id":"hy2-owned-lab-paired","git_commit":args.commit,"status":"BLOCKED_HARNESS","failure_stage":args.stage,"contract":{"runs_per_implementation":args.runs,"payload_bytes":args.bytes,"payload_prepared":args.payload_prepared=="true","payload_sha256":args.payload_hash},"samples":load_jsonl(args.records),"cleanup_status":"verified" if ok else "failed","cleanup_evidence":cleanup}
        atomic_write(args.output, doc)
        validate_result(args.output)
    else:
        validate_result(args.path)
        print("validated")

if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, json.JSONDecodeError) as error:
        raise SystemExit(str(error))
