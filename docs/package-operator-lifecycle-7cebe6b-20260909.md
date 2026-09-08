# Installed-package lifecycle attempt — exact `7cebe6b`

## Scope

One materially changed, bounded self-owned installed-package lifecycle invocation for exact `7cebe6b34008fcd6aae096ca7cd3a579f9aa6f3b`, after the executable was changed to externally emit ordered `DRAINING` then `STOPPED` on signal. Endpoint identity is intentionally opaque (`self-owned-vps-A` / `private-bind-A`). This is not release, production, public-reachability, daemon/service-manager, or security approval. HY2 and repeated-failover were not run.

- Experiment ID: `package-lifecycle-7ce-20260909`
- Experimental ports: TCP `40080`, UDP `40081`
- Intended profile: package build/smoke and installed hash match; for each transport, READY, SIGTERM, DRAINING/STOPPED, listener release, same-port restart, one authenticated 32-byte exchange, and cleanup
- Invocation count: exactly one changed-hypothesis VPS invocation
- Command exit status: `0`

## Provenance chronology and erratum

The invocation's captured command result returned exit status `0` and contained the complete sanitized summary and phase stream below. A separate immediate post-check found no experimental listener, process, or remote temporary directory.

Exact `e84770e` incorrectly described this same invocation as a nonzero, TCP-pre-readiness prefix. That was a reporting/interpretation error: it did not reflect the captured command exit status or complete stdout, and it did not represent a separate VPS invocation. Exact `c4ee406` corrected the outcome but did not explain this chronology or provide an inspectable sanitized anchor. This document is the explicit erratum. No second lifecycle invocation occurred and no rerun was used to chase a pass.

The canonical sanitized result below has 26 newline-terminated records and SHA-256 `4b293249846c7c7db3322ad864fb9797e5e36387f6bebaa0b7cdb2e518834bd9`; it is also stored verbatim as [`package-operator-lifecycle-7cebe6b-20260909.result.txt`](package-operator-lifecycle-7cebe6b-20260909.result.txt).

## Sanitized result

```text
experiment_id=package-lifecycle-7ce-20260909
commit=7cebe6b34008fcd6aae096ca7cd3a579f9aa6f3b
endpoint=self-owned-vps-A
private_bind=private-bind-A
archive_sha256=c30497b4a31ec36f8a1f39d7d39baa80a2fc1a8c78e792253b5be740f7a14404
binary_sha256=6283a25a93b2fabd0f8cb06008ec72225cf52e29950811aec2b6a8dbf5da80de
installed_binary_sha256=6283a25a93b2fabd0f8cb06008ec72225cf52e29950811aec2b6a8dbf5da80de
tcp_result=probe_ok transport=tcp bytes=32 elapsed_ms=184
udp_result=probe_ok transport=udp bytes=32 elapsed_ms=62
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
cleanup_post_delete=verified
```

The installed package completed the intended bounded lifecycle for both transports: readiness, SIGTERM shutdown with ordered `DRAINING`/`STOPPED`, listener release, same-port restart, and one authenticated 32-byte exchange after restart.

## Cleanup and boundary

The invocation's cleanup removed its local temporary package, identities, logs, and dedicated remote path. The immediate independent post-check observed no experimental listener, no experiment process, and no retained remote experiment directory. No production service, route, firewall, DNS, proxy, tunnel, qdisc, repository identity, or production data was modified.

The earlier exact-`14be1c8` incomplete orchestration negative remains immutable and separate. This exact-`7cebe6b` result is bounded operator evidence only; it does not establish daemon/service-manager hardening, sustained WAN reliability, public reachability, performance superiority, release, production readiness, or security approval. Release flags remain false and D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`.
