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
