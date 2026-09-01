# Nekomusume ChatGPT Handoff

Checked at: 2026-09-01 18:00 Asia/Shanghai
Repository HEAD: `e24cc8d26f2c0652cb0d836234ac1c6929d1457a`
Previous checked implementation HEAD: `c7e0a211cbc74f065d03b374bd3cc1bbf2a97356`
Previous reviewer handoff commit: `8c835601bcce7eee38bc71b1baab8032d1245036`

## What changed

Two substantive coding-agent commits landed after the previous reviewed implementation state:

- `6b8a931` — **implementation/tests**. `CarrierManager::fail_udp_to_tcp()` no longer immediately promotes the TCP fallback. It creates a pending switch, clears the active owner, and adds target/generation-scoped `PromotionEvidence`; invalid target/generation/auth/resume/readiness fields fail closed in deterministic tests. This materially repairs the previous pre-connect promotion defect.
- `e24cc8d` — **implementation/tests**. It adds a stateful bounded TCP `FramedReader` that preserves partial header/payload bytes across socket poll timeouts, adds a one-second `HealthObservationWindow` so short UDP socket timeouts are only polling quanta, distinguishes wrong-peer/malformed/stale traffic from exact permitted progress, and integrates the framed reader into the periodic authenticated TCP Session runner. This materially repairs the prior periodic framing defect and the prior “100 ms poll == one health failure” defect.

The project remains in `IMPLEMENTATION_PLAN.md` item 3, **Bounded release evidence matrix**. N9 and negotiation-path completion remain closed; no RC/security/production state changed.

No new exact-head VPS replacement evidence was committed after these repairs. The latest implementation HEAD has no attached GitHub commit-status/CI checks through the available GitHub status API; local gates remain coding-environment evidence rather than independent CI attestation.

## Review verdict

**CONTINUE WITH REQUIRED FIXES — R-001 and R-003 from the previous handoff are substantially repaired; fallback promotion is structurally improved, but the accepted-readiness subclaim is still not truthfully observed. One small resource-budget defect also remains. The periodic current-head VPS run is now READY and should be harvested immediately during the rental window.**

Do not roll back the new pending-switch or stateful-framing work. Do not rerun the old pre-repair failover row as release evidence. Close the remaining readiness/evidence defect, then collect current-exact-HEAD VPS evidence in the same batch.

## Review findings

### R-001 HIGH — TCP fallback readiness is still caller-asserted rather than observed

`6b8a931` correctly separates failure from promotion and scopes promotion to the pending target/generation. However, the current failover runner still does this immediately after Noise/resume setup:

```text
let resume_validated = tcp_authenticated;
let readiness_satisfied = resume_validated;
promote_failed_udp_target(PromotionEvidence {
    target_path: PathId(2),
    generation: PathGeneration(1),
    authenticated: true,
    resume_validated: true,
    readiness_observations: 3,
})
```

The three readiness observations are therefore a literal caller-supplied number, not three observed target-path readiness events. `readiness_satisfied` is also currently equal to the authentication timestamp. The manager checks the shape of evidence but does not own or verify its provenance.

This is better than pre-connect activation, but it still cannot support a claim that the accepted target-readiness gate was actually exercised. A code path that can write `readiness_observations: 3` without three observations has not proven readiness hysteresis.

**Required correction:** promotion evidence must be produced by a bounded stateful readiness tracker tied to the pending target and pending generation, or the runtime/evidence claim must be narrowed so it does not say the target satisfied a multi-observation readiness gate. Do not invent new protocol bytes merely to manufacture three observations.

### R-002 HIGH — the current handoff relies on a “D064” contract that is not present in the repository decision ledger

`docs/decisions.md` currently goes from D063 directly to D065. The previous reviewer handoff repeatedly called the failover/readiness contract “D064” and treated `k_failure=3` / `k_ready=3` as accepted architecture, but `AGENTS.md` explicitly says architecture changes must not live only in chat/commit messages/handoffs.

The coordination file is not a normative source. Before the repository can make a release-matrix claim whose meaning depends on this readiness contract, the agent must reconcile the provenance:

1. search git history and tracked specs/ADRs for the actual accepted source of the missing D064 semantics;
2. if a tracked accepted source exists, restore/link it without changing semantics;
3. if no such source exists, do **not** silently promote reviewer prose into protocol architecture. Keep the evidence class narrower (“authenticated cold recovery succeeded; manager target/generation scoping enforced”) until the readiness semantics are recorded in the normal decision/spec hierarchy.

This is a governance/spec-drift defect, not a reason to stop unrelated READY VPS evidence such as the periodic Session run.

### R-003 MEDIUM — `max_ignored` caps diagnostics/counter value, not health-path packet processing

`HealthObservationWindow` now prevents wrong-peer/malformed/stale traffic from advancing or resetting health state and prevents packet rate from making the deadline occur earlier. That closes the main correctness defect.

But `max_ignored` currently uses a saturating counter only. Once the count reaches the configured maximum, the loop can still keep dequeuing and classifying an arbitrary number of junk datagrams until the one-second deadline. The observation is time-bounded, but the advertised bounded admission budget is not actually a packet-processing bound.

For release/security review quality, add a fail-closed bounded policy that does not let junk force failover. Acceptable direction: on health-admission budget exhaustion, abort/mark that health observation as `admission_budget_exhausted` (or equivalent diagnostic failure of the experiment/path-assessment mechanism) without converting the junk count into a health failure/success. Do not let hitting the junk budget accelerate the TCP fallback transition.

This should be a small local hardening slice; do not turn it into a general DoS framework.

### R-004 PASS — periodic partial-frame state is now preserved

The new `FramedReader` preserves partial length-header and payload bytes across `TimedOut`/`WouldBlock`, enforces maximum length before payload allocation, distinguishes clean EOF / partial truncation / idle deadline, uses caller absolute deadlines, and is integrated into the periodic handshake/data/DeliveryAck path. The deterministic tests deliberately fragment header/payload around timeout gaps.

This closes the previous periodic framing blocker sufficiently to make a **current-exact-HEAD bounded periodic VPS run READY**. It is not evidence until rerun on the repaired commit family.

## Evidence boundaries

- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, `RELEASED=false` remain correct.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `6b8a931` proves pending-switch target/generation validation and no pre-connect active owner in deterministic code/tests; it does **not** yet prove three real readiness observations occurred.
- `e24cc8d` proves a bounded stateful TCP framing mechanism and truthful one-second health observation windows in local code/tests; it does **not** itself create VPS/WAN evidence.
- Existing historical five-minute periodic evidence remains valid only for its recorded source/binary; it is not current-head evidence.
- Existing previous automatic-failover VPS rows remain valuable application-recovery evidence but must not be reinterpreted as proof of the repaired target-readiness contract.
- IPv6 remains environment-blocked unless the owned endpoint environment actually changes; do not spend rental time repeating unchanged IPv6 failures.
- Standing VPS authorization remains active. Current-head periodic TCP, current-head bounded failover replacement, process-resource sampling, cleanup, and the already-authorized HY2 comparison are not waiting on maintainer permission.
- No superiority, public/general reachability, production, security-audit, or RC claim follows from any one replacement row.

## Work Package — close readiness provenance, then harvest exact-head VPS evidence

Execute A -> B -> C -> D -> E in dependency order where applicable. This is intentionally a thick package; do not stop after the first small helper if the next step is already READY.

### Primary A — Make fallback readiness evidence real and bounded

#### A1 — Reconcile the missing D064 provenance before encoding more semantics

Search repository history/specs/ADRs for the source that actually established the current failure/readiness thresholds and activation semantics.

- If a tracked accepted source exists: restore/link/index it and make code/tests reference that contract.
- If no accepted source exists: do not invent a new three-observation network protocol solely from the old reviewer handoff. Keep the runtime claim narrow and create/repair only the minimum governance record needed to describe already-existing manager semantics. If multiple materially different readiness meanings remain, do not choose one silently; leave the multi-observation `active` subclaim blocked and continue the independent periodic/HY2 VPS work below.

The reviewer handoff itself must never become the normative architecture source.

#### A2 — Replace caller-authored readiness count with stateful target/generation evidence

For any path that claims a multi-observation readiness gate:

- readiness state must live in a bounded object keyed to the pending target path and pending generation;
- authentication and resume validation are prerequisites, not substitute readiness observations unless the tracked decision/spec explicitly defines them as such;
- stale/wrong target/wrong generation events cannot advance readiness;
- duplicate replay of the same evidence cannot advance the counter twice;
- failed/rejected target attempts cannot leak readiness into a later generation;
- `CarrierManager` remains the sole active-owner selector;
- promotion consumes/validates stateful evidence rather than trusting a freely constructed integer.

If the truthful tracked contract is only “authenticated + resume-validated cold recovery” and contains no `k_ready=3` requirement, remove the fabricated readiness-count claim rather than preserving a magic constant.

Required deterministic tests:

- no promotion from a fabricated `readiness_observations=3`-style caller assertion;
- wrong target/generation rejected atomically;
- duplicate/stale evidence does not accumulate;
- rejected target attempt does not pollute the next generation;
- promotion occurs exactly once after the tracked readiness contract is truly satisfied;
- at most one active owner is visible;
- uncertain replay/dedup state survives failed target attempts.

#### A3 — Bound ignored health admission without converting junk into a path failure

Turn the current diagnostic-only `max_ignored` into a real bounded processing/admission rule for the health observation mechanism.

Required behavior:

- wrong-peer/malformed/stale traffic still does not advance/reset health;
- reaching the junk/admission budget must not count as one of the three path-health failures and must not accelerate fallback;
- fail closed or yield a distinct bounded diagnostic result rather than spinning indefinitely on queued junk;
- exact permitted authenticated progress remains detectable within the observation contract;
- resource bounds and reason labels are deterministic/tested.

Required tests include high junk count, junk followed by exact permitted progress, budget exhaustion, and proof that transition timing/counter cannot be attacker-accelerated.

#### A4 — Local full gate

Run at minimum:

- targeted CarrierManager pending/promotion tests;
- target-readiness provenance/generation tests;
- health-window junk/admission-budget tests;
- periodic FramedReader fragmentation/deadline tests;
- focused failover process tests;
- `cargo fmt --all -- --check`;
- `cargo check --workspace --locked`;
- `cargo test --workspace --all-targets --locked --no-fail-fast`;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- `bash scripts/check.sh`;
- `git diff --check`.

Run fuzz smoke only if wire/parser behavior changes or the repository gate requires it.

### Follow-up B — current-exact-HEAD five-minute periodic Session VPS run

**Dependency:** periodic framing repair is already present; run after A if A changes the shared binary so the evidence points at one final exact commit.

Use the self-owned client/VPS path under standing authorization and the existing process-resource sampler. Reproduce a bounded profile comparable to the historical five-minute row, for example the existing 60-record / 32-byte / ~5-second interval profile if that remains the documented workload.

Record at minimum:

- experiment ID, exact git commit, client/server binary SHA-256;
- actual TCP port, duration/count/bytes/interval/ACK deadline;
- authenticated negotiation/Noise/Session success;
- attempted/confirmed/missing/duplicate records and application bytes;
- P50/P95 confirmation latency already emitted by the runner;
- server and client elapsed, CPU user/system, max RSS, peak FD, owned socket count where available;
- signal/exit status;
- cleanup verification: no experimental listener/process remains.

Acceptance for this row is **current-head bounded periodic authenticated Session evidence**, not “production long-lived stability”. Preserve any failure exactly; do not rerun unchanged until there is a new diagnostic variable.

### Follow-up C — replacement automatic-degradation -> TCP recovery VPS row

**Dependency:** A1/A2/A3 green and the exact runtime claim is now defined truthfully.

Run one small self-owned current-exact-commit automatic-degradation row using the existing explicit UDP reply-cessation seam. Record the complete current semantics:

- failure observation window start and each failed observation timestamp;
- ignored/admission-budget diagnostics without promoting them to health evidence;
- manager pending-switch creation and target/generation;
- TCP connect, canonical negotiation, Noise authentication, resume validation;
- each **real** readiness event required by the tracked contract, if any;
- manager promotion/new-active timestamp only after the actual gate;
- uncertain ranges resent, deduplicated/conflict-free final application bytes;
- first resumed logical data accepted and cold recovery interval;
- process-resource samples if practical without obscuring the functional evidence;
- cleanup verification.

Keep the fault classification explicit: this is a bounded controlled application-level UDP reply-cessation scenario on owned endpoints, not natural Internet blackhole detection or general reachability.

If the readiness contract remains intentionally unresolved after A1, do not fake this row. Preserve the narrower existing application-recovery evidence and move directly to D.

### Follow-up D — close the Nekomusume side of the HY2 equal-application comparison seam

**Dependency:** independent of the fallback-readiness subclaim once the repository is locally green.

The HY2 v2.9.3 artifact, hash, and forwarding seam are already pinned. The remaining high-value rental-window gap is an equivalent Nekomusume application command/orchestrator that can satisfy `docs/bench/hy2-comparison-workload.md` without semantic cheating.

Implement/verify the minimum adapter so both implementations receive the same deterministic payload/file, exact application byte count/hash, bounded timeout, same owned client/VPS route/time window/MTU/security class/load metadata, and emit the required comparison JSON fields.

Do not compare a one-record echo with a streaming/forwarding HY2 workload and call it fair. If the current Nekomusume runtime cannot yet express the same application workload, document the exact missing seam and implement only that seam; do not redesign the transport.

Run local/loopback contract validation first. If an actually equivalent pair is READY in the same cycle, take a small first paired VPS sample (>= existing harness minimum runs) with raw samples, median/P95/failures, CPU/RSS/FD/application bytes. `wire_bytes` stays null unless capture metadata is trustworthy. No superiority claim.

### Follow-up E — evidence/status reconciliation

After B/C/D results exist, update normal evidence documents and `docs/status.md`/`ROADMAP.md` only to the level actually proven.

In particular:

- link the new current-head periodic evidence if successful, but do not upgrade to production/sustained/general-WAN wording;
- link the repaired automatic-failover evidence only if C actually passed the tracked readiness contract;
- leave IPv6 blocked if the environment is unchanged;
- preserve all negative rows and supersession relationships;
- do not check HY2 comparison complete unless an equal-application paired run actually occurred;
- keep RC/security/production flags unchanged.

## Completion gates

This batch is complete only when:

- readiness semantics have a real tracked normative/governance source rather than reviewer-only “D064” prose;
- promotion evidence cannot be satisfied by a caller writing a magic readiness count;
- target/generation/stale/duplicate readiness evidence is fail-closed;
- junk/admission traffic has a real bounded processing policy and cannot become health success/failure;
- the full local repository gate passes;
- a current-exact-commit five-minute periodic VPS row is captured or a new evidence-backed blocker is recorded;
- a replacement automatic-degradation row is captured if and only if the readiness contract is truthfully executable;
- the HY2 equal-application seam is materially closer to an executable paired run, preferably with the first small paired sample if dependencies are satisfied;
- cleanup and negative-result retention are verified;
- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, `RELEASED=false` remain unchanged.

## Fallback

If A1 shows there is genuinely no accepted readiness contract and multiple reasonable activation semantics exist:

- do not invent one;
- keep automatic manager-active release evidence blocked at that subclaim;
- continue B (periodic current-head VPS evidence), D (HY2 equal-application seam), process-resource evidence, package/operator evidence, and other independent bounded rental-window work;
- record exactly what architectural decision is missing for a later maintainer/reviewer decision.

If the periodic current-head run fails, preserve the exact failure and use its new logs/resource/framing state as the next diagnostic variable rather than mechanically rerunning.

## Do not expand into

- public/general reachability claims;
- production exposure or production network changes;
- third-party targets or scanning;
- >10-minute runs, >256 MiB single-run traffic, or >32 experimental Sessions without new authorization;
- enabled FEC, 0-RTT, striping/aggregation, exotic carriers, or unrelated feature work;
- one-off HY2 superiority claims;
- silently treating the reviewer handoff as a protocol/architecture specification.

## Questions requiring maintainer decision

none at this moment. If A1 confirms that no tracked readiness contract exists and there are multiple materially different architecture choices that cannot be resolved from existing repository facts, record that as the only maintainer-decision candidate while continuing the independent READY VPS work above.
