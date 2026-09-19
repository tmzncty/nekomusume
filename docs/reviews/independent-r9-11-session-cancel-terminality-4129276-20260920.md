# Independent R9-11A SessionRuntime terminality challenge — H-R9-078 repeated cancel mutates terminal Error state

Review anchor: reachable `main` exact `4129276dd457ad3a049bcbcfdeaac7259a11afa2`. Latest developer-owned source/test change reviewed is exact `e3dca29babfaf330931f26aa3697aa3cd3c98545`; its code change is carrier-only, so the inspected `SessionRuntime` owner is unchanged on this tree.

Scope is the R9-11A SessionRuntime terminal cleanup / post-terminal mutation lane only: `SessionRuntime::cancel`, `check`, `tick`, `clear_runtime_state`, runtime events, and the existing runtime lifecycle tests in `crates/neko-session/src/lib.rs`. This review does not change Session/Carrier/ACK/wire/crypto architecture, D019, any capacity value, or release/live policy.

## H-R9-078 — HIGH correctness/resource-lifecycle finding

`RuntimeState::Error` is terminal everywhere except `SessionRuntime::cancel()` itself.

Current owner behavior:

- normal mutators call `check()`, which rejects `cancelled || state == Error` with `RuntimeError::Cancelled` before mutation/event emission;
- `tick()` explicitly treats `Error` as terminal and returns `Ok(())` without adding evidence;
- the first `cancel()` sets `cancelled=true`, sets `state=Error`, clears runtime-owned queues/dedup/watermark/window state, emits one `RuntimeEventKind::Error`, and returns `Ok(())`;
- a second and every later `cancel()` does **not** stop at `Error`: the special terminal fast-path only checks `state == Closed`. It re-runs `clear_runtime_state()` and appends another `Error` event each time.

Concrete sequence:

1. create a `SessionRuntime` and optionally populate runtime-owned state;
2. call `cancel(t1)` — runtime becomes `Error`, owned mutable delivery/window state is cleared, one terminal `Error` event is emitted;
3. call `cancel(t2)`, `cancel(t3)`, ... after terminalization;
4. every call still returns `Ok(())` and appends a fresh terminal `Error` event.

This violates the current R9-11A invariant that post-`Closed` / post-`Error` mutating APIs are fail-closed or explicitly idempotent. More importantly, it creates a policy-independent retained-state growth path **after terminalization**: `events` is intentionally retained as lifetime diagnostic history, so repeated post-Error `cancel()` calls can grow that retained vector indefinitely even though the Session has no remaining live work. The already-known `SessionRuntime.events` capacity question remains a maintainer/security policy gate; H-R9-078 does not choose a cap and does not depend on choosing one. Terminal quiescence can and should stop manufacturing new events without changing any capacity policy.

The current tests cover one cancel followed by ordinary mutators rejecting with `Cancelled`, but do not challenge repeated cancel on the already-`Error` runtime.

## Smallest repair contract

Keep current terminal/error architecture and all limits unchanged.

1. Make `cancel()` on an already-terminal `Error` runtime either:
   - explicitly idempotent (`Ok(())`) with **no** new event/state mutation, matching the existing idempotent `Closed` fast-path; or
   - fail closed with the existing committed terminal/cancelled error semantics, again with no new event/state mutation.
   Prefer the smallest shape consistent with current API behavior; do not invent a new error code.
2. Add a focused deterministic regression:
   - populate at least one owned runtime surface before the first cancel;
   - first cancel proves `state == Error`, owned runtime state is cleared, and exactly one new `Error` event appears;
   - capture event count and surviving lifetime diagnostics;
   - invoke cancel repeatedly on the terminal runtime;
   - prove event count does not increase, no delivery/window/success evidence appears, state remains `Error`, and already-cleared runtime-owned state remains empty/zero.
3. Preserve current behavior of ordinary post-Error mutators (`queue_send`, `receive`, `delivery_ack`, queue pops, stream mutation, graceful close) failing before evidence mutation.
4. Do not fold the known `SessionRuntime.events` retained-cap policy gate into this repair; no TTL/LRU/history-size/capacity number is selected here.
5. After repair, run focused `neko-session` tests, then the required developer-local clean exact-tree gate on the final pushed source/test SHA: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree; persist exact pushed SHA, UTC start/end, exit codes, OS/arch, and stable Rust version. No decoder/parser/crypto-framing change is required, so decode fuzz is not mechanically required.

## R9-11 remainder

After H-R9-078 is closed, continue R9-11A rather than declaring SessionRuntime lifecycle complete. Independently challenge remote close, idle timeout, graceful-close deadline, terminal release of all non-lifetime runtime ownership, and all other post-terminal mutators. Then continue R9-11B reliable-UDP combined terminal ownership, R9-11C retained Session replay/Carrier-generation terminality, and R9-11D executable process/socket lifecycle/result truth.

`READY_LIVE` remains `none`; this finding is entirely local and creates no new real-network question.
