# R9-11A–D factual reconciliation

**Anchor:** exact source/test tree `8cbd9af` + reviewer/dev commits through `e51a630`.

This is a factual index of the reachable R9-11 review state. It is not a new
review, not a promotion, and not item-4 closure — the dedicated final
independent R9 bounded review remains open.

| Lane | Scope | State | Anchor / note |
|---|---|---|---|
| R9-11A | Session terminal-remainder correctness | CLOSED — independent no-finding | `independent-r9-11a-session-terminal-remainder-4f9dfff-20260920.md` |
| R9-11B | UDP terminal ownership + H-R9-081 closure/interval gaps | CLOSED — defects repaired at `730f993`, `3b3a918`; bounded note `dev-r9-11b-udp-terminal-ownership-20260920.md` | independent notes `…-730f993`, `…-3202796`, `…-c52efef`, `…-6fe8ae8` |
| R9-11C | replay / Carrier terminality | CLOSED — repaired at `defb2de`; bounded note `dev-r9-11c-replay-carrier-terminality-20260920.md` | `independent-r9-11c-replay-carrier-terminality-730f993-20260920.md` |
| R9-11D1 | process result truth / terminal-exit boundary | CLOSED — independent bounded no-finding | `reviewer-r9-11d1-result-truth-20260920.md` (`6164fc4`) |
| R9-11D2 | TCP/UDP listener + socket ownership | CLOSED — dev bounded no-finding | `dev-r9-11d2-socket-ownership-20260920.md` (`72c5e06`) |
| R9-11D3 | child/process-group/temp-runtime cleanup | CLOSED — dev bounded no-finding | `dev-r9-11d3-child-process-group-temp-runtime-20260920.md` (`5a4082c`) |
| R9-11D4 | shutdown / post-return false-success negatives | CLOSED — dev bounded no-finding | `dev-r9-11d4-shutdown-post-return-false-success-20260920.md` (`e51a630`) |

## Defects found and repaired inside R9-11

- **H-R9-081** — UDP terminal ownership: closed at `730f993c687683f00f82859cbb7587d9fd6f5b84`.
- **H-R9-082 / H-R9-083** — process-resource sampler: escaped setsid TCP/UDP
  holder, readiness fail-closed timeout, `pathlib` reap, four-table
  present/absent/unknown oracle, `--net-dir` result-construction seam; source/test
  contract accepted at `2f92781`.
- **H-R9-084** — sampler exit status failed to follow terminal cleanup truth:
  repaired at `8cbd9af`; wrapper returns nonzero on `!cleanup_complete` while
  `result["exit"]` preserves the measured child fact.

## Boundary

All seven R9-11 lanes are closed at reachable anchors. R9-11 as a whole is
**not** marked complete — the dedicated final independent R9 bounded review
(READY_LOCAL 5) and final exact-tree provenance (READY_LOCAL 4) remain open.
No release flag, D019, or policy value was touched. `READY_LIVE: none`.
