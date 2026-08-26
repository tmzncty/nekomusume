# ADR M1-S3a: Crypto/Handshake Security Gate — Noise direction and synthetic contract

- **Date:** 2026-08-26
- **Status:** Candidate security gate (stage 1: documentation and synthetic contract)
- **Scope:** M1-S3a, no-dependency planning and test-contract stage

## Decision

M1-S3a takes the **Noise Framework** as the direction for a future authenticated
handshake. This ADR does not select a concrete Noise pattern, library, or
version, nor does it approve the license compatibility of any future
cryptographic dependency. Those choices require explicit approval at the
applicable **G0/G2** gates, after review of the threat model, API/maintenance
posture, dependency license compatibility, interoperability, and implementation
evidence. The project license decision in D010 remains accepted and unchanged:
`MIT OR Apache-2.0`.

Stage 1 is deliberately dependency-free. It establishes reviewable invariants,
rejection semantics, and synthetic test vectors only. It does not implement
Noise, TLS, AEAD, KDF, key or nonce generation, handshake processing, identity
loading, replay windows, path authentication, UDP encryption, runtime behavior,
or a network service. No Cargo manifest, production `src/` code, wire/carrier/
session/CLI code, or dependency is changed by this stage.

## Security contract for the future implementation

The following are requirements to be proved by later gates; they are not claims
that the current code provides these properties.

1. **Identity and authorization.** Peer identity must be authenticated before it
   is authorized for a session or path. Identity representation, trust roots,
   authorization policy, rotation, and failure handling remain to be specified;
   an authenticated cryptographic transcript alone is not authorization.
2. **Transcript and AAD binding.** Handshake transcript, negotiated parameters,
   protocol/version context, session/path context, direction, epoch, key phase,
   and all authenticated associated data must be bound consistently. Any
   mismatch is a hard rejection; unauthenticated metadata must never be used to
   authorize delivery.
3. **Direction, epoch, and key phase.** Send and receive directions are distinct
   domains. Epoch and key phase are explicit state, checked against the accepted
   transition rules, and cannot be silently swapped, rolled back, or mixed.
4. **Nonce uniqueness and overflow.** Every encryption nonce must be unique for
   its key and direction. Counters must use checked arithmetic; exhaustion or
   overflow is a terminal refusal to encrypt (never wrap, reuse, or continue).
5. **Replay and old epochs.** Replays, duplicates, stale packets, and packets
   from retired epochs are rejected according to a later-approved replay policy.
   A packet must not advance replay state before authentication succeeds.
6. **0-RTT.** 0-RTT data is disabled. No early application/session delivery is
   permitted before the approved handshake and authorization boundary.
7. **Resources and anti-amplification.** Pre-auth work is bounded by explicit
   size, CPU, memory, and pending-state limits. An unauthenticated peer must not
   induce disproportionate response traffic; rate limits, response quotas, and
   teardown behavior require later testable definitions.
8. **Logging and secret boundaries.** Logs and errors may identify a rejection
   class and safe correlation data, but must not contain keys, nonces, identity
   secrets, plaintext, transcripts, or raw authentication material. Secret
   buffers and ownership/lifetime rules require a later implementation review.

## Synthetic contract (stage 1)

The synthetic contract is a test oracle for state and rejection behavior, not a
cryptographic simulator. Fixtures contain labels, bounded metadata, and opaque
byte strings; they do not claim authentication or encryption success. Tests may
assert only that an input is accepted as a *syntactically valid candidate state*
or rejected with the specified class.

The fixture matrix must cover rejection of:

- authentication failure;
- transcript or payload tampering;
- wrong direction;
- wrong epoch;
- wrong key phase;
- AAD mismatch;
- truncation;
- duplicate input;
- oversize input; and
- nonce-counter overflow.

The contract must also assert the negative boundary: every rejection, and every
synthetic acceptance, produces **no** `SessionDelivery`, `PathValidated`, or
`ACK` evidence. In particular, fixtures must not be interpreted as proof of
peer identity, path validation, decryption, authorization, delivery, or receipt.
No fixture may require real key material, nonce generation, a handshake, a
socket, a runtime, or network I/O. If fixtures are added, they must be confined
to `tests/vectors/m1-s3a/` and remain plain-text or binary contract data only.

## Non-goals and gate boundary

M1-S3a stage 1 is **only a candidate security gate**. It is not a security
audit, does not freeze the protocol, and does not approve a Noise pattern,
library, version, or cryptographic-dependency license compatibility,
cryptographic construction, identity scheme, or wire encoding. The accepted
project license decision in D010, `MIT OR Apache-2.0`, remains unchanged. It does not establish production security or interoperability.
A later G0/G2 review must turn these invariants into implementation-specific
requirements and evidence before any cryptographic or handshake code is
permitted.

## Consequences

The project now has a stable, dependency-free review target without implying
that cryptography exists. Future implementation work must preserve the stated
negative evidence boundary and obtain the required gate approvals before
changing production code or dependencies.
