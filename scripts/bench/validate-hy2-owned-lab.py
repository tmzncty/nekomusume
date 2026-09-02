#!/usr/bin/env python3
import argparse, json, math, os, pathlib, re, tempfile

SENTINEL = "nekomusume.gnu-time.v1"
SHA256 = re.compile(r"^[0-9a-f]{64}$")


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
                payload_bytes, payload_hash):
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
    expected_identity = "sha256:" + ("66dbdb0608f25f3057b433afe975a9fc1af2ca8e512479e294988b3ef363d6c1" if implementation == "hy2" else resource.get("identity", "").removeprefix("sha256:"))
    resource_ok = (validate_resource(resource)
                   and resource.get("implementation") == implementation
                   and resource.get("role") == "client"
                   and resource.get("identity") == expected_identity
                   and resource.get("experiment_id") == f"{implementation}-owned-lab-{run}"
                   and resource.get("sampling", {}).get("scope") == "sampler-created process group"
                   and all(finite_nonnegative(v) for v in (cpu_user, cpu_system))
                   and finite_nonnegative(rss_kib, True)
                   and resource.get("exit", {}).get("code") == exit_code
                   and resource.get("exit", {}).get("timed_out") is (exit_code == 124))
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
    return {
        "name": f"{implementation}-{run}", "implementation": implementation,
        "run": run, "failures": failure,
        "elapsed_seconds": timing["elapsed_seconds"] if timing else None,
        "cpu_user_seconds": cpu_user if resource_ok else None,
        "cpu_system_seconds": cpu_system if resource_ok else None,
        "rss_kib": rss_kib if resource_ok else None,
        "fd_count": fd_count, "application_bytes": application_bytes,
        "payload_sha256": observed_hash, "wire_bytes": None,
        "exit_code": exit_code, "failure_stage": stage,
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
        if row["wire_bytes"] is not None and not finite_nonnegative(row["wire_bytes"], True):
            raise ValueError("invalid wire_bytes")
    if len(set(seen)) != len(seen):
        raise ValueError("duplicate sample")
    if seen != expected[:len(seen)]:
        raise ValueError("sample order is malformed")
    if require_complete and seen != expected:
        raise ValueError("sample set is incomplete")


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
    if contract.get("client_lifecycle") != "fresh transport per timed sample" or contract.get("client_resource_scope") != "sampler-created process group":
        raise ValueError("client lifecycle/resource contract is missing")
    expected_clients = {(implementation, f"{implementation}-owned-lab-{run}")
                        for implementation in ("nekomusume", "hy2")
                        for run in range(1, contract["runs_per_implementation"] + 1)}
    observed_clients = {(item.get("implementation"), item.get("experiment_id"))
                        for item in resources if item.get("role") == "client"
                        and item.get("sampling", {}).get("scope") == "sampler-created process group"}
    if observed_clients != expected_clients:
        raise ValueError("per-sample client transport resource evidence is incomplete")
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
    sample.add_argument("--payload-hash", required=True)
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
                            args.client_output, args.bytes, args.payload_hash)
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
