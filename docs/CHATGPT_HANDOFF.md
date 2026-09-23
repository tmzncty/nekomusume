# ChatGPT reviewer handoff — H-I4-112 H-I4-111 source/provenance mismatch HIGH FRONT

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- Developer-owned `1d727f065436fbe15ff9151e48a746da9fb2383b` is a **test/harness implementation** commit; `54c5aa5a1edf0a236360d21c7c42fb75767b24d9` is the subsequent **docs/handoff** closure commit. Reviewer exact-current inspection found that the H-I4-111 closure/provenance overstates what `1d727f0` actually changed.
- `first_udp_selection_loss_recovers_from_same_peer_duplicate_hello` is correctly bounded at `1d727f0`: its intentionally concurrent spawned `failover-client` now ends through `bounded_wait_with_output(9s)`. `warm_readiness_failures_close_before_admission_or_application_data` also has its real client/server process owners bounded at `1d727f0`.
- **H-I4-111 is CLOSED** at `fc23a7a88c917e116f1140333faa6d8f441ca557`: `expired_preprogress`'s sequential expired (`--duration 2 → 7s`) and recovery (`--duration 3 → 8s`) clients now use `bounded_client_output`. **H-I4-112 is CLOSED**: the `1d727f0` provenance's source-coverage claim about the `expired_preprogress` pair was false — corrected by `fc23a7a`. Erratum recorded in `docs/notes/h-i4-111-provenance-1d727f0-20260923.md`. Exact-tree provenance: `docs/notes/h-i4-112-provenance-fc23a7a-20260923.md` — `check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-23T07:50:14Z → 07:56:00Z, Linux x86_64, rustc 1.98.0.
- The developer-local gate record in the H-I4-111 provenance note (`check.sh`, `git diff --check`, clean tree, Linux x86_64, stable rustc 1.98.0) may remain a genuine execution record for exact `1d727f0`, but it does **not** prove the falsified source-coverage sentence. Exact source is authoritative. GitHub combined status for exact `1d727f0` exposed no hosted status entries. Reviewer did not execute Rust/full-gate locally in this pass.
- H-I4-107/108/109 remain accepted/provenanced through developer-owned `4c445ce36aa06edc1b3d4952036073865e3f20d0`. H-I4-110 remains CLOSED at developer-owned `259495fad9949a526b12187ba59cf74aec0ee7f7` with reachable developer-local exact-tree provenance.
- H-I4-097..107 and H-I4-109/110 remain closed on their exact source claims absent exact-current falsification. H-I4-108 remains a continuing owner-inventory effort rather than a repository-wide statement that every process owner is bounded. H-I4-090..095 remain closed on malformed-resource causality/cfg proof surfaces absent owner change/falsification.
- Candidate A (`Recovery::on_ack` future/unsent ACK) and Candidate B (`record_datagrams` mixed drop reasons) remain closed unless materially changed or falsified by exact-current source/tests.
- `SessionRuntime.events` retained-history capacity and D019 source-retention/no-reset remain maintainer/security policy gates. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely for freshness.

## FRONT HIGH — H-I4-112 H-I4-111 source/provenance mismatch

### Concrete defect

Exact developer source at `1d727f0` still has this control flow in `crates/neko-cli/tests/probe.rs::expired_preprogress_udp_session_is_retired_before_delivery_and_fresh_handshake_recovers`:

1. start a real `failover-server` and pass `ready_failover_server`;
2. run an intentionally expired real `failover-client` (`--duration 2`, `--test-first-data-delay-ms 1200`) through synchronous `.output().unwrap()`;
3. assert the expired client failed and emitted no UDP DeliveryAck validation;
4. run a fresh recovery real `failover-client` (`--duration 3`) through a second synchronous `.output().unwrap()`;
5. only after both clients return call `finish_server(server)`.

Either client can strand the gate if connect/negotiation/Noise/auth/DeliveryAck/lifecycle/shutdown regresses. Product `--duration` is behavior under test, not an independent harness deadline. This is release/item-4 **test-harness correctness + evidence truthfulness HIGH**, not production Session/Carrier/ACK/crypto/wire semantics.

The reachable provenance note compounds the issue by claiming these exact two owners were already converted at `1d727f0`; they were not. Preserve the historical gate record, but supersede/correct the false coverage claim.

### Closure contract

1. Reuse exact-current `bounded_client_output`; do not create another process framework.
2. Route the intentionally expired client through `bounded_client_output(..., Duration::from_secs(7))` (`--duration 2 + 5s`). Preserve `--test-first-data-delay-ms 1200`, expected failure, and the assertion that no `udp_delivery_ack_validated` is emitted.
3. Route the fresh recovery client through `bounded_client_output(..., Duration::from_secs(8))` (`--duration 3 + 5s`). Preserve every existing recovery, ACK-count, ordering, ResumeGuard and `failover_client_ok` / `failover_server_ok` assertion.
4. Do not touch the already-correct `first_udp_selection_loss` and `warm_readiness_failures` repairs except as needed for compilation. Do not mechanically convert keygen, help/capabilities, invalid-argument/configuration, or socket-free fixture `.output()` calls.
5. Existing `bounded_client_output_fails_when_client_never_exits` is sufficient for the ordinary shared deadline path unless this exact owner exposes a distinct failure mode; do not add duplicate generic sleep churn.
6. No decoder/parser/crypto-framing implementation change is implicated; do not run fuzz mechanically.
7. On the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist developer-local exact-tree provenance with reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust and clean-tree state.
8. Add a reachable erratum/superseding provenance record that explicitly says the `1d727f0` note's two `expired_preprogress` source-coverage bullets were false. Do not erase the original historical execution record.
9. Continue immediately to the next ownership/review slice after closure; do not wait for reviewer cadence and do not infer repository-wide queue exhaustion from this repair.

## Rolling queue — keep continuous after H-I4-112

Do not collapse this to one ticket. Dependency-ready order:

1. **H-I4-112 repair + corrected exact-tree provenance** — current FRONT HIGH; this fully closes the remaining H-I4-111 owner pair.
2. **Remaining process/socket/thread ownership causal sweep.** Continue exact-current `crates/neko-cli/tests/probe.rs`, `crates/neko-cli/tests/multistream.rs`, and other process tests. Pay special attention to later direct/indirect `.output()`, `wait`, `wait_with_output`, `try_wait`, periodic/endpoint-rebind clients, blocking socket `accept`/`recv`/`read_exact`, pipe drains and peer/reader joins. Classify each as exit/peer-completion proven, source-self-bounded, externally bounded, or concrete unbounded owner; repair only the last class. Preserve explicit fail-fast/self-bounded exclusions.
3. **Cross-platform CLI/process factual reconciliation.** Reconcile I4-CLI-PROC-096 and H-I4-097..112 with exact-current helper/cfg/socket/process semantics. Keep Linux-local execution, macOS/BSD source/cfg reasoning and Windows claims separate.
4. **Algorithmic/resource boundedness reconciliation.** Reconcile accepted boundedness reviews with current channel/output bounds, stdout accumulation, cleanup deadlines, socket peer lifetime, reader lifetime and child/thread ownership. Do not run capacity-pressure benchmarks or invent capacity/security policy values.
5. **Release packet / item-4 factual reconciliation.** `docs/release-security-review-packet.md` top coverage boundary predates H-I4-097..112. Qualify stale statements that process failure paths are globally bounded or review is exhausted; index reachable repair/review notes only after relevant exact-tree provenance exists. Local process/socket tests are not WAN/performance/security approval.
6. **Pre-auth malformed/rejection accounting exact-current reuse challenge.** Reuse prior independent review only where owner source/tests remain unchanged; otherwise narrowly re-challenge changed ownership. D019 remains a policy gate.
7. **CLI diagnostic / exit-code / JSON / human-output boundary exact-current reuse challenge.** Diagnostics remain evidence-only and never authentication/Delivery/Path/ACK evidence.
8. **Package/build/reproducibility spot re-challenge.** Verify recent process-test repairs do not stale manifests/scripts/provenance assumptions. Do not invent signing/SBOM/key-custody/publication policy.
9. **Reliable UDP / CarrierState / CarrierManager / FairScheduler / SessionRuntime targeted spot re-challenge.** Re-open materially changed owners; otherwise record exact-current reuse boundaries rather than rerunning equivalent sweeps.
10. **Observability + carrier adapters + dependency/build exact-current reuse challenge.** Same owner-diff rule; no checker/schema/framework/docs filler.
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
- Exact `1d727f0` has a reachable developer-local gate record, but its H-I4-111 provenance source-coverage claim is partly false; exact source wins. GitHub combined status exposed no hosted entries. Do not call the absence a hosted pass/fail result.
- Reviewer H-I4-112 is source/control-flow inspection only; reviewer did not run Rust/full gate, cross-platform execution, fuzz, WAN or performance work in this pass.
- Hosted Actions are cross-evidence, not a wait condition and not a replacement for the developer-local clean exact-tree gate.
- Fuzz only when wire decoder/parser/crypto framing changes materially.
- Standing VPS authorization permits bounded self-owned TCP/UDP lab work, but current authoritative classification remains `READY_LIVE: none`; no repeat live run without a new concrete question.
- Never decide D019, capacity/TTL/LRU/history/security values, signing/key-custody/SBOM/publication policy, core Session/Carrier/ACK/crypto/wire architecture, destructive/canonical migration, RC/freeze/release/production authority.
