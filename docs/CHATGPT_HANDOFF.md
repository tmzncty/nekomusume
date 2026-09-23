# ChatGPT reviewer handoff — H-I4-111 remaining probe client ownership HIGH FRONT

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- H-I4-107/108/109 repair cluster is accepted/provenanced through developer-owned `4c445ce36aa06edc1b3d4952036073865e3f20d0`; reachable local provenance is `docs/notes/h-i4-107-108-109-provenance-4c445ce-20260923.md`. This is developer-reported local exact-tree evidence, not reviewer-local execution or hosted CI.
- **H-I4-110 is CLOSED** at developer-owned `259495fad9949a526b12187ba59cf74aec0ee7f7`: the canonical `failover --role client` in `executable_loopback_controlled_udp_stop_tcp_resume` now runs under `bounded_client_output` with `--duration 3 + 5s = 8s`. Reachable developer-local exact-tree provenance is `docs/notes/h-i4-110-provenance-259495f-20260923.md`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, UTC 2026-09-23T03:49:56Z → 03:55:40Z, Linux x86_64, stable rustc 1.98.0. The reviewed GitHub combined-status view for exact `259495f` exposed no hosted status entries; do not convert that absence into either a hosted-CI pass or a local-gate claim.
- **H-I4-111 is CLOSED** at `1d727f065436fbe15ff9151e48a746da9fb2383b`: `first_udp_selection_loss`'s concurrent-injection client uses `bounded_wait_with_output(9s = --duration 4 + 5s)`; `expired_preprogress`'s sequential expired (`--duration 2 → 7s`) and recovery (`--duration 3 → 8s`) clients use `bounded_client_output`; `warm_readiness_failures`'s `failover-client` (`--duration 2 → 7s`) and `server.wait_with_output` are bounded too. Exact-tree provenance: `docs/notes/h-i4-111-provenance-1d727f0-20260923.md` — `check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-23T05:51:19Z → 05:57:00Z, Linux x86_64, rustc 1.98.0.
- H-I4-097..107 and H-I4-109/110 remain closed on their exact source claims absent exact-current falsification. H-I4-108 remains a continuing owner-inventory effort rather than a repository-wide statement that every process owner is already bounded. H-I4-090..095 remain closed on malformed-resource causality/cfg proof surfaces absent owner change/falsification.
- Candidate A (`Recovery::on_ack` future/unsent ACK) and Candidate B (`record_datagrams` mixed drop reasons) remain closed unless materially changed or falsified by exact-current source/tests.
- `SessionRuntime.events` retained-history capacity and D019 source-retention/no-reset remain maintainer/security policy gates. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely for freshness.

## FRONT HIGH — H-I4-111 remaining probe client ownership

### Concrete defects

`crates/neko-cli/tests/probe.rs::first_udp_selection_loss_recovers_from_same_peer_duplicate_hello` has this exact-current control flow:

1. start a real failover server and pass `ready_failover_server`;
2. spawn a real `failover-client` with piped stdout/stderr and product `--duration 4`;
3. after 30 ms inject an unrelated UDP datagram while the client is live;
4. call direct `client.wait_with_output().unwrap()`;
5. only after that returns call `finish_server(server)`.

The injection is part of the test semantics, but the final wait is not harness-bounded. A client lifecycle/protocol regression can therefore strand the test before already-bounded server cleanup is reachable.

`crates/neko-cli/tests/probe.rs::expired_preprogress_udp_session_is_retired_before_delivery_and_fresh_handshake_recovers` likewise starts a real server, then executes an intentionally expired real client (`--duration 2`, delayed first data) through direct synchronous `.output()`, then a fresh recovery real client (`--duration 3`) through a second direct `.output()`, and only then calls `finish_server`. Either client can strand the gate.

Product `--duration` is behavior under test, not an independent harness deadline. This is release/item-4 **test-harness correctness HIGH**, not production Session/Carrier/ACK/crypto/wire semantics.

### Closure contract

1. Reuse exact-current `bounded_client_output` / `bounded_wait_with_output`; do not create another process framework.
2. For `first_udp_selection_loss_recovers_from_same_peer_duplicate_hello`, preserve the intentional 30 ms unrelated-datagram injection while the spawned client is live. Keep the spawned child and replace only the raw final `wait_with_output()` with `bounded_wait_with_output(&mut client, ...)`. Under the already-established duration-plus-5s convention, `--duration 4` maps to a 9s harness bound.
3. For `expired_preprogress_udp_session_is_retired_before_delivery_and_fresh_handshake_recovers`, route both real clients through `bounded_client_output`: `--duration 2` → 7s and `--duration 3` → 8s. Preserve the delayed-first-data negative and every subsequent recovery/guard/order assertion.
4. Do not mechanically convert local fail-fast keygen, help/capabilities, invalid-argument/configuration or socket-free fixture `.output()` calls. Continue causal classification owner by owner.
5. Existing `bounded_client_output_fails_when_client_never_exits` already proves the ordinary shared deadline path. Do not add duplicate generic sleep churn unless one of these exact owners exposes a distinct failure mode.
6. No decoder/parser/crypto-framing implementation change is implicated; do not run fuzz mechanically.
7. On the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist developer-local exact-tree provenance with reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust and clean-tree state.
8. Continue immediately to the next ownership/review slice after closure; do not wait for reviewer cadence and do not infer repository-wide queue exhaustion from this repair.

## Rolling queue — keep continuous after H-I4-111

Do not collapse this to one ticket. Dependency-ready order:

1. **H-I4-111 repair + exact-tree provenance** — current FRONT HIGH.
2. **Remaining process/socket/thread ownership causal sweep.** Continue exact-current `crates/neko-cli/tests/probe.rs`, `crates/neko-cli/tests/multistream.rs`, and other process tests. Pay special attention to later direct/indirect `.output()`, `wait`, `wait_with_output`, `try_wait`, periodic/endpoint-rebind clients, blocking socket `accept`/`recv`/`read_exact`, pipe drains and peer/reader joins. Classify each as exit/peer-completion proven, source-self-bounded, externally bounded, or concrete unbounded owner; repair only the last class. Preserve explicit fail-fast/self-bounded exclusions.
3. **Cross-platform CLI/process factual reconciliation.** Reconcile I4-CLI-PROC-096 and H-I4-097..111 with exact-current helper/cfg/socket/process semantics. Keep Linux-local execution, macOS/BSD source/cfg reasoning and Windows claims separate.
4. **Algorithmic/resource boundedness reconciliation.** Reconcile accepted boundedness reviews with current channel/output bounds, stdout accumulation, cleanup deadlines, socket peer lifetime, reader lifetime and child/thread ownership. Do not run capacity-pressure benchmarks or invent capacity/security policy values.
5. **Release packet / item-4 factual reconciliation.** `docs/release-security-review-packet.md` predates H-I4-097..111 in its top coverage boundary. Qualify stale statements that process failure paths are globally bounded or review is exhausted; index reachable repair/review notes only after relevant exact-tree provenance exists. Local process/socket tests are not WAN/performance/security approval.
6. **Pre-auth malformed/rejection accounting exact-current reuse challenge.** Reuse prior independent review only where owner source/tests remain unchanged; otherwise narrowly re-challenge changed ownership. D019 remains a policy gate.
7. **CLI diagnostic / exit-code / JSON / human-output boundary exact-current reuse challenge.** Diagnostics remain evidence-only and never authentication/Delivery/Path/ACK evidence.
8. **Package/build/reproducibility spot re-challenge.** Verify recent process-test repairs do not stale manifests/scripts/provenance assumptions. Do not invent signing/SBOM/key-custody/publication policy.
9. **Reliable UDP / CarrierState / CarrierManager / FairScheduler / SessionRuntime targeted spot re-challenge.** Re-open materially changed owners; otherwise record exact-current reuse boundaries rather than rerunning equivalent sweeps.
10. **Observability + carrier adapters + dependency/build exact-current reuse challenge.** Same owner-diff rule; no checker/schema/framework/docs filler.
11. **Repository-wide 13-surface refill.** Re-apply every required core surface after the HIGH and dependent reconciliation close. Queue exhaustion is legal only if the broad inventory finds no concrete defect, no uncovered implemented core surface, no READY review-support and no READY live question.
12. **Conditional live.** Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved real-network question within standing authorization. Otherwise keep `READY_LIVE: none`.

If several coherent slices complete in 10–30 minutes with stable quality, deepen the queue and continue implementation/review → tests → commit → push → next slice without waiting. Every 3–4 coherent slices or important repair cluster, do one factual item-4/release reconciliation rather than rewriting large docs after every small commit.

## Required 13-surface refill inventory

At each meaningful repository-wide refill, explicitly ask whether each surface has a reachable, dedicated, independent bounded review that remains valid on current owners:

1. `neko-reliable` UDP recovery — ACK range, future/unsent ACK, loss/retransmit, RTT/PTO, persistent congestion, Reno, fault simulation;
2. `neko-carrier::CarrierState` — generation, validation, hysteresis, single-active, drain/fail/activate;
3. Concurrent Carrier Manager / health / migration-back;
4. FairScheduler / multi-stream / session+stream flow-control accounting;
5. Memory/UDP/TCP carrier adapter close/error/resource semantics;
6. `SessionRuntime` lifecycle/resource/window/DeliveryAck accounting;
7. `neko-observe` projection/event/counter/high-water correctness;
8. package/reproducibility/operator scripts;
9. Cargo manifests/lock/features/build/native hooks/unsafe inheritance;
10. cross-platform CLI/process-test semantics;
11. CLI exit-code / JSON / human-output contract;
12. algorithmic resource boundedness without capacity-pressure benchmarks or new policy values;
13. release-packet factual consistency and evidence boundary.

A bounded no-finding review is valid item-4 support when it identifies inspected owners, challenged invariant, deterministic checks/commands, exclusions and an exact reachable anchor. Do not create checker/schema/framework/docs churn merely to manufacture a slice.

## Evidence discipline

- Developer-reported local CI, repository-persisted provenance, reviewer source review, hosted CI, live WAN evidence and performance conclusions are distinct evidence classes.
- Accepted exact-tree provenance must anchor a GitHub-resolvable pushed SHA. Never publish a local-only/unreachable SHA as shared exact-tree evidence.
- H-I4-110 exact `259495f` has reachable developer-local provenance; the reviewed combined-status endpoint exposed no hosted status entries. This is not a hosted pass/fail claim.
- H-I4-111 is reviewer source/control-flow inspection only until a developer repair receives a clean exact-tree local gate and reachable provenance.
- Hosted Actions are cross-evidence, not a wait condition and not a replacement for the developer-local clean exact-tree gate.
- Fuzz only when wire decoder/parser/crypto framing changes materially.
- Standing VPS authorization permits bounded self-owned TCP/UDP lab work, but current authoritative classification remains `READY_LIVE: none`; no repeat live run without a new concrete question.
- Never decide D019, capacity/TTL/LRU/history/security values, signing/key-custody/SBOM/publication policy, core Session/Carrier/ACK/crypto/wire architecture, destructive/canonical migration, RC/freeze/release/production authority.
