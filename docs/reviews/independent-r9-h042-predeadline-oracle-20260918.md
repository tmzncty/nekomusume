# Independent R9 review — H-R9-042 pre-deadline oracle mismatch

**Reviewed anchor:** `f7bc6a19d7e2d0b6051810aa0cb808b879bf0051` (reachable on `main` under handoff `88f8194fe587cd2f161c98e8bc9c5b7913c58fa9`).

**Classification:** HIGH evidence/oracle correctness (`H-R9-055`). This does **not** presently prove a new runtime defect in the post-return PTO caller. It proves that the deterministic regression used to close H-R9-042 does not execute the caller-side guard it claims to prove, and in fact invokes the mutating PTO transition while its injected time is still before the deadline.

## Scope inspected

- `crates/neko-reliable/src/lib.rs::Recovery::{next_pto_deadline_us,on_pto}` and exact-`f7bc6a1` regression `tests::pto_query_returns_deterministic_deadline_and_guard_holds_before_it`
- current post-return reliable-UDP caller in `crates/neko-cli/src/main.rs` around `next_pto_deadline_us` / `now_us >= deadline` / `rt.pto_probe()` and receive-deadline clamping
- current `docs/CHATGPT_HANDOFF.md` H-R9-042 closure claim
- `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, `docs/standing-vps-lab-authorization.md`, `docs/vps-rental-window-priority.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, and `docs/release-security-review-packet.md`

No wire decoder/parser/crypto framing changed, so fuzz is not implicated by this finding. Reviewer could not execute a local repository checkout in this automation environment because outbound DNS for `github.com` was unavailable; this note therefore does not claim reviewer-local test execution. Exact `f7bc6a1` hosted Rust CI run `35336471528` is green cross-evidence only and cannot strengthen a non-discriminating oracle. Developer-reported local exact-tree provenance remains separately recorded in the handoff, including its unrelated `sigterm_after_ready` flake.

## Challenged invariant

At `now_us = next_pto_deadline_us(...) - 1`, the executable post-return recovery owner must perform **zero PTO transition**:

- do not call `pto_probe` / `Recovery::on_pto`;
- do not increment `pto_count`;
- do not increment persistent-congestion events;
- do not create retransmit work/ownership or positive PTO/retransmit diagnostics;
- do not alter outstanding packet/frame state merely because the clock is just before the deadline.

At `now_us >= deadline`, the same executable guard may invoke the PTO transition normally. The negative and positive controls must exercise the same decision owner.

## Finding

Exact `f7bc6a1` says the regression challenges the caller at `deadline_us - 1`, but the test does this:

```rust
let now = deadline - 1;
assert!(now < deadline);
let probes_before = r.on_pto(4).unwrap();
...
let probes_at = r.on_pto(4).unwrap();
let _ = probes_before;
```

That is the opposite of the claimed pre-deadline invariant. `Recovery::on_pto` is intentionally time-agnostic and mutating: it increments `pto_count`, may increment `persistent_congestion_events`, and returns outstanding probe frames. The test calls that transition while its injected `now` is still before the deadline, discards the result, and only asserts the scalar relation `deadline > now`.

Therefore the test would remain green even if the executable caller lost its `now_us >= deadline` guard entirely. It proves the deadline query arithmetic and proves that `on_pto` can return a probe; it does **not** prove “zero PTO/retransmit transition just before the deadline.” The current handoff closure sentence is consequently too strong.

The current production caller still appears to contain the intended guard (`next_pto_deadline_us(...)` followed by a `now_us >= deadline` filter before `rt.pto_probe()`), so this finding is presently an acceptance-oracle gap rather than a demonstrated runtime regression.

## Smallest closure contract

Do not redesign PTO, Recovery, Session/Carrier semantics, clocks, or wire behavior. Keep the current deadline query and current executable ordering unless a focused test proves an implementation defect.

Make the **real executable decision owner** testable and use it from production code. A minimal helper extraction is acceptable if it owns the exact `now_us >= deadline` decision and the `pto_probe()` invocation rather than merely re-testing a duplicated boolean expression.

Add one deterministic paired regression:

1. prepare one outstanding frame and obtain its current PTO deadline;
2. snapshot `pto_count`, persistent-congestion count, Recovery in-flight/outstanding-frame state, and any caller-visible retransmit bookkeeping that the helper can mutate;
3. invoke the production decision owner at `deadline - 1`;
4. require no PTO probe/transition, no retransmit work, and every snapshot above unchanged;
5. invoke the **same** decision owner at exactly `deadline` (or later);
6. require one legitimate PTO transition/probe and the expected single increment/state effect;
7. the regression must deterministic-red if the production guard is weakened or removed.

Do **not** call `Recovery::on_pto` directly in the pre-deadline negative. Do not add sleeps or wall-clock timing dependence merely to satisfy the oracle.

After the focused repair/test commit, run the normal clean exact-tree developer-local gate and continue immediately to R9-4; do not wait for reviewer cadence. H-R9-054 remains closed absent contradictory new source/test evidence.