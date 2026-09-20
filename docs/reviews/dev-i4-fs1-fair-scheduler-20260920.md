# Developer bounded I4-FS1 review — FairScheduler queue/fairness current owner

**Anchor:** exact source/test tree `8cbd9af` + reviewer commits through `9d36c27`.
**Owner:** `FairScheduler` (`open`/`enqueue`/`next_frame`/`snapshots`), `FlowLimits`,
`StreamPriority`, `INTERACTIVE_BURST`, in `crates/neko-carrier/src/lib.rs`.

## Invariants checked

| Invariant | Coverage |
|---|---|
| enqueue/dequeue/remove/empty transitions consistent | `enqueue` checked-adds `session_bytes`/`stream.queued_bytes` before mutating (reject-before-mutate); `next_frame` pops the front record and decrements both counters exactly; there is no stream remove — `open` is idempotent on an existing id and `queued_bytes==0` makes a stream scheduler-inert |
| fairness rotation | `cursor` advances round-robin over `order`; `INTERACTIVE_BURST=3` prefers Interactive then falls back to Bulk; `find(None)` pops any stream when the preferred priority is empty, so a queued frame is never starved indefinitely |
| closed/terminal owner exclusion | `next_frame` skips streams with `queued_bytes==0`; `session_bytes`/`queued_bytes` return to 0 when all queued frames drain; a stream that is never re-enqueued contributes nothing |
| deterministic queue accounting | `session_bytes` and per-stream `queued_bytes` are decremented on every pop and incremented on every enqueue — counters cannot drift apart because the same code path updates both |

## Reachable regressions named

- `fair_scheduler_interactive_prefers_bounded_burst_then_bulk` — interactive
  burst then bulk rotation.
- `fair_scheduler_rejects_limits_atomically` — enqueue reject-before-mutate
  on stream/session limits.
- `fair_scheduler_round_robins_and_reopens` — round-robin + idempotent open.

## Result

No concrete defect found in FairScheduler queue/fairness. Enqueue is
reject-before-mutate, counters stay consistent, rotation is deterministic,
and empty/closed streams cannot produce frames.

**READY_LIVE: none** — deterministic local evidence only.
