# Exact-tree check.sh provenance — 5cc4a6f (2026-09-25)

Developer-local exact-tree gate on the final reachable pushed SHA for the
Reno window / PTO probe-deadline-persistent-congestion edge pins.

- **SHA:** `5cc4a6f935d67b9bc8d2a9dfbfe91fdf048f61fd`
  (`test(reliable): pin Reno window transitions and PTO probe/deadline/persistent-congestion edges`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmRN`) at the pushed SHA;
  `git status --short` empty before and after the run; `git diff --check` exit 0
- **UTC start:** 2026-09-25T15:50:37Z (log captured in `/tmp/nmrn-check.log`)
- **UTC end:** 2026-09-25T15:56:43Z
- **Exit code:** 0
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)` (`cargo 1.98.0 (797e8a9bc 2026-08-05)`)
- **Scope:** full `scripts/check.sh`. Pre-commit local workspace run: 36
  suites, 465 tests passed, 0 failed; neko-reliable lib 43/43 including the
  three new tests.
- **Slice scope:** test-only, no semantics change. Mutation probing of
  `Reno` and `Recovery::on_pto` / `next_pto_deadline_us` on `90a50d7`
  (full workspace test suite per mutant, 19 mutants) found eleven survivors:
  `Reno::new` accepting mss 0; slow-start boundary `<` → `<=`;
  congestion-avoidance growth dropping the mss factor; loss floor lowered to
  1 MSS or removed; persistent congestion not setting ssthresh; pacing floor
  instead of ceiling division; persistent-congestion events counted with
  `>=` instead of the single `==` threshold crossing; `on_pto(0)`
  accepted; probe set not capped by `max_probe_frames`; PTO deadline
  anchored at newest instead of oldest outstanding packet. Eight other
  mutants were already killed. New tests `reno_window_transitions_are_exact`,
  `pto_probe_cap_zero_limit_and_single_persistent_congestion_event` and
  `pto_deadline_is_anchored_at_oldest_outstanding_packet` kill all eleven
  (re-verified per mutant: 42/43, one named failure each).
- **Evidence boundary:** developer-local execution only; not reviewer-local,
  hosted CI, WAN, or performance evidence. The `pto_count` reset on ACK
  (H-I4-119 maintainer semantics gate) is not asserted or changed; no D019,
  release-flag or policy-value change. `READY_LIVE: none`.
