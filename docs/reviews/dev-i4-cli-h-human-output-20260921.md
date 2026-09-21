# Developer bounded I4-CLI-H review — human-output / stderr contract

**Anchor:** exact source/test tree `26aa4e8` + reviewer commits through `895bc4a`.
**Owners inspected:** `neko-cli` `fail()` (eprintln + exit 2), `failover_gate`
output (`pass: 喵~！`/`fail: 喵呜呜呜呜…` on `reachable`), `print_json`,
probe tests.

## Per-invariant coverage

| Invariant | Coverage |
|---|---|
| no `pass`/`ok`/success wording after a typed terminal failure or nonzero wrapper | `fail()` writes `neko: …` to stderr and exits 2 — never prints `pass`/`喵~！`; `failover_gate` prints `fail: 喵呜呜呜呜…` and exits 1 on `reachable:false` |
| `喵~！`/`喵呜呜呜呜…` match promised public wording | `failover_gate` emits `pass: 喵~！` only when the artifact contains `"reachable":true`, else `fail: 喵呜呜呜呜…` — success wording is driven by the parsed result, not by an earlier stage |
| stderr does not contaminate machine stdout | `fail()`/`eprintln!` go to stderr; `--json` mode prints only the JSON artifact on stdout — human wording never leaks into machine output |
| signal/timeout/cleanup failure summaries truthful, no later success line | `failover-client`/`failover-server`/`failover-adapter` emit `ok:false` + `last_stage`/`reason` on failure; no post-failure success print (H-R9-084) |
| unsupported/usage failures fail-closed | usage/arg errors exit nonzero via `fail()`; `--json`/`--record-file` bounds are validated — no broadened CLI format invented |

## Reachable regressions named

- `failover_gate` probe asserts `pass: 喵~！`/`fail: 喵呜呜呜呜…` + exit code
  agree with `reachable`.
- `fail()`-driven argument/port rejects (`ports outside 40080-40100`,
  `readiness delay outside 0-3000`) produce stderr + nonzero exit.
- Machine-mode probe asserts stdout contains only the JSON document.

## Result

No concrete defect found across human-output/stderr semantics. Failure
wording is consistent between human output and exit status; stderr does not
contaminate machine stdout; no post-failure success line is printed.

**READY_LIVE: none** — deterministic local evidence only.
