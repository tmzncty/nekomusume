# Reviewer checkpoint — R9-2 P3 exact terminal oracle at `2bddf1d`

## Anchor and classification

- Reviewed developer-owned source/test commit: exact `2bddf1d5ce255f94d990d56dffcd6f2e2dae9058` (`test(cli): P3 exact terminal oracle — applied ACK at malformed=2, typed bound, no continuation (H-R9-023)`).
- Parent reviewer handoff: exact `6cd1acd9522e1e149bc79fff6f3226fde08971a9`.
- Change class: built-binary test/oracle only (`crates/neko-cli/tests/probe.rs`); no Session, Carrier, ACK, crypto, wire, decoder, framing, policy-value, or WAN behavior change.
- Open PRs at review time: none.

## Inspected owners and applicable claims

This bounded recheck inspected the exact-current P3/P2 process tests, the reliable UDP authenticated receive/settlement owner, the TCP uncertain-replay path, and the post-migration reliable-UDP owner, against the current architecture/spec/governance boundaries in `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, `docs/standing-vps-lab-authorization.md`, `docs/vps-rental-window-priority.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, `docs/release-security-review-packet.md`, and the prior `docs/CHATGPT_HANDOFF.md`.

The challenged H-R9-023 claim was: the malformed-budget P3 process fixture must prove one real Carrier ACK application between malformed #2 and malformed #3, then terminate specifically on the existing malformed bound without silently taking a successful settlement, rejection, health, failover, or migration continuation path.

## H-R9-023 — CLOSED NARROWLY

Exact `2bddf1d` closes the four concrete oracle gaps from the prior handoff:

1. It collects `r9_udp_packet_ack_applied` event lines and requires **exactly one**.
2. That selected applied event must carry the live operation-wide `malformed=2` discriminator, while the fixture forbids any `r9_udp_packet_ack_rejected` event.
3. The client must exit nonzero and stderr must contain the existing terminal `UDP delivery acknowledgement malformed bound exceeded` error, rather than a broad `malformed || bound` disjunction.
4. After the bound, the fixture forbids both reliable settlement markers plus downstream UDP health, TCP-warm, and migration-back continuation markers.

The applied-event instrumentation itself is still emitted at the actual `UdpAcknowledgement::Carrier { applied: true }` handling point, so the tightened test is checking Recovery-owned feedback, not a synthetic pre-apply marker. The terminal branch uses the existing fail-closed process exit; no source semantic repair was required for this closure.

This is an evidence/oracle closure only. It does not establish that R9-2 as a whole is complete.

## Exact-tree hosted cross-evidence

GitHub-hosted checks on exact `2bddf1d` are both successful:

- `stable checks` — SUCCESS;
- `nightly decode fuzz smoke` — SUCCESS.

The reviewer did **not** run a separate local checkout gate in this pass. Hosted CI remains cross-evidence only, and the required final developer-local clean exact-tree provenance for R9-2 is still absent.

## Remaining concrete R9-2 work

The current count=4 migration-back P2 fixture is positive, but its acceptance oracle is still weaker than its comments/claims in several mechanically repairable ways:

- **C1 TCP replay identity/cardinality:** it counts one client `tcp_delivery_ack_validated` event but does not yet require that event to be exactly `seq=2`, stream 1, offset 32, nor mechanically exclude replay evidence for offsets 0/16/48. The corresponding server logical ACK identity/cardinality is likewise not pinned.
- **C2 post-return ordering/cardinality:** the client proves challenge `<` validation `<` migration, and separately proves post-return offset 48 exists, but it does not yet require exactly one of each event nor the full strict chain `challenge < validated < migrated < post-return send`.
- **C3 server post-return identity/cardinality:** the server proves first-occurrence order `owner started < validated < Session DACK sent < packet ACK sent`, but not exactly-one cardinality or full logical identity for the offset-48 record.
- **C4 dual-domain settlement:** the client proves both acknowledgement-domain event names occur before a settled marker, but not exactly-one cardinality and exact logical/domain identity for those selected events.

These are test/instrumentation closure tasks, not a reason to invent a parallel fixture or new protocol semantics.

A separate source-level item remains genuinely open in the automatic-health replay path: the client currently reduces `FailoverController::tcp_resend()`'s exact `(DataId, payload)` ownership set to `.len()` and then reconstructs replay records positionally from `records.skip(uncertain_start)`. The controller already owns the exact identity/payload set, so the later R9-2 slice must preserve and validate that set rather than infer equivalent ownership from a count.

## Exclusions and governance boundary

- No WAN/VPS experiment was run or reclassified.
- `READY_LIVE` remains none.
- No decoder/parser/crypto-framing source changed in `2bddf1d`; the hosted fuzz success is extra cross-evidence, not a newly required gate for this test-only commit.
- No D019, retained-state capacity, TTL/LRU/history-size, signing/key-custody/SBOM/publication, frozen-release, destructive migration, RC/freeze/release/production, or adversarial-capacity policy was decided.
- Release item 3 and item 4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.

## Reviewer disposition

H-R9-023 is closed narrowly. R9-3 remains dependency-blocked on the remaining R9-2 exact P2/ACK-order/automatic-replay/P4 evidence-correctness work and final developer-local exact-tree provenance. The external coding agent should continue those READY_LOCAL slices immediately without waiting for the next reviewer cadence.
