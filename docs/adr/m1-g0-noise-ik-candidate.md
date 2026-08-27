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

## G0 non-escalation rules

This ADR is a candidate-review target only. It must not be described as
selected, frozen, approved, or **PASS**, and it does not select IK, a library,
a dependency, or key material. D014 is limited to the Noise direction and its
dependency-free synthetic contract; synthetic results are not authentication,
interoperability, implementation, or security-approval evidence.

D010's `MIT OR Apache-2.0` is the project license/SPDX decision only. It does
not approve a cryptographic or other dependency. D012 `Accepted` is limited to
the `127.0.0.1` loopback UDP slice and does not escalate to Noise,
authentication, authorization, production transport, or this candidate.

The v0 document is a provisional normative entry point, not a frozen protocol.
The threat model is research input only: it is non-normative, not an audit, and
not an approval. No `Accepted` decision may implicitly promote a candidate or
research result. If documents conflict, G0 is **STOP**: do not implement,
merge, or claim approval; record an explicit new ADR (or an explicit amendment
with the same review gate) before proceeding.

## Trust and authorization candidate boundary (non-frozen)

The following is a **candidate-only, non-frozen contract for review**. It is not
an implementation specification, and every item requires later real
implementation, reproducible negative tests, and two-person security/design
review before it can be frozen.

A future trust record is a candidate tuple with explicit fields for: record
schema/version; subject identity and stable identity key or key identifier;
trust root/issuer and source/provenance; status; validity and policy scope;
rotation lineage and activation/overlap metadata; revocation reference and
time; and rollback protection/precedence metadata. The exact field names,
canonical encoding, cryptographic proof, version negotiation, source
validation, rotation procedure, revocation distribution/freshness, and
rollback rules remain **to be frozen**. A record is not trusted merely because
it is present, parseable, locally cached, or supplied by a peer.

Authentication and authorization are separate decisions. Successful
cryptographic authentication may establish a candidate peer identity only; it
does not grant permission. Authorization must be evaluated independently
against policy and an applicable trust record, and must be explicitly bound to
the session and the carrier context (including the authenticated transcript,
carrier instance/role, direction, epoch/key phase, and relevant path/context
identifiers). A decision for one session, carrier, direction, epoch, or path
must not be reused as authorization for another. mDNS names, IP addresses,
and peer IDs are routing/discovery or correlation inputs only and must never
be used as authorization predicates or trust roots.

The candidate failure boundary is fail-closed: unknown, revoked, expired,
invalid, malformed/corrupt, unsupported, downgraded, or otherwise
indeterminate trust state denies authentication-dependent authorization and
must not be promoted by cache fallback, rotation ambiguity, rollback, or
partial parsing. Authentication, authorization, and trust failures must not
produce or imply `Delivery`, `PathValidated`, or `ACK` evidence. Before
authentication completes, no such evidence may be created; before the
independent authorization decision succeeds, delivery and path/ACK evidence
remain prohibited as well.

This section is deliberately **candidate/non-frozen**. It requires subsequent
real implementation and independent dual review; it does not authorize any
Cargo, dependency, `src/`, test, runtime, or network change and does not
constitute security approval, interoperability evidence, or a G0 PASS.

## Candidate wire field map (non-frozen)

This is a review map only. It describes the currently visible framing seam and
candidate future bindings; it does not define a frozen wire format, prove an
AEAD construction, or authorize a codec change.

| Candidate field | Candidate representation/boundary | Candidate authentication domain | Status and open questions |
|---|---|---|---|
| `NK` | Fixed magic/preamble at the start of the record | Candidate header context; exact inclusion and representation are open | Candidate only; value, width, and rejection behavior require G0/G2 review and vectors |
| `version` | Version field after `NK` | Candidate authenticated header context | Candidate only; negotiation, downgrade binding, and unknown-version behavior are open |
| `type` | Record/message-type field | Candidate authenticated header context | Candidate only; registry, authorization implications, and unknown-type behavior are open |
| `flags` | Flags field in the candidate header | Candidate authenticated header context | Candidate only; reserved bits, criticality, and canonical zero/unknown-bit behavior are open |
| `length` | Length of the candidate payload boundary, not an implicit message/session length | Candidate authenticated header context | Candidate only; integer width/encoding, maximum, overflow, exact-consumption, and whether a tag is counted remain open |
| `payload` | Exactly the bytes selected by the candidate `length` boundary | Candidate ciphertext/plaintext domain depends on the later protocol stage | Candidate only; content type, padding, tag placement, limits, and trailing-byte behavior require vectors |

The map above is deliberately limited to `NK/version/type/flags/length/payload`.
It must not be read as freezing, or even selecting, fields for
`session_id`, `carrier_id`, `path_id`, `data_id`, `stream`, `offset`,
`delivery_epoch`, `key_phase`, `packet_sequence`, ACK, or path challenge.
Those names are research identifiers only: presence, width, signedness,
encoding, scope, uniqueness, lifecycle, replay binding, ordering, and
cross-carrier semantics are all **not frozen**. In particular, neither a
packet sequence nor an ACK/path-challenge value may be treated as delivery,
path-validation, or authorization evidence merely because it is parseable.

### Canonical encoding and malformed-input candidate policy

The candidate requires one canonical encoding per field before implementation:
fixed-width versus variable-width integers, byte order (endianness), length
unit and maximum, permitted padding, and exact record consumption must be
specified together and covered by golden, tamper, truncation, boundary, and
negative vectors. No alternate byte order, non-minimal integer form, implicit
native-endian conversion, or silently accepted trailing data may be assumed.
The length boundary must be checked before allocation or authentication work;
overflow, impossible bounds, truncation, and inconsistent framing are
candidate decode failures.

Unknown `version` and unknown `type` behavior is unresolved and must be made
explicit per message class (for example, uniform rejection versus a safely
skippable extension); unknown or reserved `flags` must not silently change
security meaning. Malformed, truncated, overlong, overflowed, non-canonical,
unsupported, or otherwise ambiguous records must fail closed without producing
session, delivery, ACK, or path evidence. Error surface, response behavior,
resource cost, and anti-amplification consequences remain open review items.

### Candidate authenticated-field boundary

The candidate assumption is that security-relevant header fields are bound as
associated data only after the final field map and canonical encoding are
approved; payload authentication/encryption and tag placement are separately
specified. The exact authenticated subset, ordering, tag coverage, and whether
`NK`/version are authenticated as raw bytes or normalized fields remain
**unresolved** and require G0/G2 design review plus reproducible vectors and
implementation review. No field is declared authenticated merely by appearing
in this document.

The Noise prologue is a separate transcript/context domain from per-record AAD.
A prologue value must not be reused as AAD, and AAD must not be used as a
prologue substitute. This document is not a real AEAD proof, does not establish
nonce/key/tag behavior, and must not be cited as authentication or
interoperability evidence. Any future binding must also show direction,
epoch,
key phase, carrier/path context, replay state, and packet ordering without
promoting unauthenticated or partially parsed values.

This entire field map is **candidate/non-frozen** and requires G0/G2 approval,
canonical test vectors, implementation review, and negative/security review
before any implementation or claim. Until those gates pass, the existing
framing is a candidate boundary only.
