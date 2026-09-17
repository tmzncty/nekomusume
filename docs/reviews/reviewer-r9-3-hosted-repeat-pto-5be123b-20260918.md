# Independent bounded review — R9-3 repeated-PTO oracle at exact `5be123b`

Date: 2026-09-18

Reviewed developer source/test anchor: exact `5be123b8aee40a8487e995927dde23283f97b20d`.

Current repository head at review start was the docs-only descendant `1d90f82e2b2bebf53d3604ae6b5ee6fafdd908d5`; no source/test owner changed after `5be123b`.

Scope: H-R9-043 repeated-PTO process oracle, the exact-current built-binary fixture `reliable_udp_post_return_data_loss_recovers_via_pto_retransmit`, the accepted packet-vs-Session evidence separation, and GitHub-hosted exact-tree cross-evidence. This review adds no live/WAN or release conclusion.

## Accepted semantic conclusion that remains closed

The semantic correction from H-R9-043 remains valid: while a retransmitted frame still has Carrier recovery copies outstanding and no positive Carrier retirement has resolved the lifecycle, another authoritative PTO may legitimately become due. Session `DeliveryAck` is a separate evidence domain and must not be used to suppress Carrier PTO/recovery merely because logical Session bytes were confirmed.

Do not repair the current failure by merging Session and Carrier ACK semantics, suppressing a legal second PTO, or changing core recovery architecture.

## H-R9-044 — repeated-PTO sibling-ACK oracle contradicts the accepted semantics

**Severity: HIGH evidence/correctness gate blocker.** This is presently an oracle/test defect, not a newly proven Recovery state-machine defect.

The exact `5be123b` fixture was changed from `exactly one PTO/retransmit` to `>=1`, but it still asserts that the client log contains **zero** `r9_udp_return_packet_ack_accepted_empty` events. That assertion contradicts the already accepted H-R9-043 overlap-copy semantics: once repeated PTO has created sibling packet copies for the same stable frame, a late sibling ACK may legitimately classify as accepted-empty after another copy resolves the relevant lifecycle. Accepted-empty is a typed classification-only outcome; it is not a rejection and must not manufacture Session delivery.

The current test also comments that **all** PTO events fired at/after their own authoritative deadlines, but parses and checks only `pto_ev[0]`. Under the newly accepted repeated-PTO path, this is not an acceptance-grade proof for later PTO probes.

## Hosted exact-tree evidence

GitHub-hosted Rust CI run `35260878111` for exact source/test tree `5be123b` failed `stable checks -> bash scripts/check.sh`; nightly decode fuzz smoke passed. The failing test was exactly `reliable_udp_post_return_data_loss_recovers_via_pto_retransmit`.

Its trace showed a legal repeated-PTO sequence:

1. original post-return packet 4 was recovery-owned and deliberately dropped;
2. PTO #1 fired after its deadline and sent fresh packet 5 for stable frame 48;
3. exact Session DeliveryAck for stream 1 / offset 48 / len 16 arrived;
4. before positive Carrier retirement, PTO #2 fired after its own deadline and sent fresh packet 6 for the same frame;
5. one sibling Carrier ACK classified accepted-empty;
6. packet 6 received positive Carrier retirement;
7. settlement reached `remaining_in_flight=0` and the process otherwise completed.

The docs-only descendant `1d90f82` reran the unchanged source/test tree in hosted run `35262066403` and failed the same fixture with the same two-PTO / accepted-empty / final-zero pattern. Therefore the current source/test closure is not merely waiting for a hosted rerun: the exact-current oracle deterministically rejects a behavior the reviewer contract explicitly permits on that hosted environment.

The developer-local clean exact-tree gate recorded for `5be123b` remains truthful local provenance for that run, but it cannot be promoted into acceptance closure while the same exact source/test owner has reproducible hosted failures exposing an environment/scheduling-sensitive contradictory assertion. Hosted CI is still cross-evidence, not a replacement for local provenance.

## Smallest next repair / challenge

Keep current Recovery/Session semantics and repair the process oracle first.

1. Preserve `>=1` bounded PTO/retransmit before the first positive Carrier retirement.
2. Parse **every** `r9_udp_pto_fired` event and require each event's `fired_at_us >= deadline_us`; do not claim all deadlines from the first event only.
3. Require every retransmit packet number to be fresh/distinct from the original and from sibling retransmits, while preserving stable frame/logical identity (`frame=48`, Session range stream 1 / offset 48 / len 16).
4. Require exactly one Session delivery transition for that logical range.
5. Keep typed Carrier rejection at zero. Do **not** require accepted-empty to be zero: a sibling/late ACK may classify accepted-empty. If present it must remain classification-only and must not create a second Session transition, a rejection, or a new recovery ownership transition.
6. Identify the packet copy that actually receives positive Carrier retirement. Bind that packet number to a real client retransmit event and a corresponding server `udp_return_packet_ack_sent` event. Do not assume `last retransmit == last server ACK == last client retirement` under UDP reordering unless the fixture itself deterministically establishes that order.
7. Prove no retransmit event occurs after the first positive Carrier retirement that resolves the frame lifecycle.
8. Require exactly one final settlement with `remaining_in_flight=0`, after exact Session confirmation and Recovery zero.
9. Both client and server processes must succeed.

If strengthening the oracle exposes a real Recovery contradiction, switch immediately to the smallest owner repair + positive/negative regression. Otherwise this should remain a test/oracle-only correction.

After H-R9-044 closes, the previously open H-R9-041 exact-wire congestion-refusal regression and H-R9-042 deterministic just-before-PTO-deadline negative are still required before R9-3 can be declared complete. Then run a new developer-local clean exact-tree gate on the final pushed source/test SHA and persist provenance.

## Evidence and queue boundary

`READY_LIVE` remains `none`. This finding is local correctness/evidence work and creates no new real-network question. Release item 3 and item 4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.

Repository-wide queue exhaustion is false. Preserve the downstream R9-4 through R9-12 lanes, dedicated independent R9 review, and Q10/Q11/Q12 factual reconciliation after the H-R9-044/H-R9-041/H-R9-042 dependency chain closes.
