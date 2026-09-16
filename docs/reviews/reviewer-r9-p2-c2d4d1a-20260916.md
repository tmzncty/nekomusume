# Reviewer checkpoint — R9-2 P2 C3/C4 evidence at `c2d4d1a`

Exact reviewed developer-owned source/test anchor: `c2d4d1a507f69bbc768abe38d179518099131508` (`test(cli): P2 C3/C4 exact post-return dual-ACK identity + one-only cardinality (M-R9-008)`). Parent reviewer handoff: `81a6a01b1b6be0b95af22ac7fcc566826bbed5fc`.

## Classification

This commit is evidence-oriented CLI diagnostic/process-test work. It does not change the Session, Carrier, ACK, crypto, or wire architecture.

Changed owners:

- `crates/neko-cli/src/main.rs`: adds post-return diagnostic fields for the server Session DeliveryAck / Carrier packet ACK and a client Session-ACK application event.
- `crates/neko-cli/tests/probe.rs`: tightens the existing count=4 migration-back fixture for one-only post-return ACK events.

No decoder/parser/crypto framing source changed.

## Exact-tree hosted cross-evidence

GitHub Actions run `35056977108` on exact `c2d4d1a507f69bbc768abe38d179518099131508` completed successfully:

- `stable checks`: SUCCESS; hosted job ran `bash scripts/check.sh`.
- `nightly decode fuzz smoke`: SUCCESS; hosted job ran the pinned fuzz build and 30-second decode smoke.

This is hosted cross-evidence only, not the required final developer-local clean exact-tree R9-2 provenance. Open PRs at review time: none.

## Accepted narrow progress

The existing positive count=4 process fixture now requires both processes to succeed and has materially stronger post-return observations:

- server `udp_return_delivery_ack_sent` and `udp_return_packet_ack_sent` each occur exactly once;
- client `r9_udp_return_delivery_ack` occurs exactly once and is bound to stream 1 / offset 48;
- client `r9_udp_return_packet_ack` occurs exactly once and must carry `applied=true`;
- the existing terminal `r9_udp_post_return_settled` still requires stream 1 / offset 48 / `remaining_in_flight=0`;
- server causal ordering remains recovery owner -> validated -> Session DeliveryAck -> Carrier packet ACK.

This is useful P2 progress, but it does **not** yet close C3/C4 exactly.

## H-R9-024 — HIGH evidence-domain misbinding in C3

### Invariant challenged

Carrier packet ACK evidence must identify Carrier packet ownership, not borrow logical Session identity. Session DeliveryAck and Carrier packet ACK are separate evidence domains throughout the architecture and provisional Session contract.

### Current source truth

The server already obtains the actual post-return packet number as `pn` from the received authenticated post-return datagram and passes that exact value to `server_rt.on_packet_received(pn, true)` before polling outgoing ACK ranges.

However, `udp_return_packet_ack_sent` does not emit that packet number (or an equivalent exact ACK-range identity). Instead `c2d4d1a` annotates the packet-ACK event with the logical Session `stream` / `offset` from the Data record.

The current process test then treats `"offset":48` on `udp_return_packet_ack_sent` as the C3 identity proof.

### Why this is not sufficient

That assertion proves only that the packet-ACK diagnostic was emitted while handling logical offset 48. It does not mechanically prove that the ACK ranges being sent actually acknowledge the post-return Recovery packet registered by the client. A future wrong-range / wrong-packet bug could still satisfy the current `offset=48` oracle because the logical record and Carrier packet sequence spaces are distinct.

This is an evidence-integrity HIGH for the release/item-4 support lane, not a newly demonstrated runtime transport defect.

### Smallest repair contract

Do not redesign ACK framing or Session semantics.

1. Preserve the real post-return `pn` already available at `server_rt.on_packet_received` and include a secret-free `packet_number` field on `udp_return_packet_ack_sent`.
2. Add the same client-side `packet_number` to `r9_udp_post_return_sent` at the existing `on_packet_sent` point so the process fixture can bind sender Recovery ownership to server ACK emission.
3. The packet-ACK event may retain logical stream/offset only as correlation metadata, but the test must use packet number / actual Carrier owner as the Carrier-domain identity. Do not treat logical offset as the packet-ACK identity.
4. Keep the Session DeliveryAck proof separate.

No policy value, new receive owner, or wire change is required.

## Remaining exact P2 obligations

### C1 TCP replay identity/cardinality

Still not mechanically complete from the previous review. Require exactly one client and server TCP DeliveryAck for `seq=2`, stream 1, offset 32, and no TCP replay/ACK evidence for offsets 0/16/48.

### C2 migration -> post-return send

Still require selected recovery challenge / validated / migrated milestones to be one-only, then prove the full strict chain ending in exactly one `r9_udp_post_return_sent` for stream 1 / offset 48. Keep offset 48 absent from uncertain/TCP/pre-promotion reliable ownership evidence.

### C3 server dual-domain evidence

After H-R9-024, require exactly one Session DeliveryAck event with `seq=3`, stream 1, offset 48, len 16, and exactly one Carrier packet ACK bound to the actual post-return packet number. Current Session-ACK event/test still omits `len`, and the test does not yet assert `seq=3` + stream 1 mechanically.

### C4 client dual-domain settlement

The new `r9_udp_return_delivery_ack` is the useful actual Session-confirmation event, but it still lacks `len`. Add/retain enough fields to prove stream 1 / offset 48 / len 16 exactly once.

For ordering, use the actual `r9_udp_return_delivery_ack` mutation event and the actual `r9_udp_return_packet_ack` event. The current final order check still uses `udp_return_delivery_ack_validated`, which is emitted only after the dual-settlement loop has already completed and is therefore a post-loop summary, not the actual ACK-domain transition.

Require exactly one `r9_udp_post_return_settled` and prove it occurs strictly after both actual ACK-domain events. Keep one-only `r9_udp_return_packet_ack` with `applied=true` and no rejected/false shortcut.

## Continuous next queue

1. Repair H-R9-024 with exact Carrier packet-number correlation.
2. Finish C1/C2 exact identity/cardinality/order in the existing count=4 fixture.
3. Finish C3/C4 exact Session len/seq/stream fields, Carrier packet identity, actual-event ordering, and one-only settlement.
4. Exercise both post-return ACK arrival orders through the same bounded authenticated receive owner / absolute deadline / malformed budget.
5. Tighten both P4 ACK-domain suppression negatives so the non-suppressed domain is positively observed and terminal failure cannot continue into downstream success/health/failover/migration.
6. Record final developer-local clean exact-tree R9-2 provenance (`PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean initial/final tree, exact pushed SHA, UTC/OS/arch/Rust/exit codes).
7. Continue immediately with R9-3 data-loss recovery.
8. R9-4 ACK-loss + delayed-original/reorder challenge.
9. R9-5 adversarial authenticated feedback atomicity / future-ACK / stale-feedback challenge.
10. R9-6 send-admission / retransmit ownership / bounded retained-state challenge.
11. R9-7 process/result truth separation.
12. R9-8/R9-9 warm-TCP health promotion challenge.
13. R9-10 uncertain Session replay + cleanup negatives.
14. R9-11/R9-12 exact-tree coherent gate and factual reconciliation.
15. Q10 dedicated independent cross-process R9 integration review, then Q11/Q12 status/release-packet reconciliation only after a reachable independent anchor.

## Release/live boundary

`READY_LIVE: none` remains authoritative. Standing VPS authorization remains valid, but this review creates no new dependency-satisfied real-network question. Release item 3 and item 4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.
