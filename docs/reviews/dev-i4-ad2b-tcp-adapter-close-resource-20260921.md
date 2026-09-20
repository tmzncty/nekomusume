# Developer bounded I4-AD2b review — TCP adapter close/error/resource semantics

**Anchor:** exact source/test tree `26aa4e8` + reviewer/dev commits through `5f24cbf`.
**Owners inspected:** `neko-carrier` `TcpLoopbackEndpoint`/`TcpLoopbackPair`/
`send_frame`/`recv_frame`/`close`/`is_closed`/`local_addr`;
`neko-cli` `sigterm_after_ready_stops_and_releases_tcp_and_udp_bindings`,
`endpoint_rebind_real_sockets_promote_new_source_and_reject_stale_old_source`,
`migration_back_tamper_fails_closed_before_return`,
`warm_readiness_failures_close_before_admission_or_application_data`.

## Per-invariant coverage

| Invariant | Coverage |
|---|---|
| logical close is real OS shutdown, not merely a flag | `close()` performs `TcpStream::shutdown(Both)` inside the `closed` mutex before setting the flag — the peer observes EOF; `close()` is idempotent (second call is `Ok(())` without re-shutdown) |
| post-close send/recv fail closed | `send_frame`/`recv_frame` return `TcpError::Closed` when `closed` is set — no bytes are written or read after logical close |
| buffered/pre-close data is drained before EOF | `shutdown(Both)` only after the flag is set; `recv_frame` reads complete length-prefixed frames — a partial header or partial body yields `Truncated`, not a silently truncated frame |
| partial/read framing errors are truthful | `recv_frame` maps `UnexpectedEof` on header or body to `Truncated`; `FrameTooLarge` is rejected before allocating the frame buffer |
| peer close ordering | peer `close()` produces `UnexpectedEof` on the local read → `Truncated`; local `close()` produces `Closed` on local ops — neither is confused with the other |
| accepted-stream/listener lifetime | `TcpLoopbackPair::new` binds a `TcpListener`, accepts one connection, and drops the listener — only the accepted `TcpStream` is retained; loopback-only is enforced (`InvalidLimits` on non-loopback peer) |
| FD release and local deterministic rebind | `TcpStream`/`TcpListener` are `Drop`-released; `sigterm_after_ready` proves SIGTERM releases the TCP binding — rebinding the same port fails while the process lives and succeeds after teardown |
| no Session delivery inferred from TCP reliability | `TcpLoopbackEndpoint` is a framed byte carrier; `properties()` reports `reliable: true, ordered: true` at the Carrier level only — no ACK/SessionDelivery evidence is produced |

## Reachable regressions named

- `tcp_framing_preserves_empty_and_boundaries`
- `tcp_and_udp_reject_malformed_unsupported_and_duplicate_negotiation_before_echo`
- `tcp_and_udp_reject_unsupported_selected_version_before_noise`
- `sigterm_after_ready_stops_and_releases_tcp_and_udp_bindings`
- `endpoint_rebind_real_sockets_promote_new_source_and_reject_stale_old_source`
- `migration_back_tamper_fails_closed_before_return`
- `warm_readiness_failures_close_before_admission_or_application_data`

## Result

No concrete defect found across TCP adapter close/error/resource semantics.
`close()` performs a real `shutdown(Both)` + flag under the mutex; post-close
ops fail `Closed`; partial framing is `Truncated`, not silently truncated;
listener lifetime ends at accept; loopback-only is enforced; no Session
delivery evidence is fabricated from TCP reliability.

**READY_LIVE: none** — deterministic local evidence only.
