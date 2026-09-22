# ChatGPT reviewer handoff — H-I4-102 `bounded_reap_or_kill` try_wait-error ownership HIGH FRONT

## Current repository truth

- Synchronize to current `main` before work. Code/tests/reachable pushed commits outrank this prose, chat memory and stale checkbox state.
- This reviewer pass re-read the required governance/spec set plus the exact-current process-test owner on `main = 6ed7dc28ddf5a32ff07c3adf0e3c127983d1e6f9`, accepted the developer-owned H-I4-101 repair/provenance, checked its hosted Rust CI separately, then added H-I4-102 at reviewer finding commit `1903a3cc1e591aa427d15b4be0c421eeb0ca0a10`.
- Developer-owned commits since the prior reviewer handoff `87bc841b47d6dcf3f97bac249316288798bd9947`:
  - `5da05085c1b006633174bba123db0f68ff88af8c` — **tests / process-harness implementation**, adds `bounded_kill_reap` and routes `bounded_wait_exit`/`finish_server` post-`try_wait` error ownership through it;
  - `6ed7dc28ddf5a32ff07c3adf0e3c127983d1e6f9` — **docs / provenance / handoff**, records the exact-tree gate and closes H-I4-101 on those original call-site claims.
- **H-I4-101 is CLOSED on its original two call-site claims** at `5da0508`: `bounded_wait_exit`'s `try_wait` `Err` and `finish_server`'s `try_wait` `Err` now attempt bounded termination/reap through `bounded_kill_reap` before the error/panic escapes. Developer-local exact-tree provenance is `docs/notes/h-i4-101-provenance-5da0508-20260922.md`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, 2026-09-22T14:50:55Z → 14:56:40Z, Linux x86_64, rustc 1.98.0 stable. GitHub-hosted Rust CI run `35742860989` for exact `5da0508` also completed successfully. These are distinct evidence classes; neither is reviewer-local execution.
- **H-I4-102 is CLOSED** at `319ac5d8caa7c245c35e4477bb3cf707ffb4fee2`: `bounded_reap_or_kill`'s own initial `try_wait` `Err` now runs `bounded_kill_reap` before panicking — the same ownership rule as the H-I4-101 caller-side repairs. Three states kept truthful: `Ok(Some)` exit proven, `Ok(None)` live → bounded kill+reap, `Err` observation failed → bounded best-effort kill+reap then explicit failure. Exact-tree provenance: `docs/notes/h-i4-102-provenance-319ac5d-20260922.md` — `check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-22T15:50:41Z → 15:56:25Z, Linux x86_64, rustc 1.98.0.
- H-I4-097..101 otherwise remain closed absent a new exact-current counterexample. H-I4-090..095 remain closed on their bounded malformed-resource causality/cfg proof surfaces absent owner changes or falsification.
- Candidate A (future/unsent `Recovery::on_ack`) and Candidate B (mixed datagram drop reasons) remain closed unless materially changed or falsified by exact-current source/tests.
- `SessionRuntime.events` retained-history capacity and D019 source-retention/no-reset remain maintainer/security policy gates. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely for freshness.

## FRONT HIGH — H-I4-102 shared cleanup wrapper loses ownership on its own `try_wait()` error

### Concrete defect

Exact-current `crates/neko-cli/tests/probe.rs::bounded_reap_or_kill` still has this control flow:

```rust
fn bounded_reap_or_kill(child: &mut Child) {
    match child.try_wait() {
        Ok(Some(_)) => return,
        Err(e) => {
            panic!("bounded_reap_or_kill: try_wait failed before cleanup: {e}");
        }
        Ok(None) => {}
    }
    bounded_kill_reap(child);
}
```

If that first `try_wait()` returns `Err`, exit is unproven and the wrapper panics before any termination/reap attempt. H-I4-101 introduced exactly the primitive needed for this class (`bounded_kill_reap`) but only used it at two callers whose *earlier* `try_wait` failed.

This is reachable from multiple exact-current process-test cleanup paths: readiness timeout/EOF/read-error/channel-disconnect, malformed-classification timeout/EOF/channel-error, bounded-wait deadline cleanup and other direct wrapper users. The outer path may be wall-clock bounded until cleanup begins, but process/pipe/resource ownership is not shown to converge if the wrapper's own observation errors.

This is **item-4/release-evidence process-harness correctness**, not evidence of a production transport leak and not a Session/Carrier/ACK/crypto/wire finding.

### Closure contract

1. Make `bounded_reap_or_kill`'s own `try_wait`-error branch follow the same ownership rule as H-I4-101: perform the existing bounded best-effort termination/reap path before the error/panic escapes instead of panicking immediately with exit unproven.
2. Preserve truthful classes: `Ok(Some(_))` = exit proven; `Ok(None)` = live -> bounded termination/reap; `Err(_)` = exit observation failed -> bounded best-effort termination/reap, then explicit failure. Cleanup errors may fail closed but must not be mislabeled as successful cleanup.
3. Keep the repair test-local and minimal. Reuse `bounded_kill_reap`; do not create a generic process framework and do not invent new repository-wide timeout/security numbers.
4. Preserve H-I4-097..101 bounds, H-I4-099 off-thread stdout drain, readiness/barrier reader ownership, and H-I4-090..095 malformed-resource causality/cfg proofs. No runtime/session/wire semantic change is needed.
5. Do not manufacture unsafe/platform-specific machinery merely to force a kernel `try_wait` error. Source-level ownership proof plus existing deterministic live-child/EOF/deadline regressions is acceptable unless a natural small regression seam exists.
6. Final pushed source/test SHA: run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist developer-local exact-tree provenance with reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust. No fuzz unless decoder/parser/crypto-framing owners change.
7. Immediately continue the process ownership sweep after repair; do not wait for reviewer cadence and do not declare repository-wide queue exhaustion from this narrow closure.

Full finding: `docs/reviews/reviewer-h-i4-102-bounded-reap-try-wait-error-ownership-20260922.md`.

## Rolling queue — keep continuous after H-I4-102

Do not collapse this to one ticket. Dependency-ready order:

1. **H-I4-102 repair + focused source/regression proof + exact-tree provenance** — current FRONT HIGH.
2. **Remaining process wait/output causal re-challenge.** Re-read every exact-current `try_wait`, direct/indirect `wait`, `wait_with_output`, `output`, stdout/stderr drain, reader-thread `join`, channel timeout and child-ownership site in `crates/neko-cli/tests/probe.rs` and other process-test owners. Classify each as exit-proven, success-path self-bounded, or failure-path externally bounded. Repair only concrete unproven-exit blockers; do not turn synchronous CLI test calls into a framework project without a concrete hang/control-flow counterexample.
3. **Cross-platform CLI/process factual reconciliation.** Reconcile I4-CLI-PROC-096 and H-I4-097..102 with current helper/cfg semantics. Keep Linux-local execution, macOS/BSD source/cfg reasoning and Windows claims separate.
4. **Algorithmic/resource boundedness reconciliation.** Reconcile accepted boundedness reviews with current channel bounds, stdout accumulation, cleanup deadlines, reader lifetime, child ownership and changed helpers. No capacity-pressure benchmark and no invented policy/security numbers.
5. **Release packet / item-4 factual reconciliation.** Qualify stale statements that every process failure path is bounded or that the repository queue is exhausted. Retain valid H-I4-090..101 boundaries. Do not promote local process tests to WAN/performance/security approval.
6. **Pre-auth malformed/rejection accounting exact-current reuse challenge.** Reuse prior independent review only where owner source/tests remain unchanged; otherwise narrowly re-challenge changed ownership. D019 remains a policy gate.
7. **CLI diagnostic / JSON / human-output boundary exact-current reuse challenge.** Diagnostics remain evidence-only and never authentication/Delivery/Path/ACK evidence.
8. **Package/build/reproducibility spot re-challenge.** Verify process-test repairs do not stale manifests/scripts/provenance assumptions. Do not invent signing/SBOM/key-custody policy.
9. **Reliable UDP / CarrierState / CarrierManager / FairScheduler / SessionRuntime targeted spot re-challenge.** Re-open materially changed owners; otherwise record exact-current reuse boundaries rather than rerunning equivalent sweeps.
10. **Observability + carrier adapters + dependency/build exact-current reuse challenge.** Same owner-diff rule; no checker/schema/docs filler.
11. **Repository-wide 13-surface refill.** Re-apply all required surfaces after the HIGH and dependent reconciliation close. Queue exhaustion is legal only if the broad inventory finds no concrete defect, no uncovered implemented core surface, no READY review-support and no READY live question.
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

Reviewer H-I4-102 is exact-current GitHub source/control-flow review only; no reviewer-local Rust/full-gate, cross-platform execution, fuzz, WAN or performance execution is claimed.

## Stop / escalation conditions

Normal progress does not require administrator notification. Escalate only for an automatically undecidable BLOCKER/HIGH, core Session/Carrier/ACK/crypto/wire architecture decision, D019 or another genuine policy/value choice, destructive/canonical migration, action outside standing authorization, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
