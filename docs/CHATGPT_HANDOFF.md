# ChatGPT reviewer handoff — repository-wide local queue exhaustion after PORT closure

## Current repository truth

- Synchronize to current `main` before doing work. The repository-wide refill decision immediately preceding this handoff is `4315a5cfbd89c362838844e5802f7c238a50869a` (`docs/notes/item4-refill-check-20260921.md`).
- The latest executable source/test owner remains `0dbb93145c4e1cc031ca30258b3c58a5c3155b51` (`test(cli): H-I4-090 resource baseline sampled pre-churn, not after`). Commits after `0dbb931` through `4315a5c` are review/provenance/handoff/reconciliation documentation only.
- **H-I4-090 is CLOSED** at `0dbb931`: the malformed-UDP Linux resource baseline is sampled after server readiness and before the first malformed datagram; the post sample follows churn plus the existing settle point; unavailable Linux `/proc` snapshots fail the resource assertion rather than becoming skip/pass.
- Developer-local exact-tree provenance for `0dbb931` is retained at `docs/notes/h-i4-090-provenance-0dbb931-20260921.md`: focused affected testing, `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, Linux x86_64, stable Rust, UTC timestamps and exit codes. This is developer-local evidence, not reviewer-local or hosted CI.
- A GitHub combined-status query for `0dbb931` returned no hosted statuses. Absence is neither success nor failure.
- H-I4-089 remains CLOSED. The follow-on I4-PORT resource, Linux `/proc` inventory, Unix/platform-boundary, process cleanup/signal, CLI machine/human owner-diff, boundedness owner-diff, release-packet consistency and PORT/item-4 reconciliation lanes have current bounded no-finding notes through `b6cffc44d20d9e1aa7341bae2f2f265b517f8845`.
- The required repository-wide 13-surface refill check at `4315a5c` found every implemented core surface either currently independently challenged or unchanged under still-valid dedicated coverage. No exact-current concrete defect or missing dedicated bounded core review was identified.
- Candidate A (future/unsent Recovery ACK), Candidate B (mixed datagram drop reasons), H-I4-085..090, prior R9 process/result seams, Session/Carrier/ACK separation, CarrierState/CarrierManager, FairScheduler/flow accounting, carrier adapters, observability, package/operator, dependency/build, portability, CLI machine/human contract and algorithmic boundedness remain closed unless a material owner change or new concrete counterexample falsifies them.
- `SessionRuntime.events` retained-history capacity remains a maintainer/security policy gate. Do not invent a history-size/TTL/LRU/capacity value.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not rerun prior HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track merely for freshness.

## Queue state

**Repository-wide dependency-ready local review/repair queue is exhausted at this exact inventory.** This is a repository-wide conclusion, not a narrow PORT result: the required 13-surface refill was performed after H-I4-089/090, and no uncovered implemented core surface, concrete defect, READY review-support lane, or READY live question remains.

Do **not** manufacture checker/schema/framework/docs churn to keep the agent busy. Do **not** replay unchanged closed review lanes merely because item 4 is still unchecked. Item 4's remaining release/security decision is not equivalent to an uncovered local implementation surface.

The coding/review agent should resume immediately, without waiting for reviewer cadence, when repository truth gains any real trigger such as:

1. a new developer-owned source/test/build/operator-script commit or PR;
2. a material change to a previously reviewed core owner or evidence claim;
3. a new concrete counterexample against a closed invariant;
4. a new instrumentation/code/configuration/path-condition hypothesis that creates a truthful `READY_LIVE` question inside standing authorization;
5. an environment change that actually unblocks an item-3 row (for example owned IPv6 availability), rather than an unchanged retry;
6. a maintainer decision on `SessionRuntime.events` retained-history capacity or another policy/value gate;
7. a genuine transition into RC/freeze/release decision work authorized by the maintainer.

When such a trigger appears, refill toward roughly 8–15 coherent dependency-ordered slices when repository truth supports that depth. A BLOCKER/HIGH goes first and must close before widening the affected error surface.

## Repository-wide refill inventory retained

1. `neko-reliable` UDP recovery: ACK ranges, future/unsent ACK, loss/retransmit, RTT/PTO, persistent congestion, Reno, fault simulation;
2. `neko-carrier` `CarrierState`: generation, validation, hysteresis, single-active, drain/fail/activate;
3. Concurrent Carrier Manager / health / migration-back;
4. FairScheduler / multi-stream / Session+stream flow-control accounting;
5. Carrier adapters: Memory/UDP/TCP close/error/resource semantics;
6. `SessionRuntime` lifecycle/resource/window/DeliveryAck accounting;
7. `neko-observe` projection/event/counter/high-water correctness;
8. package/reproducibility/operator scripts;
9. dependency/build: manifests/lock/features/build/native hooks/unsafe inheritance;
10. cross-platform CLI/process-test semantics;
11. CLI exit-code / JSON / human-output contract;
12. algorithmic resource boundedness, excluding capacity-pressure benchmark/new policy values;
13. release-packet factual consistency/evidence boundary.

A future queue-exhaustion claim must repeat this broad inventory if any relevant owner/evidence changed. Unchanged owners with current dedicated independent bounded coverage do not need ritual re-review; changed owners or falsified claims do.

## Release item 3 / live boundary

Current classification remains `READY_LIVE: none`.

- IPv6 remains environment-blocked unless a real owned IPv6 endpoint/path becomes available.
- HY2 and repeated-warm-failover current lines remain frozen against same-class retry without a materially new hypothesis.
- Periodic/soak, package lifecycle, migration-back, endpoint/key migration and already answered bounded live questions are not repeated for freshness.
- Live PLPMTUD is not reopened without the separate required design/security gate or a new accepted path condition/instrumentation question.
- Standing authorization continues to permit bounded self-owned client↔VPS TCP/UDP work when a genuine new question becomes READY; it does not authorize third-party targets, production changes, privileged/exotic-carrier work that the authorization file reserves, or pressure/adversarial-load experiments requiring maintainer choices.

## Review / repair and evidence contract retained

For each future bounded lane: read exact-current source/tests + applicable spec/ADR/status claim; state the invariant; try to falsify it with source reasoning and focused deterministic tests. If a concrete defect exists and current semantics determine the answer, make the smallest repair, add positive/negative regression, commit/push, then on the final pushed developer SHA run in a safe clean checkout/worktree:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- `git diff --check`
- verify clean tree.

Persist exact reachable pushed SHA, commands, UTC start/end, exit codes, OS/arch, Rust stable version and clean-tree state; do not record secrets/private topology/credentials/unnecessary absolute paths. Hosted CI is extra evidence, never a waiting condition. Run the pinned decode fuzz toolchain only for decoder/parser/crypto-framing changes.

No-finding slices should produce a scope-precise bounded note naming inspected owners, commands/tests actually run, exclusions and exact reachable anchor; do not change code solely to create activity.

## Stop / escalation conditions

Normal review/repair progress does not require administrator notification. Escalate only for an automatically undecidable BLOCKER/HIGH, core Session/Carrier/ACK/crypto/wire architecture change, D019 or another real policy/value choice, destructive/canonical migration, action beyond standing authorization, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
