# R9-2H P1 evidence recheck — exact `3d7206a`

**Reviewed tree:** `3d7206a73bf0638b1bdc0cb39bb527ca57b037df`

**Scope:** the built-process reversed Session DeliveryAck regression only. This is an evidence/acceptance review, not a protocol/security/release approval and not a WAN result.

## Verdict

`3d7206a` is useful progress but **P1 is not yet closed**. The new test now requires exactly two `r9_udp_delivery_ack_validated` lines and pins their applied identities/order as `stream=1, offset=0, buffered!=true` followed by `stream=1, offset=16, buffered=true`. Hosted `stable checks` and `nightly decode fuzz smoke` pass on this exact SHA.

The remaining P1 acceptance points from the standing handoff are still not asserted by the built-binary regression:

1. client exit status is not required to be success;
2. the buffered observation is only checked by broad event presence, not exactly one structured `r9_udp_delivery_ack_buffered` with `stream=1, offset=16, watermark=0`;
3. the test does not reject an `r9_udp_delivery_ack_covered` shortcut for these records;
4. it does not prove logical `outstanding + pending_acks` is empty before Carrier settlement begins;
5. `remaining_in_flight=0` is still a broad substring check rather than the final relevant settlement event.

The older broad `offset` ordering assertions also remain, but the new exact applied-event assertions substantially reduce the prior false-positive surface. This is an acceptance-evidence gap, not a newly identified runtime correctness defect.

## Next slice

Finish P1 in the same existing test using exact per-line structured event matching for the five points above; do not change protocol semantics or add a new evidence framework. Then continue directly to P2, P3, P4 and the clean exact-tree developer-local provenance gate already queued in `docs/CHATGPT_HANDOFF.md`.

No decoder/parser/crypto-framing change occurred in this commit, so no new local fuzz obligation is created by this test-only slice. Hosted fuzz remains supplementary evidence.
