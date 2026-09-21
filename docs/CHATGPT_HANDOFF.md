# ChatGPT reviewer handoff — H-I4-090 reopened for missing ordering regression

## Current repository truth

- Synchronize to current `main` before doing work.
- The latest executable source/test owner remains `0dbb93145c4e1cc031ca30258b3c58a5c3155b51` (`test(cli): H-I4-090 resource baseline sampled pre-churn, not after`). Commits after `0dbb931` through the prior `e83d192` handoff are review/provenance/reconciliation documentation only.
- Exact `0dbb931` correctly restores the Linux malformed-UDP resource baseline to **after READY and before the first malformed datagram**, with the post sample after churn + the existing settle point. H-I4-089's Linux-only `/proc` truth boundary remains intact.
- Developer-local exact-tree provenance for `0dbb931` remains retained at `docs/notes/h-i4-090-provenance-0dbb931-20260921.md`: focused affected testing, `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, Linux x86_64, stable Rust, UTC timestamps and exit codes. This is developer-local evidence, not reviewer-local or hosted CI.
- **H-I4-090 is CLOSED** at `de223a5128223d3667961e772002dcc6d75c563c`: a `churn_phase` sentinel (`assert_eq!(churn_phase, 0)` before the baseline snapshot, `assert_eq!(churn_phase, 2)` after the malformed sends) makes moving the baseline into the post-churn window a hard failure, not a silent green. The pre/post Linux `/proc` snapshots remain affirmative on the true pre-churn and post-churn+settle points. Exact-tree provenance: `docs/notes/h-i4-090-provenance-de223a5-20260921.md` — `check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-21T13:50:34Z → 13:57:05Z, Linux x86_64, rustc 1.98.0.
- The prior repository-wide 13-surface refill decision at `4315a5c` and queue-exhaustion handoff at `e83d192` are superseded only as to queue state; their unchanged-surface inventory remains useful after the HIGH is closed.
- Candidate A (future/unsent Recovery ACK), Candidate B (mixed datagram drop reasons), H-I4-085..089, prior R9 process/result seams, Session/Carrier/ACK separation, CarrierState/CarrierManager, FairScheduler/flow accounting, carrier adapters, observability, package/operator, dependency/build, CLI machine/human contract and algorithmic boundedness remain closed unless a material owner change or a new concrete counterexample falsifies them.
- `SessionRuntime.events` retained-history capacity remains a maintainer/security policy gate. Do not invent a history-size/TTL/LRU/capacity value.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not rerun prior HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track merely for freshness.

## FRONT HIGH — H-I4-090 ordering-oracle closure

Read exact-current `crates/neko-cli/tests/probe.rs` plus `docs/reviews/h-i4-090-mutation-guard-20260921.md` before changing anything.

The production/test behavior is currently correct; the missing part is regression strength. Close the HIGH with the smallest test-local change that makes ordering mechanically observable:

1. preserve READY -> affirmative Linux baseline -> malformed sends -> settle -> affirmative Linux post sample;
2. add a deterministic **mutation-sensitive sequencing guard** so moving the baseline after malformed-send start fails even if FD/RSS do not change (a tiny test-local phase/sentinel tied to the send path is enough; do not build a framework/checker);
3. preserve the existing FD/RSS margins and `#[cfg(target_os = "linux")]` `/proc` boundary; add no new capacity/security values;
4. keep both Linux snapshots fail-closed/affirmative;
5. run focused affected tests, then on the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, verify clean tree, and persist exact developer-local provenance (SHA, commands, UTC start/end, exit codes, OS/arch, Rust stable, clean state);
6. no decoder/parser/crypto-framing change => no mechanical fuzz run.

Once that final source/test SHA is pushed and gated, **continue immediately**; do not wait for reviewer cadence.

## Rolling queue after H-I4-090

These are dependency-ordered and intentionally narrower than the old repository-wide queue because the prior broad refill found unchanged owners already covered. Do not manufacture extra lanes.

1. **H-I4-090 repair + exact-tree provenance** — READY_LOCAL, FRONT HIGH.
2. **I4-PORT-RES independent re-challenge** — inspect exact-current malformed-UDP resource pre/post oracle, Linux snapshot failure behavior, lifecycle cleanup, and the new ordering guard; if no defect, write one scope-precise no-finding note.
3. **PORT evidence reconciliation** — update/supersede the prior `1c8eacc`/`c3c1664`/`b6cffc4`/queue-exhaustion prose only where the new source/test owner changes the evidence anchor; do not rewrite unrelated closed history.
4. **Release-packet factual consistency challenge** — verify the packet does not overstate portability/resource evidence or independent-review closure after the new anchor.
5. **Repository-wide 13-surface refill** — repeat the broad inventory once after this material test/evidence owner change; unchanged core owners with still-valid dedicated review do not need ritual replay.
6. **Conditional live** — only if a genuinely new code/instrumentation/hypothesis/path condition creates a specific unresolved real-network question inside standing authorization. Otherwise remain `READY_LIVE: none`.

If the post-repair 13-surface refill again finds no uncovered implemented core owner, no concrete defect, no READY review-support and no READY live question, then repository-wide local queue exhaustion may be re-established. Item 4 being unchecked by itself is not permission to invent review churn, but a narrow no-finding sweep is not sufficient to claim exhaustion.

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

## Release item 3 / live boundary

Current classification remains `READY_LIVE: none`.

- IPv6 remains environment-blocked unless a real owned IPv6 endpoint/path becomes available.
- HY2 and repeated-warm-failover current lines remain frozen against same-class retry without a materially new hypothesis.
- Periodic/soak, package lifecycle, migration-back, endpoint/key migration and already answered bounded live questions are not repeated for freshness.
- Live PLPMTUD is not reopened without the separate required design/security gate or a new accepted path condition/instrumentation question.
- Standing authorization continues to permit bounded self-owned client↔VPS TCP/UDP work when a genuine new question becomes READY; it does not authorize third-party targets, production changes, privileged/exotic-carrier work reserved by the authorization file, or pressure/adversarial-load experiments requiring maintainer choices.

## Review / repair and evidence contract retained

For each future bounded lane: read exact-current source/tests + applicable spec/ADR/status claim; state the invariant; try to falsify it with source reasoning and focused deterministic tests. If a concrete defect exists and current semantics determine the answer, make the smallest repair, add positive/negative regression, commit/push, then on the final pushed developer SHA run in a safe clean checkout/worktree:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- `git diff --check`
- verify clean tree.

Persist exact reachable pushed SHA, commands, UTC start/end, exit codes, OS/arch, Rust stable version and clean-tree state; do not record secrets/private topology/credentials/unnecessary absolute paths. Hosted CI is extra evidence, never a waiting condition. Run the pinned decode fuzz toolchain only for decoder/parser/crypto-framing changes.

No-finding slices should produce a scope-precise bounded note naming inspected owners, commands/tests actually run, exclusions and exact reachable anchor; do not change code solely to create activity.

## Stop / escalation conditions

Normal review/repair progress does not require administrator notification. Escalate only for an automatically undecidable BLOCKER/HIGH, core Session/Carrier/ACK/crypto/wire architecture change, D019 or another real policy/value choice, destructive/canonical migration, action beyond standing authorization, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
