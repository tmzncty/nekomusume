# Developer bounded I4-AD2a review — UDP adapter close/error/resource semantics

**Anchor:** exact source/test tree `26aa4e8` + reviewer/dev commits through `168584e`.
**Owners inspected:** `neko-carrier` `UdpLoopbackEndpoint`/`UdpLoopbackPair`/
`send_datagram`/`recv_datagram`/`close`/`closed`/`local_addr`;
`neko-cli` `sigterm_after_ready_stops_and_releases_tcp_and_udp_bindings`,
`endpoint_rebind_real_sockets_promote_new_source_and_reject_stale_old_source`.

## Per-invariant coverage

| Invariant | Coverage |
|---|---|
| local logical close is idempotent and prevents later send success | `oversize_is_rejected_and_close_is_local_idempotent` — `close()` twice is `Ok(())`; post-close `send_datagram` returns `Io(NotConnected)` |
| recv after local logical close cannot fabricate payload/evidence | `recv_datagram` returns `Ok(None)` when `closed` is set — no bytes are synthesized; the peer's queued datagrams are not drained into evidence |
| oversize send fails before socket mutation | `send_datagram` checks `m.len() > max_datagram_bytes` before touching `self.socket.send` — `MessageTooLarge` is rejected before any I/O |
| nonblocking `WouldBlock` distinct from close and I/O failure | `recv_datagram` maps `WouldBlock` to `Err(UdpError::WouldBlock)` — distinct from `Ok(None)` (logical close) and `Err(Io)` (I/O failure) |
| connected loopback endpoint does not claim peer-close semantics UDP cannot prove | `send_datagram` on a closed local endpoint fails `Io(NotConnected)`; there is no peer-close detection — UDP is connectionless, `recv` returns `WouldBlock`/`None`, never a fabricated "peer closed" signal |
| OS socket lifetime is Rust ownership/drop, not merely the logical closed flag | `closed` is a `Mutex<bool>` flag; `close()` sets it without touching the `UdpSocket` fd — fd release is `Drop` on `UdpSocket`, which occurs when the `UdpLoopbackEndpoint` is dropped. `close()` is logical teardown only. |
| local deterministic listener/socket release or same-address rebind evidence is not conflated with process-sampler truth | `sigterm_after_ready` proves SIGTERM releases the UDP binding via process teardown (not `close()`); `endpoint_rebind` proves a stale old source is rejected while the new source is promoted — neither substitutes for the process-resource-sampler terminal-truth oracle |
| no adapter observation promotes itself into SessionDelivery/PathValidated/ACK/release/WAN evidence | `UdpLoopbackEndpoint` is a test/bench carrier; `recv_datagram`/`send_datagram` produce `UdpError`/`CarrierError`, not Session delivery events |

## Reachable regressions named

- `oversize_is_rejected_and_close_is_local_idempotent`
- `loopback_preserves_boundaries_empty_and_would_block`
- `authenticated_encrypted_loopback_echo_and_close`
- `sigterm_after_ready_stops_and_releases_tcp_and_udp_bindings`
- `endpoint_rebind_real_sockets_promote_new_source_and_reject_stale_old_source`

## Result

No concrete defect found across UDP adapter close/error/resource semantics.
`close()` is idempotent logical teardown (the fd is released on `UdpSocket`
drop, not by `close()`); post-close send fails `Io(NotConnected)` and recv
returns `Ok(None)`; oversize send is rejected before socket mutation;
`WouldBlock` remains distinct from close and I/O failure; no peer-close or
Session-delivery evidence is fabricated.

**READY_LIVE: none** — deterministic local evidence only.
