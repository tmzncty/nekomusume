# Exact-tree check.sh provenance — 4fdde89 (2026-09-30 UTC/local)

Developer-local exact-tree gate on the pushed SHA for the HK real-path survey
evidence write-back (docs + artifacts + status row).

- **SHA:** `4fdde898b990d585fe34354ee0d4c6b50d372b2f`
  (`docs(status): live PMTUD row — hk real-path survey evidence write-back …`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** main worktree; HEAD asserted equal to the literal SHA;
  `git status --porcelain` empty before and after
- **UTC start:** 2026-09-30T11:35:06Z; **end:** 2026-09-30T11:41:05Z
  (load average 16.57 — high; green regardless)
- **Exit code:** 0, first attempt. 43 suites, 745 passed, 0 failed.
- **OS / arch:** Linux 6.8.0-137-generic x86_64; **Rust:** `rustc 1.98.0
  (88d9e12ae 2026-08-18)`

## Sequencing corrections (both recorded honestly)

1. The first write-back attempt (`6b32181`) committed the executed note and
   sanitized artifacts but **not** the status row: the row-replacement script
   asserted on a stale copy of the row text (the row had been rewritten by the
   administrator's 2026-09-30 `13f6712` resolution write-back), failed with
   AssertionError, and because the python block and the git commands were not
   joined by `&&`, the commit proceeded anyway. Detected immediately; fixed by
   `4fdde89` which replaces the row against its actual current text. No wrong
   content was ever pushed.
2. A first gate attempt for this record was aborted before running: the gate
   note itself had been written to the tree first, so the clean-tree
   precondition failed silently (the `&&` chain stopped at PRE check). The
   note was removed, the gate run on the clean literal SHA as recorded above,
   and this note written afterwards.

## What this gate covers

- `docs/notes/plpmtud-hk-survey-executed-20260930.md` — executed-survey
  evidence note (aliases only).
- `artifacts/plpmtud-hk-survey/hk-survey-20260930A/` — per-cell client
  outputs, per-cell summary JSON, oracle logs (sanitized to aliases; a
  post-sanitization scan found zero remaining real addresses).
- `docs/status.md` live PMTUD row — `BLOCKED_ENVIRONMENT` →
  `PARTIAL_EVIDENCE_SELF_OWNED` with the survey results and the two named
  missing pieces (live small-MTU blackhole path; policy candidates not yet in
  decisions.md).

No source or harness change in either commit; the survey ran against the
frozen gated binary `efe5d922…` verified on both endpoints.
