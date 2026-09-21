# ChatGPT reviewer handoff — post-`6911193` repository-wide queue exhaustion

## Current repository truth

- Synchronize to current `main` before doing work; code/tests/reachable commits outrank older handoff prose and stale checkboxes.
- Latest executable source/test owner relevant to the recent HIGH is `69111937075a158a87fa96a4d1a0f9d056f40d70` in `crates/neko-cli/tests/probe.rs`.
- **H-I4-090 is CLOSED at `6911193`.** `churn_started` remains false for the affirmative Linux pre-churn baseline, flips true immediately before the first malformed `send_to`, and stays true through the post-churn point. Moving the baseline into the churn/post-churn window therefore deterministically violates the pre-churn assertion even if FD/RSS values are unchanged.
- Developer-local exact-tree provenance for `6911193` is retained at `docs/notes/h-i4-090-provenance-6911193-20260921.md`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, Linux x86_64, stable rustc 1.98.0, UTC 2026-09-21T15:49:55Z -> 15:55:50Z. This is developer-reported local provenance, not reviewer-local execution or hosted CI.
- Reviewer current-owner re-challenge is `docs/reviews/reviewer-i4-port-res-post-6911193-20260921.md`: no finding; Linux `/proc` observation fails closed when incomplete, non-Linux Unix retains portable socket/lifecycle coverage, and no policy value was invented.
- Release-packet factual consistency + repository-wide refill are recorded in `docs/reviews/reviewer-post-6911193-release-refill-20260921.md`: no factual overclaim found and the 13 required surfaces are all covered by a dedicated current-owner bounded challenge or still-valid unchanged-owner review.
- Candidate A (future/unsent Recovery ACK), Candidate B (mixed datagram drop reasons), H-I4-085..090, prior R9 process/result seams, Session/Carrier/ACK separation, CarrierState/CarrierManager, FairScheduler/flow accounting, carrier adapters, observability, package/operator, dependency/build, CLI machine/human contracts and algorithmic boundedness remain closed unless a material owner change or a new concrete counterexample falsifies them.
- `SessionRuntime.events` retained-history capacity remains a maintainer/security policy gate. Do not invent a history-size/TTL/LRU/capacity value.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** No new code/instrumentation/hypothesis/path condition has created a new real-network question. Do not rerun HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track merely for freshness.

## Repository-wide queue state

**No dependency-ready local implementation/repair/review-support slice is currently identified. Queue exhaustion is repository-wide, not narrow-sweep.**

The post-`6911193` broad refill checked all required surfaces:

1. `neko-reliable` UDP recovery — covered; owner unchanged after dedicated ACK/loss/PTO/Reno/fault-simulation challenges.
2. `CarrierState` generation/validation/hysteresis/single-active/drain/fail/activate — covered; owner unchanged.
3. Concurrent Carrier Manager / health / migration-back — covered; owner unchanged.
4. FairScheduler / multi-stream / Session+stream flow-control accounting — covered; prior FS reviews plus H-I4-086/087 closure remain current.
5. Memory/UDP/TCP carrier adapters — covered; adapter reviews plus H-I4-088 closure remain current.
6. `SessionRuntime` lifecycle/resource/window/DeliveryAck — covered for dependency-ready engineering semantics; event-history capacity remains policy-blocked rather than READY_LOCAL.
7. `neko-observe` projection/event/counter/high-water correctness — covered; owner unchanged.
8. package/reproducibility/operator scripts — covered; owner unchanged.
9. dependency/build manifests/lock/features/build/native hooks/unsafe inheritance — covered by H-I4-085 closure; owner unchanged.
10. cross-platform CLI/process-test semantics — **re-challenged on exact current owner after `6911193`; no finding**.
11. CLI exit-code / JSON / human-output contract — covered; runtime owner unchanged.
12. algorithmic resource boundedness — covered; no new capacity-pressure benchmark or policy value is READY.
13. release-packet factual consistency/evidence boundary — **re-challenged post-`6911193`; no finding**.

The only material executable owner change since the prior refill was the test/evidence owner in `probe.rs`; no runtime/product owner, package script, manifest/lockfile, wire/crypto owner, or live-experiment implementation changed.

## What the coding agent should do now

Do **not** manufacture checker/schema/framework/docs churn solely to create activity. Do not keep replaying already-independent-reviewed unchanged owners.

Remain idle only while all of the following stay true:

- no new developer/source/test commit materially changes an owner;
- no new concrete correctness/security/evidence counterexample appears;
- no currently policy-blocked item receives a maintainer value decision;
- no blocked environment/path condition materially changes;
- no new release stage/authority decision is made;
- `READY_LIVE` remains none.

On the first real trigger, resume continuously without waiting for reviewer cadence:

1. synchronize to exact current `main`;
2. classify the changed owner/surface;
3. if correctness/security/evidence BLOCKER/HIGH exists, place it first and repair with the smallest semantics-preserving change + positive/negative regressions;
4. otherwise open only the bounded independent review lane actually invalidated by the material change;
5. run the required developer-local clean exact-tree gate on final pushed source/test SHA and persist provenance;
6. continue through every newly dependency-ready slice until a genuine stop condition or another repository-wide exhaustion decision.

## Release item 3 / live boundary

Current classification remains `READY_LIVE: none`.

- IPv6 remains environment-blocked unless a real owned IPv6 endpoint/path becomes available.
- HY2 and repeated-warm-failover current lines remain frozen against same-class retry without a materially new hypothesis.
- Periodic/soak, package lifecycle, migration-back, endpoint/key migration and already answered bounded live questions are not repeated for freshness.
- Live PLPMTUD is not reopened without its required design/security gate or a genuinely new accepted path condition/instrumentation question.
- Standing authorization still permits bounded self-owned client<->VPS TCP/UDP work when a real new question becomes READY; it does not authorize third-party targets, production changes, privileged/exotic-carrier work reserved by the authorization file, or pressure/adversarial-load conditions requiring maintainer choice.

## Review / repair and evidence contract retained

For each future bounded lane: read exact-current source/tests + applicable spec/ADR/status claim; state the invariant; try to falsify it with source reasoning and focused deterministic tests. If a concrete defect exists and current committed semantics determine the answer, make the smallest repair, add positive/negative regression, commit/push, then on the final pushed developer SHA run in a safe clean checkout/worktree:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- `git diff --check`
- verify clean tree.

Persist exact reachable pushed SHA, commands, UTC start/end, exit codes, OS/arch, Rust stable version and clean-tree state; do not record secrets/private topology/credentials/unnecessary absolute paths. Hosted CI is extra evidence, never a waiting condition. Run the pinned decode fuzz toolchain only for decoder/parser/crypto-framing changes.

No-finding slices should produce a scope-precise bounded note naming inspected owners, commands/tests actually run, exclusions and exact reachable anchor; do not change code solely to create activity.

## Stop / escalation conditions

Normal progress does not require administrator notification. Escalate only for an automatically undecidable BLOCKER/HIGH, core Session/Carrier/ACK/crypto/wire architecture change, D019 or another real policy/value choice, destructive/canonical migration, action beyond standing authorization, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
