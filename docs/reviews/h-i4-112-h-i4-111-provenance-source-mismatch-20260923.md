# H-I4-112 — H-I4-111 provenance/source mismatch (HIGH)

**Reviewer anchor:** exact current `main` before this note was `54c5aa5a1edf0a236360d21c7c42fb75767b24d9`; developer source under review is `1d727f065436fbe15ff9151e48a746da9fb2383b`.

**Classification:** release/item-4 test-harness correctness + evidence truthfulness HIGH. This is not a production Session/Carrier/ACK/crypto/wire finding.

## Finding

H-I4-111 was marked CLOSED by `54c5aa5` and `docs/notes/h-i4-111-provenance-1d727f0-20260923.md` states that both real clients in
`expired_preprogress_udp_session_is_retired_before_delivery_and_fresh_handshake_recovers` use `bounded_client_output` (`--duration 2 -> 7s`, `--duration 3 -> 8s`). Exact developer source at `1d727f0`, and therefore exact current source at `54c5aa5`, falsifies that statement.

The function still contains both synchronous owners:

```rust
let expired = Command::new(bin)
    .args([ /* failover-client, --duration 2, delayed first data */ ])
    .output()
    .unwrap();

let recovered = Command::new(bin)
    .args([ /* failover-client, --duration 3 */ ])
    .output()
    .unwrap();
```

The server is already live and `ready_failover_server(server)` has returned before these clients run; `finish_server(server)` is only reachable after both synchronous `.output()` calls return. A client lifecycle/protocol regression can therefore still strand the gate indefinitely before bounded server cleanup is reachable. Product `--duration` remains behavior under test, not an independent harness deadline.

The same `1d727f0` commit *did* correctly bound `first_udp_selection_loss_recovers_from_same_peer_duplicate_hello`, and it also bounded the client/server owners in `warm_readiness_failures_close_before_admission_or_application_data`. Those source repairs remain accepted absent falsification. The defect is the missed `expired_preprogress` pair plus the resulting false closure/provenance claim.

## Evidence-truth boundary

`docs/notes/h-i4-111-provenance-1d727f0-20260923.md` records a developer-local clean exact-tree gate (`scripts/check.sh`, `git diff --check`, clean tree, Linux x86_64, stable rustc 1.98.0). That gate record may be genuine as an execution record, but it does **not** prove the source claim that the two `expired_preprogress` clients were converted: exact source is authoritative and shows they were not. GitHub combined status for exact `1d727f0` exposes no hosted status entries. Reviewer has not executed the Rust/full gate locally in this pass.

Do not delete or silently rewrite historical evidence. Supersede/correct the false source-coverage statement with a reachable erratum or new final provenance after the repair.

## Closure contract

1. In `expired_preprogress_udp_session_is_retired_before_delivery_and_fresh_handshake_recovers`, route the intentionally expired real client through exact-current `bounded_client_output(..., Duration::from_secs(7))` (`--duration 2 + 5s`). Preserve `--test-first-data-delay-ms 1200`, the expected client failure, and the assertion that no `udp_delivery_ack_validated` appears.
2. Route the subsequent fresh recovery real client through `bounded_client_output(..., Duration::from_secs(8))` (`--duration 3 + 5s`). Preserve every existing recovery, ACK-count, ordering, ResumeGuard and `failover_*_ok` assertion.
3. Do not create another process framework and do not mechanically convert local/fail-fast `.output()` calls such as keygen, help/capabilities, invalid-argument/configuration, or socket-free fixtures. Continue owner-by-owner causal classification after this repair.
4. Existing generic `bounded_client_output_fails_when_client_never_exits` is sufficient for the ordinary shared deadline path unless this exact owner exposes a distinct failure mode; do not add duplicate sleep-only churn.
5. No decoder/parser/crypto-framing implementation change is implicated; do not run fuzz mechanically.
6. On the final pushed source/test SHA, run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist a new reachable developer-local exact-tree provenance record with exact SHA, UTC start/end, exit codes, OS/arch, stable Rust and clean-tree state.
7. Explicitly supersede/correct the H-I4-111 provenance sentence that falsely claims these two clients were already bounded at `1d727f0`. Keep the original execution record distinguishable from the corrected source-coverage claim.
8. After closure, continue the existing remaining process/socket/thread ownership sweep and rolling queue immediately; do not infer repository-wide exhaustion from this repair.

## Queue effect

H-I4-112 is FRONT HIGH. H-I4-111 is only **partially closed** until this repair and corrected provenance land. The deeper queue remains unchanged: remaining process/socket/thread ownership causal sweep -> cross-platform reconciliation -> resource boundedness -> release packet/item-4 factual reconciliation -> pre-auth reuse -> CLI exit/JSON/human-output contract -> package/build -> core-runtime spot challenge -> observability/adapters/build reuse -> repository-wide 13-surface refill -> conditional live.

`READY_LIVE: none`; release items 3 and 4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
