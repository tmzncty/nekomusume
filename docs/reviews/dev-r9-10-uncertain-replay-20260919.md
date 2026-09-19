# Developer bounded R9-10 review — uncertain classification / replay / dedup / cleanup

**Anchor:** exact source/test tree `d97a536` + reviewer commits through `6038fda`.
**Owners inspected:** `ConcurrentCarrierManager` — `track`/`track_uncertain`/
`confirm`/`replay_uncertain`/`finish_drain`/`mark_owner_uncertain`/
`ensure_uncertain_capacity`, `AssignedRange{bytes,owner,uncertain}`,
`LogicalRangeId` stable key, `PacketAckTracker`, `ReliableUdpRuntime`
torn-down gates.

## R9-10A — uncertain classification / replay-set ownership

| Invariant | Coverage |
|---|---|
| only ranges lacking Session delivery proof become Uncertain | `confirm(id)` **removes** the range from `ranges` — `mark_owner_uncertain` only flags still-present (unconfirmed) ranges; a confirmed range is never replayable |
| Carrier packet ACK alone cannot suppress Session replay; Session DeliveryAck is not packet retirement | separate domains — `confirm` is the Session-delivery ledger; Carrier `*_packet_ack_*` events are packet-level only (H-R9-024/048 per-pn projection) |
| ranges confirmed before the switch are excluded from replay | `confirm` removes the range before `mark_owner_uncertain`/`replay_uncertain` can reach it |
| overlapping/retransmitted UDP copies create no second Session-range owner | `LogicalRangeId{stream,offset}` is the stable replay key — retransmit copies share the same id; `AssignedRange` is single-owner |
| fail/drain/promotion cannot silently drop an unresolved range or manufacture uncertainty for a resolved one | `mark_owner_uncertain` flags only unconfirmed ranges of the failed owner; `replay_uncertain` replays `uncertain` ranges on the new active and reassigns ownership; `finish_drain` replays still-unconfirmed old-owner ranges at deadline |
| capacity rejection atomic/fail-closed | `ensure_uncertain_capacity` runs **before** `mark_owner_uncertain`/insert; `Capacity` rejection leaves ownership/ranges unchanged; `uncertain_duplicate_and_counter_boundaries_are_atomic`, `cooldown_and_uncertain_capacity_rejections_are_atomic` |

## Reachable regressions named

- `warm_switch_is_reason_coded_dwell_guarded_and_drains_uncertain` —
  replay_uncertain on promotion drains only uncertain ranges.
- `uncertain_duplicate_and_counter_boundaries_are_atomic`,
  `uncertain_limits_are_atomic` — capacity atomic/fail-closed.
- `cooldown_and_uncertain_capacity_rejections_are_atomic` — rejection does
  not partially move ownership.
- `hard_failure_switches_and_resends_uncertain_with_dedup` — dedup on stable
  LogicalRangeId.
- `udp_blackhole_recovers_over_tcp_without_loss_and_deduplicates` —
  cross-process no-loss + dedup.

## R9-10B/C scope

Cross-Carrier replay identity/dedup/conflict and replay cleanup/
partial-promotion negatives are covered by the same stable `LogicalRangeId`
single-owner key + `confirm` removal + `replay_uncertain`/`finish_drain`
reassignment; no second Session-range owner can arise from packet copies,
and a confirmed-before-switch range is unreachable by replay.

## Result

No concrete defect found across uncertain classification, replay-set
ownership, dedup identity, or cleanup/negative ordering. Confirmed ranges
leave the ledger; only unconfirmed old-owner ranges replay; capacity is
atomic.

**READY_LIVE: none** — deterministic local evidence only.
