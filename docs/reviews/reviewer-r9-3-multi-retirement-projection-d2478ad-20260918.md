# Independent reviewer note — H-R9-048 multi-retirement Carrier projection

**Reviewed source/test anchor:** `d2478ada80bd553210aaab631c7392fcdfc2355c`

**Classification:** HIGH — process/evidence truth; auto-resolvable under current committed semantics. This finding does **not** establish a new Recovery, Session, wire, crypto, or Carrier architecture defect.

## Scope

Inspected the exact-current reliable-UDP ACK owner and its R9-3 process oracle:

- `crates/neko-reliable/src/lib.rs` — `AckRanges`, `RecoveryResult`, `Recovery::on_ack`;
- `crates/neko-cli/src/main.rs` — `UdpAcknowledgement::Carrier`, `recv_udp_delivery_ack`, post-return Carrier outcome projection;
- `crates/neko-cli/tests/probe.rs` — `reliable_udp_post_return_data_loss_recovers_via_pto_retransmit` and the H-R9-045/H-R9-046 identity/order assertions;
- `docs/CHATGPT_HANDOFF.md` — the H-R9-047 repair contract and current R9 queue.

Excluded: changing ACK/wire/crypto/Session architecture, introducing new capacity/security policy values, live/WAN experiments, release/freeze/production authority.

## Invariant challenged

Every actual positive Carrier Recovery retirement must have truthful process evidence naming the packet identity that Recovery retired. Accepted-empty is reserved for `applied=false && rejected=false`; a positive Recovery mutation must not disappear from the diagnostic projection.

The immediately preceding H-R9-047 contract explicitly required positive evidence using the actual packet identity/identities in `acked_packets` and allowed a per-retired-packet diagnostic when one ACK range retires multiple packets.

## Finding

`Recovery::on_ack` can retire more than one outstanding packet in a single call. It builds an `acked` list from every `sent` packet contained by the received `AckRanges`, removes each one, and pushes every retired packet number into `RecoveryResult.acked_packets`.

`recv_udp_delivery_ack` preserves that complete vector in `UdpAcknowledgement::Carrier { applied, acked_packets, rejected }`, with `applied=true` whenever the vector is non-empty.

At exact `d2478ad`, the post-return projection correctly stopped using mutable latest `post_pn_client` as the retirement predicate, but it still collapses a non-empty vector to one identity:

```text
let retired = !acked_packets.is_empty();
if applied && retired {
    let acked_pn = *acked_packets.last().unwrap_or(&post_pn_client);
    emit r9_udp_return_packet_ack(packet_number = acked_pn)
}
```

Therefore a valid authenticated ACK that retires two or more outstanding packet copies mutates Recovery for all of them but emits positive process evidence for only the last packet number. The omitted retirements are neither represented as positive evidence nor classified accepted-empty/rejected. The process evidence is consequently not a complete projection of the typed Recovery transition.

This matters under overlapping retransmission copies and multi-range/range ACK semantics even if the ordinary server path commonly emits singleton ACKs. The public typed owner already supports `Vec<u64>` retirement truth, and R9-5 explicitly plans adversarial feedback across every Carrier continuation.

## Why current tests do not close it

The current repeated-PTO process oracle validates the set of **emitted** `r9_udp_return_packet_ack` lines: every emitted retirement must name a client retransmit and have a matching server ACK. It does not compare emitted positive diagnostics against the complete `acked_packets` set returned by the owner. A projection that silently drops one of two real retirements can therefore remain green.

The `d2478ad` source commit changed only the CLI projection and did not add a deterministic multi-retirement regression.

## Minimal repair contract

1. In the exact-current post-return `UdpAcknowledgement::Carrier` positive branch, represent **every** packet identity in non-empty `acked_packets`. A per-retired-packet `r9_udp_return_packet_ack` is the smallest shape and is already compatible with the prior H-R9-047 contract.
2. `rejected=true` remains typed rejection. Only `applied=false && rejected=false` may emit accepted-empty. No non-empty `acked_packets` member may be silently omitted or relabeled.
3. Add a focused deterministic regression that constructs at least two simultaneously outstanding packet identities and applies one canonical ACK range/range-set that retires at least two in one owner transition. Assert exact set equality between actual retired packet identities and emitted positive evidence, zero rejection/accepted-empty for that transition, and no duplicate Session delivery transition.
4. Preserve the singleton positive case, H-R9-046 no-retransmit-after-positive-retirement invariant, stable Session/Carrier evidence separation, and all current rejection behavior.
5. Prefer the smallest CLI/test repair. Do not redesign ACK architecture and do not invent policy values. No decode/parser/crypto-framing change is required, so fuzz is not mechanically required solely for this repair.

## Verification / evidence boundary

This review is a bounded source/oracle challenge, not a reviewer-local clean-tree gate. The developer-local exact-tree gate recorded for `d2478ad` remains provenance for that tree only.

GitHub-hosted Rust CI for exact `d2478ad` is run `35288334930`, completed successfully (`stable checks` and `nightly decode fuzz smoke`). That is cross-evidence only and does not discriminate the omitted-identity projection because the existing oracle does not require completeness of the emitted retirement set.

`READY_LIVE` remains `none`: this is a local process/evidence correctness question and creates no new unresolved real-network hypothesis.
