# N4 lifecycle readiness repair evidence — 2026-08-31

This bounded loopback evidence supersedes only the lifecycle-readiness claim in the historical N8 bundle. The original `artifacts/n8-20260831/` files remain byte-for-byte unchanged.

- TCP and UDP servers emitted `lifecycle_state=READY readiness=true` before SIGTERM.
- Each SIGTERM run then emitted `lifecycle_state=STOPPED readiness=false` and exited successfully.
- Each released address was immediately rebound by a fresh server, which reached READY and was stopped.
- The invalid-bind run exited unsuccessfully and emitted no READY line.

The old N8 authenticated TCP/UDP exchange matrix remains valid exchange evidence. Its readiness subclaim is superseded because those historical server logs recorded `lifecycle_state=FAILED readiness=false` before serving.

Scope: same-host loopback, temporary identities, bounded duration, no public-WAN or production-readiness claim.
