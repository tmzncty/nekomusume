# R9-2H exact applied-identity recheck — exact `442f60c`

**Reviewed developer source/test SHA:** `442f60c8d1a3719b3b5b8c86e1f6e60017bb6ac0` (`fix(cli): offset-bearing ACK evidence + reversed-order process test (H-R9-013, M-R9-008 P1)`).

**Review scope:** current `crates/neko-cli/src/main.rs`, `crates/neko-cli/tests/probe.rs`, the current reviewer handoff contract, Session-above-Carrier / ACK-domain architecture, and hosted check state. This is a bounded reviewer recheck, not a release/security approval and not a WAN result.

## Current truth

- GitHub-hosted `stable checks` on exact `442f60c` succeeded.
- GitHub-hosted `nightly decode fuzz smoke` on exact `442f60c` succeeded.
- These are supplementary hosted cross-evidence only. No new developer-local exact-tree provenance file for `442f60c` is present in this developer commit.
- No open PR or WAN/VPS experiment is introduced by this change.

## Accepted progress

### Direct/current logical ACK attribution — ACCEPTED

The immediate Session-ACK application path now emits `r9_udp_delivery_ack_validated` after the successful `SessionRuntime::delivery_ack` mutation with explicit `stream` and `offset`. Keeping the legacy first-confirmation event in parallel does not change Session semantics.

The new built-binary `--reverse-ack-order` process test is also a useful regression seam: it exercises real failover server/client processes, observes the later logical ACK being buffered, and checks that the process reaches zero Carrier in-flight in the successful fixture.

## OPEN HIGH — H-R9-014 buffered applied ACK still lacks exact stream identity

The handoff requires exact `stream + offset` attribution after **every** successful Session DeliveryAck mutation. The direct/current path now satisfies that, but the pending-buffer drain still emits:

```text
r9_udp_delivery_ack_validated
  ciphertext_bytes=0
  offset=<...>
  buffered=true
```

without `stream`.

That second event corresponds to a real Session mutation (`SessionRuntime::delivery_ack` on the buffered record), so the exact applied identity remains incomplete. This is an evidence-integrity HIGH for the R9-2H closure contract; it does not imply a new Session semantic defect.

### Smallest repair

Preserve the current pending-ACK state machine and emit the buffered applied event with the same exact identity fields as the direct applied event: at minimum `stream`, `offset`, and `buffered=true`. Do not change wire, crypto, Session semantics, ACK semantics, or capacity policy.

## P1 process regression is useful but not yet closure-grade

The new process test currently proves only broad string ordering. It does **not** yet prove the full P1 contract:

1. it does not assert client exit success;
2. it does not parse/count exactly two `r9_udp_delivery_ack_validated` events;
3. it does not require `stream=1` on both applied events;
4. generic `contains("\"offset\":...")` checks are not scoped to the applied-event class;
5. `rfind("\"buffered\":true")` is not tied to exact offset 16 / stream 1 / applied event type;
6. it does not assert each exact applied confirmation occurs once;
7. it does not explicitly prove `outstanding` and `pending_acks` are empty before transition into Carrier settlement.

### Required P1 strengthening

Use the real built binary and the existing reverse-order seam, but validate the structured events themselves. At minimum prove:

- the client exits successfully;
- exactly one buffered-observation event identifies stream 1 / offset 16 while watermark is 0;
- exactly two applied `r9_udp_delivery_ack_validated` events exist;
- applied event 1 is stream 1 / offset 0 / non-buffered;
- applied event 2 is stream 1 / offset 16 / `buffered=true`;
- neither exact applied identity appears twice;
- no `r9_udp_delivery_ack_covered` shortcut is used for these two records;
- logical ownership is empty before Carrier-only settlement begins;
- Carrier settlement reaches `remaining_in_flight=0`.

A small explicit pre-settlement invariant check (`outstanding.is_empty() && pending_acks.is_empty()`) is sufficient; no new numeric policy is required.

## Remaining front queue remains valid

R9-2H-P2 (exact migration-back reserved-record ownership), P3 (operation-wide malformed budget across valid Carrier feedback), P4 (incomplete-settlement terminality), and final developer-local exact pushed-tree provenance are still required. None is superseded by `442f60c`.

R9-3 remains blocked until the R9-2H closure tree is reachable and green. After closure, continue directly through R9-3..R9-12 and Q10/Q11/Q12 without waiting for reviewer cadence.

## Governance / live boundary

No release flag changes. Item 3 and item 4 remain incomplete. `READY_LIVE: none` remains authoritative; current blocker is local implementation/evidence ownership, not WAN permission.
