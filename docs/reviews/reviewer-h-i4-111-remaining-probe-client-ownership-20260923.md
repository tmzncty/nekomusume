# H-I4-111 — remaining probe client ownership is not harness-bounded

**Severity:** HIGH  
**Area:** release item 4 / CLI process-test harness correctness  
**Reviewed anchor:** `5ecd5c673d8e311a03c702a816e3b5c080bf728a` (`main` at review start)  
**Review type:** exact-current source/control-flow inspection only; no reviewer-local Rust/full-gate, hosted CI, WAN, fuzz, cross-platform or performance execution is claimed here.

## Finding

H-I4-110 is closed on its exact owner at developer-owned `259495fad9949a526b12187ba59cf74aec0ee7f7`: the canonical `failover --role client` in `executable_loopback_controlled_udp_stop_tcp_resume` now runs under `bounded_client_output`, and its reachable developer-local exact-tree provenance is recorded in `docs/notes/h-i4-110-provenance-259495f-20260923.md`.

The continued owner-by-owner sweep found two additional real networked client owner shapes in `crates/neko-cli/tests/probe.rs` that still block synchronously before the already-bounded server cleanup can become reachable.

### 1. `first_udp_selection_loss_recovers_from_same_peer_duplicate_hello`

Current shape:

1. spawn a real `failover-server` and pass `ready_failover_server`;
2. spawn a real `failover-client` with piped stdout/stderr and product `--duration 4`;
3. sleep 30 ms and inject an unrelated UDP datagram while that client is live;
4. call `client.wait_with_output().unwrap()` directly;
5. only after the client returns call `finish_server(server)`.

The intentional concurrent datagram injection is semantically valuable, but the final direct `wait_with_output()` has no harness-local deadline. If the client remains live in connect/negotiation/Noise/authenticated I/O/DeliveryAck/shutdown, the test can hang indefinitely and never reach `finish_server`.

### 2. `expired_preprogress_udp_session_is_retired_before_delivery_and_fresh_handshake_recovers`

Current shape:

1. spawn a real `failover-server` and pass `ready_failover_server`;
2. run an intentionally expired first real `failover-client` (`--duration 2`, delayed first data) through direct synchronous `.output()`;
3. run a fresh recovery real `failover-client` (`--duration 3`) through a second direct synchronous `.output()`;
4. only after both return call `finish_server(server)`.

Either client can therefore strand the test before server cleanup if the lifecycle under test regresses.

## Challenged invariant

A test that owns a real networked product process must not rely solely on the product's own `--duration` to bound the harness. Product duration/lifecycle is behavior under test. Before control reaches another owner such as `finish_server`, every real client child that can wait on networking/protocol progress must itself have an independent harness deadline plus truthful bounded termination/reap ownership.

This is a **test-harness / release-evidence correctness HIGH**, not evidence of a production Session/Carrier/ACK/crypto/wire defect.

## Closure contract

1. Reuse the exact-current `bounded_client_output` / `bounded_wait_with_output` ownership primitives. Do not create another process framework.
2. For `first_udp_selection_loss_recovers_from_same_peer_duplicate_hello`, preserve the concurrent 30 ms unrelated-datagram injection. The smallest shape is to keep the existing spawned child and call `bounded_wait_with_output(&mut client, ...)` after the injection instead of raw `wait_with_output()`. The harness deadline must exceed product `--duration`; the established `duration + 5s` pattern makes 9 s appropriate for this `--duration 4` owner without inventing a new repository-wide policy value.
3. For `expired_preprogress_udp_session_is_retired_before_delivery_and_fresh_handshake_recovers`, route both sequential real clients through `bounded_client_output`: `--duration 2` → 7 s, `--duration 3` → 8 s under the already-established duration-plus-slack convention.
4. Preserve every current product argument, intentional delay/injection, authentication/recovery assertion and server-side causality check. Do not change Session/Carrier/ACK/crypto/wire architecture.
5. Existing `bounded_client_output_fails_when_client_never_exits` already challenges the ordinary helper deadline path. Do not add duplicate generic sleep churn unless this exact concurrent-injection owner exposes a distinct failure mode.
6. Continue the owner-by-owner sweep after this repair. Remaining `.output()`, `wait`, `wait_with_output`, `try_wait`, socket `accept`/`recv`/`read_exact`, pipe drain and thread `join` sites must be causally classified as exit/peer-completion proven, source-self-bounded, externally bounded, or a concrete unbounded owner. Do not mechanically convert keygen, help/capabilities, invalid-argument/configuration and other clearly local fail-fast calls.
7. No decoder/parser/crypto-framing implementation change is implicated; do not run fuzz mechanically.
8. On the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist developer-local exact-tree provenance with reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust and clean-tree state.
9. Continue immediately to the next dependency-ready review/repair slice after closure; do not infer repository-wide queue exhaustion from this seam.

## Evidence boundary

- H-I4-110 remains closed/provenanced on its exact canonical client owner.
- H-I4-107/109 remain closed on their helper ownership claims absent exact-current falsification; H-I4-108 remains a continuing owner-inventory effort rather than a globally complete claim.
- `READY_LIVE: none` remains authoritative; this finding creates no new real-network question.
- Release items 3 and 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- D019/source-retention capacity and all other maintainer/security policy values remain outside this repair.
