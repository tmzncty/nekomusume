# Exact-tree check.sh provenance — a9ddc7d (2026-09-30 UTC/local)

Developer-local exact-tree gate on the pushed SHA for the fold-to-base false
success fix (fix 3, 85461-directed).

- **SHA:** `a9ddc7de82b76163348267027939e227dad75b9a`
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** HEAD asserted equal to the literal SHA; `git status
  --porcelain` empty before and after
- **UTC start:** 2026-09-30T14:32:06Z; **end:** 2026-09-30T14:38:12Z
  (load average 7.52)
- **Exit code:** 0, first attempt. 43 suites, **750 passed** (+3 fix-3 model
  tests), 0 failed.
- **OS / arch:** Linux 6.8.0-137-generic x86_64; **Rust:** `rustc 1.98.0
  (88d9e12ae 2026-08-18)`

## Live verification in this slice (host-local, privileged topology only)

Two netns/veth topologies were created with local `sudo -n ip …` (host-local
only — the no-sudo/no-probe boundary applies to `tmzn-hk`, not this host;
both were fully removed and verified: netns deleted, both veth ends gone,
ports free):

1. **C4 shape (egress MTU 1000, base 1278):** first probe 1389 → local
   EMSGSIZE → kernel `IP_MTU` reports 1000 → `plpmtud_failed transport=udp
   reason=base_incompatible`, exit 2. Under `e9fe890` the same shape had
   reported `converged 1278` exit 0 — the false success 85461 identified.
2. **Regression (egress MTU 1500):** 8 probes, `converged confirmed_mtu=1500`,
   exit 0 — the healthy path is undamaged.

Model tests (17 plpmtud total) pin both fix-3 mechanisms end to end,
including the 85461-required assertion that the C4 shape's terminal outcome is
`BaseIncompatible`, never a converged at the unverified base.

## Corrections recorded in the same commit

- `plpmtud-emsgsize-loop-fix-20260930.md`: the "only true value below a 1000
  path" sentence is replaced by an explicit statement that the fix-2 outcome
  was a false success closed by fix 3 (with the before/after table).
- `docs/status.md` live-PMTUD row: no "C4 solved" claim; the row states the
  below-base shape is fixed and verified **host-locally** only, no live
  small-MTU sample exists, and the policy values remain undecided (measured
  facts recorded, no value chosen).

Intermediate stumbles recorded honestly: a first client patch used a `libc`
`unsafe` helper and was rejected by `-F unsafe-code` (replaced by the existing
rustix `pmtu_socket::kernel_path_mtu` adapter, whose stale "diagnostic only"
doc comment was updated); two first-draft model tests mis-stated their
premises (Some(1200) against base 1278 is immediately BaseIncompatible; the
timeout fold loop needed the outstanding-retry re-entry) and were corrected
before commit — neither broken version was ever pushed.

**Correction (2026-09-30 22:42 host-local, forward fix; history untouched).**
The note above says the netns topologies were fully removed. That was wrong at
the time it was written: `ip netns list` still held `plm-f3, plm-f3b`
(created 2026-09-30 22:30 host-local time; no processes inside, veths already gone, but the netns
entries themselves survived the teardown commands). All leftover entries were
deleted with `sudo -n ip netns del` and `ip netns list` was re-run and returned
empty (command output, not memory: `NETNS_EMPTY`). "Cleanup complete" claims
are command-verified from this point on; the raise-probe live script self-fails
if any `plm-*` netns remains after teardown.

