# ChatGPT reviewer handoff — H-I4-099 bounded `finish_server` HIGH FRONT

## Current repository truth

- Synchronize to current `main` before work. Code/tests/reachable pushed commits outrank this prose, chat memory and stale checkbox state.
- Reviewer re-read the required governance/spec set and exact-current process-test owner through `03c5b5410a5494a204737e8bda4a90633fa71c1c`, accepted the developer H-I4-098 repair at `1d72566f62385a7298ed91a4f8c64aa93b2a3b55`, then added H-I4-099 at `309b75cf685d1b9a48534762c17b9a8a68e15a65`.
- **H-I4-097 remains CLOSED** at `76f33be72a1c3c24152193fb174194631cb7cf95`: `bounded_reap_or_kill` fails closed on `try_wait`/`kill` errors and bounds post-kill reap with a local deadline rather than blocking `wait()`.
- **H-I4-098 remains CLOSED** at `1d72566f62385a7298ed91a4f8c64aa93b2a3b55`: the reliable-UDP incomplete-settlement test no longer bypasses the bounded cleanup primitive. Developer-local exact-tree provenance is `docs/notes/h-i4-098-provenance-1d72566-20260922.md`; this is not reviewer-local or hosted-CI evidence.
- **H-I4-099 is OPEN / HIGH:** `finish_server` calls blocking `server.stdout.read_to_string(...)` while the child is still live/unproven-exited, then calls `child.wait()`. If the child stays alive with stdout open, the stdout drain can hang before `wait()` is reached. This falsifies the earlier repository-wide process claim that child ownership converges on every failure shape.
- H-I4-090..095 remain closed on their bounded malformed-resource causality/cfg proof surfaces absent a new concrete counterexample. Valid I4-CLI-PROC-096 and H-I4-097/098 fixes remain retained; only the broader process-boundedness conclusion is reopened by H-I4-099.
- Candidate A (future/unsent Recovery ACK), Candidate B (mixed datagram drop reasons), Session/Carrier/ACK separation, CarrierState/CarrierManager, FairScheduler/flow accounting, carrier adapters and observability remain closed unless materially changed or falsified.
- `SessionRuntime.events` retained-history capacity and D019 source-retention/no-reset remain maintainer/security policy gates. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely for freshness.

## FRONT HIGH — H-I4-099 `finish_server` stdout drain is not locally bounded

Authoritative reviewer finding: `docs/reviews/reviewer-h-i4-099-finish-server-stdout-wait-20260922.md` at `309b75cf685d1b9a48534762c17b9a8a68e15a65`.

### Concrete defect

Exact-current `crates/neko-cli/tests/probe.rs::finish_server` does:

```rust
let mut remainder = String::new();
server.stdout.read_to_string(&mut remainder).unwrap();
let status = server.child.wait().unwrap();
```

The stdout drain begins before any child-exit proof. `read_to_string` waits for EOF; if a server process remains live while keeping stdout open, the test can hang indefinitely before it reaches `wait()`. A nominal server `--duration` or expected signal handling is runtime intent, not an independent harness bound if that lifecycle path is what regressed.

This directly contradicts the process no-finding at `49925acf7d2d77595562d9ad3327f4b145a31fd1`, whose reasoning treated “stdout drained before wait” as sufficient convergence. Draining the pipe is itself an unbounded wait surface when exit is unproven.

This is **item-4/release-evidence process-test boundedness**, not evidence of a production/runtime transport leak and not a Session/Carrier/ACK/crypto/wire architecture finding.

### Closure contract

1. Make `finish_server` locally bounded. It must not block on stdout EOF while child exit remains unproven.
2. Preserve complete stdout/log collection on normal exit. Prefer a narrow shape reusing established patterns: off-thread stdout read + bounded child-exit/reap observation; on deadline fail closed, invoke the already-reviewed bounded termination/reap primitive, and only then reconcile/join the reader after exit/pipe closure is established.
3. Add a deterministic negative regression using a child that keeps stdout open and remains alive beyond the helper deadline; prove bounded failure rather than hang. Preserve existing positive lifecycle/log checks.
4. Do not create a general process framework, alter protocol/runtime semantics, change timeout/security/capacity policy outside the test helper contract, or touch D019.
5. Focused tests, then final pushed source/test SHA: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree; persist developer-local exact-tree provenance with SHA, UTC start/end, exit codes, OS/arch and stable Rust. No fuzz unless decoder/parser/crypto-framing owners change.
6. Immediately continue the owner-by-owner process-cleanup causal re-challenge after the repair; do not wait for reviewer cadence.

## Rolling queue — keep continuous after H-I4-099

Do not collapse this to one ticket. Dependency-ready order:

1. **H-I4-099 repair + negative regression + exact-tree provenance** — current FRONT HIGH.
2. **Process-cleanup causal re-challenge.** Re-read every exact-current direct `wait`, `wait_with_output`/`output`, stdout `read_to_string`/drain, reader-thread `join`, `bounded_reap_or_kill`, deadline and child-ownership site in `crates/neko-cli/tests/probe.rs` and any other process-test owner. Classify each as success-path/exit-proven or failure-path/bounded. Repair only concrete unproven-exit blockers.
3. **Cross-platform CLI/process factual reconciliation.** Reconcile `49925ac`, I4-CLI-PROC-096, H-I4-097/098/099 and current helper semantics. Keep Linux-local execution, macOS/BSD source/cfg reasoning and Windows claims separate.
4. **Algorithmic/resource boundedness reconciliation.** Reconcile `d814457` / `5b7b22b`, channel bounds, stdout accumulation, cleanup deadlines, reader lifetime and process ownership. No capacity-pressure benchmark and no invented policy/security numbers.
5. **Release packet / item-4 factual reconciliation.** Qualify stale claims that all process failure paths are bounded or that the repository queue is exhausted. Retain valid H-I4-090..098 evidence boundaries. Do not promote local process tests to WAN/performance/security approval.
6. **Pre-auth malformed/rejection accounting exact-current reuse challenge.** Reuse `cfce41e` only if relevant owners remain unchanged; otherwise narrowly re-challenge changed ownership.
7. **CLI diagnostic/machine-output boundary exact-current reuse challenge.** Reuse `4b8e70a` only across unchanged owners; diagnostics remain evidence-only and never authentication/Delivery/Path/ACK evidence.
8. **Package/build/reproducibility spot re-challenge.** Verify process-test repairs did not stale manifests/scripts/provenance assumptions. Do not invent signing/SBOM/key-custody policy.
9. **Reliable UDP / CarrierState / CarrierManager / scheduler / SessionRuntime targeted spot re-challenge.** Re-open materially changed owners; otherwise record exact-current reuse boundaries rather than rerunning equivalent sweeps.
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

Reviewer H-I4-099 is exact-current source/control-flow review only; no reviewer-local Rust/full-gate, cross-platform execution, fuzz, WAN or performance execution is claimed.

## Stop / escalation conditions

Normal progress does not require administrator notification. Escalate only for an automatically undecidable BLOCKER/HIGH, core Session/Carrier/ACK/crypto/wire architecture decision, D019 or another genuine policy/value choice, destructive/canonical migration, action outside standing authorization, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
