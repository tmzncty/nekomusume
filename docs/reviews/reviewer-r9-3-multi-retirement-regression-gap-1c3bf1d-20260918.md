# Independent bounded review — H-R9-049 multi-retirement projection regression gap

**Reviewed source/test anchor:** exact `1c3bf1d30e3089da230b2aedce54dbcecd6107f9` (`fix(cli): H-R9-048 emit positive evidence for EVERY retired packet in a multi-retirement ACK`).

**Repository HEAD observed by reviewer before this note:** `8029700f976ee6c9842ed01bf74c3f2d6668685b` (docs-only handoff descendant of `1c3bf1d`).

## Scope

Inspected exact-current owners:

- `crates/neko-cli/src/main.rs`: post-return `UdpAcknowledgement::Carrier` projection and `recv_udp_delivery_ack` typed `acked_packets` handoff;
- `crates/neko-reliable/src/lib.rs::Recovery::on_ack`: one canonical `AckRanges` may retire more than one outstanding packet and returns every retired packet in `RecoveryResult.acked_packets`;
- `crates/neko-cli/tests/probe.rs`: current R9-3 process oracle, especially `reliable_udp_post_return_data_loss_recovers_via_pto_retransmit`;
- current `docs/CHATGPT_HANDOFF.md` closure claim for H-R9-048.

The reviewer also checked GitHub-hosted Rust CI run `35292537222` for exact `1c3bf1d`: `stable checks` and `nightly decode fuzz smoke` both completed successfully. Hosted CI is cross-evidence only.

Excluded: no WAN/VPS execution, no performance conclusion, no wire/crypto/parser change, no release-authority decision, no new capacity/policy value.

## Invariant challenged

If one authenticated canonical Carrier ACK retires packet identities `{p1, p2, ...}` in the typed Recovery result, process evidence must expose the complete same set of positive Carrier-retirement identities. A later regression back to a single `.last()`/representative identity must fail a deterministic test.

## Finding — HIGH evidence/oracle correctness

The source repair in `1c3bf1d` is directionally correct: it iterates `acked_packets` and emits one `r9_udp_return_packet_ack` for each retired packet identity.

However H-R9-048 was closed without the discriminating regression required by the review -> repair contract:

1. `1c3bf1d` modifies only `crates/neko-cli/src/main.rs`; it adds no focused regression.
2. The current `crates/neko-cli/tests/probe.rs` contains no H-R9-048 / multi-retirement-specific fixture.
3. The existing repeated-PTO R9-3 oracle checks **soundness of emitted positives** — every emitted retirement PN must belong to the retransmit set and have a matching server ACK — but it does not force one ACK to retire at least two outstanding packets and does not compare the complete typed `acked_packets` set with the emitted diagnostic set.
4. Therefore the pre-fix `.last()` projection could be reintroduced while the current process test suite still remains green unless scheduling happens to create an unasserted multi-retirement case. Green hosted CI at `1c3bf1d` cannot substitute for a missing discriminator.

This does **not** establish a new Recovery/Session state-machine defect. The current implementation loop appears to preserve every identity handed to it. The blocker is that the claimed H-R9-048 closure is not acceptance-grade evidence yet.

## Smallest repair / acceptance contract

Keep the current source semantics unless the regression exposes a contradiction. Add one focused deterministic regression that reaches the actual post-return projection owner and makes a single canonical Carrier ACK retire **at least two** currently outstanding packet copies.

Acceptance must prove:

- the typed retirement set for that ACK has cardinality `>= 2`;
- emitted positive `r9_udp_return_packet_ack.packet_number` values for that ACK have exactly the same set and cardinality as the typed retired-packet set (no omitted identity, no duplicate identity, no representative-only `.last()` behavior);
- the same ACK is not also projected as rejected or accepted-empty;
- Session `DeliveryAck` remains a separate evidence domain and does not acquire a second logical transition merely because multiple Carrier packet copies retire;
- final Recovery/Session settlement remains truthful.

Prefer the smallest existing test-only fault/seam that can make the server return an ACK range covering two outstanding siblings. If reaching the process owner otherwise requires disproportionate framework churn, a tiny extracted projection helper is acceptable only if the test exercises the exact helper used by the runtime and separately proves that `recv_udp_delivery_ack` forwards the full `acked_packets` vector unchanged. Do not change ACK architecture, wire format, crypto framing, D019, or any capacity/security value.

After the regression is pushed, run the normal developer-local clean exact-tree gate on the final source/test SHA and persist exact provenance. No decoder/framing change is expected, so fuzz is not mechanically required beyond ordinary hosted cross-evidence.

## Classification

`H-R9-049 = HIGH / evidence-oracle correctness`.

R9-3 remains blocked until this discriminator closes. H-R9-041 exact-wire refusal regression and H-R9-042 deterministic pre-deadline negative remain valid downstream work and must stay queued.
