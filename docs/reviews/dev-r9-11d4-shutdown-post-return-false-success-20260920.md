# Developer bounded R9-11D4 review — shutdown / post-return false-success negatives

**Anchor:** exact source/test tree `8cbd9af` + reviewer commits through `72c5e06`.
**Owners inspected:** `probe.rs` `invalid_bind_never_emits_ready` /
`sigterm_after_ready` / `endpoint_rebind_real_sockets_promote_new_source_and_reject_stale_old_source`;
`reliable_udp_post_return_*` family (incomplete, withheld carrier/session ACK,
stale ACK, future ACK, malformed bound, reversed order, data-loss PTO);
`neko-carrier` `old_generation_and_late_old_path_cannot_advance_watermark`;
`process-resource-sampler` `fixture.timeout-group` / `fixture.escaped-*` /
`fixture.partial-obs` terminal-result boundary.

## Per-invariant coverage

| Invariant | Coverage |
|---|---|
| shutdown during setup/traffic/recovery leaves no later READY/success/delivery event | `invalid_bind_never_emits_ready` proves a failed bind emits no READY; `sigterm_after_ready` proves SIGTERM teardown releases bindings with no post-termination READY; `reliable_udp_post_return_*` family proves no delivery/settlement event is emitted after the return boundary on any challenged path |
| timeout followed by late child/socket completion cannot overwrite terminal outcome | `fixture.timeout-group` records `timed_out=true`/`signal=15` as terminal — the child's late state does not rewrite it; `reliable_udp_post_return_incomplete_is_terminal` and `reliable_udp_post_return_malformed_bound_is_terminal` fix the post-return outcome as terminal |
| cleanup/rebind evidence belongs to the same exact run/process owner, not a stale predecessor | `owned_port_sockets_present` is keyed to the exact `--owned-port` set of this run; `endpoint_rebind_real_sockets_promote_new_source_and_reject_stale_old_source` proves a stale old-source binding is rejected while the new source is promoted |
| application/session success proof plus truthful cleanup/result state required; cleanup-only success insufficient | H-R9-084 makes the sampler exit nonzero on incomplete cleanup while `result["exit"]` preserves the measured child fact; `reliable_udp_post_return_carrier_ack_withheld_fails` / `session_ack_withheld_fails` prove withheld success proof fails even when cleanup is clean |
| failure/unknown remains failure/unknown even if a later independent postcheck finds zero residue | `fixture.partial-obs` / `fixture.escaped-*` keep `complete=false`/`owned_sockets_after_exit=None` and nonzero exit regardless of later state; `old_generation_and_late_old_path_cannot_advance_watermark` proves a late old-path observation cannot advance the watermark |

## Reachable regressions named

- `invalid_bind_never_emits_ready`
- `sigterm_after_ready_stops_and_releases_tcp_and_udp_bindings`
- `endpoint_rebind_real_sockets_promote_new_source_and_reject_stale_old_source`
- `reliable_udp_post_return_incomplete_is_terminal` / `post_return_malformed_bound_is_terminal`
- `reliable_udp_post_return_carrier_ack_withheld_fails` / `session_ack_withheld_fails`
- `reliable_udp_post_return_stale_ack_is_accepted_empty` / `future_ack_is_rejected` / `reversed_ack_order_settles`
- `old_generation_and_late_old_path_cannot_advance_watermark`
- `fixture.timeout-group` / `fixture.escaped-descendant` / `fixture.escaped-udp` / `fixture.partial-obs`

## Result

No concrete defect found across shutdown / post-return false-success negatives.
No challenged path emits a READY/success/delivery event after shutdown or the
return boundary; a late child/socket completion cannot overwrite a recorded
terminal outcome; cleanup/rebind evidence is bound to the exact run and owner;
application success proof and truthful cleanup/result state are both required —
cleanup-only or residue-free-later observations are never promoted to success.

**READY_LIVE: none** — deterministic local evidence only.
