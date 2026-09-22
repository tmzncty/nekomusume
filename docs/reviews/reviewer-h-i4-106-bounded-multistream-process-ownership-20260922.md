# H-I4-106 — multistream process waits are not harness-bounded

**Severity:** HIGH — release/item-4 test-harness correctness

**Reviewed anchor:** `945d03dbf1b671811be498b769873349120c2b4c`

**Developer change reviewed first:** `af214408e8350e95723e3d855b1786da738588b3` (`test(cli): H-I4-105 bounded auxiliary peer ownership`). H-I4-105's original auxiliary socket/thread ownership defect is closed: the affected peer workers now install bounded accept/read behavior before the main test joins them, and the persisted developer-local exact-tree provenance at `docs/notes/h-i4-105-provenance-af21440-20260922.md` records focused regressions plus `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and a clean tree. No exact-`af214408` hosted workflow/status was visible during this reviewer pass; that local provenance is not reviewer-local execution or hosted CI.

## Challenged invariant

A built-binary/process integration test must have an **independent harness-local completion bound** for every child whose exit is part of the oracle. The process behavior under test cannot be the only proof that the process will eventually terminate. A lifecycle/protocol regression must become bounded negative evidence, not an indefinitely hung item-4 gate.

An unbounded process API is acceptable only where exact-current source establishes a separate finite termination proof. Ordinary synchronous `.output()` is therefore not automatically a finding; this finding is limited to networked multistream process owners for which that independent proof is absent.

## Concrete counterexamples

Exact-current `crates/neko-cli/tests/multistream.rs` still contains product-process owners with no harness-local deadline:

1. `bounded_tcp_multistream_loopback_is_ordered_and_json_evidenced`
   - starts the real `neko-cli multistream --mode server` with `spawn()`;
   - starts the real client with synchronous `.output()`;
   - after the client returns, calls `server.wait_with_output()`;
   - neither process wait has an independent wall-clock bound.
   - The 50 ms startup sleep is not a lifetime bound and is not a readiness proof.

2. `unauthorized_client_is_rejected_by_allowlist`
   - uses the same real server + real client ownership pattern;
   - the negative authorization result is supposed to terminate both processes, but `.output()` / `wait_with_output()` themselves provide no deadline.

3. `executable_rejects_unsupported_only_negotiation_before_noise_or_data`
   - its direct test socket has a bounded startup connect and one-second read timeout;
   - after observing the terminal-close side of the negotiation rejection, the test still calls `server.wait_with_output()` without an independent child-exit deadline.
   - If the executable closes/resets the connection yet regresses by remaining alive, the test hangs instead of reporting the lifecycle defect.

`crates/neko-cli/src/multistream.rs` does not supply a source-level outer wall-clock proof that makes those waits bounded under regression: accepted/client TCP and framed protocol work use blocking I/O as part of the behavior being tested. The intended one-connection/terminal-return semantics are the oracle, not an independent harness guarantee.

The earlier `49925acf7d2d77595562d9ad3327f4b145a31fd1` cross-platform process no-finding does not close this surface: its declared process owners were `probe.rs` readiness/cleanup helpers plus `main.rs`/manifest portability, not `tests/multistream.rs` real server/client lifecycle waits.

## Impact and boundary

A multistream lifecycle/protocol regression can strand local CI/item-4 evidence indefinitely instead of producing a deterministic failing test. This is a **test-harness/release-evidence HIGH**. It is not, by itself, a production multistream correctness finding and does not authorize changes to Session/Carrier/ACK/crypto/wire architecture.

No live/WAN question is created. `READY_LIVE: none` remains authoritative.

## Closure contract

1. Give the affected multistream child owners a narrow test-local completion bound. The helper/design must retain ownership of the child so timeout/error can perform bounded termination/reap; merely running `Command::output()` in an unkillable worker thread is not sufficient.
2. Preserve stdout/stderr evidence. Do not introduce a new pipe deadlock: if output is piped while the child is live, drain it concurrently or otherwise use a bounded ownership pattern; only perform blocking final drain after exit/pipe-closure proof.
3. On timeout, fail closed after bounded best-effort termination/reap. Do not turn timeout into protocol evidence.
4. Add one deterministic negative regression for the common process-owner shape: intentionally keep a child/process alive past the test-local deadline and prove the harness returns/fails within its bound rather than hanging. Reuse a narrow helper across the affected callers where that reduces duplicate cleanup logic.
5. Keep fast pre-network/config/keygen calls out of scope unless source review finds a separate unproven process owner; do not mechanically rewrite every `.output()` occurrence.
6. Do not create a repository-wide timeout/capacity/security policy value or a generic process framework. Use established small test-local bounds where practical.
7. Preserve H-I4-090..105 closure semantics. No decoder/parser/crypto-framing implementation change is implicated, so fuzz is not mechanically required.
8. On the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm a clean tree, and persist exact-tree developer-local provenance with reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust.
9. Continue immediately into the remaining process/socket/thread ownership sweep after closure; do not infer repository-wide queue exhaustion from this repair.

## Reviewer execution / exclusions

This finding is from exact-current GitHub source/control-flow review. The reviewer did **not** claim local Rust/full-gate execution, cross-platform execution, fuzz, WAN/live evidence, or performance evidence in this pass.
