# ChatGPT reviewer handoff — H-I4-103 barrier-success reader join HIGH FRONT

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- Reviewer exact-current source/spec anchor before the new finding: `33085124fc30acbe15183bcaa9b0478ef5c50ed3`. Reviewer finding commit: `657381d4c5bee7dac5a0eb62c8de8ddfc3bf43fe` (`docs(review): flag H-I4-103 unbounded barrier reader success join`).
- Developer-owned H-I4-102 source/test commit `319ac5d8caa7c245c35e4477bb3cf707ffb4fee2` is accepted on its original claim: `bounded_reap_or_kill` now routes its own initial `try_wait()` error through `bounded_kill_reap` before explicit failure. Developer-local exact-tree provenance is `docs/notes/h-i4-102-provenance-319ac5d-20260922.md`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, 2026-09-22T15:50:41Z → 15:56:25Z, Linux x86_64, rustc 1.98.0 stable. No GitHub-hosted workflow/status was visible for exact `319ac5d` during this reviewer pass. This is developer-reported local provenance, not reviewer-local execution.
- **H-I4-103 is CLOSED** at `d2aa63b3b02f988a580919f397fabd6db65e4bcf`: after `malformed_classification_barrier` succeeds, the post-barrier `reader_handle.join()` now follows `bounded_wait_exit` — child exit makes stdout EOF so the `read_line` loop returns and the join is EOF-bounded. `barrier_success_reader_join_bounded_when_child_lives_and_silent` proves bounded join after the barrier count is satisfied. Exact-tree provenance: `docs/notes/h-i4-103-provenance-d2aa63b-20260922.md` — `check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-22T16:50:41Z → 16:56:35Z, Linux x86_64, rustc 1.98.0.
- H-I4-097..102 remain closed on their original bounded process-cleanup claims absent a new exact-current counterexample. H-I4-090..095 remain closed on their malformed-resource causality/cfg proof surfaces absent owner change or falsification.
- Candidate A (future/unsent `Recovery::on_ack`) and Candidate B (mixed datagram drop reasons) remain closed unless materially changed or falsified by exact-current source/tests.
- `SessionRuntime.events` retained-history capacity and D019 source-retention/no-reset remain maintainer/security policy gates. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely for freshness.

## FRONT HIGH — H-I4-103 barrier-success reader join CLOSED at `d2aa63b`

### Concrete defect

Exact-current `crates/neko-cli/tests/probe.rs::malformed_classification_barrier(...)` runs the child stdout `read_line()` loop in a reader thread and uses a bounded `recv_timeout` while counting the required malformed classifications. On success it drops the receiver and returns the still-live `child`, the `reader_handle`, and `BarrierProof`.

Dropping the receiver does not wake a thread already blocked in `read_line()`. The reader exits only after another stdout line lets `tx.send(...)` observe the disconnected receiver, or after EOF/read error.

The positive malformed-churn test later runs the authenticated failover client and then immediately executes:

```rust
let (stdout, barrier_lines) = reader_handle.join().expect("reader thread joinable");
```

This raw join occurs **before** any bounded child-exit observation or `finish_server(...)` cleanup. The comment assumes the client run necessarily caused another server stdout line, but that is not a harness-local proof. A lifecycle/diagnostic regression in which the client returns while the server remains live and emits no additional stdout strands the reader and blocks `join()` indefinitely instead of producing bounded negative evidence.

A deterministic counterexample can be built test-locally without protocol or network-policy change: a child emits exactly the required `malformed_or_unadmitted` line, then stays alive and silent with stdout open. The barrier succeeds; a raw join cannot complete until the child/pipe changes state.

This is item-4 / release-evidence process-harness correctness. It is not evidence of a production Session/Carrier/ACK/crypto/wire defect.

Full finding: `docs/reviews/reviewer-h-i4-103-barrier-reader-success-join-20260922.md`.

### Closure contract

1. Remove the direct unbounded post-barrier success `reader_handle.join()`. Join only after child exit/pipe closure is mechanically established, or after reader completion is itself observed through a local bound.
2. If the bound expires while child/reader ownership is unresolved, fail closed through the existing bounded process cleanup path before any final join. Reuse existing helpers or a minimal test-local helper; do not create a generic process framework.
3. Add a deterministic negative regression: satisfy the barrier count, then keep the child alive with stdout open and emit no additional line. The regression must terminate within its local bound and converge child/reader ownership instead of hanging.
4. Preserve H-I4-090..095: READY-derived baseline, `churn_started`, `ATTEMPTS`, barrier-derived post proof, Linux-only affirmative `/proc` measurement, existing FD/RSS margins, and the final authenticated success/lifecycle assertions.
5. Preserve H-I4-097..102 cleanup behavior. No runtime/session/wire/crypto semantic change and no new timeout/capacity/security policy values.
6. No decoder/parser/crypto-framing change is implicated; do not run fuzz mechanically.
7. Final pushed source/test SHA: run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist developer-local exact-tree provenance with reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust.
8. Continue immediately into the next process ownership slice after closure; do not wait for reviewer cadence and do not declare repository-wide queue exhaustion from this narrow repair.

## Rolling queue — keep continuous after H-I4-103

Do not collapse this to one ticket. Dependency-ready order:

1. **H-I4-103 repair + deterministic negative regression + exact-tree provenance** — current FRONT HIGH.
2. **Remaining process wait/output ownership causal sweep.** Re-read every exact-current `try_wait`, direct/indirect `wait`, `wait_with_output`, `.output()`, stdout/stderr drain, reader-thread `join`, channel timeout and child-ownership site in `crates/neko-cli/tests/probe.rs` and other process-test owners. Classify each as exit-proven, success-path self-bounded, or failure-path externally bounded. Repair only concrete unproven-exit/hang counterexamples; do not turn ordinary synchronous CLI calls into a process framework without a real control-flow defect.
3. **Cross-platform CLI/process factual reconciliation.** Reconcile I4-CLI-PROC-096 and H-I4-097..103 with exact-current helper/cfg semantics. Keep Linux-local execution, macOS/BSD source/cfg reasoning and Windows claims separate.
4. **Algorithmic/resource boundedness reconciliation.** Reconcile accepted boundedness reviews with current channel bounds, stdout accumulation, cleanup deadlines, reader lifetime and child ownership. No capacity-pressure benchmark and no invented policy/security values.
5. **Release packet / item-4 factual reconciliation.** Qualify any stale statement that all process failure paths are bounded or that the repository queue is exhausted. Retain valid H-I4-090..102 boundaries. Do not promote local process tests to WAN/performance/security approval.
6. **Pre-auth malformed/rejection accounting exact-current reuse challenge.** Reuse prior independent review only where owner source/tests remain unchanged; otherwise narrowly re-challenge changed ownership. D019 remains a policy gate.
7. **CLI diagnostic / exit-code / JSON / human-output boundary exact-current reuse challenge.** Diagnostics remain evidence-only and never authentication/Delivery/Path/ACK evidence.
8. **Package/build/reproducibility spot re-challenge.** Verify process-test repairs do not stale manifests/scripts/provenance assumptions. Do not invent signing/SBOM/key-custody policy.
9. **Reliable UDP / CarrierState / CarrierManager / FairScheduler / SessionRuntime targeted spot re-challenge.** Re-open materially changed owners; otherwise record exact-current reuse boundaries rather than rerunning equivalent sweeps.
10. **Observability + carrier adapters + dependency/build exact-current reuse challenge.** Same owner-diff rule; no checker/schema/docs filler.
11. **Repository-wide 13-surface refill.** Re-apply every required core surface after the HIGH and dependent reconciliation close. Queue exhaustion is legal only if the broad inventory finds no concrete defect, no uncovered implemented core surface, no READY review-support and no READY live question.
12. **Conditional live.** Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved real-network question within standing authorization. Otherwise keep `READY_LIVE: none`.

If several coherent slices complete in 10–30 minutes with stable quality, deepen the queue and continue implementation/review -> tests -> commit -> push -> next slice without waiting. Every 3–4 coherent slices or important repair cluster, do one factual item-4/release reconciliation rather than rewriting large docs after every small commit.

## Required 13-surface refill inventory

At each meaningful repository-wide refill, explicitly ask whether each surface has a reachable, dedicated, independent bounded review that remains valid on current owners:

1. `neko-reliable` UDP recovery — ACK range/future-unsent ACK/loss/retransmit/RTT/PTO/persistent congestion/Reno/fault simulation;
2. `neko-carrier::CarrierState` — generation/validation/hysteresis/single-active/drain/fail/activate;
3. Concurrent Carrier Manager / health / migration-back;
4. FairScheduler / multi-stream / session+stream flow-control accounting;
5. Memory/UDP/TCP carrier adapter close/error/resource semantics;
6. `SessionRuntime` lifecycle/resource/window/DeliveryAck accounting;
7. `neko-observe` projection/event/counter/high-water correctness;
8. package/reproducibility/operator scripts;
9. Cargo manifests/lock/features/build/native hooks/unsafe inheritance;
10. cross-platform CLI/process-test semantics;
11. CLI exit-code / JSON / human-output contract;
12. algorithmic resource boundedness, without capacity-pressure benchmarking or invented policy numbers;
13. release-packet factual consistency and evidence boundary.

A bounded no-finding challenge is valid item-4 support. If a concrete defect is found, convert it immediately into the repair lane rather than manufacturing checker/docs work.

## VPS / live boundary

Current classification remains `READY_LIVE: none`.

- IPv6 remains environment-blocked unless a real owned IPv6 endpoint/path appears.
- HY2 and repeated warm-failover current lines remain frozen against same-class retry without a materially new hypothesis.
- Periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD and Experimental Track are not repeated for freshness when their bounded question is already answered or the current line is frozen/blocked.
- Standing authorization permits bounded self-owned client<->VPS ordinary TCP/UDP work only when a real new question becomes READY. It does not authorize third-party targets, production route/firewall/DNS/proxy/tunnel/qdisc changes, privileged/exotic-carrier work reserved by the authorization file, or pressure/adversarial-load conditions requiring maintainer choice.

## Evidence discipline retained

Developer-reported local CI, persisted local provenance, reviewer-local execution, GitHub-hosted CI, live WAN evidence and performance conclusions are distinct evidence classes. All shared exact-tree provenance anchors must be reachable pushed commits. No unpublished/local-only SHA becomes repository evidence. No secret, protected identity, private topology or unnecessary absolute path belongs in provenance.

This reviewer H-I4-103 pass is exact-current GitHub source/control-flow review only; no reviewer-local Rust/full-gate, cross-platform execution, fuzz, WAN or performance execution is claimed.

## Stop / escalation conditions

Normal progress does not require administrator notification. Escalate only for an automatically undecidable BLOCKER/HIGH, core Session/Carrier/ACK/crypto/wire architecture decision, D019 or another genuine policy/value choice, destructive/canonical migration, action outside standing authorization, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
