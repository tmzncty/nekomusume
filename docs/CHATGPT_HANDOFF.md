# ChatGPT reviewer handoff — H-I4-091 regression/provenance closure HIGH

## Current repository truth

- Synchronize to current `main` before work. Code/tests/reachable pushed commits outrank this prose and older notes.
- Reviewer observed developer `main = 53e502ecbc38c349f61653cdd1a46739da0bbdfd`, reviewed that exact implementation/test tree, then committed the continuation review note at `2afe7ceda5ed7dcd2e8cc6253fd9fe76f2056cae`. Re-read `main` immediately because the coding agent may advance it without waiting for reviewer cadence.
- New developer-owned commit reviewed since the prior handoff:
  - `53e502ecbc38c349f61653cdd1a46739da0bbdfd` — **implementation/test**: moves `malformed_or_unadmitted` after `preauth.release(admission)` and replaces the direct blocking stdout barrier with an off-thread reader + bounded `recv_timeout`, killing/reaping the child on timeout.
- **Accepted source-level repair:** the two concrete source defects from the prior H-I4-091 continuation are fixed in exact `53e502e`: the test-thread barrier no longer blocks directly in `read_line`, and the observable malformed classification is post-release.
- **H-I4-091 nevertheless remains FRONT HIGH for evidence-oracle closure.** Exact `53e502e` does not add the two required mutation-sensitive regressions: (a) a focused insufficient/missing-classification case proving bounded failure rather than hang, and (b) an ordering oracle that deterministically fails if the Linux post-resource snapshot is moved before the post-cleanup barrier. Exact review note: `docs/reviews/reviewer-h-i4-091-regression-closure-20260922.md` at `2afe7ceda5ed7dcd2e8cc6253fd9fe76f2056cae`.
- **Exact-tree provenance is also still open.** At review time there was no accepted developer-local clean exact-tree gate for final H-I4-091 source/test SHA. `53e502e` had no visible combined status or PR-triggered workflow run; absence of hosted evidence is not interpreted as failure.
- **H-I4-090 remains closed:** affirmative Linux baseline is before `churn_started = true`, which flips before the first malformed `send_to`.
- Candidate A (future/unsent Recovery ACK), Candidate B (mixed datagram drop reasons), H-I4-085..090, prior R9 process/result seams, Session/Carrier/ACK separation, CarrierState/CarrierManager, FairScheduler/flow accounting, carrier adapters, observability, package/operator, dependency/build, and prior CLI machine/human contracts remain closed unless materially changed or falsified by a new concrete counterexample.
- `SessionRuntime.events` retained-history capacity remains a maintainer/security policy gate; do not invent TTL/LRU/history-size/capacity values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD, or Experimental Track merely for freshness.

## FRONT HIGH — H-I4-091 remaining closure contract

Do **not** redo the already-correct `53e502e` source repair unless a new concrete defect is found. Close the evidence-oracle contract with the smallest repository-consistent test shape:

1. Keep H-I4-090's pre-churn baseline/mutation guard unchanged in meaning.
2. Preserve the current off-thread reader + `recv_timeout` bounded wait and timeout kill/reap semantics.
3. Add a focused negative oracle that deliberately supplies an insufficient/missing classification sequence and proves the barrier exits within its bounded deadline rather than hanging. Prefer a small test helper/seam around the barrier; do not add a protocol/runtime feature.
4. Preserve `preauth.release(admission)` before the observable `malformed_or_unadmitted` event.
5. Add a mutation-sensitive ordering oracle so the Linux post snapshot is permitted only after the `ATTEMPTS`th post-release classification. Moving the post snapshot before that barrier must deterministically fail.
6. Preserve Linux-only `/proc/<pid>/fd` + `/proc/<pid>/status` evidence, fail closed on unavailable pre/post observations, and preserve the existing FD/RSS margins unchanged.
7. Preserve the non-Linux Unix portable lifecycle/socket path without claiming `/proc` evidence.
8. Keep `malformed_or_unadmitted` diagnostic-only; do not promote it into authentication, Session delivery, Carrier readiness, packet ACK, wire, or release semantics.
9. Do not invent new capacity/security/TTL/history policy values. If refactoring the reader/channel helper, prefer a bound derived from the already-needed event count or the repository's established one-item handoff pattern rather than adding an unexplained policy-like capacity.
10. No fuzz unless wire decoder/parser/crypto framing owners actually change.
11. Run the focused regression(s). Then on the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and verify clean tree. Persist exact reachable SHA, UTC start/end, exit codes, OS/arch, Rust stable version, clean-tree state; do not record secrets/private addresses/topology/credentials/unnecessary absolute paths.
12. After closure, continue immediately into the rolling queue; do not wait for reviewer cadence.

## Rolling queue — keep continuous after the HIGH

Do not shrink this to one ticket. Proceed dependency-order until a real stop condition appears:

1. **H-I4-091 negative bounded-failure regression + post-cleanup ordering oracle + exact-tree provenance.**
2. **I4-PORT-RES exact-current independent re-challenge.** Explicitly challenge both causal edges: `READY -> affirmative pre-churn snapshot -> first malformed send`, and `all bounded malformed inputs classified + cleanup-complete -> affirmative post-churn snapshot`.
3. **Pre-auth malformed/rejection resource-accounting bounded challenge.** Inspect exact-current `admit_carrier` / `charge_input` / invalid-negotiation release path and prove the barrier itself does not retain unbounded state or promote rejection into authentication/delivery evidence. If no defect exists, write a scope-precise no-finding note rather than manufacturing code churn.
4. **Cross-platform CLI/process-test semantics.** Challenge Linux-only `/proc` cfg boundaries; ensure the new barrier path is warning-clean and portable on non-Linux Unix at compile/contract level.
5. **CLI diagnostic/machine-output boundary.** Challenge `malformed_or_unadmitted`: exact scope, field/event stability, collision with normal JSON/human output, and guarantee diagnostic-only evidence is not promoted into public protocol semantics.
6. **Algorithmic/resource boundedness reconciliation.** Challenge reader-thread/channel/wait/counter lifetime and derivation, including timeout/EOF/disconnect cleanup and the test-local channel bound; do not add a new project capacity policy.
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
