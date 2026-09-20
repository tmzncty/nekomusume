# H-I4-086 — Session delivery ACK can release credit before outbound drain

**Severity:** HIGH — release-item-4 correctness / flow-control and delivery-evidence truth

**Reviewed tree:** exact reachable `d7b630f310c504ee439bbcb4bb4c5b9a42a51993`.

**Scope:** current `crates/neko-session/src/lib.rs` `SessionRuntime::{queue_send,pop_send,delivery_ack}` plus the bounded-window and runtime regressions, the provisional Session v0 delivery-evidence boundary, and the carrier/session layering contract. This is a source/spec/evidence challenge only; no reviewer-local test execution is claimed.

## Concrete counterexample

`queue_send()` immediately:

- pushes the record into the still-undrained `send` queue;
- increments per-stream `send_inflight`;
- increments `session_send_inflight`.

`delivery_ack()` then accepts a contiguous positive ACK when `delta <= send_inflight` and `delta <= session_send_inflight`; it does **not** require the acknowledged bytes to have crossed `pop_send()` or otherwise become transport-drained/sent.

Therefore this sequence is currently accepted:

1. `queue_send(stream, "abcd")`;
2. do **not** call `pop_send()`;
3. `delivery_ack(stream, 0, 4)` succeeds;
4. the stream/session window credit is released and the confirmed watermark advances even though the record is still present in the outbound queue;
5. a later `pop_send()` can return bytes already represented as Session-confirmed.

The current regression `send_window_exhaustion_ack_release_and_resume_are_atomic_and_observable` encodes exactly this ordering: it queues `abcd`, observes window exhaustion, calls `delivery_ack()` without first draining the outbound record, and then queues `efgh` after the premature credit release. The test therefore does not merely fail to detect the seam; it positively blesses it.

This conflicts with the current evidence boundary: Session delivery ACK is peer proof that logical bytes were accepted for transport delivery, not local queue admission. A peer-controlled/authenticated ACK must not be able to fabricate Session confirmation for bytes that the local carrier has not even drained from the outbound queue. It also weakens the documented stream/session flow-control window: ACKing queued-but-unsent bytes can free window credit while those bytes remain locally queued.

## Required bounded closure

Do not redesign Session/Carrier/ACK architecture. Use the smallest current-semantics repair that distinguishes queued outbound ownership from ACK-eligible sent/drained ownership. The exact implementation shape is left to the coding agent, but the following regressions are required:

1. **ACK-before-drain negative:** after `queue_send()` and before `pop_send()`, an ACK covering that range fails closed and leaves confirmed watermark, per-stream/session window accounting, queue contents and observable events unchanged.
2. **Drain-then-ACK positive:** after `pop_send()` returns that range, the same ACK succeeds and releases the existing per-stream/session window credit exactly once.
3. **Mixed-prefix negative:** queue two contiguous records, drain only the first, then reject an ACK whose positive advance crosses into the still-queued second record; accepting exactly the drained first range remains valid.
4. **Cross-stream isolation:** draining stream A must not make stream B's queued range ACK-eligible.
5. **Terminal cleanup:** any new sent/drained bookkeeping introduced by the repair must be released on the already-reviewed terminal paths together with the other runtime-owned state.

Queued bytes may continue to count against the existing stream/session send window before drain; `pop_send()` must not itself release delivery credit. Only validated Session `DeliveryAck` may do that. Do not invent a new capacity/TTL/history value.

No decoder/parser/crypto framing change is required, so the pinned fuzz gate is not mechanically required for this repair.

## Evidence boundary

- H-I4-085 dependency/build truth is independently closed by its corrected review text plus developer-reported exact-tree provenance at `38379b1`; this finding does not reopen it.
- I4-FS1 FairScheduler remains independently closed.
- I4-FS2 is **not** closed: developer note `051f588` is useful support but its window-release claim is contradicted by the current owner/test ordering above.
- `READY_LIVE: none`; no WAN question is created by this local semantic defect.
- Release item 4 remains open; release/governance flags remain unchanged.
