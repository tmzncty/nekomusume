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
