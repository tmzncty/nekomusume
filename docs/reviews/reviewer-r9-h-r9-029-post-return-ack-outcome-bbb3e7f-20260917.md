# Reviewer checkpoint — exact `bbb3e7f`: post-return Carrier ACK consumer re-collapses accepted-empty and rejected outcomes

**Source/test anchor reviewed:** `bbb3e7f5310dea6a2342a2eda212cbc34326fd89`

**Repository HEAD at review:** `6e9c154f9e1d3aa379d8f64a700c58f4432771e4` (review/handoff docs only after the source/test anchor).

**Classification:** HIGH correctness/evidence defect in the materially new cross-process R9 integration. This is local item-4 support only; no WAN, release, freeze, production, or protocol-policy decision.

## Exact-current facts

The H-R9-026 source repair remains correct in the shared receive owner and in the primary/settlement callers:

- `recv_udp_delivery_ack` preserves three Carrier outcomes: real retirement (`applied=true` with non-empty `acked_packets`), accepted-empty stale/duplicate (`applied=false`, `rejected=false`, empty `acked_packets`), and typed Recovery rejection (`applied=false`, `rejected=true`).
- the primary receive loop increments applied only on real retirement and rejected only on typed `Err`;
- the settlement loop does the same and does not count accepted-empty as either class.

However the migration-back **post-return** consumer still matches:

```rust
Ok(UdpAcknowledgement::Carrier {
    applied,
    acked_packets,
    ..
})
```

It therefore discards `rejected`. It then emits `r9_udp_return_packet_ack` for every Carrier outcome with only:

```text
applied=<applied && current_packet_retired>
packet_number=<current expected post-return packet>
retired=<whether acked_packets contains current packet>
```

Consequently two semantically different authenticated inputs are observationally collapsed again at this caller:

1. a legal stale/duplicate ACK accepted by Recovery with an empty transition;
2. a future/never-sent ACK rejected atomically by Recovery.

Both currently become the same `r9_udp_return_packet_ack applied=false ... retired=false` event. The protocol success gate remains conservative because the loop still requires Session logical completion plus `rt.in_flight()==0`, but the process/result evidence is false: item-4 cannot tell accepted-empty from rejected feedback on the post-return owner.

The existing stale/future process regressions do not exercise this branch; they run the initial reliable-UDP phase. The current positive P2 process fixture likewise does not inject a stale/future Carrier ACK after migration-back. Thus no current built-binary oracle closes this caller-specific collapse.

## Required smallest repair

Do not redesign ACK framing, Recovery, Session, Carrier architecture, crypto/wire, deadlines, malformed budgets, or migration policy.

1. In the existing post-return `recv_udp_delivery_ack` consumer, retain `rejected` instead of `..`-discarding it.
2. Make the outcome diagnostics mutually exclusive and truthful:
   - **current positive retirement:** emit the existing positive `r9_udp_return_packet_ack` only when `acked_packets` contains the exact `post_pn_client`; this is the only event that may claim the current packet retired;
   - **accepted-empty:** emit a classification-only diagnostic such as `r9_udp_return_packet_ack_accepted_empty`; it increments no applied/rejected/delivery/health/failover state;
   - **typed rejection:** emit a distinct `r9_udp_return_packet_ack_rejected` (or an equivalently explicit phase-tagged rejection diagnostic); it must not claim the current packet number was ACKed and must not change Recovery/Session state.
3. If an accepted Carrier ACK retires some packet other than `post_pn_client`, do not label the current post-return packet as retired; continue the same bounded receive owner until the existing dual-domain success condition is met or the existing deadline fails.
4. Add focused deterministic built-binary regressions that inject exactly one fresh-envelope stale semantic ACK and exactly one future/never-sent ACK **after migration-back/post-return send**, proving the three classes remain distinct at this caller. Reuse the H-R9-028 one-shot injection machinery where practical; do not add a second receive owner or checker framework.
5. For the post-return future case, require a typed rejection diagnostic, no false current-packet retirement, later legitimate Carrier ACK progress, exact Session DeliveryAck, final Recovery zero, and client/server success. For accepted-empty, require exactly one accepted-empty classification, zero rejection, no extra positive retirement, exact Session completion, Recovery zero, and client/server success.

## Queue consequence

This HIGH moves ahead of ordinary R9 expansion. It can be repaired coherently with H-R9-028 because both need one-shot stale/future fault injection and exact outcome diagnostics. After H-R9-029 and H-R9-028 are closed, continue the already-valid P2 C1-C4 closure, ACK-arrival-order challenge, P4 single-domain negatives, R9-2 local exact-tree provenance, R9-3 through R9-12, dedicated cross-process R9 independent review, and factual release-packet/status reconciliation.

`READY_LIVE` remains none: this finding is deterministically local and creates no new real-network question.
