# Independent bounded R9-11B review — Recovery / reliable-UDP terminal ownership

**Source/test anchor:** exact developer tree `730f993c687683f00f82859cbb7587d9fd6f5b84` (current `main` before this reviewer note was docs-only `61ab7eaf736e3e4d1e8f90ffe6904131c4f2ce32`).

## Scope challenged

Inspected exact-current owners in `crates/neko-reliable/src/lib.rs` and `crates/neko-carrier/src/lib.rs`: `Recovery::{on_sent,on_ack,abandon_sent,quiesce}`, `PathRecovery::{on_sent,on_ack,abandon_sent,quiesce,fresh_health_sample}`, and `ReliableUdpRuntime::{on_packet_sent,on_packet_received,poll_outgoing_ack,apply_ack,poll_health,pto_probe,on_retransmit_sent,abandon_sent,abandon_retransmit,teardown,manager_mut}` plus the reachable teardown/quiesce regressions.

The challenge was limited to R9-11B terminal ownership/evidence. Retained Session replay / Carrier-generation terminality is R9-11C; executable process/socket lifecycle is R9-11D; no D019 or retention/capacity policy value is decided here.

## Independent challenge result

No concrete defect found on this bounded surface.

- `ReliableUdpRuntime::teardown()` clears packet-to-frame ownership and retained retransmit plaintext, calls `PathRecovery::quiesce()`, replaces the receiver `PacketAckTracker` with a fresh tracker, then marks the runtime `torn_down`.
- `Recovery::quiesce()` clears the live sent map, outstanding/ACKed frame ownership, packet-number high-water reservation state and `largest_sent`, while retaining documented lifetime RTT/PTO/persistent-congestion diagnostics. `PathRecovery::quiesce()` also zeros Reno bytes-in-flight and charged-packet ownership while preserving lifetime counters.
- Post-teardown send, receive, ACK emission/application, retransmit admission, health polling and mutable Carrier-manager access are fail-closed or inert. `pto_probe()` is observationally inert because teardown leaves zero in-flight ownership. `abandon_sent` / `abandon_retransmit` are not separately `torn_down`-gated, but after teardown their packet maps and Recovery sent ownership are empty, so they return false / mutate no positive evidence.
- Future/never-sent ACK remains rejected inside `Recovery::on_ack` before RTT/loss/PTO mutation; after teardown `largest_sent` is `None`, so a nonempty ACK cannot become valid recovery evidence.
- H-R9-081 is accepted closed at `730f993`: quiesce now aligns `health_epoch` plus resolved interval baselines to the quiescent boundary. The strengthened regression first proves nonzero resolved pre-quiesce loss, then proves a clean post-quiesce resolved interval reports `loss_per_mille == 0`, remains fresh exactly once, and leaves lifetime loss diagnostics intact. The assertion relies only on a positive pre-quiesce loss set, not the comment's exact packet-count wording.

## Evidence boundary

This is source/test review plus reachable developer provenance; **no reviewer-local command execution is claimed**. The developer-local clean exact-tree gate recorded for exact `730f993` remains developer-reported provenance, not reviewer execution. No decoder/parser/crypto framing changed in this slice, so fuzz is not newly required by this review.

**R9-11B: bounded independent no-finding support accepted.** Continue immediately to R9-11C; `READY_LIVE: none` remains unchanged.
