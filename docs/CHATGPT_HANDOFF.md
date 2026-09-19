# ChatGPT reviewer handoff — sampler determinism closed at e021b27; R9-11 is front

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `e021b27cffe4a87de185757a8d47b8685f462bb6` (`fix(bench): sampler positive-fixture real observation handshake`).
- Latest developer bounded review support reviewed: exact `15c0d9f1f15fcbc2f40039d7622bfdce9a33fb26`, `docs/reviews/dev-r9-10c-replay-cleanup-20260919.md`.
- Latest independent reviewer support: exact `d56ed28fe65127698b0fe850c0037e7eeccef621`, `docs/reviews/independent-process-resource-sampler-observation-handshake-e021b27-20260920.md`.
- **Sampler code-level HIGH finding CLOSED:** the previous `5d860dc` timing-only fixture was not deterministic. Exact `e021b27` repairs the seam: the child owns five `/dev/null` FDs plus the loopback listener before readiness, then waits for `sampler.release`; the sampler writes that marker only after real `/proc` observation reaches the requested FD minimum and observes an owned socket. The independent bounded review found no code defect in that repaired synchronization seam.
- **First-class closure evidence recorded for exact `e021b27`:** developer-local clean exact-tree gate `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-19T17:49:54Z → 2026-09-19T17:54:54Z, Linux x86_64, rustc 1.98.0. This remains developer-reported local evidence; reviewer-local execution is not claimed.
- GitHub-hosted Rust CI push run `35459239845` completed `success` — supplemental cross-evidence only.
- The earlier docs-only closure `df02bd0da7356fdb5708148d5d1e714b33e429cb` remains superseded. The source/test synchronization repair is now accepted and its required exact-tree developer-local provenance is recorded.
- **R9-10B remains closed** at source/test exact `70e87f086b0bbd2d13cd9bae073cb1cdb4c5abd4`: automatic-health TCP replay constructs each replay record from the authoritative retained `(DataId, bytes)` returned by `FailoverController::tcp_resend()` rather than an equal-count positional slice.
- **R9-10C remains independently bounded no-finding closed** at reviewer exact `a1de16f5568951fdc5e99846436f918ed8d99ed2` over source/test exact `70e87f086b0bbd2d13cd9bae073cb1cdb4c5abd4`. Full lifecycle/resource terminalization remains R9-11.
- Repository truth for duplicate semantics: a repeated `FailoverController::confirm` is typed `NotFound`; exact duplicate **receive** is the idempotent `Ok(false)` case.
- **H-R9-075 remains closed.** `SessionRuntime::delivery_ack` rejects a forward gap before watermark/in-flight/event mutation, and the strengthened negative proves atomic rejection.
- **H-R9-076 remains closed.** The resumed-session fixture applies the already-delivered offset-0 Session proof before later resumed proofs and confirms retained identity only after successful proof application.
- R9-7 remains independently bounded no-finding closed at `cdac663e4d2649c8f87fc2263766616dc9b4bbe9` over source/test exact `d97a536`.
- R9-8 warm readiness remains independently bounded no-finding closed at `7c87ac675a381154ae7fceca987e7f2f106c26cf`.
- R9-9 health/promotion/switch ordering remains independently bounded no-finding closed at `b4a527a1fff994ee0836893f850c36eadb609eb1`.
- Candidate A (`Recovery::on_ack` future/unsent `largest`) remains closed by the current fail-closed guard before RTT/loss/PTO mutation and dedicated regressions.
- Candidate B (`record_datagrams` mixed queue/terminal drop projection) remains closed by current mixed-delta regressions and separate reason projection.
- `READY_LIVE: none`. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD, or Experimental Track without a new code/instrumentation/hypothesis/path condition that creates a concrete unresolved real-network question.
- Release item 3 and item 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- No reviewer-local test execution is claimed for this pass. GitHub-hosted CI is supplemental; the accepted first-class gate remains a developer-local clean exact-tree gate on the final pushed source/test SHA.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency and is never a ticket length or reason to idle.

## READY_LOCAL 1 — FRONT HIGH: persist exact-tree local provenance for sampler repair

Independent review: `d56ed28fe65127698b0fe850c0037e7eeccef621`, `docs/reviews/independent-process-resource-sampler-observation-handshake-e021b27-20260920.md`.

Do **not** re-implement the sampler handshake unless the gate finds a real regression. The code-level synchronization finding is accepted at exact `e021b27`; the remaining blocker is evidence truth.

On a safe clean checkout/worktree of the exact final pushed source/test SHA containing the `e021b27` repair, run and persist:

`PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`

`git diff --check`

and record:

- exact GitHub-resolvable pushed SHA;
- UTC start/end;
- exit code of each command;
- clean-tree state after the gate;
- OS/arch;
- stable Rust version.

If the source/test tree is still exact `e021b27`, provenance must name that SHA. If a later developer source/test commit supersedes it before the gate, first re-review the relevant diff and gate the new final source/test SHA. No decoder/parser/crypto framing change is present in `e021b27`, so fuzz is not mechanically required.

Once the developer-local exact-tree gate/provenance is persisted and green, close this evidence HIGH and proceed immediately to R9-11 without waiting for reviewer cadence.

## READY_LOCAL 2 — R9-11 lifecycle / terminal resource closure

Bounded independent challenge of the full failover/migration lifecycle after R9-10. Read exact-current owners and challenge **live ownership release plus terminal mutator behavior**, not merely event labels.

Owners/surfaces:

- executable automatic-health failover and migration-back process path;
- `SessionRuntime` close/cancel/idle/close-deadline cleanup and post-terminal mutators;
- `PathRecovery` / `ReliableUdpRuntime` live sent/retransmit/Reno/ACK ownership and `quiesce` behavior;
- `FailoverController` retained replay ownership and migration-back state;
- `ConcurrentCarrierManager` active/draining/failed generations, retained ranges and replay ownership;
- TCP/UDP socket/process cleanup fixtures and existing lifecycle regressions.

Required invariants:

1. success, explicit failure, timeout, shutdown and post-return terminal paths release live socket/runtime/recovery/replay ownership they no longer own;
2. Session terminal states fail closed on later state-mutating APIs while approved lifetime diagnostics/history may remain;
3. a failed/retired **path generation** cannot be reactivated or receive new ownership without a fresh legal generation/transition;
4. quiescing Recovery may preserve lifetime diagnostic facts, but no stale in-flight/charged/retransmit ownership may survive as live state;
5. unresolved retained Session replay ownership must not be silently erased merely because a Carrier/path terminates;
6. resource cleanup evidence must not be inferred from process success alone — use existing deterministic socket/process or owner-state oracles;
7. do not invent a new global terminal API or lifecycle architecture unless a concrete invariant requires it; distinguish per-path terminality, Session terminality and process teardown.

If a concrete defect is found and current committed semantics decide the answer, smallest repair -> positive/negative regression -> exact-tree gate -> provenance -> continue. If no defect is found, commit a scope-precise independent bounded no-finding note with inspected owners/tests/exclusions and continue immediately.

## READY_LOCAL 3 — R9-12 final exact-tree provenance

After the coherent R9 repair/review group is stable, run and persist developer-local provenance for the final pushed developer source/test SHA:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- `git diff --check`
- clean tree
- exact GitHub-resolvable pushed SHA, UTC start/end, exit codes, OS/arch and stable Rust version

Run pinned decoder fuzz only if the coherent group actually touched wire decoder/parser/crypto framing; otherwise do not mechanically repeat it. Hosted CI remains supplemental cross-evidence.

## READY_LOCAL 4 — dedicated final independent R9 bounded review

On the final R9 exact source/test tree, independently re-challenge the combined invariants across Recovery, Session logical proof, Carrier readiness/promotion, uncertain retention, TCP replay/dedup, migration-back, process result truth and terminal cleanup. Include the repaired sampler only as evidence-reliability support, not transport semantics. A no-finding bounded review is valid item-4 support. If a concrete defect appears, immediately convert to the smallest repair lane and do not claim R9 closure first.

## READY_LOCAL 5 — Q10 factual reconciliation

Reconcile release packet/status claims against exact reachable R9 review/provenance anchors. Update only factual indexes/boundaries that actually changed. Do not mark release item 4 complete, and do not promote local evidence into WAN/security/release claims.

## READY_LOCAL 6 — Q11 release-evidence boundary reconciliation

Recheck release item 3 / WAN rows, historical negative classifications, and `READY_LIVE`. Current authoritative classification remains `READY_LIVE: none`; only a newly created concrete unresolved real-network question may open a live lane. VPS rental priority does not justify repeating a closed/blocked same-class experiment.

## READY_LOCAL 7 — Q12 governance/release-state reconciliation

Confirm D019 and other policy/value gates remain separated, and that RC/production/freeze/release authority has not been inferred from engineering completion. Do not set policy numbers, signing/key-custody/SBOM/publication policy, destructive migration, or release authority here.

## READY_LOCAL 8 — repository-wide item-4 refill

Item 4 remains incomplete, so repeat the broad core-surface inventory rather than declaring queue exhaustion from R9. Prefer any implemented core surface that lacks a **dedicated reachable independent bounded challenge on the current/relevant tree** or whose semantics changed since its last review:

1. `neko-reliable` ACK range / future-unsent / loss-retransmit / RTT-PTO / persistent congestion / Reno / fault simulation;
2. `CarrierState` generation / validation / hysteresis / single-active / drain-fail-activate;
3. concurrent manager / health / migration-back;
4. FairScheduler / multi-stream / Session+stream flow-control accounting;
5. Memory/UDP/TCP carrier close/error/resource semantics;
6. SessionRuntime lifecycle/resource/window/DeliveryAck accounting;
7. observability event/counter/high-water projection;
8. package/reproducibility/operator scripts;
9. Cargo manifests/lock/features/build/native hooks/unsafe inheritance;
10. cross-platform CLI/process-test semantics;
11. CLI exit-code / JSON / human-output contract;
12. algorithmic resource boundedness without capacity-pressure benchmarking or invented policy numbers;
13. release-packet factual consistency/evidence boundaries.

Do not count a narrow no-finding as repository-wide exhaustion. No-finding independent review is valid item-4 support; concrete defects become repair lanes.

## READY_LOCAL 9 — conditional live question only

Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved self-owned real-network question, classify it against `docs/standing-vps-lab-authorization.md` and `docs/vps-rental-window-priority.md`. Ordinary bounded self-owned TCP/UDP work inside standing authorization does not need per-run approval. Do not manufacture live work merely because the VPS is rented.

## Stop / escalation conditions

Continue implementation/review -> tests -> commit -> push -> next dependency-ready slice without waiting for reviewer cadence. Stop and escalate only for an unresolved BLOCKER/HIGH that cannot be auto-decided, core Session/Carrier/ACK/crypto/wire architecture change, D019 or other policy/value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
