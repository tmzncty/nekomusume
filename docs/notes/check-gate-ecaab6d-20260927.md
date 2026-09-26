# Exact-tree check.sh provenance — ecaab6d (2026-09-27 local / 2026-09-26 UTC)

Developer-local exact-tree gate on the final reachable pushed SHA for the
neko-carrier `CarrierHealthEvidence` / `HealthEvidenceLimits` boundary-pin slice.

- **SHA:** `ecaab6d15a39d2a2d5ae721617478817fbe4b066`
  (`test(carrier): pin health-evidence event json, ring eviction and the shared transition bound`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmHG2`) created from the
  literal pushed SHA `ecaab6d` (worktree head asserted equal to it);
  `git diff --check` exit 0; `git status --short` empty before and after
- **UTC start:** 2026-09-26T21:38:41Z (log `/tmp/nm-he-gate.log`, 1192 lines)
- **UTC end:** 2026-09-26T21:44:54Z
- **Exit code:** 0 (host load average ~8.5 from unrelated jobs)
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)`

## Pre-commit sequence (strictly serial)

`cargo fmt --all --check` 0, `clippy --workspace --all-targets -D warnings` 0,
then `cargo test --workspace`: 38 suites, 675 passed, 0 failed
(2026-09-26T21:34:10Z-21:38:16Z, load average ~12.3). Commit, push, remote
verification and the gate were separate tool invocations; `origin/main` was
fetch-verified equal to HEAD before the gate. The gate was issued **alone**,
asserting the literal SHA.

## Slice scope

Test-only, **additions only: +196 lines** (0 deletions) in
`crates/neko-carrier/src/lib.rs` - three tests in `health_evidence_tests`. No
semantics change.

37 hand-written mutants over `CarrierHealthEvidence` (new, observe,
observe_event, the accessors, `json` and `health_state_name`); **37 killed, 0
survivors, 0 HANGs, 0 invalid constructions, 0 BADANCHOR**. There is **no
residual to report for this slice**. The pre-existing tests left 17 survivors;
the three new tests close all 17. Baseline measured green in a separate worktree
(`/tmp/nmH2b`).

Highest-value gaps closed: the whole `events` branch of `json` (the single
existing json assertion used an EMPTY events array, so the event key names, the
inter-event separator, the `"cause"` key, both cause strings and the progress arm
with no cause were all free); the `unknown` and `failed` arms of
`health_state_name` (neither state was ever produced by a test); `observe_event`'s
ring eviction (nothing ever pushed the event ring past capacity); the transitions
ring, which shares the SAME `max_samples` budget from both entry points; and the
"record a transition only where the state changed" rule on the event path.

## Process notes

- One invalid mutant of mine (`HealthError::UnknownPath`) did not compile - that
  enum has only `InvalidLimit` and `ResourceLimit`. It was dropped and replaced
  with a `ResourceLimit` variant mutant rather than reported as a survivor.
- Two mutant anchors initially matched zero or several times: the json literal
  `"samples":[` also occurs inside the pre-existing test's expected string. They
  were re-anchored to the full `format!(` line, which is unique in the file
  (verified `count == 1`).
- Because the test file changed during development (a wrong expected string, then
  the fixes above), the whole probe was re-run against the final formatted
  content. The pre-probe snapshot was md5-compared against the worktree **and** a
  grep confirmed a pristine-only line was present in it (the extra check added
  after the previous slice's contaminated snapshot).
- The snapshot/format/parse steps and the probe were separate, serialized tool
  calls.

## Evidence boundary

Developer-local execution only; not reviewer-local, hosted CI, WAN, or
performance evidence. No production source changed. No H-I4-119, D019,
release-flag or policy-value change. `READY_LIVE: none`.
