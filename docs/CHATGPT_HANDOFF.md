# ChatGPT reviewer handoff — H-I4-092 split resource oracle HIGH

## Current repository truth

- Synchronize to current `main` before work. Code/tests/reachable pushed commits outrank this prose and older notes.
- Reviewer re-read the required governance/spec/status/release surface and reviewed all developer-owned commits after prior reviewed anchor `586b332ddcf47bcb0480df795dd850e4e489c5c8`.
- Developer source/test commit accepted for its narrow H-I4-091 fixes:
  - `8e507ac09b971ee9c3adbc7d3d452f9a87c29ec7` — Linux-gates `barrier_complete` for non-Linux Unix warning cleanliness and adds `malformed_classification_barrier_fails_bounded_on_early_stdout_eof`.
  - Developer-reported clean exact-tree provenance is persisted in `docs/notes/h-i4-091-provenance-8e507ac-20260922.md`: `scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, 2026-09-22T00:50:28Z → 00:56:25Z, Linux x86_64, rustc 1.98.0 stable.
  - No visible hosted status/workflow result is classified only as absence of hosted evidence, not CI failure.
- **Accepted from H-I4-091:** bounded off-thread classification wait; timeout/disconnect/EOF child kill+reap and reader join; post-`preauth.release(admission)` malformed diagnostic ordering; focused missing-event and early-EOF negative regressions; non-Linux Unix warning-clean token placement. The exact-current execution order is presently correct.
- **New HIGH H-I4-092** at reviewer commit `b4a7041799576f1e07ffabe74918eddf319d2fee`: the Linux pre/post resource measurements can be split away from the assertions/tokens meant to guard their ordering. Review note: `docs/reviews/h-i4-092-resource-oracle-split-20260922.md`.
- H-I4-092 is an **item-4/release evidence-oracle correctness** finding, not evidence of a runtime resource leak. The actual current snapshot order is correct; the regression protection is not mechanically coupled to the snapshot operation.
- H-I4-090/H-I4-091 source/runtime cleanup facts remain accepted except for the split-oracle closure claim superseded by H-I4-092.
- Candidate A (future/unsent Recovery ACK), Candidate B (mixed datagram drop reasons), H-I4-085..089, prior R9 process/result seams, Session/Carrier/ACK separation, CarrierState/CarrierManager, FairScheduler/flow accounting, carrier adapters, observability, package/operator, dependency/build, and prior CLI machine/human contracts remain closed unless materially changed or falsified by a new concrete counterexample.
- `SessionRuntime.events` retained-history capacity remains a maintainer/security policy gate; do not invent TTL/LRU/history-size/capacity values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat prior WAN/VPS evidence merely for freshness.

## FRONT HIGH — H-I4-092 exact closure contract

Do not redo the already-correct runtime diagnostic ordering, bounded stdout wait, EOF cleanup, or non-Linux Unix cfg repair. Close only the split-oracle seam:

1. Preserve H-I4-090's affirmative Linux baseline before the first malformed send, H-I4-091's post-cleanup classification barrier, existing FD/RSS margins, ATTEMPTS count, and Linux-only `/proc` evidence boundary.
2. Couple each resource snapshot to its phase proof so the assertion/token and `process_resource_snapshot(pid)` cannot be separated by an ordinary refactor. A small Linux-only test helper that performs the phase check and snapshot acquisition together is sufficient; do **not** create a general phase/checker framework.
3. Pre-churn: calling the coupled snapshot helper after churn starts must fail deterministically.
4. Post-barrier: the coupled snapshot helper must require a value/token that only exists after successful `malformed_classification_barrier(...)`; moving the coupled post snapshot before barrier success must fail to compile or fail deterministically.
5. Keep `malformed_classification_barrier_fails_bounded_when_events_missing` and `malformed_classification_barrier_fails_bounded_on_early_stdout_eof`; do not weaken kill/reap + reader-join ownership restoration.
6. Keep `malformed_or_unadmitted` diagnostic-only and after `preauth.release(admission)`; do not promote it into authentication, Session, Carrier, ACK, wire, or release semantics.
7. Do not change capacity/security/TTL/history numbers. No fuzz unless decoder/parser/crypto framing owners change.
8. Run focused CLI process tests, then on the **final pushed source/test SHA** run:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - verify clean tree
   Persist exact reachable SHA, UTC start/end, exit codes, OS/arch, `rustc --version`, and clean-tree state. Keep developer-local, hosted, reviewer-local, live, and performance evidence classifications separate.
9. After closure continue immediately through the rolling queue; do not wait for reviewer cadence.

## Rolling queue — keep continuous after the HIGH

Maintain a real dependency-ordered queue; do not collapse to a one-ticket idle state:

1. **H-I4-092 split-oracle repair + exact-tree provenance.** Smallest test/helper repair only.
2. **I4-PORT-RES exact-current independent causal re-challenge.** Re-attempt both causal edges with the coupled helper: `READY -> affirmative pre-churn snapshot -> first malformed send` and `all bounded malformed inputs classified + corresponding cleanup complete -> affirmative post-churn snapshot`. Attempt whole-block and split/oracle mutations; a scoped no-finding is valid only if both fail closed.
3. **Pre-auth malformed/rejection resource-accounting bounded challenge.** Inspect exact-current `admit_carrier`, `charge_input`, invalid-negotiation release, response-admission rejection, and pending-owner paths. Verify rejection cannot become authentication/delivery evidence and diagnostics do not retain unbounded state.
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
