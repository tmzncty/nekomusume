# Exact `2b784a3` changed-hypothesis repeated warm-failover attempt

## Invocation boundary

Exactly one bounded self-owned client/VPS outer invocation was made after RWFDIAG-001 exact-head CI passed. No retry was made. Historical `9fd2411`, `a117086`, `c6ab8fd` artifacts remain unchanged.

- commit: `2b784a345fe87e8ef22152a1570bf3d125cdcdf5`
- binary SHA-256: `3fef6064a47726a509dce6c3760820df45cfb9d50042be6c310beca5fb2b848b`
- binary size: `1260192` bytes
- start: `2026-09-08T08:49:04Z`
- end: `2026-09-08T08:49:05Z`
- bounded plan: six sequential cycles, one client/VPS server pair per cycle, controlled application-level UDP reply cessation, ports in the repository's bounded 40080–40100 range

## Result

The outer runner exited `1` before retaining a cycle row. `completed_cycles=0`; there is no valid prefix, runtime failover row, accounting, timing, endpoint, or application result to claim.

The newly instrumented inner failure marker propagated the earliest known category as:

- `diagnostic_category: cleanup`
- `kind: invalid_cycle_evidence`
- sanitized diagnostic SHA-256: `f061a26f0b93f752906fbf96f514bc4c3e43a3841feef6a53157ab37f3028faf`
- sanitized diagnostic bytes: `202`
- `truncated: false`

This is a cleanup/evidence-collection boundary only. It does not identify a remote runtime, negotiation, authentication, readiness, or failover cause. Because no cycle row was emitted, no WAN failover or reliability conclusion follows.

## Cleanup

A serialized post-run cleanup check verified `cleanup_verified=true`: no experiment directory, listener, or process residue remained on the VPS. The private diagnostic is not tracked; only its sanitized hash metadata is retained.

## Next action boundary

This is the one authorized materially changed attempt for RWFDIAG-001. The same classified cleanup failure must not be mechanically retried. Further progress requires a new material cleanup hypothesis/fix and fresh exact-head review; release, production, freeze, and released states remain unchanged.