# Fair local HY2 comparison workload contract

This is an implementation-neutral harness, not a HY2 installation or network experiment. Both exact commands receive the same deterministic payload via `BENCH_PAYLOAD_FILE`, byte count, SHA-256, target and timeout. Each must perform one bounded local exchange and print one JSON object with integer `application_bytes` and `fd_count`; `wire_bytes` is nullable and may be non-null only with trusted capture metadata.

## Contract

`scripts/bench/compare-hy2.sh OUTPUT.json` requires exact `NEKO_BENCH_CMD` and `HY2_BENCH_CMD`, equal server/route/MTU/security/load metadata, and 3–100 runs (default 5). Raw samples record elapsed latency, median/P95-ready timing inputs, failures, CPU user/system time, maximum RSS, FD count, application bytes, and nullable wire bytes. Reports must calculate median and P95 from successful raw samples and retain failure counts; no throughput claim is valid for a failed or incomplete exchange.

## Fail-closed boundary

Execution refuses without explicit isolated-lab and command-evaluation consent, complete equality metadata, finite bounds, `jq`, GNU `time`, and a loopback-only target (`127.0.0.1` or `::1`). Non-zero exit, timeout, malformed output, wrong payload length, or missing FD is a failure. Absolute/traversal output paths are rejected. The harness never installs HY2, downloads artifacts, opens WAN routes, or executes WAN targets. No run is comparative evidence unless both implementations pass the same contract.
