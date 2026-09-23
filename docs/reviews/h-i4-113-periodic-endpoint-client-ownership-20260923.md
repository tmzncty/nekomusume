# H-I4-113 — periodic / endpoint-rebind real client owners remain unbounded

**Severity:** HIGH — release/item-4 test-harness correctness and evidence reliability

**Reviewer anchor:** exact current `main` inspected at `453f90e5d2c833cd926bfde198855a34e9b59a78` after accepting the developer-owned H-I4-112 source repair `fc23a7a88c917e116f1140333faa6d8f441ca557` and its reachable local provenance/erratum.

## Inspected owners and invariant

Inspected exact-current `crates/neko-cli/tests/probe.rs` process ownership around the remaining networked process tests after H-I4-107..112.

Invariant challenged: when a product child participates in real local socket/session behavior, a regression in connect/negotiation/Noise/auth/DeliveryAck/lifecycle/shutdown must become bounded negative evidence. A product `--duration` is behavior under test and cannot substitute for an independent harness wall-clock deadline. The server-side `finish_server` bound is not reachable while the test thread is blocked in a synchronous client `Command::output()`.

## Concrete defect

The following exact-current tests start a real server, then run a real network client through synchronous `.output().unwrap()`, and only after that client returns call `finish_server(server)`:

### Periodic-session family

1. `periodic_session_delayed_confirmations_are_counted_on_one_session` — `periodic-client --duration 5`.
2. `periodic_session_synchronized_key_update_crosses_authenticated_socket` — `--duration 5`.
3. `periodic_session_mismatched_key_update_schedule_fails_closed` — `--duration 5`.
4. `periodic_session_accounts_missing_ack_and_fails_closed` — `--duration 2`.
5. `periodic_session_duplicate_ack_is_authenticated_and_idempotent` — `--duration 5`.
6. `periodic_setup_timeout_is_separate_from_ack_timeout` — `--duration 5`.
7. `periodic_setup_timeout_fails_before_application_records` — `--duration 2`.

`start_periodic_server` is now readiness-bounded and `finish_server` is exit/drain-bounded, but those protections do not help if the client never returns from `.output()`.

### Endpoint-rebind family

8. `endpoint_rebind_real_sockets_promote_new_source_and_reject_stale_old_source` — `endpoint-rebind-client --duration 5`.
9. `endpoint_rebind_wrong_challenge_fails_after_candidate_without_success` — `endpoint-rebind-client --duration 3`.

These tests likewise reach `finish_server(server)` only after the synchronous real client exits. A client regression can therefore strand the entire gate indefinitely.

This is not a claim that every `.output()` is defective. Keygen/help/capabilities/argument-validation/socket-free fixture calls that are source-local/fail-fast or otherwise independently self-bounded remain excluded. It is also not a production Session/Carrier/ACK/crypto/wire correctness finding; it is a process-test ownership/evidence finding.

## Smallest repair contract

1. Reuse exact-current `bounded_client_output`; do not create another process framework.
2. Convert the seven periodic real client owners above to `bounded_client_output` while preserving every current argument and assertion.
   - `--duration 5` clients: use the established `duration + 5s` harness convention => `Duration::from_secs(10)`.
   - `--duration 2` clients: use `Duration::from_secs(7)`.
3. Convert the two endpoint-rebind real client owners similarly:
   - `--duration 5` => `Duration::from_secs(10)`;
   - `--duration 3` => `Duration::from_secs(8)`.
4. Preserve all current periodic delayed-confirmation, key-update, mismatch, missing/duplicate ACK, setup-timeout and authentication assertions; preserve all endpoint candidate/challenge/promotion/sync/stale-source/failure assertions.
5. Existing `bounded_client_output_fails_when_client_never_exits` already proves the ordinary shared deadline path. Do not add duplicate generic sleep churn unless one of these owners exposes a distinct failure mode.
6. Do not mechanically convert keygen, help/capabilities, invalid argument/configuration, or socket-free fixture `.output()` calls.
7. No decoder/parser/crypto-framing implementation change is implicated; do not run fuzz mechanically.
8. On the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist developer-local exact-tree provenance with exact reachable SHA, UTC start/end, exit codes, OS/arch, stable Rust, and clean-tree state.
9. Continue immediately with the remaining process/socket/thread ownership causal sweep after closure; do not infer repository-wide queue exhaustion from this family repair.

## Evidence boundary

Reviewer activity for H-I4-113 is exact-current source/control-flow inspection only. Reviewer did not execute Rust/full gate, cross-platform process tests, fuzz, WAN, or performance work. Exact `fc23a7a` has reachable developer-reported local provenance for H-I4-112; GitHub combined status exposed no hosted status entries for that exact source SHA. Those evidence classes remain separate.

`READY_LIVE: none` remains unchanged. Release items 3 and 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.
