# Developer bounded R9-5 review — adversarial Carrier feedback / every continuation

**Anchor:** exact source/test tree `7ea9a84` + reviewer commits through `67028be`.
**Scope:** `UdpAcknowledgement::Carrier` continuations — initial receive
(`recv_udp_delivery_ack` demux), settlement phase, post-return owner —
`ReliableUdpRuntime::apply_ack` / `Recovery::on_ack`, packet-ACK tracker,
and all current adversarial fault seams.

## Per-continuation coverage (all dependency-ready, all reachable)

| Feedback class | Continuation | Executable seam | Regression |
|---|---|---|---|
| stale / duplicate | settlement | `--send-stale-ack-late` | `reliable_udp_stale_ack_settlement_phase_is_accepted_empty` (3034) |
| stale / duplicate | post-return | `--send-stale-ack` | post-return stale test (4187): exactly 1 `r9_udp_return_packet_ack_accepted_empty`, 0 rejected |
| future / never-sent | post-return | `--send-future-ack` | post-return future test (4346): `r9_udp_return_packet_ack_rejected` == 1, atomic |
| order-only (ACK before DeliveryAck) | post-return | `--reverse-post-return-ack` | `reliable_udp_post_return_reversed_ack_order_settles` (3966) |
| ACK-loss + delayed/reorder | post-return | `--delay-r9-ack-reorder` | `reliable_udp_ack_loss_delayed_original_reorder_settles` (3802) |
| malformed / tampered | initial+settlement | bounded malformed budget | `reliable_udp_malformed_budget_persists_across_carrier_ack` (2568) |
| multi-range projection | all | `acked_packets` per retired pn | H-R9-048 (e.g. `packet_ack_applied` names every retired pn) |
| aborted pre-send reservation | runtime | `abandon_retransmit` | H-R9-052/053/054 regressions — aborted pn is not ACK-valid, watermark restored |

## Contract checks

1. **Positive retirement** — every `packet_ack_applied` names the actual
   retired pn set (`acked_packets`), never a representative (H-R9-048).
2. **Sent-and-retired historical** — accepted-empty where committed
   semantics permit; H-R9-025/026/029 typed classes.
3. **Future / never-sent** — `largest > largest_sent` rejected atomically
   (Candidate A closed); post-return `rejected` event + no RTT/PTO/loss/Reno
   mutation.
4. **Aborted pre-send reservation** — `Recovery::abandon_sent` +
   `watermark_on_reserve` restore committed high-water (H-R9-052/053/054);
   a never-sent pn is not ACK-valid history.
5. **Malformed/tampered** — crypto unopen fails before any demux; malformed
   wire decode is bounded `malformed` budget, atomic.
6. **Multi-range** — every retired packet projected.
7. **Session vs Carrier** — separate evidence domains; Session
   `accepted_empty_logical_ack` never substitutes Carrier evidence.

## Result

No concrete defect found across any current `UdpAcknowledgement::Carrier`
continuation. All seams and regressions are reachable at `7ea9a84..67028be`.
Rejected feedback is atomic; multi-range projects every retired pn;
aborted reservations stay non-ACK-valid.

**READY_LIVE: none** — deterministic local evidence only; no new
real-network hypothesis.
