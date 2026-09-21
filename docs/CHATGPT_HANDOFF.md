# ChatGPT reviewer handoff — H-I4-091 bounded post-cleanup barrier HIGH

## Current repository truth

- Synchronize to current `main` before work. Code/tests/reachable pushed commits outrank this prose and older notes.
- Reviewer observed `main = bb3d79625fcac568ea390aa531c8ed7df7b10961` before the review-note commit. The new reviewer finding is reachable at `94c858052498c4f5200c01b9f9cd997631a336f9`; re-read `main` because the coding agent may advance it immediately.
- New developer-owned commits reviewed since the prior handoff:
  - `9b6d90a145b8c13534591336345d2b37b2e13c25` — **implementation/test**: adds `malformed_or_unadmitted` server diagnostic and replaces the fixed 100 ms sleep with an `ATTEMPTS`-count stdout barrier.
  - `91bf170991e3f5b7f480f25b9f47047792e059d8` — **test formatting only**.
  - `bb3d79625fcac568ea390aa531c8ed7df7b10961` — **checker/inventory maintenance** for the changed pre-auth producer anchor.
- **H-I4-090 remains closed:** affirmative Linux baseline is before `churn_started = true`, which flips before the first malformed `send_to`.
- **H-I4-091 remains FRONT HIGH.** The new diagnostic-count idea is directionally correct, but exact-current test code is not a bounded fail-closed barrier: it checks a 5 s deadline and then calls blocking `BufRead::read_line` on child stdout. If the next diagnostic never arrives, that call can block beyond the deadline indefinitely. The same test file's `ready_failover_server` already documents this exact hazard and solves it with an off-thread reader plus `recv_timeout` and child kill/reap on timeout.
- A second causal defect remains in exact-current `main.rs`: `malformed_or_unadmitted` is emitted **before** `preauth.release(admission)`. The test snapshots `/proc` immediately after observing the `ATTEMPTS`th diagnostic, so the resource oracle can race ahead of the final admission cleanup. Exact reviewer finding: `docs/reviews/reviewer-h-i4-091-bounded-barrier-continuation-20260922.md`.
- No reviewer-local Rust/full-gate/fuzz/WAN/performance execution is claimed for this finding. Latest queried `bb3d796` had no visible hosted status/workflow evidence; absence of hosted evidence is not interpreted as failure.
- Existing developer-local provenance for older `6911193` remains valid for that exact tree only. No final H-I4-091 source/test exact-tree provenance has yet been accepted.
- Candidate A (future/unsent Recovery ACK), Candidate B (mixed datagram drop reasons), H-I4-085..090, prior R9 process/result seams, Session/Carrier/ACK separation, CarrierState/CarrierManager, FairScheduler/flow accounting, carrier adapters, observability, package/operator, dependency/build, and prior CLI machine/human contracts remain closed unless materially changed or falsified by a new concrete counterexample.
- `SessionRuntime.events` retained-history capacity remains a maintainer/security policy gate; do not invent TTL/LRU/history-size/capacity values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD, or Experimental Track merely for freshness.

## FRONT HIGH — H-I4-091 closure contract

Repair the current barrier with the smallest repository-consistent shape:

1. Keep H-I4-090's pre-churn baseline/mutation guard unchanged in meaning.
2. Make the classification wait **actually bounded and fail closed**. A deadline check before blocking `read_line` is insufficient. Prefer the already-established off-thread stdout reader + bounded channel wait pattern (or an equivalent interruptible mechanism). On timeout, terminate/reap the test child so no reader/thread can remain stranded.
3. Add a focused negative oracle proving that an insufficient/missing classification sequence exits within the bounded deadline rather than hanging.
4. Move the observable barrier point to **after the malformed admission cleanup**. The `ATTEMPTS`th observable event must not become visible before the corresponding `preauth.release(admission)` cleanup point. An equivalent explicit post-cleanup event is acceptable. Do not promote the diagnostic into authentication, Session delivery, Carrier readiness, or packet-feedback evidence.
5. Add an ordering oracle that fails if the Linux post snapshot is moved before the post-cleanup barrier.
6. Preserve Linux-only `/proc/<pid>/fd` + `/proc/<pid>/status` evidence, fail closed on unavailable pre/post observations, and preserve the existing FD/RSS margins unchanged.
7. Preserve the non-Linux Unix portable lifecycle/socket path without claiming `/proc` evidence.
8. Do not invent new capacity/security policy values and do not redesign Session/Carrier/ACK/wire/crypto semantics.
9. No fuzz unless wire decoder/parser/crypto framing owners actually change.
10. On the final pushed source/test SHA, run focused regression(s), then `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and verify clean tree. Persist exact reachable SHA, UTC start/end, exit codes, OS/arch, Rust stable version, clean-tree state.
11. After closure, continue immediately into the rolling queue; do not wait for reviewer cadence.

## Rolling queue — keep continuous after the HIGH

Do not shrink this to one ticket. Proceed dependency-order until a real stop condition appears:

1. **H-I4-091 repair + negative/ordering regressions + exact-tree provenance.**
2. **I4-PORT-RES exact-current independent re-challenge.** Explicitly challenge both causal edges: `READY -> affirmative pre-churn snapshot -> first malformed send`, and `all bounded malformed inputs classified + cleanup-complete -> affirmative post-churn snapshot`.
3. **Pre-auth malformed/rejection resource-accounting bounded challenge.** Inspect exact-current `admit_carrier` / `charge_input` / invalid-negotiation release path and prove the barrier itself does not retain unbounded state or promote rejection into authentication/delivery evidence. If no further defect exists, write a scope-precise no-finding note rather than manufacturing code churn.
4. **Cross-platform CLI/process-test semantics.** Challenge Linux-only `/proc` cfg boundaries; ensure the new barrier path is warning-clean and portable on non-Linux Unix at compile/contract level.
5. **CLI diagnostic/machine-output boundary.** Challenge the new `malformed_or_unadmitted` diagnostic: exact scope, field/event stability, collision with normal JSON/human output, and guarantee that diagnostic-only evidence is not promoted into public protocol semantics.
6. **Algorithmic/resource boundedness reconciliation.** Ensure the reader thread/channel/wait/counter has a bounded derivation and cleanup path and adds no new capacity/TTL/history/security policy number.
7. **Item-4 + release-packet factual reconciliation.** Reconcile repaired resource evidence with `docs/release-security-review-packet.md`, `docs/status.md`, `IMPLEMENTATION_PLAN.md`, and current review notes. Do not promote local resource testing into WAN/performance/security approval.
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
