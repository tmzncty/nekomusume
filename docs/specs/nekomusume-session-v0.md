# Nekomusume Session v0 — normative entry point

**Status: provisional normative entry point; not frozen.**

This is the single canonical entry point for Session v0. Current Rust models
are bounded, synchronous, and transport-independent. They do not implement
sockets, runtime integration, cryptography, or live failover.

## Candidate evidence boundary

The candidate ledger's `confirm_received` proof means that the peer accepted
logical bytes for transport delivery. It does not prove application delivery
(`delivered`) or an application side effect (`effect`). Packet feedback, path
validation, and Session delivery remain separate evidence domains.

The remaining wire, identity, replay, cryptographic, and live-carrier gates
must be frozen by reviewed ADRs and deterministic vectors before conformance
may be claimed.


## Delivery segment overlap invariant

An inserted fragment may merge overlapping or contiguous existing segments only
when their union covers every byte in the resulting range. If a proposed merge
would bridge an uncovered gap, the insertion is rejected with `Conflict`; the
ledger never zero-fills or otherwise synthesizes missing bytes. Exact duplicates
and byte-identical overlaps are idempotent.

## Context commit invariant

`insert` validates all bounds, stream, offset, reorder, overlap, and byte-limit
conditions before committing `SessionContext`. A rejected insertion leaves the
ledger context and delivery state unchanged. This remains a bounded state-model
invariant, not a claim of complete protocol validation.

## Governance boundary (G0)

This document is a **provisional normative entry point**, not a frozen
protocol and not an implementation approval. Its existence, heading, or any
bounded model must not be used to claim that a wire format, Noise IK, a
library, a dependency, or key material has been selected.

D012 `Accepted` remains limited to the `127.0.0.1` loopback UDP slice. D014 is
limited to the Noise direction and dependency-free synthetic contract. D015 is
only a candidate review target. D010 only records the project license/SPDX
expression; it does not approve dependencies. Synthetic contract output and
research material do not create authentication, path, delivery, or security
evidence.

The threat model is research input only and is non-normative, not an audit, and
not an approval. No `Accepted` record implicitly escalates a candidate. If this
entry point conflicts with an ADR or research record, G0 is **STOP**; an
explicit new ADR (or explicit reviewed amendment) is required before any
selection, freeze, implementation, merge, or security claim.

## Unreliable datagram boundary (provisional)

The existing `seal_unreliable`/`open_unreliable` APIs are authenticated record
operations, not a downgrade switch for Session semantics. They do not turn an
inner reliable `ProcessMessage::Data` into an unreliable message: reliable
Session delivery retains its ordering, retransmission, flow-control and
independent delivery-evidence contract. An opened unreliable record is not an
application delivery receipt.

For an eventual datagram-shaped Session message, the candidate semantics are:
a payload cap of 1200 bytes distinct from authenticated envelope and
carrier/path limits; bounded drop/admission with no retransmission or ACK;
no ordering guarantee; no independent flow/congestion algorithm; and explicit
separation of mixed reliable/unreliable queues so unreliable traffic cannot
starve reliable delivery. Candidate counters may describe offered/admitted/
opened/drop/rejection outcomes, but are diagnostic only and do not promote
packet or carrier observations to `delivered` or `effect`. This paragraph is
provisional and introduces no wire/API/code/runtime/WAN change. The existing
bounded probe remains the CLI surface; no user-level datagram CLI is added.
