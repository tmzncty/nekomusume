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
