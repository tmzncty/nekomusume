# Reviewer H-I4-099 — `finish_server` can block before exit proof

**Severity:** HIGH (item-4 / release-evidence process-test boundedness)

**Exact reviewed tree:** `03c5b5410a5494a204737e8bda4a90633fa71c1c`

**Owners inspected:** `crates/neko-cli/tests/probe.rs` process ownership/readiness/cleanup helpers and call sites; current `docs/CHATGPT_HANDOFF.md`; the independent cross-platform CLI/process no-finding at exact `49925acf7d2d77595562d9ad3327f4b145a31fd1`; current implementation/release/status/governance/spec boundaries.

## Concrete counterexample

`finish_server` currently does:

```rust
fn finish_server(mut server: ReadyServer) -> (std::process::ExitStatus, String) {
    let mut remainder = String::new();
    server.stdout.read_to_string(&mut remainder).unwrap();
    let status = server.child.wait().unwrap();
    server.startup_log.push_str(&remainder);
    (status, server.startup_log)
}
```

The child is still live/unproven-exited when `read_to_string` begins. `ChildStdout::read_to_string` waits for EOF, and EOF is controlled by the child closing stdout or exiting. If the server remains alive while keeping stdout open (for example because shutdown/duration handling regresses or the child enters a stuck state), `finish_server` can block indefinitely **before it ever reaches `child.wait()`**.

Therefore the statement in `docs/reviews/independent-cross-platform-cli-process-b4af007-20260922.md` that `finish_server` “waits only after stdout drained” establishes neither an exit proof nor a local bound. Draining stdout is itself the unbounded wait surface here. The stronger statement in that review — “No unbounded `wait()` on an unproven-exited child” / child ownership converges on every failure shape — cannot be retained as repository-wide process boundedness evidence without qualifying or repairing this helper.

This is a process-test/evidence-oracle defect. It is **not** evidence of a production transport resource leak, and it does not reopen Session/Carrier/ACK/crypto/wire architecture.

## Why nominal `--duration` is not a closure proof

Many `finish_server` call sites launch a server with a finite nominal duration or signal it before finishing. That is useful runtime intent but not an independent local cleanup bound: the helper is exactly where the harness must remain fail-closed if the child fails to honor the intended exit path. A process/lifecycle regression must make the test fail within a bounded interval rather than hang the gate.

## Closure contract

1. Make `finish_server` itself locally bounded. Do not enter a blocking stdout drain while the child is still unproven-exited.
2. Preserve complete log collection when the child exits normally. A narrow acceptable shape is an off-thread stdout reader plus a bounded child-exit/reap loop; on deadline, fail closed and route termination/reaping through the already-reviewed bounded cleanup primitive, then reconcile/join the reader only after child exit/pipe closure is established.
3. Do not add a generic process framework or invent new security/capacity policy. Reuse existing local timeout/cleanup conventions where possible.
4. Add a deterministic negative regression with a child that keeps stdout open and remains live past the helper deadline, proving `finish_server` fails within a bound rather than hanging. Preserve existing positive lifecycle/log assertions.
5. Re-challenge all remaining `wait`, `wait_with_output`/`output`, stdout drain and reader `join` sites after this repair. Do not infer repository-wide exhaustion from this one helper.
6. Final pushed source/test SHA must receive focused tests plus `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean-tree verification, and persisted developer-local exact-tree provenance (SHA, UTC start/end, exits, OS/arch, stable Rust). No fuzz is required unless decoder/parser/crypto-framing owners change.

## Evidence boundary

This review is exact-current source/control-flow reasoning only. Reviewer did **not** execute Rust tests, the full local gate, cross-platform CI, fuzz, WAN experiments, or performance tests in this pass.

`READY_LIVE` remains `none`; release items 3 and 4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.
