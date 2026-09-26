# Exact-tree check.sh provenance — def0e15 (2026-09-26)

Developer-local exact-tree gate on the final reachable pushed SHA for the fourth
neko-cli boundary-pin slice (framed.rs, the shared frame reader).

- **SHA:** `def0e1575803b4e3daf2d96f39a7ae0b0a879c90`
  (`test(cli): pin staged frame reader deadline/eof parity, exact-length frames, reader reuse and set_max_frame_len`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmFG`) created from the
  literal pushed SHA `def0e15` (worktree head asserted equal to it);
  `git diff --check` exit 0; `git status --short` empty before and after
- **UTC start:** 2026-09-26T08:40:53Z (log `/tmp/nm-fr-gate.log`, 1088 lines)
- **UTC end:** 2026-09-26T08:47:00Z
- **Exit code:** 0 (host load average ~15.1 from unrelated jobs)
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)`

## Pre-commit sequence (strictly serial)

`cargo fmt --all --check` 0, `clippy --workspace --all-targets -D warnings` 0,
then `cargo test --workspace`: 36 suites, 583 passed, 0 failed
(08:36:08Z-08:40:07Z, load average ~10.8). Commit, push, remote verification and
the gate were separate tool invocations; `origin/main` was fetch-verified equal
to HEAD before the gate. The gate was issued **alone**, asserting the literal
SHA.

## Slice scope

Test-only, +349 lines in `crates/neko-cli/src/framed.rs` (seven tests added to
the existing `mod tests` plus a `pinned` clock helper), no semantics change.

35 hand-written mutants; **31 killed, 4 equivalent, 0 invalid constructions and
0 BADANCHOR**. The pre-existing eight tests left 21 survivors; the new tests
close 17.

Anchor discipline for this file: `read_until_staged_with_clock` and
`read_until_with_clock` are near-duplicates, so line-level anchors would match
both functions and report BADANCHOR. Every mutant instead anchors on the whole
function body (unique by construction) and edits exactly one fragment inside it,
verified as a single occurrence before the run.

The four equivalences are unreachable-by-arithmetic and were checked against
source rather than inferred from a green run:

- `payload_len >= payload.len()` for `==`: the loop reads into
  `&mut self.payload[self.payload_len..]`, so the returned count never exceeds
  the remaining space and the counter cannot overrun; equality is the only way
  the branch can fire.
- `header_len >= header.len()` for `==`: the same argument for the header
  counter.
- dropping `!self.payload.is_empty()` from `is_partial()`: `payload` is only
  made non-empty by `vec![0; len]` at the instant the header completes (or
  partially filled afterwards), both of which require `header_len == 4`, and
  `take_frame` empties it - so the first clause always holds when this one does.
- dropping `self.payload_len != 0` from `is_partial()`: `payload_len` is only
  incremented on the body branch, reachable only once `header_len == 4`, so the
  first clause subsumes it as well.

## Evidence fidelity

The recorded probe run was made against content byte-identical to the committed
file. `cargo fmt` reformatted the new tests after an earlier probe, so the
probes were re-run on the formatted file, and the pre-probe snapshot was then
diffed against the committed file (clean). A first commit attempt failed with
`could not read log file '/tmp/fr-commit-msg.txt'` despite the file existing
immediately afterwards - a write-visibility delay, not a content problem; the
commit was retried and the tree was clean and correctly staged beforehand.

## Evidence boundary

Developer-local execution only; not reviewer-local, hosted CI, WAN, or
performance evidence. No production source changed. No H-I4-119, D019,
release-flag or policy-value change. `READY_LIVE: none`.
