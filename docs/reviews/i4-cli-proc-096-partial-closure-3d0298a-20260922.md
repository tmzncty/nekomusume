# Independent item-4 review — I4-CLI-PROC-096 partial closure

**Reviewed anchor:** `3d0298ac6870d334f36b1d0489c420895c838413` (`main` at review start).

**Review class:** exact-current source/diff + evidence-boundary review. No reviewer-local Rust/full-gate, cross-target, fuzz, WAN, or performance execution is claimed here.

## New developer commits reviewed

- `0ed814ce4adbc6de9b803904baa665629dc2bb15` — H-I4-095 target-neutral `ReadyProof` / `BarrierProof` repair.
- `20480d40582e07f0a792d6cdbd585a0817384b1b` — H-I4-095 handoff/provenance closure.
- `a50ce9d37c24646c5824f3dd738942a3ded16b36` — `ready_endpoint_rebind_server` bounded-read repair plus silent-child regression.
- `3d0298ac6870d334f36b1d0489c420895c838413` — handoff/provenance update.

H-I4-095 remains accepted as closed. The exact-tree `a50ce9d` local gate in `docs/notes/i4-cli-proc-096-provenance-a50ce9d-20260922.md` is accepted only as developer-reported local provenance; no hosted run/status was visible during this review, and no non-Linux execution is inferred.

## I4-CLI-PROC-096 is only partially closed

This remains a **READY_LOCAL item-4 process-test boundedness defect, not HIGH/BLOCKER and not a runtime transport defect**.

### 1. `start_server_for(...)` still has a non-bounding deadline

Exact-current `crates/neko-cli/tests/probe.rs` still does:

```text
let deadline = Instant::now() + Duration::from_secs(2);
loop {
    assert!(Instant::now() < deadline, ...);
    let read = stdout.read_line(&mut line).unwrap();
    ...
}
```

The clock is checked before a blocking `read_line`. A live child that emits no newline/EOF can therefore block forever after the deadline check. The prior review explicitly named this helper together with `ready_endpoint_rebind_server`; `a50ce9d` repaired only the latter.

### 2. `ready_endpoint_rebind_server(...)` timeout is bounded, but EOF/read-error cleanup can still block

`a50ce9d` correctly moved stdout reading off-thread and bounds the silent-child path with `recv_timeout(5s)`. On channel timeout it kills then waits for the child, so the silent-live regression is meaningful.

However, the reader thread sends `Err(startup_log)` on stdout EOF or read error, and the main thread handles `Ok(Err(...))` with only:

```text
let _ = child.wait();
panic!(...)
```

A child is allowed to close stdout while remaining alive (or a read error may occur before process exit). In that case `child.wait()` is itself unbounded, so the helper can still hang on the failure path despite the nominal five-second readiness bound. The persisted provenance sentence that says timeout **or stdout EOF** kills/reaps the child is therefore stronger than the exact source.

The current negative regression covers the timeout/silent case only; it does not exercise a child that closes stdout and remains alive.

## Closure contract

Use the smallest test-helper repair; do not build a general process framework.

1. Make `start_server_for(...)` genuinely bounded when the child stays alive but emits no newline/EOF. Reuse the existing off-thread-reader + bounded receiver pattern or an equivalent interruptible mechanism.
2. Keep `ready_endpoint_rebind_server(...)` success semantics, startup-log accumulation and five-second bound, but make **all failure exits** bounded: timeout, EOF, read error and channel disconnect must deterministically terminate/reap the owned child before panic/return. Do not call an unbounded `wait()` on a child that has not been proven exited.
3. Ensure reader ownership converges after termination; do not leave a reader thread that can remain blocked indefinitely.
4. Add focused negative coverage for the still-uncovered failure shape(s): at minimum a live-but-silent path for `start_server_for` or its extracted narrow wait primitive, and an endpoint-readiness child that closes stdout (or otherwise terminates the reader path) while remaining alive, proving bounded failure rather than blocking in `wait()`.
5. Preserve all existing readiness strings, positive process semantics, `ReadyProof` / `BarrierProof` causal gates, FD/RSS margins, ATTEMPTS and release/resource policy values.
6. No fuzz unless decoder/parser/crypto framing owners change.
7. On the final pushed source/test SHA run focused process tests, `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, verify clean tree, and persist exact reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust version.
8. Immediately continue into the pre-auth malformed/rejection resource-accounting bounded challenge after commit/push/provenance; do not wait for reviewer cadence.

## Evidence/governance boundary

- `READY_LIVE: none` remains authoritative; no repeated WAN/HY2/failover evidence is requested merely for freshness.
- Release items 3 and 4 remain incomplete.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- `SessionRuntime.events` retained-history capacity remains a maintainer/security policy gate; this review chooses no TTL/LRU/history-size/capacity value.
