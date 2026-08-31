# Era-4 L: protocol release and versioning slice

**Status: candidate release process, not a protocol freeze or production release.**

This document defines the first Era-4 L slice at a specific commit. It makes
release claims auditable without declaring the current candidate wire format a
stable protocol. The authoritative release object is the tuple `(release_id,
candidate commit, capability manifest, evidence set)`; the manifest records the
exact parent separately so it remains self-contained without a hash cycle. a tag or crate version alone is
not release evidence.

## Release criteria

An Era-4 protocol release candidate may be called **candidate** only when all
of the following are true:

1. The exact parent and candidate commit are recorded (the parent is in the manifest; the candidate is the commit carrying this manifest) and reproducible; the
   manifest schema validates and every capability has an evidence class and
   explicit limits.
2. Canonical wire vectors cover valid, truncated, oversized, unknown-version,
   unknown-type and reserved-flag cases. Unknown critical input fails closed;
   ignorable extensions are bounded and preserved only where specified.
3. Negotiation is authenticated before enabling a capability. Both peers
   advertise a bounded version range and capability set; the selected version
   is the highest mutually supported version within policy. No unilateral
   assumption enables a feature.
4. Downgrade is explicit and fail-closed: a peer may select a mutually
   supported lower version, but an absent, malformed, contradictory, or
   unauthenticated negotiation is a rejection—not an implicit downgrade.
   A selected version/capability transcript is bound to the authenticated
   session and cannot change mid-session.
5. Mixed-feature behavior has a declared boundary: each feature is enabled
   only if negotiated by both peers; reliable and unreliable traffic retain
   separate delivery/ack/ordering domains; unsupported optional features are
   rejected or omitted without changing the semantics of enabled features.
6. Unit/property/interoperability vectors and repository policy gates pass;
   evidence is labeled E0–E4 and no E3 observation is promoted to E4 review.
7. Independent security review, compatibility policy, rollback, and authorized
   network evidence exist before **released**, **production**, or **public**
   wording. These are intentionally absent from this slice.

Criteria 1–6 establish a reviewable candidate boundary only. They do not freeze
field assignments, promise previous/current interoperability, or authorize a
listener, WAN run, 0-RTT, FEC, multipath, proxy, or tunnel.

## Capability negotiation and downgrade contract

A capability identifier is an opaque, bounded token with a registry owner and
semantic version. The candidate registry records support as `supported`,
`experimental`, or `blocked`; only `supported` entries may be selected. The
intersection is computed after authentication and policy filtering. An empty
intersection is a clean `no_common_capability` failure unless the base session
version remains usable without optional features.

A downgrade is valid only when `selected_version < offered_max` and
`selected_version >= offered_min`, both peers advertised it, and the
transcript verifies. It must emit a reason-coded diagnostic. A peer must reject
rollback, version gaps, duplicate capability identifiers, unknown mandatory
capabilities, or a selected capability not present in both offers. Negotiation
is one-shot for a session; re-negotiation is a future protocol decision.

## Mixed-feature conformance boundary

Conformance is tested per feature and per pair, not inferred from a feature's
presence in one peer. The minimum matrix is: base-only/base-only,
base+reliable, base+unreliable, reliable+unreliable mixed traffic, optional
feature offered but declined, and incompatible version ranges. A mixed result
must report each domain independently. In particular, unreliable datagrams do
not receive retransmission, ACK, ordering, or flow/congestion semantics from a
reliable stream, and cannot starve reliable delivery. Unsupported optional
features may be omitted; mandatory mismatches fail the session before data.

## Not yet release evidence

The repository now contains the bounded N1 wire-negotiation primitive and TCP
multistream binds the exact N1 transcript into Noise before Session data
admission, with executable positive and malicious-peer coverage. That is narrow
authenticated-admission evidence for the TCP multistream path, not a global
negotiation claim: probe, failover/resume and UDP paths have not thereby proven
version negotiation, downgrade handling, replay/amplification safety or
interoperability.

Current/current and unsupported/future compatibility behavior and executable
candidate vectors exist, but the corpus remains `freeze=false`; no previous
frozen release exists for previous/current interoperability. Remaining release
Remaining L work includes independent review, a reviewed freeze decision, the
missing path matrix above, sustained WAN/release evidence, and rollback/release
governance.
