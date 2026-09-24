# Independent bounded post-H-I4-120 Recovery owner-delta review — `4511e4f`

**Source/test anchor:** exact reachable `4511e4f147eb9511907bb4ec52cad687f113bb7a`.

**Result:** bounded **NO-FINDING** for the H-I4-120 closure delta and the currently moved Recovery test owner, with H-I4-119 explicitly excluded.

## Invariant challenged

The closure must not create a new correctness regression while rolling back an unauthorized core-semantic choice. In particular:

- future/unsent ACK remains fail-closed before RTT/PTO/loss mutation;
- duplicate/stale ACK cannot use time threshold to lose newer packets beyond the ACK frontier;
- newly retired packets and threshold-loss candidates remain tied to actually sent/stored state;
- RTT sampling and loss/retransmit behavior are unchanged by the governance rollback;
- zero-loss Reno remains a no-op and positive loss remains a single aggregate reduction;
- persistent-congestion effect remains wired at the existing PTO threshold;
- the added non-ack-eliciting ACK regression remains semantics-neutral about the unresolved PTO reset rule.

## Owners inspected

- `crates/neko-reliable/src/lib.rs::Recovery::on_ack`
- `RttEstimator`, frame-copy/outstanding ownership, `Reno`, `PathRecovery`, `ReliableUdpRuntime::pto_probe`
- exact-current Recovery tests including future/unsent ACK rejection, stale-ACK loss ordering, positive time-threshold loss, Reno zero-loss/positive-loss controls, persistent-congestion threshold/effect, deterministic fault simulation, and `non_ack_eliciting_ack_keeps_older_packet_outstanding`
- source history from the prior reviewed Recovery anchor `68d938279478b91661f786e830e99af83db457f1` through `4511e4f...`

## Falsification result

No new concrete counterexample was found in this bounded scope.

1. `largest > largest_sent` is still rejected before newly-acked retirement, RTT update, PTO reset, loss scan, or retransmit generation.
2. Loss eligibility still requires an outstanding packet to be at or before the acknowledged frontier; the H-I4-118 stale-old-ACK regression therefore continues to protect newer in-flight packets while the positive control permits an actually later ACK to time-threshold an older packet.
3. The H-I4-120 rollback removes the unauthorized ack-eliciting-only reset guard; it does not alter packet/loss ordering, RTT arithmetic, retransmit ownership, Reno, or persistent-congestion wiring.
4. The added non-ack-eliciting test now observes only non-disputed facts: packet 1 is legally retired, packet 0 remains outstanding under the chosen timing, no false time-threshold loss is generated, and a later ACK can retire packet 0. It does not assert `pto_count` after the first ACK.
5. Net source history from the prior `68d9382` Recovery review through this final source/test tree leaves the prior production Recovery owner intact after rollback and adds focused test coverage. Existing R-REC-1/2/3, R-REC-4a/4b and `independent-r-rec-residual-68d9382-20260924.md` therefore remain reusable outside the explicitly policy-gated seam.

## Explicit exclusion — H-I4-119

This review does **not** decide whether PTO backoff should reset after any newly acknowledged sent packet or only after newly acknowledged ack-eliciting packets. The exact-current code remains on the pre-gate baseline solely because H-I4-120 required rollback to the last authorized semantics. H-I4-119 remains a maintainer/spec decision.

## Execution/evidence boundary

No reviewer-local Rust/full-gate command was run for this note. Developer-local exact-tree `scripts/check.sh` / `git diff --check` provenance for `4511e4f...` is separately persisted in `docs/notes/check-gate-4511e4f-20260924.md`; GitHub exposes no hosted status entry for that SHA. No cross-platform, fuzz, WAN, performance, adversarial-load, security-approval, or release claim is made.

No decoder/parser/crypto-framing owner moved, so fuzz is not requested.

`READY_LIVE: none`; release items 3/4 and all release/governance flags remain unchanged.
