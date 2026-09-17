# Independent bounded review — H-R9-045 order/settlement closure at `0766dd8`

Date: 2026-09-18

Reviewed developer source/test anchor: exact `0766dd8559d44fb7aa4f43764f794f560c3a8c70`.

Repository head at review start: docs-only descendant `cb66c0a8811c22ac87022e297a13b11c6c8b695a`.

Scope: exact-current built-binary fixture `reliable_udp_post_return_data_loss_recovers_via_pto_retransmit`, the prior H-R9-045 contract in `docs/reviews/reviewer-r9-3-h-r9-045-oracle-20260918.md`, and current hosted/local provenance claims. This review makes no live/WAN, release, or production claim.

## Accepted progress

`0766dd8` correctly repairs the packet-identity half of H-R9-045:

- every observed retransmit packet number is non-sentinel, differs from the original packet number, and is pairwise distinct from sibling retransmits;
- every retransmit carries stable `frame=48`;
- every positive Carrier retirement packet number belongs to the client retransmit set;
- every positive Carrier retirement packet number has a matching server `udp_return_packet_ack_sent` packet number;
- packet identity is therefore matched by value, not by `last == last == last` log position.

The exact `0766dd8` GitHub-hosted Rust CI run `35272921609` completed successfully; both `stable checks` and `nightly decode fuzz smoke` are green. Hosted CI remains cross-evidence only. The handoff also records a developer-local clean exact-tree gate for exact `0766dd8`; this review does not re-label hosted evidence as local provenance.

## H-R9-045 remains OPEN — order/terminal half was not implemented

**Severity: HIGH evidence/oracle correctness blocker.** This remains an oracle/evidence defect unless the strengthened assertions expose a runtime contradiction.

The prior accepted H-R9-045 repair contract explicitly required two additional invariants that exact `0766dd8` still does not assert:

1. **No post-retirement retransmit proof is absent.** The fixture collects retransmit events and positive Carrier retirement events, but never compares their positions. A run can therefore still pass if `r9_udp_retransmit_sent` appears after the lifecycle-resolving positive `r9_udp_return_packet_ack`.
2. **Final settlement ordering is absent.** The fixture requires exactly one `r9_udp_post_return_settled` with `remaining_in_flight=0`, but never proves that this settlement occurs after both the exact `stream=1 offset=48 len=16` Session confirmation and the lifecycle-resolving positive Carrier retirement.

These are not new requirements: they were items 5 and 6 of the reachable H-R9-045 review contract at `7f66bfc98c8d02f569547e702357b2f10bc2b66e`. The handoff currently marks H-R9-045 fully closed, so repository evidence truth is stale.

## Smallest repair / challenge

Keep current Recovery/Session architecture and repair the process oracle first.

1. Identify the **first lifecycle-resolving positive Carrier retirement event** in client-log order. Its packet number must already satisfy the value-binding checks added by `0766dd8`.
2. Prove that no `r9_udp_retransmit_sent` occurs after that positive retirement event.
3. Keep exactly one exact Session transition for `stream=1 offset=48 len=16`; accepted-empty sibling/late Carrier ACKs remain legal and classification-only; typed Carrier rejection remains zero.
4. Require exactly one `r9_udp_post_return_settled` with `remaining_in_flight=0` and prove its client-log position is strictly after both the exact Session transition and the lifecycle-resolving positive Carrier retirement.
5. Both processes must succeed.

If these assertions expose a real runtime contradiction, switch immediately to the smallest owner repair plus deterministic regression. Otherwise this is test-only closure.

H-R9-041 exact-wire refusal regression and H-R9-042 deterministic pre-deadline negative remain independently open after the remaining H-R9-045 order/terminal assertions close.

## Queue / evidence boundary

`READY_LIVE` remains `none`; this is local correctness/evidence work and creates no new real-network hypothesis. Release items 3 and 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.

Repository-wide queue exhaustion is false. Preserve the downstream R9-4 through R9-12 lanes, dedicated independent R9 review, and Q10/Q11/Q12 factual reconciliation.
