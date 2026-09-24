# Independent bounded Recovery residual refill — `68d9382`

**Source anchor:** exact reachable `main` `68d938279478b91661f786e830e99af83db457f1`.

**Result:** bounded **NO-FINDING** for the inspected, non-policy Recovery residuals. This review explicitly excludes the already-open H-I4-119 ACK/PTO reset-semantics gate; it neither chooses nor changes that core ACK/PTO policy.

## Scope and invariant challenged

Re-read exact-current `crates/neko-reliable/src/lib.rs` plus the `PathRecovery` / `ReliableUdpRuntime` integration in `crates/neko-carrier/src/lib.rs`, current session/release docs, and the closed H-I4-116/117/118 repair evidence. The bounded challenge covered:

- duplicate/stale ACK behavior after H-I4-118;
- future/unsent ACK atomic rejection and high-water ownership;
- packet- and time-threshold loss eligibility around the acknowledged frontier;
- frame-copy / retransmit lifetime and suppression after sibling-copy ACK;
- reservation rollback through `abandon_sent` in the committed serialized send path;
- Reno zero-loss and positive-loss integration after H-I4-116;
- persistent-congestion threshold/effect after H-I4-117, excluding the disputed ACK reset trigger;
- deterministic fault-simulation invariants;
- recovery outcome epochs and fresh health-delta projection.

## Challenge result

No new concrete counterexample was found in this bounded scope.

1. `Recovery::on_ack` rejects `largest > largest_sent` before RTT, packet, frame-copy, PTO-count, or loss-state mutation. ACK numbers below the high-water do not require the packet to remain in `sent`; only actually stored packets are newly retired.
2. The H-I4-118 repair now restricts threshold-loss candidates to outstanding packets at or before the acknowledged frontier. A stale ACK for an older retired packet therefore cannot time-threshold newer in-flight packets, while a genuinely later ACK may still retire older packets under the existing time threshold.
3. Frame-copy ownership remains coherent: acknowledging one copy marks the logical frame acknowledged; later sibling loss does not create retransmit work; an unacknowledged frame is retransmitted only when the final outstanding copy retires as lost. ACK markers are pruned once the final copy is gone.
4. `Reno::lost(0)` is now a no-op for `bytes_in_flight`, `cwnd`, and `ssthresh`; positive aggregate loss is applied once through `PathRecovery::on_ack`. The existing current tests cover the zero-loss regression and positive-loss control.
5. PTO processing still increments the recovery count and emits the persistent-congestion transition at the existing threshold. `ReliableUdpRuntime::pto_probe` now applies the existing `PathRecovery::persistent_congestion` effect, so the runtime window collapse is wired; this review deliberately does not decide which class of later ACK should reset the PTO count.
6. Deterministic loss/fault simulation remains bounded and deterministic under the current tests; no capacity-pressure or adversarial-load benchmark was introduced.
7. `PathRecovery` advances outcome epochs on resolved packet outcomes or PTO probe outcomes; `fresh_health_sample` consumes each epoch once and separates resolved-loss accounting from PTO pressure. No contradictory current health-freshness path was found.
8. `abandon_sent` restores the previous high-water only when abandoning the current latest reservation. The executable send path remains serialized around reservation/seal/admission/rollback; this pass found no reachable current owner that stacks independently live nested reservations and then relies on a different rollback order. No broader reservation architecture is inferred from this no-finding.

## Explicit exclusion: H-I4-119

Exact-current `Recovery::on_ack` resets `pto_count` when there is any newly acknowledged stored packet. Whether the project should instead reset only when a newly acknowledged packet is ack-eliciting is **not** decided by current repository semantics strongly enough for reviewer-side automatic repair. `docs/reviews/reviewer-h-i4-119-pto-reset-semantics-gate-20260924.md` remains authoritative for that maintainer/core ACK-PTO semantics gate.

This no-finding must not be cited as closing H-I4-119 or as a decision about persistent-congestion reset policy.

## Evidence boundary

This was an exact-current GitHub source/test/control-flow review. No reviewer-local Rust/full-gate command, hosted-CI result, fuzz run, WAN experiment, cross-platform run, performance conclusion, or new developer-local provenance is claimed. No decoder/parser/crypto-framing owner changed, so fuzz is not requested.

Release items 3 and 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`; `READY_LIVE: none` remains unchanged.

## Queue consequence

The non-policy Recovery refill may advance as bounded item-4 support. Keep H-I4-119 separately policy-gated and continue with exact-owner pre-auth diff/reuse, repository-wide 13-surface current-owner inventory, then only genuinely moved/unreviewed core-owner challenges and grouped release/security reconciliation. Do not manufacture duplicate review lanes solely to preserve queue depth.
