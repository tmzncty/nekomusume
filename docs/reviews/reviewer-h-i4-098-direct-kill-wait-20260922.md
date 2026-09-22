# H-I4-098 — direct `kill(); wait()` bypasses bounded cleanup

**Severity:** HIGH — item-4 / release-evidence process-test boundedness.  
**Reviewed tree:** `c7a6eb32fcc3bfd4d8a7c12f74dcc3e11e704ba0` (developer source repair at `76f33be72a1c3c24152193fb174194631cb7cf95`).  
**Scope:** reviewer source/control-flow challenge only; no reviewer-local Rust/full-gate, cross-platform execution, fuzz, WAN or performance run.

## What is accepted from H-I4-097

The shared `bounded_reap_or_kill` repair at `76f33be` fixes the helper-local defect that motivated H-I4-097: `try_wait` and `kill` errors fail closed, and post-kill reaping is a bounded `try_wait` poll loop instead of an unconditional blocking `wait()`. `malformed_classification_barrier` now routes its timeout/EOF/channel-error cleanup through that helper. The persisted exact-tree gate remains developer-reported local provenance only.

## Concrete remaining defect

The dependency-ordered process-cleanup re-challenge found one direct cleanup site that bypasses the repaired helper:

`crates/neko-cli/tests/probe.rs::reliable_udp_incomplete_settlement_fails_not_settled`

```rust
let _ = server.child.kill();
let _ = server.child.wait();
```

This reproduces the exact control-flow class H-I4-097 was intended to remove. The `kill()` result is ignored. If termination fails while the child is still live, the following `wait()` has no local deadline and can block past the test's intended bound. The server's nominal `--duration 8` is not a sufficient cleanup proof: a process/lifecycle regression is one of the failure modes the integration test surface must fail closed around, rather than relying on the child to self-terminate correctly.

This finding is **not** evidence of a production/runtime transport leak, and does not reopen Session/Carrier/ACK/crypto/wire architecture. It is a boundedness defect in the release/item-4 process-test oracle/cleanup surface.

## Closure contract

1. Apply the smallest test-only repair: route this direct cleanup through `bounded_reap_or_kill` (or an exactly equivalent already-reviewed bounded primitive). Do not introduce a new process framework.
2. Preserve the negative test's semantic purpose: the reliable-UDP client must still fail with settlement-incomplete evidence and must not emit the successful settled marker.
3. Do not weaken H-I4-090..097 causal/resource/reader ownership behavior, `ReadyProof` / `BarrierProof`, Linux `/proc` measurement scope, FD/RSS margins, one-item handoff, or diagnostic ordering.
4. Focused validation should include `reliable_udp_incomplete_settlement_fails_not_settled` and any directly affected process-cleanup regressions.
5. Final pushed source/test SHA must run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, verify clean tree, and persist developer-local exact-tree provenance with SHA, UTC start/end, exit codes, OS/arch, and stable Rust. No fuzz unless decoder/parser/crypto-framing owners change.
6. After repair, immediately continue the repository-wide process-cleanup causal re-challenge. In particular, classify remaining direct `wait` / `wait_with_output` / stdout-drain / reader-join owners instead of declaring queue exhaustion from the repaired helper alone.

## Queue impact

H-I4-098 becomes FRONT HIGH. The prior repository-wide `queue exhausted` conclusion remains non-authoritative until this direct bypass is closed and the dependent process-cleanup/boundedness/release-packet reconciliation is repeated. `READY_LIVE` remains `none`; release items 3/4 and all release/governance flags remain unchanged.
