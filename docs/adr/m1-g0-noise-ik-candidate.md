# ADR M1-G0: Noise IK candidate contract (25519/ChaChaPoly/SHA256)

- **Date:** 2026-08-26
- **Status:** **Candidate only — not frozen, not implementation approval**
- **Scope:** G0 candidate review for a future M1 authenticated handshake

## Decision

`Noise_IK_25519_ChaChaPoly_SHA256` is recorded as a **candidate only** for the
future authenticated handshake. This is a review target, not a protocol freeze,
cryptographic implementation decision, dependency approval, or interoperability
claim. No Cargo manifest, dependency, `src/`, test, network, or runtime change is
authorized by this ADR.

The candidate name follows the Noise naming convention: IK pattern,
Curve25519 DH, ChaChaPoly AEAD, and SHA-256 hash. The name is not evidence that
any library, version, API, license compatibility, wire encoding, identity store,
or operational policy has been selected.

`docs/specs/nekomusume-session-v0.md` remains a provisional normative-source
entry point. Its **v0 normative** heading does not mean that the protocol is
frozen. D012 `Accepted` is limited to the M1-S1 loopback UDP slice acceptance
criteria; it is not acceptance of Noise, authentication, authorization,
production transport, or this candidate contract.

## Unresolved blockers

The following must be resolved with explicit design, threat-model, and test
 evidence before G0 can pass, and before any cryptographic or handshake
implementation is considered:

1. **Trust and identity governance:** trust-store format and versioning,
   signature rules, identity rotation, revocation, and rollback behavior.
2. **Authentication versus authorization:** authenticated identity is not
   authorization. The session/path authorization policy, trust-root selection,
   and denial semantics remain open.
3. **Privacy and failure behavior:** identity leakage resistance and a uniform
   failure surface (including timing, error text, and observable response
   behavior) remain open.
4. **Transcript domains:** the prologue and AAD must be strictly separate,
   explicitly specified, and bound to the correct transcript/context domains;
   no accidental reuse or cross-domain interpretation is allowed.
5. **Wire contract:** the complete header and wire-field mapping, including
   authenticated versus unauthenticated fields, encoding, lengths, and version
   behavior, remains unresolved.
6. **Nonce lifecycle:** uniqueness across restart, concurrency, crash recovery,
   key/epoch changes, and rollback must be demonstrated; reuse and silent
   counter wrap are forbidden.
7. **Replay state:** replay-window capacity, TTL, eviction policy, and behavior
   under memory pressure and restart remain unresolved.
8. **Epoch transitions:** key phase and old-epoch acceptance/retirement rules,
   ordering, overlap, rollback, and deletion semantics remain unresolved.
9. **Resource and anti-amplification budget:** absolute pre-auth and established
   state budgets for CPU, memory, concurrency, packet/response work, queues,
   and traffic amplification remain unresolved and must be enforceable.
10. **Early data and evidence domains:** 0-RTT is disabled. ACK, path, and
    session evidence must remain isolated and must not be fabricated or promoted
    by authentication failure, synthetic fixtures, or partial handshakes.

## G0 PASS conditions

G0 may be marked **PASS** only when all of the following are reviewable,
versioned, and backed by reproducible evidence:

- A threat model and security review explicitly approve or reject this exact
  candidate, its pattern assumptions, and its intended identity/authorization
  boundary.
- Trust-store versioning, signatures, rotation, revocation, rollback, identity
  privacy, and uniform failure behavior have normative rules and negative tests.
- Prologue/AAD domain separation and the complete header/wire mapping are
  specified with canonical encoding and tamper/mismatch rejection vectors.
- Nonce uniqueness across restart, concurrency, epoch/key phase, crash, and
  rollback is proven by design and tested; exhaustion fails closed.
- Replay capacity/TTL/eviction and key-phase/old-epoch transitions are bounded,
  specified, and tested, including restart and resource-pressure cases.
- Absolute CPU, memory, concurrency, queue, response, and anti-amplification
  budgets are stated, instrumented, and tested; 0-RTT remains disabled.
- ACK/path/session evidence isolation is demonstrated, and authentication is
  shown not to imply authorization or delivery.
- The selected implementation dependency (if any) has an approved version,
  maintenance/interoperability review, and license review under D010; this
  ADR is amended or superseded before implementation begins.

## STOP conditions

Stop G0 and do not implement, merge, or claim security if any of these occurs:

- The candidate is treated as selected/frozen, or a v0 normative title is used
  as proof of freeze.
- Trust-store/version/signature/rotation/revocation/rollback rules are absent,
  ambiguous, unverifiable, or fail closed neither on corruption nor downgrade.
- Authentication is used as authorization, identity is exposed, or failures
  reveal distinguishable sensitive state.
- Prologue and AAD are conflated, or any header/wire field lacks an explicit
  authenticated-domain mapping.
- Nonce uniqueness cannot be guaranteed across restart, concurrent senders, or
  rollback; counters wrap or overflow is not a terminal refusal.
- Replay limits, TTL, eviction, epoch/key-phase, or old-epoch behavior are
  unbounded or unspecified.
- Any CPU/memory/concurrency/traffic budget or anti-amplification limit is
  missing, unenforced, or exceeded; 0-RTT is enabled.
- ACK/path/session evidence crosses its boundary, or partial/authentication
  failure produces delivery, path validation, or ACK evidence.
- Required evidence is replaced by synthetic success, an unreviewed dependency,
  fuzz-smoke output, or an implementation change outside the two allowed docs.

## Consequences and next gate

This record makes the candidate auditable while preserving the existing
negative boundary. It does not authorize code or dependency changes. The next
step is a focused G0 review that resolves every blocker and records evidence;
until then the candidate remains **candidate only**.
