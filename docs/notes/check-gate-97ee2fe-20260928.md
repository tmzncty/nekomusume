# Exact-tree check.sh provenance — 97ee2fe (2026-09-27 UTC / 2026-09-28 local)

Developer-local exact-tree gate on the final reachable pushed SHA for the
multistream / endpoint-rebind argument-validation boundary-pin slice.

- **SHA:** `97ee2fe41966a3966021bb8e266cf6e092aff5dc`
  (`test(cli): pin the multistream and endpoint-rebind argument validations`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmVaG`) created from the
  literal pushed SHA `97ee2fe` (worktree head asserted equal to it);
  `git diff --check` exit 0; `git status --short` empty before and after
- **UTC start:** 2026-09-27T20:36:10Z (log `/tmp/nm-va-gate.log`, 1257 lines)
- **UTC end:** 2026-09-27T20:42:31Z
- **Exit code:** 0 (host load average ~5.0 from unrelated jobs)
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)`

File name uses the **local** date (2026-09-28); the run was 2026-09-27 UTC.

Passed on the first attempt; the load-sensitive
`scripts/bench/run-periodic-command-test.py` step printed
`periodic command tests: ok`.

## Pre-commit sequence (strictly serial)

`cargo fmt --all --check` 0, `clippy --workspace --all-targets -D warnings` 0, then
`cargo test --workspace`: 42 suites, **716** passed (three more than before), 0
failed (2026-09-27T20:31:31Z-20:35:44Z, load average ~4.5). Commit, push, remote
verification and the gate were separate tool invocations; `origin/main` was
fetch-verified equal to HEAD before the gate. The gate was issued **alone**,
asserting the literal SHA.

## Slice scope

Test-only, **+218/-1** across two test files
(`crates/neko-cli/tests/capabilities.rs`, `crates/neko-cli/tests/multistream.rs`).
The one deletion is the previous weak assertion `contains("streams outside")`,
replaced by the full text - coverage strictly increases. No source change.

27 hand-written mutants; **26 killed, 1 survivor (proven unreachable)**, 0 HANGs, 0
BADANCHOR, 0 COMPILE_ERR.

## The two clusters

**1. `multistream`'s whole argument surface.** `streams outside 1-16`,
`records outside 1-64`, `bytes outside 1-1024`, `port outside 40080-40100`,
`invalid --addr`, `--client-key is required`, `--server-key is required`,
`mode must be server or client`, `key must be 32-byte hex`, `invalid key hex` and
`--identity is required` had **zero** occurrences in the test tree. The one
pre-existing assertion checked only the prefix `streams outside` plus the exit code,
so even the numbers in that message were free; it is strengthened to the full text.
Boundary values inside each window are shown to be accepted (the run reaches the
next requirement), which separates inclusive from exclusive bounds.

**2. `endpoint-rebind-server` / `-client`'s clique.**
`endpoint rebind requires count=2, bytes=1-1200, duration=1-30` is a **five-clause**
condition with one message, emitted by **both** sides, and had no witness. Each
clause is exercised on its own for both sides; both `--bytes` edges (1 and 1200) are
shown inside; and the client's later guards `invalid UDP port` / `bad UDP target` are
reachable only once the clique holds, so they also pin the guard **order**.

All of this validation runs before any socket exists, so the tests need no peer and
no port.

## Deliberately NOT covered (recorded, with the reason)

The `failover-server` delay/cessation guards (`invalid TCP delivery delay`,
`TCP delivery delay outside 0-1000 ms`, `invalid UDP reply cessation point`,
`UDP reply cessation point outside 1..count`) sit **after** its bind, so covering them
means binding a port. Binding here would risk colliding with the socket tests that
lease ports 40080-40100 **from a different test binary** (the ephemeral range
32768-60999 covers that window, so a `:0` bind can land on a leased port during the
release-then-rebind gap). That risk is not worth the coverage, so those messages
remain unwitnessed and are recorded rather than silently skipped.

## The one survivor: provably unreachable

`total payload outside 1 MiB` in `multistream`'s `bounds`. With `streams <= 16`,
`records <= 64` and `bytes <= 1024`, the largest product is
`16 * 64 * 1024 == 1 << 20`, which the check `<= 1 << 20` accepts - the
individually-legal maxima sum **exactly** to the ceiling, so the clause can never
fire and `checked_mul` can never overflow. It is left in place as a defence that
would become live if those maxima changed, and the inline unit test
`bounds_are_inclusive_and_the_total_is_one_mib` already asserts that identity. Recorded
as **unreachable with current constants**, not as an equivalence.

## Evidence boundary

Developer-local execution only; not reviewer-local, hosted CI, WAN, or performance
evidence. No production source changed. No H-I4-119, D019, release-flag or
policy-value change. `READY_LIVE: none`.
