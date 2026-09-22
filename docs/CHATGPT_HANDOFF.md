# ChatGPT reviewer handoff — H-I4-107 multistream helper failure-ownership HIGH FRONT

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- **H-I4-106 direct owner defect is closed on its normal path** by developer-owned `a5e80bb85948419e20a01430ee8381bf475fa3bd`, with follow-up deterministic timeout regression `f3093d5a6c5ec063d29f08b0eada0a4afee6a538`. The three original `wait_with_output()` server owners in `crates/neko-cli/tests/multistream.rs` now use `bounded_wait_with_output`; developer-local exact-tree provenance is `docs/notes/h-i4-106-provenance-a5e80bb-20260922.md` (`PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, Linux x86_64, rustc 1.98.0 stable). No exact-`a5e80bb` hosted workflow/status was visible in the reviewer pass that opened H-I4-107; do not relabel this as hosted or reviewer-local execution.
- **H-I4-107 is OPEN HIGH** at reviewer anchor `0dfed87fc96f1427e333f8d42dde34f96a31ae2a`: the new `bounded_wait_with_output` helper is wall-clock bounded on its ordinary timeout path but still abandons unresolved child/pipe ownership on `try_wait`/cleanup error branches. Full finding: `docs/reviews/reviewer-h-i4-107-bounded-wait-helper-error-ownership-20260923.md`.
- H-I4-097..105 remain closed on their original child/readiness/barrier/socket-thread ownership claims except where an exact-current counterexample identifies a distinct owner. H-I4-090..095 remain closed on malformed-resource causality/cfg proof surfaces absent owner change/falsification.
- Candidate A (`Recovery::on_ack` future/unsent ACK) and Candidate B (`record_datagrams` mixed drop reasons) remain closed unless materially changed or falsified by exact-current source/tests.
- `SessionRuntime.events` retained-history capacity and D019 source-retention/no-reset remain maintainer/security policy gates. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely for freshness.

## FRONT HIGH — H-I4-107 bounded helper error-path ownership

### Concrete defect

Exact-current `crates/neko-cli/tests/multistream.rs::bounded_wait_with_output` starts stdout/stderr `read_to_end` threads before proving child exit and then has two ownership-invalid failure paths:

1. **Outer `try_wait Err`** immediately executes `panic!("try_wait failed: {e}")`. Child exit is unproven, no bounded termination/reap is attempted, and both pipe readers may still be blocked on a live child.
2. **Deadline cleanup** executes `let _ = child.kill()` and then treats `Ok(Some(_)) | Err(_)` from the reap-side `try_wait()` identically. Therefore `kill` failure is ignored and `try_wait Err` is incorrectly accepted as if reap/exit were established. The helper then panics before joining the readers; if the child still owns the write ends, pipe ownership is unresolved.

`f3093d5` proves the ordinary no-client deadline branch fails within a finite bound when kill/reap behave normally. It does not exercise or prove the error branches above.

Invariant: a bounded process helper must be ownership-truthful on every branch. `try_wait Err`, `kill Err`, and post-kill `try_wait Err` do **not** establish child exit. A branch may report bounded cleanup failure, but it must not silently classify unresolved child/pipe ownership as converged.

This is a release/item-4 **test-harness correctness HIGH**, not a production Session/Carrier/ACK/crypto/wire semantic finding.

### Closure contract

1. Keep the H-I4-106 normal deadline semantics and three converted call sites.
2. Initial `try_wait Err` must enter bounded best-effort terminate/reap before failing; do not immediately panic with unresolved child ownership.
3. Deadline cleanup must not ignore `kill()` result and must not treat post-kill `try_wait Err` as `Ok(Some)`. Preserve truthful classes: exit proven; bounded cleanup succeeded; bounded cleanup itself failed / exit unproven.
4. Do not block on stdout/stderr drain joins while exit/pipe closure is unproven. If cleanup itself fails and a live child may still own the write ends, fail closed without falsely claiming drain convergence. Prefer the smallest test-local repair; do not build a generic process framework.
5. Add focused regression coverage where portable. If forcing real OS `try_wait`/`kill` failures would require unsafe/kernel fault injection, source-level ownership proof plus the existing deterministic live-child timeout regression is acceptable; do not manufacture brittle platform tricks.
6. Preserve current multistream negotiation/crypto/JSON/human-output assertions. No decoder/parser/crypto-framing implementation change is implicated; do not run fuzz mechanically.
7. On the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist developer-local exact-tree provenance with reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust.
8. Continue immediately into the next ownership/review slice after closure; do not wait for reviewer cadence and do not infer repository-wide queue exhaustion from this helper repair.

## Rolling queue — keep continuous after H-I4-107

Do not collapse this to one ticket. Dependency-ready order:

1. **H-I4-107 repair + focused helper failure-ownership proof/regression + exact-tree provenance** — current FRONT HIGH.
2. **Remaining process/socket/thread ownership causal sweep.** Re-read exact-current direct/indirect `try_wait`, `wait`, `wait_with_output`, `.output()`, socket `accept`/`recv`/`read_exact`, stdout/stderr drain, peer/reader `join`, channel timeout and child/thread ownership sites in `crates/neko-cli/tests/probe.rs`, `crates/neko-cli/tests/multistream.rs`, and other process-test owners. Classify each as exit/peer-completion-proven, success-path source-self-bounded, or failure-path externally bounded. Repair only concrete unproven hang/ownership counterexamples; ordinary synchronous CLI calls are not automatically defects.
3. **Cross-platform CLI/process factual reconciliation.** Reconcile I4-CLI-PROC-096 and H-I4-097..107 with exact-current helper/cfg/socket/process semantics. Keep Linux-local execution, macOS/BSD source/cfg reasoning and Windows claims separate.
4. **Algorithmic/resource boundedness reconciliation.** Reconcile accepted boundedness reviews with current channel/output bounds, stdout accumulation, cleanup deadlines, socket peer lifetime, reader lifetime and child/thread ownership. No capacity-pressure benchmark and no invented policy/security values.
5. **Release packet / item-4 factual reconciliation.** Qualify stale statements that all process failure paths are bounded or that repository-wide review is exhausted. Retain valid H-I4-090..106 boundaries. Do not promote local process/socket tests to WAN/performance/security approval.
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
