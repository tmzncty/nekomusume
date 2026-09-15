# Reviewer R9 P2/accounting checkpoint — exact `ff295a7`

**Reviewed exact developer source/test commit:** `ff295a711bf526b1691625fc37f79aaf6ec83841`

**Scope:** current `failover-client` R9 reliable-UDP + migration-back ownership/accounting path and the built-binary `reliable_udp_migration_back_reserves_final_record` fixture. This is a bounded reviewer correctness/evidence check, not a security audit, release decision, protocol freeze, WAN result, performance conclusion, or production approval.

## Repository / CI facts

- `ff295a7` is test-only relative to its parent and changes `crates/neko-cli/tests/probe.rs`.
- GitHub-hosted `stable checks` completed successfully for exact `ff295a7`.
- GitHub-hosted `nightly decode fuzz smoke` completed successfully for exact `ff295a7`.
- No reviewer-local `scripts/check.sh` run is claimed here. Final R9-2 developer-local clean exact-tree provenance remains required by the handoff.
- No open PR and no new WAN/VPS experiment were observed in this review.

## Accepted progress

`ff295a7` adds two useful assertions to the existing positive count=4 migration-back fixture:

1. exactly one client `tcp_delivery_ack_validated` event is observed;
2. the terminal `r9_udp_post_return_settled` event is observed after both the Session DeliveryAck diagnostic and the Carrier packet-ACK diagnostic.

These are useful incremental evidence and should be preserved.

## HIGH — H-R9-020: final failover accounting contradicts the actual ownership partition

Current runtime computes the actual pre-TCP uncertain ownership as:

```text
uncertain_end = records.len() - 1          // when recovery_enabled
uncertain_start = 2                        // reliable UDP owns records 0 and 1
uncertain_count = uncertain_end - uncertain_start
```

For the current positive P2 fixture (`count=4`, 16-byte records, reliable UDP + migration-back), that is exactly one uncertain record: index 2 / offset 32. Index 3 / offset 48 is explicitly reserved/unassigned until migration-back and is then sent and confirmed by the post-return reliable-UDP owner.

However, the final `failover_accounting` diagnostic later recomputes:

```text
uncertain_records = records.len() - 2
confirmed_records = 2 + replayed_records
```

For this same successful P2 run that reports:

```text
uncertain_records = 2
uncertain_bytes = 32
replayed_records = 1
confirmed_records = 3
confirmed_bytes = 48
```

while the actual completed ownership chain is:

```text
initial reliable-UDP confirmed: offsets 0,16       -> 2 records / 32 bytes
TCP uncertain replay confirmed: offset 32          -> 1 record  / 16 bytes
post-return reliable-UDP confirmed: offset 48       -> 1 record  / 16 bytes
final confirmed total                              -> 4 records / 64 bytes
pre-fallback uncertain partition                   -> 1 record  / 16 bytes
```

So the machine-readable final accounting both overstates uncertainty and omits the successfully confirmed post-return record. This is an evidence-integrity defect on the exact path currently being prepared for release-item-3/live evidence; it must be repaired before P2 closure or any later WAN claim.

### Required smallest repair

Do not change Session/Carrier/ACK architecture or invent a new counter policy.

- Reuse the already-authoritative `uncertain_count` ownership partition for `uncertain_records` / `uncertain_bytes`.
- Track the post-return record as confirmed only after the existing dual-domain success condition has actually completed (`post_outstanding` empty and Recovery `in_flight()==0`).
- Derive final `confirmed_records` / `confirmed_bytes` from the actual completed ownership transitions, not from `records.len() - 2` formulas.
- Add a focused built-binary regression on the existing count=4 positive P2 fixture requiring the final accounting to report exactly: `udp_confirmed_records=2`, `udp_confirmed_bytes=32`, `uncertain_records=1`, `uncertain_bytes=16`, `replayed_records=1`, `replayed_bytes=16`, `confirmed_records=4`, `confirmed_bytes=64`, with duplicate/lost/conflicting still zero for this fixture.
- The regression must also ensure the existing post-return terminal event remains after both acknowledgement domains and `remaining_in_flight=0`.

## P2 C1-C4 evidence still incomplete at exact `ff295a7`

The new commit message says “P2 exact cardinality … C1-C4”, but the actual assertions remain narrower than the current handoff contract:

- **C1:** the fixture counts one client `tcp_delivery_ack_validated`, but does not assert that it is `seq=2`; it does not assert exactly one server `tcp_delivery_ack_sent` for `seq=2`; and it does not explicitly reject replay diagnostics for seq 0/1/3.
- **C2:** the fixture proves `challenge < validated < migrated_back` and separately requires a post-return send at offset 48, but does not yet assert `migrated_back < post_return_sent` or exactly-one post-return send.
- **C3:** server causal order is checked, but exact cardinality and `seq=3` identity for the post-return Session DeliveryAck are not yet asserted.
- **C4:** settled-after-both-domain order is now checked, but `udp_return_delivery_ack_validated` still carries no stream/offset identity in runtime diagnostics, so the built-binary test cannot mechanically prove that this logical confirmation is stream 1 / offset 48; exactly-once cardinality for the two acknowledgement-domain diagnostics is also not pinned.

These are READY_LOCAL evidence gaps, not reasons to redesign the protocol. Preserve the current fixture and tighten it in place after H-R9-020.

## Queue consequence

R9-3 remains blocked. The immediate order is:

1. repair H-R9-020 + focused accounting regression;
2. finish exact P2 C1-C4 in the existing fixture;
3. execute the already queued acknowledgement-order seam, automatic-health exact replay-identity regression, P3 persistent malformed budget, and P4 incomplete-settlement terminal checks;
4. persist clean exact-tree developer-local provenance;
5. continue R9-3 through R9-12, then Q10/Q11/Q12 without waiting for reviewer cadence.

`READY_LIVE` remains none until the cross-process R9 surface is completed and independently bounded-reviewed.
