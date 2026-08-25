# Nekomusume Session v0 — normative entry point

**Status: candidate gate; not a frozen protocol specification.**

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
