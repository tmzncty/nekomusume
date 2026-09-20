# Developer bounded R9-11B review — reliable-UDP / Recovery terminal ownership

**Anchor:** exact source/test tree `052b6ea` + reviewer commits through `3adcbd1`.
**Owners inspected:** `ReliableUdpRuntime::teardown` (torn_down flag +
packet_frames/largest_sent clear + retained plain reset + Recovery::quiesce +
PacketAckTracker reset), `Recovery::quiesce` (in_flight/lost/retransmits/
lost_ranges/plaintext zero, lifetime diagnostics preserved), `poll_outgoing_ack` /
`on_send` / `on_packet_received` / `on_ack` / `on_pto` / `send_plaintext` /
`ready_standby` / `activate_udp` / `manager_mut` / `poll_health` torn_down gates.

## Per-invariant coverage

| Invariant | Coverage |
|---|---|
| sent map / outstanding copies / Reno charge / retransmit plaintext / packet-frame map / receiver ACK obligations zero or terminal-inert | `teardown` clears `packet_frames`+`largest_sent`, resets `retained` plaintext to `None`, `recovery.quiesce()` zeroes `in_flight`/`lost`/`retransmits`/`lost_ranges`/`plaintext` and quiesces `congestion_window`/`recovery_entry`, and replaces `acks` with a fresh empty tracker |
| quiesce preserves only lifetime diagnostics, never stale retransmission eligibility or fresh health | `Recovery::quiesce` keeps `packets_sent`/`packets_lost`/`largest_lost`/`retransmits_sent`/RTT/PTO history counters; `retransmit_candidates` is empty; `last_health_*` None — no new health outcome can be manufactured |
| post-teardown mutators cannot manufacture positive evidence | `send_plaintext`/`on_send`/`on_packet_received`/`on_ack`/`on_pto`/`poll_outgoing_ack`/`fresh_health_sample`/`ready_standby`/`activate_udp`/`manager_mut` all return early/None after `torn_down`; `poll_health` → `Idle` |
| abort/teardown cannot orphan retained plaintext or make aborted packets ACK-valid | `retained` plaintext `None` post-teardown; `packet_frames` cleared — no stale retransmit candidate; ACK tracker fresh so a replayed ACK still decodes but never produces `packet_ack_applied`/retransmit for torn-down packets |
| future/never-sent ACK fail-closed before RTT/loss/PTO mutation | `on_ack` mutator path checks `largest_sent`; post-teardown `largest_sent` is None so an ACK has nothing to apply — fail-closed without RTT/loss/PTO mutation |

## Reachable regressions named

- `teardown_is_terminal_and_fail_closed_on_every_mutator` — packet_frames /
  largest_sent / retained empty, poll_outgoing_ack None, every mutator no-op.
- `teardown_preserves_lifetime_history_and_gates_control_plane` — lifetime
  Recovery diagnostics preserved; ready_standby/activate_udp/manager_mut/
  poll_health gated to Idle.
- `retransmit_requires_retained_ownership_and_teardown_is_deterministic` —
  no retained plaintext ⇒ no retransmit candidates after teardown.
- `dropped_retransmit_after_teardown_does_not_reenter_acks` — a post-teardown
  retransmit never re-enters ACK evidence.
- `new_generation_resumable_after_teardown` — a new generation builds fresh
  state on the new generation key.

## Result

No concrete defect found across Recovery/reliable-UDP terminal ownership.
Sender-side frames/retained/Reno charge, receiver ACK tracker, and retransmit
eligibility are all quiesced; only documented lifetime diagnostics survive;
every mutator is fail-closed or inert after `torn_down`.

**READY_LIVE: none** — deterministic local evidence only.
