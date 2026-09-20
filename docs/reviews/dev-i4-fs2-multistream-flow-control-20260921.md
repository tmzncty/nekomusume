# Developer bounded I4-FS2 review — multi-stream + session/stream flow-control accounting

**Anchor:** exact source/test tree `8cbd9af` + reviewer/dev commits through `dab0be3`.
**Owners inspected:** `neko-session` `RuntimeLimits` / `open_stream` / `close_stream` /
`queue_send` / `delivery_ack` / `queued_bytes` / `recv_window_used` /
`session_recv_window_used`; `neko-carrier` `reliable_udp_runtime` and
`integration_gates` stream exercises.

## Per-invariant coverage

| Invariant | Coverage |
|---|---|
| stream/session limit interaction is enforced before mutation | `open_stream` rejects `InvalidStreamId`, `InvalidTransition` (duplicate), and `StreamLimit` before any insert; `hostile_runtime_limits_are_rejected_before_allocation` proves invalid `RuntimeLimits` never allocate |
| queued/inflight/released byte accounting is atomic | `queued_bytes`/`queued_records` are exact; `queue_send` enforces per-stream + session windows; `send_window_exhaustion_ack_release_and_resume_are_atomic_and_observable` proves exhaustion → ACK release → resume is atomic and observable |
| reject-before-mutate behavior | `rejected_limits_do_not_commit_context`; `delivery_ack_rejects_unknown_stream_and_missing_inflight_atomically`; `zero_length_delivery_ack_is_rejected_without_event` — no event emitted on rejection |
| duplicate/terminal paths | `duplicate_data_id_is_idempotent_and_conflict_is_rejected`; `session_layer_suppresses_exact_duplicates_and_fails_closed_on_conflict`; `queue_limits_and_cancel_are_atomic_terminal_operations`; close on already-Closed is idempotent no-op |
| cross-stream isolation | `streams_close_independently_and_invalid_or_exhausted_ids_are_bounded`; `receive_ordering_and_session_resource_limits_apply_across_streams`; `supports_stream_zero_one_and_two_plus_with_per_stream_ordering` |
| Session delivery evidence kept separate from Carrier packet feedback | `packet_recovery_events_are_layered_and_not_session_delivery` proves Carrier packet events do not masquerade as Session delivery |

## Reachable regressions named

- `streams_close_independently_and_invalid_or_exhausted_ids_are_bounded`
- `receive_ordering_and_session_resource_limits_apply_across_streams`
- `send_window_exhaustion_ack_release_and_resume_are_atomic_and_observable`
- `invalid_window_limits_are_rejected_without_state`
- `hostile_runtime_limits_are_rejected_before_allocation`
- `queue_limits_and_cancel_are_atomic_terminal_operations`
- `delivery_ack_rejects_unknown_stream_and_missing_inflight_atomically`
- `zero_length_delivery_ack_is_rejected_without_event`
- `session_layer_suppresses_exact_duplicates_and_fails_closed_on_conflict`
- `duplicate_data_id_is_idempotent_and_conflict_is_rejected`
- `packet_recovery_events_are_layered_and_not_session_delivery`

## Result

No concrete defect found across multi-stream + session/stream flow-control
accounting. Stream/session limits are enforced reject-before-mutate; queued,
inflight and released byte accounting is atomic and observable; duplicate and
terminal paths are idempotent/fail-closed; cross-stream isolation holds;
Session delivery evidence remains distinct from Carrier packet feedback.

**READY_LIVE: none** — deterministic local evidence only.
