# Nekomusume ChatGPT Handoff

Checked at: 2026-09-01 19:00 Asia/Shanghai
Repository HEAD: `859f91570444daf11969c03fe6c153d56919fc3d`
Previous checked implementation HEAD: `e24cc8d26f2c0652cb0d836234ac1c6929d1457a`
Previous reviewer handoff commit: `225abc5efdb285f3e6498a6f7f6ea32db9bc7131`

## What changed

One substantive coding-agent commit landed after the previous reviewer handoff:

- `859f915` — **implementation/tests/status repair**. The Carrier Manager no longer accepts a caller-authored integer readiness count. It now has target/generation-scoped readiness observations with duplicate IDs ignored, a separate cold authenticated+resume-validated promotion path, and deterministic generation/duplicate/uncertain-replay tests. `HealthObservationWindow` now returns a distinct `AdmissionBudgetExhausted` error instead of merely saturating a diagnostic counter; the CLI fails closed on that condition rather than turning junk into path-health evidence. `docs/status.md` narrows the executable automatic threshold seam to authenticated, resume-validated **cold** recovery and explicitly says it does not exercise D064 warm-readiness observations.

This materially closes the previous junk-processing bound defect and avoids claiming that the current CLI executed an untracked warm-readiness contract. No current-exact-HEAD VPS run has yet been committed after `859f915`, so the time-limited VPS window should now be spent on exact-head evidence rather than more unrelated local polish.

No GitHub commit-status/CI checks are attached to `859f915` through the available status API. Local coding-environment gates remain local evidence, not independent CI attestation.

## Review verdict

**CONTINUE WITH ONE EVIDENCE-LABEL REPAIR, THEN IMMEDIATELY HARVEST CURRENT-HEAD VPS EVIDENCE.**

The project is not blocked. The release plan remains at `IMPLEMENTATION_PLAN.md` item 3, **Bounded release evidence matrix**. N9 and negotiation-path completion remain closed. The highest-value next work is now the rented-VPS evidence path.

Do not implement a new warm-readiness protocol. D064 is still absent from the tracked decision ledger, so warm-readiness remains non-normative/unproven. The current executable path is allowed to remain the narrower cold authenticated+resume-validated recovery path.

## Review findings

### R-001 PASS — junk admission is now a real processing bound

`HealthObservationWindow` now stops accepting ignored/wrong/stale datagrams when its bounded admission budget is exhausted and returns a distinct diagnostic error. The CLI fails closed on this condition. The budget does not increment the path-health failure count and therefore does not accelerate automatic fallback. Deterministic tests cover high junk count, exact permitted progress at the budget boundary, and unchanged health evidence.

This closes the previous resource-bound finding sufficiently for bounded release-evidence experiments. It is not a general DoS/security proof.

### R-002 PASS WITH NARROWED CLAIM — current runtime no longer pretends to exercise missing D064 warm readiness

The tracked decision ledger still jumps from D063 to D065; no accepted D064 source was found in the current repository hierarchy. `859f915` takes the safe branch permitted by the previous handoff: current CLI evidence is narrowed to **cold authenticated, resume-validated recovery**. `docs/status.md` now says the automatic threshold seam does not exercise D064 warm-readiness observations.

The separate `observe_failed_udp_target_readiness()` API therefore remains candidate/local code only. Do not use it for a release/WAN warm-readiness claim until a normal tracked decision/spec actually defines that contract.

### R-003 HIGH — cold structured timing still emits a misleading warm-readiness field

The runtime correctly calls `promote_cold_authenticated_resume(...)`, but the automatic failover timing event still does:

```text
resume_validated = tcp_authenticated
readiness_satisfied = resume_validated
...
"fallback_class":"cold"
"resume_validated_us": ...
"readiness_satisfied_us": ...
```

The duplicated `readiness_satisfied_us` field can be read as evidence that a readiness gate was observed even though the same event explicitly classifies the path as cold and no D064 warm-readiness sequence exists. That is evidence-label drift.

**Required correction before the next automatic-degradation VPS row:** for `fallback_class=cold`, remove the warm-readiness field or rename it to an unambiguous cold-promotion field such as `cold_promotion_ready_us` / `promotion_gate=cold_authenticated_resume`. Keep `resume_validated_us`. Add a regression that cold evidence cannot contain a warm-readiness claim. Do not invent new network observations to preserve the old field name.

This is a structured-evidence repair, not a transport behavior change.

### R-004 LOW — readiness-observation ownership comment overstates what the API owns

`ReadinessObservation.observation_id` is supplied by the caller while `CarrierManager` owns the bounded accumulation/deduplication set. The current comment says observation IDs are manager-owned. Since this warm path is not used for current release evidence, this is not a blocker, but correct the comment or naming while touching the file so a future implementer does not mistake caller-provided IDs for manager-generated provenance.

Do not spend this rental window implementing a full warm-readiness protocol merely to address this wording.

## Evidence boundaries

- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, `RELEASED=false` remain correct.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `859f915` is local implementation/test/status evidence; it is not VPS/WAN evidence.
- Current CLI automatic threshold mode may claim only controlled owned-endpoint UDP reply cessation -> threshold decision -> authenticated/resume-validated **cold** TCP recovery, once rerun on an exact commit after R-003 is fixed.
- It may not claim D064 warm readiness, natural Internet blackhole detection, public/general reachability, production failover, or security approval.
- Existing historical VPS rows remain valid for their recorded exact commits and conditions; they are not automatically promoted to current-head evidence.
- IPv6 remains environment-blocked while the owned endpoints lack a real end-to-end IPv6 path. Do not repeat unchanged IPv6 failures.
- Standing VPS authorization permits the periodic Session run, automatic controlled degradation row, bounded resource sampling, temporary listeners, cleanup, and equal-application HY2 comparison. None need renewed per-run approval.
- The rented VPS is time-limited. Prefer exact-head real-socket evidence and the HY2 comparison seam over documentation polish or speculative features.

## Work Package — repair one label, then run the rented-VPS evidence batch

Execute A -> B -> C -> D -> E in dependency order where applicable. This package is deliberately thick enough to keep the coding agent moving; do not stop after A if B/C/D are READY.

### Primary A — close the cold-evidence label defect and verify the exact candidate

**Goal:** make the automatic cold-recovery structured event impossible to misread as warm-readiness evidence, then establish a green local candidate for VPS deployment.

Required changes:

1. For `fallback_class=cold`, remove `readiness_satisfied_us` or replace it with an explicitly cold name/enum such as `promotion_gate=cold_authenticated_resume` and/or `cold_promotion_ready_us`.
2. Preserve the useful timestamps: failure decision, TCP connect start/connected, negotiation, authentication, resume validation, manager activation, first resumed data, recovery latency.
3. Add a regression that a cold timing event cannot advertise D064/warm readiness.
4. Correct the `ReadinessObservation` ownership comment: the manager owns bounded accumulation/deduplication; the current observation ID is supplied by the event source/caller.
5. Do not wire `observe_failed_udp_target_readiness()` into the CLI or invent a D064 contract in this slice.

Run at minimum:

- targeted automatic-health-failover diagnostics/tests;
- CarrierManager cold-promotion/readiness-dedup/generation tests;
- health-window admission-budget tests;
- periodic `FramedReader` fragmentation/deadline tests;
- `cargo fmt --all -- --check`;
- `cargo check --workspace --locked`;
- `cargo test --workspace --all-targets --locked --no-fail-fast`;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- `bash scripts/check.sh`;
- `git diff --check`.

Fuzz smoke is not required solely for an evidence-label/comment change unless parser/wire code changes or the normal gate requires it.

After A is green, use the resulting exact commit for B and C so the rental-window evidence does not immediately become stale.

### Follow-up B — current-exact-HEAD five-minute periodic authenticated Session VPS row

**Dependency:** A green. The periodic framing path is already repaired and READY.

Run one bounded self-owned client<->VPS periodic TCP Session using the existing periodic runner and process-resource sampler. Prefer the already documented comparable profile if still valid: approximately 60 records, 32 bytes each, ~5 s interval, ~5 minutes total, one Session/stream, finite ACK deadline. Keep the single run under the 10-minute standing-authorization limit.

Record at minimum:

- experiment ID; exact git commit; client/server binary SHA-256;
- actual port, count, bytes, interval, duration, ACK timeout;
- canonical negotiation + Noise authentication + Session identity;
- attempted/confirmed/missing/duplicate record counts and application bytes;
- P50/P95 confirmation latency from the runner;
- client/server elapsed, CPU user/system, max RSS, peak FD and owned socket count where available;
- exit/signal state;
- cleanup: no experimental listener/process/temp runtime remains.

Preserve any failure. Do not mechanically rerun an unchanged failure.

Acceptance: **current-exact-commit bounded periodic authenticated Session evidence**. Do not call five minutes production long-lived stability.

### Follow-up C — current-exact-HEAD automatic controlled-degradation -> cold TCP recovery VPS row

**Dependency:** A green. Run after or adjacent to B while the same exact binary is deployed.

Use the existing explicit UDP reply-cessation/automatic-health-failover seam on self-owned endpoints with a small bounded count/payload. Record the real current semantics:

- UDP canonical negotiation/Noise authentication and initial validated DeliveryAck;
- uncertain logical range sent before reply cessation;
- three one-second health failure observation windows and timestamps;
- ignored/admission diagnostics, with no junk promoted into health success/failure;
- pending manager switch target/generation;
- TCP connect, canonical negotiation, Noise authentication, resume guard/validation;
- **cold promotion only** (`promotion_gate=cold_authenticated_resume` or equivalent), with no warm-readiness field;
- manager activation timestamp;
- uncertain resend, authenticated exact-semantic TCP DeliveryAck, receiver dedup/conflict result, final logical bytes;
- recovery latency and process-resource samples when practical;
- cleanup verification.

Classification must remain: controlled application-level UDP reply cessation on owned endpoints + bounded threshold-driven cold recovery. It is not natural Internet blackhole detection, D064 warm readiness, or general production failover.

### Follow-up D — make and execute the first fair HY2 equal-application paired sample if dependencies permit

**Dependency:** local repository green; independent of D064 warm readiness.

The repository already pins HY2 v2.9.3 and the temporary forwarding seam. Do not weaken the loopback-only guard in `scripts/bench/compare-hy2.sh`; create/finish a separate fail-closed self-owned-VPS orchestrator or adapter that reuses the result schema/methodology.

First ensure Nekomusume and HY2 answer the same application question:

```text
send exact deterministic payload bytes -> receive exact same bytes
```

Required fairness contract:

- same owned client/VPS pair and close time window;
- same deterministic payload file/byte count/SHA-256;
- same route and recorded MTU metadata;
- both authenticated+encrypted with experiment-only credentials;
- same stream/load shape, finite timeout and run count;
- temporary high ports only; existing production HY2 process/config untouched;
- Nekomusume and HY2 commands both emit/validate the required application-byte result contract.

If the current Nekomusume CLI cannot consume the exact workload payload/file semantics required by the comparison, implement only the smallest benchmark adapter/seam needed; do not redesign the transport.

When an actually equivalent pair is executable, run the first small paired sample using the existing minimum-repeat methodology (normally 5 runs). Prefer interleaved/nearby runs rather than doing all of one implementation hours before the other. Record raw samples, median/P95/failures, CPU user/system, RSS, FD and application bytes. `wire_bytes` remains null unless capture metadata is trustworthy. Preserve slower/failed Nekomusume results exactly. No superiority claim.

If a genuine environment dependency prevents the paired run (for example a missing usable experimental HY2 client artifact on the owned client), still finish and locally validate the Nekomusume-side adapter/orchestrator and record the exact environment gap; do not substitute an unequal workload.

### Follow-up E — reconcile the bounded release-evidence ledger

After B/C/D, update normal evidence documents, `docs/status.md`, `ROADMAP.md`, and any release-evidence matrix only to the level actually proven.

- Link the new exact-head periodic row if it ran.
- Link the new cold automatic-recovery row only with its narrow classification.
- Keep D064 warm readiness unproven/non-normative.
- Keep IPv6 environment-blocked unless the owned path actually changed.
- Check HY2 comparison complete only if an equal-application paired run actually occurred.
- Preserve negative evidence and supersession relationships.
- Do not mark `IMPLEMENTATION_PLAN.md` item 3 complete merely because B/C pass; the matrix still includes environment-dependent/NAT/endpoint-change/comparison evidence.
- Keep RC/security/production/global freeze flags unchanged.

## Completion gates

This package is complete only when:

- cold structured evidence no longer contains a misleading warm-readiness field;
- local full gate is green on the exact commit used for VPS work;
- one current-exact-commit five-minute periodic authenticated Session row is captured or a new evidence-backed blocker is recorded;
- one current-exact-commit controlled automatic-degradation -> cold TCP recovery row is captured or a new evidence-backed blocker is recorded;
- both rows preserve resource/cleanup/negative evidence boundaries;
- the HY2 equal-application seam is materially advanced, preferably through the first valid paired sample if the owned environment supports it;
- evidence/status documents reflect only what actually ran;
- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` remain unchanged.

## Fallback

If B fails on the repaired exact commit, preserve the failure and use the new logs/resource samples to isolate the smallest new diagnostic variable; do not repeat unchanged. D remains independently READY.

If C fails for a real runtime reason, preserve the row and keep cold automatic-failover release evidence blocked; B and D remain READY.

If HY2 paired execution is environment-blocked, finish/validate the Nekomusume comparison adapter and use remaining VPS time for another already-defined, scientifically distinct release-evidence task such as package/operator smoke or current-head process-resource validation. Do not invent a speculative feature and do not rerun unchanged IPv6 failures.

## Do not expand into

- inventing or implementing D064 warm-readiness semantics without a tracked decision;
- calling five-minute periodic evidence production long-lived stability;
- natural-WAN/public/general reachability claims from controlled owned-endpoint fault injection;
- previous/current interoperability before a real prior release exists;
- 0-RTT, enabled FEC, striping/aggregation or exotic carriers without an observed-problem gate;
- third-party targets, scanning, production network changes, or experiments outside standing authorization;
- touching the existing production Hysteria config/service for the comparison.

## Questions requiring maintainer decision

none.
