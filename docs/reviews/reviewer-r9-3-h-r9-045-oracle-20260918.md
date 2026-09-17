# Independent bounded review — R9-3 repeated-PTO exact-oracle closure at `ebfc620`

Date: 2026-09-18

Reviewed developer source/test anchor: exact `ebfc620344d5abf4799883290c2ceb311079f951` (source/test owner unchanged from `05b1d7b60e755238c8d5ad3bf3a285ba7710e195` except dead local cleanup).

Current repository head at review start: docs-only descendant `8dff0a3d2eca19f869d3d0ae2633599a35427c5d`.

Scope: exact-current built-binary fixture `reliable_udp_post_return_data_loss_recovers_via_pto_retransmit`, prior H-R9-044 repair contract, packet/Session evidence separation, and current hosted cross-evidence. This review makes no live/WAN, release, or production claim.

## Accepted progress

The `05b1d7b` repair correctly closes two parts of the prior H-R9-044 finding:

- every observed `r9_udp_pto_fired` event is parsed individually and must satisfy `fired_at_us >= deadline_us`;
- typed Carrier rejection remains forbidden while sibling/late ACK classification may legitimately be accepted-empty without manufacturing another Session transition.

The current exact-tree hosted Rust CI for `ebfc620` is green (run `35266888219`). That is hosted cross-evidence only; it does not enlarge the oracle proved by the test.

## H-R9-045 — H-R9-044 was closed before the exact repeated-PTO identity/order oracle was complete

**Severity: HIGH evidence/oracle correctness blocker.** This is presently a test/evidence defect, not a newly proven Recovery or Session state-machine defect.

The previous independent H-R9-044 contract explicitly required the repeated-PTO fixture to avoid UDP ordering assumptions and to bind the packet that actually receives positive Carrier retirement to a real client retransmit event and a corresponding server `udp_return_packet_ack_sent` event. The exact-current fixture still does not prove that contract:

1. **It still uses a `last == last == last` identity assumption.** The test selects the last client retransmit, the last positive client retirement, and the last server Carrier-ACK event and asserts equality. Under repeated PTO there may be multiple sibling packet copies and UDP feedback may reorder. Equality of three final log positions is not a proof that the positively retired packet has a matching client retransmit and server ACK; it is exactly the ordering assumption the prior review prohibited unless the fixture deterministically establishes that order.
2. **Fresh/distinct retransmit identity is not proved for every retransmission.** The test checks only that the final retransmit packet number differs from the original. It does not require every retransmit PN to be distinct from the original and from every sibling retransmit.
3. **Stable frame identity is only asserted on the first retransmit.** The accepted contract requires all retransmit copies in this lifecycle to preserve the same frame/logical identity; the exact-current test checks `frame=48` only on `re_ev[0]`.
4. **No-post-retirement retransmit is not proved.** The prior contract requires no new `r9_udp_retransmit_sent` after the first positive Carrier retirement resolving the frame lifecycle. The exact-current fixture does not compare event positions for that invariant.
5. **Final settlement ordering is incomplete.** It proves exactly one `remaining_in_flight=0` settlement, but does not require that settlement to occur after the exact Session confirmation and after the positive Carrier retirement that actually resolves Recovery.

A green run can therefore still pass while the test has not identified the actual retired sibling copy under reordered feedback or while another retransmit occurs after lifecycle resolution. This is a false closure of an evidence gate, not proof that the runtime is wrong.

## Smallest repair / challenge

Keep current Recovery/Session architecture and repair the process oracle first.

1. Parse every `r9_udp_retransmit_sent`; require each packet number to be non-sentinel, fresh relative to the original, and pairwise distinct; require every retransmit to carry the expected stable frame/logical identity (`frame=48`).
2. Parse all positive `r9_udp_return_packet_ack` events. Identify the first event that positively retires the frame lifecycle; extract its `packet_number`.
3. Prove that exact retired packet number appears in the client retransmit set and in the server `udp_return_packet_ack_sent` set. Compare by packet-number membership, not by final log position. Accepted-empty sibling ACKs remain legal and classification-only.
4. Require exactly one exact Session transition for `stream=1 offset=48 len=16` and zero typed Carrier rejection.
5. Prove no `r9_udp_retransmit_sent` event occurs after the lifecycle-resolving positive Carrier retirement.
6. Require exactly one final settlement with `remaining_in_flight=0`, and prove it occurs after both the exact Session confirmation and lifecycle-resolving positive Carrier retirement.
7. Both processes must succeed.

If this strengthened oracle exposes a real runtime contradiction, switch immediately to the smallest owner repair plus deterministic regression. Otherwise this should remain a test-only correction.

H-R9-041 exact-wire refusal regression and H-R9-042 deterministic pre-deadline negative remain independently open after H-R9-045.

## Queue / evidence boundary

`READY_LIVE` remains `none`; this is a local correctness/evidence question and creates no new real-network hypothesis. Release item 3 and item 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.

Repository-wide queue exhaustion is false. Preserve the downstream R9-4 through R9-12 lanes, dedicated independent R9 review, and Q10/Q11/Q12 factual reconciliation after H-R9-045/H-R9-041/H-R9-042 close.