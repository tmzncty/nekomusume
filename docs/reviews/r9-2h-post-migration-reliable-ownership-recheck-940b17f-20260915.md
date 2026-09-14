# R9-2H post-migration reliable ownership recheck — exact `940b17f`

**Reviewed revision:** `940b17f5450314acae1b87dd63440b540185395c` (`fix(cli): post-migration reliable-UDP ownership for reserved record (H-R9-015)`).

**Review type:** independent bounded source/evidence recheck of H-R9-015. This is not a release/security approval, WAN result, protocol freeze, or production claim.

## Verdict

`940b17f` is **useful partial repair, but H-R9-015 remains HIGH / OPEN**.

Hosted GitHub checks on the exact source SHA are green (`stable checks`, `nightly decode fuzz smoke`). They are supplementary evidence only. No developer-local exact-tree provenance note for `940b17f` is accepted by this review.

## What `940b17f` fixes

The client post-migration reserved record now:

1. reuses the existing `ReliableUdpRuntime` when `--reliable-udp` is active;
2. requires `can_send` before reliable ownership registration;
3. calls `on_packet_sent` with packet number, ciphertext byte count, stable `FrameId(post_record.offset)`, and retained plaintext before `UdpSocket::send_to`;
4. passes `rt.as_mut()` into the post-return authenticated receive helper rather than `None`.

That closes the **client send-ownership bypass** portion of H-R9-015.

## Remaining HIGH — server post-return packet is still not Carrier-ACK owned

The existing primary reliable-UDP server path already demonstrates the required layering: after authenticated `ProcessMessage::Data` reaches `SessionRuntime::receive`, it extracts the received packet number, calls `server_rt.on_packet_received(pn, true)`, independently emits the Session `DeliveryAck`, and separately obtains `server_rt.poll_outgoing_ack(0)` to encode/seal/send a Carrier `RecordType::Ack`.

The migration-back post-return server owner does **not** do that. It authenticates the datagram, decodes `ProcessMessage::Data`, feeds `SessionRuntime::receive`, emits the Session `DeliveryAck`, and marks migration-back complete. It never calls `server_rt.on_packet_received` for that post-return packet and never emits the canonical Carrier packet ACK.

Therefore the client's newly registered Recovery packet has no truthful post-return server Carrier-ACK obligation. H-R9-015 is not closed.

## Remaining HIGH — client completion is order-sensitive and does not require Carrier settlement

The post-return client currently invokes `recv_udp_delivery_ack(...)` once and immediately pattern-matches the first typed result:

- `Session` -> apply `SessionRuntime::delivery_ack` and continue;
- `Carrier` -> fail with `post-return path requires a Session DeliveryAck, got a Carrier ACK`.

Once the server is repaired to emit both independent acknowledgements, either authenticated datagram may arrive first. A valid Carrier ACK arriving before the Session DeliveryAck must not make the operation fail.

Also, after accepting the Session DeliveryAck, the current post-return path does not require `rt.in_flight() == 0` before success. A missing/suppressed Carrier ACK could leave Recovery ownership outstanding while the command continues as if post-return reliable delivery were complete.

This violates the existing committed separation of packet feedback and Session delivery: both domains must settle independently; neither may stand in for the other.

## Smallest repair contract

Do not change wire grammar, Session semantics, migration policy, ACK architecture, or any numeric resource policy.

1. **Server:** reuse the already-existing primary reliable-UDP receive pattern for the post-return Data. Only after authentication + exact Data decode + successful `SessionRuntime::receive`, derive the received packet number, call `server_rt.on_packet_received(pn, true)`, and emit the canonical authenticated Carrier ACK separately from the Session DeliveryAck. No ACK for malformed/tampered/unadmitted/non-Data input.
2. **Client:** make the post-return bounded owner wait under one existing absolute deadline and one malformed budget until **both** conditions are true:
   - the exact reserved Session record has been logically confirmed once;
   - `rt.in_flight() == 0` for the post-return Recovery ownership.
3. A Carrier ACK may arrive before or after the Session DeliveryAck. Apply/reject it through `ReliableUdpRuntime` with typed evidence; do not treat arrival order as a protocol error.
4. A Session DeliveryAck may not retire Carrier in-flight state, and a Carrier ACK may not advance Session delivery.
5. Timeout/malformed-bound/receive failure with either domain incomplete is terminal/typed and must not emit a success/settled claim.
6. Keep the exact reserved identity (`stream=1`, current fixture offset `32`) visible in process evidence so P2 can prove no pre-promotion ownership, one post-promotion reliable registration/send, one exact Session confirmation, independent Carrier settlement, and final `in_flight=0`.

## Evidence still required before R9-2H closure

The existing handoff requirements remain active:

- P1 exact built-process reversed Session ACK evidence;
- P2 exact reserved post-migration reliable ownership and dual settlement;
- P3 operation-wide malformed budget across a valid Carrier ACK;
- P4 incomplete settlement remains terminal;
- one reachable source/test SHA with developer-local clean exact-tree `scripts/check.sh` + `git diff --check` provenance.

R9-3 remains blocked until those are closed. The downstream R9-3..R9-12 and Q10/Q11/Q12 queue remains valid and should continue immediately afterward without waiting for reviewer cadence.

## Exclusions

No D019/source-retention decision, no new capacity/TTL/LRU values, no WAN/VPS execution, no HY2 rerun, no release/RC/freeze decision, and no crypto/wire redesign were performed in this review.
