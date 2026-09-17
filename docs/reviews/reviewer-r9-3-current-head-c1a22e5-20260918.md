# Independent bounded review — current R9-3 head `c1a22e5`

Date: 2026-09-18

Reviewed exact developer source/test head: `c1a22e5095c81132786da7b4f3298adf1f560970`.

Scope: the two developer commits after reviewer handoff `56f495f` (`9a113f2`, `c1a22e5`), the exact-current R9-3 post-return PTO/retransmit owner, `Recovery` overlap-copy bookkeeping, the built-binary R9-3 process oracle, and current hosted CI. This is local correctness/evidence review only; no live/WAN or release conclusion is added.

## Commit classification

- `9a113f2`: implementation + unit tests + handoff edit. It implements H-R9-040 lifecycle pruning for `acked_frames` and moves retransmit congestion admission to exact encoded wire bytes in both the post-return and lab paths.
- `c1a22e5`: process-test/oracle tightening only. It adds server `udp_return_packet_ack_sent.packet_number` cardinality and binds that packet number to the retransmitted packet number.

## Closed source seams

### H-R9-040 source repair

The exact-current `Recovery` keeps the positive-ACK marker only while sibling copies of that frame remain outstanding and removes it when the final copy retires on either ACK or loss. The new single-copy reuse and sibling-copy regressions directly challenge stale-marker poisoning. No contradictory defect was found in this bounded source review.

### H-R9-041 implementation ordering

The exact-current retransmit path now performs:

`build/seal -> can_send(exact encoded bytes) -> on_retransmit_sent(exact same bytes) -> socket send`.

The old plaintext-length admission mismatch is no longer present in the inspected callers.

H-R9-041 is **not yet acceptance-closed**, however: the previously required discriminating refusal regression is still absent. A generic cwnd refusal test is not enough; the regression must make the remaining budget distinguish plaintext length from encoded-wire length and prove refusal commits no retransmission ownership, emits no retransmit-send event, and performs no socket send, with a normal admitted control.

## H-R9-042 partial closure

`c1a22e5` closes the missing server side of the three-way packet-number bind in the positive R9-3 fixture. The process test now binds retransmit send == server Carrier ACK == positive client retirement when the single-retransmit assumption holds.

The required deterministic **just-before-PTO-deadline negative** remains absent. The current positive assertion `fired_at_us >= deadline_us` does not prove that the implementation cannot fire early. Add a query/runtime-level deterministic challenge at `deadline_us - 1` (or equivalent injected time) with zero PTO/retransmit transition; do not use wall-clock sleep as the oracle.

## H-R9-043 — current-head exact-tree gate failure / repeated-PTO oracle contradiction

**Severity: HIGH evidence/correctness gate blocker.** This does not yet prove a new Recovery state-machine defect.

GitHub-hosted Rust CI run `35254921227` for exact `c1a22e5` failed in `stable checks -> bash scripts/check.sh`; nightly decode fuzz smoke passed. The failing test is `reliable_udp_post_return_data_loss_recovers_via_pto_retransmit`.

The captured process trace is discriminating:

1. original post-return packet 4 is recovery-owned and intentionally suppressed;
2. PTO #1 fires after its deadline and sends fresh packet 5 for stable frame 48;
3. exact Session DeliveryAck for stream 1 / offset 48 / len 16 arrives;
4. before a positive Carrier packet ACK has settled Recovery, PTO #2 legitimately becomes due and sends fresh packet 6 for the same stable frame;
5. one late sibling Carrier ACK classifies accepted-empty and packet 6 receives the positive retirement;
6. final settlement is exactly zero-in-flight and the process otherwise completes successfully.

The test fails because its oracle requires exactly one PTO/retransmit. Exact-current `Recovery::on_pto` is allowed to select an outstanding frame on a later PTO while no positive Carrier ACK has yet retired the overlap lifecycle. Session DeliveryAck is a separate evidence domain and must not be used as a substitute Carrier ACK merely to suppress the second probe.

Therefore do **not** repair this by merging Session delivery confirmation with Carrier packet recovery or by suppressing packet recovery solely because Session bytes were confirmed.

### Smallest next repair / challenge

First make the R9-3 fixture deterministic with respect to the semantics already committed:

- preserve separate Session DeliveryAck and Carrier ACK ownership;
- accept bounded repeated PTO probes before the first positive Carrier retirement unless the fixture deliberately arranges the first retransmit ACK before the next authoritative PTO deadline;
- prove every retransmission uses a fresh packet number/nonce and stable logical/frame identity;
- bind the **actually positive-retired copy** to the corresponding server Carrier ACK and client retransmit event;
- if sibling ACKs arrive after another copy resolved the frame, accepted-empty is a valid typed classification, not a rejection or second transition;
- receiver Session delivery/confirmation remains exactly once despite overlapping packet copies;
- no retransmit occurs after positive Carrier retirement of the frame lifecycle;
- final Recovery is exactly zero and settlement occurs once after Session confirmation + Recovery zero.

If the coding agent prefers to retain a single-retransmission process oracle, it must make that timing a deterministic fixture condition (first retransmit ACK observed before the next PTO deadline) rather than changing Recovery semantics. Do not add sleeps as proof.

After resolving H-R9-043, close the still-missing H-R9-041 exact-wire refusal regression and H-R9-042 pre-deadline negative before declaring R9-3 complete. Then run the normal developer-local clean exact-tree gate on the final pushed source/test SHA and persist provenance.

## Evidence boundary / queue

Current exact `c1a22e5` has **failed hosted stable CI** and therefore is not an accepted exact-tree closure. Its hosted nightly fuzz smoke is separately green. No final developer-local clean exact-tree provenance for `c1a22e5` is accepted here.

`READY_LIVE` remains `none`; this finding is answerable locally and creates no new real-network question. Release item 3 and item 4 remain incomplete; RC/production/freeze/release authority flags remain false.

Repository-wide queue exhaustion is false. After H-R9-043 + remaining H-R9-041/H-R9-042 closure, continue R9-4 through R9-12, then the dedicated independent R9 review, then Q10/Q11/Q12 factual reconciliation.