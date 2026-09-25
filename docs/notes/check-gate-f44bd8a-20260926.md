# Exact-tree check.sh provenance — f44bd8a (2026-09-26)

Developer-local exact-tree gate on the final reachable pushed SHA for the
AckRanges / PLPMTUD / FEC boundary pins.

- **SHA:** `f44bd8a9657353bafd1e0d25c4852f435fc650a3`
  (`test(reliable): pin AckRanges, PLPMTUD and FEC boundary behaviour`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmAPF`) at the pushed SHA;
  `git status --short` empty before and after the run; `git diff --check` exit 0
- **UTC start:** 2026-09-25T20:20:14Z (log captured in `/tmp/nmapf-check.log`)
- **UTC end:** 2026-09-25T20:26:23Z
- **Exit code:** 0
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)` (`cargo 1.98.0 (797e8a9bc 2026-08-05)`)
- **Scope:** full `scripts/check.sh`. Pre-commit local workspace run: 36
  suites, 472 tests passed, 0 failed; neko-reliable lib 50/50 including the
  seven new tests.
- **Slice scope:** test-only, no semantics change. Mutation probing on
  `cdea289` of `AckRange`/`AckRanges`, `Plpmtud`, `PathMtuLimits` and
  `FecBlock` (47 mutants; neko-reliable crate tests per mutant, and the six
  AckRanges survivors additionally confirmed to survive the full workspace
  suite) found 23 survivors. Seven new tests kill 21 of them (re-verified per
  mutant). The two remaining survivors are equivalent mutants by
  construction and are deliberately not tested:
  1. `AckRanges::insert` merging with `r.end` instead of `max(last.end, r.end)`:
     insert only adds a single point not already contained, and existing
     ranges are canonical/disjoint, so an earlier range can never enclose it.
  2. `Plpmtud::acknowledge` dropping the `generation != self.generation`
     check: an outstanding probe always carries the current generation because
     `reset_generation` clears `outstanding`, so the remaining
     `probe.path_generation` check is equivalent.
- **Evidence boundary:** developer-local execution only; not reviewer-local,
  hosted CI, WAN, or performance evidence. No H-I4-119, D019, release-flag or
  policy-value change. `READY_LIVE: none`.
