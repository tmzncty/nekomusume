# Independent bounded challenge — R-REC-3 RTT/PTO/persistent congestion

**Date:** 2026-09-24  
**Exact developer source anchor:** `6c01d63e2b477b099f154461d8fdba1d8220d0d4` (`main` at challenge start)  
**Owner:** `crates/neko-reliable/src/lib.rs` — `RttEstimator::update`/`loss_delay_us`/`pto_us`, `Recovery::on_pto`, `persistent_congestion_events`  
**Classification:** independent source/control-flow challenge; no production protocol change

## Challenged invariant

RTT samples must be eligible only for packets actually sent (not fabricated by ACK). PTO deadline/count transition must be truthful exponential backoff. Persistent-congestion event counting must be a threshold crossing, not per-PTO accumulation. Quiesce must preserve lifetime history without resetting RTT/PTO truth.

## Source reasoning

`RttEstimator::update` (line 155): `sample_us == 0` → skip; `latest_us = sample`; `min_us` tracks minimum; `adjusted = sample - ack_delay` when `sample - min >= ack_delay`; initialized → `variance = (3*var + delta)/4`, `smoothed = (7*smoothed + adjusted)/8`. RTT update runs only inside `on_ack` when `self.sent.get(&largest)` finds the packet — a purged/never-sent `largest` produces no RTT sample, so ACK cannot fabricate RTT.

`loss_delay_us` (line 180): `base = max(latest, smoothed)`; `base * 9/8`. Loss delay is RTT-derived, not a fixed policy value.

`pto_us` (line 185): `base = smoothed + max(4*var, granularity) + max_ack_delay` (initialized) or `1_000_000` (uninitialized); `base * 2^pto_count` — exponential backoff via `pto_count` shift. `pto_count` increments only in `on_pto`, resets to 0 in `on_ack` when `out.acked_packets` is non-empty (positive ACK).

`on_pto` (line 450): `max_probe_frames == 0` → `InvalidLimit`; `pto_count + 1`; `== PERSISTENT_CONGESTION_PTO_THRESHOLD` → `persistent_congestion_events + 1` — exactly once at threshold crossing, not `>=` repeated counting. Returns `outstanding_frames` minus `acked_frames`, capped at `max_probe_frames`.

`quiesce` (line 436): clears `sent`/`outstanding`/`acked`/`largest_sent`/`watermark` but preserves `rtt`/`pto_count`/`persistent_congestion`/`packets_sent`/`packets_lost` — lifetime history is truthful history, not reset to zero. Post-quiesce `largest_sent = None` gates any stale ACK before state access.

## Result

No concrete defect found. RTT eligibility is tied to `sent` membership; PTO backoff is exponential and reset only on positive ACK; persistent-congestion counting is a single threshold crossing; quiesce preserves lifetime history without creating stale/future mutation paths. R-REC-3 is a no-finding challenge on the exact-current owner.

## Evidence boundary

Developer-source reasoning, not reviewer-local execution, hosted CI, WAN evidence or performance conclusion. No decoder/parser/crypto framing change; fuzz not required. No release flag, D019, or policy value touched. `READY_LIVE: none`.
