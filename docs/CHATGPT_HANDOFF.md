# ChatGPT reviewer handoff — H-I4-098 direct cleanup bypass HIGH FRONT

## Current repository truth

- Synchronize to current `main` before work. Code/tests/reachable pushed commits outrank this prose, chat memory and stale checkbox state.
- Reviewer re-read the current required governance/spec set and developer-owned work through exact `c7a6eb32fcc3bfd4d8a7c12f74dcc3e11e704ba0`, then added H-I4-098 at `feba55bed0f05ae03e65c42a5f56ccd99f079c16`.
- **H-I4-097's helper-local defect is CLOSED** at `76f33be72a1c3c24152193fb174194631cb7cf95`: `bounded_reap_or_kill` now fails closed on `try_wait`/`kill` errors and uses a bounded post-kill `try_wait` poll instead of unconditional blocking `wait()`. `malformed_classification_barrier` timeout/EOF/channel-error exits use that helper. Developer-local exact-tree provenance is `docs/notes/h-i4-097-provenance-76f33be-20260922.md` (`scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, Linux x86_64, rustc 1.98.0 stable). This is not reviewer-local or hosted-CI evidence.
- **H-I4-098 is OPEN HIGH** because exact-current `crates/neko-cli/tests/probe.rs::reliable_udp_incomplete_settlement_fails_not_settled` still bypasses the repaired helper with `let _ = server.child.kill(); let _ = server.child.wait();`. Ignoring `kill()` failure and then entering an unconditional blocking `wait()` recreates the same unproven-exit cleanup class outside the helper.
- H-I4-090..095 remain closed on their bounded malformed-resource causality/cfg proof surfaces unless a new concrete counterexample appears. Valid H-I4-096/097 helper/readiness/channel work remains retained, but **the repository-wide process-cleanup / I4-BND / `queue exhausted` conclusion is not authoritative until H-I4-098 and its dependent reconciliation close**.
- Pre-auth malformed/rejection accounting, CLI diagnostic/machine-output, package/build reproducibility and unchanged core-runtime findings remain reusable only across unchanged owners and absent a concrete current-tree counterexample.
- Candidate A (future/unsent Recovery ACK), Candidate B (mixed datagram drop reasons), Session/Carrier/ACK separation, CarrierState/CarrierManager, FairScheduler/flow accounting, carrier adapters and observability remain closed unless materially changed or falsified.
- `SessionRuntime.events` retained-history capacity and D019 source-retention/no-reset remain maintainer/security policy gates. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely for freshness.

## FRONT HIGH — H-I4-098 direct `kill(); wait()` cleanup bypass

Authoritative reviewer finding: `docs/reviews/reviewer-h-i4-098-direct-kill-wait-20260922.md` at `feba55bed0f05ae03e65c42a5f56ccd99f079c16`.

### Concrete defect

Exact-current `crates/neko-cli/tests/probe.rs::reliable_udp_incomplete_settlement_fails_not_settled` ends its server cleanup with:

```rust
let _ = server.child.kill();
let _ = server.child.wait();
```

The `kill()` result is ignored. If termination fails while the child is still live, the following `wait()` has no local deadline and can block past the test's intended bound. The server's nominal `--duration 8` is not an adequate cleanup proof: a child/lifecycle failure is precisely the class the process-test surface must fail closed around rather than trusting self-termination.

This is **item-4/release-evidence process-test boundedness**, not evidence of a production/runtime transport leak and not a Session/Carrier/ACK/crypto/wire architecture finding.

### Closure contract

1. Apply the smallest test-only repair: route this direct cleanup through `bounded_reap_or_kill` (or an exactly equivalent already-reviewed bounded primitive). Do not create a new process framework.
2. Preserve the test's semantic purpose: the reliable-UDP client must still fail with settlement-incomplete evidence and must not emit `r9_udp_in_flight_settled`.
3. Preserve H-I4-090..097 valid causality/resource/reader behavior, `ReadyProof` / `BarrierProof`, Linux `/proc` measurement scope, FD/RSS margins, malformed `ATTEMPTS`, post-`preauth.release` diagnostic ordering and the one-item handoff.
4. Run the focused negative plus directly affected process-cleanup regressions.
5. Final pushed source/test SHA: run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, verify clean tree, and persist developer-local exact-tree provenance with SHA, UTC start/end, exit codes, OS/arch and stable Rust. No fuzz unless decoder/parser/crypto-framing owners change.
6. Commit/push and immediately continue the process-cleanup causal re-challenge; do not wait for reviewer cadence.

## Rolling queue — keep continuous after H-I4-098

Do not collapse this to one ticket. Dependency-ready order:

1. **H-I4-098 repair + focused regression + exact-tree provenance** — current FRONT HIGH.
2. **Process-cleanup causal re-challenge.** Re-read all exact-current owners of direct `kill`, `wait`, `wait_with_output`, `read_to_string`/stdout drain, reader-thread `join`, `bounded_reap_or_kill`, deadline handling and child ownership. Classify each wait as success-path/exit-proven or failure-path/bounded; repair only concrete unproven-exit blockers. A narrow helper being correct is not repository-wide exhaustion.
3. **Cross-platform CLI/process factual reconciliation.** Reconcile `49925ac`, I4-CLI-PROC-096, H-I4-097/098 and the current helper semantics. Keep Linux-local execution, macOS/BSD source/cfg reasoning and Windows claims separate.
4. **Algorithmic/resource boundedness reconciliation.** Reconcile `d814457` / `5b7b22b`, channel bounds, stdout accumulation, cleanup deadlines, reader lifetime and process ownership. No capacity-pressure benchmark and no new policy/security numbers.
5. **Release packet / item-4 factual reconciliation.** Remove or qualify stale statements that all process failure paths are bounded or that the repository queue is exhausted. Retain valid H-I4-090..097 evidence boundaries. Do not promote local process tests to WAN/performance/security approval.
6. **Pre-auth malformed/rejection accounting exact-current reuse challenge.** Reuse `cfce41e` only if relevant owners remain unchanged; otherwise narrowly re-challenge changed ownership.
7. **CLI diagnostic/machine-output boundary exact-current reuse challenge.** Reuse `4b8e70a` only across unchanged owners; diagnostics remain evidence-only and never authentication/Delivery/Path/ACK evidence.
8. **Package/build/reproducibility spot re-challenge.** Verify process-test repairs did not stale manifests/scripts/provenance assumptions. Do not invent signing/SBOM/key-custody policy.
9. **Reliable UDP / CarrierState / CarrierManager / scheduler / SessionRuntime targeted spot re-challenge.** Re-open materially changed owners; otherwise record exact-current reuse boundaries instead of rerunning equivalent sweeps.
10. **Observability + carrier adapters + dependency/build exact-current reuse challenge.** Same owner-diff rule; no checker/schema/docs filler.
11. **Repository-wide 13-surface refill.** Re-apply all 13 required surfaces after the HIGH and its dependent reconciliation close. Queue exhaustion is legal only if the broad inventory finds no concrete defect, no uncovered implemented core surface, no READY review-support and no READY live question.
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

A bounded no-finding challenge is valid item-4 support. If a concrete defect is found, convert it immediately into the repair lane rather than manufacturing additional checker/docs work.

## VPS / live boundary

Current classification remains `READY_LIVE: none`.

- IPv6 remains environment-blocked unless a real owned IPv6 endpoint/path appears.
- HY2 and repeated warm-failover current lines remain frozen against same-class retry without a materially new hypothesis.
- Periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD and Experimental Track are not repeated for freshness when their bounded question is already answered or the current line is frozen/blocked.
- Standing authorization permits bounded self-owned client<->VPS ordinary TCP/UDP work only when a real new question becomes READY. It does not authorize third-party targets, production route/firewall/DNS/proxy/tunnel/qdisc changes, privileged/exotic-carrier work reserved by the authorization file, or pressure/adversarial-load conditions requiring maintainer choice.

## Evidence discipline retained

Developer-reported local CI, persisted local provenance, reviewer-local execution, GitHub-hosted CI, live WAN evidence and performance conclusions are distinct evidence classes. All shared exact-tree provenance anchors must be reachable pushed commits. No unpublished/local-only SHA becomes repository evidence. No secret, protected identity, private topology or unnecessary absolute path belongs in provenance.

## Stop / escalation conditions

Normal progress does not require administrator notification. Escalate only for an automatically undecidable BLOCKER/HIGH, core Session/Carrier/ACK/crypto/wire architecture decision, D019 or another genuine policy/value choice, destructive/canonical migration, action outside standing authorization, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
