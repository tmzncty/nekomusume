# Reviewer R9 P3/P4 checkpoint — exact `fb6911a`

**Reviewed exact tree:** `fb6911afff2b8d77bc642b7f5649a5dd5f91ef0c`

**Scope:** new developer-owned changes after the prior handoff checkpoint: `9e3f4a0` (H-R9-020 accounting repair), `92e1154` (P3 malformed-budget fixture), and `fb6911a` (P4 post-return logical-ACK suppression fixture), plus the current failover server/client owners and existing R9 built-binary tests.

This is a bounded reviewer checkpoint, not a security approval, release decision, protocol freeze, WAN result, or production claim.

## Current tree / CI facts

- `main` is exact `fb6911afff2b8d77bc642b7f5649a5dd5f91ef0c` at review time.
- Hosted `stable checks` and `nightly decode fuzz smoke` both succeed on exact `fb6911a`; hosted CI remains cross-evidence only.
- Open PRs: none.
- No new WAN/VPS experiment is present in this sequence.
- Developer-local final R9-2 clean exact-tree provenance is still absent.

## Accepted progress

### H-R9-020 failover accounting — CLOSED on current tree

The client now derives `uncertain_records` from the authoritative ownership partition (`uncertain_end - uncertain_start`) and counts the reserved post-return reliable record only after the dual-domain post-return loop completes. The positive count=4 P2 fixture pins the resulting `2 UDP-confirmed / 1 uncertain / 1 replayed / 4 confirmed` accounting.

Do not reopen this unless a new reproducer contradicts the partition/accounting result.

### P4 acknowledgement-domain suppression — useful partial evidence

The existing `--suppress-r9-ack` test withholds Carrier packet ACK while allowing Session DeliveryAck, and the new `--suppress-r9-dack` seam withholds the post-return Session DeliveryAck while still allowing Carrier packet ACK. The new logical-ACK-suppression run requires a nonzero exit and forbids `r9_udp_post_return_settled`.

Keep both seams. Before R9-2 closure, tighten the post-return P4 assertion set so both suppression cases mechanically forbid all success/continuation evidence named by the handoff (`r9_udp_post_return_settled`, `r9_udp_in_flight_settled`, and downstream health/failover continuation) rather than relying on comments.

## H-R9-021 — P3 does not exercise the claimed Carrier-ACK interleaving

**Severity: HIGH for R9-2 evidence correctness. Mechanically repairable; no policy or architecture decision required.**

The required P3 sequence is:

```text
malformed #1 -> malformed #2 -> canonical Carrier packet ACK -> malformed #3
```

and malformed #3 must hit the existing operation-wide `MAX_POST_HANDSHAKE_MALFORMED` without the valid Carrier feedback resetting that budget.

Current server ordering does not do this. In the reliable receive path, `server_rt.poll_outgoing_ack(0)` is encoded/sealed/sent and `udp_packet_ack_sent` is emitted **before** the later `--malformed-budget-test` block. The test seam then sends malformed #1 and #2, sends the Session DeliveryAck, and only then sends malformed #3. Therefore the current fixture demonstrates, at best, persistence across a Session DeliveryAck; it does not demonstrate persistence across a valid Carrier packet ACK located between malformed #2 and #3.

The test assertion is also too weak for its stated contract:

```rust
assert!(
    !out.status.success()
        || client_err.contains("malformed bound")
        || client_err.contains("malformed")
        || client_log.contains("malformed_bound")
        || client_log.contains("malformed_or_unadmitted"),
    ...
);
```

A successful process can satisfy the assertion merely because any generic malformed diagnostic appears. That does not prove the third malformed input hit the bound, produced a typed terminal result, or prevented success.

### Required smallest repair

Do not change the malformed numeric limit and do not redesign ACK/Session/Carrier semantics.

1. Under the **test-only** `--malformed-budget-test` seam, defer the already-canonical Carrier packet ACK for the selected admitted packet so observed send order is exactly:
   `malformed #1 -> malformed #2 -> Carrier ACK -> malformed #3`.
   The ordinary path must remain unchanged.
2. Preserve the separate Session DeliveryAck domain; do not use it as a substitute for the Carrier ACK required by P3.
3. Add the minimum diagnostic/order evidence needed to show the Carrier ACK was actually emitted/applied between malformed #2 and #3.
4. Require `!out.status.success()` unconditionally.
5. Require a specific existing typed malformed-budget terminal marker/error; remove the disjunction that lets generic malformed output or a successful process pass.
6. Require no `r9_udp_in_flight_settled`, no `r9_udp_post_return_settled`, and no downstream health/failover continuation after the malformed bound terminates the operation.

Smallest repair -> focused built-binary regression -> commit/push -> continue immediately.

## P2 exact-evidence closure remains open

The positive `reliable_udp_migration_back_reserves_final_record` fixture now has useful process success, causal milestones, exact final accounting, one TCP ACK count, post-return send/dual-domain settlement, and final `remaining_in_flight=0`. It still does not satisfy all exact C1-C4 requirements from the current handoff:

- C1: assert the sole TCP replay/DeliveryAck identity is exactly `seq=2` / stream 1 / offset 32 on both client and server, not only `tcp_acks.len()==1`.
- C2: require strict `udp_migrated_back < r9_udp_post_return_sent(offset=48)` and exact cardinality one; forbid reserved offset 48 on every legacy uncertain/replay path.
- C3: require exact-once server owner/validation/DeliveryAck/packet-ACK events and exact reserved identity (`seq=3` / offset 48), not only first-occurrence ordering.
- C4: make the client logical post-return ACK diagnostic carry sufficient stream/offset identity, require exact-once Session and Carrier ACK evidence, and keep `r9_udp_post_return_settled` after both domains regardless of arrival order.

Do not create a parallel P2 scenario; tighten the existing count=4 fixture.

## Remaining R9-2 closure work

After H-R9-021 and exact P2 C1-C4:

- exercise post-return Session-ACK -> Carrier-ACK and Carrier-ACK -> Session-ACK arrival orders using one bounded ordering seam and one receive owner;
- challenge automatic-health replay identity against the exact `(DataId, payload)` set returned by `FailoverController::tcp_resend()`; if contiguous reconstruction is not exactly equivalent, consume exact controller ownership;
- retain both P4 acknowledgement-domain suppression cases with explicit no-success/no-continuation assertions;
- persist developer-local clean exact-tree provenance on the final pushed R9-2 SHA using `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` and `git diff --check`, recording UTC start/end, OS/arch, Rust stable version, exits, and clean initial/final tree. Decoder fuzz is only required if decoder/parser/crypto framing changes.

## Release/live boundary

R9-2 is not closed; R9-3 must not expand the surface until the HIGH above and exact-evidence closure are repaired. `READY_LIVE` remains none. VPS standing authorization is valid, but correctness/evidence dependencies are the current blocker.
