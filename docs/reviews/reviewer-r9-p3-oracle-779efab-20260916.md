# Reviewer R9 P3 oracle checkpoint — exact `779efab`

**Reviewed exact tree:** `779efab949f2b383599a0c6b33781ab5615e830a`

**Scope:** the developer-owned H-R9-022 repair, current reliable receive/demux ownership, the built-binary malformed-budget P3 regression, and the still-open R9-2 evidence queue. This is a bounded reviewer checkpoint, not a security approval, release decision, protocol freeze, WAN result, or production claim.

## Current repository / CI facts

- `main` was exact `779efab949f2b383599a0c6b33781ab5615e830a` at review start.
- Exact `779efab` changes only `crates/neko-cli/src/main.rs` and `crates/neko-cli/tests/probe.rs`.
- Hosted Rust CI on exact `779efab` is green: `stable checks` ran `bash scripts/check.sh` successfully and `nightly decode fuzz smoke` also succeeded. Hosted CI remains cross-evidence only.
- Open PRs: none.
- No new WAN/VPS run is present in this sequence.
- Final developer-local clean exact-tree provenance for R9-2 is still absent.
- Governance remains unchanged: `READY_LIVE: none`; item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

## H-R9-022 applied-evidence seam — CLOSED narrowly

Exact `779efab` adds a diagnostic at the existing outer reliable receive owner when `UdpAcknowledgement::Carrier { applied: true }` is handled. The event carries the live operation-wide malformed counter. The P3 regression now requires an `r9_udp_packet_ack_applied` event with `malformed=2`.

This is the missing discriminator from the prior checkpoint: the interleaved Carrier ACK is not merely present on the wire; the client proves that the current Recovery owner accepted it while the same malformed counter had already observed malformed #1 and #2.

Preserve this event and the source ordering established at `966eb49`:

```text
malformed #1 -> malformed #2 -> Carrier packet ACK -> Session DeliveryAck -> malformed #3
```

## H-R9-023 — P3 acceptance oracle is still under-specified

**Severity: HIGH for R9-2 evidence correctness. Mechanically repairable under current semantics; no maintainer policy or architecture decision required.**

The new applied event is necessary but the current built-binary regression still does not enforce the complete P3 contract already stated by the handoff:

1. It uses `contains(...)` rather than requiring exactly one selected applied event with `malformed=2`.
2. It does not forbid `r9_udp_packet_ack_rejected` for the selected interleaved Carrier feedback.
3. It still accepts the broad terminal condition `client_err.contains("malformed") || client_err.contains("bound")` instead of pinning the existing terminal error `UDP delivery acknowledgement malformed bound exceeded`.
4. It forbids only `r9_udp_in_flight_settled` and `r9_udp_post_return_settled`; it does not forbid downstream health/failover/migration continuation after the malformed-bound terminal path.

A future unrelated malformed failure, duplicate/rejected packet-ACK diagnostic, or accidental continuation could therefore still satisfy the current test while violating the intended operation-wide malformed-budget invariant.

### Smallest closure

Do not change `MAX_POST_HANDSHAKE_MALFORMED`; do not change ACK, Session, Carrier, crypto or wire semantics; do not add a second receive owner.

Tighten only the existing built-binary P3 regression so that it:

- collects `r9_udp_packet_ack_applied` lines and requires exactly one selected event with `malformed=2`;
- requires no `r9_udp_packet_ack_rejected` event for this fixture;
- requires unconditional client nonzero exit;
- requires the exact existing terminal error text `UDP delivery acknowledgement malformed bound exceeded`;
- requires no `r9_udp_in_flight_settled` and no `r9_udp_post_return_settled`;
- requires no downstream health/failover/migration success markers after terminalization (at minimum no health sample/degraded/failed, TCP promotion, migration-back, or final failover-success result markers used by this CLI path).

Focused built-binary test, commit, push, then continue immediately. No decoder/framing fuzz is required solely for this assertion repair.

## R9-2 exact P2 closure is still real work

The current count=4 positive migration-back test is useful but is not yet the exact C1-C4 oracle promised by the handoff:

- **C1:** the test counts exactly one `tcp_delivery_ack_validated` line but does not mechanically pin its exact `seq=2`, stream/offset 32 identity, the matching server ACK identity/cardinality, or exclude TCP replay/ACK of offsets 0/16/48 by exact structured events.
- **C2:** it proves `challenge < validated < migrated` and separately proves a post-return send at offset 48, but does not mechanically require `migrated < post-return-send` or exact cardinality of that post-return send.
- **C3:** server milestones are ordered, but exact post-return `seq=3` / offset 48 identity and one-only cardinality are not pinned.
- **C4:** both acknowledgement domains and the exact settled event exist and settled follows both, but one-only cardinality and exact identity of each domain still need mechanical assertions.

Do not create a parallel fixture. Tighten the existing count=4 process test using existing diagnostics first; add fields only where exact identity is otherwise impossible.

## P4 remains partial

The missing-Session-ACK post-return test requires nonzero exit and forbids `r9_udp_post_return_settled`, but still accepts a broad terminal-error disjunction and does not explicitly forbid `r9_udp_in_flight_settled` or downstream health/failover continuation. The missing-Carrier-ACK seam requires the same final tightening. Preserve both seams and make the comments mechanical before R9-2 closure.

## Continuous dependency-ordered queue

After H-R9-023: exact P2 C1-C4 -> post-return ACK arrival-order seam -> automatic-health replay exact-identity challenge -> P4 tightening -> final developer-local clean exact-tree provenance -> R9-3 data-loss recovery -> R9-4 ACK-loss/delayed-original/reorder -> R9-5 tamper/future-ACK/malformed negatives -> R9-6 pacing/cwnd/plaintext ownership -> R9-7 process/result truth -> R9-8/R9-9 authenticated warm TCP plus resolved-health promotion -> R9-10 uncertain Session replay/cleanup -> R9-11/R9-12 coherent gate and dedicated independent cross-process integration review -> Q10/Q11 factual reconciliation -> only then a specific `READY_LIVE` row may unlock one changed-hypothesis self-owned VPS run.

## Live / release boundary

R9-2 remains open on local evidence correctness. `READY_LIVE` remains none. Standing VPS authorization is valid; current blocker is local correctness/evidence and the later dedicated independent R9 integration review, not permission.
