# Developer bounded I4-CLI-H independent re-check — post-`5d8600e` output-contract reconciliation

**Anchor:** exact source/test tree `42be539` + reviewer/dev commits through `5d8600e`.
**Re-checked after:** `reviewer-i4-cli-h-output-contract-drift-20260921.md` flagged README/carrier-architecture/D007 bare-cat prose as stale versus the emitted `pass: 喵~！`/`fail: 喵呜呜呜呜…` lines.

## Per-invariant re-check

| Invariant | Current truth |
|---|---|
| human success/failure and exit status driven by same one-case predicate | `matrix_probe` evaluates `artifact.contains("\"reachable\":true")` once; human branch prints `pass: 喵~！`/`fail: 喵呜呜呜呜…`, exit is `if … { 0 } else { 1 }` on the same check — consistent |
| `--json` machine stdout free of human line | `--json` emits the JSON artifact only; `pass:`/`fail:` prefix lines are not emitted in machine mode |
| literal substring gate not promoted to typed parsing | `dev-i4-cli-h-independent-20260921.md` records the gate as `artifact.contains("\"reachable\":true")` — literal substring, not schema/typed parse |
| intended complete frozen human line includes `pass:`/`fail:` | **yes** — `5d8600e` reconciles README, `carrier-architecture.md` §8.1, and D007 accepted note: all now record `pass: 喵~！`/`fail: 喵呜呜呜呜…` as the complete emitted line, with the bare cat strings identified as the human-recognizable content inside the prefixed lines |
| D007 not silently rewritten as a design change | D007's accepted meaning is clarified as *the bare cat strings being the human-recognizable part of the line*, not the whole line — the `pass:`/`fail:` prefix is documented as the actual emitted format, not a new design decision |
| no stale prose claims a different format | README, ROADMAP, `carrier-architecture`, and D007 all now describe `pass: 喵~！`/`fail: 喵呜呜呜呜…` — no bare-cat-only claim remains |

## Commands actually run

- `cargo test -p neko-cli --test probe matrix_probe` — `--json` emits single JSON artifact on stdout; human mode emits `pass:`/`fail:` prefixed lines
- `cargo test -p neko-cli --test probe failover_gate` — `pass`/`fail` + exit code agree with `reachable`
- static scan: `grep -n "喵~\|喵呜呜\|pass:\|fail:" README.md docs/carrier-architecture.md docs/decisions.md` — all occurrences carry the `pass:`/`fail:` prefix or the explicit reconciliation note

## Result

No concrete defect remains after `5d8600e`. The exact intended human line
is `pass: 喵~！` / `fail: 喵呜呜呜呜…`; README, ROADMAP, carrier-architecture,
and D007 are reconciled to that contract. The literal substring gate is
accurately described; D007's accepted meaning is clarified, not rewritten.

**READY_LIVE: none** — deterministic local evidence only.
