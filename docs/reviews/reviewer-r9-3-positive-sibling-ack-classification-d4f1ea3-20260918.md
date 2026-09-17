# Independent bounded review — R9-3 positive sibling Carrier-ACK classification

**Reviewed developer source/test anchor:** exact `d4f1ea378740b96398a140b98f568ab9a8982b03`  
**Repository navigation head at review start:** `46a905a93f61d8866bd2cd854f41a08da605edbb`  
**Classification:** `HIGH` evidence/process-truth correctness; current semantics already determine the repair; no maintainer policy/architecture decision required.

## Scope inspected

- `crates/neko-cli/src/main.rs`
  - `UdpAcknowledgement::Carrier` contract and `recv_udp_delivery_ack` outcome construction;
  - post-return PTO/retransmit loop;
  - post-return Carrier ACK classification/diagnostic branch.
- `crates/neko-reliable/src/lib.rs`
  - `Recovery::on_ack` positive retirement semantics and `RecoveryResult::acked_packets`.
- `crates/neko-cli/tests/probe.rs`
  - `reliable_udp_post_return_data_loss_recovers_via_pto_retransmit` after `0766dd8`, `68e364a`, and `d4f1ea3`.
- New developer-owned commits reviewed in this bounded pass:
  - `0766dd8559d44fb7aa4f43764f794f560c3a8c70` — repeated-PTO packet identity/value binding;
  - `68e364a45466ce958892d536aeea450678919401` — settlement ordering after Session + Carrier transition;
  - `d4f1ea378740b96398a140b98f568ab9a8982b03` — no retransmit diagnostic after the lifecycle-resolving positive Carrier retirement.

Hosted Rust CI for exact `d4f1ea3` is separately green at run `35283642218` (`stable checks` and `nightly decode fuzz smoke`). This is hosted cross-evidence only; it does not repair the oracle/classification defect below.

## Invariant challenged

`UdpAcknowledgement::Carrier` defines three disjoint outcomes:

1. `rejected=true`: fail-closed typed rejection, Recovery unchanged;
2. `applied=true` with non-empty `acked_packets`: Recovery positively retired one or more packet identities;
3. `applied=false && rejected=false`: authenticated accepted-empty stale/duplicate classification, Recovery did not retire a packet.

A positive `acked_packets` result must therefore never be reclassified as accepted-empty merely because it does not contain the *latest retransmission packet number*.

## Finding H-R9-047 — positive older sibling ACK can be falsely labeled accepted-empty

The current repeated-PTO path updates mutable `post_pn_client` to every fresh retransmission packet number. The receive branch later computes:

```text
retired = acked_packets.contains(post_pn_client)
if applied && retired => r9_udp_return_packet_ack
else if rejected      => r9_udp_return_packet_ack_rejected
else                   => r9_udp_return_packet_ack_accepted_empty
```

But `recv_udp_delivery_ack` constructs `applied=true` exactly when `Recovery::on_ack` returns a non-empty `acked_packets`. Under legal repeated PTO there may be sibling packet copies in flight. If retransmit `pn1` is sent, then retransmit `pn2` becomes the current `post_pn_client`, and an authenticated ACK for `pn1` arrives afterwards, Recovery truth is positive (`applied=true`, `acked_packets=[pn1]`) while the diagnostic branch sees `pn1 != post_pn_client` and emits `r9_udp_return_packet_ack_accepted_empty`.

That is a false process classification: state changed and a real Carrier packet identity retired, but the structured event says no packet retirement occurred. The same shape can also under-report an ACK range that positively retires more than the one current/latest packet identity.

This does **not** establish a new Recovery state-machine defect. `Recovery::on_ack` already returns the authoritative retired packet identities. The defect is in the CLI projection from that typed result to process evidence.

## Why existing R9-3 tests do not close it

- `0766dd8` value-binds packet numbers only for events that were emitted as `r9_udp_return_packet_ack`; it cannot detect a real positive sibling retirement that the source incorrectly projected as accepted-empty.
- `68e364a` orders settlement after the first emitted positive retirement, not necessarily after the first actual positive Recovery retirement if an earlier sibling was misclassified.
- `d4f1ea3` proves no retransmit diagnostic occurs after the *emitted* lifecycle-resolving positive event; it does not challenge the projection of a positive non-latest sibling ACK.
- H-R9-044 intentionally permits legitimate accepted-empty sibling/late feedback, so accepted-empty presence alone is not discriminating.

## Smallest repair contract

Keep Recovery/Session/wire/crypto semantics unchanged. Fix only the typed outcome projection:

- `rejected=true` => rejection diagnostic;
- otherwise, if `applied=true`, emit positive retirement evidence using the packet identity or identities actually present in `acked_packets` (if one ACK retires multiple packets, do not silently collapse them into accepted-empty; per-retired-packet events are an acceptable minimal shape);
- only `applied=false && rejected=false` may emit `r9_udp_return_packet_ack_accepted_empty`.

Do not use mutable `post_pn_client` as the truth predicate for whether Recovery applied an ACK. It may remain as send/navigation state if separately needed.

Add a deterministic discriminating regression at the exact-current owner: create at least two overlapping retransmission packet identities for the same frame, make the second one current/latest, then deliver a valid authenticated Carrier ACK that retires only the earlier sibling. Require:

- Recovery positive retirement for the earlier packet identity;
- positive `r9_udp_return_packet_ack` evidence names that actual earlier packet number;
- that datagram is **not** labeled accepted-empty or rejected;
- Session DeliveryAck remains an independent exactly-once transition;
- after the first positive ACK for the frame, no later retransmit is emitted for that frame;
- a genuinely stale/duplicate later sibling ACK may still classify accepted-empty without creating a second Session transition.

Prefer a bounded process regression if the existing test seam can deterministically stage the order. A tiny classification helper/unit regression may supplement it but must not replace process evidence for R9-3.

No fuzz run is required solely for this diagnostic/test repair unless wire decoder/parser/crypto framing code changes.

## Queue impact

H-R9-047 becomes the first dependency-ready local slice. H-R9-041 exact-wire refusal and H-R9-042 pre-deadline negative remain valid and follow immediately after it. R9-4 through R9-12, dedicated independent R9 review, Q10/Q11/Q12 reconciliation, and repository-wide item-4 refill remain valid downstream work. `READY_LIVE` remains `none`.
