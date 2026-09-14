# R9-2H settlement recheck — exact `6ca11a0`

**Reviewed source/test revision:** `6ca11a0c505d7634c487520b96e37d3a358309dd` (`fix(cli): R9-2 settlement typed outcome + single deadline`).

**Developer provenance descendant:** `047bf8e6cdddba207755d98e952740329b02d362` records a clean developer-local exact-tree gate for `6ca11a0` (`scripts/check.sh` exit 0, `git diff --check` exit 0, clean initial/final tree, Linux x86_64, rustc 1.98.0). Hosted `stable checks` and `nightly decode fuzz smoke` are also green on the provenance descendant. They remain supplementary evidence.

## Verdict

**H-R9-010 remains OPEN. M-R9-009 is CLOSED. R9-2H is not accepted and R9-3 remains blocked.**

The patch correctly removes the fresh settlement deadline and correctly stops emitting `r9_udp_in_flight_settled` when `ReliableUdpRuntime::in_flight()` is nonzero. However, the incomplete branch only changes the diagnostic name. Control flow then falls through into health/failover setup and uncertain-range/fallback processing exactly as the successful branch does. That violates the previously committed closure contract: incomplete Carrier settlement must terminate the R9 reliable operation and must not become a successful-settlement premise for downstream health/failover work.

## Finding H-R9-010 — typed incomplete diagnostic without typed incomplete control flow

Current source after the settlement loop does:

```text
if rt.in_flight() == 0:
    emit r9_udp_in_flight_settled
else:
    emit r9_udp_settlement_incomplete

# unconditional fallthrough
create CarrierHealthEvidence
create FailoverController / CarrierManager
track/send uncertain ranges
prepare warm TCP / later failover logic
```

The observable diagnostic is more truthful than before, but the operation itself is still treated as if it may continue after incomplete settlement. A timeout, receive error, or malformed-budget exhaustion can therefore leave Carrier-owned reliable packets unresolved and still enter downstream health/failover code.

### Required repair

Do not add a new policy value or wire type. The smallest acceptable shape is one of:

- make incomplete settlement immediately return/fail from this bounded failover-client operation after emitting the typed negative; or
- make the reliable receive owner return an explicit `Complete` / `Incomplete(reason, remaining_in_flight)` result and gate all downstream health/failover work on `Complete`.

The success predicate remains:

```text
outstanding.is_empty() && rt.in_flight() == 0
```

No path with nonzero in-flight state may emit `r9_udp_in_flight_settled`, initialize the downstream R9 health/failover sequence as a success continuation, or claim R9-2H closure.

## M-R9-009 — CLOSED

`settle_deadline` now aliases the pre-existing absolute `application_deadline`; settlement receives no fresh budget after logical confirmation. This matches the required single-deadline contract.

## Missing closure evidence — still required

Exact `6ca11a0` changes only `crates/neko-cli/src/main.rs`; it adds no discriminating built-binary/process regressions. The prior handoff's three process-level closure tests therefore remain required before R9-2H acceptance:

1. reversed Session DeliveryAck order across two reliable-owned logical records;
2. `--reliable-udp + migration-back` reserved final record is not tracked or wire-sent before explicit post-promotion ownership;
3. persistent malformed budget across authenticated malformed -> malformed -> Carrier ACK -> malformed reaches the same operation-wide bound.

Add one explicit incomplete-settlement process regression as well: force reliable Carrier settlement to end with `remaining_in_flight > 0`; require `r9_udp_settlement_incomplete`, forbid `r9_udp_in_flight_settled`, and prove no downstream successful health/failover continuation occurs.

## Gate

After the smallest repair and process regressions: focused tests -> `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` -> `git diff --check` -> clean exact pushed-tree provenance. If all pass, continue directly to R9-3 without waiting for reviewer cadence.

No WAN/VPS run is authorized by this review result itself; `READY_LIVE: none` remains until the cross-process R9 chain earns a specific changed live question.
