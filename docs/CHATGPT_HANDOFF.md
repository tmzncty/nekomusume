# ChatGPT reviewer handoff — H-I4-101 try_wait-error ownership HIGH FRONT

## Current repository truth

- Synchronize to current `main` before work. Code/tests/reachable pushed commits outrank this prose, chat memory and stale checkbox state.
- This reviewer pass re-read the required governance/spec set and current process-test owner on exact `main = daaec2fe6d366f288aa40fe3f0ab5e8491ceb0c3`, reviewed the developer-owned H-I4-100 repair/provenance, and added H-I4-101 at reviewer finding commit `ab9381b2b146f763d006b74b9e74540027ef39f5`.
- Developer-owned commits since the prior reviewer handoff `f0d96821b426840d0fc3fa2737e6b0badfcef3bc`:
  - `833962e4499d2d6b969485176f35151a242e43f9` — **tests / process-harness implementation**, replaces the direct application-deadline `Child::wait()` with caller-bounded `bounded_wait_exit` and updates `bounded_reap_or_kill` ownership to `&mut Child`;
  - `daaec2fe6d366f288aa40fe3f0ab5e8491ceb0c3` — **docs / provenance / handoff**, records the exact-tree gate and closes the original H-I4-100 direct-wait defect.
- **H-I4-100 is CLOSED on its original direct-wait claim** at `833962e`: `udp_application_wait_fails_at_bounded_overall_deadline` no longer blocks in `Child::wait()` before the `< 3 s` timing oracle. Developer-local exact-tree provenance is `docs/notes/h-i4-100-provenance-833962e-20260922.md`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, 2026-09-22T13:55:37Z → 14:01:20Z, Linux x86_64, rustc 1.98.0 stable. This remains developer-reported local evidence, not reviewer-local execution.
- **H-I4-101 is CLOSED** at `5da05085c1b006633174bba123db0f68ff88af8c`: `bounded_wait_exit`'s `try_wait` `Err` and `finish_server`'s `try_wait` `Err` now converge ownership via `bounded_kill_reap` (kill + bounded poll reap) before the error escapes — no `Err`/panic while a live child remains ownership-unresolved. `bounded_kill_reap` is a distinct primitive from `bounded_reap_or_kill` (no leading `try_wait`; used when the caller's own exit observation already failed). Exact-tree provenance: `docs/notes/h-i4-101-provenance-5da0508-20260922.md` — `check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-22T14:50:55Z → 14:56:40Z, Linux x86_64, rustc 1.98.0.
- H-I4-097..100 otherwise remain closed absent a new exact-current counterexample. H-I4-090..095 remain closed on their bounded malformed-resource causality/cfg proof surfaces absent owner changes or falsification.
- Candidate A (future/unsent `Recovery::on_ack`) and Candidate B (mixed datagram drop reasons) remain closed unless materially changed or falsified by exact-current source/tests.
- `SessionRuntime.events` retained-history capacity and D019 source-retention/no-reset remain maintainer/security policy gates. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely for freshness.

## FRONT HIGH — H-I4-101 unresolved child ownership on `try_wait()` error

### Concrete defect

Exact-current `crates/neko-cli/tests/probe.rs::bounded_wait_exit` has three exit classes:

```rust
match child.try_wait() {
    Ok(Some(status)) => return Ok(status),
    Ok(None) => {
        if Instant::now() >= end {
            bounded_reap_or_kill(child);
            return Err("bounded_wait_exit: child did not exit within deadline".to_string());
        }
    }
    Err(e) => return Err(format!("bounded_wait_exit: try_wait failed: {e}")),
}
```

The deadline branch converges through the bounded cleanup primitive, but the `try_wait` error branch returns ordinary `Err` while child exit is still unproven. Its current caller immediately `.expect(...)`s that result, so this failure shape panics and drops `Child` without a kill/reap guarantee. That violates the helper's own documented ownership postcondition that an error corresponds to a child that failed to exit within the supplied deadline and was reaped/killed accordingly.

`finish_server` contains the same ownership gap:

```rust
Err(e) => {
    panic!("finish_server: try_wait failed: {e}");
}
```

At that point the off-thread stdout reader may still be blocked on a live child's pipe. The branch is wall-clock bounded because it panics, but the child/process/pipe ownership is not shown to converge. This is **item-4/release-evidence process-harness correctness**, not evidence of a production transport leak and not a Session/Carrier/ACK/crypto/wire finding.

### Closure contract

1. Make every `bounded_wait_exit` failure shape truthful about ownership: either child exit is proven, or a bounded best-effort termination/reap convergence is attempted before ownership is abandoned. A `try_wait` error must not simply return an ordinary `Err` that callers can panic on with a live child still unresolved.
2. Apply the same ownership rule to `finish_server`'s `try_wait` error branch while preserving H-I4-099's off-thread stdout drain and normal-exit full log collection. Do not reintroduce blocking stdout drain or blocking `wait()` on an unproven-live child.
3. Keep the repair test-local and minimal. If `bounded_reap_or_kill` cannot express post-`try_wait`-error cleanup because it begins by repeating `try_wait`, split/refactor only the narrow termination/reap primitive needed. Do not build a generic process framework or invent a repository-wide timeout/security policy.
4. Keep failure semantics truthful: an unrecoverable OS cleanup error may fail the test explicitly, but comments/provenance must not call it successful cleanup. No silent ownership abandonment.
5. Focused deterministic regression/source proof, then final pushed source/test SHA: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree; persist developer-local exact-tree provenance with exact reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust. No fuzz unless decoder/parser/crypto-framing owners change.
6. Immediately continue the owner-by-owner process-cleanup sweep after repair; do not wait for reviewer cadence.

## Rolling queue — keep continuous after H-I4-101

Do not collapse this to one ticket. Dependency-ready order:

1. **H-I4-101 repair + focused regression/source proof + exact-tree provenance** — current FRONT HIGH.
2. **Remaining process wait/output causal re-challenge.** Re-read every exact-current `try_wait`, direct/indirect `wait`, `wait_with_output`, `output`, stdout/stderr drain, reader-thread `join`, channel timeout and child-ownership site in `crates/neko-cli/tests/probe.rs` and other process-test owners. Classify each as exit-proven/success-path bounded or failure-path externally bounded. Repair only concrete unproven-exit blockers; do not turn synchronous commands into a framework project.
3. **Cross-platform CLI/process factual reconciliation.** Reconcile I4-CLI-PROC-096 and H-I4-097..101 with current helper/cfg semantics. Keep Linux-local execution, macOS/BSD source/cfg reasoning and Windows claims separate.
4. **Algorithmic/resource boundedness reconciliation.** Reconcile accepted boundedness reviews with current channel bounds, stdout accumulation, cleanup deadlines, reader lifetime, child ownership and changed helpers. No capacity-pressure benchmark and no invented policy/security numbers.
5. **Release packet / item-4 factual reconciliation.** Qualify stale statements that every process failure path is bounded or that the repository queue is exhausted. Retain valid H-I4-090..100 boundaries. Do not promote local process tests to WAN/performance/security approval.
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

Reviewer H-I4-101 is exact-current source/control-flow review only; no reviewer-local Rust/full-gate, cross-platform execution, fuzz, WAN or performance execution is claimed.

## Stop / escalation conditions

Normal progress does not require administrator notification. Escalate only for an automatically undecidable BLOCKER/HIGH, core Session/Carrier/ACK/crypto/wire architecture decision, D019 or another genuine policy/value choice, destructive/canonical migration, action outside standing authorization, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
