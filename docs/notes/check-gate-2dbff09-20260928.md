# Exact-tree check.sh provenance — 2dbff09 (2026-09-28 UTC/local)

Developer-local exact-tree gate on the final reachable pushed SHA for the matrix-probe
argument-rejection slice.

- **SHA:** `2dbff09e40dffc01899e9edab5e50b1619fc8750`
  (`test(cli): pin the exact reason for every probe --matrix argument rejection`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmVbG`) created from the literal
  pushed SHA `2dbff09` (worktree head asserted equal to it); `git diff --check` exit 0;
  `git status --short` empty before and after
- **UTC start:** 2026-09-28T13:32:00Z (log `/tmp/nm-vb-gate.log`, 1258 lines)
- **UTC end:** 2026-09-28T13:38:24Z
- **Exit code:** 0 (host load average ~4.7 from unrelated jobs)
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)`

Local and UTC dates are both 2026-09-28.

Passed on the **first** attempt; the load-sensitive
`scripts/bench/run-periodic-command-test.py` step printed `periodic command tests: ok`.

## Pre-commit sequence (strictly serial)

`cargo fmt --all --check` 0, `clippy --workspace --all-targets -D warnings` 0, then
`cargo test --workspace`: 42 suites, **718 passed** (one new test), 0 failed
(2026-09-28T13:27:06Z-13:31:21Z, load average ~4.1). Commit, push, remote verification
and the gate were separate tool invocations; `origin/main` was fetch-verified equal to
HEAD before the gate. The gate asserted the **literal** SHA.

## Work-history disclosure

This slice was prepared in an earlier round that was interrupted after the baseline
probe and a partial test run, but before the after-probe and commit. The work was left
staged in the main checkout and in the scratch worktree `/tmp/nmVb`. This round
confirmed it was this session's own work from the scratch artifacts
(`/tmp/vb-mutants.json`, `/tmp/vb-*oracle.sh`, `/tmp/vb-base.log`). The staged file,
the scratch copy and the snapshot were byte-identical (md5
`d51ecb6f1e1472ae3a753929cb5741a9`). This round then **re-ran everything** before
committing: fmt, clippy, the new test, the after-probe and the full workspace test. The
interrupted round's partial workspace log (24 suites) is **not** used as evidence.

## Slice scope

Test-only, **additions only: +109 lines** (0 deletions) in
`crates/neko-cli/tests/capabilities.rs`: one new test,
`matrix_probe_argument_rejections_name_the_exact_reason` (21 cases, socket-free,
0.04s).

### The gap

The networked test `matrix_probe_distinguishes_invalid_failed_and_reachable_outcomes`
in `probe.rs` feeds many invalid argument lists to `probe --matrix`, but for them
asserts only exit code 2 and empty stdout. So every rejection reason was
interchangeable.

The new test asserts exit 2, stderr exactly `neko: <reason>\n` and empty stdout. It
covers:

- duplicate flags
- missing values, including a following `--flag` not being taken as a value
- unknown arguments
- invalid or absent `--target`, `--transport` and `--ip-version`; absent must be
  refused, not defaulted
- unparseable `--timeout-ms` / `--bytes`
- three `reachability::validate` refusals forwarded verbatim

Every case is rejected during argument parsing, before any socket is opened.

### Mutants (14)

| Mutant | Change |
|---|---|
| `X_dupmatrix_msg` / `X_dupjson_msg` | swap the two duplicate-flag messages |
| `X_dupopt_msg` | `duplicate {option}` → fixed `duplicate argument` |
| `X_missval_msg` | `missing value for {option}` → fixed `missing value` |
| `X_missval_prefix` | drop the `!starts_with("--")` filter (a following flag is taken as the value) |
| `X_unknown_msg` | drop `{option}` from `unknown matrix argument` |
| `X_target_msg` / `X_transport_msg` / `X_version_msg` | cross-wire the three required-flag messages |
| `X_transport_none` / `X_version_none` | an absent flag silently defaults instead of failing |
| `X_timeout_msg` / `X_bytes_msg` | swap the two parse-error messages |
| `X_validate_msg` | replace the forwarded `reachability::validate` error with a fixed string |

- **Baseline, unmodified tests:** **14/14 SURVIVED**. The oracle was `--bin neko-cli`
  plus the `capabilities`, `human_reports`, `lab_json`, `fixtures_json` and
  `health_observe_json` suites, then the `probe.rs` matrix test. `--list` confirmed the
  `matrix_probe` filter selects
  `matrix_probe_distinguishes_invalid_failed_and_reachable_outcomes`, so the baseline
  really exercised the existing matrix coverage.
- **After, oracle = the new test:** **14/14 killed** (rc=101), 0 BADANCHOR, 0
  COMPILE_ERR, no equivalents.

## Recorded, not pinned

Range guards on the failover, endpoint-rebind and periodic paths remain unasserted
(for example `TCP delivery delay outside 0-1000 ms`, `readiness delay outside 0-3000`,
`promotion delay outside 0-1000 ms`, and the periodic `setup delay` / `ack delay`
guards). They need live-socket harnesses. Separately, `multistream.rs` has no
kill-on-drop guard: a panic between spawn and `bounded_wait_with_output` can orphan a
listening server. This was observed once during mutation probing and is not fixed
here.

## Evidence boundary

Developer-local execution only; not reviewer-local, hosted CI, WAN, or performance
evidence. No production source changed. No H-I4-119, D019, release-flag or policy-value
change. `READY_LIVE: none`.
