# Developer bounded R9-11D review — executable process/socket lifecycle & result truth

**Anchor:** exact source/test tree `730f993` + reviewer commits through `0c5ce73`.
**Owners inspected:** `neko-cli` failover-client/failover-server/failover-adapter
process owners (`fail()` JSON writer, `print_json` result, exit status),
readiness tuple auth/admission sequence, post-return terminal classification,
`sigterm_after_ready` fixture, TCP/UDP listener close and owned-port release,
`process-resource-sampler` child/group reaping.

## R9-11D1 — process result-truth / terminal classification

| # | Invariant | Coverage |
|---|---|---|
| terminal failure/timeout/stop cannot be rewritten as READY/success | `fail()` writes `{ok:false,last_stage,error}` and exits non-zero; `failover_client_ok`/`failover_server_ok`/`failover_adapter_ok` are emitted only on the success path — a later cleanup/read step returning 0 cannot rewrite the terminal result |
| structured JSON, human output, exit status agree | `fail()` and the success printer emit the same terminal outcome; probe tests assert the process exit code and `ok`/`settled`/`last_stage` together |
| missing/unknown cleanup observation stays failure/unknown | `cleanup`/`group_empty`/`owned_sockets_after_exit` are observed, never inferred — unknown stays unknown |
| post-return work cannot manufacture new success after terminal result | post-return settlement is the terminal classification; `reliable_udp_post_return_incomplete_is_terminal` fails rather than reporting settled |
| negative fixtures prove the failure phase was entered | every negative probe seeds the harness's matching kill/detach/malformed flag and asserts `last_stage`/`reason` reached the intended phase — no vacuous pass |

## R9-11D2 — TCP/UDP listener & socket ownership

`sigterm_after_ready_stops_and_releases_tcp_and_udp_bindings` asserts SIGTERM
releases both bindings (port rebinding fails while process lives, succeeds
after teardown). `udp_listener_rejects_bounded_malformed_churn_then_authenticates`
proves listener ownership survives malformed churn and cleans up on close.
TCP/UDP sockets are owned by the runner process and released on stop — no
orphaned listener.

## R9-11D3 — child/process-group/temp-runtime cleanup

`process-resource-sampler` creates a process group (`os.setpgid`), samples all
members, and asserts `cleanup.process_group_empty`/`owned_sockets_after_exit==0`
on exit; the `timeout-group`/`normal-group` fixtures prove grandchild and
listener descendants are reaped and bindings released. Temp/runtime ownership
follows the process group — nothing escapes reaping.

## Result

No concrete defect found across process result-truth/terminal classification,
socket ownership, or child/process-group/temp-runtime cleanup. Terminal
results are consistent across JSON/output/exit status; sockets and process
groups are released on teardown.

**READY_LIVE: none** — deterministic local evidence only.
