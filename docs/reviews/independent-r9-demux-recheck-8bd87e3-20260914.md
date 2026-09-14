# Independent R9-2E demux recheck — exact `8bd87e3`

**Reviewed source revision:** `8bd87e388745ae175f8b188f13936f1c1fe86ea4` (`fix(cli): R9-2E bounded authenticated receive/demux owner`).

**Current docs-only descendant at review time:** `42f0c9f14321994f17e6d91795743d62f6acada1`; the code blob for `crates/neko-cli/src/main.rs` is unchanged from `8bd87e3`.

**Review scope:** the R9 reliable-mode authenticated receive/demux owner, its exact source/caller behavior, the new focused regressions, the recorded local gate, and the prior reviewer contract in `docs/CHATGPT_HANDOFF.md`. This is a bounded correctness/evidence review, not a security audit, WAN run, release decision, or policy decision.

## Verdict

`8bd87e3` **does repair the original order-sensitive Session DeliveryAck match**: an authenticated DeliveryAck can now match any outstanding reliable-owned logical record, and the caller applies `SessionRuntime::delivery_ack` to the record that actually matched. The previous `let _ = rt.apply_ack(...)` at that receive boundary is also removed.

However R9-2 is **not closed**. Two HIGH correctness/evidence defects remain in the receive-owner shape, and the required built-binary/process evidence is incomplete.

## H-R9-006 — HIGH — malformed budget resets across demux returns

`recv_udp_delivery_ack` declares `let mut malformed = 0usize` inside the helper. The helper returns immediately not only for a matched Session DeliveryAck, but also for every canonical Carrier ACK, whether `apply_ack` is accepted or rejected. The caller then invokes the helper again while outstanding logical confirmations remain.

Therefore the advertised finite authenticated malformed/ignored budget is not a budget over the reliable receive owner. A peer can, for example, send two authenticated malformed/unexpected plaintexts, then one canonical Carrier ACK, then repeat. The Carrier ACK returns to the caller and the next helper invocation resets `malformed` to zero. Rejected Carrier ACKs also cause the reset because they are returned as `UdpAcknowledgement::Carrier { applied: false }`.

The absolute application deadline still bounds wall-clock time, but it does **not** make `MAX_POST_HANDSHAKE_MALFORMED` truthful as the finite invalid-input count claimed by the handoff and provenance note.

### Required repair

Keep malformed/ignored accounting owned by the entire reliable receive/settlement operation, not one helper invocation. Either:

- implement one stateful demux loop that owns outstanding logical ACKs, recovery settlement, malformed count, and typed counters until both Session confirmations and Carrier in-flight settlement complete; or
- pass a persistent mutable budget/counter state through every classification call, including settlement.

A valid or rejected Carrier ACK must not reset the count of prior malformed authenticated plaintext.

Add a discriminating regression that interleaves malformed authenticated plaintext with a canonical Carrier ACK and proves the third malformed input still trips the bound across the ACK boundary.

## H-R9-007 — HIGH — settlement is a second, untyped receive owner

After `outstanding` becomes empty, `failover_client` leaves `recv_udp_delivery_ack` entirely and enters a second settlement loop that reparses authenticated datagrams itself.

That loop:

- applies only canonical Carrier ACKs;
- silently ignores authenticated non-ACK plaintext instead of consuming the same malformed/ignored budget and typed classification path;
- records only rejected `apply_ack` outcomes locally; successful Carrier ACK applications in this phase are not added to `packet_ack_applied`;
- emits `r9_udp_packet_ack_outcomes` **before** settlement, so the emitted applied/rejected totals can omit the ACKs that actually drain the remaining in-flight state.

This contradicts the R9-2E invariant that every successfully authenticated plaintext is classified exactly once by one bounded owner and makes the typed packet-ACK outcome summary incomplete.

### Required repair

Use the same classifier/owner for both logical-confirmation and recovery-settlement phases. Do not duplicate the Carrier-ACK decoder in a second loop. Emit final packet-ACK outcome totals only after settlement is complete or explicitly partial/timed out.

The completion condition should be explicit: all required reliable-owned Session DeliveryAck expectations retired **and** `ReliableUdpRuntime::in_flight()==0`; otherwise end with typed bounded timeout/partial failure.

## M-R9-008 — MEDIUM evidence gap — required process regressions not yet landed

The new regressions are unit tests inside `crates/neko-cli/src/main.rs` using real loopback UDP sockets and authenticated `SecureSession` pairs. They are useful and discriminating for the matching helper, but the prior reviewer contract explicitly required built-binary/process coverage of:

1. reversed order of the two reliable-owned Session DeliveryAcks through the real failover command path; and
2. `--reliable-udp + migration-back` reserved-final-record ownership, proving the reserved final record is neither reliable-tracked nor legacy wire-sent before post-promotion return authorization.

`8bd87e3` changes only `crates/neko-cli/src/main.rs`; it adds no built-binary/process test file change. The provenance note itself explicitly excludes the migration-back reserved-record process regression from its claims.

These tests remain required before R9-2 closure and R9-3 progression.

## Accepted parts

- Order-independent exact `(session, stream, offset, len)` Session DeliveryAck matching for the currently bounded outstanding set.
- Caller applies logical confirmation to the actual matched `OutboundRecord`, not the formerly assumed first record.
- Unexpected logical ACK is typed and bounded **within one helper invocation**.
- Carrier `apply_ack` rejection is no longer silently discarded at the first receive boundary.
- Current code/local gate is green; current hosted stable/fuzz checks are additional cross-evidence only.

## Next queue order

1. Repair H-R9-006 and H-R9-007 as one coherent receive-owner change.
2. Add the interleaved malformed/Carrier-ACK budget regression.
3. Add/complete the two required built-binary/process R9-2F regressions.
4. Run exact pushed-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and clean-tree provenance.
5. Reviewer rechecks actual diff/tests. Only then proceed to R9-3 Data-loss recovery and retain the remaining R9-4..R9-12 / Q10..Q12 queue.

No D019, retention, capacity/security numeric, wire/crypto architecture, or release-authority decision is needed for these repairs.
