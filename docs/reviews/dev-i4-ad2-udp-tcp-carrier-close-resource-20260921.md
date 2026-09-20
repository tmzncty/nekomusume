# Developer bounded I4-AD2 review — UDP/TCP carrier close/error/resource semantics

**Anchor:** exact source/test tree `8cbd9af` + reviewer/dev commits through `9288b83`.
**Owners inspected:** `neko-carrier` `UdpLoopbackPair` send/recv/close,
`closed_socket_rejects_send_and_recv_cleans_up`, `oversize_is_rejected_and_close_is_local_idempotent`,
`authenticated_encrypted_loopback_echo_and_close`, `bounded_long_session_preserves_exact_order_through_uncertain_ack_and_close`;
`neko-cli` `sigterm_after_ready_stops_and_releases_tcp_and_udp_bindings`,
`endpoint_rebind_real_sockets_promote_new_source_and_reject_stale_old_source`,
`migration_back_tamper_fails_closed_before_return`,
`warm_readiness_failures_close_before_admission_or_application_data`,
`lab_scenario_arguments_fail_closed_before_sockets`.

## Per-invariant coverage

| Invariant | Coverage |
|---|---|
| listener/socket ownership and release | `sigterm_after_ready` proves SIGTERM releases TCP and UDP bindings — rebinding the same port fails while the process lives and succeeds after teardown; `endpoint_rebind_real_sockets` proves a stale old source is rejected while the new source is promoted |
| connect/accept/send/recv errors are fail-closed | `closed_socket_rejects_send_and_recv_cleans_up`; `migration_back_tamper_fails_closed_before_return`; `lab_scenario_arguments_fail_closed_before_sockets` — invalid input is rejected before any socket is created |
| shutdown/rebind/resource release and bounded cleanup | `oversize_is_rejected_and_close_is_local_idempotent` proves close is idempotent and local; `sigterm_after_ready` proves SIGTERM releases both bindings; `warm_readiness_failures_close_before_admission` proves a failed warm path closes before admission |
| adapter resource truth distinct from process-sampler evidence | adapter-level close/rebind is exercised by `neko-cli`/`neko-carrier` socket tests; the `process-resource-sampler` oracle is a separate terminal-truth boundary — neither substitutes for the other |

## Reachable regressions named

- `closed_socket_rejects_send_and_recv_cleans_up`
- `oversize_is_rejected_and_close_is_local_idempotent`
- `authenticated_encrypted_loopback_echo_and_close`
- `bounded_long_session_preserves_exact_order_through_uncertain_ack_and_close`
- `sigterm_after_ready_stops_and_releases_tcp_and_udp_bindings`
- `endpoint_rebind_real_sockets_promote_new_source_and_reject_stale_old_source`
- `migration_back_tamper_fails_closed_before_return`
- `warm_readiness_failures_close_before_admission_or_application_data`
- `lab_scenario_arguments_fail_closed_before_sockets`

## Result

No concrete defect found across UDP/TCP carrier close/error/resource semantics.
Owned listeners are released on SIGTERM/close; rebinding is impossible while a
binding lives and succeeds after release; send/recv/connect errors fail closed
before socket creation or state mutation; adapter resource truth remains
distinct from benchmark process-sampler evidence.

**READY_LIVE: none** — deterministic local evidence only.
