# Exact-tree check.sh provenance — 0dcd7aa (2026-09-26)

Developer-local exact-tree gate on the final reachable pushed SHA for the
neko-crypto boundary-pin slice.

- **SHA:** `0dcd7aacbd4e2d90c562e843a2ce192751dc0235`
  (`test(crypto): pin nonce, replay window, trust, context and preauth budget edges`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmCRG`) created from
  `git rev-parse origin/main` (asserted equal to local HEAD, to the literal
  pushed SHA, and to worktree HEAD); `git diff --check` exit 0;
  `git status --short` empty before and after
- **UTC start:** 2026-09-26T06:05:45Z (log `/tmp/nmcr-gate.log`)
- **UTC end:** 2026-09-26T06:11:51Z
- **Exit code:** 0 (host load average ~11.9 from unrelated jobs)
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)` (`cargo 1.98.0 (797e8a9bc 2026-08-05)`)

## Pre-commit sequence (strictly serial)

fmt --check 0, `clippy --workspace --all-targets -D warnings` 0, then
`cargo test --workspace`: 36 suites, 543 passed, 0 failed
(05:57:06Z-06:05:04Z, load average ~15.5), no re-run needed. Commit, push,
remote verification and the gate were four separate tool invocations;
`origin/main` was fetch-verified equal to HEAD before the gate was issued.

## Slice scope

Test-only, +335 lines in `crates/neko-crypto/src/lib.rs`
(`mod boundary_tests`), no semantics change. 46 hand-written mutants over
NonceManager, ReplayWindow, TrustPolicy::authorize, RecordContext::encode,
prologue/bound_prologue and PreauthBudget were probed with the whole
neko-crypto package (2 suites, 37 tests) as the oracle: 42 killed, 4
equivalent. Six new tests kill 21 survivors; the equivalence reasons for
the other 4 are in the commit message.

Canonical outcomes were measured first in a throwaway probe module inside
the scratch worktree (`/tmp/nmCR`, not committed). Two witnesses were
corrected there because a second budget dimension masked the one under
test: the response amplification witness needed the input split across
four packets (otherwise the response packet allowance rejected first), and
the rollback witness needed a charged packet present with an unbacked byte
amount (otherwise the packet counter rejected first). One C-class mutant
was manually re-checked in a second scratch worktree (`/tmp/nmCRchk`) to
confirm the probe's kill attribution.

No cross-crate re-run was done for the 4 survivors because none is argued
killable: the three byte-order mutants keep `encode` injective over the
comparison, and the prologue length-field swap stays injective in
(domain, binding). The cross-crate consumers (neko-cli probe.rs and
multistream.rs) only assert that flipping one negotiation-binding byte
must fail, which holds under any injective encoding.

## Evidence boundary

Developer-local execution only; not reviewer-local, hosted CI, WAN, or
performance evidence. No production source changed. No H-I4-119, D019,
release-flag or policy-value change. `READY_LIVE: none`.
