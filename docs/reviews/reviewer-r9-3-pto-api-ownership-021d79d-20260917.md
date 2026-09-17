# R9-3 reviewer navigation — authoritative PTO clock/API ownership

**Classification:** reviewer/navigator support; no code change and no new runtime finding.

**Repository anchor inspected:** reviewer tree `4fa3bde80e59888afb938932b7ea9f9662af4a23`; latest developer-owned source/test tree remains `021d79d88dadc9dbc7cf749745b2395c06602b29`.

## Scope inspected

- `crates/neko-reliable/src/lib.rs`: `Recovery::on_ack`, RTT/loss/PTO state.
- `crates/neko-carrier/src/lib.rs`: `PathRecovery`, `ReliableUdpRuntime`, `pto_probe`, `on_retransmit_sent`, retained plaintext ownership.
- `crates/neko-cli/src/main.rs`: cross-process `failover_client` reliable sends / shared ACK owner and the existing bounded `lab_pump` scheduling reference.
- `docs/spec/m2-udp-recovery.md`, `AGENTS.md`, `SECURITY.md`, current handoff and release boundaries.

No decoder/parser/crypto-framing change was made. No local gate was run by this reviewer note.

## Exact-current ownership facts

1. `Recovery` already owns the authoritative outstanding `SentPacket` map, RTT estimator and `pto_count`; duplicating those mutable facts in `failover_client` is unnecessary.
2. The R8 `lab_pump` currently mirrors outstanding packet send times in `LabState` and computes `oldest + rtt.pto_us(LAB_PTO_GRANULARITY_US, LAB_MAX_ACK_DELAY_US, pto_count)`. That is a useful scheduling reference, not a reason to create a second mutable R9 recovery ledger.
3. The already-committed lab inputs are `LAB_PTO_GRANULARITY_US = 1_000` and `LAB_MAX_ACK_DELAY_US = 0`. R9-3 may reuse these existing M2 baseline inputs; this reviewer does not authorize new timer/security policy numbers.
4. `ReliableUdpRuntime::pto_probe()` advances PTO state only when called; therefore the deadline decision must be read before firing the probe.
5. `ReliableUdpRuntime::on_retransmit_sent(...)` intentionally does not enforce congestion admission internally. The caller must prove `can_send(bytes)` before committing retransmission ownership. Do not interpret successful `on_retransmit_sent` as a substitute for the gate.
6. `recv_udp_delivery_ack` can block. The ACK application timestamp must be sampled after authenticated receive/decrypt/decode and immediately before `apply_ack`, from the same operation clock origin used by all sends.

## Smallest implementation shape

Prefer one read-only deadline query over a caller-side mirror. A minimal shape consistent with the current model is conceptually:

```text
Recovery::next_pto_deadline_us(granularity_us, max_ack_delay_us) -> Option<u64>
  = oldest outstanding ack-eliciting SentPacket.sent_at_us
    + rtt.pto_us(granularity_us, max_ack_delay_us, pto_count)
```

Thread that read-only query through `PathRecovery` / `ReliableUdpRuntime` with no state mutation. The exact function names are not normative; the invariant is that the timer base comes from the same authoritative outstanding-packet owner that ACK/loss removal mutates.

Do not expose the whole mutable sent map. Do not add a second packet-number/timer allocator. Do not change ACK, Session delivery, crypto, wire, or Carrier architecture.

For the cross-process owner, establish one `Instant` at the beginning of the bounded reliable operation and derive all recovery microsecond timestamps from that origin. Apply it to:

- first reliable `on_packet_sent` calls;
- post-return reliable `on_packet_sent` calls;
- retransmit `on_retransmit_sent` calls;
- authenticated Carrier `apply_ack` calls, sampled at observation time rather than before the blocking receive.

A small conversion helper is preferable to repeated ad-hoc casts; overflow must fail/saturate deterministically without introducing a new time policy.

## R9-3 PTO/retransmit order

The dependency/order contract for one intentionally suppressed Recovery-owned Data is:

1. construct/seal the current reliable Data under a fresh packet number;
2. check congestion admission for the exact encoded byte count before Recovery ownership commit;
3. call `on_packet_sent(packet_number, sent_at_us, ..., stable_frame_id, plaintext)`;
4. suppress exactly this one socket send in the test seam, after ownership exists;
5. drain authenticated ACKs first;
6. read the authoritative PTO deadline; while `now_us < deadline_us`, do not call `pto_probe`;
7. at/after the deadline, call `pto_probe` exactly once for this firing and require the expected stable `FrameId`;
8. re-seal the same logical Data under a fresh packet number/nonce;
9. check `can_send(exact_encoded_bytes)` before `on_retransmit_sent`;
10. call `on_retransmit_sent(fresh_packet_number, now_us, ..., same_frame_id)` before the socket send;
11. apply the retransmission Carrier ACK using observation-time `now_us` from the same origin;
12. require the committed loss rules to remove the suppressed original as well as retire the fresh copy before declaring Recovery settled.

Sealing may consume a fresh crypto sequence before a later congestion refusal if exact encoded length is needed for admission; that is not permission to reuse the nonce. The required invariant is still no socket send and no Recovery ownership commit after a failed congestion gate.

## Focused regressions / evidence

The first R9-3 process regression should prove all of the following on one bounded operation:

- selected first Data is already Recovery-owned when its wire send is suppressed;
- a diagnostic records the computed `deadline_us` and actual `fired_at_us`, with exactly `fired_at_us >= deadline_us` and no earlier PTO event;
- exactly the expected stable `FrameId` is probed/retransmitted;
- retransmit packet number differs from the suppressed original while stream/offset/len/payload identity is unchanged;
- every retransmit admission is explicitly proven before ownership commit;
- receiver logical delivery remains exactly once;
- Session DeliveryAck and Carrier ACK remain separate typed observations;
- retransmission ACK is applied at observation time and is a real positive retirement, not accepted-empty/rejected substitution;
- final evidence shows the retransmitted packet retired and the suppressed original was removed by loss detection, yielding exactly `remaining_in_flight=0` plus complete Session confirmation;
- terminal negative paths report residual Session/Recovery domains and never emit false settlement.

A negative timing test should stop just before the authoritative deadline and prove no PTO/retransmit diagnostic, then advance to/after the deadline and prove the single firing. Keep this deterministic/local; do not add a capacity-pressure benchmark.

## Reviewer conclusion

No new BLOCKER/HIGH is established by this navigation pass. The current queue remains non-exhausted and `READY_LOCAL 1 = R9-3 Data-loss recovery` is dependency-ready. This note narrows the implementation/API ownership enough that absence of a pre-existing helper is not a maintainer gate: under `AGENTS.md` the coding agent should choose the smallest read-only query shape, implement it, test it, push it, and continue.

`READY_LIVE` remains `none`; this local timing/recovery work creates no new real-network question by itself.
