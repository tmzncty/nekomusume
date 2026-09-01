# Bounded benchmark result schema v1

`schema/benchmark-result.v1.json` is the common result envelope for deterministic,
netns, VPS, and later comparison experiments. It requires a unique experiment ID,
source commit, explicit mode/transport/scope, duration and application-byte
bounds, per-sample failure and latency fields, aggregate summary, and cleanup
status.

`wire_bytes` is nullable because it must not be fabricated when capture metadata
cannot reliably measure it. Median/P95 fields are nullable for pre-session
failures. A result with no established exchange is a failure/availability record,
not a throughput claim. Secrets, identities, payloads, and unbounded logs are
never part of this schema.


## Owned-lab paired harness retention

The paired harness retains each raw sample atomically in execution order before the next sample starts. GNU `time` data is accepted only from one `nekomusume.gnu-time.v1` JSON sentinel; missing, duplicate, non-finite, or negative timing/resource values fail closed. A complete result requires exactly the configured interleaved sample set, exact application-byte and payload-hash evidence, complete process-resource evidence, and verified zero-residual cleanup. Median/P95 are computed only from retained successful samples after that complete-set validation.

If setup, sampling, cleanup, resource collection, or final assembly fails, the harness emits `nekomusume.benchmark-blocked-harness.v1` with `status: BLOCKED_HARNESS`, the failure stage, all atomically retained partial records, and cleanup evidence. It contains no partial summary or superiority claim.
