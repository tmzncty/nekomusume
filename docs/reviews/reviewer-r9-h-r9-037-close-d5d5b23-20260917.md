# Independent bounded review — H-R9-037 reverse-order exact packet binding

Date: 2026-09-17

## Exact anchor

- Developer source/test anchor: `d5d5b23dd007c4293e9df7e0eb9aa2bee828bc74`.
- Reachability at review start: `d5d5b23` is the parent of reachable `main` handoff commit `64e035183c15a4bb6067d52970eb2177ff411462`.
- Scope is the H-R9-037 reverse post-return ACK-order process oracle only. This note is release-item-4 support, not release/security approval.

## Inspected owners

- `crates/neko-cli/tests/probe.rs::reliable_udp_post_return_reversed_ack_order_settles` at exact `d5d5b23`.
- Exact-current server diagnostic owner in `crates/neko-cli/src/main.rs` for `udp_return_packet_ack_sent`.
- Exact-current client diagnostic owner for `r9_udp_post_return_sent`, `r9_udp_return_packet_ack`, `r9_udp_return_delivery_ack`, and `r9_udp_post_return_settled`.
- Applicable evidence boundary: Session DeliveryAck and Carrier packet ACK are independent evidence domains; packet feedback does not substitute for Session logical delivery.

## Challenge

Attempt to falsify the handoff claim that the reverse-order regression proves one real post-return packet identity across the sender, server Carrier ACK, and client Recovery retirement, while independently proving the Session transition and final zero-in-flight settlement.

The exact test now requires:

1. client and server processes both succeed;
2. exactly one positive `r9_udp_return_packet_ack` with `applied=true` and `retired=true`;
3. exactly one `r9_udp_return_delivery_ack` carrying `stream=1`, `offset=48`, `len=16`;
4. exactly one client `r9_udp_post_return_sent`;
5. exactly one server `udp_return_packet_ack_sent`;
6. client-send packet number equals server-ACK packet number;
7. that same packet number equals the positive client retirement packet number;
8. exactly one `r9_udp_post_return_settled` containing `remaining_in_flight=0`;
9. Carrier retirement precedes Session transition and both precede settlement;
10. zero rejected and zero accepted-empty post-return Carrier classifications on this ordinary reverse-order path.

The runtime diagnostic emitters place `packet_number` as the terminal numeric field of the relevant diagnostic payloads, so the test's split/trim/parse expression is discriminating for these exact events rather than comparing a fallback value produced by unrelated trailing fields.

## Result

**No finding in this bounded seam. H-R9-037 may remain closed at exact `d5d5b23`.**

The strengthened oracle addresses the prior evidence defect: a missing/duplicate/wrong-identity server ACK or a nonzero settlement can no longer satisfy this fixture merely because client-side transition events exist.

## Verification/provenance boundary

- Developer-local provenance is repository-recorded separately for exact `d5d5b23`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, Linux x86_64, rustc 1.98.0, 2026-09-17T03:49:17Z through 2026-09-17T03:53:01Z. The reviewer did not re-execute that local gate and does not relabel it as reviewer-executed evidence.
- GitHub-hosted Rust CI run `35179469348` for exact `d5d5b23` completed successfully. `stable checks` ran `bash scripts/check.sh`; `nightly decode fuzz smoke` also completed successfully. Hosted CI remains cross-evidence and is not a substitute for developer-local provenance.
- No decoder/parser/crypto-framing source changed in H-R9-037; the hosted fuzz job is incidental workflow cross-evidence, not a reviewer requirement for this test-only change.

## Exclusions

This review does not independently re-review the full failover state machine, Recovery algorithms, SessionRuntime, crypto, wire parser, P4 single-domain negative coverage, WAN behavior, package/release policy, D019, capacity values, or release authority. The next dependency-ready work remains the two P4 single-domain suppression negatives followed by final R9-2 provenance and the remaining R9 queue.
