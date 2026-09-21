# Developer bounded I4-CLI-H independent review — human-output / stderr / exit contract

**Anchor:** exact source/test tree `42be539` + reviewer/dev commits through `7197abf`.
**Owners inspected:** `neko-cli` `failover_gate` (lines 4788/5774-5780), `fail()` (eprintln + exit 2), `print_json`, `matrix_probe`, `reachability::run`, probe tests.

## Per-invariant coverage

| Invariant | Coverage |
|---|---|
| `pass: 喵~！` / `fail: 喵呜呜呜呜…` agrees with exit status and the one-case machine artifact | `failover_gate` evaluates the literal substring gate `artifact.contains("\"reachable\":true")` once and prints `pass: 喵~！` + exit 0, else `fail: 喵呜呜呜呜…` + exit 1 — output and exit are driven by the same single boolean check on the one-case `reachability::run` artifact |
| stderr/usage/config failures do not print a later human success line or contaminate machine stdout | `fail()` writes `neko: …` to stderr and exits 2 — never reaches the `println!("pass: …")` branch; `--json` mode prints only the JSON artifact on stdout, human wording never leaks into machine output |
| "parsed result" wording is qualified to literal substring gate | the current contract is `artifact.contains("\"reachable\":true")` — a literal substring match on the single-case `reachability-matrix.v1` artifact, not a typed/schema parse; the corrected reconciliation at `7197abf` records this wording |
| README/ROADMAP/D007 wording truthful vs emitted line format | `failover_gate` emits exactly `pass: 喵~！` or `fail: 喵呜呜呜呜…` on stdout plus exit 0/1 — matching the documented human contract; no stale prose claims a different format |
| schema/multi-case or alternate `reachable` occurrence is a future re-review trigger | the current artifact has exactly one case and one `reachable` field; a future multi-case schema or a second `reachable`-shaped token would require re-challenge — no counterexample exists on the exact-current one-case schema |

## Commands actually run

- `cargo test -p neko-cli --test probe failover_gate` — `failover_gate` assertions pass (pass/fail + exit agree)
- `cargo test -p neko-cli --test probe matrix_probe` — `--json` emits single JSON artifact on stdout, exit agrees with `reachable`
- `cargo test -p neko-cli --test probe invalid_bind` — `fail()` produces stderr + nonzero exit, no success line

## Result

No concrete defect found across human-output/stderr/exit semantics.
`failover_gate`'s literal substring gate produces consistent `pass`/`fail`
output and exit status; `fail()` failures never print a later success line;
machine stdout is never contaminated by human wording. The current one-case
`reachable` contract is accurately described as a literal substring gate.

**READY_LIVE: none** — deterministic local evidence only.
