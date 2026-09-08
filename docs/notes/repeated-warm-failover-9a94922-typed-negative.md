# Exact `9a94922` fresh repeated warm-failover attempt

## Invocation boundary

Exactly one fresh materially changed self-owned client/VPS outer invocation was made after the corrected RWFDIAG contract at exact `9a94922` passed exact-head CI. No retry was made. Historical attempts and artifacts remain unchanged.

- commit: `9a94922794110ca0d9c0d8878f9903173d2de495`
- exact-head CI: GitHub Actions run `34216283788`, success
- binary SHA-256: `6d21d7b230fa14054abd3b33e2a905f1a591feebcb38ad21a9d677c2485dd187`
- binary size: `1262432` bytes
- start: `2026-09-08T11:09:07Z`
- end: `2026-09-08T11:09:13Z`
- bounded plan: at most six sequential cycles, one local client and one self-owned VPS server per cycle, three 16-byte application records, controlled application-level UDP reply cessation, unprivileged ports in 40080–40091

## Typed result

The outer runner exited `1` after 5,768 ms, before retaining cycle 1:

- `status=failed`
- `completed_cycles=0`
- `kind=invalid_cycle_evidence`
- primary `diagnostic_category=startup_setup`
- sanitized diagnostic SHA-256: `5867711ad01f5cc505e982f9b5956b4b05d08758c2071f4fe6d5a6edaec4d101`
- sanitized diagnostic bytes: `160`
- `truncated=false`

There is no valid cycle prefix and therefore no retained per-cycle endpoint provenance, runtime negotiation/authentication, readiness, failover, application accounting, timing, resource, or cleanup row. `startup_setup` is only the earliest evidence boundary established by the corrected explicit stage ownership; it is not a root-cause statement.

## Cleanup and privacy

A separate serialized post-run check reported zero owned listeners and zero owned processes. The persistent per-run deployment was then removed. Raw stderr, endpoint values, identities, credentials, private keys, payloads, and temporary plans are not tracked; only the bounded typed result and sanitized diagnostic hash metadata are retained here.

## Next-action boundary

This consumes the single fresh changed-hypothesis attempt authorized after RWFDIAG-002/003/004. The same startup/setup-class attempt must not be mechanically retried. A further attempt requires a concrete new setup hypothesis and a material code/configuration/path change followed by green exact-head gates. No WAN failover, reliability, reachability, performance, security, release, or production conclusion follows.