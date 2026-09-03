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
