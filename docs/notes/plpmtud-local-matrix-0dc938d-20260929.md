# PLPMTUD local netns+tc matrix — exact `0dc938d`, binary `b443ad9905c2`

One bounded, host-local, self-owned matrix run of the opt-in `--plpmtud` probe
exchange. Grid: family(v4/v6) × path(router-icmp at base/mid, clean,
silent-blackhole at 1400) × size-selective loss(0/1/5%) × session(short/long) —
48 cells, plus the DF-off control and the IPv6×1278 N/A cell. Artifacts:
`artifacts/plpmtud-local-matrix/20260929-0dc938d/` (per-cell client/server logs,
`summary.json`, `summary.md`). Run window 2026-09-29 19:19–19:26 local
(369 s), Linux 6.8, load ~7–10.

This is plumbing and search-behaviour evidence under controlled topology. It is
**not** a WAN-path result, a policy decision, or release authorization. The
VPS changed-hypothesis run (blocked on the host-key and Meta-TUN decisions) is
still the missing mainline-1 evidence.

## Headline

- **48/48 cells, zero false successes.** No DF-on cell ever confirmed an MTU
  above its path's true bottleneck.
- 26 cells `converged_exact` (v4: rc1400/rc1500/sb1400; v6: rc1500/sb1400) — the
  search lands on the bottleneck (or the 1500 ceiling) exactly.
- 18 cells (v4-rc1278, v6-rc1298, v6-rc1350 families) end with
  `ProbeTimeout::Converged` **silently**: every probe above the true base times
  out twice, the upper bound folds down to the confirmed base, and the loop
  stops. Semantically correct (confirmed stays at the base = the true
  bottleneck), but that exit path emits no event and prints no
  `plpmtud_converged` line, so the summary classifies them `client_exit_0` with
  `converged=None`. **Diagnostic gap, not a bug**; a follow-up slice should emit
  an explicit `plpmtud_converged_by_timeout` event + line.
- 4 cells `deadline_bounded` (sb1400-long, all losses): the search needs more
  than the long-session budget under the 1400 black hole; bounded exit, no
  false success. Recorded, not retried.
- 1 cell (v6-rc1298-l0-short) exit 2: the **server** logged `handshake timeout`
  during matrix teardown churn; the same config passes in its other five cells.
  Harness timing flake, recorded as-is.
- Control (DF-off on a 1278 router-icmp path): **PASS** — sizes 1389/1445/1500
  are all delivered from the *second* send (false success without DF), while the
  DF-on 1500-byte datagram is never delivered. This is the D067 measured
  behaviour reproduced under the frozen binary's own topology.
- IPv6×1278: **N/A** — the bottleneck link loses its global IPv6 addresses
  (count 0), matching D067.

## Loss-independence detail worth keeping

Loss is size-selective (only datagrams above the traffic base: v4 IP>1278, v6
IP>1250 payload-len, A→B direction). Under 5% probe-path loss every
`rc1400/rc1500/sb1400` cell still converges exactly (a few extra retries at
most): the outcome never crossed the bottleneck, and no cell confirmed above
the truth. The Session layer (all exchanges ≤ base size) was never the failure
source — v4-rc1500 and v6-rc1500 handshake/exchange/echo are loss-free by
construction, and their cells show 0 ICMP and 8/8 acks.

## Follow-up: the silent-exit cells are now explicit (0522e14)

85461 approved `plpmtud_converged_by_timeout` as an independent slice. Exact tree
`0522e14` (gate `check-gate-0522e14-20260929.md`, first-attempt green) makes the
`ProbeTimeout::Converged` arm emit the event and print the same
`plpmtud_converged` line as the acked path. The three silent families were rerun
with the rebuilt binary (`efe5d922…`): **18/18 `converged_exact`**, each at its
true bottleneck (v4-rc1278 → 1278, v6-rc1298 → 1298, v6-rc1350 → 1350); DF-off
control PASS and IPv6×1278 N/A unchanged. A hand mutant deleting the event
regresses the summary to `conv=None / client_exit_0`, so the classification no
longer depends on manual log reading. Rerun artifacts:
`artifacts/plpmtud-local-matrix/20260929-0522e14-3fam/`.

## Provenance notes

- The matrix script and the ProbeOutstanding resend fix are exact-tree
  `0dc938d` (gate: `check-gate-0dc938d-20260929.md`, second-attempt green, one
  recorded load flake). The script pins `FROZEN_SHA=b443ad99…`, which equals the
  fingerprint rebuilt from that tree; the run refused-to-run guard verified it.
- Commit `0dc938d` has an external-provenance question pending with the
  administrator (second inquiry via 85461). The session audited the diff
  line-by-line, agrees with the fix, and ran the missing literal-SHA gate
  itself; see the gate note for the audit statement.
- The 18 silent-exit cells were classified by reading their full event logs
  (e.g. `v4-rc1278-l0-short`: 14 sent, 0 acked, 7 sizes × 2 attempts, upper
  folding 1389→1333→…→1279→1278=confirmed). The v6-rc1350 family does reach its
  truth via ACKs (1333, 1347, 1350 acked) before the final 1351 timeouts fold
  the bound — same silent exit, confirmed=1350, again the true bottleneck.
