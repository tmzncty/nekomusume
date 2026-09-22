# ChatGPT reviewer handoff — H-I4-100 direct bounded-wait HIGH FRONT

## Current repository truth

- Synchronize to current `main` before work. Code/tests/reachable pushed commits outrank this prose, chat memory and stale checkbox state.
- This reviewer pass re-read the required governance/spec set and current process-test owner on `main` beginning at `9e4b162e31c44e06a9f04a27db2e3f28181bf582`, reviewed all developer-owned commits since the prior handoff, and added H-I4-100 at reviewer finding commit `585c5c69418a1b2e423b3320fbea605a6b8d0a56`.
- Developer commit classification since prior reviewer handoff `702f782a658c739826f14a649cce2e7c8487abc3`:
  - `857ce9d9ad4507ba1a5c1c428c68de79d64f8c1b` — **tests / process-harness implementation**, H-I4-099 off-thread stdout drain + bounded child-exit polling;
  - `ea3e735a7e073093c947a6881556af7cf7f1e4fa` — **tests / process-harness repair**, raises the local exit-poll deadline to cover the existing max server duration and updates the negative regression;
  - `9e4b162e31c44e06a9f04a27db2e3f28181bf582` — **docs / provenance / handoff**, closes H-I4-099 and records exact-tree developer-local provenance.
- **H-I4-099 is CLOSED** at `ea3e735a7e073093c947a6881556af7cf7f1e4fa`: `finish_server` drains stdout off-thread, polls child exit with a local bound, and on deadline routes through `bounded_reap_or_kill`. Developer-local exact-tree provenance is `docs/notes/h-i4-099-provenance-ea3e735-20260922.md`. Hosted Rust CI run `35730719557` also completed successfully; hosted CI remains a separate evidence class.
- **H-I4-097 and H-I4-098 remain CLOSED** absent a new exact-current counterexample. H-I4-090..095 likewise remain closed on their bounded malformed-resource causality/cfg proof surfaces absent owner changes or falsification.
- **H-I4-100 is CLOSED** at `833962e`: `udp_application_wait_fails_at_bounded_overall_deadline` now calls `bounded_wait_exit(&mut server.child, 5s)` — polls `try_wait` against a caller-supplied deadline; on deadline routes through `bounded_reap_or_kill` (now `&mut Child`) and fails closed. The nominal `--duration` remains runtime intent, not a harness bound; the `< 3 s` overall oracle is preserved. Exact-tree provenance: `docs/notes/h-i4-100-provenance-833962e-20260922.md` — `check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-22T13:55:37Z → 14:01:20Z, Linux x86_64, rustc 1.98.0.
- Candidate A (future/unsent `Recovery::on_ack`) and Candidate B (mixed datagram drop reasons) remain closed unless materially changed or falsified by exact-current source/tests.
- `SessionRuntime.events` retained-history capacity and D019 source-retention/no-reset remain maintainer/security policy gates. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely for freshness.

## FRONT HIGH — H-I4-100 direct `Child::wait()` defeats the UDP deadline boundedness oracle

### Concrete defect

Exact-current `crates/neko-cli/tests/probe.rs::udp_application_wait_fails_at_bounded_overall_deadline` starts a server with `--duration 1`, runs an authenticated UDP client with a deliberate 1.25 s data delay, then performs:

```rust
let status = server.child.wait().unwrap();
let elapsed = started.elapsed();
let mut log = server.startup_log;
server.stdout.read_to_string(&mut log).unwrap();
```

The test later asserts `elapsed < 3 s`, but the elapsed sample occurs only after the direct blocking wait returns. A live child caused by a server shutdown/lifecycle regression therefore produces a hang, not a deterministic bounded failure. The nominal server duration is runtime intent, not an independent process-harness deadline when the exit path itself is under test.

The post-exit stdout/stderr drains are not the finding: once child exit is actually proven, EOF is causally available. The unbounded direct wait before that proof is the HIGH.

This is **item-4/release-evidence process-test boundedness**, not evidence of a production/runtime transport leak and not a Session/Carrier/ACK/crypto/wire architecture finding.

### Closure contract

1. Replace this direct `server.child.wait()` with a narrow bounded exit observer. Prefer `try_wait` polling against the already-existing test upper-bound contract and route deadline cleanup through `bounded_reap_or_kill` before failing closed.
2. Preserve the current semantic oracle: the delayed authenticated UDP application operation must fail, server status must be unsuccessful, the current lower-bound timing check must remain meaningful, and the current `< 3 s` overall upper-bound oracle must remain truthful. Do not widen runtime/session semantics.
3. Do not create a general process framework or a new repository-wide timeout policy. If a helper is useful, it should accept an explicit caller-supplied deadline/bound and remain test-local.
4. Add only the minimum deterministic regression required to mechanically prove a live child cannot make this path hang. Fixture timing is not protocol/security/capacity policy.
5. Focused tests, then final pushed source/test SHA: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree; persist developer-local exact-tree provenance with exact reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust. No fuzz unless decoder/parser/crypto-framing owners change.
6. Immediately continue the owner-by-owner process-cleanup sweep after repair; do not wait for reviewer cadence.

## Rolling queue — keep continuous after H-I4-100

Do not collapse this to one ticket. Dependency-ready order:

1. **H-I4-100 repair + focused regression + exact-tree provenance** — current FRONT HIGH.
2. **Remaining process wait/output causal re-challenge.** Re-read every exact-current direct `wait_with_output`, `output`, stdout drain, reader-thread `join`, `try_wait`, deadline and child-ownership site in `crates/neko-cli/tests/probe.rs` and any other process-test owner. Classify each as exit-proven/success-path bounded or failure-path externally bounded. Repair only concrete unproven-exit blockers; do not turn every synchronous command into a framework project.
3. **Cross-platform CLI/process factual reconciliation.** Reconcile I4-CLI-PROC-096 and H-I4-097..100 with current helper/cfg semantics. Keep Linux-local execution, macOS/BSD source/cfg reasoning and Windows claims separate.
4. **Algorithmic/resource boundedness reconciliation.** Reconcile the accepted boundedness reviews with current channel bounds, stdout accumulation, cleanup deadlines, reader lifetime, process ownership, and any changed helper. No capacity-pressure benchmark and no invented policy/security numbers.
5. **Release packet / item-4 factual reconciliation.** Qualify any stale statement that every process failure path is bounded or that the repository queue is exhausted. Retain valid H-I4-090..099 evidence boundaries. Do not promote local process tests to WAN/performance/security approval.
6. **Pre-auth malformed/rejection accounting exact-current reuse challenge.** Reuse prior independent review only where owner source/tests remain unchanged; otherwise narrowly re-challenge changed ownership. D019 remains a policy gate.
7. **CLI diagnostic / JSON / human-output boundary exact-current reuse challenge.** Diagnostics remain evidence-only and never authentication/Delivery/Path/ACK evidence.
8. **Package/build/reproducibility spot re-challenge.** Verify process-test repairs do not stale manifests/scripts/provenance assumptions. Do not invent signing/SBOM/key-custody policy.
9. **Reliable UDP / CarrierState / CarrierManager / FairScheduler / SessionRuntime targeted spot re-challenge.** Re-open materially changed owners; otherwise record exact-current reuse boundaries rather than rerunning equivalent sweeps.
10. **Observability + carrier adapters + dependency/build exact-current reuse challenge.** Same owner-diff rule; no checker/schema/docs filler.
11. **Repository-wide 13-surface refill.** Re-apply all 13 required surfaces after the HIGH and dependent reconciliation close. Queue exhaustion is legal only if the broad inventory finds no concrete defect, no uncovered implemented core surface, no READY review-support and no READY live question.
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

Reviewer H-I4-100 is exact-current source/control-flow review only; no reviewer-local Rust/full-gate, cross-platform execution, fuzz, WAN or performance execution is claimed.

## Stop / escalation conditions

Normal progress does not require administrator notification. Escalate only for an automatically undecidable BLOCKER/HIGH, core Session/Carrier/ACK/crypto/wire architecture decision, D019 or another genuine policy/value choice, destructive/canonical migration, action outside standing authorization, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
