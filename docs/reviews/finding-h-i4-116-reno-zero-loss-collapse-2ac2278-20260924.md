# H-I4-116 — loss-free ACK path invokes Reno loss reduction with zero lost bytes

**Severity:** HIGH correctness / item-4 release-support blocker  
**Exact source anchor reviewed:** `2ac22784051d0870bd6645495a6df23394cc97bb`  
**Review lane:** R-REC-4 (`neko-reliable` Reno + deterministic fault accounting)

## Finding

The exact-current `PathRecovery::on_ack` integration in `crates/neko-carrier/src/lib.rs` always executes:

- `self.reno.acked(released_acked);`
- `self.reno.lost(released_lost);`

for every successful Recovery ACK outcome, including an ACK with **no lost congestion-accounted bytes**, where `released_lost == 0`.

The exact-current `neko_reliable::Reno::lost` implementation in `crates/neko-reliable/src/lib.rs` subtracts `b` from bytes-in-flight but then **unconditionally** sets `ssthresh = max(cwnd / 2, 2*mss)` and `cwnd = ssthresh`. It has no `b == 0` guard.

Therefore a clean ACK-only outcome is classified as a congestion-loss event by the Reno owner. From the initial `cwnd = 10*mss`, even a loss-free ACK can grow the window in `acked(...)` and then immediately halve that updated window in `lost(0)`. Repeated successful ACKs can drive the window toward the 2-MSS floor despite zero packet loss.

This is a concrete cross-owner correctness defect, not a policy-value question: a zero-byte loss report is not positive loss evidence and must not cause a multiplicative decrease.

## Why existing tests did not catch it

Current tests check that ACK retirement drains `bytes_in_flight` and re-opens some admission, and direct Reno tests call `lost(...)` with a positive byte count. They do not challenge the zero-loss call edge or assert that a loss-free ACK cannot reduce the congestion window. A collapsed window can still admit a small 400-byte send, so the existing `can_send(400)` control does not falsify this defect.

## Required smallest repair

Keep the current recovery / ACK / congestion architecture and all numeric policy values unchanged.

1. Make zero lost congestion bytes a no-op for Reno loss reduction. The narrowest robust owner-level invariant is that `Reno::lost(0)` must not change `cwnd`, `ssthresh`, or `bytes_in_flight`; an equivalent caller-side guard is acceptable only if every committed caller is demonstrably covered.
2. Add a focused direct regression proving `Reno::lost(0)` cannot reduce congestion state.
3. Add a focused `PathRecovery` / `ReliableUdpRuntime` regression: send an ack-eliciting packet, apply an ACK that retires it with no loss, and prove the loss-free ACK does not cause multiplicative decrease. Use existing observable admission/pacing/state seams; do not add a production debug API solely for the test.
4. Preserve the positive-loss control: when `released_lost > 0`, the existing Reno reduction must still occur exactly once for that aggregate loss outcome.
5. Preserve ACK validity, RTT/PTO, persistent-congestion, retransmit ownership, D019/source-retention behavior, and current congestion constants. No wire/parser/crypto framing change is expected, so fuzz is not required.

After repair: commit/push source + focused tests, then run the final reachable developer-local exact-tree gate (`PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree) and persist provenance with exact SHA / UTC times / exit codes / OS+arch / stable Rust. Continue R-REC-4 immediately after closure; do not wait for reviewer cadence.

## Evidence boundary

This finding is exact-current GitHub source/control-flow review. No reviewer-local Rust/full-gate, hosted CI, WAN, fuzz, cross-platform execution, or performance result is claimed here.
