# R9-3 reviewer navigation — ACK observation time and PTO ownership

**Reviewed source/test anchor:** exact `021d79d88dadc9dbc7cf749745b2395c06602b29`, still the latest developer-owned source/test commit reachable from current `main` at review time.

**Classification:** reviewer/navigator refinement for the already-READY R9-3 implementation slice. This is not a new defect in the accepted no-loss path, does not reopen H-R9-039/H-R9-038, and changes no Session/Carrier/ACK/crypto/wire architecture or policy value.

## Scope inspected

- `crates/neko-cli/src/main.rs`
  - cross-process reliable `on_packet_sent(..., 0, ...)` sites;
  - shared `recv_udp_delivery_ack` receive/apply owner;
  - existing `lab_pump` PTO scheduling reference.
- `crates/neko-reliable/src/lib.rs`
  - `RttEstimator::{update,loss_delay_us,pto_us}`;
  - `Recovery::{on_ack,on_pto,frame_outstanding}`.
- `crates/neko-carrier/src/lib.rs`
  - `ReliableUdpRuntime::{recovery_engine,pto_probe,on_retransmit_sent,apply_ack}`.
- `docs/spec/m2-udp-recovery.md` and current R9-3 handoff contract.

## Refinement 1 — ACK time must be sampled after receive, not before a blocking receive helper

The previous R9-3 note correctly requires one monotonic relative recovery clock. The exact-current helper adds one important placement constraint: `recv_udp_delivery_ack` can block in `recv_from`, repeatedly using read timeouts while waiting for a datagram. Therefore R9-3 must not compute one scalar `now_us` before entering that helper and later reuse that stale value for `apply_ack`.

The safe current-semantics shape is to thread the operation `Instant` origin (or a tiny callback/value source derived from that same origin) into the shared ACK owner and sample `elapsed()` **after the authenticated Carrier ACK has actually been received/decrypted/decoded and immediately before `apply_ack`**. Every first-send and retransmission timestamp must use the same origin.

Why this matters beyond RTT precision:

- `Recovery::on_ack` uses `now_us.checked_sub(sent_at_us)` for the largest acknowledged packet;
- `RttEstimator::update` ignores a zero sample;
- time-threshold loss is disabled while `loss_delay_us()==0`;
- PTO does not itself declare the suppressed original lost.

So if a retransmission is timestamped at a positive time but its ACK is applied with an earlier/stale value, the result can be `Error::Arithmetic`. If the applied time collapses to the retransmit send instant and yields a zero RTT sample, the fresh retransmission may retire while the intentionally suppressed original remains Recovery-owned, especially when packet-threshold distance is below 3. The dedicated R9-3 proof must reject both outcomes rather than accepting a fresh-packet ACK as sufficient completion.

## Refinement 2 — PTO deadline ownership should not fork Recovery truth

The M2 spec fixes the algorithmic formula: PTO is based on smoothed RTT + `max(4*variance, granularity)` + max ACK delay with backoff. It does **not** authorize this reviewer to invent a new granularity/max-ACK-delay policy value.

The current `lab_pump` keeps a fixture-local outstanding timestamp map. For the cross-process R9 runtime, prefer the smallest read-only Recovery/Runtime seam that exposes the authoritative oldest outstanding sent time or computed PTO deadline, instead of creating a second mutable timing/recovery ledger in `failover_client`. A read-only helper such as `oldest_outstanding_sent_at_us()` or a deadline query parameterized by already-committed M2 inputs is compatible with the current architecture because Recovery remains the sole sent-packet owner.

If the implementation instead keeps a caller-side timestamp mirror, it must be demonstrably derived-only and exact under ACK/loss/retransmit removal; it must never become a second source of recovery truth. Do not promote the R8 lab fixture constants into a new runtime/security policy merely because they are convenient test values.

## R9-3 discriminating acceptance additions

Keep the existing handoff requirements and additionally prove:

1. the intentionally suppressed original is committed to Recovery after congestion admission;
2. the PTO diagnostic includes enough timing evidence to show `fired_at_us >= deadline_us`, with no earlier PTO/retransmit event;
3. the retransmission uses the same stable FrameId/logical stream+offset+payload and a fresh packet number/nonce;
4. the authenticated Carrier ACK for the retransmission is applied using an observation time sampled after receipt from the same monotonic origin;
5. a positive Carrier transition for the fresh retransmission alone is not treated as final settlement: the test must prove the suppressed original is also eventually removed from Recovery by the committed loss rules, and final `in_flight == 0` is exact;
6. Session delivery remains exactly once and Session DeliveryAck remains a separate evidence domain;
7. no accepted-empty or typed-rejected ACK may substitute for the required real retransmission retirement/loss closure.

If this focused regression exposes an implementation contradiction, repair the smallest existing-semantics owner and continue. Do not redesign ACK framing or Session semantics.

## Validation / evidence boundary

This navigation note changes documentation only and ran no developer-local exact-tree gate. Existing exact-`021d79d` developer-local provenance remains the source/test gate for that tree; hosted CI remains separate cross-evidence.

No decoder/parser/crypto framing change is requested, so decode fuzz is not mechanically required for this navigation slice. `READY_LIVE` remains `none`; this is local cross-process recovery correctness/evidence work.