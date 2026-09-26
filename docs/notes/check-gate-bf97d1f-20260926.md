# Exact-tree check.sh provenance — bf97d1f (2026-09-26)

Developer-local exact-tree gate on the final reachable pushed SHA for the
neko-cli `main.rs` pure-logic boundary-pin slice.

- **SHA:** `bf97d1f5e0ac2efc33b97709e3a22590c40fb85f`
  (`test(cli): pin capability report, usage text and pre-connect argument bounds`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmCG`) created from the
  literal pushed SHA `bf97d1f` (worktree head asserted equal to it);
  `git diff --check` exit 0; `git status --short` empty before and after
- **UTC start:** 2026-09-26T13:33:07Z (log `/tmp/nm-cli2-gate.log`, 1135 lines)
- **UTC end:** 2026-09-26T13:39:19Z
- **Exit code:** 0 (host load average ~9.1 from unrelated jobs)
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)`

## Pre-commit sequence (strictly serial)

`cargo fmt --all --check` 0, `clippy --workspace --all-targets -D warnings` 0,
then `cargo test --workspace`: 38 suites, 618 passed, 0 failed
(13:28:34Z-13:32:38Z, load average ~9.5). The suite count rose from 37 to 38
because a new integration test target was added. Commit, push, remote
verification and the gate were separate tool invocations; `origin/main` was
fetch-verified equal to HEAD before the gate. The gate was issued **alone**,
asserting the literal SHA.

## Slice scope

Test-only, additions only: +28 lines in `crates/neko-cli/src/main.rs` (two unit
tests) and a new 281-line integration suite
`crates/neko-cli/tests/capabilities.rs`. No semantics change.

55 hand-written mutants over the capability report, the usage text, the argument
parsing and the small pure helpers. **50 killed, 3 invalid constructions, and 2
not killed.** The pre-existing coverage left the whole capability report, the
usage text and every pre-connect argument bound free; the earlier probe against
the pre-existing tests alone killed only 9 of the 55.

Oracle: `cargo test -q -p neko-cli --bin neko-cli --test capabilities` - unit
tests plus the new integration target, ~1-2s per run. The integration suite
drives the real binary via `CARGO_BIN_EXE_neko-cli`, which is required because
every rejection exits from inside the process (`fail()`), and this avoids the
235s known-fragile full-target run. Baseline measured green in a separate
worktree (`/tmp/nmCb`).

## The two mutants that are NOT killed (stated plainly)

`DI_MIN` and `DI_MAX` - the experiment-id length window `8..=72`. Both mutants
widen the window (accepting 1..7, or 73). The only reachable rejection path is
the server's datagram loop, which needs a full authenticated UDP negotiation and
admission sequence before any `emit_diagnostic` runs; a junk datagram is instead
refused earlier with "incompatible negotiation", which was measured rather than
assumed. The guards are present in source but **no test in this slice witnesses
them**. Closing them would need the traffic harness in `tests/probe.rs` (a
separate, known-fragile oracle). Recorded as a known residual, not claimed as
pinned.

The three invalid constructions: `C_cargo_version` is a no-op mutant (my error -
the replacement was identical to the original, so its "survival" is
meaningless), and `C_port_max_field` / `CF_RETIRE_PACKET` each change a format
string's argument count and do not compile.

## Three of my own test bugs, found and fixed before the final run

1. `parse` reads the FIRST occurrence of a flag, so passing `--port 40079`
   alongside the default `--port 40080` silently shadowed the value under test
   and the rejection never happened. The helpers now build each flag exactly
   once (`over` replaces rather than appends), with a comment recording why.
2. "invalid hex" was asserted for an odd-length key, but odd lengths take the
   separate `hex length` branch; both branches are now asserted distinctly.
3. An empty key is even-length (zero), so it reaches the 32-byte check rather
   than the length check.

Each correction changed the test files, so the whole probe was re-run against
the corrected, formatted content and the pre-probe snapshots diffed clean
against the committed files (absolute paths). The probe results recorded here
are that final run.

## Evidence boundary

Developer-local execution only; not reviewer-local, hosted CI, WAN, or
performance evidence. No production source changed. No H-I4-119, D019,
release-flag or policy-value change. `READY_LIVE: none`.
