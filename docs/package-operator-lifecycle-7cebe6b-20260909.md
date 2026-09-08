# Installed-package lifecycle attempt — exact `7cebe6b`

## Scope

One materially changed, bounded self-owned installed-package lifecycle attempt for exact `7cebe6b34008fcd6aae096ca7cd3a579f9aa6f3b`, after the executable was changed to externally emit ordered `DRAINING` then `STOPPED` on signal. Endpoint identity is intentionally opaque (`self-owned-vps-A` / `private-bind-A`). This is not release, production, public-reachability, daemon/service-manager, or security approval. HY2 and repeated-failover were not run.

- Experiment ID: `package-lifecycle-7ce-20260909`
- Experimental ports: TCP `40080`, UDP `40081`
- Intended profile: package build/smoke and installed hash match; for each transport, READY, SIGTERM, DRAINING/STOPPED, listener release, same-port restart, one authenticated 32-byte exchange, and cleanup

## Incremental evidence and result

The orchestration wrote phase markers incrementally. The retained phase prefix is:

```json
{"phase":"start","status":"ok"}
{"phase":"package_smoke","status":"ok"}
{"phase":"installed_hash","status":"ok"}
{"phase":"tcp-preclean","status":"ok"}
```

The command exited nonzero before recording `tcp-ready`. Therefore the exact boundary is `BLOCKED_ORCHESTRATION_CURRENT_LINE_PACKAGE_LIFECYCLE` at TCP startup/readiness evidence collection. Package smoke and installed-binary hash equality completed; no truthful claim is made that remote TCP readiness, SIGTERM handling, drain/stop, restart/rebind, authenticated TCP exchange, or any UDP phase passed or failed at runtime. In particular, this result does not disprove the green local process tests and does not identify a runtime root cause.

## Cleanup

Independent post-attempt checks found no experimental TCP/UDP listener, no experiment process, and no retained remote experiment directory. Local temporary package, identities and logs were removed by cleanup. No production service, route, firewall, DNS, proxy, tunnel, qdisc, or repository identity was modified.

## Boundary

This package-lifecycle line is now frozen against same-class retries until a concrete new code/configuration/instrumentation hypothesis changes materially. The earlier exact-`14be1c8` incomplete negative remains immutable and separate. Release flags remain false and D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`.
