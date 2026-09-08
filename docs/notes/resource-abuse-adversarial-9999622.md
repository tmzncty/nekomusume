# Bounded pre-auth adversarial evidence — exact `d120e74`

## Scope and boundary

This is a focused evidence package against the existing candidate pre-auth limits. It does not change numeric policy, claim public-listener safety, establish production capacity, or substitute for independent security review.

## Exact tree and coverage

- tested tree: exact `d120e74b47df22959a23a39b031b10c7b360f3e1`; this commit newly adds the two `adversarial_*` tests below, while the source-projection, expiry, staged-terminalization, response-send and process-window tests are inherited evidence re-run at that tree
- source projection: `crates/neko-cli/src/preauth.rs::source_key` projects carrier, IP family, binary address bytes and port; it retains no textual address. Tests cover TCP/UDP separation, IPv4/IPv6 separation and port separation.
- listener inventory: `python3 scripts/check-preauth-responder-inventory.py` reports 7 responder surfaces and 6 admission sites. The manifest covers ordinary TCP/UDP, periodic TCP, multistream TCP, failover UDP pending/new and failover TCP.
- charge ordering: every inventoried TCP responder uses `read_staged_frame`; ordinary/failover UDP charges bounded datagram input before negotiation or Noise parsing; response permits are charged before bounded TCP/UDP send helpers.
- response accounting: `response_send_restores_socket_write_timeouts` exercises both TCP and UDP charged-send paths; the helper abandons permits on timeout, socket setup, partial-write or short-send failure and completes only an exact successful send.
- staged failure cleanup: `inner_budget_rejection_terminalizes_outer_ticket` and `staged_outer_rejection_terminalizes_inner_and_ticket` prove fail-closed terminalization; no retry or queue admission remains after rejection.
- source/global concurrency: `adversarial_same_source_reservations_are_bounded_and_redacted` fills the existing per-source state ceiling, rejects the plus-one attempt, releases all tickets and observes zero live states/zero memory. Existing crypto process tests cover global max-plus-one and atomic source/global limits.
- input rejection and policy boundary: `adversarial_input_budget_rejects_before_parse_and_reopens_source` consumes the existing per-ticket input-packet ceiling, rejects the next input and response, then demonstrates that current terminal release removes the final source entry and permits re-admission. This is evidence of the unresolved D019 terminal source-retention conflict, not evidence that source-lifetime no-reset semantics are closed. Existing process tests separately cover global rate-window rejection and rollover terminality.
- expiry/release: existing `expired_ticket_invalidates_application_queue_owner`, `ordinary_expiry_invalidates_pending_owner_without_stopping_controller`, and crypto `queue_expiry_boundaries_release_state_memory_and_ownership_once` cover idle/lifetime expiry, queue invalidation, memory release and subsequent admission.
- redacted aggregate diagnostics: `ProcessPreauthAdmission` has a custom `Debug` surface containing only aggregate counts (`live_states`, `source_count`, memory, queue, window input/work/response totals). The adversarial test asserts the complete permitted output shape; source keys, tuple bytes, state IDs, limits and timestamps are absent and cannot be reconstructed from it.

## Verification

- `cargo test -p neko-cli preauth --locked`: 10 passed
- `cargo test -p neko-crypto preauth_tests --locked`: 20 passed
- `python3 scripts/check-preauth-responder-inventory.py`: passed, 7 surfaces / 6 admission sites
- `git diff --check`: passed

The package closes only the non-policy implementation/test evidence listed above. Full RSEC-001 remains blocked: terminal source retention after the last live state is a maintainer/security-policy checkpoint already documented in `docs/adr/m1-g0-preauth-source-retention-amendment-request.md`. This package neither chooses TTL/LRU/history values nor weakens D019. Release, production and freeze flags remain false.