# H-I4-108 — probe.rs network-client process ownership remains unbounded

**Severity:** HIGH — release item 4 / test-harness correctness

**Reviewer source anchor:** `75d65e653b3c245e6d6146a293a183f058952804`

## Finding

The H-I4-107 repair in `crates/neko-cli/tests/multistream.rs` correctly moved the real multistream client processes away from synchronous `Command::output()` into spawned children with a harness-local bounded wait. The same ownership invariant is still violated by multiple networked product-client call sites in `crates/neko-cli/tests/probe.rs`.

Concrete exact-current examples:

- `authenticated_tcp_and_udp_loopback_probe_starts_after_ready` obtains a `ReadyServer`, then runs the real `client` process with synchronous `.output()`, and only afterwards calls `finish_server(server)`.
- `authenticated_tcp_benchmark_echoes_exact_payload_and_hash` has the same shape for the real TCP benchmark client.
- failover/recovery process tests such as `reliable_udp_failover_settles_packet_acks_to_zero_in_flight` start a real `failover-server`, establish readiness, then run the real `failover-client` with synchronous `.output()` before server cleanup.
- additional exact-current failover / warm-cold / retry paths use the same server-ready -> client `.output()` -> `finish_server` pattern and must be classified owner-by-owner.

If a client lifecycle/protocol regression leaves one of those networked product processes blocked in connect, negotiation, Noise/authentication, DeliveryAck, failover, or framed I/O, the test thread never reaches `finish_server` / `bounded_reap_or_kill`. Product `--duration` is behavior under test and is not an independent harness termination proof. The result is an unbounded `scripts/check.sh` failure shape rather than bounded negative evidence.

This is not a finding that every synchronous `.output()` in `probe.rs` is wrong. Local fail-fast commands (`keygen`, `--help`, capabilities, validation-before-network cases, etc.) must remain separately classified. The concrete defect is the networked product child whose completion is part of the item-4 oracle while another owned peer/server remains live.

## Invariant challenged

Every networked child whose lifecycle/completion is part of an item-4 process-test oracle needs an independent harness-local completion bound. The bound must preserve process ownership so timeout can terminate/reap the actual child; moving `Command::output()` into an unkillable worker thread is not sufficient.

## Required closure

1. Add or reuse a narrow **test-local** spawned-child bounded output primitive in `probe.rs`: piped stdout/stderr must be drained without pipe-full deadlock, child exit must be observed with a local deadline, and timeout/error paths must distinguish exit proof, bounded cleanup success, and explicit cleanup failure.
2. Convert the concrete networked client owners that run while a `ReadyServer` / failover server is live. At minimum close the examples above, then mechanically classify the remaining `probe.rs` `.output()` call sites so another same-shape networked owner is not left behind.
3. Do **not** mechanically convert clearly local, validation-before-network, bounded-by-source commands just to remove `.output()` text. Record exclusions instead.
4. Add a deterministic negative regression where a spawned client-like child remains live beyond the helper deadline; prove the helper fails within a bounded interval and does not strand pipe-reader ownership.
5. Preserve all protocol/crypto/diagnostic/JSON/human-output assertions. This is harness ownership only; do not change Session/Carrier/ACK/crypto/wire semantics.
6. No decoder/parser/crypto-framing implementation change is implicated; do not run fuzz mechanically.
7. On the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist developer-local exact-tree provenance with reachable SHA, UTC start/end, exit codes, OS/arch, and stable Rust.
8. Continue immediately into the remaining process/socket/thread ownership sweep after closure. Do not infer repository-wide queue exhaustion from this repair.

## H-I4-107 relationship

Exact-current source at `75d65e6` shows the H-I4-107 multistream client-owner repair and the `bounded_wait_with_output` `try_wait`/kill/reap error-branch classifications. That source repair is accepted for review direction, but this reviewer pass did not find persisted final exact-tree provenance for H-I4-107 on the current head. Do not relabel it as a completed developer-local gate until such reachable provenance exists.

H-I4-108 is a distinct owner family in `probe.rs`; it does not reopen the multistream owner once H-I4-107 provenance is complete.

## Evidence boundary

Reviewer work here is exact-current GitHub source/control-flow review only. It is not reviewer-local Rust/full-gate execution, hosted-CI evidence, cross-platform execution, WAN evidence, or a performance conclusion.
