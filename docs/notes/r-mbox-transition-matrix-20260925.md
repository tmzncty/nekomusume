# R-MBOX transition matrix — injected impairment classes vs tested boundaries

**Date:** 2026-09-25. **Reconciled tree:** `483b55297d342f373616f46631639c07d0536d55`
(gate provenance: `docs/notes/check-gate-483b552-20260925.md`).

This is a factual reconciliation of every injected impairment class in
`FaultInjectCarrier` (`crates/neko-carrier/src/lib.rs`) against the current
Carrier/Session state and event expectations, the exact regression that
proves each mapping, and the explicit unresolved boundary. It makes no
policy decision, no release claim, and does not expand any seam. All
fixtures are deterministic in-memory Carrier-seam models; none is a
network-topology, directional-path, WAN, performance, or production claim.

## Drop-class primitives (send never reaches the inner carrier)

| Primitive | Deterministic behavior | Session/state expectation | Proven by | Unresolved boundary |
|---|---|---|---|---|
| `loss_percent` (seeded draw) | draw % 100 < percent drops the send | receiver never sees the record; sender-side ownership stays uncertain until existing confirm/PTO/replay semantics resolve it | `one_way_udp_reply_cessation_and_hard_loss_are_deterministic` (100%); CLEAN-RECOVERY chain via `loss_window` | partial-percent statistical behavior is not asserted beyond seeded determinism; no retransmission policy implied (H-I4-119) |
| `blackhole_after: N` | every send with index >= N drops | identical to loss from N onward | `deterministic_blackhole_loss_and_close_are_bounded` | permanent-from-N only; no recovery window (see `loss_window`) |
| `close_after: N` | same guard path as blackhole_after at the seam | n/a (combined with blackhole guard in `blocked()`) | `deterministic_blackhole_loss_and_close_are_bounded` (construction rejects loss_percent > 100) | close_after exercises the same blocked() branch as blackhole_after; a distinct half-open transport-close transition is not separately modeled |
| `one_way` | every even-indexed send drops (alternating) | models one-way reply cessation on ONE wrapped endpoint | `one_way_udp_reply_cessation_and_hard_loss_are_deterministic`; H-I4-121/122 chain narrowed its process-level counterpart to `--drop-post-auth-delivery-acks` | NOT a directional topology model (historical evidence boundary, unchanged); alternation is permanent |
| `max_payload_bytes: M` (R-MBOX-MTU) | send strictly longer than M drops | oversized datagram drop at a path boundary; no feedback of any kind | `mtu_boundary_drops_oversized_only`; `mtu_drop_is_drop_class_not_reorder_class` | drop-only: no fragmentation, no PTB/ICMP, no interface-MTU change, no PLPMTUD policy, no wire-framing change; exact-boundary passes |
| `loss_window: (start, end)` (R-MBOX-CLEAN-RECOVERY) | sends indexed in [start, end) drop; clean from `end` | bounded transient burst followed by a clean path; drives the existing track_uncertain/confirm/PTO-failover/tcp_resend/receive-dedup recovery chain end to end | `loss_window_drops_bounded_burst_then_clean_path`; `loss_window_drop_is_drop_class_not_reorder_class`; `clean_recovery_after_bounded_loss_replays_uncertain_without_duplicates` | empty window drops nothing; window semantics only — no jitter/rate variation model |

**Drop-class invariants, asserted directly:**
`reorder_with_loss_keeps_buffer_bounded_and_deterministic`,
`mtu_drop_is_drop_class_not_reorder_class`,
`loss_window_drop_is_drop_class_not_reorder_class` — a dropped send never
enters the reorder buffer and never releases a withheld record. Every
drop-class primitive shares this isolation.

## Delivery-class primitives (send reaches the inner carrier, possibly transformed)

| Primitive | Deterministic behavior | Session/state expectation | Proven by | Unresolved boundary |
|---|---|---|---|---|
| `reorder` (R-MBOX-REORDER-DELAY) | bounded one-record pending slot withholds one surviving record; the NEXT surviving record releases it after itself (adjacent-pair swap). **Swap atomicity (c389ec6 + 9c6208b):** a first-half failure (caller's record rejected, zero copies delivered) always restores the withheld record; a second-half zero-copy failure restores it cleanly for retry; a duplicate-mode one-copy partial failure leaves the slot empty (record under-delivered but in flight, error surfaced) so the documented exactly-two multiplicity is preserved under all failure combinations — never a silent loss | receiver observes pairwise-swapped delivery order; ordering assumptions are challenged without loss | `reorder_swaps_adjacent_pairs_and_is_bounded`; `reorder_swap_failure_restores_pending_atomically`; `reorder_swap_first_half_failure_restores_pending`; `duplicate_partial_delivery_never_exceeds_two_copies`; interaction tests with loss/MTU/window above | bounded to one withheld record by construction (`pending_len` 0/1); only adjacent-pair swaps, no wider permutation model; no reorder-impact-on-Session-queue claim (Session-level reorder tolerance is separately covered by existing SessionRuntime order-independent DeliveryAck semantics) |
| `duplicate` (R-MBOX-REORDER-DELAY) | exactly two consecutive copies of every surviving record at the delivery point | receiver-side exact-duplicate dedup must absorb the copy; `FailoverController::receive` returns Ok(false) once | `duplicate_emits_exact_multiplicity_and_ordering`; `duplicate_reorder_combined_multiplicity_and_ordering` | multiplicity is exactly 2 by fixture contract, not configurable; no burst-duplicate stress model |
| `delay_ms` | pre-existing send-time sleep | wall-clock lower bound only | `delay_ms_lower_bound_is_bounded_and_configured` (2×2ms >= 4ms, no upper bound) | no jitter/rate variation; upper bound intentionally unasserted (anti-flaky contract from the handoff) |

## Transition-ownership expectations (failover/recovery seams)

| Expectation | Proven by | Unresolved boundary |
|---|---|---|
| single active owner at every step (controller `active()` and manager `pending_switch` consistent) | `repeated_failover_cycles_keep_single_active_and_no_duplicates`; `repeated_transitions_converge_with_no_leaked_in_flight_state` | controller/manager duality is fixture-level; process-level ownership is proven by the existing CLI probe suite, not re-proven here |
| strictly monotonic generation across cycles (+1 per cycle; fail must pass the CURRENT generation, promotion produces generation+1, migration-back commits it) | same two regressions | warm-resume generation flow (D064 readiness) is exercised by existing manager tests, not repeated here |
| the hold gate is real: first migration-back after each promotion returns HoldGate with state unchanged; second commits | same two regressions | hold policy values unchanged (`min_hold_events` fixture-set to 1; no new hysteresis invented) |
| PTO window resets on healthy observation (`udp_progress`), making cycles independent | same two regressions | reset semantics themselves are pre-existing; no new timer policy |
| authoritative replay set is exactly the still-uncertain set at switch time (`tcp_resend`); replayed already-delivered records dedup to Ok(false) at the receiver; zero duplicate Session delivery | CLEAN-RECOVERY end-to-end + RESOURCE per-cycle drain assertions | `tcp_resend` is TCP-gated (WrongCarrier on UDP) — that guard is the single-owner invariant, asserted not bypassed |
| retained in-flight ownership returns to the empty baseline after every cycle; same small fixed capacities suffice for all cycles (no monotonic leak) | `repeated_transitions_converge_with_no_leaked_in_flight_state` | convergence proof is per-cycle emptiness + fixed-capacity reuse, not a capacity benchmark; no new limits invented |

## Explicitly NOT tested / NOT claimed here

- No fragmentation, PTB/ICMP feedback, interface-MTU change, or PLPMTUD
  behavior (R-MBOX-MTU is drop-only).
- No directional topology or one-way network model (`one_way` is
  alternating-send suppression on one wrapped endpoint — unchanged
  historical boundary).
- No WAN/middlebox observation, performance, capacity-pressure, or
  adversarial-load evidence.
- No production retransmission/timer/readiness/hysteresis/congestion
  policy: recovery chains reuse ONLY existing
  `track_uncertain`/`confirm`/`udp_pto_at`/`udp_progress`/`tcp_resend`/
  `receive`/manager-gate semantics. H-I4-119 (core ACK-PTO) and D019
  remain untouched maintainer seams.
- `--drop-post-auth-delivery-acks` (process seam, H-I4-121/122/123 closure
  chain) is listed for completeness: it suppresses Session DeliveryAck only
  and is proven by
  `executable_loopback_post_auth_delivery_ack_blackhole_drives_tcp_failover`
  and
  `reliable_udp_delivery_ack_suppression_replays_full_owned_set` in the
  neko-cli probe suite; it is NOT part of the in-memory matrix above.
- `READY_LIVE: none` unchanged; no VPS/live work is implied.

## Gate SHAs for this matrix

REORDER-DELAY `21c1a61` (+provenance `6687036`), MTU `f8d6ab4`
(`9943270`), CLEAN-RECOVERY `1cc267e` (`2eb0d9c`), grouped reconciliation
`7196012` (`7ed6b40`), REPEATED-FAILOVER `056366e` (`5e45e70`), RESOURCE
`483b552` (`b01843f`). All exact-tree gates exit 0 with clean trees at the
pushed SHAs; per-gate provenance notes live in `docs/notes/`.
