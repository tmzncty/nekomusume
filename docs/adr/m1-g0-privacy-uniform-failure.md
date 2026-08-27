# ADR M1-G0: Privacy-preserving uniform failure candidate contract

- **Date:** 2026-08-27
- **Status:** **Candidate only — non-frozen; no implementation authorization**
- **Scope:** Privacy and failure-observability review for the future M1-G0 handshake

## Decision

This ADR records a candidate contract for preventing identity and sensitive-state
leakage across handshake and authorization failures. It is documentation-only.
It does not freeze a protocol, establish a timing constant, prove a real
implementation, or approve a security gate. No Cargo, `src/`, test, runtime,
network, carrier, wire, or cryptographic change is authorized.

The external failure surface should be uniform for failures that must not reveal
whether a peer, identity, trust record, authorization policy, key, epoch,
replay state, path, or other sensitive state exists or was reached. The exact
failure code, response shape, retry behavior, and protocol mapping remain to be
specified and reviewed before any implementation. Uniformity is a contract
boundary, not a claim that all executions take constant time.

Internal diagnostics may remain bounded and actionable for operators and review,
but must not cross the external boundary or become a new correlation channel.
No diagnostic may promote a failed or partial operation into protocol evidence.

## Identity and state leakage boundary

The candidate external behavior must not disclose, directly or by distinguishable
combination, any of the following: peer or device identity, identity-key
identifier, trust-store membership, issuer or provenance, revocation or expiry
state, authorization policy result, key/epoch/key-phase state, replay-window
state, path or carrier existence, session existence, parse/decode detail, or
whether a cryptographic step was reached or failed. Error text, status/code,
response size and shape, retry/close behavior, and externally visible counters
must be reviewed as one surface.

Discovery and correlation inputs such as mDNS names, IP addresses, peer IDs,
carrier IDs, path IDs, session IDs, data IDs, and packet/stream identifiers are
not authorization or trust evidence. They must not be reflected into an
unauthenticated failure in a way that distinguishes sensitive state. Identifiers
that are necessary for an authorized operator workflow require an explicitly
reviewed, access-controlled handling rule; presence or parseability is not trust.

Unknown, malformed, unsupported, downgraded, revoked, expired, corrupt,
unauthorized, authentication-failed, replay-rejected, resource-limited, and
indeterminate inputs remain fail-closed. This ADR does not define their final
wire encoding or select whether any response is sent.

## Uniform external failure and bounded internal diagnostics

The candidate external boundary has one privacy-preserving failure class (or a
reviewed equivalence class) for sensitive handshake/authorization failures.
It must not expose a reason string, stack, identity detail, trust lookup result,
policy name, key state, parse offset, retry oracle, or per-peer distinction.
A response, when later authorized by a separate protocol decision, must be
bounded in size and work and must not amplify the request. Absence of a response
must not be treated as proof of a particular internal reason.

Internal diagnostics are for bounded, access-controlled local operations only.
They may carry a coarse failure category and a short-lived opaque correlation
handle, but only under an approved schema with fixed cardinality, bounded byte
length, bounded retention, rate limits, and redaction at construction. Raw peer
input and sensitive identifiers must not be used as diagnostic labels. The
handle must be non-reversible and must not be sufficient to recover identity,
trust, key, path, or session state. Diagnostics must be dropped or collapsed
when their budget is exhausted; they must never cause a retry or response.

This separation is a candidate design requirement, not a timing-constant claim,
side-channel proof, or evidence that any real implementation satisfies it.

## Logs, metrics, packet capture, and correlation

The following are candidate redaction requirements for every failure path,
including error handling and cleanup:

- **Logs:** never record identity material, identity-key IDs, trust contents,
  raw peer input, secrets, keys, nonces, tokens, payloads, or unreviewed network
  identifiers. Use bounded coarse categories and approved opaque handles only.
- **Metrics:** use a fixed, low-cardinality failure class; do not label by peer,
  identity, IP, port, session, path, carrier, key phase, packet, stream, or
  parse detail. Counts and dimensions must have bounded retention and access.
- **PCAP/traces:** failure traffic and trace annotations must not expose payload,
  identity, secrets, keys, nonces, tokens, or sensitive correlation IDs. Capture
  policy, retention, access, and export must be explicitly reviewed; disabling
  capture is preferable to unsafe capture.
- **Correlation:** no value may be reused across external responses, logs,
  metrics, traces, PCAP, retries, or sessions unless an approved, opaque,
  access-controlled correlation design proves that it cannot disclose sensitive
  state. Cross-system joins must be bounded, auditable, and redacted by default.

### PCAP and trace metadata boundary

Capture is **disabled by default** for this candidate. No implementation may enable
packet capture, wire capture, event tracing, or equivalent recording merely for
debugging or review. If a separately approved exception permits a capture, it
must be limited to synthetic or minimized data produced for that purpose; it
must not contain production payloads or identifiers. The approval must name the
scope, owner, purpose, expiry, retention limit, access list, export prohibition
(or explicit destination), and redaction procedure before capture starts.

The approved capture/trace schema must prohibit, or remove before formatting,
serialization, storage, aggregation, export, or access, all of the following
metadata: IP addresses, ports, peer/device/carrier/path identifiers, session,
connection, packet, frame, datagram, and stream IDs; payload and headers that
carry equivalent identifiers; lengths, sizes, offsets, sequence numbers,
acknowledgements, timestamps, inter-event timing, ordering, direction,
retransmission, loss, timeout, retry, reset, shutdown, close, and handshake
patterns; and any other field that can distinguish a failure state, reached
state, peer, path, carrier, session, or implementation branch. This prohibition
applies equally to trace attributes, filenames, indexes, labels, sidecar
metadata, and derived aggregates. A field is not safe merely because it is
hashed, truncated, encrypted, sampled, or kept out of the payload.

Any approved synthetic/minimized capture must use fixed schemas and bounded
work, size, retention, and cardinality. Redaction must be fail-closed at
construction and be verified before persistence; post-hoc filtering is not
sufficient. Capture files and trace stores must have explicit access control,
short bounded retention, auditable reads, and no export, upload, or cross-system
join unless separately approved with the same redaction and expiry controls.
When an approval expires, its artifacts must be securely deleted or irreversibly
redacted. If a required field cannot be proven safe, capture is denied and the
artifact is discarded. No capture or trace may be used to reconstruct the
forbidden metadata or to classify distinguishable failure outcomes.

These controls reduce observability risk only. They do not eliminate timing,
traffic-analysis, or other side channels, and no PCAP/trace, synthetic fixture,
minimized sample, or redaction result may be presented as evidence of side-
channel elimination or as a G0 PASS.

Redaction must occur before formatting, serialization, export, or aggregation;
post-hoc filtering is not sufficient. Test fixtures and review artifacts must
use synthetic values and must not be treated as production privacy evidence.

## Failure cleanup ordering and evidence barrier

A future implementation must define and review a fail-closed ordering. The
candidate ordering is:

1. Stop admission and prevent delivery, path validation, ACK, retry, or other
   externally meaningful promotion as soon as failure is determined.
2. Revoke or invalidate tentative identity, authorization, transcript, replay,
   key, epoch, path, carrier, and session state; do not retain partial trust.
3. Release bounded buffers, queues, handles, and sensitive material according
   to their separately reviewed lifetime rules; zeroization and crash behavior
   remain open implementation questions.
4. Construct only the approved uniform external outcome, if any, without raw
   failure detail or sensitive correlation data.
5. Emit only bounded, redacted internal diagnostics after the evidence barrier;
   diagnostics cannot reopen admission or alter the external outcome.

This ordering is a candidate contract, not a proof of cleanup, zeroization,
race freedom, or crash safety. A failed or partial operation must produce no
`Delivery`, `PathValidated`, or `ACK`; it must also produce no equivalent
session/path authorization evidence. Authentication does not imply
authorization, and either failure must remain isolated from delivery and path
promotion.

## Review gates and non-escalation

D018 is **candidate/non-frozen** and requires a later explicit G0/G2 review,
real implementation review, reproducible negative tests, and independent
two-person security/design review. Until those gates pass, this ADR must not be
described as selected, frozen, approved, implemented, timing-constant, or
security-proven. It does not authorize implementation, merge, dependency
selection, network exposure, or cryptographic claims.

The candidate remains bounded by the existing D014–D017 contracts and their
non-escalation rules. If this ADR conflicts with another document, G0 is STOP:
record an explicit reviewed amendment or superseding ADR before implementation.
No synthetic contract, fixture, log sample, metric, trace, or PCAP can prove
that a real implementation meets this privacy boundary.

## Open questions before any freeze

The review must still resolve the exact equivalence classes and response policy,
maximum work and response budgets, cleanup behavior under cancellation/crash,
redaction schema and key management for opaque handles, capture/export controls,
retention and access policy, concurrency/race behavior, and negative test
vectors. It must also define how operators investigate incidents without
reconstructing identity or sensitive state from cross-system correlation.
