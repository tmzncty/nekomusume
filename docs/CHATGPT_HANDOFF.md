# Nekomusume ChatGPT Handoff

Checked at: 2026-09-07 05:02 Asia/Shanghai
Repository main HEAD reviewed: `a2d5c4d10bc881ff0e9a3bec34e97f9950e1a635`
Previous reviewer handoff commit: `a2d5c4d10bc881ff0e9a3bec34e97f9950e1a635`
Previous checked implementation HEAD: `d271a99a2ab26abbcb146c411ba0fde697395abe`
Current execution branch: `work/e1a-staged-accounting-20260907` at exact `a2d5c4d10bc881ff0e9a3bec34e97f9950e1a635`
Historical partial-E2 branch retained: `work/continue-20260904` at `d271a99a2ab26abbcb146c411ba0fde697395abe`

## What changed

No coding-agent implementation has landed since `d271a99`. The active E1A branch has remained exactly aligned with reviewer-only `main` through multiple execution opportunities after all known coordination excuses were removed:

- the staged one-logical-record accounting contract is explicit;
- autonomous proposal/implementation authority is durable in `AGENTS.md`;
- the historical partial-E2 branch is explicitly not an E1A prerequisite;
- a clean execution branch exists at current `main`;
- exact `a2d5c4d` Rust CI is green on both `main` (run `34047295194`) and `work/e1a-staged-accounting-20260907` (run `34053519427`).

Current code still has the reviewed HIGH defect. `FramedReader` reads the four-byte length, interprets attacker-controlled length and allocates the body before the responder's later input/work charge, while the existing inner/process `charge_input` APIs couple every byte/work charge with one packet/record increment. No runtime/accounting change has landed on either active branch.

This condition is now classified as **external executor inactivity / coordination failure**, not `STALLED_IMPLEMENTATION` caused by an underspecified repository contract. There is no current CI, repository, VPS, credential, standing-authorization, environment or core-architecture blocker for E1A.

The reviewer must not respond to continued inactivity by inventing more checker layers, provisioning more equivalent branches, or rewriting the same design contract every hour. The next meaningful repository event must be a coding checkpoint or a genuinely new technical blocker discovered by code/test work.

## Review verdict

**BLOCKED_EXTERNAL_EXECUTOR_INACTIVITY — repository work is ready; execute E1A.**

The external coding agent should use `work/e1a-staged-accounting-20260907`, briefly compare acceptable local API shapes under the proposal protocol, choose the smallest fail-closed design, implement/test/commit/push, and continue immediately through dependency-ready slices. If its local checkout cannot use that named branch for a tooling reason, it may create/use another ordinary work branch from exact current `main`; branch naming is not a design gate. It must not wait for reviewer approval of ordinary local API shape.

No administrator design decision is required. No VPS work is useful until the deterministic HIGH defect is repaired.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- A1 absolute response-I/O deadline, B1 queue ownership/expiry and D1 terminal rejection remain accepted closed subfindings unless a concrete regression appears.
- Exact `a2d5c4d` has green CI on `main` and the active E1A work branch, but it is reviewer documentation/coordination only; it adds no runtime, WAN or performance evidence.
- `d271a99` remains partial E2 inventory/checker hardening only; retain its useful semantics for later E2, but it is not an E1A prerequisite.
- Existing 16 KiB per-state memory reservation is not staged input/work accounting.
- Existing inner/process input APIs couple packet ownership to each charge call; a staged single-record primitive or equivalent structural state machine is required.
- Historical WAN/HY2/failover/periodic positive and negative evidence remains immutable at its exact commit boundary.
- Standing VPS authorization remains valid for future dependency-ready self-owned work; deterministic D019 accounting correctness must not be replaced by WAN/load testing.
- Protected identity, credentials, private endpoints and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This queue remains pre-authorized multi-hour work. The coding agent owns ordinary local design choices. Finish one coherent closure package -> targeted/full gates -> commit -> push -> immediately consume the next dependency-satisfied package. One commit, one nominal hour, one reviewer interval, or one proposal note is never a stop condition.

### E1A — Staged one-record TCP accounting + four-responder migration

**Status:** `READY_LOCAL`; immediate HIGH correctness repair. Executor inactivity is external to the repository, not a technical dependency.

**Goal / why now:** one TCP frame must be exactly one D019 input record even though its header/body bytes and work are charged in stages. Charge must precede attacker-controlled interpretation/allocation.

**Execution base:** `work/e1a-staged-accounting-20260907` from exact green `a2d5c4d` (or an ordinary new work branch from exact current `main` if local tooling cannot use that branch name).

**Design authority:** the agent should compare 1–3 minimal local shapes and implement one without waiting. Preferred family is a typed logical-record reservation composed through `ListenerAdmission`, for example:

```text
begin_input_record(header_bytes, header_work)
  -> exactly one record/packet ownership
extend_input_record(reservation, body_bytes, body_work)
  -> bytes/work only; no second packet
complete_input_record(reservation)
```

Equivalent typed-permit/state-machine APIs are acceptable if structural invariants hold:

- exactly one packet/record charge per TCP frame;
- cumulative bytes/work across header + body;
- cumulative per-record work cannot exceed the existing `max_work_per_packet`;
- inner and process/source/global accounting cannot diverge;
- failed extension/expiry/overflow terminalizes the logical state;
- attacker-caused body reservation is not refunded after truncation/timeout/EOF;
- no new numeric policy, wire format or Session/Carrier/ACK/crypto semantic change.

**Required receive order:** admit -> bounded raw 4-byte header read -> begin/charge header + one record before interpreting length -> decode/check length -> extend/reserve declared body bytes/work before allocation/read -> allocate/read body -> complete exactly once -> only then negotiation/Noise parse.

**Migrate in this same closure package:** ordinary TCP probe, periodic TCP responder, multistream TCP responder, failover TCP responder. Do not leave one path with complete-frame-read -> later-charge semantics.

**Minimum tests:** fragmented header/body; zero body; one frame == one packet; extension adds no packet; cumulative work exact/max+1; oversize before body allocation; truncated/EOF/timeout after reservation remains charged and terminal; inner/outer exhaustion does not leave a reusable ticket; overflow/backwards clock/expired state fail closed; double extend/complete rejected; anti-amplification observes one input packet; rejection cannot reach auth/readiness/Session/PathValidated/Delivery/ACK/authz-equivalent success evidence.

**Gate:** targeted tests + `scripts/check.sh` + `git diff --check`; run fuzz smoke because untrusted framing/length behavior changes, or truthfully record `NOT_RUN_ENVIRONMENT` if the toolchain is unavailable. Commit and push real implementation/tests.

**Continue immediately to E2:** yes.

### E2 — Semantic responder inventory/evidence-barrier closure

**Status:** `PREAUTHORIZED_AFTER_E1A`.

Rebuild/strengthen the responder inventory against the repaired runtime. Reapply/cherry-pick only semantically valid parts of historical `d271a99`.

Required: every TCP responder anchors the staged charged-frame primitive before negotiation/Noise parse; every UDP responder anchors bounded raw receive -> charge -> protocol parse; pending ownership proves queue-reserve-before-store and exactly-once cancel/dequeue/expiry invalidation; rejection/timeout/malformed/I/O paths cannot reach success-evidence anchors; expected externally reachable responder set remains explicit.

Do not preserve stale string anchors or make checker-only closure around wrong runtime code.

**Gate:** semantic inventory/checker + full repository gate; commit/push.

**Continue immediately to C1:** yes.

### C1 — Explicit bounded carrier/source projection

**Status:** `PREAUTHORIZED_AFTER_E2`.

Add an explicit bounded carrier discriminator so current TCP and UDP pre-auth source domains cannot accidentally alias. Preserve family/address/port representation without textual/raw logging; add deterministic cross-carrier/family/address/port non-collision tests. Do not invent source-retention TTL/LRU/history limits.

**Continue immediately to C2:** yes.

### C2 — Terminal-source persistence policy checkpoint

**Status:** `ADR_CHECKPOINT_AFTER_C1`.

Re-read D019 against exact implementation. If literal no-reset-on-retry/reconnect/carrier-change semantics cannot coexist with bounded source-accounting memory without a new retention policy, write a compact ADR amendment request with exact conflicting clauses, attacker/resource rationale, feasible policy shapes without convenience numbers, and required tests/evidence.

Do not invent numeric policy. Stop only this policy-dependent lane if a real maintainer/reviewer choice is needed; continue H -> I -> J independently.

### F — Full D019 adversarial/evidence-barrier matrix

**Status:** `PREAUTHORIZED_AFTER_C2_RESOLVED`.

Close source/global concurrency; staged bytes/packets/work; one-second windows; cumulative per-record work; state/global memory; queue; source/global response + inner 3x anti-amplification; idle/lifetime/100 ms response deadline; arithmetic/backwards-clock failure; terminal non-revival; resolved retry/reconnect/carrier-transition semantics; cancellation/double cleanup; and no success evidence on rejection.

Do not substitute VPS/load tests for deterministic accounting correctness.

**Gate:** full local gate; push; exact-head CI green before G.

### G — Fresh exact-tree D019/security evidence closure

**Status:** `PREAUTHORIZED_AFTER_F`.

Independently re-read exact implementation/tests and reconcile the resource-abuse review, release-security packet, `docs/status.md` and closure navigation. RSEC-001 may close as an implementation finding only if E1A/E2/C1/C2/F are actually satisfied. Independent external/two-person security review remains separate. Never promote RC/production/freeze/release automatically.

**Continue immediately to H if no new HIGH/BLOCKER:** yes.

### H — Compatibility/freeze-boundary review

**Status:** `READY_LOCAL_AFTER_G`; safe fallback during a genuine C2 policy wait.

Audit corpus-v1 freeze vs global protocol non-freeze, current/current negotiation, unsupported/future rejection, downgrade/transcript/resume/replay boundaries and stale wording. Add regression only for a concrete defect.

**Continue immediately to I:** yes.

### I — Package/operator/evidence-provenance review

**Status:** `READY_LOCAL_AFTER_H`; safe fallback during C2 wait.

Verify existing package lifecycle/build identity, install-readiness-upgrade-rollback evidence, cleanup, canonical evidence manifests and exact-head references without reading protected identity material. Do not rerun already-sufficient VPS/package work merely for freshness.

**Continue immediately to J:** yes.

### J — Reclassify release opportunities and deliberately return to milestone-output work

**Status:** `READY_LOCAL_AFTER_I`.

Re-evaluate every remaining release/evidence row: answered bounded question -> `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`; executable missing assertion -> `OPEN_READY` with exact action/dependencies/scope; otherwise record the exact implementation/environment/governance/review blocker.

Then end the audit-only loop. If no new HIGH/BLOCKER remains, select one concrete highest-value `BLOCKED_IMPLEMENTATION` runtime seam that can progress to local validation and then a truthful changed-hypothesis VPS question. Candidate families remain NAT/source endpoint change, migration-back, live key update or live PMTUD only when existing architecture supports a bounded path.

**Continue immediately to K:** yes once one candidate is selected without a maintainer-level design decision.

### K — One runtime-seam closure package

**Status:** `PREAUTHORIZED_AFTER_J_WHEN_ONE_CANDIDATE_IS_SELECTED`.

Treat implementation + local verification + evidence instrumentation for the selected seam as one coherent package. The agent may propose/select the smallest design inside existing architecture without reviewer pre-approval. Do not open several speculative seams at once.

**Continue immediately to L if local gate and exact-head CI make a truthful live row READY:** yes.

### L — One changed-hypothesis VPS evidence row + reconciliation

**Status:** `PREAUTHORIZED_AFTER_K_AND_EXACT_HEAD_GREEN`; only if a declared missing question is genuinely `READY_LIVE`.

Under standing authorization, run the minimum bounded self-owned client/VPS scenario needed to answer that one question; retain positive or negative evidence with exact code/binary/parameters/resource/cleanup provenance; reconcile release matrix/status. Do not repeat an unchanged failed scenario or promote one run to production/performance evidence.

## 24–48 hour output check

Repository motion remains dominated by reviewer/security infrastructure rather than executable progress. E1A is still a genuine correctness blocker, but the repository contract, proposal authority, clean green execution branch and tests/gates required to start are already sufficient. Further reviewer-only commits do not increase project capability.

The next meaningful checkpoint must contain runtime/accounting implementation and tests. Once D019 closes (or only C2 becomes a genuine policy wait), the queue deliberately returns to a concrete runtime seam and one bounded real-network evidence question so the project produces executable capability, a real evidence conclusion, package/operator capability or genuine milestone/security closure.

## Completion gates

D019/RSEC-001 implementation closure requires all of:

- one TCP frame remains one D019 packet/record under staged accounting;
- header charge precedes attacker-controlled length interpretation;
- body bytes/work reserved before allocation/body read;
- cumulative per-record work enforced across stages;
- incomplete reserved body remains charged and terminal;
- all real TCP responders use the shared staged contract;
- semantic responder inventory/evidence barriers machine checked;
- carrier/source projection explicit and bounded;
- terminal-source persistence resolved by reviewed policy rather than invented limits;
- complete deterministic adversarial matrix and exact-head CI green;
- security/release prose does not outrun exact implementation;
- governance flags unchanged.

## Do not expand into

- more reviewer-only branch provisioning or repeated rewording of E1A while the external executor is inactive;
- protocol/wire/Noise/Session/Carrier redesign for E1A;
- new numeric D019/security/source-retention limits without reviewed ADR work;
- public or production listener deployment;
- VPS/load tests as a substitute for deterministic security accounting;
- renewed HY2 work without a changed missing-question hypothesis;
- speculative FEC/0-RTT/striping/multipath/exotic carriers;
- reading, hashing, copying, modifying or committing protected identity/secrets/private endpoint material;
- RC/freeze/release/production promotion.

## Questions requiring maintainer decision

None at this review point.

The current blocker is external executor inactivity, not a repository design decision. Do not ask the maintainer to choose an E1A API. If later C2 genuinely requires a new retention policy value or core semantic choice, isolate that policy lane and surface the exact decision while continuing independent work.