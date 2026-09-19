# Independent R9-11A SessionRuntime terminal ownership challenge — H-R9-079

Review anchor: reachable `main` exact `ee623386192e3892b8001a8d4617ce76e076d84b`. Latest developer-owned source/test commit reviewed is exact `442a8058e4ae27f867bd27db632494a945ec30f2`, on top of `dd7afc6a1af69f5722250fa475fef805fd330c31`. H-R9-078 remains closed: repeated `cancel()` on terminal `Error` is idempotent and the restored tests are active.

Scope is the R9-11A SessionRuntime terminal cleanup remainder only: `SessionRuntime::{close_remote,cancel,tick,clear_runtime_state,check}`, `RuntimeStream`, `close_deadline_ms`, and current lifecycle regressions in `crates/neko-session/src/lib.rs`. No Session/Carrier/ACK/wire/crypto architecture, D019, resource-cap value, WAN policy, or release authority is changed.

## H-R9-079 — HIGH release-gate correctness/resource-lifecycle finding

`clear_runtime_state()` claims to release every runtime-owned in-progress state on terminalization, while preserving only explicitly named lifetime/cumulative facts. It clears send/recv queues, dedup state, confirmation watermarks, per-stream/session window state and queued bytes, but it leaves two live-session ownership surfaces behind:

1. `streams: BTreeMap<StreamId, RuntimeStream>` survives terminalization with each stream's mutable `state`, `next_send`, and `next_receive` values;
2. `close_deadline_ms: Option<u64>` survives graceful-close deadline terminalization (and any other terminal path entered after a deadline was armed).

These are not listed among the intentionally retained lifetime facts (`total_bytes`, `last_activity_ms`, events, `next_event`, `cancelled`). The R9-11A contract explicitly requires remaining non-lifetime stream/timer ownership to be challenged rather than assuming `clear_runtime_state()` is exhaustive.

The current regressions can therefore report "releases_all_runtime_owned_state" while missing exactly these owners: `idle_timeout_releases_all_runtime_owned_state` and `close_deadline_releases_all_runtime_owned_state` assert queues/dedup/watermarks/window counters but do not assert `streams.is_empty()` or `close_deadline_ms.is_none()`. `repeated_cancel_is_idempotent_no_new_error_events` likewise does not prove the first cancel released stream ownership.

This is bounded by existing stream limits and the terminal guards make the stale state behaviorally inert to ordinary mutators, so this finding does not claim an unbounded exploit. It is nevertheless a release-gate correctness/resource-lifecycle defect: terminal state currently retains mutable live-session ownership that repository comments/tests claim has been released, and the timer owner remains armed in representation after the session is terminal.

## Smallest accepted repair contract

Keep all committed Session semantics and all policy values unchanged.

1. On every terminalization path that calls `clear_runtime_state()`, release stream/timer ownership as well: clear the runtime stream map and disarm `close_deadline_ms` (or an equivalent smallest implementation that proves those owners cannot survive terminalization).
2. Add focused deterministic regressions for at least cancel/Error, remote close, idle timeout, and graceful-close deadline. Populate a stream before terminalization and, for the deadline path, prove the deadline is armed before expiry. After terminalization prove:
   - `streams` is empty;
   - `close_deadline_ms` is `None`;
   - existing send/recv/dedup/watermark/window/queued-byte cleanup remains true;
   - lifetime facts intentionally documented as surviving still survive;
   - exactly the expected terminal event is emitted and no false delivery/window/success evidence appears.
3. Preserve already committed idempotence/fail-closed behavior for repeated terminal calls and post-terminal mutators; do not invent a new error code.
4. Do not use this repair to choose a capacity for the intentionally retained `SessionRuntime.events` history. That remains the existing separate policy gate.
5. Run focused `neko-session` tests, then on the final pushed developer source/test SHA run and persist the required clean exact-tree gate: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean worktree, exact GitHub-resolvable SHA, UTC start/end, exit codes, OS/arch, stable Rust. No wire decoder/parser/crypto-framing change means no mechanical fuzz requirement.

After H-R9-079 is repaired and gated, continue immediately with the remaining R9-11A post-terminal API challenge, then R9-11B Recovery/reliable-UDP terminal ownership, R9-11C retained Session replay/Carrier-generation terminality, and R9-11D executable process/socket lifecycle/result truth. `READY_LIVE` remains `none`; this finding creates no new real-network question.
