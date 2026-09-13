# Superseded — `independent-session-runtime-8e11de0-20260913.md`

The prior note [`independent-session-runtime-8e11de0-20260913.md`](independent-session-runtime-8e11de0-20260913.md) remains the historical record, but one sentence in it is **incorrect for its claimed exact anchor `8e11de0`** and is superseded by the reachable repair.

## The incorrect sentence

Under "Reset/close/failure release state consistently", the note asserted that at exact `8e11de0` the `tick()` idle-timeout and close-deadline paths already cleared `received`, `confirmed`, per-stream `send_inflight`/`recv_window_used`, and session-level window counters consistently with `close_remote()`/`cancel()`.

That was wrong for `8e11de0`. On the exact `8e11de0` source, the two `tick()` terminal paths cleared only `send`, `recv`, and `queued_bytes`, leaving dedup history (`received`), confirmation watermarks (`confirmed`), and per-stream/session window accounting (`send_inflight`, `recv_window_used`, `session_send_inflight`, `session_recv_window_used`) retained until object drop.

## Correction

Reachable exact `9697ee7a0045b39c6ef46c1951fefea486ccf13b` (`fix(session): symmetric terminal cleanup on idle-timeout and close-deadline`) extracts a shared `clear_runtime_state()` and makes `tick()` idle-timeout and close-deadline release the same bounded runtime-owned state as `close_remote`/`cancel`, while preserving cumulative lifetime facts (`total_bytes`, the event log, `next_event`, `cancelled`). Focused regressions `idle_timeout_releases_all_runtime_owned_state` and `close_deadline_releases_all_runtime_owned_state` prove the symmetric cleanup on that exact tree. Developer-local exact-tree provenance for `9697ee7` is recorded in [`docs/local-gate-9697ee7-20260913.md`](../local-gate-9697ee7-20260913.md).

The historical `8e11de0` note is otherwise unchanged — its challenged invariants (limit-before-mutation, exactly-once window release, duplicate handling, DeliveryAck range checks, atomic rejection) remain accurate for `8e11de0` — only the terminal-cleanup sentence is superseded. The distinct `SessionRuntime.events` retained-state bound gap it also raised remains a real, separately classified `POLICY_BLOCKED_RESOURCE_BOUND` finding on current source.
