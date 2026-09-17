# Independent bounded review — H-R9-046 post-retirement retransmit oracle

**Classification:** HIGH — evidence/oracle correctness

**Reviewed repository anchor:** source/test exact `68e364a45466ce958892d536aeea450678919401`; current `main` before this note was docs-only descendant `a39a93fd895a3f548f12b9216f3476c974344e95`.

## Scope

This bounded review challenged the exact-current R9-3 built-binary oracle in `crates/neko-cli/tests/probe.rs::reliable_udp_post_return_data_loss_recovers_via_pto_retransmit`, specifically the H-R9-045 claim that the first lifecycle-resolving positive Carrier retirement terminates the frame retransmit lifecycle and that final settlement follows both real evidence domains.

Inspected owners:

- `crates/neko-cli/tests/probe.rs`, including the complete post-return data-loss fixture around the PTO/retransmit/Carrier-ACK/Session-DeliveryAck/settlement assertions;
- `crates/neko-cli/src/main.rs` diagnostic emission for `r9_udp_return_packet_ack` and `udp_return_packet_ack_sent`;
- `docs/CHATGPT_HANDOFF.md` exact-current H-R9-045 closure claim;
- applicable Session/Carrier evidence-separation boundaries in `AGENTS.md`, `docs/carrier-architecture.md`, and `docs/specs/nekomusume-session-v0.md`.

Cross-evidence checked: GitHub-hosted Rust CI run `35278461883` for exact `68e364a` completed successfully. Hosted CI is cross-evidence only and is not developer-local provenance.

## Finding

The H-R9-045 closure claim is stronger than the test that currently passes.

Exact `68e364a` adds two ordering assertions:

- the single `r9_udp_post_return_settled` line occurs after the exact Session `r9_udp_return_delivery_ack` transition;
- the same settlement line occurs after the first positive `r9_udp_return_packet_ack` whose packet number belongs to the retransmit set.

The fixture already proves retransmit packet-number freshness/distinctness, stable `frame=48`, value-based positive-retirement membership, matching server Carrier ACK by packet-number value, exactly-one Session transition, and exactly-one zero-in-flight settlement.

However, it does **not** assert that no `r9_udp_retransmit_sent` occurs after the lifecycle-resolving positive Carrier retirement. A trace with one correct positive retirement, then an additional retransmit, then the exact Session confirmation and final zero-in-flight settlement can still satisfy all current assertions. That directly leaves the final H-R9-045 lifecycle claim unproven even though `docs/CHATGPT_HANDOFF.md` currently says it is closed.

This is currently an evidence/oracle blocker, not a demonstrated Recovery/Session implementation defect. The existing source intent (`acked_frames` lifecycle suppression after positive frame acknowledgement) may already be correct; the process-level oracle simply does not discriminate a forbidden post-retirement retransmit.

## Minimal repair contract

Test/oracle first; do not redesign Recovery, Session, ACK, wire, crypto, or timing policy.

1. In `reliable_udp_post_return_data_loss_recovers_via_pto_retransmit`, identify the lifecycle-resolving positive Carrier retirement by the existing packet-number membership rule.
2. Assert that **every** `r9_udp_retransmit_sent` log position is strictly before that retirement position. Equivalently, after the lifecycle-resolving positive retirement, the count of retransmit diagnostics must remain zero.
3. Preserve all already-valid exact assertions: every retransmit PN fresh/non-sentinel/pairwise-distinct; stable `frame=48`; positive retirement PN belongs to the retransmit set; matching server Carrier ACK by value; exactly-one `stream=1 offset=48 len=16` Session transition; zero typed rejection; accepted-empty remains legal/classification-only; exactly-one `remaining_in_flight=0` settlement after both evidence-domain transitions.
4. If the strengthened assertion exposes a real runtime contradiction, apply the smallest semantics-preserving repair and add the focused regression. Otherwise this remains a test-only repair.
5. After repair, run the normal developer-local clean exact-tree gate and record reachable pushed provenance. No decoder/framing change is implied, so fuzz is not mechanically required by this finding.

## Exclusions

This review does not reopen H-R9-040 lifecycle-marker pruning semantics, H-R9-041 exact encoded-wire admission implementation, H-R9-043 repeated-PTO legality, H-R9-044 accepted-empty legality/per-event deadline checks, or the packet-identity half of H-R9-045. It makes no WAN, performance, RC, freeze, release, or production claim.
