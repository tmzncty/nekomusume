# H-I4-105 — auxiliary peer threads can still strand CLI process tests

**Severity:** HIGH (release/item-4 test-harness correctness; not a production transport defect)

**Exact reviewed anchor:** `67dad3caaafa1709e2ced8719aad079bd56b5866`

## Reviewed delta and boundary

H-I4-104 is accepted as closed at developer-owned `0467152d4e595c30ba6373333cd19bf8fa382cc0`: `start_periodic_server` now delegates readiness to the existing bounded `wait_for_ready_marker(..., 5s)` primitive. Its persisted developer-local exact-tree provenance records `scripts/check.sh` exit 0, `git diff --check` exit 0 and a clean Linux x86_64 / Rust stable tree. No exact-`0467152` hosted workflow/status was visible during this reviewer pass. This review did not execute Rust locally.

The follow-on ownership sweep challenged a different invariant:

> Any auxiliary socket peer thread that the test later `join()`s must have an independent test-local completion bound. A regression in the CLI path under test must become deterministic negative evidence, not leave the gate blocked forever in `accept`, `recv_from`, `read_exact`, or `join`.

Ordinary synchronous CLI `.output()` calls are not classified as defects merely because they are synchronous. This finding is specifically about auxiliary peer owners whose blocking socket operation can remain live after the CLI process has already returned.

## Concrete counterexamples

### 1. Matrix probe positive TCP/UDP peers

`crates/neko-cli/tests/probe.rs::matrix_probe_distinguishes_invalid_failed_and_reachable_outcomes` creates:

- a TCP worker that blocks in `TcpListener::accept()` with no local deadline, then the main test calls `tcp_peer.join()` after the CLI returns;
- a UDP worker whose first `recv_from()` has no read timeout, then the main test calls `udp_peer.join()` after the CLI returns.

If a reachable-path regression makes `neko-cli probe --matrix` return without connecting/sending, the executable has already returned but the auxiliary peer remains blocked forever. The assertion that the probe should have been reachable is never reached.

### 2. Negotiation transcript-mismatch peers

`crates/neko-cli/tests/probe.rs::tcp_and_udp_transcript_mismatch_rejects_before_application_echo` joins an auxiliary peer after `run_client(...)`, but:

- the TCP branch blocks in `accept()` and then in `frame_read_test()` / `read_exact()` without socket deadlines;
- the UDP branch blocks in its first and second `recv_from()` without read deadlines.

An early client failure before one of those expected network steps can therefore strand `peer.join()` instead of producing the intended handshake-rejection evidence.

### 3. Unsupported-selected-version TCP peer

`crates/neko-cli/tests/probe.rs::tcp_and_udp_reject_unsupported_selected_version_before_noise` correctly applies a UDP read timeout, but the TCP branch still performs an unbounded `accept()` and first `frame_read_test()` before any read timeout is installed. The caller then joins the peer after `run_client(...)` returns. The TCP half therefore retains the same hang shape.

### 4. Multistream transcript-binding peer

`crates/neko-cli/tests/multistream.rs::executable_rejects_one_byte_different_negotiation_binding_before_session_data` spawns an auxiliary TCP peer that performs unbounded `listener.accept()` followed by two `frame_read()` calls using `read_exact()`. The main thread runs the CLI, checks its expected rejection, then `peer.join()`s. If the client exits before an expected connect/frame, the peer can remain blocked indefinitely.

These are distinct owners from H-I4-097..104. Those findings hardened child-process cleanup/readiness/barrier ownership in `probe.rs`; they do not establish boundedness of independent test peer socket threads.

## Why HIGH

These tests are part of the release/item-4 correctness surface and are reachable from the normal workspace gate. Their purpose is to catch CLI/network regressions. In the counterexamples above, the regression can instead turn the test itself into an indefinite hang, so the repository cannot truthfully claim that the relevant process/CLI test failure shapes are causally bounded.

This does **not** establish a production runtime leak, wire/crypto defect, Session/Carrier semantic defect, WAN failure, or performance issue.

## Closure contract

1. Give each affected auxiliary peer a narrow test-local completion bound before any blocking operation that precedes `join()`:
   - UDP peers: set a read timeout before the first relevant `recv_from()`;
   - accepted TCP streams: set read deadlines before blocking frame reads;
   - listener acceptance: use a bounded/nonblocking accept loop or a narrow equivalent helper with a local deadline.
2. Preserve the exact positive/rejection semantics and existing negotiation/crypto assertions. A timeout is a harness failure, not protocol evidence.
3. Do not add a repository-wide timeout/capacity/security policy value. Reuse an established small test-local bound where practical.
4. Add focused deterministic negative coverage for the shared shape: leave the auxiliary peer without the expected connect/datagram/frame and prove the peer owner completes/fails within its local bound rather than hanging the join. Keep this narrow; do not build a generic process framework.
5. Keep H-I4-090..104 closed on their original claims unless exact-current source provides a distinct counterexample. Do not weaken ReadyProof/BarrierProof, malformed-resource ordering, cleanup, or existing socket assertions.
6. No decoder/parser/crypto-framing implementation change is required by this finding; do not run fuzz mechanically.
7. On the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm a clean tree, and persist developer-local exact-tree provenance with reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust.
8. Continue immediately into the remaining process/socket/thread ownership sweep after closure; do not infer repository-wide queue exhaustion from this repair.

## Reviewer execution statement

This finding is based on exact-current GitHub source/control-flow review at `67dad3c`. No reviewer-local Rust/full-gate, cross-platform execution, fuzz, WAN, or performance run is claimed.
