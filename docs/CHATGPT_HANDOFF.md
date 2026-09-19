# ChatGPT reviewer handoff — R9-10B independently closed; R9-10C is front

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `70e87f086b0bbd2d13cd9bae073cb1cdb4c5abd4` (`fix(cli): R9-10B TCP replay bytes/identity from authoritative retained set`).
- Latest independent bounded review support: `586148eb15e049226a1999808d6174023a361814`, `docs/reviews/independent-r9-10b-executable-replay-70e87f0-20260919.md`.
- **R9-10B is closed at exact source/test `70e87f086b0bbd2d13cd9bae073cb1cdb4c5abd4`.** Automatic-health TCP replay now constructs each replayed `OutboundRecord` directly from the authoritative retained `(DataId, bytes)` returned by `FailoverController::tcp_resend()`, rather than using retained count plus a positional slice. Per-record authenticated TCP Session DeliveryAck is applied to `SessionRuntime` before the exact retained identity is removed by `confirm`.
- The previous handoff spelled the full `70e87f0` SHA incorrectly. The GitHub-resolvable pushed commit is `70e87f086b0bbd2d13cd9bae073cb1cdb4c5abd4`; use this exact SHA for shared provenance/evidence references.
- Developer-local clean exact-tree provenance recorded for short SHA `70e87f0` remains: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-19T15:00:05Z → 2026-09-19T15:04:57Z, Linux x86_64, rustc 1.98.0. This remains developer-reported local evidence; reviewer-local execution is not claimed. The unique pushed `70e87f0` prefix resolves to the exact SHA above.
- GitHub combined status for exact `70e87f086b0bbd2d13cd9bae073cb1cdb4c5abd4` currently has no status contexts. Hosted CI is supplemental and is not a wait condition.
- **H-R9-075 remains closed.** `SessionRuntime::delivery_ack` rejects forward gaps (`offset > confirmed watermark`) before any watermark/in-flight/event mutation; the strengthened negative proves atomic rejection, and ordered exact proofs still advance.
- **H-R9-076 remains closed.** The resumed-session fixture applies the already-delivered offset-0 Session proof before later resumed proofs and only calls `confirm` after successful proof application.
- The independent process-resource sampler finding remains open as a package/evidence-reliability lane: `43bad9d5cfd91333b2f6df513d58aa29cfac1c55`. It is not a transport correctness blocker and must be repaired with deterministic synchronization, not a longer arbitrary sleep.
- R9-7 remains independently bounded no-finding closed at `cdac663e4d2649c8f87fc2263766616dc9b4bbe9` over source/test exact `d97a536`.
- R9-8 warm readiness remains independently bounded no-finding closed at `7c87ac675a381154ae7fceca987e7f2f106c26cf`.
- R9-9 health/promotion/switch ordering remains independently bounded no-finding closed at `b4a527a1fff994ee0836893f850c36eadb609eb1`.
- Candidate A (`Recovery::on_ack` future/unsent `largest`) remains closed by the current fail-closed guard before RTT/loss/PTO mutation and dedicated regressions.
- Candidate B (`record_datagrams` mixed queue/terminal drop projection) remains closed by current mixed-delta regressions and separate reason projection.
- `READY_LIVE: none`. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD, or Experimental Track without a new code/instrumentation/hypothesis/path condition that creates a concrete unresolved real-network question.
- Release item 3 and item 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency and is never a ticket length or reason to idle.

## READY_LOCAL 1 — FRONT: R9-10C replay cleanup / partial-promotion negatives

Challenge exact-current executable and policy-core cleanup/failure atomicity around UDP -> TCP promotion and partial replay.

Owners to inspect:

- `crates/neko-cli/src/main.rs` automatic-health promotion + TCP replay loop;
- `FailoverController::{apply_manager_decision,tcp_resend,confirm,receive,apply_migration_back}`;
- `ConcurrentCarrierManager::{mark_owner_uncertain,replay_uncertain,finish_drain,confirm}` where applicable;
- current process tests and D005/D064/Session v0 evidence boundaries.

Required invariants:

1. failed authentication/readiness/promotion cannot consume retained uncertain ownership or emit success evidence;
2. a TCP write/read/auth/DeliveryAck failure before `confirm` cannot fabricate Session confirmation or remove the unsatisfied retained identity;
3. successful authenticated Session proof removes exactly that proved identity once; exact duplicate confirmation/receive is idempotent or typed-rejected according to the existing API, never double-counted;
4. partial replay cannot silently drop a later unsent/unconfirmed identity; if the operation terminates, final success/settled evidence must remain impossible;
5. Carrier packet feedback cannot substitute for Session proof, and TCP must not acquire a duplicate UDP packet-ACK ownership layer;
6. migration-back must not bypass unresolved replay ownership or turn a failed partial replay into success;
7. terminal cleanup releases live replay/socket/runtime ownership while preserving only already-approved bounded diagnostic history semantics.

Prefer focused deterministic existing fixtures plus smallest new negative only if a real oracle gap is found. Do not add soak/capacity pressure, invent policy numbers, redesign DataId/ACK/wire/crypto, or alter D019/D064.

If no concrete defect is found, commit a scope-precise independent bounded no-finding note with exact inspected owners/tests/exclusions and proceed immediately.

## READY_LOCAL 2 — process-resource sampler positive-fixture determinism

Independent finding: `43bad9d5cfd91333b2f6df513d58aa29cfac1c55`, `docs/reviews/independent-process-resource-sampler-flake-6f4a61c-20260919.md`.

Repair the test synchronization contract only. Replace the `known_child.py` short fixed lifetime race with a bounded readiness/release handshake (or equivalently deterministic synchronization) that proves the expected FDs/listener exist before the positive oracle requires them to be sampled. Preserve real `/proc` observation, truthful `null` exit-race semantics, process-group cleanup and existing sampler production behavior. Do not simply lengthen an arbitrary sleep. Run the normal exact-tree local gate.

## READY_LOCAL 3 — R9-11 lifecycle / terminal resource closure

Bounded independent challenge of the full failover/migration lifecycle after R9-10: socket/Session/Recovery/CarrierManager/replay ownership on success, failure, timeout, shutdown and post-return paths. Reuse existing deterministic process fixtures; do not add soak/capacity pressure. Terminal state must fail closed on further mutators while approved diagnostic history may remain.

## READY_LOCAL 4 — R9-12 final exact-tree provenance

After the coherent R9 repair/review group is stable, run and persist developer-local provenance for the final pushed developer SHA:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- `git diff --check`
- clean tree
- exact GitHub-resolvable pushed SHA, UTC start/end, exit codes, OS/arch and stable Rust version

Run pinned decoder fuzz only if the coherent group actually touched wire decoder/parser/crypto framing; otherwise do not mechanically repeat it. Hosted CI remains supplemental cross-evidence.

## READY_LOCAL 5 — dedicated final independent R9 bounded review

On the final R9 exact tree, independently re-challenge the combined invariants across Recovery, Session logical proof, Carrier readiness/promotion, uncertain retention, TCP replay/dedup, migration-back and terminal cleanup. A no-finding bounded review is valid item-4 support. If a concrete defect appears, immediately convert to the smallest repair lane and do not claim R9 closure first.

## READY_LOCAL 6 — Q10 factual reconciliation

Reconcile release packet/status claims against exact reachable R9 review/provenance anchors. Update only factual indexes/boundaries that actually changed; do not mark release item 4 complete and do not promote local evidence into WAN/security/release claims.

## READY_LOCAL 7 — Q11 release-evidence boundary reconciliation

Recheck release item 3 / WAN rows, historical negative classifications, and `READY_LIVE`. Current authoritative classification remains `READY_LIVE: none`; only a newly created concrete unresolved real-network question may open a live lane.

## READY_LOCAL 8 — Q12 governance/release-state reconciliation

Confirm D019 and other policy/value gates remain separated, and that RC/production/freeze/release authority has not been inferred from engineering completion. No policy numbers, signing/key-custody/SBOM/publication policy, destructive migration, or release authority change may be made here.

## READY_LOCAL 9 — repository-wide item-4 refill

Item 4 remains incomplete, so repeat the broad core-surface inventory rather than declaring queue exhaustion from R9. Prefer any implemented core surface that lacks a dedicated reachable independent bounded challenge: reliable recovery; CarrierState; concurrent manager/health/migration-back; scheduler/flow-control; Memory/UDP/TCP adapters; SessionRuntime; observability; package/reproducibility/operator scripts; dependency/build/native hooks/unsafe inheritance; cross-platform CLI/process semantics; CLI JSON/human/exit contracts; algorithmic boundedness; release-packet factual consistency. No-finding independent review is valid support; concrete defects become repair lanes.

## READY_LOCAL 10 — conditional live question only

Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved self-owned real-network question, classify it against `docs/standing-vps-lab-authorization.md` and `docs/vps-rental-window-priority.md`. Ordinary bounded self-owned TCP/UDP work inside standing authorization does not need per-run approval. Do not manufacture live work merely because the VPS is rented.

## Stop / escalation conditions

Continue implementation/review -> tests -> commit -> push -> next dependency-ready slice without waiting for reviewer cadence. Stop and escalate only for an unresolved BLOCKER/HIGH that cannot be auto-decided, core Session/Carrier/ACK/crypto/wire architecture change, D019 or other policy/value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.