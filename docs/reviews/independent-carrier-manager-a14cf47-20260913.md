# Independent bounded CarrierManager / health / migration-back review — exact `a14cf47`

Bounded independent review of `CarrierManager`, `CarrierHealth`, `CarrierHealthEvidence`, warm/cold promotion, migration-back hold/margin, and `FailoverController` commit path in `crates/neko-carrier/src/lib.rs` (roughly `:1340-1610`, `:1932-2360`, `:2380-2750`), against `docs/spec/m0-carrier-state.md`, `docs/adr/m3-concurrent-carrier-semantics.md`, and `docs/adr/m4-migration-back-state-machine.md`, at reachable exact `a14cf471b6ef97442f36419ee8df3648037a5ef4` (post switch-margin saturation repair). Not a WAN/live claim, not Session delivery, not an independent security/release approval.

## Challenged invariants and result

- **Single-active, multi-ready; no application striping — holds.** `active: Option<PathId>` is a single slot; `choose`, `migrate_back_to_udp`, `fail_udp_to_tcp`, and `promote_pending_target` all serialize through it. No path ever stripes application data across two actives.
- **Warm path cannot carry new application data before promotion — holds.** `prepare_warm_candidate` installs a standby that "cannot carry application data through this manager API"; it only becomes promotable after `observe_warm_candidate_readiness` accumulates exactly 3 D064 readiness observations (`:2503-2509`) and `promote_warm_authenticated_resume` re-checks the full path/generation/session/epoch tuple plus `warm && ready_observations == 3`.
- **Health feedback cannot masquerade as Session delivery — holds.** `HealthSample`/`HealthObservation`/`HealthState` are a separate channel from `SessionDelivery`; `migrate_back_to_udp` additionally requires `candidate.validated` (explicit path-challenge evidence) which health feedback alone cannot set.
- **Failed/generation-stale paths cannot be promoted — holds.** `migrate_back_to_udp` rejects `generation < active` (`OldGeneration`) and `generation != active` (`GenerationMismatch`); `promote_cold/warm` reject `generation < pending` / `!= pending`. `observe_warm_candidate_readiness` rejects stale/mismatched tuples and resets the readiness accumulator atomically.
- **Voluntary-switch hysteresis/hold is monotonic and generation-scoped — holds.** `choose` requires `hold >= min_hold_events` plus a score-margin win and resets `hold` on switch; `migrate_back_to_udp` accumulates `migration_hold` only on fully-gated valid candidates and resets it on success. Repaired this pass: `switch_margin` is now added with `saturating_add` in both `migrate_back_to_udp` and `choose` (`a14cf47`), so a caller-supplied `i64::MAX` margin can no longer overflow the comparison and admit an unjustified switch.
- **Migration-back cannot bypass validation or active-owner handoff — holds.** `FailoverController::apply_migration_back` only commits a `Tcp -> Udp` transition already authorized by the manager (`:1497-1503`); `migrate_back_to_udp` enforces validated + healthy + score-margin + hold before mutating `active`.
- **Failure reason/event timestamps do not invert the transition — holds.** `CarrierHealthEvidence.observe`/`observe_event` record `from -> to` transitions from the real previous and resulting states (`:2001-2009`, `:2028-2036`); the evidence vector cannot record a transition whose `to` precedes or contradicts the state that produced it.
- **Bounded evidence/state — holds.** `samples`, `events`, `transitions`, and `readiness_observation_ids` are all bounded by `max_samples`/`max_paths`; `observe` rejects new-path samples past `max_paths` (`:2372`).

## Defect repaired this pass

`current_score + switch_margin` / `x + switch_margin` used plain `i64` addition with an unbounded caller-supplied margin, overflowing (debug panic / release wrap) and defeating the margin gate. Now `saturating_add` in both sites; `migrate_back_switch_margin_addition_does_not_overflow` and `choose_switch_margin_addition_does_not_overflow` cover it on the exact tree.

## Evidence

- `cargo test -p neko-carrier` on exact `a14cf47`: 49 lib + integration tests, all passed.
- Exact-tree gate for `a14cf47` green (`scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree).

No remaining defect in this scope. No new `READY_LIVE` question; release/governance state unchanged.
