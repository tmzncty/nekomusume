# Exact-tree check.sh provenance — 98e6ade (2026-09-26)

Developer-local exact-tree gate on the final reachable pushed SHA for the
neko-session DeliveryLedger mutation-pin slice.

- **SHA:** `98e6ade6ae9952e012cd96f7a974771127f30afe`
  (`test(session): pin ledger limit boundaries, empty ranges, containment and context regressions`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmSESSG`) created from
  `git rev-parse origin/main` (asserted equal to local HEAD, to the literal
  pushed SHA, and to worktree HEAD); `git diff --check` exit 0;
  `git status --short` empty before and after
- **UTC start:** 2026-09-26T04:09:26Z (log `/tmp/nmsess-gate.log`)
- **UTC end:** 2026-09-26T04:15:39Z
- **Exit code:** 0 (host load average ~33.9 from unrelated jobs)
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)` (`cargo 1.98.0 (797e8a9bc 2026-08-05)`)

## Pre-commit sequence (strictly serial)

fmt --check 0, `clippy --workspace --all-targets -D warnings` 0, then
`cargo test --workspace` with no background mutation run active:
36 suites, 518 passed, 0 failed (04:00:56Z-04:08:57Z, load average ~32.8).
Commit, push, remote verification and the gate were four separate tool
invocations; `origin/main` was fetch-verified equal to HEAD before the
gate was issued.

## Slice scope

Test-only, +109 lines in `crates/neko-session/src/lib.rs`, no semantics
change. 49 hand-written mutants over `checked_total`, `watermark`,
`validate_context`/`context_ok`, `insert` (empty range, end/offset
overflow, stream and byte limits, offset jump, reorder window, overlap
detection, existing-coverage walk, merge and hole rejection) and
`transition`/`mark_in_flight`/`mark_uncertain`/`confirm_received` were
probed with the whole neko-session package (3 suites) as the oracle:
40 killed, 9 equivalent. Three new tests kill 7 survivors (byte/jump/
reorder inclusive boundaries, empty range rejection, fully contained
duplicates in Unsent and InFlight segments, repeated in-flight
transition, path-generation regression alone and inside a mixed
migration); the equivalence reasons for the other 9 are in the commit
message.

Three points of the slice rest on observations made before the tests were
written, in a throwaway probe module inside the scratch worktree
(`/tmp/nmSESS`, not committed): the exact byte budget, jump and reorder
boundary outcomes, the contained-duplicate merges, the repeated in-flight
transition error, and that a key-phase regression below the segment
context is reported as `OldEpoch` by the ledger-wide check rather than by
the segment-level one. The probe module was removed with the worktree.

## Evidence boundary

Developer-local execution only; not reviewer-local, hosted CI, WAN, or
performance evidence. No production source changed. No H-I4-119, D019,
release-flag or policy-value change. `READY_LIVE: none`.
