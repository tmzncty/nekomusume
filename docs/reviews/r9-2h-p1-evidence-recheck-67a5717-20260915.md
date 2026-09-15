# R9-2H P1 exact-evidence recheck — `67a5717`

**Reviewed source/test SHA:** `67a57173a5c904f138cf0d920d5a71d426d50bc5`

**Scope:** bounded review of the built-process reversed Session DeliveryAck regression only. This is not a release/security approval and does not close R9-2H P2-P4.

## Verdict

`67a5717` is useful progress but **P1 remains PARTIAL**.

Accepted new evidence:

- exactly one `r9_udp_delivery_ack_buffered` event is required;
- that event is pinned to `offset=16` and `watermark=0`;
- `r9_udp_delivery_ack_covered` is forbidden;
- exactly two applied `r9_udp_delivery_ack_validated` events remain pinned to `stream=1, offset=0` followed by `stream=1, offset=16, buffered=true`;
- the terminal settlement assertion is now tied to the structured `r9_udp_in_flight_settled` event with `remaining_in_flight=0` rather than a broad substring.

Remaining acceptance gaps in the same regression:

1. the test still does **not** assert `out.status.success()` for the client process;
2. the explicit order proof still uses generic `find("\"offset\":0")` / `rfind("\"buffered\":true")` positions instead of the exact buffered/applied structured event lines already collected, so unrelated diagnostics can satisfy the positional proof;
3. it does not explicitly prove the existing pre-settlement boundary (`outstanding + pending_acks` empty before Carrier-only settlement) from an unambiguous structured event/invariant boundary.

These are evidence/test gaps, not a new protocol-architecture finding. The smallest repair is to strengthen this one built-binary regression only; do not change Session/Carrier semantics.

## CI truth

GitHub-hosted `stable checks` and `nightly decode fuzz smoke` both succeeded on exact `67a5717`. Hosted checks are supplementary evidence only. No final R9-2H developer-local clean exact-tree provenance is accepted yet.

## Queue implication

Finish P1 in one small test-only slice, then continue immediately through P2, P3, P4 and the exact-tree local gate. Do not wait for another reviewer pass between these slices. The deeper R9-3..R9-12 and Q10/Q11/Q12 queue remains valid.
