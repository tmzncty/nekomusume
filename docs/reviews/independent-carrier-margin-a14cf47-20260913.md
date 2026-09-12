# Independent bounded CarrierManager margin review — source exact `a14cf47`

Scope: reviewer source/contract challenge of reachable exact `a14cf471b6ef97442f36419ee8df3648037a5ef4`. This is item-4 review support, not a full audit, production approval, performance conclusion, or policy change.

## Reviewed facts

- Exact `a14cf47` replaces both plain `current_score + switch_margin` comparisons with `saturating_add`, preventing `i64` overflow from panicking in debug or wrapping in release.
- The two added regressions exercise `switch_margin = i64::MAX` for `migrate_back_to_udp` and `choose`; the resulting fail-closed/no-switch behavior is consistent with the current score-margin gate.
- GitHub-hosted `stable checks` and `nightly decode fuzz smoke` for exact `a14cf47` completed successfully. Hosted CI is cross-evidence only and does not replace developer-local exact-tree provenance.

## Finding C1 — negative `switch_margin` domain is not rejected

**Severity: MEDIUM candidate; must be confirmed with a focused regression before repair.**

Owner: `crates/neko-carrier/src/lib.rs`, `ManagerLimits` / `CarrierManager::new`, and the two score-margin comparisons.

Current constructor validation rejects only `min_hold_events == 0` and `max_paths == 0`. `switch_margin` is a public caller-supplied `i64` and therefore accepts negative values.

The accepted M3 carrier-manager ADR states that migration-back requires a score margin and that voluntary promotion requires the candidate score to be above the active score. A negative margin inverts that gate: `candidate >= current + negative_margin` can admit a candidate whose score is lower than the current active path. Extreme negative values become a near-unconditional threshold after saturating addition.

This is distinct from the overflow fixed by `a14cf47`. Saturating arithmetic prevents numeric overflow but does not validate the semantic domain of the margin.

### Required challenge

Before changing code, add focused tests that prove the current behavior with a negative margin can select/migrate to a strictly worse healthy candidate after the other existing gates are satisfied. If the behavior is reproducible and no current committed spec/ADR explicitly defines negative margins as meaningful, reject negative `switch_margin` at `CarrierManager::new` using the existing invalid-limit error path. Do not invent a new positive numeric bound or retune the existing margin policy.

The regression should cover both:

- `choose` voluntary selection; and
- `migrate_back_to_udp` voluntary reverse migration.

Rejection must occur at construction, before any path/sample state exists. Existing zero/nonnegative configurations and the `i64::MAX` overflow regressions must remain green.

## Current ordering constraint

This candidate does **not** supersede the still-open observability O2/O3 findings in the current handoff. Those stable-v1 evidence-contract repairs remain earlier in the execution order. Once O2/O3 are closed on a coherent source tree, C1 can be validated/repaired immediately before continuing the broader Carrier Manager review.
