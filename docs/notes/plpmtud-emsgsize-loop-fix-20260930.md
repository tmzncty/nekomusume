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
  `plpmtud_converged confirmed_mtu=1278` (= base — the only true value below a
  1000 path), exit 0. The 600-event loop is gone. Topology removed and
  verified (netns, veth, port all clean).

## Notes

- The first commit of this slice (`2c3e810`) fixed only defect 1; its live
  check is what exposed defect 2. Both are in the final commit; the
  intermediate state is preserved in history.
- No default behaviour change: the loop only runs behind the opt-in
  `--plpmtud` flag; `RetryAt` semantics above base are unchanged.
