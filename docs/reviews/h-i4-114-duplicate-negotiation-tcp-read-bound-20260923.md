# H-I4-114 — duplicate-negotiation TCP response read is not harness-bounded

**Severity:** HIGH — release/item-4 test-harness correctness and evidence reliability

**Reviewer anchor:** exact current `main` inspected at `d2827851751b0eddc782943ee3498aeba029f513`, including developer-owned H-I4-113 source commits `8fc460c8eeb2369efaa6e0651f2a3fd8f864ccae` and formatting-only `d2827851751b0eddc782943ee3498aeba029f513`.

## Current closure state of H-I4-113

The nine H-I4-113 real clients are now source-bounded through existing `bounded_client_output` with the requested duration + 5s bounds, and their prior assertions remain in place. That source repair is accepted. Final reachable developer-local exact-tree provenance for the final source tree has not yet been found, so H-I4-113 provenance remains pending; reviewer source inspection is not a substitute for that gate.

## Inspected owner and invariant

Inspected exact-current `crates/neko-cli/tests/probe.rs`, especially `tcp_and_udp_reject_malformed_unsupported_and_duplicate_negotiation_before_echo`, `frame_read_test`, the H-I4-105 transcript-mismatch / unsupported-selected-version peer threads, and current bounded child helpers. Also checked exact-current `crates/neko-cli/tests/multistream.rs` process helper ownership while continuing the remaining process/socket/thread causal sweep.

Invariant challenged: any socket read whose completion depends on behavior under test must have an independent harness bound before the blocking operation. A later timeout, bounded server cleanup, or product lifecycle expectation is not reachable while the test thread is already blocked in `read_exact`.

## Concrete defect

In the TCP duplicate-negotiation branch of `tcp_and_udp_reject_malformed_unsupported_and_duplicate_negotiation_before_echo`:

1. a real server is started;
2. the test connects a real `TcpStream` and sends the first valid negotiation hello;
3. it immediately calls `frame_read_test(&mut socket)` to receive the server's negotiation response;
4. `frame_read_test` performs two blocking `read_exact` calls;
5. only **after** that response succeeds does the test call `socket.set_read_timeout(Some(Duration::from_secs(1)))` for the second/duplicate-hello close check.

If the server accepts the connection but regresses into never emitting or only partially emitting the first negotiation response, the test can block forever in the first `frame_read_test`. The later 1s timeout and `finish_server(server)` are unreachable. This turns a protocol/lifecycle regression that should be bounded negative evidence into a hung gate.

The UDP duplicate branch already installs a read timeout before its first `recv`, and the H-I4-105 transcript-mismatch / unsupported-selected-version TCP peer branches install a read timeout before their first `frame_read_test`; those owners are not part of this finding.

This is not a production wire/negotiation architecture finding and does not require changing negotiation semantics. It is a test-harness socket-ownership/evidence defect.

## Smallest repair contract

1. In the TCP duplicate-negotiation branch, install the existing local read bound **before the first** `frame_read_test(&mut socket)`. Reusing the already-established 1s or another exact-current test-local bound already used by this owner family is appropriate; do not introduce repository-wide timeout policy.
2. Keep the existing first response assertion, duplicate hello, terminal-close assertion, UDP branch, server status/log assertions, and protocol bytes unchanged.
3. Add or retain a focused deterministic negative regression/proof that a connected peer which never supplies the expected response cannot strand the blocking read. If a separate synthetic regression would only duplicate `set_read_timeout` semantics, source-level owner proof plus the existing timed socket tests is sufficient; do not manufacture framework churn.
4. Continue the remaining process/socket/thread ownership sweep after repair. Do not infer repository-wide queue exhaustion from closing this one read owner.
5. No decoder/parser/crypto-framing implementation change is implicated; do not run fuzz mechanically.
6. On the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist developer-local exact-tree provenance with exact reachable SHA, UTC start/end, exit codes, OS/arch, stable Rust, and clean-tree state. That provenance may also close the still-pending H-I4-113 final exact-tree gate if it explicitly and accurately covers the unchanged H-I4-113 source repair.

## Evidence boundary

Reviewer activity for H-I4-114 is exact-current GitHub source/control-flow inspection only. Reviewer did not execute Rust/full gate, cross-platform process tests, fuzz, WAN, or performance work in this pass. H-I4-113 source conversion is accepted from exact-current source, but its final developer-local exact-tree provenance remains pending. Hosted CI was not used as closure evidence.

`READY_LIVE: none` remains unchanged. Release items 3 and 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.
