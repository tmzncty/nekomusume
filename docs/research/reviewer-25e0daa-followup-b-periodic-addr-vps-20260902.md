# Reviewer 25e0daa Follow-up B — corrected periodic `--addr` result

## Scope and identity

This evidence-only review ran from a fresh detached worktree at exact authoritative parent `25e0daa4e74b3239568067afa967412ec4c0ebc7`. After `git fetch`, the local `origin/main` and independent `git ls-remote origin refs/heads/main` both resolved to that exact commit. I read `AGENTS.md`, the standing authorization, current Follow-up B/Fallback/completion handoff, periodic runtime/tests, sampler contract/schema, Primary A, and immutable `b041211823b6b26e53553e4f0137ee2b6081c2e0`.

The release binary SHA-256 was independently checked on both self-owned endpoints as `8ede1564015561498559586a83c6aeea2a75171bd2ae45b93e547e1793082852`. The established direct self-owned path used fresh TCP port 40096, concurrency 1, count 60, 32 bytes/record, 5000 ms interval, duration 300 s, setup timeout 5000 ms, and ACK timeout 1000 ms. The corrected client invocation supplied the resolved endpoint using `--addr`; `--connect` was not used. Endpoint and ephemeral identity details are omitted.

## Result

**Positive, one bounded self-owned sample only; not sustained or production proof.** Setup was attempted and authenticated. The client wrapper started at `2026-09-02T03:18:27.158Z`; authentication was directly observed at `2026-09-02T03:18:27.985Z` (827 ms observation delta). The client then attempted and confirmed all 60 records: 1,920 actual application bytes, zero missing, zero duplicate, and zero conflict records. All 60 raw confirmation-latency samples are retained in `result.json` and the raw client log; the runtime summary reports median 272 ms and P95 510 ms. The server independently reported authenticated=true, received=60, confirmed=60, duplicates=0. Client and server exited 0.

The application accounting above is derived from authenticated per-record runtime lines, not the sampler's caller-configured `application_bytes` annotation.

## Directly observed resources

The existing sampler reports direct-child measurements only. Client: user/system CPU 0.116749/0.237595 s, max RSS 10,044 KiB, peak FD 4, peak caller-port socket count 0. Server: user/system CPU 0.008349/0.004691 s, max RSS 10,092 KiB, peak FD 5, peak caller-port socket count 1. The raw JSON preserves source strings and sampler scope. No unavailable metric was promoted to zero.

## Cleanup and boundaries

Direct post-exit observations found no exact experiment process and no listener on port 40096 on either endpoint. Exact temporary runtime directories, copied binary/sampler, and ephemeral identities were removed. No firewall, route, qdisc, NAT, tunnel, provider, service, production configuration, network policy, handoff, release/freeze flag, or persistent secret changed. No push or handoff was performed.

A mistaken management-endpoint preparation/run was recognized before authentication and application delivery, retained outside Git for diagnosis, and cleaned; it is not represented as the corrected direct-path Session sample. The earlier `--connect`/missing-`--addr` pre-setup typo row was not repeated or included.
