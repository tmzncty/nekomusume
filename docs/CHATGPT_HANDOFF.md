# ChatGPT reviewer handoff — H-I4-091 malformed-churn processing barrier HIGH

## Current repository truth

- Synchronize to current `main` before doing work; code/tests/reachable commits outrank older handoff prose and stale checkboxes.
- Latest executable source/test owner relevant to the current resource-evidence lane is `69111937075a158a87fa96a4d1a0f9d056f40d70` in `crates/neko-cli/tests/probe.rs`.
- **H-I4-090's pre-churn ordering repair remains accepted:** `churn_started` is false for the affirmative Linux baseline and flips true immediately before the first malformed `send_to`, so moving the baseline into the churn window fails the pre-churn assertion.
- **New FRONT HIGH: H-I4-091.** The current malformed-UDP resource-growth test still takes its Linux `after` snapshot after only `send_to` completion plus a fixed 100 ms sleep. That does not prove that the failover server has actually received/classified all eight malformed datagrams. The server's pre-auth path consumes at most one UDP datagram per outer loop iteration and currently exposes no per-malformed processing barrier to this test. A delayed/descheduled server can therefore process some/all malformed inputs only after the asserted post sample, allowing a false-negative resource-growth pass. Exact reviewer finding: `docs/reviews/reviewer-h-i4-091-malformed-churn-processing-barrier-20260922.md`.
- Developer-local exact-tree provenance for `6911193` remains retained at `docs/notes/h-i4-090-provenance-6911193-20260921.md`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, Linux x86_64, stable rustc 1.98.0, UTC 2026-09-21T15:49:55Z -> 15:55:50Z. This is valid developer-reported local provenance, not proof that the evidence oracle is causally complete and not reviewer-local/hosted CI.
- The prior `docs/reviews/reviewer-i4-port-res-post-6911193-20260921.md` no-finding conclusion is superseded only for the post-churn causal edge; its Linux/non-Linux `/proc` boundary analysis remains useful.
- The prior repository-wide queue-exhaustion conclusion in `docs/reviews/reviewer-post-6911193-release-refill-20260921.md` is temporarily invalidated by H-I4-091. Unchanged-owner coverage in that note remains reusable after this HIGH is repaired/re-reviewed.
- Candidate A (future/unsent Recovery ACK), Candidate B (mixed datagram drop reasons), H-I4-085..090, prior R9 process/result seams, Session/Carrier/ACK separation, CarrierState/CarrierManager, FairScheduler/flow accounting, carrier adapters, observability, package/operator, dependency/build and prior CLI machine/human contracts remain closed unless a material owner change or a new concrete counterexample falsifies them.
- `SessionRuntime.events` retained-history capacity remains a maintainer/security policy gate. Do not invent a history-size/TTL/LRU/capacity value.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** No new code/instrumentation/hypothesis/path condition currently creates a specific unresolved real-network question. Do not rerun HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track merely for freshness.

## FRONT HIGH — H-I4-091

The malformed-UDP resource oracle needs a deterministic **server-side processing barrier**, not a timing sleep.

Closure contract:

1. Keep H-I4-090's pre-churn baseline and mutation guard unchanged in meaning.
2. Prove, with a bounded fail-closed barrier, that all `ATTEMPTS` malformed datagrams used by this test were received and rejected/classified by the server before the Linux `after` snapshot. A diagnostic/test-only monotonically counted rejection event is acceptable; an equivalent explicit barrier is acceptable if it does not alter transport/delivery semantics.
3. `send_to` completion, fixed sleep, scheduler assumptions, or later authenticated-client success are **not** sufficient barriers.
4. Add a focused regression/oracle that fails if the post snapshot is moved before the processing barrier.
5. Preserve Linux-only `/proc/<pid>/fd` + `/proc/<pid>/status` evidence, fail closed on unavailable pre/post observations, and preserve the existing FD/RSS margins without changing their numbers.
6. Preserve the non-Linux Unix portable socket/lifecycle path without claiming `/proc` evidence.
7. Do not invent new capacity/security policy values. Do not redesign Session/Carrier/ACK/wire/crypto semantics.
8. No fuzz is required unless decoder/parser/crypto-framing owners are actually changed.
9. Final pushed source/test SHA must receive developer-local clean exact-tree provenance: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, exact SHA, UTC start/end, OS/arch, stable Rust version.
10. After repair, continue immediately to the queued independent slices below; do not wait for the next hourly reviewer.

## Rolling queue — keep continuous after the HIGH

Do not shrink this to one work ticket. Proceed dependency-order through every READY lane until a real stop condition appears:

1. **H-I4-091 repair + focused regressions + exact-tree provenance.**
2. **I4-PORT-RES exact-current independent re-challenge.** Explicitly challenge both causal edges: `READY -> affirmative pre-churn snapshot -> first malformed send`, and `all bounded malformed inputs processed/classified -> affirmative post-churn snapshot`.
3. **Pre-auth malformed/rejection resource-accounting bounded challenge.** If the repair changes `main.rs` diagnostics/admission ownership, inspect exact-current `admit_carrier` / `charge_input` / invalid-negotiation release path and prove the barrier itself does not retain unbounded state or promote rejection into authentication/delivery evidence. If no production owner changes, record a precise no-finding note rather than manufacturing code churn.
4. **Cross-platform CLI/process-test semantics.** Challenge Linux-only `/proc` cfg boundaries and ensure any new diagnostic/barrier test path remains warning-clean and portable on non-Linux Unix at compile/contract level.
5. **CLI diagnostic/machine-output boundary.** If new structured diagnostic output is introduced, challenge exact field/event stability and ensure ordinary human/JSON command contracts are unchanged; do not treat test diagnostics as public protocol evidence.
6. **Algorithmic/resource boundedness reconciliation.** Ensure any new counter/event/wait has an existing bounded derivation/deadline and does not add a capacity/TTL/history/security policy number.
7. **Item-4 + release-packet factual reconciliation.** Reconcile the repaired resource evidence with `docs/release-security-review-packet.md`, `docs/status.md`, implementation plan and current review notes. Do not promote local resource testing into WAN/performance/security approval.
8. **Repository-wide 13-surface refill.** Re-apply the required broad inventory. Reuse prior dedicated reviews only for genuinely unchanged owners; any materially changed owner gets a fresh bounded challenge. Queue exhaustion may be re-declared only if this broad inventory finds no concrete defect, no uncovered implemented core surface, no READY review-support and no READY live question.
9. **Conditional live.** Only if new code/instrumentation/hypothesis/path condition creates a concrete unresolved real-network question under standing authorization. Otherwise keep `READY_LIVE: none`.

If the agent completes several coherent slices quickly with low defect rate, preserve/deepen the queue rather than waiting for reviewer cadence. Every 3–4 coherent slices or important repair cluster, do one factual release/item-4 reconciliation instead of rewriting large docs after each small commit.

## Release item 3 / VPS boundary

Current classification remains `READY_LIVE: none`.

- IPv6 remains environment-blocked unless a real owned IPv6 endpoint/path becomes available.
- HY2 and repeated-warm-failover current lines remain frozen against same-class retry without a materially new hypothesis.
- Periodic/soak, package lifecycle, migration-back, endpoint/key migration and already answered bounded live questions are not repeated for freshness.
- Live PLPMTUD is not reopened without its required design/security gate or a genuinely new accepted path condition/instrumentation question.
- Standing authorization permits bounded self-owned client<->VPS ordinary TCP/UDP work when a real new question becomes READY; it does not authorize third-party targets, production changes, privileged/exotic-carrier work reserved by the authorization file, or pressure/adversarial-load conditions requiring maintainer choice.

## Review / repair and evidence contract retained

For each bounded lane: read exact-current source/tests + applicable spec/ADR/status claim; state the invariant; try to falsify it with source reasoning and focused deterministic tests. If a concrete defect exists and current committed semantics determine the answer, make the smallest repair, add positive/negative regression, commit/push, then on the final pushed developer SHA run in a safe clean checkout/worktree:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- `git diff --check`
- verify clean tree.

Persist exact reachable pushed SHA, commands, UTC start/end, exit codes, OS/arch, Rust stable version and clean-tree state; do not record secrets/private topology/credentials/unnecessary absolute paths. Hosted CI is extra evidence, never a waiting condition. Run the pinned decode fuzz toolchain only for decoder/parser/crypto-framing changes.

No-finding slices should produce a scope-precise bounded note naming inspected owners, commands/tests actually run, exclusions and exact reachable anchor; do not change code solely to create activity.

## Stop / escalation conditions

Normal progress does not require administrator notification. Escalate only for an automatically undecidable BLOCKER/HIGH, core Session/Carrier/ACK/crypto/wire architecture change, D019 or another real policy/value choice, destructive/canonical migration, action beyond standing authorization, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
