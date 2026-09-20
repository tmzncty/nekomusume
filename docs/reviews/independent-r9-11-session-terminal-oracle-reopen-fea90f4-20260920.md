# Independent R9-11 SessionRuntime terminal-oracle review — H-R9-080 reopened after fea90f4

**Reviewed reachable tree:** `cbe8c8e75082416c9d7f0eead5892995651ee66e`

**Developer source/test candidate under review:** `fea90f4808c5de757b9a371a7e0465472035acb5` on top of `eeec59f5b2f877ebb1dc2b9b1b569d5fc513b324`

**Scope:** bounded independent review of the remaining H-R9-080 cancel/Error + remote-close terminal-cleanup oracle contract. This note does not claim reviewer-local test execution, release completion, WAN evidence, protocol freeze, or production readiness.

## Accepted progress

The positive accounting gap from the prior review is repaired: the shared fixture now queues two sends, acknowledges only the first, and separately proves positive `session_send_inflight`, per-stream `send_inflight`, session/per-stream receive-window ownership, non-empty send/recv/dedup/confirmed state, positive queued bytes, and an armed close deadline before terminalization. The source cleanup remains shared through `clear_runtime_state()`.

## Finding — HIGH (release-gate evidence / closure truth)

The repository handoff now marks H-R9-080 fully CLOSED, but two explicit parts of the accepted repair contract are still missing, plus the fixture still skips the requested sent-record transition:

1. **No documented lifetime-survivor oracle.** The accepted contract required snapshotting a survivor such as `total_bytes` before terminalization and proving it remains unchanged after first cancel/Error and remote close. Current `populated_runtime()` does not snapshot one, and neither terminal regression asserts survivor preservation.
2. **Cancel/Error has no representative rejected post-terminal mutator.** `repeated_cancel_is_idempotent_no_new_error_events` proves repeated `cancel()` does not append a second Error event, but it does not then call a normal mutator such as `queue_send`/`open_stream` and prove the call fails closed with no fresh success/delivery/window evidence. The remote-close path does have this negative control.
3. **The fixture never `pop_send()`s the first range before acknowledging it.** The prior accepted contract explicitly required one record to leave the send queue before `delivery_ack`, so the confirmation oracle represents a record that has crossed the runtime send-drain boundary while a second queued/outstanding record remains. Current exact `fea90f4` still acknowledges a range that remains in `send`. This is a test-evidence weakness, not a claim of a new source leak.

Current source still appears to route all four terminal paths through shared cleanup, so this finding does **not** claim that `clear_runtime_state()` currently leaks stream/timer/queue/window ownership. The HIGH is that release-item-4 closure truth is stronger than the mutation-sensitive regression actually proves.

## Minimal repair contract

Keep the existing source cleanup. Strengthen only the focused tests:

- In the shared fixture, queue two sends, `pop_send()` exactly the first, then acknowledge only that first range; keep separate positive assertions for remaining send accounting, receive-window ownership, non-empty owned maps/queues, queued bytes, and armed deadline.
- Snapshot `total_bytes` (or another already documented lifetime survivor) before terminalization; after first cancel/Error and remote close separately, assert the survivor is unchanged while runtime-owned mutable state is empty/zero.
- On the cancel/Error path, after proving repeated cancel event-count idempotence, execute one representative normal mutator and assert the typed terminal failure plus unchanged observable-event count. Do not invent a new error code; use current committed semantics.
- Keep the remote-close negative, idle-timeout control, and close-deadline control green.
- Do not alter `SessionRuntime.events` retention policy, D019, capacity/TTL/LRU/history/security values, or Session/Carrier/ACK/wire/crypto semantics. No decoder/parser/framing change means no mechanical fuzz requirement.

After the focused `neko-session` tests, run the final pushed source/test SHA through the required developer-local clean exact-tree gate: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, exact reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust. Then continue directly to the remaining R9-11A review without waiting for reviewer cadence.
