# H-I4-100 — UDP application-deadline process test still has a direct unbounded child wait

**Severity:** HIGH for release item 4 / process-test evidence correctness.

**Reviewed current tree:** `9e4b162e31c44e06a9f04a27db2e3f28181bf582` (`main` at review start).

**Scope:** exact-current `crates/neko-cli/tests/probe.rs`, especially `udp_application_wait_fails_at_bounded_overall_deadline`, after accepting the H-I4-099 `finish_server` repair at `ea3e735a7e073093c947a6881556af7cf7f1e4fa`.

## Accepted preceding repair

H-I4-099 is closed on its narrow owner: `finish_server` now drains stdout off-thread, polls child exit with a local bound, and on deadline routes through `bounded_reap_or_kill`. The exact source/test tree `ea3e735` has developer-reported clean exact-tree provenance in `docs/notes/h-i4-099-provenance-ea3e735-20260922.md`; hosted Rust CI run `35730719557` also completed successfully. Those evidence classes remain distinct.

## Concrete counterexample

`udp_application_wait_fails_at_bounded_overall_deadline` intentionally starts a server with `--duration 1`, runs an authenticated UDP client delayed by 1.25 s, and then performs:

```rust
let status = server.child.wait().unwrap();
let elapsed = started.elapsed();
let mut log = server.startup_log;
server.stdout.read_to_string(&mut log).unwrap();
```

The direct `Child::wait()` has no independent harness deadline. The test later asserts that the overall path is bounded (`elapsed < 3 s`), but `elapsed` is sampled only *after* the blocking wait returns. If the server exit/lifecycle path regresses and the child remains live, the test does not fail its boundedness assertion; it hangs before reaching it.

The nominal `--duration 1` is runtime intent, not an independent process-harness bound. This is the same causal class that H-I4-099 repaired for `finish_server`: an expected server lifetime cannot substitute for a bounded observer when the exit path itself is under test.

The `read_to_string` calls after a proven child exit are not the finding; once exit is established, EOF is causally available. The defect is the unbounded direct wait before that proof.

This does **not** establish a production/runtime transport leak, does not reopen Session/Carrier/ACK/crypto/wire architecture, and does not change the UDP application deadline semantics. It invalidates only the repository-wide claim that every relevant process-test ownership path is independently bounded.

## Closure contract

1. Replace the direct `server.child.wait()` in `udp_application_wait_fails_at_bounded_overall_deadline` with a bounded exit observation. Reuse the current process-test cleanup primitive where possible; do not reintroduce a blocking wait on an unproven-live child.
2. Preserve the existing semantic oracle: authenticated delayed UDP application data must fail closed, server status must be unsuccessful, the existing lower bound remains meaningful, and the existing `< 3 s` overall test contract must remain the factual upper-bound oracle. Do not widen runtime/session semantics.
3. If a helper is introduced, keep it narrow: poll `try_wait` against an explicitly supplied test deadline; on deadline route through `bounded_reap_or_kill` and fail closed. Do not create a generalized process framework or invent a new repository-wide timeout/security policy.
4. Add a deterministic negative regression only if needed to prove the helper itself cannot hang with a live child. Any short helper-test duration is a fixture parameter, not a protocol/security/capacity policy value.
5. After repair, continue the exact-current owner-by-owner process sweep. In particular classify every remaining direct `wait_with_output` / `output`, stdout drain and reader/thread `join` in `probe.rs` as either exit-proven/success-path bounded or failure-path externally bounded. Convert only concrete unproven-exit blockers into repairs.
6. Focused tests, then the final pushed source/test SHA must run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and finish clean. Persist developer-local exact-tree provenance with exact reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust. No fuzz unless decoder/parser/crypto-framing owners change.

## Evidence boundary

This review is source/control-flow reasoning on the exact current GitHub tree. The reviewer did not execute Rust tests, the full local gate, cross-platform tests, fuzz, WAN, or performance work in this review. `READY_LIVE` remains `none`; release items 3 and 4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.
