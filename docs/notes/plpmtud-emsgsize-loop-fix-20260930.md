# on_emsgsize no-progress loop — fix and live verification (2026-09-30)

Slice: repair the `upper >= confirmed` invariant violation and the resend loop
(85461-sequenced, second queue slot after the D067 facts entry).

## Two stacked defects, both fixed

1. **Hidden invariant violation.** `on_emsgsize` clamped the EMSGSIZE ceiling
   with `.max(confirmed)`: when the real path capacity was below the search
   base, `upper` was silently pinned to `confirmed` while the caller was told
   `RetryAt(confirmed)`. Fixed: `upper` records the true ceiling; a ceiling
   below `confirmed` returns `BaseIncompatible` (fail-closed).
2. **No-progress resend loop.** With `reported_mtu=None` (the client shape:
   the socket reports no MTU value), each EMSGSIZE could only imply
   `ceiling = attempted − 1`, but the outstanding probe was left in place, so
   the client resent the same refused size forever. **Measured before the
   second fix**: a single C4-shape session (veth MTU 1000, base 1278) emitted
   **600 `plpmtud_probe_emsgsize_retry` events for one size (1389) with zero
   search movement**, ending only at the deadline. Fixed: `on_emsgsize` always
   clears the outstanding probe, so the next `start_probe()` picks from the
   narrowed interval.

## Verification

- Model tests (`crates/neko-reliable`): the old behaviour-pinning test is
  rewritten to the new invariant; new tests pin (a) below-base ceiling →
  `BaseIncompatible` with `upper` recording truth (the C4 shape as a test
  case: first 1389 probe, ceiling 1000), and (b) the None-MTU search exhausts
  in bounded steps with nothing outstanding after any EMSGSIZE. 14 plpmtud
  tests green.
- **Live re-check** (post-fix binary, host-local netns, veth egress MTU 1000,
  the same C4 shape): **7 EMSGSIZE events, 7 distinct sizes**
  (1389→1333→1305→1291→1284→1281→1279), search moving every step,
  `plpmtud_converged confirmed_mtu=1278`, exit 0. The 600-event loop was gone.
  Topology removed and verified (netns, veth, port all clean). **Correction
  (85461 review, same day): that converged-at-base outcome was itself a FALSE
  SUCCESS** — 1278 exceeds the 1000-MTU path, the base had never actually been
  transmitted, and the search had merely folded onto an unverified assumption.
  Fix 2 turned the loud loop into a quiet false success; fix 3 closes it (see
  below).

## Notes

- The first commit of this slice (`2c3e810`) fixed only defect 1; its live
  check is what exposed defect 2. Both are in the final commit; the
  intermediate state is preserved in history.
- No default behaviour change: the loop only runs behind the opt-in
  `--plpmtud` flag; `RetryAt` semantics above base are unchanged.

## Fix 3 (85461-directed, 2026-09-30) — the fold-to-base false success

**Root cause.** With `reported_mtu=None` (the client shape until this fix),
each EMSGSIZE could only imply `ceiling = attempted − 1`, so the ceiling could
never drop below the base. The search narrowed onto the configured base and
reported it as converged — but the base was an assumption, never transmitted.
A 1000-MTU path with base 1278 therefore "converged" at 1278: a false success.

**Two mechanisms, both implemented, both tested:**

1. **Kernel ceiling (primary).** On EMSGSIZE the client now reads the
   connected socket's kernel path-MTU estimate (`IP_MTU`/`IPV6_MTU` via the
   existing rustix `pmtu_socket::kernel_path_mtu` adapter) and feeds it to
   `on_emsgsize` as `reported_mtu`. A below-base value is judged
   `BaseIncompatible` in one step.
2. **Base-verification probe (defense in depth, covers the None shape).** The
   model tracks `emsgsize_seen`; a fold onto the base under EMSGSIZE evidence
   is *provisional*: `needs_base_verification()` requires one real
   base-sized probe (issued by `start_base_verification_probe()`). Its ACK
   verifies the base (genuine converged-at-base); its EMSGSIZE concludes
   `BaseIncompatible` (fail-closed exit 2). A successful ACK of any probe
   clears the flag (the path just carried a base-sized-or-larger datagram);
   `reset_generation` discards the evidence with the generation.

**Live verification (host-local netns, veth egress MTU 1000, base 1278):**

| binary | outcome |
|---|---|
| after fix 2 (`e9fe890`) | `converged confirmed_mtu=1278`, exit 0 — **false success (FAIL class)** |
| after fix 3 (this slice) | first probe 1389 EMSGSIZE, kernel reports 1000, `plpmtud_failed transport=udp reason=base_incompatible`, **exit 2** — the honest outcome |

Regression: a clean 1500-MTU topology with the same binary still converges
exactly (`8 probes, confirmed_mtu=1500`, exit 0) — the fix does not damage the
healthy path. Model tests pin the C4 shape end to end for both mechanisms
(kernel-reported ceiling → immediate `BaseIncompatible`; None shape →
provisional fold → base-verification probe's EMSGSIZE → `BaseIncompatible`;
never a plain converged at the unverified base), the ACK-verifies-base
transition, and the timeout-only fold exemption.

Topologies were created and fully removed per check (netns + both veth ends +
port verified clean). The netns/veth construction used local `sudo -n ip …`
(host-local only; the no-sudo boundary applies to `tmzn-hk`, not this host).
