# Installed-package lifecycle attempt — exact `7cebe6b`

## Scope

One materially changed, bounded self-owned installed-package lifecycle attempt for exact `7cebe6b34008fcd6aae096ca7cd3a579f9aa6f3b`, after the executable was changed to externally emit ordered `DRAINING` then `STOPPED` on signal. Endpoint identity is intentionally opaque (`self-owned-vps-A` / `private-bind-A`). This is not release, production, public-reachability, daemon/service-manager, or security approval. HY2 and repeated-failover were not run.

- Experiment ID: `package-lifecycle-7ce-20260909`
- Experimental ports: TCP `40080`, UDP `40081`
- Intended profile: package build/smoke and installed hash match; for each transport, READY, SIGTERM, DRAINING/STOPPED, listener release, same-port restart, one authenticated 32-byte exchange, and cleanup

## Incremental evidence and result

The orchestration wrote phase markers incrementally. The complete retained phase record is:

```json
{"phase":"start","status":"ok"}
{"phase":"package_smoke","status":"ok"}
{"phase":"installed_hash","status":"ok"}
{"phase":"tcp-preclean","status":"ok"}
{"phase":"tcp-ready","status":"ok"}
{"phase":"tcp-signal-stop","status":"ok"}
{"phase":"tcp-rebind-ready","status":"ok"}
{"phase":"tcp-authenticated-exchange","status":"ok"}
{"phase":"tcp-clean","status":"ok"}
{"phase":"udp-preclean","status":"ok"}
{"phase":"udp-ready","status":"ok"}
{"phase":"udp-signal-stop","status":"ok"}
{"phase":"udp-rebind-ready","status":"ok"}
{"phase":"udp-authenticated-exchange","status":"ok"}
{"phase":"udp-clean","status":"ok"}
{"phase":"cleanup_pre_delete","status":"ok"}
{"phase":"cleanup_post_delete","status":"ok"}
```

The installed package completed the intended bounded lifecycle for both transports: readiness, SIGTERM shutdown with ordered `DRAINING`/`STOPPED`, listener release, same-port restart, and one authenticated 32-byte exchange after restart. This is an exact-tree, self-owned VPS operator observation only; it does not establish daemon/service-manager hardening, sustained WAN reliability, public reachability, performance superiority, release, production readiness, or security approval.

## Cleanup

Independent post-attempt checks found no experimental TCP/UDP listener, no experiment process, and no retained remote experiment directory. Local temporary package, identities and logs were removed by cleanup. No production service, route, firewall, DNS, proxy, tunnel, qdisc, or repository identity was modified.

## Boundary

This package-lifecycle line is now frozen against same-class retries until a concrete new code/configuration/instrumentation hypothesis changes materially. The earlier exact-`14be1c8` incomplete negative remains immutable and separate. Release flags remain false and D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`.
