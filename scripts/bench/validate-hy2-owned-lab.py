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


def validate_samples(samples, contract, require_complete):
    runs = contract.get("runs_per_implementation")
    payload_bytes = contract.get("payload_bytes")
    payload_hash = contract.get("payload_sha256")
    if not isinstance(runs, int) or not 3 <= runs <= 10:
        raise ValueError("invalid runs_per_implementation")
    if not isinstance(payload_bytes, int) or payload_bytes <= 0:
        raise ValueError("invalid payload_bytes")
    if not isinstance(payload_hash, str) or not SHA256.fullmatch(payload_hash):
        raise ValueError("invalid payload_sha256")
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
        if not isinstance(row["exit_code"], int) or not 0 <= row["exit_code"] <= 255:
            raise ValueError("invalid sample exit code")
        success = row["failures"] == 0
        numeric = ("elapsed_seconds", "cpu_user_seconds", "cpu_system_seconds", "rss_kib", "fd_count")
        if success and any(not finite_nonnegative(row[key], key in ("rss_kib", "fd_count")) for key in numeric):
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
    return isinstance(cleanup, dict) and cleanup.get("process_reaped") is True and cleanup.get("owned_sockets_after_exit") == 0 and cleanup.get("complete") is True


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
        if doc["cleanup_status"] != "verified" or doc["cleanup_evidence"] != {"local_processes_reaped": True, "local_listeners_remaining": 0, "remote_process_groups_reaped": True, "remote_listeners_remaining": 0, "remote_temp_path_removed": True}:
            raise ValueError("blocked artifact lacks verified cleanup evidence")
        return
    if schema != "nekomusume.benchmark-result.v1":
        raise ValueError("unexpected result schema")
    required = {"schema", "experiment_id", "git_commit", "contract", "samples", "summary", "resources", "cleanup_status", "cleanup_evidence"}
    if not required.issubset(doc):
        raise ValueError("result missing required evidence")
    validate_samples(doc["samples"], doc["contract"], True)
    if doc["summary"] != expected_summary(doc["samples"]):
        raise ValueError("summary does not match complete retained sample set")
    if not isinstance(doc["resources"], list) or not doc["resources"] or not all(validate_resource(item) for item in doc["resources"]):
        raise ValueError("resource evidence is missing or incomplete")
    if doc["cleanup_status"] != "verified" or doc["cleanup_evidence"] != {"local_processes_reaped": True, "local_listeners_remaining": 0, "remote_process_groups_reaped": True, "remote_listeners_remaining": 0, "remote_temp_path_removed": True}:
        raise ValueError("cleanup evidence is missing")


def main():
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="command", required=True)
    parse = sub.add_parser("parse-time"); parse.add_argument("path")
    validate = sub.add_parser("validate-result"); validate.add_argument("path")
    blocked = sub.add_parser("blocked")
    blocked.add_argument("--records", required=True); blocked.add_argument("--output", required=True)
    blocked.add_argument("--stage", required=True); blocked.add_argument("--commit", required=True)
    blocked.add_argument("--runs", required=True, type=int); blocked.add_argument("--bytes", required=True, type=int)
    blocked.add_argument("--payload-hash", required=True); blocked.add_argument("--cleanup-ok", required=True)
    args = parser.parse_args()
    if args.command == "parse-time":
        print(json.dumps(parse_time(args.path), sort_keys=True, separators=(",", ":")))
    elif args.command == "blocked":
        ok = args.cleanup_ok == "1"
        doc = {"schema":"nekomusume.benchmark-blocked-harness.v1","experiment_id":"hy2-owned-lab-paired","git_commit":args.commit,"status":"BLOCKED_HARNESS","failure_stage":args.stage,"contract":{"runs_per_implementation":args.runs,"payload_bytes":args.bytes,"payload_sha256":args.payload_hash},"samples":load_jsonl(args.records),"cleanup_status":"verified" if ok else "failed","cleanup_evidence":{"local_processes_reaped":ok,"local_listeners_remaining":0 if ok else 1,"remote_process_groups_reaped":ok,"remote_listeners_remaining":0 if ok else 1,"remote_temp_path_removed":ok}}
        atomic_write(args.output, doc)
        if ok:
            validate_result(args.output)
    else:
        validate_result(args.path)
        print("validated")

if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, json.JSONDecodeError) as error:
        raise SystemExit(str(error))
