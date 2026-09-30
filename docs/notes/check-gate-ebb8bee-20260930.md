# Exact-tree check.sh provenance — ebb8bee (2026-09-30 UTC/local)

Developer-local exact-tree gate on the pushed SHA for the HK survey
forward-fix (85461's three cited deviations, items a+b).

- **SHA:** `ebb8beef1fa60463978323e315da5af5afba2eb6`
  (`docs(notes): hk survey forward-fix — alias all interface names … Boundary
  deviations section …`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** main worktree; HEAD asserted equal to the literal SHA;
  `git status --porcelain` empty before and after
- **UTC start:** 2026-09-30T11:45:17Z; **end:** 2026-09-30T11:51:16Z
  (load average 13.60)
- **Exit code:** 0, first attempt. 43 suites, 745 passed, 0 failed.
- **OS / arch:** Linux 6.8.0-137-generic x86_64; **Rust:** `rustc 1.98.0
  (88d9e12ae 2026-08-18)`

## What this gate covers

- `docs/notes/plpmtud-hk-survey-executed-20260930.md` — all interface names
  aliased (first/second overlay member, small-MTU link, far-side subnet, one
  stale-member wording); the "Boundary deviations" section records the two
  operator violations as-is (pre-run OSPF Full=5 ran anyway; sudo used on the
  endpoint), plus the standing correction (no further sudo or probing on the
  endpoint).
- `docs/notes/plpmtud-hk-survey-plan-20260930.md` — one aliasing fix
  (the always-on local TUN).
- Leak verification performed before commit: across every file added since
  `13f6712`, `git grep` for interface names (first/second overlay member
  names, the small-MTU link, proxy members, the physical NIC, the local TUN
  product name) and for all real addresses/public IP/subnets returned zero
  matches.

No source or harness change. History untouched (forward-fix only).
