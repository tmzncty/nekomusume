# R9-2H terminal-settlement recheck — exact `54da45f`

**Reviewed source/test revision:** `54da45fcb5e818c3137ded0f2460565f72c4b0af` (`fix(cli): R9-2 incomplete settlement is terminal, not fallthrough (H-R9-010)`).

**Review scope:** the H-R9-010B incomplete-settlement control-flow finding and the remaining R9-2H closure evidence. This is a bounded reviewer recheck, not release/security approval, WAN evidence, RC, freeze, or production authorization.

## Verdict

### H-R9-010B — CLOSED

`failover_client` now emits `r9_udp_settlement_incomplete` when `ReliableUdpRuntime::in_flight()` remains nonzero and immediately calls `fail("r9 reliable-UDP settlement incomplete")`. `CarrierHealthEvidence`, `FailoverController`, `CarrierManager`, uncertain-range handling and later fallback work are below this terminal branch and therefore cannot consume an incomplete settlement as a successful premise.

The new built-binary process regression `reliable_udp_incomplete_settlement_fails_not_settled` runs a real `failover-server`/`failover-client` pair with `--reliable-udp`. The server-side `--suppress-r9-ack` fault seam withholds Carrier packet ACK while still allowing Session DeliveryAck, forcing remaining in-flight state. The client is required to exit nonzero, emit an incomplete marker, and not emit `r9_udp_in_flight_settled`.

The implementation does not change wire grammar, Session/Carrier layering, crypto semantics, ACK semantics, or policy values.

### M-R9-009 — remains closed

Settlement still reuses the existing `application_deadline`; no fresh post-confirmation receive budget was reintroduced.

## Exact-tree checks visible to reviewer

GitHub-hosted checks on exact `54da45f`:

- `stable checks` — success.
- `nightly decode fuzz smoke` — success.

These are supplementary cross-evidence only. There is **no later developer provenance commit yet** recording the required developer-local clean exact-tree `scripts/check.sh`, `git diff --check`, UTC timing, OS/arch, Rust stable, and clean initial/final tree for exact `54da45f`.

## R9-2H closure still open — evidence/test completeness, not a new source defect

The prior handoff required four built-binary discriminating regressions. Exact `54da45f` adds the fourth (incomplete settlement). Since the only source/test change after exact `6ca11a0` is `54da45f`, the other three remain outstanding:

1. **Reversed logical ACK order:** two reliable-owned logical records; server deliberately emits Session DeliveryAck for record 1 before record 0; both exact ranges must confirm once, Carrier ACKs remain Carrier-local, Recovery settles to zero.
2. **Reliable + migration-back reservation:** reserved final logical record is neither Recovery-tracked nor legacy wire-sent before post-promotion return authorization; only its later explicit owner sends it.
3. **Persistent malformed budget across Carrier feedback:** authenticated malformed #1 -> malformed #2 -> canonical Carrier ACK -> malformed #3 must hit the same operation-wide `MAX_POST_HANDSHAKE_MALFORMED` ceiling; feedback must not reset the budget.

Focused/helper tests are useful but do not replace the required built-binary process path.

## Next gate

R9-2H closes on one reachable pushed source/test tree that contains those three remaining process regressions plus the already accepted incomplete-settlement regression, followed by developer-local clean exact-tree provenance. If the full gate is green, proceed immediately to R9-3 without waiting for reviewer cadence.

No new BLOCKER/HIGH was found in this recheck. `READY_LIVE` remains false until the later R9 cross-process recovery/promotion chain and independent review create a specific changed live question.
