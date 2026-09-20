# Developer bounded R9-11D2 review — TCP/UDP listener & socket ownership

**Anchor:** exact source/test tree `8cbd9af` + reviewer commits through `5ebc681`.
**Owners inspected:** `failover-server`/`failover-client`/`failover-adapter`
socket bind/close paths, `sigterm_after_ready` SIGTERM fixture,
`udp_listener_rejects_bounded_malformed_churn`, `process-resource-sampler`
`owned_port_sockets_present` oracle + escaped TCP/UDP fixtures +
`--net-dir` partial-observation seam.

## Per-invariant coverage

| Invariant | Coverage |
|---|---|
| success/failure/timeout/shutdown release owned sockets | `sigterm_after_ready_stops_and_releases_tcp_and_udp_bindings` proves SIGTERM releases both bindings — port rebinding fails while the process lives and succeeds after teardown |
| cleanup proved by owner/socket evidence or same-port rebind, never child exit / PGID emptiness alone | `owned_port_sockets_present` is an independent `/proc/net` oracle; `owned_sockets_after_exit=0` requires `group_empty AND absent`; `owned_sockets_after_exit=None`/`complete=false` on present/unknown (H-R9-082/083) |
| escaped/reparented descendant cannot retain TCP or UDP while cleanup reports zero | `fixture.escaped-descendant` (TCP) and `fixture.escaped-udp` (UDP) prove a `setsid()` descendant holding an owned socket yields `complete=false`, `owned_sockets_after_exit=None`, and a nonzero sampler exit (H-R9-084) |
| partial setup failure cannot strand a bound listener while returning success | partial `/proc/net` observation is `unknown` → `complete=false` + nonzero sampler exit; `--net-dir` seam drives the result-construction oracle |
| repeated close/shutdown idempotent, no false-positive lifecycle evidence | `close` is idempotent (`oversize_is_rejected_and_close_is_local_idempotent`); terminal results are not rewritten (H-R9-084) |
| positive rebind/absence belongs to the same run/owned port | `owned_port_sockets_present` is keyed to the exact `--owned-port` set of this run, not a stale predecessor |

## Reachable regressions named

- `sigterm_after_ready_stops_and_releases_tcp_and_udp_bindings`
- `udp_listener_rejects_bounded_malformed_churn_then_authenticates_and_cleans_up`
- `oversize_is_rejected_and_close_is_local_idempotent`
- `fixture.escaped-descendant` / `fixture.escaped-udp` (sampler test)
- `fixture.partial-obs` (sampler test — result construction under unknown)

## Result

No concrete defect found across TCP/UDP listener and socket ownership.
Owned sockets are released on every terminal path; cleanup is evidenced by
an independent owner/socket oracle; escaped/partial states cannot be
certified as cleaned up.

**READY_LIVE: none** — deterministic local evidence only.
