# Exact-tree check.sh provenance — db8af8b (2026-09-27 local / 2026-09-26 UTC)

Developer-local exact-tree gate on the final reachable pushed SHA for the
neko-reliable simulation-region boundary-pin slice.

- **SHA:** `db8af8b5a9ffb8f3016fa1866cadbc396ba345fd`
  (`test(reliable): pin simulation bounds, error class and the exact drop period`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmRG`) created from the
  literal pushed SHA `db8af8b` (worktree head asserted equal to it);
  `git diff --check` exit 0; `git status --short` empty before and after
- **UTC start:** 2026-09-26T17:36:32Z (log `/tmp/nm-rel-gate.log`, 1174 lines)
- **UTC end:** 2026-09-26T17:42:44Z
- **Exit code:** 0 (host load average ~10.8 from unrelated jobs)
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)`

## Pre-commit sequence (strictly serial)

`cargo fmt --all --check` 0, `clippy --workspace --all-targets -D warnings` 0,
then `cargo test --workspace`: 38 suites, 657 passed, 0 failed
(2026-09-26T17:32:00Z-17:36:05Z, load average ~10.6). Commit, push, remote
verification and the gate were separate tool invocations; `origin/main` was
fetch-verified equal to HEAD before the gate. The gate was issued **alone**,
asserting the literal SHA.

## Slice scope

Test-only, **additions only: +47, -0** in `crates/neko-reliable/src/lib.rs` (two
tests in `tests`). No semantics change.

38 hand-written mutants over `PacketNumbers`, `VirtualClock`,
`simulate_fault_profile`, `simulate_delivery` and `FecConfig::validate`;
**31 killed, 7 equivalent, 0 HANGs, 0 invalid constructions, 0 BADANCHOR**. The
pre-existing tests left 13 survivors; the two new tests close 6. Baseline
measured green in a separate worktree (`/tmp/nmRb`).

## The seven equivalences (all measured, in a draft worktree since removed)

- `FP_no_reorder` / `SD_no_reorder`: the `reorder` flag is **inert**. Measured by
  sweeping 72 (frames, period, profile) combinations and flipping the flag -
  **0 differing results**. Both simulators collect arrivals into a `BTreeSet`, so
  arrival order is discarded before it can be observed. Stated plainly: the
  `reorder` parameter currently cannot affect any output of these functions. That
  is a limitation of the result shape (there is no order field), recorded here
  rather than claimed as pinned.
- `FP_rounds_cap_off` / `FP_rounds_plus1` / `SD_rounds_cap_off`: the `Capacity`
  round cap cannot bind. Measured maximum rounds across the sweep is **2**,
  because each frame is retried at most once (after its first attempt it is no
  longer "first"), while the cap is `total_frames + 2`. Unreachable in both
  forms.
- `FP_burst_len_zero_off`: subsumed - with `burst_len == 0` the window test
  `frame >= start && frame < start + 0` is already false. Measured: identical
  results for `burst_start` 0 and 3 at `burst_len` 0.
- `FP_no_advance`: the `VirtualClock` is local to the function and never read;
  `advance` can only fail on `u64` overflow at one microsecond per round, and its
  value is not part of `SimulationResult`.

All scratch experiments were removed and the worktree md5-verified against the
snapshot before the final probe.

## Evidence boundary

Developer-local execution only; not reviewer-local, hosted CI, WAN, or
performance evidence. No production source changed. No H-I4-119, D019,
release-flag or policy-value change. `READY_LIVE: none`.
