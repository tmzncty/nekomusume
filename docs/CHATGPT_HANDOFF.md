# ChatGPT reviewer handoff — H-I4-091 ordering/cleanup closure HIGH

## Current repository truth

- Synchronize to current `main` before work. Code/tests/reachable pushed commits outrank this prose and older notes.
- Reviewer re-read the required repository governance/spec/status surface and reviewed all developer-owned commits after prior reviewed anchor `2da45746ad9580326aa94973f34376732369f5dc`:
  - `2a36892a958028ad3ce9c8ddd2e79e7c6cf68dec` — **implementation/test**: extracted a bounded malformed-classification barrier, added the missing-classification negative regression, kept invalid-negotiation classification after `preauth.release`, and retained the positive malformed-churn resource test.
  - `a5e8e7665041027234fd5d6ea8f125ddefe51bbc` — **docs/manifest**: corrected pre-auth producer-end anchors to the post-release diagnostic ordering.
  - `5ff70a8cefd68a87072933132b42c66701f6586c` — **handoff/provenance**: claimed H-I4-091 closed and persisted developer-local exact-tree provenance for `a5e8e76`.
- **Accepted repairs:** the direct blocking `read_line` wait in the main test thread is gone; timeout/disconnect paths use a bounded `recv_timeout`; invalid-negotiation malformed classification is currently emitted after `preauth.release(admission)`; the missing-classification timeout negative exists; and the recorded `a5e8e76` `check.sh`/`git diff --check`/clean-tree result is accepted as **developer-reported local provenance only**.
- **H-I4-091 is REOPENED / FRONT HIGH** by exact reviewer note `b36259c2f0702bd1e063d761cbba86699e3b363f` (`docs/reviews/reviewer-h-i4-091-ordering-eof-reopen-20260922.md`). Two closure requirements are still falsified by exact-current source:
  1. the supposed post-snapshot ordering oracle is not mechanically coupled to the snapshot; moving only the Linux post-resource snapshot above `malformed_classification_barrier(...)` still allows the later `barrier_lines` count to reach `ATTEMPTS` and pass, recreating the false-negative window;
  2. `malformed_classification_barrier` returns immediately on `Ok(None)` (stdout EOF before enough classifications) without deterministically killing/reaping the child or joining/reconciling the reader handle, while the current negative regression exercises only the timeout path.
- **H-I4-090 remains closed:** the affirmative Linux baseline is before `churn_started = true`, which flips before the first malformed `send_to`.
- Candidate A (future/unsent Recovery ACK), Candidate B (mixed datagram drop reasons), H-I4-085..090, prior R9 process/result seams, Session/Carrier/ACK separation, CarrierState/CarrierManager, FairScheduler/flow accounting, carrier adapters, observability, package/operator, dependency/build, and prior CLI machine/human contracts remain closed unless materially changed or falsified by a new concrete counterexample.
- `SessionRuntime.events` retained-history capacity remains a maintainer/security policy gate; do not invent TTL/LRU/history-size/capacity values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD, or Experimental Track merely for freshness.

## FRONT HIGH — H-I4-091 exact closure contract

Do **not** redo the already-correct post-release diagnostic ordering or the existing timeout regression unless the minimal repair needs a shared helper adjustment. Close only the remaining evidence-oracle/process-ownership holes:

1. Preserve H-I4-090's pre-churn baseline guard and existing Linux `/proc/<pid>/fd` + `/proc/<pid>/status` FD/RSS margins unchanged.
2. Add a **mutation-sensitive post-barrier state/token** established only after successful `malformed_classification_barrier(...)` completion, and assert that state immediately at the Linux post-resource snapshot site before sampling. Moving the snapshot site above successful barrier completion must deterministically fail (or fail to compile) even if the later log count still reaches `ATTEMPTS`.
3. Preserve `preauth.release(admission)` before the observable `malformed_or_unadmitted` event on the invalid-negotiation path. Keep the event diagnostic-only; do not promote it to authentication, Session delivery, Carrier readiness, packet ACK, wire, or release semantics.
4. Make **every unsuccessful barrier exit** restore bounded child/reader ownership before return. In particular, the stdout-EOF `Ok(None)` path must not rely on `Child` drop as a reap mechanism. Terminate if necessary, reap deterministically, and join/reconcile the reader without introducing an unbounded wait.
5. Add a focused early-EOF/insufficient-classification regression if needed so that the EOF cleanup branch is mechanically protected, not just source-reviewed. Keep the existing timeout/missing-event regression.
6. Keep non-Linux Unix portable lifecycle/socket semantics without claiming `/proc` resource evidence.
7. Do not invent new capacity/security/TTL/history values. The current test-local `sync_channel(64)` is not itself a release policy, but if the helper is touched prefer an event-derived or minimal handoff bound rather than proliferating unexplained constants.
8. No fuzz unless wire decoder/parser/crypto framing owners actually change.
9. Run focused regressions. Then on the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and verify a clean tree. Persist exact reachable SHA, UTC start/end, exit codes, OS/arch, Rust stable version, clean-tree state; do not record secrets/private addresses/topology/credentials/unnecessary absolute paths.
10. After closure, continue immediately into the rolling queue; do not wait for reviewer cadence.

## Rolling queue — keep continuous after the HIGH

Do not shrink this to one ticket. Proceed dependency-order until a real stop condition appears:

1. **H-I4-091 post-snapshot mutation guard + EOF/failure-path reap + exact-tree provenance.** Smallest test/helper repair only.
2. **I4-PORT-RES exact-current independent causal re-challenge.** Challenge both causal edges: `READY -> affirmative pre-churn snapshot -> first malformed send` and `all bounded malformed inputs classified + corresponding cleanup complete -> affirmative post-churn snapshot`. Explicitly attempt to move/split the oracle and prove the guards fail closed.
3. **Pre-auth malformed/rejection resource-accounting bounded challenge.** Inspect exact-current `admit_carrier`, `charge_input`, invalid-negotiation release, response-admission rejection, and pending-owner paths. Verify no rejection is promoted into authentication/delivery evidence and that diagnostic barrier support does not retain unbounded state. No-finding is valid if scope/exclusions/anchor are explicit.
4. **Cross-platform CLI/process-test semantics.** Re-challenge Linux-only `/proc` cfg boundaries, Unix process/signal assumptions, child reap/join behavior, warning cleanliness, and non-Linux Unix compile/contract semantics.
5. **CLI diagnostic/machine-output boundary.** Challenge `malformed_or_unadmitted` scope, JSON/human-output collision, diagnostic stability, and ensure it remains evidence-only rather than public protocol semantics.
6. **Algorithmic/resource boundedness reconciliation.** Challenge reader thread/channel lifetime, timeout/EOF/disconnect cleanup, queue/counter growth, and test-local bounds. Do not add capacity-pressure benchmarks or new policy numbers.
7. **Item-4 + release-packet factual reconciliation.** Reconcile the repaired resource oracle and subsequent bounded reviews with `docs/release-security-review-packet.md`, `docs/status.md`, `IMPLEMENTATION_PLAN.md`, current provenance, and review notes. Do not promote local resource testing into WAN/performance/security approval.
8. **Repository-wide 13-surface refill.** Re-apply the required broad inventory. Reuse prior dedicated reviews only for genuinely unchanged owners; materially changed owners require a fresh bounded challenge. Queue exhaustion may be re-declared only if broad inventory finds no concrete defect, no uncovered implemented core surface, no READY review-support, and no READY live question.
9. **Conditional live.** Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved real-network question under standing authorization. Otherwise keep `READY_LIVE: none`.

If several coherent slices complete in 10–30 minutes with stable quality, deepen the queue rather than waiting. Every 3–4 coherent slices or important repair cluster, perform one factual release/item-4 reconciliation instead of rewriting large docs after every small commit.

## Release item 3 / VPS boundary

Current classification remains `READY_LIVE: none`.

- IPv6 remains environment-blocked unless a real owned IPv6 endpoint/path appears.
- HY2 and repeated-warm-failover current lines remain frozen against same-class retry without a materially new hypothesis.
- Periodic/soak, package lifecycle, migration-back, endpoint/key migration, and already-answered bounded live questions are not repeated for freshness.
- Live PLPMTUD is not reopened without its required design/security gate or a genuinely new accepted path/instrumentation question.
- Standing authorization permits bounded self-owned client↔VPS ordinary TCP/UDP work when a real new question becomes READY; it does not authorize third-party targets, production changes, privileged/exotic-carrier work reserved by the authorization file, or pressure/adversarial-load conditions requiring maintainer choice.

## Review / repair and evidence contract retained

For every bounded lane: read exact-current owners/tests plus applicable spec/ADR/status claim; state the invariant; try to falsify it with source reasoning and focused deterministic tests. If a concrete defect exists and committed semantics determine the answer, apply the smallest repair, add positive/negative regression, commit/push, run exact-tree gates, persist provenance, and continue.

No-finding slices must name inspected owners, commands/tests actually run, exclusions, and exact reachable anchor. Do not modify code merely to manufacture activity. Hosted CI is additional cross-evidence, never a waiting condition.

## Stop / escalation conditions

Normal progress does not require administrator notification. Escalate only for an automatically undecidable BLOCKER/HIGH, core Session/Carrier/ACK/crypto/wire architecture decision, D019 or another real policy/value choice, destructive/canonical migration, action beyond standing authorization, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
