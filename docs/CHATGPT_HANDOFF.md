# Nekomusume ChatGPT Handoff

Checked at: 2026-09-04 07:00 Asia/Shanghai
Repository HEAD reviewed: `58c9bc71e73afc4b600614bbd5d61af033a28f54`
Previous reviewed coding/evidence HEAD: `d7553374ccb375dfced435caa933ba7fdc3d131e`
Previous reviewer handoff commit: `bed2940298cf00ce61b419a665c5ac1d67402b22`

## What changed

The external agent consumed A-E from the previous rolling queue quickly and in order.

- `b52734a` — **R-010 evidence-semantic repair; no transport/session/wire/Noise behavior change.** Arbitrary HY2 failure prose no longer promotes `last_success_stage`. Promotion now requires typed monotonic `nekomusume.hy2-stage-evidence.v1` events from constrained sources, and adversarial strings such as `QUIC handshake failed`, `UDP timeout`, `not authenticated`, `authentication failed`, and `connected to server: false` remain at the harness-known baseline.
- `61a6490` — **local first-pair contract rehearsal only.** It proves a valid Nekomusume first sample followed by representative HY2 failure classes cannot produce a comparative summary, while preserving pinned HY2 v2.9.3 identity, 1200-byte workload and five-run contract.
- After exact-B preparation, one changed-hypothesis C outer invocation was consumed. It stopped locally at port-range preflight (exit 2) before any VPS deployment. There were zero live samples, zero comparison/result statistics and zero runtime evidence; cleanup residue remained zero. This is a local orchestration negative, not a VPS/HY2 runtime result.
- `88f4a6d` — **status/ledger reconciliation only.** It records the current HY2 line as `BLOCKED_ORCHESTRATION_CURRENT_LINE_HY2` and preserves historical attempts separately.
- `3649492` — **release-evidence closure-index construction; no runtime behavior or VPS evidence.** It adds per-track classifications and a closure section to `docs/era4-ledger-2026-08-30.json`, then mirrors them into `docs/status.md`. Exact `3649492` GitHub Actions run `33815057421` is green: both `stable checks` and `nightly decode fuzz smoke` succeeded.
- `58c9bc7` — **independent-review packet preparation only.** It adds `docs/release-security-review-packet.md`, indexing canonical corpus, negotiation/Noise, parser/fuzz, pre-auth limits, ACK/carrier separation, failover, package/operator, VPS/HY2 and unresolved release gates. Its exact GitHub Actions run `33815657548` had started at review time and was still in progress; do not treat pending CI as green.

R-010 is closed. The changed-hypothesis HY2 line is also closed at a local preflight boundary and must not be retried unchanged. No new live/VPS behavior evidence was added in this delta.

Review found a new release-evidence classification defect that must be repaired before the closure ledger or review packet can drive release/security decisions.

### R-011 HIGH — `OPEN_READY` conflates “bounded evidence exists/is sufficient” with “real missing work is executable now”

`3649492` classifies A/B/C/D/E/F/G/H/I/J/L/M/N/O/T as `OPEN_READY`, including tracks already marked `era3-complete`, long-established local/candidate components, and bounded questions whose existing evidence is already sufficient for their declared scope. The closure then exports the same IDs in `open_ready_rows`.

That is not the meaning required by the rolling release queue. `OPEN_READY` is supposed to identify a **specific unresolved release-evidence question that can truthfully be advanced now**. A track being locally testable, candidate, or historically complete does not make it outstanding release work. The current classification can therefore cause duplicate implementation/testing, reopen closed Era-3 work, and hide the actual distinction between:

```text
bounded question already sufficiently evidenced
vs
missing release evidence that is executable now
vs
blocked by orchestration / implementation / environment / governance
```

The newly prepared review packet compounds this by calling the machine-readable closure authoritative while also saying there is no actionable live row. Those statements can coexist only if `OPEN_READY` is redefined so weakly that it is no longer useful as a work/opportunity classification.

This is an evidence/governance semantics defect, not a runtime protocol bug.

### R-012 MEDIUM — release navigation still contains stale/over-broad blocker text

`IMPLEMENTATION_PLAN.md` retains older prose such as HY2 being diagnostics-blocked and the repeated-failover command-array seam being the next executable path even though the current lines have since been closed/reclassified. The Era-4 ledger also retains a broad reachability `gate: "new authorization required"`, despite standing authorization already covering bounded self-owned TCP/UDP IPv4/IPv6 reachability and ordinary diagnostic execution; only the parts outside standing authorization require a new grant.

Do not erase historical negatives. Repair only current navigation/classification so an agent cannot recreate a fake generic WAN-authorization blocker or follow a superseded orchestration seam.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — R-010 and the HY2 changed-hypothesis line are closed; repair R-011/R-012 before treating the closure ledger or review packet as authoritative, then continue the local independent-release/security review queue without waiting between pre-authorized slices.**

The project is not globally blocked. The rented VPS remains a priority asset, but at this exact review there is no proven dependency-ready live row: repeated warm failover, periodic and HY2 current lines are closed orchestration lines; NAT/source change, live migration-back, live key update and live PMTUD are implementation-blocked; owned end-to-end IPv6 is environment-blocked. Do not manufacture a VPS run merely to consume rental time. A corrected closure audit may reveal a genuine `OPEN_READY` live row; if so, that row immediately outranks local polish under `docs/vps-rental-window-priority.md`.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only; it does not freeze the global protocol, Noise, carrier packetization or release.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- Exact `25e0daa` remains the accepted bounded positive controlled warm fallback and one approximately five-minute periodic direct-path sample; neither proves natural loss, a reliability rate or production readiness.
- Repeated warm failover current line remains closed at its retained orchestration/evidence-collection negative; no unchanged retry.
- Periodic current line remains closed at its retained pre-application/transport negative with R-009 erratum; no unchanged retry.
- HY2 current line is now `BLOCKED_ORCHESTRATION_CURRENT_LINE_HY2`: exact `61a6490` follow-up stopped at local port-range preflight before VPS deployment. It provides no HY2 runtime, QUIC, TLS/authentication, application, paired-sample or performance evidence.
- R-010 is closed for future HY2 diagnostic metadata; arbitrary error text cannot promote a lifecycle success stage.
- Exact `3649492` has green independent GitHub stable checks + nightly decode fuzz smoke. Exact `58c9bc7` CI was still pending when reviewed.
- `docs/release-security-review-packet.md` is preparation/index material only. It is not an independent review, audit, security approval or release decision.
- NAT/source-endpoint change, live migration-back, live key update and live PMTUD remain implementation-blocked under current executable reality. Do not implement them merely to fill a matrix checkbox.
- Owned end-to-end IPv6 remains environment-blocked unless a real owned path appears.
- Standing authorization continues to cover bounded self-owned TCP/UDP/HY2 work inside its limits; a closure ledger cannot narrow or expand that authorization.
- Protected identity material, credentials, private endpoint/topology data and raw private diagnostic bundles remain unread/untracked/uncommitted.

## Rolling Work Queue

This queue is deliberately multi-hour. **Every dependency-satisfied slice below is pre-authorized to start immediately after the previous coherent slice is validated, committed and pushed. Do not stop after one commit, one nominal hour or one reviewer interval.** Pause only for a new BLOCKER/HIGH correctness-security-evidence finding, a core architecture decision, an action outside standing authorization, production impact, a required maintainer value judgment, actual runtime/tool-budget termination, or true queue exhaustion.

### A — Repair release-evidence closure taxonomy and `OPEN_READY` semantics

**Status:** `READY_LOCAL` — highest priority due R-011.

**Goal**

Make the machine-readable closure answer one concrete question: **what unresolved release-evidence work is actually executable now, and what is already sufficient or blocked?**

**Likely files**

- `docs/era4-ledger-2026-08-30.json`;
- `docs/status.md`;
- a small validator/test under `scripts/` if needed;
- `docs/release-security-review-packet.md` only to keep its classification explanation truthful.

**Required behavior**

1. Separate ordinary track implementation status from release-opportunity classification. Do not equate `candidate`, `ready`, or `era3-complete` with `OPEN_READY`.
2. Add/restore an explicit class equivalent to `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION` for rows whose declared bounded question is already adequately evidenced and should not be re-run merely because more local testing is possible.
3. `OPEN_READY` is legal only when the row has a specific unresolved claim, an exact next evidence action, satisfied dependencies, and no current blocker. Every `OPEN_READY` row must carry machine-readable fields equivalent to:
   - `evidence_needed`;
   - `next_action`;
   - `requires`;
   - whether that action is local or VPS/live.
4. `BLOCKED_ORCHESTRATION_CURRENT_LINE` must name the closed current line and what materially new hypothesis/instrumentation would be required before any future attempt.
5. `BLOCKED_IMPLEMENTATION` must not become a request to build speculative features solely for matrix completion.
6. `BLOCKED_ENVIRONMENT` must state the missing environment fact, not generic authorization.
7. `GOVERNANCE_GATE` must identify the actual external/review/value-judgment gate.
8. Do not silently narrow item-3 release scope and do not promote any governance flag.

**Minimum expected correction**

Tracks already complete/sufficient for their bounded question (for example settled Era-3 local capability rows) must not remain in `open_ready_rows` merely because bounded local evidence could be generated again.

**Validation**

Add deterministic validation that fails if:

- an `OPEN_READY` row lacks a concrete next action/evidence target;
- an already-sufficient/closed row is simultaneously listed open-ready without a distinct unresolved claim;
- closure summary arrays disagree with per-row classification;
- a blocked current line has no blocker/evidence-needed explanation.

Run full local gate + `git diff --check`, commit and push.

**Continue immediately to B:** yes.

---

### B — Reconcile stale blocker/navigation text and standing-authorization semantics

**Status:** `READY_LOCAL` after A.

**Goal**

Make `IMPLEMENTATION_PLAN.md`, `ROADMAP.md`, `docs/status.md`, Era-4 ledger and review packet agree on the current line without rewriting historical artifacts.

Required repair:

- remove/supersede stale statements that still name the old repeated-failover command-array seam or HY2 diagnostics-only state as the current next path;
- express HY2 current state as the exact post-`61a6490` local port-range preflight closure;
- preserve periodic/repeated/HY2 historical negatives separately;
- split reachability authorization truthfully: bounded self-owned TCP/UDP IPv4/IPv6 reachability is standing-authorized; third-party targets, exotic raw/public carrier work and production exposure remain outside the ordinary grant;
- keep natural-loss, NAT/source change, migration-back, live key update, live PMTUD and IPv6 classifications truthful;
- keep release item 3 unchecked.

Add a small plan/ledger consistency check if the repository already has an appropriate sync-gate pattern; do not create a second competing status system.

Run full local gate, commit and push.

**Continue immediately to C:** yes.

---

### C — Re-evaluate VPS opportunity from the corrected closure map

**Status:** `PREAUTHORIZED_AUDIT`; live execution only if the corrected map exposes a genuine `OPEN_READY` VPS row already supported by current runtime/instrumentation.

**Goal**

Honor the rental-window priority without inventing work.

After A/B, inspect every unresolved release-evidence row. If one live row satisfies all of:

- current runtime path already exists;
- required instrumentation is truthful;
- hypothesis differs materially from a closed failed line;
- standing authorization covers the execution;
- it answers a declared release-evidence question;
- no exact-head CI dependency is red/pending;

then execute **one smallest meaningful bounded row** and preserve positive or negative evidence honestly.

If none exists, write `READY_LIVE: none` with exact reasons and **do not** spend VPS time on generic baselines, repeated closed lines, speculative key-update/PMTUD/migration implementation, or another HY2 retry.

**Continue immediately to D either way:** yes.

---

### D — Repair and strengthen the independent release/security review packet

**Status:** `READY_LOCAL`; `58c9bc7` is accepted as useful preparation but not yet sufficient as an independent-review packet.

**Goal**

Make the packet a precise review navigator rather than a broad link list, without claiming the independent review itself is complete.

Required improvements:

1. Correct the canonical-corpus boundary: corpus v1 is specifically frozen/content-addressed, while global protocol/release interoperability is not.
2. Replace broad links such as package lifecycle -> `docs/decisions.md` with the narrowest exact package/install/upgrade/rollback evidence and artifact hashes actually used by the repository.
3. For each review area record:
   - exact claim being reviewed;
   - exact evidence/artifact/test/commit;
   - evidence class (local deterministic / netns / bounded VPS / CI / review);
   - what the evidence does **not** prove;
   - unresolved finding/blocker.
4. Link exact current CI identity rather than implying scripts alone constitute executed evidence.
5. Do not call the closure map authoritative until A/B validation passes.
6. Do not state that a packet authorizes or de-authorizes VPS work; standing authorization and current technical readiness are separate facts.

Prefer a compact generated/indexed form over duplicated normative prose.

Run link/consistency checks + full local gate, commit and push.

**Continue immediately to E:** yes.

---

### E — Resource and abuse-limit evidence audit against `SECURITY.md`

**Status:** `READY_LOCAL`.

**Goal**

Independently map the implementation/tests to every explicit security red line before any RC decision.

Audit at minimum:

- per-connection and global memory/CPU/rate bounds;
- UDP amplification / pre-auth admission limits;
- malformed lengths/counts/offsets and allocation bounds;
- unknown version/type/frame handling;
- duplicate/old packet numbers, replay and ResumeGuard behavior;
- nonce/key-phase safety and fail-closed crypto state;
- unauthenticated control data cannot mutate Session/carrier state;
- secret-safe logging and no plaintext payload/key disclosure;
- no open-proxy/default-forwarding behavior;
- cleanup/resource exhaustion failure paths.

Do not duplicate existing tests. If one **small deterministic negative-test gap** is found, add the minimal test and run required gates. If a substantive design/resource limit is missing, record a release/security finding and stop only the affected promotion path; do not invent arbitrary production limits.

Produce/update a review finding/index artifact that the independent review can consume.

**Continue immediately to F if no new BLOCKER/HIGH:** yes.

---

### F — Compatibility, versioning and freeze-boundary audit

**Status:** `READY_LOCAL`.

Review the exact compatibility policy across:

- canonical corpus v1 content-addressed freeze;
- version negotiation supported/current/future rejection;
- downgrade and transcript binding into authentication;
- first-release previous/current policy (no fictional prior frozen release);
- ResumeGuard/replay boundaries across negotiated version;
- what is and is not frozen globally.

Verify docs/tests do not imply that corpus freeze equals full protocol freeze or RC. Add only small deterministic regression coverage if an actual gap exists. Otherwise produce a concise review record/index.

**Continue immediately to G:** yes.

---

### G — Package/operator lifecycle evidence integrity audit

**Status:** `READY_LOCAL`.

Do not rerun N5 merely for freshness. Verify an independent reviewer can trace, by exact commit/artifact/hash where available:

- reproducible x86_64 build/package identity;
- install -> readiness/smoke -> upgrade -> rollback;
- retained external-state boundary without reading protected identity material;
- server readiness/fail-closed startup;
- SIGTERM/SIGINT shutdown where claimed;
- listener/process/temp-path cleanup;
- first-RC x86_64-only architecture boundary.

If evidence is sufficient, produce only a narrow closure/index repair. A new VPS/package rehearsal is allowed only if this audit first identifies one specific missing executable assertion and the run remains inside standing authorization.

**Continue immediately to H:** yes.

---

### H — Evidence provenance / CI / link-integrity audit

**Status:** `READY_LOCAL`.

Given the project has previously encountered stale/nonexistent HEAD identities, audit the release-facing evidence graph mechanically:

- every commit SHA referenced as exact evidence exists;
- every artifact path exists;
- immutable negative artifacts are not retroactively relabeled;
- binary/package hashes are distinct from git commit identity where required;
- current CI claims point to exact head/run/job and correct conclusion;
- handoff/review packet hashes are advisory coordination provenance, not protocol evidence;
- no protected identity/private endpoint material is referenced or exposed.

Prefer a small validator over manual prose if this can be done without creating a second evidence database. Run full gate and push any validator/index repair.

**Continue immediately to I:** yes.

---

### I — Independent-review findings ledger and explicit unresolved-gate checklist

**Status:** `READY_LOCAL`.

Create/update one reviewer-facing findings ledger that distinguishes:

- verified local deterministic evidence;
- verified CI evidence;
- bounded VPS observations;
- unresolved release-evidence matrix gaps;
- security/resource findings;
- methodology limitations;
- items requiring genuinely independent human/external security judgment.

Do not self-mark item 4 complete. Do not label this an audit approval. The purpose is to make the actual independent review finite and inspectable rather than forcing a reviewer to reverse-engineer the repository.

**Continue immediately to J:** yes.

---

### J — Pre-RC blocker synthesis / maintainer decision dossier preparation

**Status:** `READY_LOCAL_PREPARATION`; any release-scope change or RC promotion is **not** pre-authorized.

After A-I, determine whether release item 3 has any truthful remaining `OPEN_READY` path. If all remaining item-3 requirements are only:

- closed orchestration lines with no materially new hypothesis;
- implementation/architecture work that should not be built merely to fill a checkbox;
- unavailable environment such as IPv6;
- or governance/independent-review decisions,

prepare a concise dossier with three options **without choosing one**:

1. keep the current release-evidence scope and remain pre-RC;
2. define a narrower first-RC evidence claim, explicitly deferring named rows;
3. authorize/plan the implementation/environment work required to satisfy the current scope.

For each option state which existing evidence remains valid, what new work is required, compatibility/security consequences and whether the rented VPS still has a meaningful role.

Do not modify release scope, `RELEASE_CANDIDATE`, `PRODUCTION_READY`, global `FREEZE` or `RELEASED` without a later explicit reviewed maintainer decision.

**Stop after J only if this is genuinely the first remaining gate requiring maintainer value judgment.**

## Completion gates for this rolling queue

- `OPEN_READY` means a concrete unresolved executable evidence action, not “this component exists/can be tested again”.
- Already-sufficient bounded questions are represented distinctly and are not requeued for duplicate work.
- Closure summary arrays are mechanically consistent with per-row classifications.
- Current periodic/repeated/HY2 lines are not retried unchanged.
- Standing authorization is not weakened or replaced by stale generic `new authorization required` wording.
- Review packet records exact claim/evidence/boundary and does not self-certify independent review.
- Resource/abuse, compatibility/freeze, package/operator and provenance/CI evidence are independently navigable with explicit unresolved gaps.
- A live VPS action occurs only if the corrected closure map reveals a real dependency-ready row; otherwise rental time is not wasted on scientifically duplicate runs.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged throughout these preparation/audit slices.
- Protected `neko-server.identity`, credentials, private topology and raw private logs remain unread/untracked/uncommitted.

## Do not expand into

- another unchanged repeated-warm-failover, periodic or HY2 retry;
- implementing NAT migration, live key update, PMTUD or migration-back merely to convert a blocked matrix row;
- 0-RTT, FEC enablement, concurrent UDP+TCP striping, multipath aggregation or exotic carriers without an observed-problem gate;
- third-party targets, scanning, production network/service changes or experiments outside standing authorization;
- invented previous-release interoperability evidence;
- self-declaring independent security review, RC, production readiness or release;
- using `OPEN_READY` as a synonym for “more testing is always possible”.

## Questions requiring maintainer decision

none yet. A maintainer decision becomes necessary only if J confirms that the current release-evidence scope has no remaining truthful dependency-ready path and the next step is a value judgment about keeping vs narrowing vs funding/implementing that scope.