# ChatGPT reviewer handoff — H-I4-104 periodic readiness HIGH FRONT

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- Reviewer exact-current source/spec anchor before H-I4-104: `a179b4c05c4291d44be35c5fbf33aab6ff53cbc9`. Reviewer finding commit: `dbb6d94190dda3c03a5ca2ffb168d039e1e0e88e` (`docs(review): flag H-I4-104 unbounded periodic readiness wait`).
- **H-I4-103 is CLOSED** at developer-owned `d2aa63b3b02f988a580919f397fabd6db65e4bcf`: after `malformed_classification_barrier` succeeds, post-barrier reader join now follows bounded child-exit observation, and `barrier_success_reader_join_bounded_when_child_lives_and_silent` exercises the live-silent child shape. Exact-tree developer-local provenance is `docs/notes/h-i4-103-provenance-d2aa63b-20260922.md`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, 2026-09-22T16:50:41Z → 16:56:35Z, Linux x86_64, rustc 1.98.0 stable. No GitHub-hosted workflow/status was visible for exact `d2aa63b` during this reviewer pass. This is developer-reported local provenance, not reviewer-local execution.
- **H-I4-104 is CLOSED** at `0467152d4e595c30ba6373333cd19bf8fa382cc0`: `start_periodic_server` replaced its raw `read_line` readiness loop with `wait_for_ready_marker(5s)` — the shared bounded primitive bounds the wait and converges child/pipe ownership on timeout/EOF/error. `start_periodic_server_bounded_when_binary_exits_silently` proves bounded panic on a silently-exiting binary. Exact-tree provenance: `docs/notes/h-i4-104-provenance-0467152-20260922.md` — `check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-22T17:50:09Z → 17:56:00Z, Linux x86_64, rustc 1.98.0.
- H-I4-097..103 remain closed on their original process-cleanup claims except where a new exact-current counterexample explicitly reopens a distinct owner. H-I4-090..095 remain closed on malformed-resource causality/cfg proof surfaces absent owner change/falsification.
- Candidate A (`Recovery::on_ack` future/unsent ACK) and Candidate B (`record_datagrams` mixed drop reasons) remain closed unless materially changed or falsified by exact-current source/tests.
- `SessionRuntime.events` retained-history capacity and D019 source-retention/no-reset remain maintainer/security policy gates. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely for freshness.

## FRONT HIGH — H-I4-104 `start_periodic_server` readiness CLOSED at `0467152`

### Concrete defect

Exact-current `crates/neko-cli/tests/probe.rs::start_periodic_server(...)` still spawns the periodic server and then waits for `periodic_server_ready` with a raw main-thread `BufRead::read_line()` loop. Unlike the already-hardened `wait_for_ready_marker(...)` path used by other process-test owners, this readiness edge has no channel timeout, independent harness deadline or bounded cleanup.

A live child that keeps stdout open but never emits a complete readiness marker can therefore block `read_line()` indefinitely. The helper has not returned yet, so `finish_server`, `bounded_wait_exit`, `bounded_reap_or_kill` and their cleanup proofs are unreachable.

The command's nominal `--duration 5` is runtime behavior under test, not an independent harness bound. A lifecycle/readiness regression that also prevents nominal exit must become bounded negative evidence rather than a hung test. This helper is shared by multiple periodic Session tests (delayed confirmations, key update, schedule mismatch, missing/duplicate ACK, setup timeout and malformed setup), so the defect is a shared process-test owner.

Full finding: `docs/reviews/reviewer-h-i4-104-periodic-readiness-boundedness-20260922.md`.

### Closure contract

1. Replace the raw readiness `read_line()` loop in `start_periodic_server` with the existing bounded readiness primitive (`wait_for_ready_marker`) or a strictly equivalent narrow wrapper. Preserve the `ReadyServer` startup-log/stdout ownership and exact `periodic_server_ready` marker semantics.
2. Reuse an existing test-local readiness bound. Do not add a new repository-wide timeout, capacity or security-policy value.
3. Add a focused deterministic negative regression for this owner: child remains live with stdout open but silent / never completes the readiness marker; periodic readiness must fail within its local bound and converge child/reader ownership. If the caller delegates completely to an already-covered helper, keep the caller-specific regression minimal rather than building a generic process framework.
4. Preserve H-I4-097..103 cleanup behavior and H-I4-090..095 causality/cfg proofs. Do not weaken `ReadyProof` / `BarrierProof`, FD/RSS margins, `ATTEMPTS`, post-release diagnostic checks or authenticated lifecycle assertions.
5. No decoder/parser/crypto-framing change is implicated; do not run fuzz mechanically.
6. Final pushed source/test SHA: run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist developer-local exact-tree provenance with reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust.
7. Continue immediately into the next process ownership slice after closure; do not wait for reviewer cadence and do not declare repository-wide queue exhaustion from this narrow repair.

## Rolling queue — keep continuous after H-I4-104

Do not collapse this to one ticket. Dependency-ready order:

1. **H-I4-104 repair + caller-specific bounded negative regression + exact-tree provenance** — current FRONT HIGH.
2. **Remaining process wait/output/join ownership causal sweep.** Re-read exact-current direct/indirect `try_wait`, `wait`, `wait_with_output`, `.output()`, stdout/stderr drain, reader `join`, channel timeout and child-ownership sites in `crates/neko-cli/tests/probe.rs` and other process-test owners. Classify each as exit-proven, success-path self-bounded, or failure-path externally bounded. Repair only concrete unproven-exit/hang counterexamples; ordinary synchronous CLI calls are not automatically defects.
3. **Cross-platform CLI/process factual reconciliation.** Reconcile I4-CLI-PROC-096 and H-I4-097..104 with exact-current helper/cfg semantics. Keep Linux-local execution, macOS/BSD source/cfg reasoning and Windows claims separate.
4. **Algorithmic/resource boundedness reconciliation.** Reconcile accepted boundedness reviews with current channel bounds, stdout accumulation, cleanup deadlines, reader lifetime and child ownership. No capacity-pressure benchmark and no invented policy/security values.
5. **Release packet / item-4 factual reconciliation.** Qualify stale statements that all process failure paths are bounded or that the repository queue is exhausted. Retain valid H-I4-090..103 boundaries. Do not promote local process tests to WAN/performance/security approval.
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
