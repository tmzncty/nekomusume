# Independent bounded UDP recovery review — exact `8f93b93`

Bounded independent review of `crates/neko-reliable/src/lib.rs` against `docs/spec/m2-udp-recovery.md`, at reachable exact `8f93b931b42e486882980f44813660ba6edb4267`. This is an independent bounded audit of the deterministic socket-free recovery model only — not a WAN/performance claim, not a frozen wire contract, not Session delivery evidence, and not an independent security/release approval.

## Method

Re-read the full `lib.rs` (`PacketNumbers`, `AckRange`/`AckRanges`, `RttEstimator`, `SentPacket`/`Recovery`, `Reno`, `VirtualClock`, `FaultProfile`/`simulate_*`) plus `tests/udp_loopback_recovery.rs`, `tests/era4_g_property.rs`, `tests/readiness_fuzz_seeds.rs`, and the unit `mod tests`. Challenged each claimed invariant below against source and focused tests on the exact tree.

## Challenged invariants and result

- **ACK range canonicalization / atomic bound — holds.** `AckRanges::insert` clones `ranges`, merges the candidate, and commits `self.ranges = merged` only after the `merged.len() <= max_ranges` check (`:100-121`); an over-limit insert returns `TooManyRanges` without mutating committed ranges. `from_ranges` builds a fresh `Self` and returns `Err` before any partial result escapes (`:123-143`). Construction work is proportional to range count, never to numeric range width (single `Vec` merge pass).
- **Packet-number exhaustion / monotonicity — holds.** `PacketNumbers::allocate` returns `u64::MAX` then sets `exhausted` on `checked_add` overflow, failing closed thereafter (`:48-58`). `on_sent` rejects duplicate numbers and any `p.number <= largest_sent` atomically (`:255-260`).
- **Future/unsent ACK — repaired this pass.** `Recovery::on_ack` now rejects `largest > largest_sent` before any RTT/packet/loss/PTO mutation (`:289-296`, commit `531c82d`). `largest == largest_sent` remains valid; `largest_sent == None` (nothing sent yet) is not rejected by this rule, which is safe because an empty `sent` map yields no acked/lost output regardless.
- **ACK/loss after retirement — holds.** Loss and ACK processing iterate only the live `sent` map; once a packet is removed by `acked`/`lost` it cannot be re-processed, and packet-number monotonicity prevents its number being reused.
- **`outstanding_frames` copy counts — holds.** `release_frame` decrements a `BTreeMap<FrameId, usize>` copy count and only reports the frame as releasable when the last copy leaves (`:349-358`), so overlapping original + retransmission copies keep a frame outstanding until every copy is settled (`overlapping_retransmission_keeps_frame_outstanding_until_all_copies_ack`).
- **Packet/time-threshold loss and reorder boundary — holds.** `largest - n >= PACKET_THRESHOLD` or `now - sent_at >= loss_delay` marks loss (`:321-330`); `reorder_inside_threshold_is_not_loss` and the repaired `time_threshold_and_reno_pacing` cover both boundaries on the exact tree.
- **RTT/ACK-delay/PTO arithmetic — holds.** `loss_delay_us = max(latest, smoothed) * 9/8` (`div_ceil`); `pto_us` uses saturating exponential backoff `base * (1 << min(pto_count, 63))`; `rtt_pto_are_deterministic` fixes the expected values.
- **PTO does not declare loss/delivery — holds.** `on_pto` only `take`s outstanding `FrameId` keys; it never removes from `sent` and never emits delivery evidence (`:361-375`).
- **Persistent-congestion accounting — holds (one boundary note).** `on_pto` increments `persistent_congestion_events` exactly once at the `pto_count == 3` crossing (`:365-368`); `loss_bytes_and_persistent_congestion_are_observable_and_bounded` confirms a single event after three PTOs.
- **Reno bounds — holds.** `lost` halves cwnd into `ssthresh = max(cwnd/2, 2*mss)`; `on_persistent_congestion` collapses to `2*mss`; all arithmetic saturating; `acked` grows cwnd only up to `ssthresh` then congestion-avoids.
- **Deterministic fault simulation termination — holds.** Both `simulate_fault_profile` and `simulate_delivery` bound `rounds <= total_frames + 2` and return `Error::Capacity` past it; `blackhole` drops without requeue so `pending` drains and the loop terminates.

## Boundary observation (not a defect, no repair)

`on_pto` increments `pto_count` even when `outstanding_frames` is empty, so a caller that fires PTO with nothing outstanding could accumulate `pto_count` toward the persistent-congestion threshold with no real probe in flight. Whether empty-outstanding PTO should arm at all is a caller-scheduling/semantic choice, not a current-semantics defect: the model exposes `on_pto` and leaves arming to the caller, no existing test or claim asserts empty-outstanding behavior, and every observed call site fires PTO only with outstanding frames. Recorded as an open semantic boundary for a future spec/ADR decision; classified `DEFER_POLICY_OR_ENVIRONMENT`, not repaired here.

## Evidence

- `cargo test -p neko-reliable` on exact `8f93b93`: 29 lib + 3 + 3 integration + 0 doc tests, all passed.
- Exact-tree gate for this exact SHA recorded under the sibling repair pass (`scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree).
- Prior independent reviews on reachable anchors remain indexed in `docs/release-security-review-packet.md`; this note adds the dedicated `neko-reliable` surface not previously covered.

No defect found beyond the already-repaired `largest > largest_sent` acceptance gap (`531c82d`). No new `READY_LIVE` question. Release/governance state unchanged.
