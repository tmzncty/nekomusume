# Independent bounded challenge — R-REC-1 ACK validity/range/future-unsent/state immutability

**Date:** 2026-09-24  
**Exact developer source anchor:** `6c01d63e2b477b099f154461d8fdba1d8220d0d4` (`main` at challenge start)  
**Owner:** `crates/neko-reliable/src/lib.rs` — `Recovery::on_ack`, `Recovery::on_sent`, `Recovery::abandon_sent`, `Recovery::quiesce`  
**Classification:** independent source/control-flow challenge; no production protocol change

## Challenged invariant

A peer ACK must not fabricate loss/retransmit evidence for packets the sender never sent, and an invalid ACK must fail closed without mutating sent/outstanding/RTT/PTO/Reno/persistent-congestion/reservation state. Valid late/duplicate ACK behavior must remain distinct from impossible future ACK. Quiesce/lifetime-history changes (`62d03a6`) must not make stale/future ACK material mutate fresh recovery ownership.

## Source reasoning

`on_ack` (line 328):

- `ack.largest().ok_or(Error::InvalidRange)` — empty ACK rejects.
- `match self.largest_sent { Some(sent) if largest <= sent => {}, _ => return Err(Error::InvalidRange) }` — `largest > largest_sent` rejects atomically **before** any `self.sent.get`, `rtt.update`, `sent.remove`, `watermark_on_reserve.remove`, `outstanding_frames` mutation, `acked_frames` mutation, Reno bytes-in-flight, or persistent-congestion accounting. `largest_sent = None` (fresh or post-quiesce) → any nonempty ACK rejects.
- `acked` collects only `self.sent.keys() ∩ ack` — an ACK containing a packet number that was already purged/acked (removed from `sent`) produces `acked` empty but remains valid, not fabricated. A packet number that was never sent would force `largest > largest_sent` → `InvalidRange`.
- `out.lost_packets` collects only `sent` entries that pass packet-threshold or loss-delay — purged/never-sent numbers cannot appear because they are not in `sent`.
- `rtt.update` runs only when `self.sent.get(&largest)` finds the packet — an ACK whose `largest` was already removed produces no RTT sample, which is the correct late/duplicate distinction.
- `watermark_on_reserve.remove(&n)` and `outstanding_frames`/`acked_frames` mutation run only inside the `acked`/`lost` loops, which execute only after the `largest <= sent` gate passes.

`on_sent` (line 266): `p.number <= largest_sent` → `InvalidRange`; packet numbers are strictly increasing. `abandon_sent` (line 293): rolls back `largest_sent` to the exact `watermark_on_reserve` predecessor, not merely the max still outstanding.

`quiesce` (line 436): clears `sent`/`outstanding_frames`/`acked_frames`/`largest_sent`/`watermark_on_reserve` while preserving `rtt`/`pto_count`/`persistent_congestion`/`packets_sent`/`packets_lost` lifetime history. Post-quiesce `largest_sent = None` → any nonempty ACK rejects until a fresh `on_sent` re-establishes `largest_sent`. A stale/future ACK cannot mutate fresh recovery ownership because the `InvalidRange` gate runs before any state access.

## Result

No concrete defect found. The ACK validity gate is fail-closed and atomic; late/duplicate ACK is correctly distinguished from impossible future ACK; quiesce does not create a stale/future mutation path. R-REC-1 is a no-finding challenge on the exact-current owner.

## Evidence boundary

This is developer-source reasoning plus focused deterministic test review, not reviewer-local execution, hosted CI, WAN evidence or performance conclusion. No decoder/parser/crypto framing change is implicated; fuzz not required. No release flag, D019, or policy value touched. `READY_LIVE: none`.
