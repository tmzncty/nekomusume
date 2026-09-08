# Exact `f17b648` post-fix repeated warm-failover attempt

## Invocation boundary

Exactly one post-fix self-owned client/VPS outer invocation was made after exact `f17b648` passed exact-head CI and the reviewer-authorized diagnostic-attribution repairs were present. No retry was made.

- commit: `f17b648690fed63cce6f52b6f24c4a79edc51695`
- exact-head CI: GitHub Actions run `34220263467`, success
- binary SHA-256: `6d21d7b230fa14054abd3b33e2a905f1a591feebcb38ad21a9d677c2485dd187`
- binary size: `1262432` bytes
- start: `2026-09-08T11:45:31Z`
- end: `2026-09-08T11:45:37Z`
- bounded plan: at most six sequential cycles; one local client and one self-owned VPS server per cycle; three 16-byte records; controlled application-level UDP reply cessation; ports 40080–40091

## Typed result

The schema-valid outer result exited `1` after 5,752 ms, before retaining cycle 1:

- `status=failed`
- `completed_cycles=0`
- `kind=invalid_cycle_evidence`
- primary `diagnostic_category=startup_setup`
- sanitized diagnostic SHA-256 `5867711ad01f5cc505e982f9b5956b4b05d08758c2071f4fe6d5a6edaec4d101`
- sanitized diagnostic bytes `160`
- `truncated=false`

There is no valid cycle prefix and therefore no retained runtime negotiation, authentication, readiness, failover, application accounting, timing, endpoint-provenance, resource, or per-cycle cleanup result. `startup_setup` is the earliest explicitly owned evidence boundary, not a causal root-cause claim.

The post-fix result repeats the same bounded category/hash/size as the immediately preceding pre-final-repair attempt at exact `9a94922`. With no new safe setup hypothesis, this closes the current repeated-warm-failover line against mechanical retries.

## Cleanup and privacy

A separate serialized post-run observation returned `listeners_remaining=0` and `processes_remaining=0`; the temporary deployment was then removed. Raw stderr, endpoints, identities, credentials, private keys, payloads, and temporary plans are not tracked.

No WAN failover, reliability, reachability, performance, security, release, or production conclusion follows.