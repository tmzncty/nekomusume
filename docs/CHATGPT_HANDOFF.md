# ChatGPT reviewer handoff — H-R9-075 HIGH is front; R9-10 remains open

## Current repository truth

- Review-start `main` was exact `50689e7ac39253bcea912509c6e7c191be4cc00e`, developer-owned docs/review only: `docs/reviews/dev-r9-10-uncertain-replay-20260919.md`. It changes no implementation, tests, fixtures, protocol/policy state, package surface, or live evidence.
- Current reviewer finding anchor is exact `ecc28e6e25c2bf6eccd21328ea57da1c7fc4e0fb`: [`docs/reviews/independent-r9-10-session-ack-gap-50689e7-20260919.md`](reviews/independent-r9-10-session-ack-gap-50689e7-20260919.md).
- Latest developer-owned source/test anchor remains exact `d97a536414a78ff44ef80b1a6d428ddf96ae6e22`; no source/test commit landed in the R9-8/R9-9/R9-10 docs-only review sequence.
- R9-8 warm TCP readiness remains independently bounded no-finding closed at review commit `7c87ac675a381154ae7fceca987e7f2f106c26cf` over developer review `7f0fcdafbe908d28a7d7715a2b04725ee31883e2`.
- R9-9 health outcome / promotion / switch ordering remains independently bounded no-finding closed at review commit `b4a527a1fff994ee0836893f850c36eadb609eb1` over developer review `5e73aead6d16506a6b299a459928b9c001022405`.
- R9-7 remains independently bounded no-finding closed at [`docs/reviews/independent-r9-7-process-result-truth-d97a536-20260919.md`](reviews/independent-r9-7-process-result-truth-d97a536-20260919.md), review commit `cdac663e4d2649c8f87fc2263766616dc9b4bbe9`.
- H-R9-074 remains closed at source/test exact `d97a536`.
- **R9-10 is NOT independently closed.** The developer note at `50689e7` correctly reviews `ConcurrentCarrierManager` uncertain marking/removal/capacity, but misses a concrete Session-confirmation defect in `SessionRuntime::delivery_ack` that can manufacture confirmation across an unproved gap.
- **H-R9-075 HIGH:** with two queued ranges `[0,2)` and `[2,4)`, `delivery_ack(stream, offset=2, len=2)` currently computes `end=4`, `current=0`, `delta=4`, observes 4 bytes in flight, accepts, releases all 4 bytes, and advances the confirmed watermark to 4. The ACK proved only `[2,4)`. This violates the current exact Session-delivery evidence boundary and can incorrectly remove still-unproved bytes from replay eligibility. The executable reliable-UDP owner already recognizes this invariant by buffering `offset > watermark` rather than applying it immediately.
- Earlier accepted Candidate A future/never-sent ACK fail-closed guard remains present before RTT/loss/PTO mutation. Candidate B mixed datagram-drop projection remains split correctly into `queue_full` subset plus terminal remainder.
- Developer-local clean exact-tree provenance for source/test exact `d97a536` remains the latest accepted code-tree local gate. Reviewer-local execution is not claimed for the docs-only R9-8/R9-9/R9-10 review commits or H-R9-075 finding.
- `READY_LIVE: none`.
- Release item 3 remains incomplete.
- Release item 4 remains incomplete.
- `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency and is never a ticket length or reason to idle.

Do not reopen R9-7/R9-8/R9-9 or H-R9-074 without contradictory current repository evidence. Do not treat developer note `50689e7` as independent R9-10 closure while H-R9-075 remains open.

# READY_LOCAL 1 — FRONT HIGH H-R9-075 SessionRuntime gapped DeliveryAck confirmation

Owner: `crates/neko-session/src/lib.rs::SessionRuntime::delivery_ack` plus the existing executable ACK-ordering callers/tests.

Current positive-advancing logic checks `end >= current` and `delta=end-current <= inflight`, but does not require the exact DeliveryAck to begin at the current confirmed watermark. A future second-range ACK can therefore release preceding unproved bytes.

Smallest repair contract:

1. Add a focused deterministic regression with two queued records/ranges. ACK the second range first and require fail-closed, with exact atomic preservation of confirmed watermark, per-stream/session in-flight accounting, and event/evidence count.
2. Then ACK the first range and second range in valid order; require watermark/accounting to advance only by the proved contiguous ranges.
3. Preserve already-committed accepted-empty duplicate behavior; do not redesign Session ACK encoding/cumulative policy as part of this repair.
4. Keep existing process-level reverse/out-of-order ACK buffering positive control green.
5. Smallest source repair only. No wire/crypto/Carrier architecture change and no policy values.
6. On final pushed repair SHA run in a safe clean checkout/worktree:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - confirm clean tree; persist exact SHA, UTC start/end, OS/arch, stable Rust version, exit codes.
7. This does not change decoder/parser/crypto framing, so do not run decode fuzz mechanically.

After the repair and exact-tree gate, continue immediately to R9-10B without waiting for reviewer cadence.

# READY_LOCAL 2 — R9-10B actual cross-Carrier replay identity / bytes / dedup / conflict

Challenge the executable promoted-TCP replay owner, not only `ConcurrentCarrierManager` state helpers.

Prove or falsify:

- the actual bytes put on TCP after UDP failure correspond exactly to the retained replay-set identity and bytes;
- stable Session/stream/offset identity survives UDP -> TCP replay;
- a range already logically confirmed is absent from actual replay, not merely absent from a manager map;
- exact duplicate receive is idempotent and produces application delivery once;
- conflicting bytes/context for the same logical identity fail closed and do not advance Session confirmation/watermarks;
- Carrier packet ACK state cannot suppress required Session replay and TCP does not acquire a duplicate UDP packet-ACK layer;
- a valid TCP Session DeliveryAck settles the logical owner once; duplicate/reordered Carrier feedback around replay cannot manufacture a second owner.

Important owner check: the current executable path uses `FailoverController::tcp_resend()` to derive replay count and then walks the original record slice for wire replay. Challenge that source-of-truth coupling explicitly. If retained replay identity/bytes and actual wire replay can diverge under a deterministic current-state case, repair by making the authoritative replay set drive/validate the actual send; do not redesign replay architecture.

A concrete defect becomes queue front immediately. If no defect, persist a scope-exact independent bounded no-finding note naming executable owners/tests/exclusions/reachable anchor, then continue to R9-10C.

# READY_LOCAL 3 — R9-10C replay cleanup / partial-promotion negatives

After H-R9-075 and R9-10B close, challenge:

- shutdown/error before promotion cleans Recovery packet ownership, Reno bytes, ACK tracker and retained plaintext without rewriting historical evidence;
- failed/partial promotion emits no false replay success/final settlement;
- failed TCP replay send preserves only ownership required for a legitimate retry and creates no phantom Session delivery receipt;
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

Perform a fresh independent bounded challenge of the materially new R9 send/admission/reservation/socket-outcome/recovery/PTO/demux/Session-ACK/Carrier-ACK/failover/readiness/promotion/migration/replay/cleanup/diagnostic behavior at the final R9 implementation anchor.

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
2. `CarrierState`;
3. concurrent Carrier Manager / health / migration-back;
4. FairScheduler / flow accounting;
5. carrier adapters;
6. `SessionRuntime`;
7. observability;
8. package/reproducibility/operator scripts;
9. dependency/build surface;
10. cross-platform CLI/process tests;
11. CLI exit/JSON/human-output contract;
12. algorithmic resource boundedness;
13. release packet factual consistency/evidence boundary.

Earlier reachable independent reviews broadly cover established core implementations. Refill only surfaces that remain unreviewed or were materially changed by R9. No checker/schema/framework/docs filler.

Repository-wide `queue exhausted` is permitted only when this broad inventory finds no concrete defect, no READY_LOCAL review/support lane, no READY_LIVE question, and every remaining item is truly policy/environment/release-authority gated.

# READY_LOCAL 9 — conditional READY_LIVE classification only for a new real-network question

**Current classification: `READY_LIVE: none`.** Standing VPS authorization remains valid, but do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

Create a READY_LIVE row only if new code/instrumentation/hypothesis/path conditions produce a concrete unresolved real-network question that cannot be answered locally and stays inside `docs/standing-vps-lab-authorization.md`.

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
