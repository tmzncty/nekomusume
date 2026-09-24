# H-I4-117 — runtime PTO path never applies committed Reno persistent-congestion collapse

**Date:** 2026-09-24  
**Severity:** HIGH correctness / release-evidence reliability  
**Exact reviewed anchor:** `8b2f2f382eca62217110c7e604f1d38bf97efd5c`  
**Owners:** `crates/neko-reliable/src/lib.rs::{Recovery::on_pto,Reno::{persistent_congestion,on_persistent_congestion}}`, `crates/neko-carrier/src/lib.rs::{PathRecovery::{on_pto,persistent_congestion},ReliableUdpRuntime::pto_probe}`, plus the existing PathRecovery congestion/pacing checkpoint.

## Challenged invariant

The committed recovery stack already defines persistent congestion as three consecutive PTO transitions and provides a PathRecovery owner that collapses Reno to `2 * MSS` when that threshold is reached. A `ReliableUdpRuntime` PTO sequence must not count the same threshold while silently bypassing the Reno collapse. Below the threshold the window must remain unchanged by persistent-congestion handling; at the threshold it must tighten; a positive ACK resets the PTO streak before a later new streak.

This is an integration/correctness challenge of already-committed semantics. It does **not** choose a new PTO threshold, congestion algorithm, capacity value, ACK architecture, D019 rule, or wire/crypto policy.

## Exact-current source reasoning

1. `neko-reliable::Recovery::on_pto` increments `pto_count` on each real PTO and increments `persistent_congestion_events` exactly when `pto_count == PERSISTENT_CONGESTION_PTO_THRESHOLD`. The committed threshold constant is `3`.
2. `neko-reliable::Reno` has both the threshold predicate (`persistent_congestion(pto_count)`) and the effect (`on_persistent_congestion`), which sets `cwnd` and `ssthresh` to `2 * MSS`.
3. `PathRecovery::persistent_congestion` is explicitly documented as “Mark persistent congestion after repeated PTOs and collapse the window.” It applies the Reno effect when the underlying recovery PTO count reaches the committed threshold.
4. `ReliableUdpRuntime` is documented as owning the recovery and automatic health-driven behavior. Its `pto_probe` is the runtime PTO transition: it first rejects the zero-in-flight case, then calls `self.recovery.on_pto(4)` and returns retained plaintext probes. Exact-current `pto_probe` never calls `self.recovery.persistent_congestion()` before or after that transition.
5. No other exact-current `ReliableUdpRuntime` path applies `PathRecovery::persistent_congestion`; therefore a real runtime may execute PTO 1, PTO 2, PTO 3 and beyond, incrementing the underlying persistent-congestion history while leaving Reno at its pre-persistent-congestion window.

The earlier R-REC-3 no-finding (`2ac22784051d0870bd6645495a6df23394cc97bb`) is not contradicted: that bounded review explicitly owned `neko-reliable::Recovery::on_pto` / event counting and proved the threshold counter itself is truthful. It did not review `PathRecovery`/`ReliableUdpRuntime` application of the Reno effect.

## Existing test/evidence gap

The existing `pacing_and_send_decision_are_deterministic_and_accounted` checkpoint contains the comment “Persistent congestion collapses the window so can_send tightens” and calls `r.persistent_congestion()`, but it never drives `pto_count` to the threshold and has no assertion after that call. The historical commit that introduced this checkpoint (`4dc3121861d0a066940bc8bc98438f7cbd6cd39d`) likewise states that `persistent_congestion` collapses the window, even though the test as committed cannot prove that claim. This is an evidence gap in addition to the runtime wiring gap.

## Required bounded repair

Use the smallest current-semantics repair; do not redesign congestion control.

1. Wire the existing `PathRecovery::persistent_congestion()` effect into the real `ReliableUdpRuntime::pto_probe` transition after a legitimate non-empty-runtime PTO increments the count. Do not change `PERSISTENT_CONGESTION_PTO_THRESHOLD` or introduce a second threshold/policy value.
2. Add a focused deterministic runtime regression that would fail on this reviewed tree:
   - admit one small ack-eliciting packet so PTO is legitimate;
   - PTO 1 and PTO 2: prove a discriminating `can_send(...)` admission still reflects the non-collapsed window;
   - PTO 3: prove admission tightens to the existing `2 * MSS` Reno persistent-congestion window (without needing a production-only debug getter);
   - retain a positive ACK/reset control so a new sub-threshold PTO streak does not immediately re-trigger persistent congestion.
3. Strengthen or replace the old PathRecovery checkpoint so “persistent congestion collapses the window” is executable evidence rather than a comment plus an unasserted false-return call. Keep a below-threshold negative control and threshold positive control.
4. Preserve existing PTO probe selection, stable FrameId/plaintext ownership, health freshness, retransmission, ACK/RTT, Reno loss, and teardown semantics. No decoder/parser/crypto-framing owner needs to move; do not run fuzz mechanically.
5. On the final pushed source/test SHA, run the first-class developer-local clean exact-tree gate: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, verify clean tree, and persist reachable provenance with exact SHA, UTC start/end, exit codes, OS/arch, stable Rust version and clean-tree state.

## Evidence boundary

This finding is exact-current GitHub source/control-flow review only. No reviewer-local Rust/full gate, hosted CI, cross-platform execution, fuzz, WAN execution, or performance conclusion is claimed. `READY_LIVE` remains `none`; release/governance flags are unchanged.