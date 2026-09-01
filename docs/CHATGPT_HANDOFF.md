# Nekomusume ChatGPT Handoff

Checked at: 2026-09-01 19:57 Asia/Shanghai
Repository HEAD: `4b95e96202a02ea496fe54eb2e0adc0480f52e83`
Previous checked implementation HEAD: `859f91570444daf11969c03fe6c153d56919fc3d`
Previous reviewer handoff commit: `d2c3d11b01a802edca59179775305285b8ad849d`

## What changed

One substantive coding-agent commit landed after the previous reviewer handoff:

- `4b95e96` — **implementation/test evidence-label repair; no transport-semantic expansion**. Cold automatic recovery no longer emits the misleading `readiness_satisfied_us` field. It now records `promotion_gate=cold_authenticated_resume` plus `cold_promotion_ready_us`, keeps `resume_validated_us`, adds a regression that cold evidence cannot advertise the old warm-readiness label, and corrects the `ReadinessObservation` ownership comment so caller-supplied observation IDs are distinguished from manager-owned bounded accumulation/deduplication.

This closes the narrow R-003/R-004 evidence-label defects from the previous handoff.

During this review, a more important repository-fact correction was found: the previous handoff incorrectly stated that tracked decision D064 was absent. It is **present and Accepted** in `docs/decisions.md`, and its full accepted design contract is `docs/adr/m3-concurrent-carrier-semantics.md`. D064 explicitly chooses **single-active, multi-ready with warm TCP fallback** for M3, requires authenticated target/generation/delivery-epoch-bound readiness before a candidate becomes warm, and sets initial `k_ready=3`. The ADR explicitly rejects cold-fallback-only as the selected M3 design, while still requiring warm and cold recovery measurements to remain separate.

Current runtime does not yet implement that accepted warm contract. `CarrierManager::observe_failed_udp_target_readiness()` only operates after `fail_udp_to_tcp()` has created a pending switch, and the CLI's automatic path connects/authenticates TCP only after UDP failure and calls `promote_cold_authenticated_resume(...)`. Therefore the current automatic runtime is truthful **cold recovery evidence**, but it cannot satisfy D064 warm-readiness/fallback evidence.

This is a reviewer/spec-reconciliation finding. It is not evidence that the current cold path is itself incorrect; it means the release-evidence plan must not silently substitute cold recovery for the accepted warm-fallback contract.

No GitHub commit-status/CI checks are attached to `4b95e96` through the available status API. Local coding-environment gates remain local evidence, not independent CI attestation.

## Review verdict

**NEEDS REPAIR — D064 accepted warm-fallback implementation/evidence gap is now the Primary release blocker; rented VPS remains immediately valuable.**

Accept `4b95e96` as the correct cold-evidence-label repair. Do not revert it and do not reintroduce a fake warm-readiness field into cold evidence.

However, do not continue treating D064 as absent/non-normative. The accepted D064/ADR contract must be restored to planning and implementation truth. The next package should implement the minimum bounded warm-ready seam required by the accepted contract, prove it deterministically, then spend the rented-VPS window on exact-head periodic Session + warm recovery evidence. The already-working cold path remains useful as a separately classified baseline.

`IMPLEMENTATION_PLAN.md` item 3, **Bounded release evidence matrix**, remains the current phase. It must not be checked complete until the D064-selected warm/cold distinction and the remaining environment/evidence rows are represented truthfully.

## Review findings

### R-001 PASS — cold recovery evidence labels are now truthful

`4b95e96` removes `readiness_satisfied_us` from `fallback_class=cold`, emits an explicit cold promotion gate, and adds a regression forbidding the misleading warm-readiness field. The `ReadinessObservation` ownership comment is also corrected.

This is accepted. It does not prove warm fallback.

### R-002 HIGH — previous reviewer conclusion about D064 was factually wrong

Current repository facts are unambiguous:

- `docs/decisions.md` contains **D064: UDP-primary + warm TCP fallback Carrier Manager contract**, status **Accepted design contract**;
- `docs/adr/m3-concurrent-carrier-semantics.md` is also **Accepted design contract**;
- the selected M3 design is `single-active, multi-ready`;
- TCP may be established/authenticated/validated/admitted while UDP is active;
- readiness is distinct from TCP connect/write, packet ACK, and Session delivery;
- default `k_ready=3` authenticated readiness observations;
- warm and cold recovery must be reported separately;
- “cold fallback only” is explicitly listed as the rejected M3 alternative.

The previous handoff's claim that the decision ledger skipped D064 must not be propagated into status, tests, or future evidence interpretation.

### R-003 HIGH — current readiness API cannot prove D064 warm readiness

The current `observe_failed_udp_target_readiness()` requires `pending_switch`, and `pending_switch` is created only after `fail_udp_to_tcp()` has already decided UDP failure. Therefore its three readiness observations are post-failure target-promotion evidence, not proof that TCP was already warm before failure decision.

The current CLI automatic path is even narrower: it creates/authenticates TCP after the failure threshold, then uses `promote_cold_authenticated_resume(...)` immediately after resume validation.

That is valid **cold recovery** behavior. It is not D064 warm standby.

### R-004 MEDIUM — release plan/status must expose the warm/cold gap instead of hiding it

`docs/status.md` truthfully says the current automatic threshold seam is cold. Keep that boundary. But planning/release-evidence navigation should explicitly state that D064 warm standby remains unimplemented/unproven in the executable process/WAN path and is required before the selected M3 fallback design has release evidence.

Do not unfreeze the canonical corpus or reopen negotiation work merely because the warm path is incomplete. This is Carrier Manager/readiness/runtime evidence work, not N9 wire-corpus work.

## Evidence boundaries

- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` remain correct.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- N9 and canonical negotiation-path completion remain closed for their stated bounded scope.
- `4b95e96` proves a truthful cold-evidence schema/test repair, not VPS/WAN behavior.
- Existing cold controlled-degradation evidence remains valid only for the exact commits/conditions recorded; it does not establish warm fallback.
- D064 warm readiness is an accepted design contract but currently lacks a matching executable process/WAN implementation/evidence path.
- IPv6 remains environment-blocked while the owned endpoints lack a real end-to-end IPv6 path. Do not repeat unchanged IPv6 failures.
- Standing authorization permits temporary owned-endpoint TCP/UDP listeners, periodic Session runs, warm/cold failover experiments, bounded resource sampling, cleanup, and fair HY2 comparison without new per-run approval.
- The VPS is a time-limited rental asset. Once the D064 correctness seam is green, exact-head real-socket evidence outranks documentation polish and speculative features.

## Work Package — implement D064 warm standby, then harvest exact-head VPS evidence

Execute A -> B -> C -> D -> E in dependency order. This is intentionally a thick closure/evidence package. If A/B consume the cycle, continue from C next cycle; do not jump to unrelated work.

### Primary A — reconcile D064 into executable planning and define the minimum warm-ready seam

**Goal:** restore the Accepted D064 contract as the implementation/evidence target without inventing a second architecture.

Before coding, reread:

- D064 in `docs/decisions.md`;
- `docs/adr/m3-concurrent-carrier-semantics.md`;
- current `CarrierManager` pending-switch/readiness APIs;
- current failover CLI cold path and its tests.

Required contract for this slice:

1. UDP remains the sole active owner for new Session data before failure.
2. A TCP fallback candidate may be connected, canonically negotiated, Noise-authenticated, resume-bound/validated, resource-admitted, and readiness-probed **while UDP is still active**.
3. The candidate becomes `warm` only after three distinct, authenticated, target/generation/session/delivery-epoch-bound readiness observations (`k_ready=3`). Duplicate/stale/wrong-generation/unvalidated observations cannot advance readiness.
4. Warm readiness must not itself confirm Session delivery and must not consume new application data.
5. When UDP later reaches the existing bounded failure decision, an already-warm TCP candidate may be promoted without repeating connect/Noise setup inside the recovery interval; the ownership change remains single-active and generation-scoped.
6. If no eligible warm candidate exists, the existing `promote_cold_authenticated_resume(...)` path remains available and must continue to emit `fallback_class=cold`.
7. Warm and cold recovery metrics remain distinct.
8. Do not implement striping/aggregation, 0-RTT, new cryptography, or speculative carrier features.

**Important API point:** do not merely rename or reuse `observe_failed_udp_target_readiness()` unchanged. Its current dependency on `pending_switch` structurally prevents pre-failure warm evidence. Refactor/add the smallest manager state/API that can accumulate bounded readiness for a standby candidate while UDP is active, then atomically consume/validate that warm state during failure promotion.

Update `IMPLEMENTATION_PLAN.md`/`docs/status.md` only enough to make the D064 implementation/evidence gap explicit; do not mark it complete before B/C evidence exists.

### Follow-up B — deterministic warm/cold state-machine and process tests

**Dependency:** A implementation present.

Add tests that prove the accepted design boundaries, not just the happy path.

At minimum cover:

- TCP candidate is standby/warm while UDP remains active; no two active owners;
- fewer than 3 readiness observations cannot promote or mark warm;
- duplicate observation IDs do not advance readiness;
- stale/wrong target/generation/session/epoch or unauthenticated/unvalidated observations fail closed and cannot mutate active state;
- readiness counters/IDs remain bounded and reset correctly on generation replacement/failure;
- warm candidate carries no new application data before promotion;
- failure with a warm candidate promotes the exact eligible generation;
- failure without a warm candidate remains cold and preserves the `4b95e96` cold evidence labels;
- warm attempt failure is retained as failed-warm evidence; later new-generation recovery is cold;
- uncertain resend + authenticated exact-semantic DeliveryAck + receiver dedup/conflict semantics remain correct across both warm and cold paths;
- recovery metrics classify warm vs cold separately.

Add/extend a loopback/process fixture that establishes the warm TCP candidate before controlled UDP reply cessation and proves the first resumed logical data only after manager promotion.

Run the full local gate after the focused tests:

- `cargo fmt --all -- --check`;
- `cargo check --workspace --locked`;
- `cargo test --workspace --all-targets --locked --no-fail-fast`;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- `bash scripts/check.sh`;
- `git diff --check`.

Run fuzz smoke only if wire/parser/external decoding behavior changes or the normal gate requires it.

### Follow-up C — current-exact-HEAD VPS lab batch: periodic Session + D064 warm recovery + cold control

**Dependency:** A/B green. Build/deploy one exact commit and reuse that exact binary for all compatible scenarios to reduce rental-window setup overhead.

Within standing authorization, run one bounded lab batch on owned client/VPS endpoints:

#### C1 periodic authenticated Session

Use the existing periodic runner/resource sampler for approximately 5 minutes (single run <10 minutes): e.g. ~60 records, 32 bytes, ~5 s interval, finite ACK deadline, one Session/stream.

Record exact commit and binary SHA-256, actual parameters, negotiation/Noise/Session identity, attempted/confirmed/missing/duplicate counts, application bytes, P50/P95 confirmation latency, client/server elapsed + CPU user/system + max RSS + peak FD/owned socket count where available, exit/signal state, and cleanup.

Classification: bounded five-minute periodic authenticated Session evidence, **not** production long-lived stability.

#### C2 D064 warm automatic recovery

Establish the TCP fallback and obtain the three accepted warm-readiness observations **before** the UDP failure decision. Then use the existing controlled owned-endpoint UDP reply-cessation seam.

Record at minimum:

- UDP active generation/session identity;
- TCP standby/warm generation and timestamps for connect, canonical negotiation, Noise authentication, resume validation/admission;
- the three distinct readiness observations and the exact timestamp warm eligibility became true;
- proof UDP remained sole active data owner until failure;
- three bounded UDP failure observation windows/decision timestamp;
- manager promotion of the already-warm generation;
- `fallback_class=warm` and recovery interval that does not include TCP connect/Noise setup;
- uncertain resend, authenticated exact-semantic TCP DeliveryAck, receiver dedup/conflict result, final logical bytes;
- recovery latency, resource sample, and cleanup.

Classification: controlled application-level UDP reply cessation on owned endpoints -> threshold decision -> **pre-established D064 warm TCP recovery**. It is not natural Internet blackhole detection, public reachability, production failover, or security approval.

#### C3 cold control row

If it can be run without changing the binary/configuration question, retain one small cold recovery row from the same exact commit so warm/cold timing/classes are directly distinguishable. Do not rerun merely to chase a faster number.

Preserve every failure. No unchanged-failure reruns.

### Follow-up D — first valid HY2 equal-application paired sample

**Dependency:** local repository green; independent of warm readiness after the exact-head lab binary exists.

Reuse the pinned HY2 v2.9.3 artifact and repository comparison methodology. Do not touch the existing production Hysteria config/service and do not weaken the loopback-only guard in `scripts/bench/compare-hy2.sh`; use/finish a separate self-owned-VPS orchestrator or adapter.

Both implementations must answer the same application question:

```text
send exact deterministic payload bytes -> receive exact same bytes
```

Require same owned client/VPS pair, close time window, payload file/length/SHA-256, route/MTU metadata, authenticated+encrypted security class, stream/load shape, finite timeout/run count, and temporary experiment-only high ports/credentials.

If the Nekomusume CLI lacks exact workload-file semantics, add only the smallest benchmark adapter necessary. Prefer 5 interleaved/nearby paired runs. Record raw samples, median/P95/failures, CPU user/system, RSS, FD and application bytes. `wire_bytes` remains null unless capture metadata is trustworthy. Preserve slower/failed Nekomusume results. Make no superiority claim.

If environment genuinely blocks the pair, finish/validate the Nekomusume adapter and record the exact environment blocker; do not substitute an unequal workload.

### Follow-up E — reconcile release-evidence/status ledgers

After C/D, update evidence/status/navigation only to what actually ran:

- link current-exact-head periodic evidence;
- link D064 warm recovery only if C2 actually proved pre-failure warm eligibility;
- retain cold recovery as a separate class, not a substitute;
- make the D064 implementation/evidence state explicit in `docs/status.md` and release-evidence navigation;
- keep IPv6 blocked unless the environment really changed;
- mark HY2 comparison complete only if an equal-application paired sample actually ran;
- preserve negative evidence and supersession relationships;
- do not mark `IMPLEMENTATION_PLAN.md` item 3 complete until the release matrix's remaining genuine rows are closed or explicitly environment-inapplicable under reviewed scope;
- keep RC/security/production/global-freeze flags unchanged.

## Completion gates

This package is complete only when:

- the accepted D064 contract is no longer treated as absent;
- a bounded pre-failure warm TCP candidate state exists and is distinct from active/cold recovery;
- readiness requires 3 distinct authenticated, properly bound observations and is generation/resource bounded;
- cold recovery remains truthful and separately classified;
- deterministic tests enforce single-active ownership, warm eligibility, invalid observation rejection, warm/cold metrics, uncertain replay and exact delivery evidence;
- full local repository gate passes on the commit used for VPS work;
- current-exact-head periodic Session evidence is captured or a new evidence-backed blocker is recorded;
- current-exact-head D064 warm recovery evidence is captured or a new evidence-backed blocker is recorded;
- the HY2 equal-application seam is materially advanced, preferably through the first valid paired sample if the owned environment supports it;
- evidence/status documents claim only what ran;
- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` remain unchanged.

## Fallback

If A/B reveals that D064 cannot be implemented without a genuinely new wire/readiness protocol decision, do not invent one silently. Preserve the exact gap, keep the current cold path valid, and use the remaining cycle/VPS window for C1 periodic Session + D HY2 comparison + already-defined package/resource evidence while escalating only the specific architecture choice through the normal decision process.

If C2 fails for a real runtime reason, preserve the row and use the new logs/resource samples to isolate the smallest changed diagnostic variable. C1 and D remain READY.

If HY2 paired execution is environment-blocked, use the time-limited VPS for another already-defined, scientifically distinct evidence row such as package/operator smoke, process-resource validation, or current-head real-socket lifecycle. Do not invent speculative features and do not repeat unchanged IPv6 failures.

## Do not expand into

- rewriting D064 as cold-fallback-only without an explicit superseding architecture decision;
- using post-failure readiness observations as proof that a fallback was warm before failure;
- calling five-minute periodic evidence production long-lived stability;
- natural-WAN/public/general reachability claims from controlled owned-endpoint fault injection;
- previous/current interoperability before a real prior release exists;
- 0-RTT, enabled FEC, striping/aggregation or exotic carriers without an observed-problem gate;
- third-party targets, scanning, production network changes, or experiments outside standing authorization;
- touching the existing production Hysteria config/service for comparison.

## Questions requiring maintainer decision

none. The repository already contains the Accepted D064 decision; this is an implementation/evidence reconciliation, not a new architecture choice.
