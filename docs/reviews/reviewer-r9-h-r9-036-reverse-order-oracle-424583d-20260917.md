# Independent bounded review — H-R9-036 reverse-order oracle gap

Date: 2026-09-17

## Classification

**HIGH — evidence/oracle correctness.** This does not currently prove a new Session/Carrier/Recovery state-machine defect. It proves that the reachable handoff overstates what the dedicated reverse-order built-binary regression actually asserts.

## Exact anchor reviewed

- developer source/test SHA: `424583dd32fc4bbe3bd69d616ceb40de1a6b45a6`
- implementation owner: `crates/neko-cli/src/main.rs`
- test owner: `crates/neko-cli/tests/probe.rs::reliable_udp_post_return_reversed_ack_order_settles`
- applicable evidence boundary: `docs/specs/nekomusume-session-v0.md` keeps Session delivery and Carrier packet feedback separate evidence domains; current R9 handoff requires exact transition identity rather than diagnostic coincidence.

The preceding `83d10b06279132d79e72240b02bf7b5f7a181d93` introduced the Carrier-before-Session seam/test. `424583d` then removed the accidentally duplicated ordinary-path Carrier ACK send, preserving ordinary Session-before-Carrier ordering and a single reverse-seam Carrier-before-Session send.

## Invariant challenged

For the reverse post-return ACK-order arm, a green process test must prove that settlement is caused by the same exact post-return packet's real Carrier retirement plus the exact Session DeliveryAck transition, independent of arrival order. A diagnostic with the right event name is not by itself sufficient evidence.

At minimum the dedicated reverse-order regression must establish:

1. both processes succeed;
2. exactly one client post-return send, exactly one server Carrier ACK for that packet, and exactly one client positive Carrier retirement;
3. three-way equality of their `packet_number` values;
4. exactly one Session transition with `stream=1`, `offset=48`, `len=16`;
5. zero post-return rejected and accepted-empty Carrier classifications;
6. exactly one `r9_udp_post_return_settled` with `remaining_in_flight=0`;
7. Carrier retirement precedes Session transition in this arm, and settlement is strictly after both transitions.

## Finding

The exact-current test proves only a subset of that contract. It currently:

- requires client/server process success;
- compares the first `r9_udp_return_packet_ack` position with the first `r9_udp_return_delivery_ack` position;
- requires exactly one settlement marker and orders settlement after those first two observations;
- excludes rejected/accepted-empty post-return classifications.

It **does not** assert exactly-one cardinality for the positive Carrier retirement or Session transition, does not parse/bind the client send/server ACK/client retirement packet numbers, and does not assert `stream=1 offset=48 len=16` on the Session transition. Therefore a duplicate or wrong-identity positive diagnostic could satisfy the present reverse-order test while violating the evidence claim recorded in `docs/CHATGPT_HANDOFF.md`.

The current runtime source still emits positive retirement only from the actual recovery outcome and `424583d` removed the known duplicate ordinary-path send, so this review does not claim such a runtime contradiction exists. The missing discriminating assertions are nevertheless release-item-4 evidence work and must be closed before treating READY_LOCAL 2 as accepted exact proof.

## Smallest repair contract

Prefer a test-only repair first. Reuse the already-existing packet-number parsing/binding pattern from the positive P2 fixture; do not add runtime instrumentation unless current diagnostics genuinely cannot identify the three owners.

Strengthen `reliable_udp_post_return_reversed_ack_order_settles` so it proves all seven conditions above. If the stronger oracle exposes a real runtime contradiction, then make the smallest source repair and retain a regression that would have failed before it.

No ACK/wire/crypto architecture change, deadline change, policy value, capacity value, or live experiment is authorized by this finding.

## Evidence/provenance boundary

Developer-local provenance already recorded for exact `424583d` remains valid for that tree: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean exact pushed tree, Linux x86_64, rustc 1.98.0, UTC 2026-09-17T01:59:34Z–02:03:04Z. Reviewer did not rerun that local gate.

GitHub-hosted checks for exact `424583d` are separately green (`stable checks`, `nightly decode fuzz smoke`, Actions run 35172632662). They are cross-evidence only and do not repair the missing oracle assertions.

No decoder/parser/crypto framing change is part of this review, so no new fuzz requirement is introduced by the smallest repair.

## Exclusions

Not reviewed here: cryptanalysis, WAN behavior, HY2 comparison, performance, D019, retained-state capacity policy, signing/key custody/SBOM/publication, frozen-release policy, destructive migration, or RC/freeze/release/production authority.

## Outcome

H-R9-036 is open. Reopen READY_LOCAL 2 narrowly for the exact reverse-order oracle repair and place it ahead of the two P4 single-domain suppression negatives. Repository-wide queue exhaustion remains false. `READY_LIVE` remains `none`.