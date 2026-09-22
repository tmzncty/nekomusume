# ChatGPT reviewer handoff — H-I4-092 reopened: post-barrier proof must be barrier-derived

## Current repository truth

- Synchronize to current `main` before work. Code/tests/reachable pushed commits outrank this prose and older review notes.
- Reviewer re-read the required governance/spec/status/release surface and reviewed all developer-owned commits after the prior reviewer handoff anchor `707c0dc7307ccbec14282c14fd05c80ee61f64a8`.
- Developer source/test commit reviewed:
  - `7f678fcac75233a1eac5e59875d56e12941e15cf` — introduces Linux-only `gated_resource_snapshot(gate, pid)` and routes both pre/post malformed-churn `/proc` samples through it.
  - Developer-reported clean exact-tree provenance is persisted in `docs/notes/h-i4-092-provenance-7f678fc-20260922.md`: `scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-22T01:50:34Z → ~01:57:00Z, Linux x86_64, rustc 1.98.0 stable.
  - No visible hosted status/workflow result is classified only as absence of hosted evidence, not CI failure.
- **Accepted from `7f678fc`:** the original split-refactor defect is fixed for the measurement call itself; the pre-churn sample executes its `!churn_started` proof inside the same helper call as `process_resource_snapshot(pid)`. The current physical post-snapshot order is also after the malformed-classification barrier. Current runtime ordering remains correct.
- **H-I4-092 is CLOSED** at `28e1caa6a68b7012ed7126e5e448608843ba7a6f`: `malformed_classification_barrier` now returns a `BarrierProof` value only on `Ok`, and the post-churn `gated_resource_snapshot` destructures that proof inside the gate — moving the post snapshot above barrier success fails to compile, closing the post-barrier proof gap flagged by `de67348`. The pre-churn edge keeps the `!churn_started` assert inside the same helper call as the measurement (prevents split-refactor regressions on both edges). Exact-tree provenance: `docs/notes/h-i4-092-provenance-28e1caa-20260922.md` — `check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-22T02:50:44Z → 02:56:35Z, Linux x86_64, rustc 1.98.0.
- H-I4-090/H-I4-091 runtime/source cleanup facts remain accepted: affirmative Linux baseline precedes malformed sends; classification wait is bounded off-thread; timeout/disconnect/EOF paths restore child/reader ownership; `malformed_or_unadmitted` is emitted only after `preauth.release(admission)`; missing-event/early-EOF regressions exist; non-Linux Unix warning-clean token placement is fixed.
- Candidate A (future/unsent Recovery ACK), Candidate B (mixed datagram drop reasons), H-I4-085..091, prior R9 process/result seams, Session/Carrier/ACK separation, CarrierState/CarrierManager, FairScheduler/flow accounting, carrier adapters, observability, package/operator, dependency/build, and prior CLI machine/human contracts remain closed unless materially changed or falsified by a new concrete counterexample.
- `SessionRuntime.events` retained-history capacity remains a maintainer/security policy gate; do not invent TTL/LRU/history-size/capacity values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat prior WAN/VPS evidence merely for freshness.

## FRONT HIGH — H-I4-092 post-barrier proof coupling

Do not redo the already-correct baseline ordering, bounded stdout wait, EOF cleanup, non-Linux Unix cfg repair, or post-release diagnostic ordering. Close only the remaining proof-seam:

1. Preserve the exact-current actual execution order, Linux-only `/proc` evidence boundary, FD/RSS margins, `ATTEMPTS`, bounded classification wait, failure-path kill/reap + reader join, and post-`preauth.release(admission)` diagnostic semantics.
2. Keep the existing coupled pre-churn helper behavior: `gated_resource_snapshot` must continue to execute the `!churn_started` proof in the same call as the baseline measurement, so moving the call after `churn_started = true` fails deterministically.
3. Replace the freely constructed post proof (`barrier_complete = true`) with a success value/token that is produced only by successful `malformed_classification_barrier(...)` completion. Smallest acceptable shape: return a tiny proof value alongside `(Child, BarrierReaderHandle)`, or use a tiny wrapper that can produce that value only on the `Ok` path.
4. The Linux post-snapshot helper/call must require that barrier-derived value as an argument. It must not remain satisfiable by an arbitrary `FnOnce()` / no-op / constant-true gate for the post edge.
5. Mutation requirement: moving the coupled post snapshot before successful barrier completion must fail to compile or deterministically fail before the resource comparison, independent of later log counting.
6. Keep `malformed_classification_barrier_fails_bounded_when_events_missing` and `malformed_classification_barrier_fails_bounded_on_early_stdout_eof`; do not weaken child kill/reap + reader-join ownership restoration.
7. Keep `malformed_or_unadmitted` diagnostic-only; do not promote it into authentication, Session, Carrier, ACK, wire or release semantics.
8. Do not change capacity/security/TTL/history numbers. No fuzz unless decoder/parser/crypto framing owners change.
9. Run focused CLI process tests, then on the **final pushed source/test SHA** run:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - verify clean tree
   Persist exact reachable SHA, UTC start/end, exit codes, OS/arch, `rustc --version`, and clean-tree state. Keep developer-local, hosted, reviewer-local, live and performance evidence classifications separate.
10. After closure continue immediately through the rolling queue; do not wait for reviewer cadence.

## Rolling queue — keep continuous after the HIGH

Maintain a real dependency-ordered queue; do not collapse to a one-ticket idle state:

1. **H-I4-092 barrier-derived post-proof repair + exact-tree provenance.** Smallest test/helper repair only.
2. **I4-PORT-RES exact-current independent causal re-challenge.** Re-attempt both causal edges after the proof repair: `READY -> affirmative pre-churn snapshot -> first malformed send` and `all bounded malformed inputs classified + corresponding cleanup complete -> affirmative post-churn snapshot`. Attempt whole-block and split/oracle mutations; a scoped no-finding is valid only if both fail closed.
3. **Pre-auth malformed/rejection resource-accounting bounded challenge.** Inspect exact-current `admit_carrier`, `charge_input`, invalid-negotiation release, response-admission rejection, cached/pending owner paths, and process-level pre-auth counters. Verify rejection cannot become authentication/delivery evidence and diagnostic emission does not retain unbounded state.
4. **Cross-platform CLI/process-test semantics.** Re-challenge Linux-only `/proc` cfgs, Unix process/signal assumptions, reader-thread lifecycle, child reap/join behavior, warning cleanliness, and non-Linux Unix compile/contract semantics.
5. **CLI diagnostic/machine-output boundary.** Challenge `malformed_or_unadmitted` scope, JSON/human-output collision, diagnostic parsing stability, exit-code coupling, and preserve evidence-only rather than protocol semantics.
6. **Algorithmic/resource boundedness reconciliation.** Challenge channel capacity/lifetime, timeout/EOF/disconnect cleanup, bounded line accumulation, queue/counter growth, and test-local resource bounds. No capacity-pressure benchmark and no new policy numbers.
7. **Package/build + reproducibility spot re-challenge.** Reuse prior no-finding only for unchanged owners; if recent CLI test/helper edits touch manifests/scripts indirectly, verify no stale provenance/package claim is promoted. Do not invent signing/SBOM/key-custody policy.
8. **Item-4 + release-packet factual reconciliation.** Reconcile the final resource-oracle closure and current bounded reviews with `docs/release-security-review-packet.md`, `docs/status.md`, `IMPLEMENTATION_PLAN.md`, provenance notes and review notes. Do not promote local resource testing into WAN/performance/security approval.
9. **Repository-wide 13-surface refill.** Re-apply the required core inventory. Reuse prior reviews only for genuinely unchanged owners; materially changed owners get a fresh bounded challenge. Queue exhaustion is legal only after the broad inventory finds no concrete defect, no uncovered implemented core surface, no READY review-support and no READY live question.
10. **Conditional live.** Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved real-network question within standing authorization. Otherwise keep `READY_LIVE: none`.

If several coherent slices complete in 10–30 minutes with stable quality, deepen the queue rather than waiting. Every 3–4 coherent slices or important repair cluster, perform one factual release/item-4 reconciliation instead of rewriting large docs after each small commit.

## Release item 3 / VPS boundary

Current classification remains `READY_LIVE: none`.

- IPv6 remains environment-blocked unless a real owned IPv6 endpoint/path appears.
- HY2 and repeated warm-failover lines remain frozen against same-class retry without a materially new hypothesis.
- Periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD and Experimental Track are not repeated for freshness when their bounded question is already answered or their current line is frozen/blocked.
- Standing authorization permits bounded self-owned client↔VPS ordinary TCP/UDP work only when a real new question becomes READY; it does not authorize third-party targets, production changes, privileged/exotic-carrier work reserved by the authorization file, or pressure/adversarial-load conditions requiring maintainer choice.

## Review / repair and evidence contract retained

For every bounded lane: read exact-current owners/tests plus applicable spec/ADR/status claim; state the invariant; try to falsify it with source reasoning and focused deterministic tests. If a concrete defect exists and committed semantics determine the answer, apply the smallest repair, add positive/negative regression, commit/push, run exact-tree gates, persist provenance and continue.

No-finding slices must name inspected owners, commands/tests actually run, exclusions and exact reachable anchor. Do not modify code merely to manufacture activity. Hosted CI is additional cross-evidence, never a waiting condition.

## Stop / escalation conditions

Normal progress does not require administrator notification. Escalate only for an automatically undecidable BLOCKER/HIGH, core Session/Carrier/ACK/crypto/wire architecture decision, D019 or another real policy/value choice, destructive/canonical migration, action beyond standing authorization, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
