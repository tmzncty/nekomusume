# Exact-tree check.sh provenance — e5fd13e (2026-10-01 UTC/local)

Developer-local exact-tree gate on the pushed SHA for the black-hole
fallback slice (85461-directed; completes the WIP found in the shared tree,
which 85461 attributed to this session's own earlier window).

- **SHA:** `e5fd13e` (full: resolved via `git rev-parse` at gate time; push
  verified `origin/main == HEAD`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** HEAD asserted at the literal SHA; `git status --porcelain`
  empty before and after
- **UTC start:** 2026-09-30T18:17:59Z; **end:** 2026-09-30T18:24:34Z
  (load average 21.32 — busy host, green regardless)
- **Exit code:** 0, first attempt. 43 suites, **758 passed** (+5 black-hole
  model tests), 0 failed.
- **OS / arch:** Linux 6.8.0-137-generic x86_64; **Rust:** `rustc 1.98.0
  (88d9e12ae 2026-08-18)`

## What this gate covers

- Model (`neko-reliable`): two authoritative black-hole triggers (kernel
  EMSGSIZE ceiling in [base, confirmed); `blackhole_threshold` consecutive
  confirmed-size losses) resetting identically — confirmed->base, fresh
  generation, fresh budget, flags cleared; EMSGSIZE path bounds the
  re-search by the clamped ceiling; `observe_confirmed_size_loss` returns
  the new generation; losses above confirmed are not evidence; progress
  resets the run; the CONFIRMED raise is allowed at `confirmed == max_mtu`
  as the RFC 8899 verification probe.
- Adapter (`neko-cli`/`pmtu_socket.rs`): `kernel_path_mtu` clamps to the
  configured ceiling at the entry (loopback 65536 pinned by test); the
  model re-clamps as belt-and-suspenders.
- CLI: `BlackholeFallback` match arm (E0004 closed) emitting
  `plpmtud_blackhole_fallback {generation, ceiling}`; raise state cleared on
  fallback; converged lines are value-change-driven (re-search prints a
  fresh line only when the confirmed value actually changes).
- Docs: `decisions.md` mechanism record; `status.md` marks the HY2 line
  prerequisites complete and the VPS rerun deferred (vps-104 expired, must
  not be used or connected to; waiting for a new VPS from the
  administrator), per 85461.

## Live verification (host-local netns, sudo ip; command-verified teardown)

1500-MTU veth path: search converged at 1500; the veth was dropped to 1300
mid-session; the next CONFIRMED verification raise was refused locally with
a kernel ceiling of 1300; the model emitted the fallback (generation 2,
ceiling 1300) and the re-search converged at 1300. Shrinkage discovered
through authoritative evidence only — no fabricated shrinkage, no silent
downgrade.

The EXIT-trap teardown pattern failed twice in this environment (left
`plm-bh` behind once here, `plm-rp` once in the raise slice); both were
caught by the immediate `ip netns list` command check and removed. Lesson
recorded: live-test teardowns in this environment call the teardown
function explicitly — traps are not trusted.

## Slice provenance

The model-layer WIP (`BlackholeFallback` result type, EMSGSIZE fallback
core) was found uncommitted in the shared tree; 85461 attributed it to this
session's earlier window and directed the completion (client arm, ceiling
clamp at adapter entry + test, model tests for threshold/EMSGSIZE fallback
and bounds, live 1500->1300 shrinkage check, gate). Two of the four
85461-specified model tests initially asserted the wrong premise (loss
fallback folding upper to base; a raw over-ceiling value triggering the
fallback — logically impossible) and were corrected to the ADR semantics
before commit; both stumbles are part of the record, not hidden.
