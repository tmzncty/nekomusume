# Transport Recovery Research

**Protocol research: 伊冯；Security research: 阿米娅；Transport research: 符玄；Plan & implementation: 庄方宜；Coordination/Review: 佩丽卡**
**检索日期：2026-08-26**
**Status: research input; M0 does not implement a live carrier.**

## Two acknowledgement domains

`packet_feedback` is Carrier/Path-local: packet number, ACK, RTT, loss, PTO and congestion signals. It cannot prove Session delivery. `session_delivery` is logical: stream/message identity and offset/range state that survives Carrier change. `path_validated` is a third, independent evidence domain for reverse reachability and address ownership. A packet ACK is not a path-validation result.

## Recovery candidates

[RFC 9002](https://www.rfc-editor.org/rfc/rfc9002) §§5–7 supplies QUIC recovery references: RTT estimation, loss thresholds, PTO and persistent congestion. [RFC 6298](https://www.rfc-editor.org/rfc/rfc6298) §§2–5 and [RFC 8985](https://www.rfc-editor.org/rfc/rfc8985) describe TCP timer/RACK references. [RFC 5681](https://www.rfc-editor.org/rfc/rfc5681) §§3–4 and [RFC 9438](https://www.rfc-editor.org/rfc/rfc9438) are congestion-control references. These are candidates, not M0 commitments. UDP recovery belongs to a future Carrier; TCP must not receive a duplicate TCP packet ACK layer.

## Logical state and failover

For each stream, represent ranges as `unsent`, `in_flight` (delivery uncertain), `confirmed` (the selected proof object says what was confirmed), or `rejected`. Receiver deduplication is by authenticated Session/stream/offset; overlapping bytes must match or fail closed. The cache is bounded by configured bytes, ranges and age.

Recommended M0 transition sketch:

1. Send on active Path; record `in_flight`.
2. Accept only an explicitly defined Session delivery proof as `confirmed`.
3. On timeout/blackhole suspicion, mark outstanding data `uncertain`, not lost or delivered.
4. Validate fallback Path with authenticated challenge bound to Session identity, epoch and transcript.
5. Replay uncertain ranges; receiver deduplicates; promote only after proof.
6. Switch `active_path_epoch` after validation; retain old path only under a later policy.

Use hysteresis and minimum hold time in M1/M2 so transient RTT/loss does not flap paths. M0 specifies states and invariants, not thresholds.

## Experiments

Use QEMU/netns/veth and Linux `tc netem` ([man7 netem](https://man7.org/linux/man-pages/man8/tc-netem.8.html)) with synthetic endpoints. Matrix: baseline; RTT 20/80/150/300 ms; random loss 1/5/10%; burst loss; reorder; duplication; bandwidth changes; UDP blackhole; TCP fallback; address change. Record recovery success, failover latency, duplicate bytes, confirmed bytes, loss/replay counts, queue/memory bounds and path flaps. Real WAN results must remain separate from controlled explanations.

## Keep out of M0/M1

No concurrent UDP+TCP striping, FEC, multipath scheduler, final congestion-control claim, adaptive scoring claim, or production benchmark claim. M0 is codec/state specification and deterministic tests only.
