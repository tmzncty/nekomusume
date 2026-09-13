# Independent bounded SessionRuntime lifecycle/resource review — exact `8e11de0`

> **SUPERSEDED (one sentence):** the terminal-cleanup claim under "Reset/close/failure release state consistently" is incorrect for exact `8e11de0` — the `tick()` idle-timeout and close-deadline paths did **not** release `received`/`confirmed`/inflight/window state there. See [`independent-session-runtime-8e11de0-20260913-SUPERSEDED.md`](independent-session-runtime-8e11de0-20260913-SUPERSEDED.md); the defect was repaired at exact `9697ee7`. All other invariants in this note remain accurate for `8e11de0`. The `SessionRuntime.events` retained-state gap remains `POLICY_BLOCKED_RESOURCE_BOUND`.

Bounded independent review of `SessionRuntime`, `RuntimeLimits`, stream/window/inflight/queue accounting, and `delivery_ack` in `crates/neko-session/src/lib.rs` (`:828-1621`), against `docs/spec/m0-session-state.md` and `docs/specs/nekomusume-session-v0.md`, at reachable exact `8e11de0e5915efa43591069172b97f311aae2c6e`. Not a WAN/live claim, not an independent security/release approval.

## Challenged invariants and result

- **Hard limits validated before state allocation/mutation — holds.** `open_stream`, `queue_send`, `receive`, and `delivery_ack` all check `max_streams`/`max_record_bytes`/`max_queue_records`/`max_queue_bytes`/`max_total_bytes`/window caps with `checked_add` **before** any map/queue insert or counter mutation (`:1283-1286`, `:1342-1374`, `:1408-1466`, `:1502-1520`).
- **Stream/session window accounting released exactly once — holds.** `send_inflight`/`session_send_inflight` are incremented at `queue_send` and decremented only at `delivery_ack` (`:1383-1384`, `:1522-1523`); `pop_send` decrements `queued_bytes` only, so an in-flight record keeps occupying its window until ACKed — a self-consistent (conservative) accounting where queueing reserves the window. `recv_window_used`/`session_recv_window_used` are incremented at `receive` and decremented at `pop_receive` (`:1480-1482`, `:1538-1543`).
- **Duplicate data cannot consume unbounded retained state or advance delivery — holds.** `receive` rejects `offset != next_receive` and duplicate-but-divergent bytes as `Protocol`; an exact duplicate hits the `received` dedup map and returns `Ok(())` with a `DuplicateDedup` event **without** enqueueing or advancing `next_receive` (`:1423-1432`).
- **`DeliveryAck` cannot ack unsent/invalid ranges — holds.** `end = offset+len` must satisfy `delta = end - current <= inflight` and `<= session_send_inflight` (`:1505-1519`); `end < current` is rejected as `Protocol`. An ACK for a never-sent range inflates `delta` beyond real in-flight bytes and is rejected.
- **Rejected transitions are atomic — holds.** Every `Err` path returns before mutating streams/queues/counters; `check`/`touch` ordering keeps `last_activity_ms` updates consistent.
- **Reset/close/failure release state consistently — holds.** `close_remote`, `cancel`, and idle/deadline `tick` clear `send`, `recv`, `received`, `confirmed`, `send_inflight`, `recv_window_used`, `session_*`, and `queued_bytes` together (`:1254-1271`, `:1557-1595`).

## Boundary finding — `events` audit log has no retained bound (DEFER_POLICY)

`self.events: Vec<RuntimeEvent>` is appended on every operation (`:1613-1621`) and is only ever read via `events()`/`observable_events()` — it is never drained and has no capacity limit, while every other retained surface (`send`, `recv`, `received`, `confirmed`, window/inflight counters) is bounded by an existing `RuntimeLimits` value. An arbitrarily long-lived `SessionRuntime` accumulates an unbounded audit log, so a bounded stream of operations can create an unbounded number of retained objects.

This is a genuine retained-state bound gap, but the corrective value is a **new capacity/policy choice** (an event-log cap or a drain contract) that no committed `RuntimeLimits`/spec currently defines. Per the standing rule for missing caps that require policy selection, this finding is classified `DEFER_POLICY_OR_ENVIRONMENT` for maintainer judgment rather than repaired with an invented constant. No committed invariant is contradicted today; `events` is internal audit evidence, not Session delivery evidence.

## Evidence

- `cargo test -p neko-session` on exact `8e11de0`: all runtime/ledger/datagram/process tests pass.
- No code change in this slice (the one finding is policy-deferred). No new `READY_LIVE` question; release/governance state unchanged.
