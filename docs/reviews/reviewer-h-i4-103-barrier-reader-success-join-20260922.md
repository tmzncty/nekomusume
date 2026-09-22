# H-I4-103 — barrier-success reader join can block before exit proof

**Severity:** HIGH (item-4 / release-evidence process-harness correctness)

**Reviewed exact repository anchor:** `33085124fc30acbe15183bcaa9b0478ef5c50ed3`

**Owner:** `crates/neko-cli/tests/probe.rs`

This is a bounded independent source/control-flow review. It is not reviewer-local Rust execution, hosted CI, WAN evidence, a performance conclusion, or a production transport finding.

## Finding

H-I4-102 is closed on its original claim at `319ac5d8caa7c245c35e4477bb3cf707ffb4fee2`: `bounded_reap_or_kill` now routes its own initial `try_wait()` error through `bounded_kill_reap` before failing, and the exact-tree developer-local gate is retained in `docs/notes/h-i4-102-provenance-319ac5d-20260922.md`.

The continuation process-ownership sweep found a separate unbounded success-side reader join.

`malformed_classification_barrier(...)` owns a reader thread whose loop blocks in `read_line()`. Once the requested classification count is reached, the function drops the receiver and returns the still-live `child`, the `reader_handle`, and `BarrierProof`. Dropping the receiver does **not** wake a thread that is already blocked in `read_line()`: the thread exits only after a later stdout line lets `tx.send(...)` observe the disconnected receiver, or after EOF/read error.

The positive malformed-churn test later does this after the failover client returns:

```rust
let (stdout, barrier_lines) = reader_handle.join().expect("reader thread joinable");
```

That join occurs **before** any bounded child-exit observation or `finish_server(...)` cleanup. The comment assumes that the failover-client run has caused another server stdout line, but the harness does not mechanically establish that fact. A server/lifecycle/diagnostic regression in which the client returns while the server remains live and emits no further stdout leaves the reader blocked and the test hangs forever instead of producing bounded negative evidence.

A deterministic counterexample shape is available without network-policy changes: a child can emit exactly the required `malformed_or_unadmitted` line, keep stdout open, and remain alive/silent. The barrier succeeds, drops its receiver, and returns; an immediate raw `JoinHandle::join()` has no local deadline and cannot complete until the child/pipe changes state.

This is the same evidence-oracle class challenged by H-I4-096/H-I4-099/H-I4-100, but it is a distinct owner edge: **barrier success -> returned live reader -> direct join before exit proof**. It does not imply a production Session/Carrier/ACK/crypto/wire defect.

## Closure contract

1. Remove the direct unbounded reader join from the post-barrier success path. A reader `join()` is legal only after either:
   - child exit / pipe closure has been mechanically established, or
   - reader completion itself is observed through a local bounded mechanism.
2. On the bound expiring while the child/reader is unresolved, fail closed through the existing bounded process cleanup path before any final join. Reuse the existing process primitives or a very small test-local helper; do not create a general process framework.
3. Add a deterministic negative regression for the success-side barrier handoff: satisfy the barrier count, then keep the child alive with stdout open and no additional line. The regression must terminate within its local bound and converge child/reader ownership rather than hanging.
4. Preserve the accepted H-I4-090..095 resource-causality proofs: READY-derived baseline, `churn_started` edge, `ATTEMPTS`, post-classification `BarrierProof`, Linux-only affirmative `/proc` observation, current FD/RSS margins, and post-churn authenticated success semantics.
5. Preserve H-I4-097..102 cleanup fixes and their evidence boundaries. Do not change runtime Session/Carrier/ACK/crypto/wire semantics or invent timeout/capacity/security policy values.
6. No decoder/parser/crypto-framing owner is implicated; fuzz is not required for this test-only repair.
7. On the final pushed source/test SHA, run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm a clean tree, and persist developer-local exact-tree provenance with reachable SHA, UTC start/end, exit codes, OS/arch, and stable Rust version.
8. Immediately continue the remaining `wait` / `output` / stdout-drain / reader-join / child-ownership sweep after closure; do not treat this narrow repair as repository-wide queue exhaustion.

## Evidence boundary

Current `README.md`, `IMPLEMENTATION_PLAN.md`, `docs/status.md`, and `docs/release-security-review-packet.md` still keep release evidence and independent review incomplete. `READY_LIVE: none` remains authoritative; this finding creates no new real-network question. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.
