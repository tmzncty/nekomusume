# ChatGPT reviewer handoff — H-I4-090 fronts PORT/resource evidence queue

## Current repository truth

- Synchronize to current `main`. Reviewer finding **H-I4-090** is reachable at `46bf499d0e937d3b7378e255f3b67dd36cb5012c` (`docs/reviews/reviewer-h-i4-090-resource-baseline-after-churn-20260921.md`). It supersedes the immediately previous PORT/resource closure only for the malformed-UDP resource-growth subclaim; unrelated closed surfaces remain closed unless their current owner changes or a new counterexample appears.
- The current executable/source-test tree is exact `26a2c40be8a524024f58c7b771aec8b924ae6524`. Commits after it through the H-I4-090 reviewer note are docs/review/handoff/provenance only. No decoder/parser/crypto-framing owner changed in this interval.
- **H-I4-089's portability correction remains valid and closed in its narrow sense** at `26a2c40`: `process_resource_snapshot` is Linux-only; Linux observation must be affirmative; non-Linux Unix retains the portable socket/lifecycle portion without fabricating `/proc` evidence. However, the same repair moved the resource `before` sample to after malformed churn, so the resource-growth oracle is not closed.
- Exact `26a2c40` developer-local provenance remains truthful execution provenance: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, UTC 2026-09-21T06:56:21Z → 07:02:25Z, Linux x86_64, rustc 1.98.0 stable. It is not reviewer-local or hosted CI and does not make an incorrectly ordered oracle valid evidence.
- **H-I4-087 remains CLOSED** at source repair `14e2f520a0fc9d41bcfcb1bd480758008a1dd4ea` with focused regressions through `d819d38559d60b6bd99df8a2453321809c046116`: `max_queue_records` is one aggregate send+recv ownership cap. Current bounded reviewer inspection found no new counterexample.
- **H-I4-088 remains CLOSED** at `26aa4e8036d61da7924d9fc5e587081d8a0f448f`: MemoryCarrier empty messages consume a finite record slot derived from the already committed queue-byte bound and dequeue releases it. Current bounded reviewer inspection found no new counterexample; do not invent a separate numeric record-cap policy.
- H-I4-086, I4-FS2, Candidate A/B, CarrierState/CarrierManager, unchanged observability, unchanged package/operator owners, H-I4-085 dependency/build truth, CLI-M/CLI-H, and closed R9 process/result seams remain closed unless their exact-current owner changes or a new concrete counterexample falsifies a claim.
- `SessionRuntime.events` retained-history capacity remains a maintainer/security policy gate. Do not choose a history-size/TTL/LRU/capacity value while repairing H-I4-090.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** H-I4-090 is entirely local test/evidence truth. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track merely because the VPS is still rented.

## FRONT — H-I4-090 repair contract

This HIGH is dependency-ready and automatically decidable from current committed semantics. It does **not** require maintainer policy input. Coding agent should repair, test, push, record exact-tree provenance and continue immediately into the queue below without waiting for the next reviewer cadence.

The concrete defect is ordering: before `26a2c40`, the Linux resource baseline was sampled after server READY and **before** the eight malformed UDP sends. Exact `26a2c40` correctly made `/proc` Linux-only and affirmative, but now takes both `before` and `after` snapshots back-to-back **after** the malformed sends and settle sleep. A persistent FD/RSS increase caused by the attack window is therefore already present in both samples and can false-pass.

1. Restore an affirmative Linux **pre-churn** `process_resource_snapshot(pid)` after server READY and before the first malformed datagram.
2. Preserve H-I4-089's `#[cfg(target_os = "linux")]` boundary. Do not re-generalize `/proc` measurement to macOS or generic Unix.
3. Take the post-churn Linux snapshot only after the malformed sends and the existing bounded settle point.
4. Both Linux snapshots must be affirmative; missing/failed observation must fail the resource claim rather than become `None -> pass`.
5. Compare the true pre/post pair using the already committed FD/RSS margins. Do **not** choose new resource/capacity/security policy values.
6. Add a small deterministic/mutation-sensitive regression or source-level guard that would fail if the baseline were moved behind the attack window again. A local helper/test seam is acceptable; do not build a generic checker/schema framework merely for this finding.
7. Preserve the non-Linux Unix portable socket/lifecycle portion of the fixture without fabricating `/proc` evidence.
8. Run focused affected CLI/process tests. Then, on the final pushed source/test SHA in a safe clean checkout/worktree, run:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - verify the worktree is clean.
   Record exact reachable pushed SHA, commands, UTC start/end, exit codes, OS/arch, Rust stable version and clean-tree state. Do not record secrets, private addresses/topology, credentials or unnecessary absolute paths.
9. No fuzz is required unless the repair unexpectedly changes decoder/parser/crypto framing. H-I4-090 should not do so.

## Rolling queue after the HIGH repair

The earlier broad item-4 sweep closed many unchanged owners, so there may be fewer real independent lanes than the steady-state 8–15 target. Do not manufacture filler, but do not call queue exhaustion from a narrow PORT seam. The following **9 coherent lanes** are dependency-ordered; lanes 2–8 may collapse only when exact-current facts make them genuinely redundant, and the repository-wide refill check must still occur before any exhaustion claim.

1. **H-I4-090 repair + focused tests + exact-tree local provenance** — FRONT HIGH, contract above.
2. **I4-PORT-RES independent pre/post-oracle challenge** — inspect the exact repaired `process_resource_snapshot` caller and challenge the invariant that the resource baseline is sampled before hostile input, the post sample after it, and unavailable Linux observation cannot be promoted to success. Try to falsify with focused deterministic/source-level mutation reasoning; concrete defect -> smallest repair before continuing.
3. **I4-PORT-LINUX `/proc` owner/callsite inventory** — bounded source sweep for `/proc` reads and related FD/RSS/process-resource helpers in current CLI/process tests. Look specifically for Linux-only interfaces hidden under broader `cfg(unix)` or optional-observation branches that can false-pass. Do not rewrite working portable `std::process` APIs.
4. **I4-PORT-UNIX/platform-boundary challenge** — verify Linux-only resource observation and Unix/POSIX lifecycle/signal semantics are truthfully separated. No macOS or non-Linux execution claim without actual evidence; preserve genuinely portable socket/lifecycle coverage.
5. **I4-PORT process cleanup/signal bounded challenge** — re-check exact-current signal/process-group/listener cleanup owners only where PORT/resource owner changes touch their evidence boundary. If prior independent closure remains exact-current and no new counterexample exists, record a scope-precise no-finding/unchanged-owner note rather than replaying unrelated tests.
6. **I4-PORT closure + release/item-4 factual reconciliation** — after lanes 2–5 are clean, update only stale PORT/resource evidence/index claims. Name exact current source/test anchor, inspected owners, commands/tests, exclusions and evidence class. Do not label developer self-review as reviewer-local evidence; preserve release items 3/4 incomplete and all four governance flags false.
7. **Release-packet factual-consistency challenge** — bounded check that the current packet/status/notes do not promote the repaired malformed-churn test into broader Unix, capacity, security-audit, production, or hosted-CI evidence. Repair prose only if a concrete stale/falsified claim exists.
8. **Repository-wide 13-surface owner-diff refill check** — compare exact-current owners with the prior broad inventory. Reopen a surface only for a material owner change, stale/falsified claim, concrete counterexample, or genuinely missing independent bounded challenge. If any real local lane exists, refill and continue; only after this broad inventory may repository-wide local technical queue exhaustion be asserted again.
9. **Conditional live** — currently `READY_LIVE: none`. Reopen only if new code/instrumentation/hypothesis/path condition creates a concrete unresolved self-owned TCP/UDP question inside standing authorization. Do not rerun old live rows for freshness.

## Repository-wide refill rules retained

For lane 8, check all 13 core surfaces rather than deriving exhaustion from PORT alone:

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

Candidate A (future/unsent Recovery ACK) and Candidate B (mixed datagram drop reasons) remain closed unless current owner truth changes or a new counterexample appears. A prior no-finding note is not immunity from a real new counterexample, but unchanged code alone is not a reason to replay every review.

## Evidence boundary

- H-I4-090 at `46bf499d` is exact-current source/diff/review-note reasoning. **No reviewer-local Rust test, `scripts/check.sh`, fuzz run, hosted CI, WAN run or performance run is claimed for the finding.**
- Exact `26a2c40` clean-gate provenance remains developer-reported local evidence only. GitHub combined-status and commit-workflow queries returned no hosted run for that SHA; absence is neither success nor failure.
- No live WAN result, benchmark/performance conclusion, macOS execution claim or capacity-limit conclusion is added by H-I4-090.
- No fuzz claim is made because decoder/parser/crypto framing is unchanged.
- The previous broad item-4 reviews remain useful for exact-current unchanged surfaces, but any repository-wide queue-exhausted conclusion that depended on PORT/resource closure is superseded until H-I4-090 and lanes 2–8 are reconciled.

## Stop / escalation conditions

H-I4-090 itself is **not** a policy/architecture stop. Repair and continue automatically. Escalate/notify only for a BLOCKER/HIGH that cannot be decided from current committed semantics, a core Session/Carrier/ACK/crypto/wire architecture change, D019 or another true policy/value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
