# ChatGPT reviewer handoff — H-I4-110 canonical failover client ownership HIGH FRONT

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- Developer-owned `10d97e7e2c03ca19d9c575d0f0897c1703af1f8d` + `75d65e653b3c245e6d6146a293a183f058952804` bound the real multistream client owners and harden `multistream.rs::bounded_wait_with_output`; `3bc1de3dcf015aac0a59fe1375323de040f251a9` makes the analogous `probe.rs` cleanup classification truthful; `4c445ce36aa06edc1b3d4952036073865e3f20d0` fixes H-I4-108 caller deadlines so they exceed the product `--duration` plus bounded slack.
- The developer-local exact-tree provenance for the H-I4-107/108/109 repair cluster is reachable at `docs/notes/h-i4-107-108-109-provenance-4c445ce-20260923.md`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, UTC 2026-09-23T03:02:29Z → 03:08:20Z, Linux x86_64, stable rustc 1.98.0. This is developer-reported local provenance, not reviewer-local execution or hosted CI.
- **H-I4-107 and H-I4-109 remain CLOSED on their exact claims. H-I4-108's helper and many networked-client conversions are accepted/provenanced, but the owner inventory was not complete.** Reviewer finding `docs/reviews/reviewer-h-i4-110-canonical-failover-client-ownership-20260923.md` identifies a missed canonical `failover --role client` process owner in `executable_loopback_controlled_udp_stop_tcp_resume`.
- Exact reviewed source shape: server spawn → `ready_failover_server` → direct synchronous `Command(... "failover", "--role", "client", ... "--duration", "3").output()` → only after client return `finish_server(server)`. A stuck client therefore prevents the already-bounded server cleanup from becoming reachable. Product `--duration` is behavior under test, not an independent harness deadline.
- H-I4-097..107 and H-I4-109 remain closed on their original source claims absent exact-current falsification. H-I4-090..095 remain closed on malformed-resource causality/cfg proof surfaces absent owner change/falsification.
- Candidate A (`Recovery::on_ack` future/unsent ACK) and Candidate B (`record_datagrams` mixed drop reasons) remain closed unless materially changed or falsified by exact-current source/tests.
- `SessionRuntime.events` retained-history capacity and D019 source-retention/no-reset remain maintainer/security policy gates. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely for freshness.

## FRONT HIGH — H-I4-110 canonical failover client ownership

### Concrete defect

`crates/neko-cli/tests/probe.rs::executable_loopback_controlled_udp_stop_tcp_resume` still owns a real canonical failover client with synchronous `.output()`. If that client remains live during connect, negotiation, Noise/authentication, framed I/O, DeliveryAck/resume or shutdown, the test can block forever before it reaches `finish_server`.

This is release/item-4 **test-harness correctness HIGH**, not production Session/Carrier/ACK/crypto/wire semantics.

### Closure contract

1. Replace this canonical `failover --role client` `.output()` with the existing exact-current `bounded_client_output`; do not create another process framework.
2. Preserve its product arguments and every existing protocol/diagnostic/assertion boundary. The harness deadline must exceed product `--duration`; using the already-established `duration + 5s` pattern gives 8s for this `--duration 3` owner and does not create a new global policy value.
3. Continue the exact-current owner-by-owner sweep after this repair. Classify remaining direct/indirect `.output()`, `wait`, `wait_with_output`, `try_wait`, socket `accept`/`recv`/`read_exact`, stdout/stderr drain and peer/reader `join` sites as: exit/peer-completion proven, source-self-bounded, externally bounded, or concrete unbounded owner. Repair only the last class.
4. Do not mechanically convert local fail-fast keygen, help/capabilities, invalid-argument/invalid-configuration paths. `probe --matrix` and other commands with their own runtime timeout still need causal classification rather than blanket conversion.
5. The existing `bounded_client_output_fails_when_client_never_exits` regression already proves the shared ordinary deadline path. Do not add a duplicate generic sleep regression unless this exact owner exposes a distinct failure mode.
6. No decoder/parser/crypto-framing implementation change is implicated; do not run fuzz mechanically.
7. On the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist developer-local exact-tree provenance with reachable SHA, UTC start/end, exit codes, OS/arch, stable Rust and clean-tree state.
8. Continue immediately to the next ownership/review slice after closure; do not wait for reviewer cadence and do not infer repository-wide queue exhaustion from this repair.

## Rolling queue — keep continuous after H-I4-110

Do not collapse this to one ticket. Dependency-ready order:

1. **H-I4-110 repair + exact-tree provenance** — current FRONT HIGH.
2. **Remaining process/socket/thread ownership causal sweep.** Re-read exact-current owners in `crates/neko-cli/tests/probe.rs`, `crates/neko-cli/tests/multistream.rs`, and other process tests. Pay special attention to canonical `failover --role client`, direct `client`, `periodic-client`, endpoint-rebind clients, synchronous `.output()` calls, child waits, socket blocking operations, pipe drains and joins. Preserve precise fail-fast/self-bounded exclusions.
3. **Cross-platform CLI/process factual reconciliation.** Reconcile I4-CLI-PROC-096 and H-I4-097..110 with exact-current helper/cfg/socket/process semantics. Keep Linux-local execution, macOS/BSD source/cfg reasoning and Windows claims separate.
4. **Algorithmic/resource boundedness reconciliation.** Reconcile accepted boundedness reviews with current channel/output bounds, stdout accumulation, cleanup deadlines, socket peer lifetime, reader lifetime and child/thread ownership. No capacity-pressure benchmark and no invented policy/security values.
5. **Release packet / item-4 factual reconciliation.** `docs/release-security-review-packet.md` predates H-I4-097..110. Qualify stale statements that process failure paths are globally bounded or review is exhausted; index reachable repair/review notes only after the relevant exact-tree provenance exists. Do not promote local process/socket tests to WAN/performance/security approval.
6. **Pre-auth malformed/rejection accounting exact-current reuse challenge.** Reuse prior independent review only where owner source/tests remain unchanged; otherwise narrowly re-challenge changed ownership. D019 remains a policy gate.
7. **CLI diagnostic / exit-code / JSON / human-output boundary exact-current reuse challenge.** Diagnostics remain evidence-only and never authentication/Delivery/Path/ACK evidence.
8. **Package/build/reproducibility spot re-challenge.** Verify recent process-test repairs do not stale manifests/scripts/provenance assumptions. Do not invent signing/SBOM/key-custody policy.
9. **Reliable UDP / CarrierState / CarrierManager / FairScheduler / SessionRuntime targeted spot re-challenge.** Re-open materially changed owners; otherwise record exact-current reuse boundaries rather than rerunning equivalent sweeps.
10. **Observability + carrier adapters + dependency/build exact-current reuse challenge.** Same owner-diff rule; no checker/schema/docs filler.
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
- Exact `4c445ce` has developer-local provenance but no visible combined hosted status in the reviewed GitHub view. Do not turn absence of hosted checks into a local-gate or hosted-CI claim.
- Reviewer H-I4-110 is source/control-flow inspection only until the developer supplies a final exact-tree local gate for the repair.
- Hosted Actions are cross-evidence, not a wait condition and not a replacement for the developer-local clean exact-tree gate.
- Fuzz only when wire decoder/parser/crypto framing changes materially.
- Standing VPS authorization permits bounded self-owned TCP/UDP lab work, but current authoritative classification remains `READY_LIVE: none`; no repeat live run without a new concrete question.
- Never decide D019, capacity/TTL/LRU/history/security values, signing/key-custody/SBOM/publication policy, core Session/Carrier/ACK/crypto/wire architecture, destructive/canonical migration, RC/freeze/release/production authority.
