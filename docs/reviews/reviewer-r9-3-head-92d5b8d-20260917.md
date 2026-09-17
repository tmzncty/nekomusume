# Independent bounded review — R9-3 head `92d5b8d` (2026-09-17 UTC)

## Scope

Exact reviewed developer source/test commits:

- `be50ed5955b7eaba88275cbcbd0be3addf1dd43f` — H-R9-040 overlap ACK/retransmit eligibility repair plus PTO deadline query.
- `92d5b8d80da645c122d679a33a806fd0c6e59ce8` — R9-3 post-return data-loss/PTO/retransmission integration and built-binary regression.

Inspected owners and contracts:

- `crates/neko-reliable/src/lib.rs` — `Recovery::{on_ack,on_pto,next_pto_deadline_us}`, overlapping frame-copy accounting and tests.
- `crates/neko-cli/src/main.rs` — authenticated Carrier ACK observation timing and post-return PTO/retransmit loop.
- `crates/neko-cli/tests/probe.rs` — `reliable_udp_post_return_data_loss_recovers_via_pto_retransmit` and neighboring P4/order fixtures.
- `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, `docs/standing-vps-lab-authorization.md`, `docs/vps-rental-window-priority.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, `docs/release-security-review-packet.md`, and the pre-existing `docs/CHATGPT_HANDOFF.md`.

Excluded from this bounded review: new WAN execution, performance conclusions, release/freeze/production policy, D019 policy values, new capacity/security numbers, and cryptographic/wire redesign.

GitHub-hosted Rust CI for exact `92d5b8d` is separately green at run `35244453584` (`stable checks` and `nightly decode fuzz smoke`). Hosted CI is cross-evidence only; no reachable developer-local exact-tree provenance for `92d5b8d` was found in the repository during this review.

## Finding 1 — H-R9-040 remains open: positive-ACK marker is not pruned

Severity: **HIGH correctness/resource blocker**.

`be50ed5` correctly separates retransmit eligibility from packet-copy lifetime by adding `acked_frames`: a positive Carrier ACK marks a stable `FrameId`, later overlapping loss does not re-emit it, and PTO filters it. That closes the resurrection half of H-R9-040.

However, the accepted repair contract also required the positive-ACK state to be pruned when the last packet copy retires. Exact `92d5b8d` never removes entries from `acked_frames`:

- ACK retirement inserts the frame into `acked_frames` and calls `release_frame`, but ignores whether that removed the last outstanding copy.
- Loss retirement with an ACK marker calls `release_frame` and continues, again without removing the marker when the last copy disappears.
- `on_pto` filters every frame still present in `acked_frames`.

This is not documentation-only. It has two concrete consequences:

1. `acked_frames` grows with positively-ACKed stable frame identities instead of remaining bounded by current Recovery ownership.
2. A later legal lifecycle that reuses the same `FrameId` can be incorrectly suppressed from PTO/retransmission because the stale marker survives after all earlier copies are gone.

A minimal discriminating sequence is: send single-copy `F`, ACK it so `frame_outstanding(F)==false`, then later send a fresh higher-numbered packet carrying `F`; before any ACK for the new lifecycle, PTO must be allowed to select `F`. Current code filters it because the old `acked_frames` entry remains.

### Required repair

Keep the existing packet-copy accounting and positive-ACK gate, but remove the per-frame ACK marker exactly when `release_frame(...)` reports that the last outstanding copy has retired. Do this on both ACK and loss retirement paths. Do not eagerly erase sibling packet copies and do not change Session/Carrier ACK semantics.

Focused deterministic regressions:

- single-copy positive ACK -> ownership empty -> same `FrameId` reused by a later higher packet number -> PTO selects it normally;
- same-call sibling ACK + time-loss -> no retransmit for the already-ACKed frame, final ownership empty, ACK marker pruned;
- split-call sibling ACK then later loss -> PTO suppressed while old sibling remains, later final loss emits no retransmit, marker pruned;
- all-copies-lost control still schedules the frame exactly once;
- future/never-sent ACK rejection remains atomic.

## Finding 2 — H-R9-041: retransmit congestion admission uses plaintext length, not encoded wire bytes

Severity: **HIGH correctness/resource blocker**.

The R9-3 contract requires retransmission order:

`encode/seal -> can_send(exact encoded bytes) -> on_retransmit_sent(exact same bytes) -> socket send`.

Exact `92d5b8d` instead does:

1. `rt.pto_probe()` returns `(frame, plaintext)`;
2. `rt.can_send(plaintext.len() as u64)`;
3. only then constructs `ProcessMessage::Data` and seals it;
4. `rt.on_retransmit_sent(..., re_sealed.len() as u64, frame)` commits the larger encoded/encrypted byte count.

Therefore congestion admission and Recovery/Reno ownership are checked against different byte counts. Near the congestion boundary, plaintext can be admitted while the actual encoded datagram should be refused. This violates the already-decided M2 ownership contract and can over-admit bytes without any architecture or policy ambiguity.

### Required repair

Build and seal the retransmission first. Then call `can_send(re_sealed.len() as u64)`. Only on success call `on_retransmit_sent` with the same exact length, then perform the socket send. Do not add a second cwnd ledger or new capacity value.

Add a deterministic refusal regression using existing cwnd/accounting state such that the remaining admissible budget is between the plaintext and sealed lengths. The refusal must commit no retransmission packet ownership, no socket send and no `r9_udp_retransmit_sent` event. A normal admitted control must still commit/send exactly once.

This change does not alter decoder/parser/crypto framing semantics, so decode fuzz is not required solely for this ordering repair unless implementation expands into those surfaces.

## Finding 3 — H-R9-042: R9-3 built-binary oracle does not yet prove its stated packet identity / PTO boundary

Severity: **HIGH evidence/oracle blocker**.

`reliable_udp_post_return_data_loss_recovers_via_pto_retransmit` proves many useful facts: both processes succeed, the original was Recovery-owned before suppression, exactly one PTO diagnostic fires with `fired_at_us >= deadline_us`, retransmission uses a fresh packet number and stable frame 48, exactly one exact Session transition occurs, exactly one positive Carrier retirement occurs, no rejected/accepted-empty substitution appears, and exactly one zero-in-flight settlement occurs.

Two acceptance claims are still missing:

1. The test comment says “three-way bind”, but it only checks retransmit packet number == client retirement packet number. It never parses the server `udp_return_packet_ack_sent.packet_number`, so server ACK identity/cardinality is not cross-bound.
2. The R9-3 acceptance contract requires a deterministic just-before-deadline negative proving no PTO/retransmit fires early. The current process test only observes the positive firing at/after the deadline.

### Required oracle closure

After H-R9-040/H-R9-041 repairs:

- require exactly one retransmission send event, exactly one server Carrier ACK event for the retransmitted copy and exactly one positive client retirement; bind all three packet numbers equal and different from the suppressed original;
- retain exactly-one `stream=1 offset=48 len=16` Session confirmation, zero rejected/accepted-empty and exactly-one zero-in-flight settlement;
- add a deterministic bounded pre-deadline challenge at `deadline_us - 1` (or equivalent injectable/query-level test) proving zero PTO probe/retransmit transition before the authoritative deadline, without sleeping as the oracle.

Do not invent a new PTO policy number; reuse the existing M2 lab granularity/ack-delay inputs already selected by the fixture.

## Queue consequence

R9-3 is not closed at `92d5b8d`. Repository-wide queue exhaustion is false. The next dependency order is:

1. finish H-R9-040 pruning/FrameId-reuse closure;
2. repair H-R9-041 exact-wire-byte retransmit admission;
3. close H-R9-042 R9-3 exact packet bind + pre-deadline oracle, then run the developer-local clean exact-tree gate on the resulting pushed source/test SHA;
4. continue R9-4 through R9-12, dedicated independent R9 review, then Q10/Q11/Q12 factual reconciliation.

`READY_LIVE` remains none: these findings are local correctness/evidence questions and do not create a new unresolved real-network hypothesis.
