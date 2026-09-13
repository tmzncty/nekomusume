# Independent bounded carrier adapter close/error/resource review — exact `8e11de0`

Bounded independent review of the carrier adapters in `crates/neko-carrier/src/lib.rs` — `MemoryEndpoint`/`MemoryPair` (`:705-811`), `UdpLoopbackEndpoint`/`UdpLoopbackPair` (`:560-669`), `TcpLoopbackEndpoint` (`:1034-1180`), and the `Carrier`/`UdpCarrier`/`TcpCarrier`/`FailoverController` commit surface (`:1497-1541`) — at reachable exact `8e11de0e5915efa43591069172b97f311aae2c6e`. Loopback/memory adapters are deterministic local state models; this is not a WAN claim, not Session delivery, not an independent security/release approval.

## Challenged invariants and result

- **Max-message/max-buffer checked before allocation/enqueue — holds.**
  - `UdpLoopbackEndpoint::send_datagram` rejects `len > max_datagram_bytes` before any socket call (`:640`); `recv_datagram` allocates `max_datagram_bytes + 1` and rejects an oversized datagram as `MessageTooLarge` (`:657`).
  - `MemoryEndpoint::send` rejects `len > max_message_bytes` (`:773`), then under the pair mutex checks local/peer closed flags, then `checked_add` against `max_queue_bytes` (`:785-789`) — all before `push_back`.
  - `TcpLoopbackEndpoint::send_frame` rejects closed then `len > max_frame_bytes` (`:1128-1133`) before encoding; `recv_frame` rejects `length > max_frame_bytes` (`:1156`) **before** allocating the frame buffer, so a hostile length header cannot drive a large allocation.
- **Close idempotent where claimed — holds.** `MemoryEndpoint::close` sets `closed[side]` idempotently (`:805-809`); `TcpLoopbackEndpoint::close` shuts down once under the flag (`:1169-1178`); `UdpLoopbackEndpoint::close` sets the flag (`:665-667`). `close_is_idempotent_and_preserves_queued_data` and `oversize_is_rejected_and_close_is_local_idempotent` cover it.
- **Peer-close/local-close/error mapping cannot become Session/path evidence — holds.** Close only flips a flag; send on a closed local/peer endpoint returns `Closed`/`PeerClosed`, and none of these paths touch Session delivery or path validation.
- **Queued-data drain semantics match each adapter's contract — holds.** `MemoryEndpoint` close preserves the peer's incoming queue (`recv` never checks `closed`, only pops the queue), so the peer drains pre-close data then sees an empty queue; `recv` decrements `queue_bytes` with `checked_sub` (`:799-801`). UDP close stops `send` and makes `recv` return `Ok(None)` — acceptable for a connectionless datagram adapter with no drain contract.
- **Poison/I/O/truncation fail closed without hidden mutation — holds.** Mutex poison maps to `StatePoisoned` before any mutation; TCP `UnexpectedEof` maps to `Truncated`; all send paths validate limits and flags before touching queues or sockets.
- **Native semantics are not silently strengthened — holds.** `properties()` reports UDP `{message_boundaries, !reliable, !ordered}`, memory `{!reliable, ordered}`, TCP `{reliable, ordered}` — no adapter claims a guarantee it does not provide.

## Evidence

- `cargo test -p neko-carrier` on exact `8e11de0`: all adapter/failover/fault tests pass (`close_is_idempotent_and_preserves_queued_data`, `oversize_is_rejected_and_close_is_local_idempotent`, `tcp_framing_preserves_empty_and_boundaries`, `deterministic_blackhole_loss_and_close_are_bounded`, `migration_back_owner_requires_tcp_and_is_idempotent`, `capabilities_do_not_duplicate_tcp_packet_ack`).
- No code change; no defect found. No new `READY_LIVE` question; release/governance state unchanged.
