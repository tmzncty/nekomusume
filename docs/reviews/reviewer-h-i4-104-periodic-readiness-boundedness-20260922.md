# H-I4-104 — `start_periodic_server` readiness wait is not harness-bounded

**Severity:** HIGH — item-4 / release-evidence process-test boundedness.

**Exact-current source/spec anchor before this finding:** `a179b4c05c4291d44be35c5fbf33aab6ff53cbc9`.

**Prior closure accepted:** developer-owned `d2aa63b3b02f988a580919f397fabd6db65e4bcf` closes H-I4-103 on its original barrier-success reader-join claim. Its developer-local exact-tree provenance is retained in `docs/notes/h-i4-103-provenance-d2aa63b-20260922.md` (`scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, Linux x86_64, rustc 1.98 stable). No GitHub-hosted workflow/status was visible for exact `d2aa63b` during this reviewer pass. That is developer-reported local evidence, not reviewer-local execution.

## Inspected owner and invariant

Inspected exact-current `crates/neko-cli/tests/probe.rs`, in particular `start_periodic_server`, `wait_for_ready_marker`, `finish_server`, `bounded_wait_exit`, `bounded_reap_or_kill`, and the periodic process tests that consume `start_periodic_server`.

Invariant challenged: a process-test readiness phase must either prove readiness or fail within a harness-local bound while converging child/pipe ownership. A nominal runtime `--duration` is behavior under test and cannot substitute for the harness deadline that detects lifecycle/readiness regressions.

## Concrete counterexample

`start_periodic_server(...)` still performs readiness synchronously on the test thread:

```rust
let mut child = command.spawn().unwrap();
let mut stdout = BufReader::new(child.stdout.take().unwrap());
let mut startup_log = String::new();
loop {
    let mut line = String::new();
    assert_ne!(stdout.read_line(&mut line).unwrap(), 0, "{startup_log}");
    startup_log.push_str(&line);
    if line.contains("periodic_server_ready") {
        break;
    }
}
```

There is no `recv_timeout`, deadline poll, or bounded cleanup on this readiness edge. If the child remains alive with stdout open but never emits a complete `periodic_server_ready` line, `read_line()` can block indefinitely. Because `start_periodic_server` has not returned, the already-reviewed `finish_server` / `bounded_wait_exit` cleanup paths are unreachable.

The helper passes `--duration 5`, but that does not make the harness wait independent: a lifecycle/readiness regression that also prevents the child from exiting at the nominal duration is precisely a failure shape the process harness must turn into bounded negative evidence rather than a hung test. The same principle already closed H-I4-100 for a direct application-deadline `wait()`.

This helper feeds multiple periodic Session tests (delayed confirmations, key update, schedule mismatch, missing/duplicate ACK, setup-timeout separation/failure and malformed setup), so the unbounded readiness edge is shared process-test infrastructure rather than a one-off assertion.

This is **not** evidence of a production Session/Carrier/ACK/crypto/wire defect and does not change runtime protocol semantics.

## Closure contract

1. Replace the raw main-thread readiness `read_line()` loop in `start_periodic_server` with the already-reviewed bounded readiness primitive (`wait_for_ready_marker`) or a strictly equivalent narrow wrapper. Preserve the existing `ReadyServer` startup-log/stdout ownership and `periodic_server_ready` marker semantics.
2. Reuse an existing test-local readiness bound; do not invent a new repository-wide timeout, capacity or security-policy value.
3. Add a focused deterministic negative regression for the periodic readiness owner: a child remains live with stdout open and silent (or never completes the readiness marker), and the periodic readiness path must terminate within its local bound, perform bounded cleanup, and not strand reader/child ownership. If the implementation delegates completely to an already-covered helper, the regression may be narrowly shaped to prove this caller actually uses that bounded edge rather than duplicating a generic process framework.
4. Preserve H-I4-097..103 process-cleanup fixes and H-I4-090..095 malformed-resource causality/cfg proofs. Do not weaken `ReadyProof` / `BarrierProof`, FD/RSS margins, `ATTEMPTS`, or authenticated lifecycle assertions.
5. No decoder/parser/crypto-framing change is implicated; do not run fuzz mechanically.
6. On the final pushed source/test SHA, run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist developer-local exact-tree provenance with reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust.
7. Continue immediately into the remaining process wait/output/join ownership sweep after closure. Do not infer repository-wide queue exhaustion from this repair.

## Evidence boundary

This review is exact-current GitHub source/control-flow review only. No reviewer-local Rust/full-gate, cross-platform execution, WAN, fuzz or performance execution is claimed. `READY_LIVE: none` remains authoritative. Release items 3/4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.
