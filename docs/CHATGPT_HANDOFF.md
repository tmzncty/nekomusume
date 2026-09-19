# ChatGPT reviewer handoff — H-R9-075/H-R9-076 closed at 1429df1; R9-10B is front

## Current repository truth

- Latest developer-owned source/test commit: exact `1429df1b565060054e24cfbae4d62bfdda5890da` (`test(session): H-R9-075 gapped ACK negative asserts full atomic contract`), on top of `f1edee86e9bd44c953915b745bd51418302db127` (`test(carrier): H-R9-076 resumed-session fixture applies first-record proof before gap ACKs`).
- Latest independent review finding: [`docs/reviews/independent-h-r9-076-resume-ack-gap-52017b5-20260919.md`](reviews/independent-h-r9-076-resume-ack-gap-52017b5-20260919.md), review commit `e3c0c5beb4a6b7240023021574fbd844154fcd3b`.
- **H-R9-075 closed at `f064424` + `52017b5`:** `SessionRuntime::delivery_ack` fails closed on forward ACK gap (`offset > confirmed watermark` → `Protocol`); `gapped_delivery_ack_cannot_skip_unconfirmed_range` proves gapped rejection + watermark/inflight/event atomicity.
- **H-R9-076 closed at `f1edee8` + `1429df1`:** `bounded_udp_blackhole_tcp_resume_preserves_order_and_exactly_once_bytes` now applies the already-delivered offset-0 Session proof first (watermark → 5) before draining later exact proofs in watermark order; `confirm` follows successful proof application. The full atomic contract is asserted in the unit negative.
- Developer-local clean exact-tree provenance for exact `1429df1`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-19T13:57:59Z → 2026-09-19T14:02:42Z, Linux x86_64, rustc 1.98.0. This remains developer-reported local evidence; reviewer-local execution is not claimed.
- R9-7 remains independently bounded no-finding closed at review commit `cdac663e4d2649c8f87fc2263766616dc9b4bbe9` over source/test exact `d97a536`.
- R9-8 warm readiness remains independently bounded no-finding closed at `7c87ac675a381154ae7fceca987e7f2f106c26cf`.
- R9-9 health/promotion/switch ordering remains independently bounded no-finding closed at `b4a527a1fff994ee0836893f850c36eadb609eb1`.
- H-R9-074 remains closed at source/test exact `d97a536`.
- R9-10 remains open. Developer note `50689e7ac39253bcea912509c6e7c191be4cc00e` is not an independent closure; H-R9-075/H-R9-076 must close first, then executable replay identity/bytes must be challenged.
- Earlier Candidate A future/never-sent ACK fail-closed guard remains present before RTT/loss/PTO mutation. Candidate B mixed datagram-drop projection remains split into queue-full subset plus terminal remainder; do not reopen without contradictory current evidence.
- Reviewer-local execution is **not** claimed this round. The concrete execution evidence above is GitHub-hosted CI only.
- `READY_LIVE: none`.
- Release item 3 remains incomplete.
- Release item 4 remains incomplete.
- `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency and is never a ticket length or reason to idle.

Do not weaken the H-R9-075 guard to make the old integration fixture pass. Repair the stale fixture/evidence ordering instead. Do not reopen R9-7/R9-8/R9-9/H-R9-074 without contradictory current repository evidence.

# READY_LOCAL 1 — FRONT HIGH H-R9-076 resumed-session gap-ACK integration repair

Owners:

- `crates/neko-session/src/lib.rs::SessionRuntime::delivery_ack`
- `crates/neko-carrier/tests/resumed_session.rs::bounded_udp_blackhole_tcp_resume_preserves_order_and_exactly_once_bytes`
- executable R9 pending-ACK ordering logic in `crates/neko-cli/src/main.rs`

The source guard correctly rejects `offset > current`. The failing integration fixture queues three send ranges, receives the first UDP range, then after resume iterates only the later ranges and immediately calls `delivery_ack` at offset 5 while the Session confirmed watermark remains 0. It then calls `FailoverController::confirm` as though logical proof had been accepted. That is precisely the behavior H-R9-075 was meant to forbid.

Smallest repair contract:

1. Preserve the fail-closed `offset > confirmed_watermark` guard. Do not restore cumulative promotion across an unproved gap and do not redesign ACK/wire semantics.
2. Preserve the adversarial ordering value of the integration fixture. Later resumed records/ACK proofs may arrive before the missing first-range proof, but they must be retained/buffered rather than applied to `SessionRuntime` across a gap.
3. A minimal coherent fixture shape is:
   - receive resumed later ranges (offsets 5 and 13) first;
   - hold their exact Session ACK proofs without calling `delivery_ack` or `FailoverController::confirm`;
   - replay/observe the already-delivered offset-0 record and retain receiver exact-dedup behavior;
   - apply the offset-0 Session proof;
   - drain held offset-5 and offset-13 proofs strictly when each equals the current confirmed watermark;
   - call `FailoverController::confirm(DataId)` only after the corresponding Session proof was successfully applied.
4. Preserve final application bytes exactly once as `alpha-bounded-exactly-once`, final confirmed watermark equal to all bytes, and `tcp_resend()` empty only after all logical identities are legitimately confirmed.
5. Strengthen `gapped_delivery_ack_cannot_skip_unconfirmed_range` to snapshot and assert unchanged after the rejected future ACK:
   - confirmed watermark;
   - per-stream `send_inflight`;
   - `session_send_inflight`;
   - observable event count.
   The unit test is in-module; do not add a production accessor just for the oracle.
6. Ordered first-range -> second-range ACKs must still advance exactly by proved contiguous ranges. Preserve accepted-empty/duplicate semantics already committed elsewhere.
7. Keep the executable reverse/out-of-order R9 ACK buffering positive control green; do not “fix” the integration test by forcing transport arrival order.
8. Final pushed repair SHA must pass in a safe clean checkout/worktree:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - confirm clean tree;
   - persist exact reachable SHA, UTC start/end, OS/arch, stable Rust version, exit codes and clean-tree state.
9. This repair does not require decoder/parser/crypto framing changes. Do not mechanically require a developer-local decode fuzz run. The hosted fuzz success at `52017b5` does not compensate for the stable gate failure.

After a green exact-tree repair/provenance, continue immediately to R9-10B without waiting for reviewer cadence.

# READY_LOCAL 2 — R9-10B actual cross-Carrier replay identity / bytes / dedup / conflict

Challenge the executable promoted-TCP replay owner, not only `ConcurrentCarrierManager`/`FailoverController` helper state.

Prove or falsify:

- the actual bytes placed on TCP after UDP failure correspond exactly to the authoritative retained replay-set `(DataId, bytes)` identities and bytes;
- stable Session/stream/offset identity survives UDP -> TCP replay;
- a range already logically confirmed is absent from actual replay, not merely absent from a manager map;
- exact duplicate receive is idempotent and produces application delivery once;
- conflicting bytes/context for the same logical identity fail closed without advancing Session confirmation/watermarks;
- Carrier packet ACK state cannot suppress required Session replay and TCP does not acquire a duplicate UDP packet-ACK layer;
- a valid TCP Session DeliveryAck settles one logical owner once; duplicate/reordered Carrier feedback cannot manufacture another owner.

**Specific source-of-truth seam:** current executable automatic-health path calls `failover.tcp_resend().unwrap().len()` and then sends a positional `records.into_iter().skip(uncertain_start).take(tcp_records)` slice. `tcp_resend()` itself returns authoritative `(DataId, Vec<u8>)` entries. Challenge whether a deterministic reachable state can make the retained identities/bytes differ from the positional slice while count remains equal. If yes, concrete defect becomes queue front: make the authoritative replay set drive or validate actual wire replay with the smallest change. Do not redesign replay architecture.

A no-finding closure is valid only with exact executable owners, focused deterministic challenge, exclusions and reachable anchor. Then continue to R9-10C.

# READY_LOCAL 3 — R9-10C replay cleanup / partial-promotion negatives

After H-R9-076 and R9-10B close, challenge:

- shutdown/error before promotion cleans Recovery packet ownership, Reno bytes, ACK tracker and retained plaintext without rewriting historical evidence;
- failed/partial promotion emits no false replay success/final settlement;
- failed TCP replay send preserves only ownership required for a legitimate retry and creates no phantom Session receipt;
- terminal cleanup prevents later replay, Carrier ACK application, Session confirmation, readiness mutation, health promotion or new Data ownership;
- repeated teardown/cleanup is idempotent;
- current committed resource bounds hold without inventing new policy values.

A no-finding review is valid only with exact owners, focused deterministic regressions/source reasoning, exclusions and reachable anchor.

# READY_LOCAL 4 — R9-11 lifecycle / terminal invariants

Close remaining lifecycle invariants for the cross-process reliable-UDP/failover integration:

- exactly one terminal success/failure classification per bounded operation;
- intermediate timeout/residual diagnostics are not terminal success/failure;
- malformed-bound/socket/receive terminal errors cannot be downgraded into timeout continuation;
- no post-terminal retransmit, delivery confirmation, Carrier ACK emission, readiness mutation, health promotion or new Data ownership;
- listener/socket/session/recovery/Reno/ACK-tracker/retained-plaintext state cleans deterministically on normal/error exits;
- cleanup does not rewrite historical failure/recovery evidence;
- repeated teardown/cleanup is idempotent at current committed semantics.

# READY_LOCAL 5 — R9-12 complete cross-process slice + final exact-tree provenance

After R9-10 and R9-11 close, run on the final pushed developer SHA in a safe clean checkout/worktree:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Confirm clean tree and persist exact reachable pushed SHA, UTC start/end, OS/arch, stable Rust version, exit codes and clean-tree state. Keep developer-local provenance distinct from hosted CI and reviewer execution.

Run pinned decode fuzz only if accumulated R9 repairs actually change wire decoder/parser/crypto framing; do not run it mechanically otherwise.

# READY_LOCAL 6 — dedicated independent final R9 review

Perform a fresh independent bounded challenge of materially new R9 send/admission/reservation/socket-outcome/recovery/PTO/demux/Session-ACK/Carrier-ACK/failover/readiness/promotion/migration/replay/cleanup/diagnostic behavior at the final R9 implementation anchor.

A no-finding review is valid item-4 support only when it names inspected owners, challenged invariants, focused deterministic tests/commands, exclusions, exact reachable anchor and evidence class. Any concrete correctness/security/evidence BLOCKER/HIGH returns immediately to smallest repair -> regression -> exact-tree gate.

# READY_LOCAL 7 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent final-R9 review anchor exists, reconcile `docs/status.md`, `docs/release-security-review-packet.md`, `IMPLEMENTATION_PLAN.md` and related Q10/Q11/Q12 evidence text against exact reachable implementation/review anchors.

Preserve evidence boundaries:

- release item 3 remains incomplete unless repository truth independently closes it;
- item 4 closes only to the extent supported by reachable independent review evidence;
- no historical WAN artifact rewrite;
- no automatic RC/freeze/release/production transition;
- policy/authority gates stay separate.

# READY_LOCAL 8 — repository-wide item-4 inventory refill if still incomplete

If item 4 remains incomplete after final R9 review/reconciliation, inventory all mandatory core surfaces and create only genuinely missing independent bounded challenges:

1. `neko-reliable` recovery;
2. `CarrierState` generation/validation/hysteresis/single-active/drain/fail/activate;
3. concurrent Carrier Manager / health / migration-back;
4. FairScheduler / multi-stream / session+stream flow accounting;
5. Memory/UDP/TCP carrier adapter close/error/resource semantics;
6. `SessionRuntime` lifecycle/resource/window/DeliveryAck accounting;
7. `neko-observe` event/counter/high-water projection;
8. package/reproducibility/operator scripts;
9. dependency/build surface including manifests/lock/features/native hooks/unsafe inheritance;
10. cross-platform CLI/process tests;
11. CLI exit/JSON/human-output contract;
12. algorithmic resource boundedness without capacity-pressure benchmark or invented values;
13. release packet factual consistency/evidence boundary.

Earlier reachable independent reviews broadly cover established core implementations. Refill only surfaces still unreviewed or materially changed by R9. No checker/schema/framework/docs filler.

Repository-wide `queue exhausted` is permitted only when this broad inventory finds no concrete defect, no READY_LOCAL review/support lane, no READY_LIVE question, and every remaining item is truly policy/environment/release-authority gated.

# READY_LOCAL 9 — conditional READY_LIVE only for a genuinely new real-network question

**Current classification: `READY_LIVE: none`.** Standing VPS authorization remains valid, but do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

Create a READY_LIVE row only if new code/instrumentation/hypothesis/path conditions produce a concrete unresolved real-network question that cannot be answered locally and remains inside `docs/standing-vps-lab-authorization.md`.

## Separate non-blocking maintainer / policy / authority gates

Do not decide or modify while executing this queue:

- `SessionRuntime.events` retention/capacity policy;
- D019 source-retention/no-reset policy or numeric candidate values;
- RSEC-001 adversarial-load/capacity suitability conditions;
- signing/key-custody/SBOM/publication policy;
- previous frozen-release policy;
- core Session/Carrier/ACK/crypto/wire architecture;
- destructive/canonical-meaning migration;
- RC/freeze/release/production authority.

A policy gate in one lane does not block independent READY_LOCAL review/support lanes.
