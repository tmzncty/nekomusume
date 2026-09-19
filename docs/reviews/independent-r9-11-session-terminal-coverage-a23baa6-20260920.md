# Independent R9-11A terminal-coverage challenge — H-R9-080

Review anchor: reachable `main` exact `a23baa6e451de99b40556b137abba27146b29ad0`. Latest developer-owned source/test commit reviewed is exact `271e512c01cefcfc40745aa41725bbd3e2585bb6`.

Scope is deliberately narrow: verify the claimed closure of H-R9-079 against its own accepted regression contract, then keep the remaining R9-11 lifecycle queue moving. This review does not change Session/Carrier/ACK/wire/crypto architecture, D019, any capacity/TTL/LRU/history value, WAN policy, or release authority.

## Source repair status

The source half of H-R9-079 is correct on exact `271e512`: `SessionRuntime::clear_runtime_state()` now clears `streams` and sets `close_deadline_ms = None`. Every existing terminalization path that calls `clear_runtime_state()` therefore inherits the stream/timer cleanup implementation.

The existing focused regressions added by `271e512` cover only two of the four terminal paths named in the accepted H-R9-079 contract:

- `idle_timeout_releases_all_runtime_owned_state` now asserts `streams.is_empty()` and `close_deadline_ms.is_none()`;
- `close_deadline_releases_all_runtime_owned_state` now asserts the same.

GitHub-hosted Rust CI run `35471440603` for exact `271e512` completed successfully. Developer-local clean exact-tree provenance is separately recorded in the handoff; hosted CI remains supplemental.

## H-R9-080 — HIGH release-gate evidence/closure finding

H-R9-079's accepted contract required deterministic terminal-cleanup regressions for **all four** paths: cancel/Error, remote close, idle timeout, and graceful-close deadline. Exact `271e512` does not add the cancel/Error or remote-close ownership oracles.

Current tests do not close that gap:

- `repeated_cancel_is_idempotent_no_new_error_events` proves repeated cancel does not append another Error event, but it does not prove the **first** cancel released `streams`, disarmed `close_deadline_ms`, preserved the existing queue/dedup/watermark/window cleanup, or emitted only the intended terminal evidence;
- `queue_limits_and_cancel_are_atomic_terminal_operations` checks the terminal state and post-cancel fail-closed calls, but not the stream/timer ownership surfaces;
- there is no focused `close_remote` regression in the current SessionRuntime test modules proving the same cleanup/evidence boundary.

Because the source repair is shared through `clear_runtime_state()`, this finding does **not** claim the current implementation still retains stream/timer state on those paths. It is an evidence/closure-truth HIGH: the repository marked H-R9-079 CLOSED even though its explicit mutation-sensitive regression contract was only partially implemented. A future path-specific bypass/regression in `cancel()` or `close_remote()` could therefore pass the currently claimed closure oracle.

## Smallest accepted repair

Do not redesign lifecycle semantics. Add only the missing focused deterministic regressions (or the smallest equivalent table-driven coverage) for:

1. **cancel/Error:** populate stream/runtime-owned state, arm a close deadline before cancel if useful to exercise timer ownership, call first `cancel()`, then prove `streams.is_empty()`, `close_deadline_ms.is_none()`, queues/dedup/watermarks/window counters/queued bytes are cleared, intended lifetime facts survive, exactly one terminal Error evidence is produced, and H-R9-078 repeated-cancel idempotence remains intact;
2. **remote close:** populate the same owned state, call `close_remote()`, prove the same cleanup plus exactly the expected SessionClosed evidence, then prove repeated close/post-terminal mutators remain idempotent or fail closed according to existing committed behavior with no fresh success/delivery/window evidence.

Keep the existing idle-timeout and close-deadline positive controls. Do not invent a new error code or a `SessionRuntime.events` retention cap. No wire/parser/crypto-framing change is required, so no mechanical decode fuzz requirement follows.

Run focused `neko-session` tests, then persist the required developer-local clean exact-tree gate on the final pushed source/test SHA: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, exact reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust.

After this narrow oracle repair, continue immediately with R9-11A remainder and the preserved R9-11B/C/D -> R9-12 -> final independent R9 -> Q10/Q11/Q12 -> repository-wide item-4 refill queue. `READY_LIVE` remains `none`.
