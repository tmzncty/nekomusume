# Era-4 E: bounded resilience slice

This slice adds deterministic in-process tests for 64 repeated stream
open/send/receive/graceful-close cycles, peer disappearance after closure, and
32 UDP PTO-to-TCP replay/confirmation cycles. Queue bounds and the existing
nonce/ledger overflow tests remain the fail-closed boundary.

The repository has no durable session store and no process restart/reconnect
implementation. Therefore this slice makes **no claim** that Session state
survives a process restart. Persistence across restart remains explicit E
backlog, requiring a separately specified durable identity, replay/nonce
recovery, and restart integration gate.

Remaining E backlog: real socket-level repeated open/close soak, repeated
cross-process failover/recovery, authenticated key-update cycles beyond the
single supported phase transition, and restart persistence once those
contracts exist.

## Follow-up D repeated/live boundary (2026-09-03)

The accepted exact-`25e0daa` D064 controlled warm fallback remains a single
positive. It does not satisfy the repeated-cross-process backlog by repetition.
Exact `1bf848d` retained zero valid cycles and a typed cycle-1 collector negative
(batch SHA-256
`5ca57b92571690f11157d636d03df554935ced3eda23e312c534020c1ddcf13e`);
exact `07545f0` then exited 127 at a shell argument boundary before the batch
runner, with empty stdout and sanitized stderr SHA-256
`a81c2170e75f57c36490be59a43f0ac5cb342f8b70341d5efb9a6814564bdeaa`.
No exact-`07545f0` batch artifact or cycle row exists. Neither event is runtime
failover evidence. Separate cleanup observations for the latter found zero
experiment processes/listeners, no deployment/identity temp path, and a clean
worktree.

The repeated live row is `BLOCKED_DIAGNOSTICS` until its command array is
corrected and a local dry run proves entry into the Python batch runner.
Migration-back, live key update, and live PMTUD remain
`BLOCKED_IMPLEMENTATION`; NAT/source-endpoint change is also
`BLOCKED_IMPLEMENTATION`; IPv6 is `BLOCKED_ENVIRONMENT`; HY2 is
`BLOCKED_DIAGNOSTICS`. No row is presently `READY_LIVE`.

## Exact `a117086` corrected structured six-cycle outcome (2026-09-04)

The sole authorized live outer invocation at exact `a117086fa69553a36021137900b6052050624a8b` produced a retained schema-valid typed negative: 0/6 cycles, cycle 1 `invalid_cycle_evidence` (`collector returned nonzero without a valid row`), launcher exit 1 after 2,303 ms; no retry. Synthetic preflight is separate and not live evidence. This is an immediate orchestration/evidence-collection boundary only: no full cycle, valid prefix, runtime failover, WAN, or deeper root-cause claim is supported. Per-cycle endpoint provenance, resources (`not_collected_remote`), accounting, timing, and exits are absent because no row exists. Explicit cleanup process/listener/temporary-path postchecks were zero. The prior `c156868` historical negative remains unchanged.

## Final A4 and C1 orchestration outcomes (2026-09-04)

Exact `4a2129e` made the final permitted repeated warm-failover attempt for this instrumentation line: one outer invocation, no retry, 0/6 retained rows, cycle 1 `invalid_cycle_evidence`, and 1,165 ms elapsed. The role-specific bounded diagnostic identifies server exit before the structured start event. This closes only that evidence lane as `BLOCKED_ORCHESTRATION_CURRENT_LINE`; it is not a runtime failover failure. The privacy-safe result SHA-256 is `8af7aab8bb5f8a70c024fd6ef28ac8bc59a53147d86a2bd82a2cf23987fe8d3d`.

Exact `60cd40d` made one no-retry periodic invocation. It stopped pre-application at `start_timeout` with SSH server exit 255, no client dispatch, and no application metrics. Its artifact cleanup remains failed as collected; later cleanup-only checks separately found zero residue. Result SHA-256 is `c5a6576f3131f8e3a6bd120b8192fdaed40e50846ace6e124b2d07741ca5f9b2`. Neither outcome changes the accepted exact-`25e0daa` single positives or any historical negative.

Exact `85346ce` retains the sole changed-hypothesis periodic follow-up from exact `00ac2c1`. It ended pre-application as `ssh_transport_exit` 255: no readiness, client launch, application traffic, or application metrics; both cleanup postchecks verified zero owned residue. The R-009 erratum states that the immutable result's legacy `protocol_entered=true` meant only local capture attachment, not remote executor acceptance. The periodic lane is `BLOCKED_ORCHESTRATION_CURRENT_LINE_PERIODIC`, with no runtime or reliability claim and no automatic retry.
