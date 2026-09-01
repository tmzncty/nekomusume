# Reviewer 3978f3f Follow-up B — exact-tree periodic TCP blocker

## Scope and provenance

This note preserves the negative result of the requested current-tree periodic TCP row. It contains no endpoint address, private topology, identity material, public key, payload, command line with secrets, or raw log.

- reviewed implementation: `df61091d379aa10ad001e24f04e2143e13c0cb08`
- reviewed tree: `0b40a1aadc2b530ee11d3344312dacbac28632ae`
- authoritative GitHub `main` before the fresh detached worktree: `3978f3fdd3fb34510468a2e1708c0b2c5c5f6aec`
- coordination identity: authoritative `main` was exactly the requested commit, and `git merge-base --is-ancestor` verified it descends from the reviewed implementation
- release binary SHA-256 on build host and both self-owned Linux endpoints: `3460fb74856f6eb53db47c4e04f6bb887c77157b85d8d12b6bbe1c1c2c980a52`
- binary: 1,126,912-byte Linux x86-64 ELF; rustc 1.98.0; Cargo 1.98.0
- intended profile: authenticated TCP Session, concurrency 1, 60 records, 32 application bytes per record, 5,000 ms interval, 1,000 ms finite ACK deadline, duration at most 300 s, at most 1,920 application bytes
- boundaries: temporary unprivileged port; no route, firewall, qdisc, tunnel, namespace, capture, service, production configuration, handoff, flag, or persistent-secret change

The execution read the repository agent contract and standing authorization, the Follow-up B/completion handoff sections, the periodic runner and process resource sampler/schema documentation, exact Primary A cleanup and blocker commit `0255551`, and historical periodic evidence at `fb601a8`. Primary A cleanup was complete. Its warm-readiness failure is scientifically distinct from this periodic TCP row and did not block execution.

## Preserved attempts and changed path

The unchanged public/no-ingress path was not used. An initial preparation mistake resolved the SSH alias address rather than selecting the established second-client-to-VPS data path. The client failed before authentication with `connect failed; reconnect/resume unsupported`; the server accepted no Session. That failed row was stopped, reaped, and cleaned. It was not repeated unchanged.

A bounded diagnostic then tested the VPS's self-owned private address candidates using one temporary TCP listener. Eight of eleven candidates were reachable. The actual periodic attempt selected a previously valid private WireGuard endpoint routed over the client's established direct tunnel; endpoint details are omitted. The exact port was observed listening before the client started.

## Actual periodic result

The sole attempt on the corrected established path ran from `2026-09-01T16:47:47Z` to `2026-09-01T16:48:12Z` and failed closed during authenticated Session setup:

1. TCP connected on the selected private path.
2. The server completed canonical negotiation and Noise authentication and emitted `periodic_server_authenticated session=7201 stream=1`.
3. The client did not complete the handshake and exited 2 with `handshake response failed`.
4. The server then exited 2 with `periodic Session disconnected; reconnect/resume unsupported`.
5. No periodic application record was attempted, confirmed, missing-after-attempt, or duplicated. No confirmation latency sample exists.

```text
client: authenticated=false attempted=0 confirmed=0 missing_after_attempt=0 duplicates=0
client: application_bytes_sent=0 p50_latency=unavailable p95_latency=unavailable
client: elapsed=2.060123 s exit=2 signal=null timed_out=false
server: authenticated=true received=0 confirmed=0 duplicates=0
server: elapsed=5.012928 s exit=2 signal=null timed_out=false
```

The sampler's configured `application_bytes=1920` is the intended workload annotation, not measured transmitted bytes. Because the client failed before the periodic loop, actual application bytes were zero.

Direct-child process measurements:

| role | CPU user / system | max RSS | peak FD | peak owned sockets | samples | cleanup |
|---|---:|---:|---:|---:|---:|---|
| client | 0.001385 / 0.002770 s | 10,100 KiB | 4 | 0 | 3 | child reaped; owned sockets after exit 0 |
| server | 0.002549 / 0.007647 s | 10,068 KiB | 5 | 1 | 6 | child reaped; owned sockets after exit 0 |

## Cleanup and claim boundary

Both endpoint checks found zero experiment listeners and zero `neko-cli` processes after exit. Temporary binaries, sampler copies, raw logs, raw sampler JSON, endpoint candidates, and disposable identities were removed from both hosts. Failure evidence is retained here without an unchanged retry.

Classification: exact-reviewed-tree, established-private-path, asymmetric post-negotiation/authentication blocker. This is not five-minute periodic stability evidence and proves no reconnect behavior, production-long-lived behavior, public reachability, capacity, performance, release readiness, or security property.
