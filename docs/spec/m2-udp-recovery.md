# M2 bounded UDP recovery candidate

**Status: candidate deterministic state model; not a frozen wire contract or production transport.**

`neko-reliable` is socket-free and models only UDP Carrier packet recovery.
Its ACK evidence must never call `neko-session::DeliveryLedger::confirm_received`
or imply application delivery, path validation, authentication, or authorization.

## State and limits

- Packet numbers are full `u64`, monotonic, and fail closed after `u64::MAX`.
- ACK ranges are inclusive, sorted, merged when overlapping/adjacent, and bounded
  by a configured range count. Construction is proportional to range count,
  never to the numeric width of a range.
- Sent-packet state defaults to at most 4096 packets and 64 frame references per
  packet. Duplicate packet numbers, empty ack-eliciting packets, and excess state
  are rejected atomically.
- RTT uses latest/min/smoothed/variance microsecond state. Loss uses packet
  threshold 3 or 9/8 of max(latest RTT, smoothed RTT). PTO uses smoothed RTT +
  max(4*variance, granularity) + max ACK delay with saturating exponential backoff.
- PTO selects a bounded number of outstanding frames as probes. It does not mark
  packets lost. Loss schedules FrameIds; it never retransmits an old packet image.
- Reno starts at 10 MSS, halves to at least 2 MSS on loss, and provides a pacing
  interval model. This is a baseline for experiments, not a performance claim.

## Deterministic evidence

Unit tests cover packet-number exhaustion, huge ACK ranges, canonical merges,
range/state limits, RTT/PTO arithmetic, packet/time threshold loss, reordering
inside the threshold, frame retransmission, Reno/pacing, and complete delivery
under deterministic 0%, 1%, 5%, and 10% first-send loss with optional reversal.
No parser or external byte decoder is added by this slice, so the existing wire
fuzz target is unaffected.
