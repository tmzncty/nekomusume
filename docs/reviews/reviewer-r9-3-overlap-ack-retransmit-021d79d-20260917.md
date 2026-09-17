# R9-3 independent bounded review — overlapping-copy ACK vs retransmit eligibility

**Finding:** `H-R9-040` — HIGH correctness/resource blocker for R9-3.

**Repository truth inspected:** current reviewer/navigation head `b51c668f0424055b38a5fe9b6eba0a3b212abaca`; latest developer-owned source/test tree remains exact `021d79d88dadc9dbc7cf749745b2395c06602b29`.

**Scope:** `crates/neko-reliable/src/lib.rs` Recovery overlapping-copy accounting and PTO/loss scheduling; `crates/neko-carrier/src/lib.rs` `PathRecovery` / `ReliableUdpRuntime` ACK application, retransmit plaintext retention and `pto_probe`; current R9-3 handoff contract. This is a source/invariant challenge only. No code was changed and no local gate was run by this reviewer note.

## Challenged invariant

For one stable Carrier `FrameId`, retransmission may create overlapping packet copies. Packet copies may remain independently tracked until ACK/loss resolution for packet-level RTT/congestion/loss accounting, and retained plaintext may remain owned until the final copy retires. However, once **any** packet copy carrying that stable frame is positively Carrier-ACKed, the frame has packet-recovery receive evidence and must no longer be eligible for another PTO/loss-triggered retransmission. A later loss of another overlapping copy must not resurrect retransmission work for a frame already positively ACKed.

This is Carrier-local recovery truth only. It does not create or substitute for Session DeliveryAck evidence.

## Exact-current contradiction

`Recovery` currently uses one `outstanding_frames: BTreeMap<FrameId, usize>` for both copy lifetime and retransmit eligibility:

1. `on_sent` increments the count for every packet copy carrying a frame.
2. ACK retirement decrements/removes the count, but does not remember that the frame has been positively ACKed.
3. Loss retirement inserts the frame into `RecoveryResult.retransmit_frames` whenever `release_frame` removes the final outstanding count.
4. `on_pto` selects directly from the keys of `outstanding_frames`.

That makes two concrete bad paths possible.

### Same-ACK-call closure

- original packet `P0` carries `F`, then a PTO retransmission `P1` carries the same stable `F`;
- ACK `P1` at a time where the old `P0` simultaneously crosses the time-loss threshold;
- ACK retirement decrements `F` from two copies to one;
- loss retirement of `P0` removes the final copy and therefore emits `F` in `retransmit_frames`;
- result: `P1` is positively ACKed, `P0` is correctly classified lost, but the already-received frame is nevertheless scheduled again.

`ReliableUdpRuntime::apply_ack` then treats `retransmit_frames` as scheduled work and intentionally does not release its retained plaintext. R9-3 therefore cannot truthfully prove “fresh retransmission ACK + suppressed original lost + final recovery closure” without either a spurious new retransmission candidate or stale retained plaintext.

### Split-call closure

If `P1` is ACKed before `P0` reaches the loss threshold, current copy accounting deliberately leaves `F` present because `P0` is still outstanding. `Recovery::on_pto` reads the same key set, so it may schedule `F` again even though `P1` already provided positive packet-recovery evidence. When `P0` is later declared lost, final-copy removal can schedule `F` yet again.

The existing `overlapping_retransmission_keeps_frame_outstanding_until_all_copies_ack` test explicitly expects `on_pto(1)` to return the frame after one overlapping copy has been ACKed. That assertion is the discriminating stale behavior to change; final-copy retention and packet-copy tracking need not be removed.

## Classification

This is a concrete current-engine correctness/resource defect, not a maintainer policy or architecture decision:

- M2 already separates packet copies from stable frame retransmission identity;
- packet ACK is authoritative Carrier-local receive evidence;
- PTO schedules frames but does not declare packets lost;
- Session DeliveryAck remains separate and unchanged;
- no new timer/capacity/security value is required.

The defect must close before R9-3 expands the cross-process recovery surface, because R9-3 intentionally creates exactly this original/retransmit overlap and requires the retransmitted copy ACK plus loss closure of the suppressed original.

## Smallest repair contract

Preserve authoritative packet-copy tracking while separating **copy lifetime** from **frame retransmit eligibility**. Exact type/API naming is non-normative. A bounded per-frame state such as `{ outstanding_copies, carrier_acked }` or an equivalently bounded ACKed-frame set is acceptable if it stays derived from the existing Recovery ownership.

Required semantics:

1. any positively ACKed packet containing frame `F` marks `F` positively received in the Carrier recovery domain;
2. while another packet copy of `F` remains outstanding, packet state and (if the current ownership contract requires it) retransmit plaintext may remain retained for final-copy accounting;
3. `on_pto` must not return `F` after any overlapping copy of `F` was positively ACKed;
4. loss of a packet carrying `F` may emit `F` in `retransmit_frames` only if no overlapping copy of `F` has been positively ACKed;
5. when the last outstanding copy retires, prune any per-frame positive-ACK marker so the new state is bounded by active Recovery ownership;
6. do not eagerly erase other packet copies merely because one copy ACKed: their packet-level loss/congestion accounting remains authoritative;
7. do not change Session DeliveryAck, Carrier ACK encoding, crypto/wire framing, packet numbering, D019, capacity values or release policy.

## Required focused regressions

1. **same-call ACK + time-loss:** old original `P0(F)`, fresh retransmission `P1(F)`; ACK `P1` at a timestamp that time-losses `P0`. Require `acked_packets=[P1]`, `lost_packets=[P0]`, `retransmit_frames=[]`, final `frame_outstanding(F)==false`; at runtime level retained plaintext is releasable and a later `pto_probe` is empty.
2. **split-call ACK then later loss:** ACK `P1` while `P0` remains outstanding. If final-copy retention is preserved, `frame_outstanding(F)` may remain true, but `pto_probe` must return no `F`. Later loss retirement of `P0` must still return no retransmit frame and must release final retained plaintext.
3. **all-copies-lost control:** when no copy of `F` has ever been positively ACKed, final loss closure must schedule `F` for retransmission exactly once.
4. Preserve existing future/never-sent ACK atomic rejection, packet loss accounting, cwnd byte release and stable `FrameId` semantics.

No decoder/parser/crypto-framing change is required for this repair, so decode fuzz is not mechanically required. After the repair, run focused tests plus the normal clean exact-tree developer gate, then continue directly into the existing R9-3 monotonic-clock/PTO cross-process slice.

## Queue effect

`H-R9-040` becomes the first dependency-ready local repair. Existing R9-3 PTO/clock implementation remains immediately behind it, followed by R9-4 through R9-12, dedicated independent R9 review and Q10/Q11/Q12 factual reconciliation. `READY_LIVE` remains `none`; this finding creates no unresolved real-network question.
