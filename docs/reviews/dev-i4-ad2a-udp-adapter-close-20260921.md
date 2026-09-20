# Developer bounded I4-AD2a review — UDP adapter close/error/resource

**Anchor:** exact source/test tree `26aa4e8` + reviewer commits through `f784946`.
**Owners inspected:** `UdpSocket`/`UdpListener` `send_datagram`/`recv`/`close`/
`closed`/`local_addr`, D012 Carrier contract, adapter tests.

## Per-invariant coverage

| Invariant | Coverage |
|---|---|
| local logical close idempotent, prevents later send success | `close` is idempotent (`oversize_is_rejected_and_close_is_local_idempotent`); after close, `send_datagram` returns `Closed` |
| recv after close cannot fabricate payload/evidence | `recv` returns `Closed` after local close — no fabricated datagram |
| oversize send fails before socket mutation | `send_datagram` rejects `len > MAX_DATAGRAM_BYTES` before `send` syscall |
| oversize receive/truncation bounded | receive path respects `MAX_DATAGRAM_BYTES`; truncation cannot exceed the committed bound |
| `WouldBlock` distinct from close and I/O failure | nonblocking `WouldBlock` maps to `Ok(None)` (no datagram), not `Closed`/`Io` — `no_datagram_is_none_not_error` |
| no peer-close claim UDP cannot prove | UDP is connectionless — `closed` reports only the local flag; no peer-close semantics are fabricated |
| OS socket lifetime is ownership/drop, not just closed flag | `UdpSocket` wraps `std::net::UdpSocket` — dropping releases the FD; `close()` flips the logical flag only, socket released on drop |
| local socket release/rebind evidence not conflated with sampler truth | `sigterm_after_ready_stops_and_releases_tcp_and_udp_bindings` exercises process-level release; the adapter's own `close`/`drop` is distinct |
| no adapter observation promotes itself into SessionDelivery/PathValidated/ACK | adapter surfaces only `Carrier`/`UdpError` — no Session/ACK/release promotion |

## Reachable regressions named

- `oversize_is_rejected_and_close_is_local_idempotent`
- `udp_listener_rejects_bounded_malformed_churn_then_authenticates_and_cleans_up`
- `no_datagram_is_none_not_error` (WouldBlock ≠ close/Io)
- `close_is_idempotent_and_preserves_queued_data` (MemoryCarrier analogue — UDP `closed` flag)

## Result

No concrete defect found across UDP adapter close/error/resource semantics.
Close is idempotent and gates send/recv; oversize fails pre-mutation;
`WouldBlock` is distinct; no peer-close or Session/ACK promotion is
fabricated.

**READY_LIVE: none** — deterministic local evidence only.
