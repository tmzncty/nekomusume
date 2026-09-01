# Nekomusume ChatGPT Handoff

Checked at: 2026-09-01 20:58 Asia/Shanghai
Repository HEAD: `9d890510c5b694b71f33e13aa68937bcf4f97814`
Previous checked implementation HEAD: `4b95e96202a02ea496fe54eb2e0adc0480f52e83`
Previous reviewer handoff commit: `2ad248550b608a81a865c1c17ff8ce9439c19c49`

## What changed

One substantive coding-agent commit landed after the previous reviewer handoff:

- `9d89051` — **implementation + deterministic tests + status/plan update**. It adds a pre-failure `WarmCandidate` seam to `CarrierManager`, binds readiness state to target path/generation/session/delivery epoch, adds warm/cold recovery counters, allows a TCP candidate to be connected/canonically negotiated/Noise-authenticated before the UDP failure threshold, keeps UDP as the active owner during that setup, adds a warm promotion API, preserves the cold promotion path, and adds CLI/process tests plus local manager tests. `IMPLEMENTATION_PLAN.md` correctly leaves the bounded release-evidence matrix open and `docs/status.md` says current-exact-head VPS warm evidence is still absent.

This is meaningful progress and restores the *shape* of the Accepted D064 single-active/multi-ready design. However, the current executable path still does **not** satisfy D064's readiness-evidence contract, so it must not yet be taken to the VPS as “warm fallback evidence”.

No GitHub commit-status/CI checks are attached to `9d89051` through the available status API. Local coding-environment gates, if run, remain local evidence rather than independent CI attestation.

## Review verdict

**NEEDS REPAIR — pre-failure warm state exists, but the three D064 “readiness observations” are currently synthesized locally rather than observed through an authenticated readiness exchange. Fix the evidence source and timing before current-head VPS warm claims.**

Accept the manager-level separation between standby/warm/active and the preservation of the cold fallback path. Do not revert that structure merely because the executable readiness seam is incomplete.

The release-evidence matrix remains the current phase. The fastest truthful path is now:

```text
real authenticated readiness exchange + truthful timing
    -> deterministic warm/cold process proof
    -> exact-head owned-VPS warm/cold evidence
    -> HY2 equal-application paired sample / remaining VPS-only rows
```

The rented VPS remains a high-priority evidence asset, but correctness/evidence provenance must be repaired before running the warm claim.

## Review findings

### R-001 PASS — manager now has a genuine pre-failure standby/warm state distinct from active ownership

`CarrierManager` can install one newer standby generation while UDP remains active, accumulate bounded readiness state, preserve a separately classified cold promotion path, and atomically promote the matching warm generation after the existing UDP failure decision. Deterministic tests cover several generation/binding/duplicate/reset cases and distinguish warm/cold counters.

This is accepted as a useful implementation seam. It is not by itself network readiness evidence.

### R-002 HIGH — current CLI fabricates the three readiness observations instead of observing authenticated challenge/response traffic

The Accepted D064 contract is explicit: readiness is a separate evidence domain and requires an **authenticated challenge/response** bound to Session identity, path generation, and delivery epoch. TCP connect/write, handshake success, or local booleans are insufficient.

At current HEAD, immediately after the warm TCP Noise handshake finishes, the client calls `prepare_warm_candidate(...)` and then directly loops `observation_id in 1..=3`, passing:

```text
authenticated=true
resume_validated=true
resource_admitted=true
```

into `observe_warm_candidate_readiness(...)` without any readiness request/response I/O between those observations. `tcp_warm_readiness` is therefore a client-local synthetic event, not an observed peer readiness exchange. The three distinct IDs prove dedup mechanics, not three successful authenticated readiness probes.

This is an evidence/implementation correctness defect relative to D064, not a cryptographic break. Do not call the current CLI path D064-ready in release evidence and do not run a VPS warm row until this is repaired.

### R-003 HIGH — `resource_admitted=true` is asserted, not derived from a bounded admission result

D064 requires a warm candidate to be authenticated, independently validated, **resource-admitted**, and ready. Current CLI sets the readiness observation's `resource_admitted` field to literal `true`; no admission result is being observed at the readiness point.

The repair must derive this bit/state from a real bounded admission path already present in the runtime, or add the smallest explicit bounded admission step needed for the warm control channel. A boolean constant is not release evidence.

### R-004 HIGH — warm timing fields cannot currently prove pre-failure preparation

`failure_observation_started` is created **after** warm TCP setup. Later `failover_timing` expresses `tcp_connect_started_us`, `tcp_connected_us`, `tcp_negotiated_us`, `tcp_authenticated_us`, `resume_validated_us`, and `readiness_satisfied_us` relative to `failure_observation_started`. For a warm path those events occurred earlier, so the diagnostic origin cannot faithfully quantify how far before failure they occurred.

Additionally, `readiness_satisfied_us` is currently populated from the same `resume_validated`/authentication instant rather than the timestamp of the third successful readiness observation.

Event order in stdout is useful, but the planned VPS evidence explicitly needs trustworthy timestamps. Introduce a common experiment origin before warm setup and record the actual third readiness success separately from authentication/resume validation.

### R-005 MEDIUM — status wording currently overstates the executable readiness source

`docs/status.md` says warm TCP is “resource-admitted and marked warm after three distinct bound observations before controlled UDP failure.” The timing/order part is directionally true, but “observations” currently means three local manager calls with asserted booleans, not D064 authenticated readiness exchanges.

Until R-002/R-003 are repaired, status should describe this as a **candidate pre-failure warm state seam with synthetic/local readiness inputs**, not as completed D064 readiness evidence.

### R-006 NOTE — VPS backlog remains real and should resume immediately after the repair

The repository already has older self-owned cross-host TCP/UDP, controlled resume, lifecycle, resource-sampler, and periodic-session evidence, but those rows precede the new warm implementation. IPv6 remains environment-blocked and should not be mechanically retried without an actual path change.

HY2 v2.9.3 remains pinned and the comparison methodology exists; no valid equal-application paired Nekomusume/HY2 result has landed yet. This remains a high-value rental-window target once the current correctness seam is green.

## Evidence boundaries

- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` remain correct.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only; this warm-readiness repair is failover/resume/runtime work and does not automatically reopen N9.
- `9d89051` provides real code/tests for pre-failure warm candidate state, but no real VPS/WAN evidence.
- The current three readiness IDs are locally synthesized manager inputs, not peer-observed authenticated challenge/response successes.
- Existing older VPS evidence remains valid only for the exact commits/scenarios recorded; it does not prove the new warm path.
- The existing cold recovery path remains separately useful and should not be relabeled warm.
- Standing authorization already covers the owned-endpoint TCP/UDP readiness/failover experiments, bounded periodic Session runs, resource sampling, cleanup, and fair HY2 comparison described below.
- The VPS rental window is time-limited; after the readiness repair, current-exact-head VPS evidence outranks unrelated local polish.
- IPv6 remains blocked by the previously observed absence of a real owned end-to-end IPv6 path; do not repeat unchanged failures.

## Work Package — make D064 readiness real, then spend the VPS window on exact-head evidence

Execute A -> B -> C -> D -> E in dependency order. This is deliberately thicker than a single small repair. If A/B consume the cycle, continue into C on the same package when green; do not stop merely because one coherent commit is complete.

### Primary A — replace synthetic readiness with a real bounded authenticated readiness exchange

**Goal:** make each of the three D064 readiness successes originate from actual peer-observed authenticated control traffic on the pre-established TCP fallback while UDP remains the sole Session-data owner.

Before implementation, inspect the existing authenticated process/control seams and reuse the smallest safe one. Do not invent a parallel Session protocol if an existing encrypted control channel can carry the readiness exchange.

Required behavior:

1. TCP fallback is connected, canonically negotiated and Noise-authenticated while UDP remains active.
2. Readiness uses an explicit bounded request/response exchange over that authenticated fallback. Each success must be bound in the authenticated payload/state to at least:
   - Session identity;
   - target PathId / PathGeneration;
   - delivery epoch;
   - a distinct bounded observation/challenge ID.
3. The server echoes/acknowledges only the exact current tuple after its bounded admission conditions are satisfied. Client-side literal `resource_admitted=true` is not acceptable evidence.
4. Only successfully decrypted, tuple-matching, current-generation responses advance readiness. Duplicate response IDs, stale/wrong session/generation/epoch, malformed/authentication failure, timeout, and unadmitted responses cannot advance the gate.
5. `k_ready=3` means three actual consecutive successful readiness exchanges for the candidate. A failed current-candidate readiness attempt must leave/reset readiness according to one documented deterministic policy; it must not silently preserve a misleading “consecutive” count.
6. Readiness traffic is control/resume only. The standby TCP channel carries **zero new application data** before manager promotion.
7. Bound the readiness exchange: payload size, outstanding probes, total observations, deadline/rate, and responder bytes. Preserve D064's amplification/resource boundary; do not create an unbounded ping loop.
8. Keep the existing cold path intact when no eligible warm candidate exists.
9. Do not change the frozen canonical vector corpus merely to hide this runtime gap. If a new network-visible process control message is required, document/specify it in the appropriate failover/runtime contract and add codec/parser tests; N9 corpus scope remains frozen only as already declared.

If there truly is no implementation path for an authenticated readiness control exchange without a new architectural choice, preserve the exact gap and use the Fallback below rather than fabricating observations.

### Follow-up B — make warm evidence timing and admission provenance truthful

**Dependency:** A present.

Introduce one common experiment origin **before** warm TCP preparation and keep the existing failure-observation-window origin separately.

Diagnostics/evidence must make these stages individually observable:

- TCP connect started / connected;
- canonical negotiation complete;
- Noise authentication complete;
- resume binding/validation complete;
- bounded resource admission complete;
- readiness probe 1/2/3 request/response success times;
- exact `warm_eligible_at` / `readiness_satisfied_at` from the third valid response;
- UDP failure observation start;
- UDP failure decision;
- manager promotion/new active;
- first resumed logical data accepted/acknowledged.

Use a common monotonic relative origin so warm setup times are measurably **before** failure decision rather than collapsing to zero. `readiness_satisfied_us` must come from the third successful readiness exchange, not from the authentication timestamp.

For cold recovery, preserve the separate cold timing/classification and do not invent warm-only fields.

Update `docs/status.md` only to the evidence actually present: until real readiness exchange tests pass, do not say D064 readiness is complete.

### Follow-up C — deterministic manager/process regressions for real readiness

**Dependency:** A/B green.

Add tests that exercise actual readiness I/O and preserve manager safety. At minimum:

- three successful encrypted readiness exchanges occur before failure decision and before any TCP application data;
- two successes are insufficient to become warm;
- replay/duplicate observation response cannot count twice;
- stale/wrong target/generation/session/epoch response cannot count;
- malformed/tampered/unauthenticated response fails closed;
- current-candidate timeout/failure follows the documented consecutive-readiness reset policy;
- resource admission failure prevents warm state;
- readiness response budget/rate/outstanding limits are bounded;
- UDP remains sole active data owner before failure;
- failure with valid warm candidate promotes that exact generation;
- no eligible warm candidate keeps the existing cold path and the `4b95e96` cold evidence labels;
- failed warm attempt is retained as failed-warm evidence and any later replacement generation starts from standby/cold semantics;
- uncertain resend + exact authenticated Session DeliveryAck + receiver dedup/conflict rules remain correct after both warm and cold promotion;
- timing tests parse numeric fields and prove warm setup/readiness happened before `failure_decided_at`, while first resumed data happens after promotion.

If process-control decoding changes, include malformed/boundary/property coverage and run the relevant fuzz smoke required by `AGENTS.md`.

Run full local gates before any VPS use:

- `cargo fmt --all -- --check`;
- `cargo check --workspace --locked`;
- `cargo test --workspace --all-targets --locked --no-fail-fast`;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- `bash scripts/check.sh`;
- `git diff --check`;
- relevant fuzz smoke if external parser/control decoding changed.

### Follow-up D — exact-head owned-VPS warm/cold + periodic/resource evidence batch

**Dependency:** A-C green. Build/deploy one exact commit and reuse that exact binary for compatible behavior rows. This is now the highest-value rental-window work.

Within standing authorization, run scientifically distinct bounded scenarios:

#### D1 — D064 warm automatic recovery

On the self-owned client/VPS pair, establish the fallback TCP candidate and obtain three **real authenticated readiness responses before UDP failure decision**. Then trigger the existing controlled UDP reply-cessation seam.

Record:

- exact git commit and binary SHA-256;
- ports/actual limits/durations and endpoint ownership classification;
- negotiation + Noise identities without secrets;
- connect/auth/resume/admission timestamps;
- all three readiness response IDs/timestamps and `warm_eligible_at`;
- proof UDP remained sole data owner before failure;
- failure observation windows + decision timestamp;
- promotion of the already-warm generation;
- `fallback_class=warm`;
- uncertain resend / exact authenticated DeliveryAck / receiver dedup-conflict result;
- logical records/bytes, missing/duplicate/conflict counts;
- recovery latency, CPU/RSS/FD/socket sample where available;
- cleanup verification.

Classification must remain: controlled owned-endpoint application-level UDP reply cessation -> threshold decision -> pre-established authenticated warm TCP recovery. Not natural Internet blackhole evidence, public reachability, production failover, or security approval.

#### D2 — same-head cold control

Run one bounded cold control from the same exact binary/config question if useful, preserving connect/auth inside the recovery interval and `fallback_class=cold`. Do not repeat merely to chase a better latency number.

#### D3 — current-head periodic Session/resource sample

If the periodic runner remains green on this exact commit, run one distinct ~5 minute bounded authenticated periodic Session (single run <10 min, low traffic) with process resource sampler. This updates the time-limited real-socket evidence to the current failover/runtime baseline without claiming production long-lived stability.

Do not rerun unchanged IPv6 failure. Preserve every negative row and cleanup state.

### Follow-up E — close the first valid HY2 equal-application paired sample, then reconcile ledgers

**Dependency:** local exact-head green; independent of D1 once a trustworthy current binary exists.

Reuse the pinned HY2 v2.9.3 artifact and existing comparison contract. Do not touch the existing production Hysteria service/config and do not weaken isolation/target guards.

The pair must answer the same application question:

```text
send exact deterministic payload bytes -> receive the exact same bytes
```

Use same owned client/VPS pair, close time window, payload file/length/hash, route/MTU metadata, authenticated+encrypted security class, single-stream/load shape, finite timeout/run count, and experiment-only ports/credentials. Prefer a small interleaved/nearby sample (for example 5 paired runs) if the current harness supports it.

Record raw samples, median/P95/failures, CPU user/system, max RSS, FD count and application bytes. `wire_bytes` stays null unless capture provenance is trustworthy. Preserve slower/failed Nekomusume results; make no superiority claim.

If the exact Nekomusume equal-workload adapter is still missing, implement only that smallest benchmark adapter first. If the environment genuinely blocks a fair pair, record the exact blocker and use remaining VPS time for another already-defined row (package/operator smoke, current-head lifecycle, process-resource validation), not speculative features.

After D/E, update release-evidence/status/navigation only to what actually ran:

- link current-exact-head warm evidence only if D1 proves real pre-failure readiness exchange;
- keep cold recovery distinct;
- retain older rows as historical exact-commit evidence, not substitutes for current HEAD;
- keep IPv6 environment-blocked unless the actual path changes;
- mark HY2 comparison complete only after a semantically equal paired sample;
- keep `IMPLEMENTATION_PLAN.md` item 3 open until remaining genuine matrix rows are closed or explicitly reviewed as environment-inapplicable;
- preserve negative/superseded evidence;
- keep RC/security/production/global-freeze flags unchanged.

## Completion gates

This package is complete only when:

- warm candidate state remains pre-failure and single-active;
- all three readiness successes come from actual authenticated peer responses, not local booleans;
- resource admission is derived from a real bounded admission result;
- readiness is session/path-generation/delivery-epoch/observation-ID bound and replay/duplicate/stale/malformed input cannot advance it;
- readiness/control traffic is bounded and carries no new application data before promotion;
- warm timing truthfully shows readiness before failure decision and first resumed data after promotion;
- cold recovery remains available and separately classified;
- deterministic process/manager tests and full local gates pass;
- current-exact-head VPS warm evidence is captured or a new evidence-backed blocker is recorded;
- a same-head cold control and/or periodic resource row is captured where scientifically useful;
- the HY2 equal-application seam is materially advanced, preferably through the first valid paired sample;
- status/evidence docs claim only what actually ran;
- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` remain unchanged.

## Fallback

If implementing a real authenticated readiness exchange genuinely requires a new unresolved architecture decision rather than an implementation detail already selected by D064:

1. keep the current manager warm-state seam but classify readiness inputs as synthetic/local only;
2. do not run/claim D064 warm VPS evidence;
3. preserve the specific protocol-choice gap in the normal decision process;
4. spend the rented-VPS window on READY independent rows: current-head periodic Session/resource sampling, equal-application HY2 comparison, package/operator/lifecycle evidence, and existing cold recovery classification;
5. do not ask for new WAN permission already covered by standing authorization.

If D1 fails for a runtime reason after A-C are green, preserve the exact failed row and use only a changed diagnostic variable before retrying. D3 and E remain independently READY.

## Do not expand into

- calling three local manager mutations “authenticated readiness probes”;
- marking literal `resource_admitted=true` as evidence;
- changing frozen canonical corpus bytes to avoid runtime failover work;
- rewriting D064 as cold-fallback-only without an explicit superseding decision;
- striping/aggregation, enabled FEC, 0-RTT or exotic carriers without observed-problem gates;
- unchanged IPv6 retries;
- third-party targets, scanning, production route/firewall/DNS/proxy/tunnel/qdisc changes, or experiments outside standing authorization;
- RC/security/production approval or performance-superiority claims.

## Questions requiring maintainer decision

none.
