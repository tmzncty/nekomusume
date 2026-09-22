# ChatGPT reviewer handoff — H-I4-091 non-Linux Unix closure HIGH

## Current repository truth

- Synchronize to current `main` before work. Code/tests/reachable pushed commits outrank this prose and older notes.
- Reviewer re-read the required governance/spec/status surface and reviewed every developer-owned commit after prior reviewed anchor `f77b853c4e84fc81ff9b4670c3d57c70d8b97a4d`.
- New developer commit reviewed:
  - `90831a72a4de7194e0c6ed1feeba806a45e5aaf6` — **test/helper implementation**: adds a successful-barrier token checked at the Linux post-resource snapshot site and makes the early-EOF `Ok(None)` barrier exit kill/reap the child and join the reader.
- **Accepted from `90831a7`:** the prior post-snapshot ordering source hole is repaired; stdout EOF now restores child/reader ownership deterministically; timeout/disconnect bounded cleanup and post-`preauth.release(admission)` malformed classification remain intact.
- **H-I4-091 remains FRONT HIGH** by exact reviewer continuation `cf97f6c5d199baad133d4329b1e363d4ef029b32` (`docs/reviews/reviewer-h-i4-091-portability-continuation-20260922.md`). Exact-current source introduces one concrete portability-gate defect: `let barrier_complete = true;` is compiled for all `#[cfg(unix)]` targets, but its only read is in a `#[cfg(target_os = "linux")]` block. On non-Linux Unix the variable is unused, while `scripts/check.sh` runs `cargo clippy --workspace --all-targets -- -D warnings`. A Linux-local green gate therefore cannot establish the required non-Linux Unix warning-clean contract.
- The previously requested focused **early-EOF/insufficient-classification regression is still not present**. The source cleanup itself is now acceptable, but this newly changed ownership branch still needs a small focused regression if feasible; do not build a generic process-test framework.
- `90831a7` has no accepted final developer-local exact-tree provenance persisted yet. No visible hosted status/workflow result is treated only as absence of hosted evidence, not CI failure.
- **H-I4-090 remains closed:** affirmative Linux baseline remains before the first malformed send; do not weaken it.
- Candidate A (future/unsent Recovery ACK), Candidate B (mixed datagram drop reasons), H-I4-085..090, prior R9 process/result seams, Session/Carrier/ACK separation, CarrierState/CarrierManager, FairScheduler/flow accounting, carrier adapters, observability, package/operator, dependency/build, and prior CLI machine/human contracts remain closed unless materially changed or falsified by a new concrete counterexample.
- `SessionRuntime.events` retained-history capacity remains a maintainer/security policy gate; do not invent TTL/LRU/history-size/capacity values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat prior WAN/VPS evidence merely for freshness.

## FRONT HIGH — H-I4-091 exact closure contract

Do not redo the already-correct runtime diagnostic ordering or timeout/disconnect cleanup. Close only the remaining portability/regression/provenance seam:

1. Preserve H-I4-090's pre-churn Linux `/proc` baseline and existing FD/RSS margins unchanged.
2. Keep a **mutation-sensitive post-barrier oracle** that is established only after successful `malformed_classification_barrier(...)` and is asserted immediately before the Linux post-resource snapshot. Moving the post snapshot above barrier success must deterministically fail or fail to compile.
3. Make that oracle **warning-clean on non-Linux Unix**. The current unconditional `barrier_complete` declaration inside the `#[cfg(unix)]` test with Linux-only use must not leave an unused local under `-D warnings`. Use the smallest cfg/placement repair; do not weaken the ordering proof.
4. Preserve `preauth.release(admission)` before the observable `malformed_or_unadmitted` event. Keep the event diagnostic-only; do not promote it into auth/Session/Carrier/ACK/wire/release semantics.
5. Preserve deterministic ownership restoration on **every unsuccessful barrier exit**, including EOF, timeout, and disconnect.
6. Add one focused early-EOF/insufficient-classification regression if it is not already present so the `Ok(None)` cleanup path is mechanically protected. Keep the existing timeout/missing-event regression. Do not create helper/schema/framework churn.
7. Keep non-Linux Unix lifecycle/socket semantics portable; do not claim `/proc` evidence outside Linux and do not claim macOS/BSD execution unless actually run there.
8. Do not invent capacity/security/TTL/history values. No fuzz unless wire decoder/parser/crypto framing owners change.
9. Run focused CLI process tests. Then on the **final pushed source/test SHA** run:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - verify clean tree
   Persist exact reachable SHA, UTC start/end, exit codes, OS/arch, `rustc --version`, and clean-tree state. Keep developer-local, hosted, reviewer-local, live, and performance evidence classifications separate.
10. After closure continue immediately through the rolling queue; do not wait for reviewer cadence.

## Rolling queue — keep continuous after the HIGH

Maintain a real dependency-ordered queue; do not collapse to a one-ticket idle state:

1. **H-I4-091 portability/regression/provenance closure.** Smallest test/helper repair only.
2. **I4-PORT-RES exact-current independent causal re-challenge.** Challenge both causal edges: `READY -> affirmative pre-churn snapshot -> first malformed send` and `all bounded malformed inputs classified + corresponding cleanup complete -> affirmative post-churn snapshot`. Attempt to move/split the oracle and verify it fails closed.
3. **Pre-auth malformed/rejection resource-accounting bounded challenge.** Inspect exact-current `admit_carrier`, `charge_input`, invalid-negotiation release, response-admission rejection, and pending-owner paths. Verify rejection cannot become authentication/delivery evidence and diagnostic support does not retain unbounded state. A scoped no-finding is valid.
4. **Cross-platform CLI/process-test semantics.** Re-challenge Linux-only `/proc` cfgs, Unix process/signal assumptions, child reap/join behavior, warning cleanliness, and non-Linux Unix compile/contract semantics.
5. **CLI diagnostic/machine-output boundary.** Challenge `malformed_or_unadmitted` scope, JSON/human-output collision, diagnostic stability, and keep it evidence-only rather than protocol semantics.
6. **Algorithmic/resource boundedness reconciliation.** Challenge reader/channel lifetime, timeout/EOF/disconnect cleanup, queue/counter growth, and test-local bounds. No capacity-pressure benchmark and no new policy numbers.
7. **Item-4 + release-packet factual reconciliation.** Reconcile repaired resource oracle and bounded reviews with `docs/release-security-review-packet.md`, `docs/status.md`, `IMPLEMENTATION_PLAN.md`, current provenance, and review notes. Do not promote local resource testing into WAN/performance/security approval.
8. **Repository-wide 13-surface refill.** Re-apply the required inventory. Reuse prior reviews only for genuinely unchanged owners; materially changed owners get a fresh bounded challenge. Queue exhaustion is legal only after the broad inventory finds no concrete defect, no uncovered implemented core surface, no READY review-support, and no READY live question.
9. **Conditional live.** Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved real-network question within standing authorization. Otherwise keep `READY_LIVE: none`.

If several coherent slices complete in 10–30 minutes with stable quality, deepen the queue rather than waiting. Every 3–4 coherent slices or important repair cluster, perform one factual release/item-4 reconciliation instead of rewriting large docs after each small commit.

## Release item 3 / VPS boundary

Current classification remains `READY_LIVE: none`.

- IPv6 remains environment-blocked unless a real owned IPv6 endpoint/path appears.
- HY2 and repeated warm-failover lines remain frozen against same-class retry without a materially new hypothesis.
- Periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD, and already-answered bounded live questions are not repeated for freshness.
- Standing authorization permits bounded self-owned client↔VPS ordinary TCP/UDP work only when a real new question becomes READY; it does not authorize third-party targets, production changes, privileged/exotic-carrier work reserved by the authorization file, or pressure/adversarial-load conditions requiring maintainer choice.

## Review / repair and evidence contract retained

For every bounded lane: read exact-current owners/tests plus applicable spec/ADR/status claim; state the invariant; try to falsify it with source reasoning and focused deterministic tests. If a concrete defect exists and committed semantics determine the answer, apply the smallest repair, add positive/negative regression, commit/push, run exact-tree gates, persist provenance, and continue.

No-finding slices must name inspected owners, commands/tests actually run, exclusions, and exact reachable anchor. Do not modify code merely to manufacture activity. Hosted CI is additional cross-evidence, never a waiting condition.

## Stop / escalation conditions

Normal progress does not require administrator notification. Escalate only for an automatically undecidable BLOCKER/HIGH, core Session/Carrier/ACK/crypto/wire architecture decision, D019 or another real policy/value choice, destructive/canonical migration, action beyond standing authorization, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
