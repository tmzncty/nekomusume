# ChatGPT reviewer handoff — H-I4-108 probe network-client ownership HIGH FRONT

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- Developer-owned `10d97e7e2c03ca19d9c575d0f0897c1703af1f8d` converts the remaining real multistream client owners to spawned bounded waits and updates `multistream_client`; `75d65e653b3c245e6d6146a293a183f058952804` makes `bounded_wait_with_output` classify `kill` / post-kill `try_wait` failures explicitly instead of treating error as exit proof. Reviewer source inspection accepts this H-I4-107 direction.
- **H-I4-107 still needs final reachable developer-local exact-tree provenance.** The reviewer pass did not find a persisted `75d65e6` (or later source-equivalent) provenance note recording `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, UTC start/end, OS/arch and stable Rust. Do not relabel reviewer source inspection as developer-local execution or hosted CI.
- Reviewer commit `382c47d3a30ec428bb1e01861cebdd5f735b305d` opens **H-I4-108 HIGH**: exact-current `crates/neko-cli/tests/probe.rs` still contains multiple real networked product clients executed with synchronous unbounded `.output()` while a separately owned ready server remains live. Full finding: `docs/reviews/reviewer-h-i4-108-probe-network-client-ownership-20260923.md`.
- Concrete H-I4-108 examples include `authenticated_tcp_and_udp_loopback_probe_starts_after_ready`, `authenticated_tcp_benchmark_echoes_exact_payload_and_hash`, and failover/recovery tests such as `reliable_udp_failover_settles_packet_acks_to_zero_in_flight`; additional same-shape failover/warm/cold/retry call sites must be classified owner-by-owner.
- H-I4-097..107 remain closed on their original source claims except H-I4-107 provenance completion noted above and any exact-current counterexample identifying a distinct owner family. H-I4-090..095 remain closed on malformed-resource causality/cfg proof surfaces absent owner change/falsification.
- Candidate A (`Recovery::on_ack` future/unsent ACK) and Candidate B (`record_datagrams` mixed drop reasons) remain closed unless materially changed or falsified by exact-current source/tests.
- `SessionRuntime.events` retained-history capacity and D019 source-retention/no-reset remain maintainer/security policy gates. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely for freshness.

## FRONT HIGH — H-I4-108 probe.rs network-client process ownership

### Concrete defect

Several process tests in exact-current `crates/neko-cli/tests/probe.rs` have this shape:

```text
spawn server -> prove READY -> run real network client with Command::output()
             -> only after client returns call finish_server / cleanup
```

The synchronous client call has no independent harness-local deadline and no retained child handle. If a client lifecycle/protocol regression leaves it blocked in connect, negotiation, Noise/authentication, DeliveryAck, failover, or framed I/O, the test thread never reaches the already-bounded server cleanup. Product `--duration` is behavior under test and cannot be the only termination proof.

This is a release/item-4 **test-harness correctness HIGH**, not a production Session/Carrier/ACK/crypto/wire semantic finding.

Do **not** mechanically classify every `.output()` as defective. Local fail-fast commands (`keygen`, help/capabilities, argument validation before network, etc.) remain valid exclusions when their source path is independently self-bounded.

### Closure contract

1. First persist final H-I4-107 developer-local exact-tree provenance if it is not already present on the synchronized head. No additional H-I4-107 code churn is needed unless the synchronized source differs materially from `75d65e6` or a concrete counterexample is found.
2. Add/reuse a narrow **test-local** spawned-child bounded-output helper in `probe.rs`. It must retain the actual child handle, concurrently drain piped stdout/stderr so pipe capacity cannot deadlock the process, poll child completion against a local deadline, and bounded terminate/reap on deadline.
3. Failure branches must remain ownership-truthful: distinguish exit proven; bounded cleanup succeeded; cleanup itself failed / exit remains unproven. Do not treat `try_wait Err` as exit and do not ignore `kill()` errors.
4. Convert networked client owners that run while a `ReadyServer` / failover server is live. At minimum cover:
   - `authenticated_tcp_and_udp_loopback_probe_starts_after_ready`;
   - `authenticated_tcp_benchmark_echoes_exact_payload_and_hash`;
   - `reliable_udp_failover_settles_packet_acks_to_zero_in_flight`;
   - then classify the remaining failover/warm/cold/retry `.output()` sites and convert every same-shape networked owner.
5. Preserve all existing protocol, crypto, diagnostic, JSON/human-output, settlement, failover and resource assertions. This repair changes harness ownership only.
6. Add a deterministic negative regression where a spawned client-like child stays live past the helper deadline; prove bounded failure and no pipe-reader strand. Do not put `Command::output()` in an unkillable worker thread.
7. Record explicit exclusions for clearly local/self-bounded `.output()` sites rather than converting text mechanically.
8. No decoder/parser/crypto-framing implementation change is implicated; do not run fuzz mechanically.
9. On the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist developer-local exact-tree provenance with reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust.
10. Continue immediately into the next ownership/review slice after closure; do not wait for reviewer cadence and do not infer repository-wide queue exhaustion from this repair.

## Rolling queue — keep continuous after H-I4-108

Do not collapse this to one ticket. Dependency-ready order:

1. **H-I4-107 provenance completion + H-I4-108 repair/provenance** — current FRONT HIGH cluster.
2. **Remaining process/socket/thread ownership causal sweep.** Re-read exact-current direct/indirect `try_wait`, `wait`, `wait_with_output`, `.output()`, socket `accept`/`recv`/`read_exact`, stdout/stderr drain, peer/reader `join`, channel timeout and child/thread ownership sites in `crates/neko-cli/tests/probe.rs`, `crates/neko-cli/tests/multistream.rs`, and other process-test owners. Classify each as exit/peer-completion-proven, success-path source-self-bounded, or failure-path externally bounded. Repair only concrete unproven hang/ownership counterexamples.
3. **Cross-platform CLI/process factual reconciliation.** Reconcile I4-CLI-PROC-096 and H-I4-097..108 with exact-current helper/cfg/socket/process semantics. Keep Linux-local execution, macOS/BSD source/cfg reasoning and Windows claims separate.
4. **Algorithmic/resource boundedness reconciliation.** Reconcile accepted boundedness reviews with current channel/output bounds, stdout accumulation, cleanup deadlines, socket peer lifetime, reader lifetime and child/thread ownership. No capacity-pressure benchmark and no invented policy/security values.
5. **Release packet / item-4 factual reconciliation.** The packet still predates H-I4-097..108. Qualify stale statements that all process failure paths are bounded or that repository-wide review is exhausted; index reachable repair/review notes only after exact-tree provenance exists. Do not promote local process/socket tests to WAN/performance/security approval.
6. **Pre-auth malformed/rejection accounting exact-current reuse challenge.** Reuse prior independent review only where owner source/tests remain unchanged; otherwise narrowly re-challenge changed ownership. D019 remains a policy gate.
7. **CLI diagnostic / exit-code / JSON / human-output boundary exact-current reuse challenge.** Diagnostics remain evidence-only and never authentication/Delivery/Path/ACK evidence.
8. **Package/build/reproducibility spot re-challenge.** Verify recent process-test repairs do not stale manifests/scripts/provenance assumptions. Do not invent signing/SBOM/key-custody policy.
9. **Reliable UDP / CarrierState / CarrierManager / FairScheduler / SessionRuntime targeted spot re-challenge.** Re-open materially changed owners; otherwise record exact-current reuse boundaries rather than rerunning equivalent sweeps.
10. **Observability + carrier adapters + dependency/build exact-current reuse challenge.** Same owner-diff rule; no checker/schema/docs filler.
11. **Repository-wide 13-surface refill.** Re-apply every required core surface after the HIGH and dependent reconciliation close. Queue exhaustion is legal only if the broad inventory finds no concrete defect, no uncovered implemented core surface, no READY review-support and no READY live question.
12. **Conditional live.** Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved real-network question within standing authorization. Otherwise keep `READY_LIVE: none`.

If several coherent slices complete in 10–30 minutes with stable quality, deepen the queue and continue implementation/review -> tests -> commit -> push -> next slice without waiting. Every 3–4 coherent slices or important repair cluster, do one factual item-4/release reconciliation rather than rewriting large docs after every small commit.

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
- Hosted Actions are cross-evidence, not a wait condition and not a replacement for the developer-local clean exact-tree gate.
- Fuzz only when wire decoder/parser/crypto framing changes materially.
- Standing VPS authorization permits bounded self-owned TCP/UDP lab work, but current authoritative classification remains `READY_LIVE: none`; no repeat live run without a new concrete question.
- Never decide D019, capacity/TTL/LRU/history/security values, signing/key-custody/SBOM/publication policy, core Session/Carrier/ACK/crypto/wire architecture, destructive/canonical migration, RC/freeze/release/production authority.
