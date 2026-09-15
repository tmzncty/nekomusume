# Reviewer bounded recheck — R9 P2 causal order at exact `1aa95ab`

## Scope

Reviewed exact developer-owned source/test commit `1aa95abddfc23d64d377fa65172ef8cf740ff987` against the current R9 P2 handoff contract, the Session/Carrier evidence separation in `README.md`, `AGENTS.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, and the provisional Session v0 boundary.

This is a bounded reviewer recheck of the existing built-binary migration-back fixture. It is not a release/security approval, protocol freeze, WAN result, capacity judgment, or architecture change.

## Exact-tree facts

- Commit classification: **test-only** (`crates/neko-cli/tests/probe.rs`). No runtime source changed.
- The migration-back fixture still uses `count=4`, `bytes=16`, `--reliable-udp`, `--automatic-health-failover`, and `--migration-back`.
- Hosted GitHub `stable checks` succeeded for exact `1aa95ab`; hosted nightly decode fuzz smoke also succeeded. These are cross-evidence only; final R9-2 developer-local clean exact-tree provenance is still absent.
- Open PRs: none at review time.
- No new VPS/WAN execution occurred.

## Verdict

**ACCEPT_PARTIAL.** The commit usefully changes four presence-only assertions into strict observed-order assertions for the client recovery challenge/validation/migration sequence and the server recovery-owner/validation/Session-ACK/Carrier-ACK sequence. No concrete runtime correctness defect is introduced by this test-only change.

However, the P2 evidence contract is not yet closed because exact identity, cardinality, and terminal ordering remain under-specified.

## Remaining P2 evidence gaps

### P2-C1 — TCP replay identity/count remains unasserted

The fixture still does not require exactly one `tcp_delivery_ack_validated` on the client and exactly one `tcp_delivery_ack_sent` on the server for `seq=2` / logical offset 32, nor reject replay evidence for `seq=0`, `seq=1`, or reserved `seq=3`.

The runtime already emits replay sequence identity, so this should be test-only unless a missing field is found.

### P2-C2 — client post-return order/cardinality is still partial

Exact `1aa95ab` proves:

`udp_recovery_challenge_sent < udp_recovery_validated < udp_migrated_back`.

It still does not mechanically prove:

`udp_migrated_back < r9_udp_post_return_sent(seq=0, offset=48)`

or that the post-return send event occurs **exactly once** for offset 48. Tighten the existing fixture with line/cardinality checks; do not create a second scenario.

### P2-C3 — server post-return order lacks exact identity/cardinality

The fixture now proves the event-name order:

`udp_recovery_owner_started < udp_recovery_validated < udp_return_delivery_ack_sent < udp_return_packet_ack_sent`.

It still does not require exactly one occurrence of each event, does not require the Session ACK event to be `seq=3` (offset 48 in this fixed 16-byte fixture), and does not mechanically bind the Carrier ACK line to the same bounded post-return owner beyond event-name order. The current server diagnostic already derives the Session-ACK `seq` from `post_offset / bytes`; assert it. For the packet-ACK line, use uniqueness + strict same-branch order first; add only the smallest contextual diagnostic field if that cannot make the association unambiguous.

### P2-C4 — exact logical + Carrier settlement is still not mechanically bound

The client test still checks only broad presence of `udp_return_delivery_ack_validated`. Current runtime diagnostics for that event carry `seq=count` plus ciphertext bytes, but no explicit stream/offset, so the fixture cannot directly prove that the validated logical ACK is the reserved stream 1 / offset 48 record.

The same fixture should require:

- exactly one logical post-return confirmation for stream 1 / offset 48; add the smallest `stream`/`offset` diagnostic fields to `udp_return_delivery_ack_validated` if needed;
- exactly one `r9_udp_return_packet_ack` with `applied=true`;
- exactly one `r9_udp_post_return_settled` with stream 1 / offset 48 / `remaining_in_flight=0`;
- the settled event strictly after **both** acknowledgement-domain events, regardless of their arrival order.

Do not change Session or Carrier ACK semantics to improve diagnostics.

## Queue consequence

R9-3 remains dependency-blocked only on unfinished R9-2 evidence/closure, not on policy or WAN authorization. Finish P2-C1..C4 in the existing fixture, then continue the already-authorized R9-2 acknowledgement-order seam, automatic-health replay exact-identity regression, persistent malformed-budget P3, incomplete-settlement P4, final developer-local clean exact-tree gate/provenance, and then R9-3..R9-12 plus Q10/Q11/Q12 without waiting for reviewer cadence.

`READY_LIVE: none` remains correct until the cross-process reliable-UDP path and real TCP promotion receive their dedicated bounded review and Q11 creates a specific changed-hypothesis WAN question.
