# Independent R9-11 SessionRuntime terminal-oracle review — H-R9-080 reopened

**Reviewed reachable tree:** `15dd77d16a0975b6e150f24794de9ce7aa250314`

**Developer source/test candidate under review:** `8c3f96de1a0de3c102c09cb2a0ff87b6971dce0a`

**Scope:** bounded independent review of the H-R9-080 cancel/Error + remote-close terminal cleanup regressions only. This note does not claim reviewer-local test execution, release completion, WAN evidence, protocol freeze, or production readiness.

## Finding — HIGH (release-gate evidence / closure truth)

The `8c3f96d` regression is useful but still does not satisfy the accepted mutation-sensitive closure contract.

`populated_runtime()` currently queues exactly one two-byte send and then immediately calls `delivery_ack(stream=1, offset=0, len=2)`. Under current `SessionRuntime::delivery_ack`, that releases the entire two-byte send accounting before terminalization: `session_send_inflight` becomes `0`, and the `send_inflight[stream]` entry is decremented to `0`. The map entry can remain present, but it no longer represents positive in-flight ownership.

The precondition assertion

```rust
assert!(!r.send_inflight.is_empty() || !r.recv_window_used.is_empty());
```

therefore does not prove send-accounting ownership is live. It passes because receive-window ownership is positive. The post-terminal assertions `session_send_inflight == 0` and `send_inflight.is_empty()` are consequently not mutation-sensitive for the send-accounting value that `clear_runtime_state()` is supposed to release.

This is exactly why the prior accepted repair contract required two queued ranges, popping one and acknowledging only the first, so `confirmed` is non-empty **while a second send remains positively in-flight**.

Two additional closure claims are still under-proved on these two paths:

- the tests/comments say intended lifetime facts survive, but neither cancel nor remote-close asserts a representative survivor such as `total_bytes` before/after terminalization;
- the cancel/Error test proves repeated `cancel()` event-count idempotence but does not execute a representative rejected post-terminal mutator and prove that it appends no fresh success/delivery/window evidence. The remote-close test does have that negative control.

This does **not** claim a current production/source leak. Current terminal paths still share `clear_runtime_state()`. The HIGH is that the repository now marks H-R9-080 closed more strongly than the regression oracle proves, which matters for release item 4.

## Minimal repair contract

Keep the current shared source cleanup. Strengthen only the test oracle:

1. open stream 1;
2. queue two sends (`"aa"`, `"bb"`);
3. `pop_send()` exactly one record;
4. `delivery_ack(stream=1, offset=0, len=2)` so `confirmed` is non-empty while the second two-byte send remains in-flight;
5. receive one record so recv queue/dedup/per-stream and session receive-window ownership are non-zero;
6. arm `close_deadline_ms` with `close_graceful()`;
7. **before terminalization**, assert positive values separately, not via OR/map-presence only: at minimum `session_send_inflight > 0`, `send_inflight[stream] > 0`, `session_recv_window_used > 0`, `recv_window_used[stream] > 0`, non-empty send/recv/received/confirmed/streams, `queued_bytes > 0`, and armed deadline;
8. snapshot a documented lifetime survivor such as `total_bytes`;
9. run cancel/Error and remote close separately and prove all owned surfaces clear while that lifetime survivor remains unchanged;
10. on both terminal paths, prove repeated terminal call is event-count idempotent and a representative rejected post-terminal mutator adds no fresh evidence.

Keep idle-timeout and close-deadline controls green. Do not invent event-retention capacity, TTL/LRU/history/security values, alter D019, or change Session/Carrier/ACK/wire/crypto semantics. No decoder/parser/framing change implies no mechanical fuzz run.

After the focused `neko-session` regression is truthful, the developer must run the final pushed source/test SHA through the clean exact-tree gate required by the repository handoff: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust version. Then continue directly to R9-11A remainder.
