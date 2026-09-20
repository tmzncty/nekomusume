# Developer bounded R9-11D3 review — child/process-group/temp-runtime cleanup

**Anchor:** exact source/test tree `8cbd9af` + reviewer commits through `72c5e06`.
**Owners inspected:** `process-resource-sampler` `terminate_group` /
`reap_group_children` / `process_group_members` / `owned_port_sockets_present`;
`owned-lab-control-plane.sh` `olcp_owned_processes` / `olcp_cleanup_owned` /
`olcp_listener_count`; `probe.rs` `sigterm_after_ready` / `invalid_bind_never_emits_ready` /
`periodic_server_signal_cleanup_is_bounded`; `run-isolated.py` / `run-netns.sh`
temporary-runtime handling.

## Per-invariant coverage

| Invariant | Coverage |
|---|---|
| bounded reap/kill on success, failure, timeout, interruption incl. descendants leaving original group | sampler `terminate_group` escalates TERM→KILL across `process_group_members(pgid)` with bounded polls, then `reap_group_children` WNOHANG-drains; `fixture.timeout-group` proves a grandchild inside the group is killed and `signal=15`/`timed_out=true` is terminal; `fixture.escaped-descendant` (TCP) + `fixture.escaped-udp` (UDP) prove a `setsid()`-escaped holder is not promoted to cleaned |
| readiness/observation handshakes deterministic, cannot succeed because observer raced ahead | `invalid_bind_never_emits_ready` proves a bind failure never emits READY; `authenticated_tcp_and_udp_loopback_probe_starts_after_ready` proves READY precedes traffic; sampler `--net-dir` seam and `fixture.partial-obs` prove an incomplete `/proc/net` observation is `unknown` not `absent` |
| temporary runtime paths removed or truthfully reported unknown/failed; cleanup failure never silently promoted | sampler result write is `tmp.write_text → chmod 0600 → os.replace`, stale tmp `unlink` in `finally`; `owned_sockets_after_exit=None`/`complete=false` on unknown; H-R9-084 makes the wrapper exit nonzero on `!cleanup_complete`; `olcp_cleanup_owned` fails unless `processes_reaped=1 AND listeners=0` — never inferred from exit alone |
| cleanup reporting exposes no secrets/private topology/unnecessary absolute paths | `process-resource.schema.json` fixes the result surface: `identity` is `binary:`/`git:` label, no filesystem path or private topology field exists; validator rejects `../` experiment ids and extra keys |
| no new timeout/capacity/security policy values merely to satisfy tests | no constants added; reap/poll bounds are the pre-existing sampler-owned values |

## Reachable regressions named

- `fixture.timeout-group` — grandchild inside group killed, `timed_out=true`, `signal=15`
- `fixture.escaped-descendant` / `fixture.escaped-udp` — `setsid()` escaped TCP/UDP holder → `complete=false`, `owned_sockets_after_exit=None`, nonzero sampler exit
- `fixture.partial-obs` — missing `/proc/net` table → `owned_sockets_after_exit=None`, `complete=false`, nonzero sampler exit
- `fixture.exit-race` — child gone before first sleep still yields `wait4` CPU/RSS + truthful null sampled-FD
- `sigterm_after_ready_stops_and_releases_tcp_and_udp_bindings` — SIGTERM reaps and releases both bindings
- `periodic_server_signal_cleanup_is_bounded` — periodic server cleanup is bounded
- `invalid_bind_never_emits_ready` — no READY on bind failure

## Result

No concrete defect found across child/process-group/temp-runtime cleanup.
Every terminal path (success, child failure, timeout, TERM/INT interruption)
reaps the sampler-owned process group with a bounded TERM→KILL escalation;
escaped descendants cannot be certified as cleaned; temporary runtime output
is atomically replaced or truthfully reported; cleanup failure is never
promoted to process or result success.

**READY_LIVE: none** — deterministic local evidence only.
