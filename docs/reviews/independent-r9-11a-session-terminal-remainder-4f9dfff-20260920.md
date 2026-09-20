# Independent bounded review — R9-11A SessionRuntime terminal cleanup remainder

Reviewed tree: reachable `main` exact `4f9dfff853642b05cab9ab57b6440194fa8275c6`.

Scope: `crates/neko-session/src/lib.rs` `SessionRuntime` terminal lifecycle after H-R9-079/H-R9-080, plus the current terminal regression fixture. This is item-4 review support only; it is not a release/security approval and does not close `SessionRuntime.events` retained-state policy.

## Claim challenged

After transition to `Closed` or `Error`, runtime-owned mutable session state must be released; mutating APIs must either fail closed or be explicitly idempotent; rejected/idempotent terminal calls must not manufacture fresh success/delivery/window evidence. Documented lifetime/cumulative facts may survive.

## Exact-current owners inspected

- `SessionRuntime::{close_remote,open_stream,close_stream,queue_send,pop_send,receive,delivery_ack,pop_receive,close_graceful,cancel,tick}`
- `SessionRuntime::{clear_runtime_state,check,event}`
- `SessionRuntime::{events,observable_events,stream_state,confirmed_watermark,total_bytes,queued_bytes,queued_records}` read-only projections
- `runtime_tests::{populated_runtime,repeated_cancel_is_idempotent_no_new_error_events,remote_close_releases_stream_and_timer_ownership}` and the existing idle-timeout / graceful-close-deadline cleanup controls
- provisional Session-v0 delivery-evidence boundary and current D064 Session-vs-Carrier separation

## Challenge result

No new concrete correctness/security defect found in this bounded remainder.

1. Every ordinary mutating SessionRuntime path inspected enters `check()` before state mutation; `check()` rejects `Error/cancelled` as `Cancelled` and `Closed` as `Terminal`. `close_remote()` is explicitly idempotent only for already-`Closed`; on `Error` it still fails through `check()`. `cancel()` and `tick()` are explicitly idempotent on both terminal states and append no additional terminal event.
2. All four terminalizing paths share `clear_runtime_state()`. It clears send/receive queues, receive dedup ownership, confirmed watermarks, per-stream/session send and receive accounting, queued bytes, stream mutable state, and the armed close deadline. The documented lifetime survivors (`total_bytes`, `last_activity_ms`, event log/sequence and `cancelled`) are not cleared.
3. The exact-current H-R9-080 fixture is now mutation-sensitive: it queues two sends, crosses the send-drain boundary with `pop_send()` on the first, ACKs only that first range, keeps positive remaining send accounting, populates receive queue/dedup/window state, and arms graceful-close deadline before terminalization. Cancel/Error and remote-close regressions assert cleanup from those positive preconditions, preserve `total_bytes`, hold terminal-event count stable on repeated terminal calls, and reject a representative ordinary post-terminal mutator without fresh observable evidence.
4. Idle-timeout and graceful-close-deadline terminalization remain covered by the same shared cleanup owner; no path-specific bypass was found in the current source.
5. Read-only projections after cleanup may return terminal/empty historical views (`UnknownStream`, zero watermark, retained lifetime counters/events) but do not mutate or create delivery evidence.

## Focused negative reasoning

- `Closed -> queue_send/open_stream/receive/delivery_ack/pop_* / close_stream / close_graceful` cannot pass `check()`.
- `Error ->` the same ordinary mutators cannot pass `check()` because `cancelled || Error` is tested first.
- repeated `cancel`, terminal `tick`, and repeated `close_remote` on `Closed` return without event growth or state resurrection.
- idle timeout reached through `check(now)` terminalizes first and returns `IdleTimeout`; the caller cannot continue its mutation after that error.

## Evidence / execution boundary

Developer-reported clean exact-tree provenance for source/test exact `052b6eab40f8e4257d4480866b21f64ce3e5230c` is retained in `docs/CHATGPT_HANDOFF.md`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, Linux x86_64, rustc 1.98.0. GitHub exposed no hosted status/workflow run for that exact SHA; absence is not failure evidence.

No reviewer-local test execution is claimed for this review: the reviewer used GitHub repository/source truth and inspected the exact-current deterministic regression owners. No wire decoder/parser/crypto-framing code changed, so no fuzz claim is made.

## Exclusions / still open

- `SessionRuntime.events` retained-state capacity remains `POLICY_BLOCKED_RESOURCE_BOUND`; this review does not select a cap, TTL, LRU or history policy.
- Recovery/reliable-UDP terminal ownership belongs to R9-11B.
- retained Session replay / Carrier-generation terminality belongs to R9-11C.
- process/socket lifecycle and result truth belong to R9-11D.
- no WAN/live/performance/release conclusion is added.

Classification: bounded independent no-finding review support. Continue immediately to R9-11B; do not wait for reviewer cadence.
