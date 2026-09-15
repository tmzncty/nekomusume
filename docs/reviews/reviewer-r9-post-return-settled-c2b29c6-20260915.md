# Reviewer R9 post-return settlement checkpoint — exact `c2b29c6`

**Reviewed source/test anchor:** `c2b29c6a140a589540c3337b822f5192a3789566`.

**Classification:** bounded reviewer/evidence checkpoint. This is not a security approval, release decision, WAN result, protocol freeze, or production claim.

## Repository truth reviewed

- `c2b29c6` is the latest developer-owned source/test commit at this checkpoint.
- It changes `crates/neko-cli/src/main.rs` and `crates/neko-cli/tests/probe.rs` only.
- GitHub-hosted `stable checks` and `nightly decode fuzz smoke` both completed successfully for exact `c2b29c6`. Hosted CI remains cross-evidence only; it is not the required developer-local final exact-tree provenance.
- No open PR and no new WAN/VPS experiment were present at review time.
- `READY_LIVE: none`; release items 3 and 4 remain incomplete; release/freeze/production flags remain false.

## Accepted progress — do not revert

### H-R9-019 exact controlled TCP replay identity remains closed

Controlled fallback still replays from the ownership partition beginning at `uncertain_start`; do not reintroduce positional `skip(1)` selection.

### Dedicated post-return settlement evidence — ACCEPT_WITH_BOUNDARIES

`c2b29c6` adds `r9_udp_post_return_settled`. In the reliable-UDP migration-back path, the event is reached only after the existing bounded post-return receive loop has observed both conditions:

- the exact logical post-return expectation is retired (`post_outstanding.is_empty()`), and
- Carrier recovery ownership is drained (`rt.in_flight() == 0`).

The positive P2 fixture now requires the exact stream/offset and `remaining_in_flight=0` on that terminal event. This closes the prior broad-substring evidence gap.

Do not move this event earlier than the dual-settlement condition.

## R9-2 is still open

The same fixture still discards `_server_status` and `_server_log`, so the current PASS proves only the client-side causal chain. The existing reviewer contract required one bounded run to establish both-process success and the server-side ownership chain. Therefore positive P2 remains **PARTIAL_ACCEPT**, not closure.

The next implementation/test slice must tighten the existing `reliable_udp_migration_back_reserves_final_record` fixture rather than create a parallel scenario. Require, from the same run:

1. client success;
2. server success;
3. exactly one TCP application replay corresponding to stream 1 / offset 32, and no replay of offsets 0, 16 or reserved 48;
4. server `udp_recovery_owner_started`;
5. server `udp_recovery_validated`;
6. server authenticated Session receive of offset 48 before acknowledgement emission;
7. server `udp_return_delivery_ack_sent` for offset 48;
8. server `udp_return_packet_ack_sent` for the post-return Carrier packet;
9. client exact Session confirmation for offset 48;
10. client `r9_udp_return_packet_ack` with `applied=true`;
11. client `r9_udp_post_return_settled` for stream 1 / offset 48 / remaining in-flight zero.

Use the existing structured events where they can express the identity exactly. If an existing event only exposes a positional sequence and cannot unambiguously prove stream/offset ownership, add the smallest diagnostic field to that existing event rather than a redundant second evidence channel.

### Diagnostic scope nit to close while touching P2

`r9_udp_post_return_settled` is currently emitted even when the post-return path has no `ReliableUdpRuntime`; `map_or(0, ...)` then reports `remaining_in_flight=0` by absence. The R9 evidence event should not imply recovery settlement when no recovery owner exists. Gate the R9 reliable-settlement event on the reliable owner being present, or otherwise make the absence explicit without changing Session/Carrier/ACK semantics.

This is an evidence-integrity correction, not a protocol architecture change.

## Remaining R9-2 closure work

After the both-process positive P2 tightening, continue immediately:

1. **Post-return ACK-order seam:** one bounded test seam must exercise both `Session DeliveryAck -> Carrier ACK` and `Carrier ACK -> Session DeliveryAck`, both reaching the same terminal settlement event. No sleep-based ordering or second receive owner.
2. **Automatic-health replay exact-identity regression:** compare actual replay identities/payloads with exact `FailoverController::tcp_resend()` ownership. If equivalent, record the no-finding regression; if not, consume exact ownership rather than cardinality.
3. **P3 persistent malformed budget:** in one operation drive `malformed #1 -> malformed #2 -> canonical Carrier ACK -> malformed #3`; the third malformed input must hit the existing operation-wide bound and the valid ACK must not reset it.
4. **P4 incomplete settlement negatives:** suppress each acknowledgement domain separately; require typed nonzero termination and no settled-success event while logical or Carrier ownership remains outstanding.
5. **Final exact-tree local provenance:** on the final pushed R9-2 source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, verify clean initial/final tree, and persist exact SHA, UTC start/end, OS/arch, stable Rust version and exit codes. Run pinned decoder fuzz only if decoder/parser/crypto framing changes.

Then continue the existing R9-3 through R9-12 and Q10/Q11/Q12 queue without waiting for reviewer cadence.

## No new BLOCKER/HIGH in this bounded checkpoint

The inspected `c2b29c6` change does not introduce a reproduced correctness/security BLOCKER/HIGH. R9-3 remains dependency-blocked on completing the R9-2 evidence/negative matrix and final local exact-tree gate, not on WAN permission.

The previously highlighted current-head candidates remain closed: `Recovery::on_ack` rejects future/never-sent largest ACKs atomically before mutation, and observability datagram drops classify `queue_dropped` only as a subset of generic dropped events rather than relabeling all drops.
