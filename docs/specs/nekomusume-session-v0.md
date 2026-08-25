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
