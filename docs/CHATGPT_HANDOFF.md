# ChatGPT reviewer handoff — H-I4-090 still open: phase flips after churn

## Current repository truth

- Synchronize to current `main` before doing work; code/tests/reachable commits outrank older handoff prose.
- The latest executable source/test owner is `de223a5128223d3667961e772002dcc6d75c563c` (`test(cli): H-I4-090 mutation-sensitive ordering guard — phase sentinel`) in `crates/neko-cli/tests/probe.rs`.
- Exact `de223a5` preserves the intended committed measurement order: server READY -> affirmative Linux `/proc` baseline -> malformed UDP sends -> settle -> affirmative Linux post snapshot. H-I4-089's Linux-only `/proc` boundary remains intact.
- Developer-local exact-tree provenance for `de223a5` is retained at `docs/notes/h-i4-090-provenance-de223a5-20260921.md`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, Linux x86_64, rustc 1.98.0 stable, 2026-09-21T13:50:34Z -> 13:57:05Z. This is developer-reported local provenance, not reviewer-local execution or hosted CI.
- **H-I4-090 is CLOSED** at `6911193`: `churn_started` is a bool sentinel that flips to `true` **before** the first malformed send — `assert!(!churn_started)` before the baseline, `assert!(churn_started)` after the sends. A baseline moved to just after the senders collect (or after the send loop) fails the `!churn_started` assert, not a silent green. The pre/post Linux `/proc` snapshots remain affirmative on the true pre-churn and post-churn+settle points. Exact-tree provenance: `docs/notes/h-i4-090-provenance-6911193-20260921.md` — `check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-21T15:49:55Z → 15:55:50Z, Linux x86_64, rustc 1.98.0.
- The prior repository-wide 13-surface refill decision at `4315a5c` and queue-exhaustion handoff at `e83d192` are superseded as to queue state while this HIGH is open; their unchanged-surface inventory remains useful after closure.
- Candidate A (future/unsent Recovery ACK), Candidate B (mixed datagram drop reasons), H-I4-085..089, prior R9 process/result seams, Session/Carrier/ACK separation, CarrierState/CarrierManager, FairScheduler/flow accounting, carrier adapters, observability, package/operator, dependency/build, CLI machine/human contract and algorithmic boundedness remain closed unless a material owner change or a new concrete counterexample falsifies them.
- `SessionRuntime.events` retained-history capacity remains a maintainer/security policy gate. Do not invent a history-size/TTL/LRU/capacity value.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not rerun prior HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track merely for freshness.

## FRONT HIGH — H-I4-090 mutation guard is still after the churn

Read exact-current `crates/neko-cli/tests/probe.rs` plus `docs/reviews/h-i4-090-phase-sentinel-gap-20260921.md` before changing anything.

Concrete counterexample on exact `de223a5`:

1. `churn_phase` starts at `0`;
2. the baseline block asserts `phase == 0`;
3. all malformed `send_to` calls run while phase is still `0`;
4. only after the loop does code assign `churn_phase = 2`.

Therefore cut/pasting the baseline block to immediately after the malformed-send loop still passes its phase assertion even though the baseline is post-churn. The later `phase == 2` assertion does not prove baseline-before-first-send.

Close this HIGH with the smallest test-local repair:

1. preserve READY -> affirmative Linux baseline -> malformed sends -> settle -> affirmative Linux post sample;
2. make a sentinel transition mechanically observable **before the first malformed `send_to` can occur** (for example, enter a `churn_started` phase immediately before the send loop, or use an equally small helper whose first action changes phase before issuing any send); the baseline assertion must require the pre-churn phase;
3. ensure moving the baseline block to immediately after the send loop now fails deterministically even when FD/RSS values are unchanged;
4. preserve existing FD/RSS margins, Linux-only `/proc` gating and fail-closed snapshots; add no capacity/security values or framework/checker churn;
5. keep the non-Linux portable socket/lifecycle test path warning-clean when touching the phase variable; do not broaden `/proc` evidence to non-Linux Unix;
6. run focused affected tests, then on the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, verify clean tree, and persist exact developer-local provenance (SHA, commands, UTC start/end, exit codes, OS/arch, Rust stable, clean state);
7. no decoder/parser/crypto-framing change => no mechanical fuzz run.

Once the final source/test SHA is pushed and gated, **continue immediately**; do not wait for reviewer cadence.

## Rolling queue after H-I4-090

These lanes are dependency-ordered. The old broad refill already covered unchanged core owners, so keep this queue focused but do not collapse it to one ticket.

1. **H-I4-090 repair + exact-tree provenance** — READY_LOCAL, FRONT HIGH.
2. **I4-PORT-RES independent re-challenge** — inspect exact-current malformed-UDP resource pre/post oracle, Linux snapshot failure behavior, lifecycle cleanup, the repaired ordering guard, and non-Linux test compilation/warning semantics; if no defect, write one scope-precise no-finding note.
3. **PORT evidence reconciliation** — update/supersede prior `1c8eacc` / `c3c1664` / `b6cffc4` / queue-exhaustion prose only where the new source/test owner changes the evidence anchor; do not rewrite unrelated closed history.
4. **Release-packet factual-consistency challenge** — verify `docs/release-security-review-packet.md` does not overstate portability/resource evidence or independent-review closure after the new anchor.
5. **Repository-wide 13-surface refill** — repeat the broad inventory once after this material test/evidence owner change; unchanged core owners with still-valid dedicated review do not need ritual replay. A narrow no-finding sweep alone is not enough to re-declare exhaustion.
6. **Conditional live** — only if a genuinely new code/instrumentation/hypothesis/path condition creates a specific unresolved real-network question inside standing authorization. Otherwise remain `READY_LIVE: none`.

If the post-repair 13-surface refill again finds no uncovered implemented core owner, no concrete defect, no READY review-support and no READY live question, repository-wide local queue exhaustion may be re-established. Item 4 being unchecked by itself is not permission to invent review churn.

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
- Standing authorization continues to permit bounded self-owned client<->VPS TCP/UDP work when a genuine new question becomes READY; it does not authorize third-party targets, production changes, privileged/exotic-carrier work reserved by the authorization file, or pressure/adversarial-load experiments requiring maintainer choices.

## Review / repair and evidence contract retained

For each future bounded lane: read exact-current source/tests + applicable spec/ADR/status claim; state the invariant; try to falsify it with source reasoning and focused deterministic tests. If a concrete defect exists and current semantics determine the answer, make the smallest repair, add positive/negative regression, commit/push, then on the final pushed developer SHA run in a safe clean checkout/worktree:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- `git diff --check`
- verify clean tree.

Persist exact reachable pushed SHA, commands, UTC start/end, exit codes, OS/arch, Rust stable version and clean-tree state; do not record secrets/private topology/credentials/unnecessary absolute paths. Hosted CI is extra evidence, never a waiting condition. Run the pinned decode fuzz toolchain only for decoder/parser/crypto-framing changes.

No-finding slices should produce a scope-precise bounded note naming inspected owners, commands/tests actually run, exclusions and exact reachable anchor; do not change code solely to create activity.

## Stop / escalation conditions

Normal review/repair progress does not require administrator notification. Escalate only for an automatically undecidable BLOCKER/HIGH, core Session/Carrier/ACK/crypto/wire architecture change, D019 or another real policy/value choice, destructive/canonical migration, action beyond standing authorization, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
