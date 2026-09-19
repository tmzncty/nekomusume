# Independent R9-11A terminal-oracle re-challenge — H-R9-080 reopened

Review anchor: reachable `main` exact `215bcad121e5be58e62ab2685cd45cf68155839f`. New developer-owned commits since the prior reviewer handoff are exact `c6f772b9f0f6b0bedf0a2e8b17f31bbfce151b43` (SessionRuntime cleanup tests), exact `ecb3330337b7809f893504d64432269ad2f32a49` (restore displaced `#[test]` attributes), and exact `215bcad121e5be58e62ab2685cd45cf68155839f` (developer handoff closure).

Scope is deliberately narrow: verify whether the H-R9-080 regression contract is actually mutation-sensitive on the cancel/Error and remote-close paths. This review does not change Session/Carrier/ACK/wire/crypto architecture, D019, any capacity/TTL/LRU/history value, WAN policy, or release authority.

## Accepted source status

The shared implementation remains correct in shape on this tree: `SessionRuntime::cancel()`, `close_remote()`, idle timeout, and graceful-close deadline terminalization all call `clear_runtime_state()`, which clears send/recv queues, receive dedup history, confirmation watermarks, per-stream/session window accounting, queued bytes, stream ownership, and `close_deadline_ms` while intentionally preserving documented lifetime facts.

This finding therefore does **not** claim a current runtime leak. It is a release-gate evidence/closure-truth finding about what the new regressions actually prove.

## H-R9-080 — REOPENED HIGH: the new path oracles are only partially mutation-sensitive

The prior accepted H-R9-080 contract required the cancel/Error and remote-close fixtures to **populate the runtime-owned surfaces before terminalization**, then prove cleanup and evidence behavior. Exact `c6f772b`/`ecb3330` adds useful tests, but the closure claim in exact `215bcad` is stronger than the tests.

### Cancel/Error fixture

`repeated_cancel_is_idempotent_no_new_error_events` currently opens one stream and calls only `queue_send("aa")` before the first cancel. Before terminalization this populates the stream map, send queue, `send_inflight`, `session_send_inflight`, and `queued_bytes`, but it does **not** populate:

- `recv`;
- `received` dedup history;
- `confirmed` watermark state;
- `recv_window_used`;
- `session_recv_window_used`;
- an armed `close_deadline_ms`.

The post-cancel assertions that `recv`, `received`, and `confirmed` are empty are therefore vacuous for this path. The test also does not assert `recv_window_used.is_empty()`, `session_send_inflight == 0`, `session_recv_window_used == 0`, or `queued_bytes == 0`, even though the developer handoff says the test proves queue/dedup/watermark/window cleanup.

### Remote-close fixture

`remote_close_releases_stream_and_timer_ownership` has the same precondition problem: it opens one stream and calls only `queue_send("aa")`. Receive/dedup/confirmed/receive-window state and the close deadline are never non-empty before `close_remote()`, so several cleanup assertions cannot detect a path-specific regression.

The test checks the event count and terminal `SessionClosed` **before** the repeated `close_remote(6)` call, but it does not assert that the repeated close leaves the event count unchanged. It then asserts only that a post-terminal `queue_send` returns `Err`; it does not assert that the rejected mutator appended no fresh success/delivery/window evidence. Thus the closure sentence “idempotent repeat + post-terminal fail-closed” is only partially encoded in the oracle.

### Why this remains a HIGH review-truth issue

A shared helper makes the current source implementation look correct today, but item 4 is explicitly an independent release/security review gate. A closure oracle should fail if either terminal path later bypasses or selectively omits one owned surface. The current tests would stay green for several such path-specific regressions, while the handoff already records H-R9-080 as fully closed.

This is the same accepted finding contract, so H-R9-080 is **reopened**, not replaced with a new architecture requirement.

## Smallest accepted repair

Do not redesign lifecycle semantics. Make the two existing path tests mutation-sensitive, preferably with one small test helper that prepares the same bounded pre-terminal state for both paths.

A minimal fixture can stay within current test limits while proving every surface is live before terminalization:

1. open stream 1;
2. `queue_send("aa")` and `queue_send("bb")`;
3. `pop_send()` one record so queue-byte capacity remains available while send in-flight ownership remains;
4. `delivery_ack(stream=1, offset=0, len=2)` so `confirmed` is non-empty while the second send remains in-flight;
5. `receive(offset=0, "xy")` so `recv`, `received`, `recv_window_used`, and session receive-window ownership are non-empty;
6. `close_graceful()` to prove an actually armed `close_deadline_ms` is released;
7. explicitly assert those preconditions are non-empty/non-zero before calling `cancel()` or `close_remote()`.

After each terminal operation, assert:

- `streams`, `send`, `recv`, `received`, `confirmed`, `send_inflight`, `recv_window_used` are empty;
- `close_deadline_ms.is_none()`;
- `session_send_inflight == 0`, `session_recv_window_used == 0`, `queued_bytes == 0`;
- intended lifetime facts such as `total_bytes` remain retained;
- exactly one path-appropriate terminal event is added;
- repeated cancel/remote-close is idempotent with unchanged observable-event count;
- representative rejected post-terminal mutators cannot append fresh success/delivery/window evidence.

Keep idle-timeout and graceful-close-deadline positive controls green. Do not choose a `SessionRuntime.events` retention cap or any other policy number. No decoder/parser/crypto-framing change is required, so there is no mechanical decode-fuzz requirement.

Run focused `neko-session` tests, then persist developer-local clean exact-tree provenance for the final pushed source/test SHA: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, exact GitHub-resolvable SHA, UTC start/end, exit codes, OS/arch, and stable Rust.

After H-R9-080 is truthfully closed, continue immediately to R9-11A remainder, then the preserved R9-11B/C/D -> R9-12 -> final independent R9 -> Q10/Q11/Q12 -> repository-wide item-4 refill queue. `READY_LIVE` remains `none` absent a new real-network-only question.