# Fair local HY2 comparison workload contract

This is an implementation-neutral harness, not a HY2 installation or network experiment. Both exact commands receive the same deterministic payload via `BENCH_PAYLOAD_FILE`, byte count, SHA-256, target and timeout. Each must perform one bounded local exchange and print one JSON object with integer `application_bytes` and `fd_count`, plus the exact `payload_sha256`; `wire_bytes` is nullable and may be non-null only with trusted capture metadata.

## Contract

`scripts/bench/compare-hy2.sh OUTPUT.json` requires exact `NEKO_BENCH_CMD` and `HY2_BENCH_CMD`, equal server/route/MTU/security/load metadata, and 3–100 runs (default 5). Raw samples record elapsed latency, median/P95-ready timing inputs, failures, CPU user/system time, maximum RSS, FD count, application bytes, and nullable wire bytes. Reports must calculate median and P95 from successful raw samples and retain failure counts; no throughput claim is valid for a failed or incomplete exchange.

## Fail-closed boundary

Execution refuses without explicit isolated-lab and command-evaluation consent, complete equality metadata, finite bounds, `jq`, GNU `time`, and a loopback-only target (`127.0.0.1` or `::1`). Non-zero exit, timeout, malformed output, wrong payload length/hash, or missing FD is a failure. Absolute/traversal output paths are rejected. The harness never installs HY2, downloads artifacts, opens WAN routes, or executes WAN targets. No run is comparative evidence unless both implementations pass the same contract.


## Exact-payload adapters

The generic authenticated TCP client now accepts `--payload-file FILE` only with
`--transport tcp --count 1 --json`. The regular-file size must exactly equal the
bounded `--bytes` value (maximum 1200 bytes); the file is read through a bounded
reader, echoed through the authenticated Noise Session, compared byte-for-byte,
and reported with `application_bytes`, `payload_sha256`, client FD count and
truthful null `wire_bytes`. Existing probe behavior is unchanged when the option
is absent. `scripts/bench/echo-payload.py` applies the same exact byte/hash
contract to a bounded TCP forwarding listener, including a 1 MiB hard cap and a
30-second timeout cap. It is suitable for the disposable HY2 `tcpForwarding`
listener described by the pinned seam note; it is not a proxy or daemon.

The existing `compare-hy2.sh` remains loopback-only and now rejects an adapter
that returns the right byte count with the wrong hash. It must not be repurposed
as a WAN orchestrator. A self-owned-VPS run still requires a separate disposable
orchestrator to own both remote process lifecycles and cleanup.
