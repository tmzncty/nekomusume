# ChatGPT reviewer handoff — PORT owner inventory after H-I4-090 closure

## Current repository truth

- Synchronize to current `main` before doing work. The latest independent bounded reviewer note is `1c8eacc256e5d27c56dbc971ba13b30926dca7c0` (`docs/reviews/reviewer-i4-port-res-post-0dbb931-20260921.md`).
- **H-I4-090 remains CLOSED** at source/test repair `0dbb93145c4e1cc031ca30258b3c58a5c3155b51`. Exact-current inspection confirms the malformed-UDP Linux resource baseline is sampled after server readiness and **before** the first malformed datagram; the post sample is after churn + the existing settle point; unavailable Linux snapshots fail the resource assertion rather than becoming a skip/pass.
- Developer-local exact-tree provenance for `0dbb931` is retained at `docs/notes/h-i4-090-provenance-0dbb931-20260921.md`: focused affected test, `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean worktree, Linux x86_64, stable Rust, UTC timestamps and exit codes. This is developer-local evidence, not reviewer-local or hosted CI.
- A current GitHub combined-status query for `0dbb931` returned no hosted statuses. Absence is neither success nor failure.
- The earlier H-I4-090 contract suggested a separate mutation-sensitive guard against re-moving the baseline. No separate checker/meta-test was added, but there is no exact-current counterexample: the executable test itself is ordered correctly and fail-closed. Treat extra mutation hardening as optional unless a concrete current defect appears; do not manufacture checker/framework churn merely to satisfy a stale suggestion.
- **H-I4-089 remains CLOSED**: `/proc/<pid>` FD/RSS resource observation is Linux-only and affirmative on Linux; non-Linux Unix does not get fabricated `/proc` resource evidence.
- H-I4-087, H-I4-088, H-I4-086, Candidate A/B, CarrierState/CarrierManager, FairScheduler/flow accounting, unchanged carrier adapters, observability, package/operator, dependency/build, CLI-M/CLI-H, and prior R9 process/result seams remain closed unless an exact-current owner materially changed or a new concrete counterexample falsifies the claim.
- `SessionRuntime.events` retained-history capacity remains a maintainer/security policy gate. Do not invent a history-size/TTL/LRU/capacity value.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not rerun prior HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track for freshness.

## Completed this reviewer slice — I4-PORT-RES

Independent exact-current source/diff challenge at `1c8eacc` found no defect in the repaired pre/post resource oracle. Reviewed owners and evidence boundaries are recorded in `docs/reviews/reviewer-i4-port-res-post-0dbb931-20260921.md`. No reviewer-local Rust test, full gate, fuzz, WAN, macOS/non-Linux execution, or performance run is claimed.

Do **not** keep H-I4-090 at the front of the queue. The previous handoff contained both `CLOSED` and stale `FRONT HIGH` text; that stale repair block is superseded by this handoff.

## Rolling queue — continue without waiting for reviewer cadence

The coding/review agent must continue through dependency-ready slices while repository truth contains work. A no-finding bounded independent challenge is valid item-4 support; a concrete defect immediately becomes the smallest repair + positive/negative regression + pushed exact-tree gate/provenance before continuing.

1. **I4-PORT-LINUX `/proc` owner/callsite inventory — READY_LOCAL.** Inspect exact-current Rust tests plus Linux process-resource scripts for every `/proc`, `/proc/net`, FD/RSS/process-group helper and caller. Challenge hidden broad `cfg(unix)`, optional-observation false-pass, partial-table promotion, or unsupported-platform invocation. `scripts/bench/process-resource-sampler.py` is explicitly Linux-specific; distinguish that fact from portable CLI behavior and from Ubuntu-only hosted CI. Do not rewrite working portable `std::process` APIs.
2. **I4-PORT-UNIX/platform-boundary challenge — READY_LOCAL after lane 1.** Verify Linux-only measurement, POSIX signal/process semantics, and genuinely portable socket/lifecycle behavior are not conflated. No macOS/non-Linux execution claim without actual evidence.
3. **I4-PORT process cleanup/signal bounded challenge — READY_LOCAL.** Re-check exact-current child/process-group/listener cleanup and assertion/error paths touched by the PORT/resource evidence surface. Look for orphan/listener leakage, success before cleanup truth, or unbounded waits. Prior H-R9-082..084 seams stay closed unless a new exact counterexample exists.
4. **Cross-platform CLI/process-test semantics owner-diff — READY_LOCAL.** Compare exact-current owners against the prior portability review and challenge only material changes or unreviewed platform seams. Keep first-RC/release policy facts distinct from portability claims.
5. **CLI machine-contract owner-diff — READY_LOCAL.** Re-check exit-code/JSON/stderr contract only where exact-current owners changed or prior evidence is stale; do not replay unchanged closed rows for activity.
6. **CLI human-output owner-diff — READY_LOCAL.** Re-check human output/prose reachability contract only for changed owners or stale claims; preserve the already reconciled `pass:`/`fail:` semantics unless falsified.
7. **Algorithmic/resource boundedness owner-diff — READY_LOCAL.** Challenge current resource ownership/termination surfaces without capacity-pressure benchmarking and without choosing new numeric policy. `SessionRuntime.events` remains policy-gated, not a reason to stop unrelated review.
8. **Release-packet factual-consistency challenge — READY_LOCAL.** Ensure current status/packet/review notes do not promote Linux-only resource observations into broader Unix, security-audit, capacity, hosted-CI, production, or release evidence. Repair prose only for concrete stale/falsified claims.
9. **PORT + item-4 factual reconciliation — READY_LOCAL.** After lanes 1–8, update only actually stale indexes/status claims. Preserve exact evidence classes and release items/flags.
10. **Repository-wide 13-surface refill check — REQUIRED before exhaustion.** Re-inventory all 13 core surfaces below. Reopen only for material owner change, missing dedicated independent bounded challenge, stale/falsified claim, or concrete counterexample. If real work exists, refill toward ~8–15 coherent slices and continue. A narrow clean PORT sweep is never queue exhaustion.
11. **Conditional live.** Currently `READY_LIVE: none`. Reopen only if new code/instrumentation/hypothesis/path condition creates a concrete unresolved self-owned TCP/UDP question inside standing authorization.

## Repository-wide refill inventory

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

Candidate A (future/unsent Recovery ACK) and Candidate B (mixed datagram drop reasons) remain closed unless current owner truth changes or a new counterexample appears. A prior no-finding note is not immunity from a new counterexample, but unchanged code alone is not a reason to replay every review.

## Review / repair and evidence contract retained

For each bounded lane: read exact-current source/tests + applicable spec/ADR/status claim; state the invariant; try to falsify with source reasoning and focused deterministic tests. If a concrete defect exists and current semantics determine the answer, make the smallest repair, add positive/negative regression, commit/push, and on the final pushed developer SHA run in a safe clean checkout/worktree:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- `git diff --check`
- verify clean tree.

Persist exact reachable pushed SHA, commands, UTC start/end, exit codes, OS/arch, Rust stable version and clean-tree state; do not record secrets/private topology/credentials/unnecessary absolute paths. Hosted CI is extra evidence, never a waiting condition. Run pinned decode fuzz only for decoder/parser/crypto-framing changes; none of the currently queued PORT lanes imply fuzz by themselves.

No-finding slices should produce a scope-precise independent bounded note naming inspected owners, tests/commands actually run, exclusions and exact reachable anchor; do not change code just to create churn.

## Stop / escalation conditions

Normal review/repair progress does not require administrator notification. Escalate only for an automatically undecidable BLOCKER/HIGH, core Session/Carrier/ACK/crypto/wire architecture change, D019 or another real policy/value choice, destructive/canonical migration, action beyond standing authorization, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.