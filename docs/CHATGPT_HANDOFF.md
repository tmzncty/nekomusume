# Nekomusume ChatGPT Handoff

Checked at: 2026-09-07 00:58 Asia/Shanghai
Repository main HEAD reviewed: `b20fa0ef4d7d8e1d23fe1c15547a79fcb952d669`
Previous reviewer handoff commit: `b20fa0ef4d7d8e1d23fe1c15547a79fcb952d669`
Previous checked implementation HEAD: `d271a99a2ab26abbcb146c411ba0fde697395abe`
Current execution branch provisioned/aligned by reviewer: `work/e1a-staged-accounting-20260907` at exact `b20fa0ef4d7d8e1d23fe1c15547a79fcb952d669`
Historical partial-E2 branch retained: `work/continue-20260904` at `d271a99a2ab26abbcb146c411ba0fde697395abe`

## What changed

No coding-agent implementation has landed since `d271a99`. Since the previous review, exact `b20fa0e` Rust CI completed `success` (run `34044390515`). The clean E1A execution branch was then fast-forwarded non-destructively from `0873109` to exact `b20fa0e`, so the active coding branch now contains the current handoff and autonomous-proposal contract rather than an older reviewer snapshot.

This branch movement is coordination only: it adds no runtime/accounting implementation, WAN evidence or performance result. E1A therefore remains a real `STALLED_IMPLEMENTATION` condition rather than a CI, environment, VPS, credential, standing-authorization or core-architecture blocker.

Current code still has the reviewed defect: `FramedReader` interprets the four-byte attacker-controlled length and allocates the payload before the responder's later `charge_input`, while the current inner/process input APIs couple every byte/work charge with one packet/record increment. The agent has explicit authority in `AGENTS.md` and below to extend that API minimally and choose the exact local typed-permit/state-machine shape without prior reviewer approval.

The historical `work/continue-20260904` / `d271a99` partial-E2 work remains retained history only. It is not an E1A prerequisite and must not be used as a reason to delay runtime repair.

## Review verdict

**STALLED_IMPLEMENTATION / EXECUTE E1A NOW.** The implementation contract, green base and active execution branch are all available. Implement the coherent staged-accounting closure, run gates, commit, push, and continue to E2 without reviewer acknowledgement. Do not spend another execution cycle only restating that the old API is insufficient.

This does not justify bypassing the HIGH correctness finding. It also does not justify growing more checker/review infrastructure before the runtime defect is repaired.

No administrator decision is required at this review point.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- A1 absolute response-I/O deadline, B1 queue ownership/expiry and D1 terminal rejection remain accepted closed subfindings unless a concrete regression appears.
- Exact `b20fa0e` has green CI but is reviewer documentation/coordination only; it adds no runtime, WAN or performance evidence.
- `work/e1a-staged-accounting-20260907` now points to exact `b20fa0e` and contains no coding change yet; branch alignment is not implementation evidence.
- `d271a99` remains partial E2 inventory/checker hardening only; it is retained history, not an E1A prerequisite.
- Existing 16 KiB per-state memory reservation is not staged input/work accounting.
- Existing `PreauthBudget::charge_input` and process `charge_input` couple packet ownership to each call; a staged single-record primitive or equivalent structural state machine is therefore required.
- No WAN/VPS run is useful as a substitute for this deterministic accounting repair.
- Historical WAN/HY2/failover/periodic positive and negative evidence remains immutable at its exact commit boundary.
- Protected identity, credentials, private endpoints and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This remains a multi-hour rolling queue. The coding agent owns ordinary local design choices and should use the proposal protocol in `AGENTS.md`: briefly compare plausible local shapes, pick the smallest fail-closed one, implement it, and let the reviewer challenge the landed design. A proposal is not a stop point.

### E1A — Staged one-record TCP accounting + four-responder migration

**Status:** `READY_LOCAL / STALLED_IMPLEMENTATION`; immediate highest priority.

**Goal / why now:** close the real HIGH pre-auth charge-order defect before more release/security evidence work. One TCP frame must be one D019 input record even though header/body accounting is staged.

**Execution base:** `work/e1a-staged-accounting-20260907` from exact green `b20fa0e`.

**Preferred local shape, not mandatory API spelling:** compose one inner and one process-level logical-record reservation through `ListenerAdmission`:

```text
begin_input_record(header_bytes, header_work)
  -> one record/packet ownership
extend_input_record(reservation, body_bytes, body_work)
  -> bytes/work only, no second packet
complete_input_record(reservation)
```

Equivalent typed-permit/state-machine designs are acceptable if these are structural invariants, not caller convention:

- exactly one record/packet charge per TCP frame;
- cumulative bytes/work across header + body;
- cumulative per-record work cannot exceed existing `max_work_per_packet`;
- inner and process/source/global accounting cannot diverge;
- any failed extension/expiry/overflow makes the logical state terminal;
- no refund of conservative attacker-caused body reservation after truncation/timeout/EOF;
- no new numeric policy, wire format or Session/Carrier/crypto semantic change.

**Required receive order:** admit source/state -> bounded raw 4-byte header read -> begin/charge header + one record before interpreting length -> decode/check length -> extend/reserve declared body bytes/work before allocation/read -> allocate/read body -> complete exactly once -> only then negotiation/Noise parse.

**Migrate in the same closure package:** ordinary TCP probe, periodic TCP responder, multistream TCP responder, failover TCP responder. No real TCP pre-auth responder may retain `read complete frame -> charge_input` semantics.

**Minimum tests:** fragmented header/body; zero body; normal frame exactly one packet; extension does not add packet; cumulative work exact/max+1; oversize before body allocation; truncated/EOF/timeout after reservation remains charged + terminal; inner/outer exhaustion cannot leave reusable ticket; overflow/backwards clock/expired state fail closed; no double extend/complete; anti-amplification sees one input packet; rejection cannot reach auth/readiness/Session/PathValidated/Delivery/ACK/authz-equivalent success evidence.

**Gate:** targeted tests + `scripts/check.sh` + `git diff --check`; run fuzz smoke because untrusted length/framing behavior is materially touched unless the environment genuinely lacks the required toolchain, in which case record `NOT_RUN_ENVIRONMENT` truthfully. Commit and push real code/tests.

**Continue immediately to E2:** yes.

### E2 — Semantic responder inventory/evidence-barrier closure

**Status:** `PREAUTHORIZED_AFTER_E1A`.

Rebuild/strengthen the existing inventory against the repaired call sites. Reapply/cherry-pick only the semantically valid parts of `d271a99`.

Required: every TCP responder anchors staged charge before negotiation/Noise parse; every UDP responder anchors bounded raw receive -> charge -> protocol parse; pending ownership proves reserve-before-store and exactly-once cancel/dequeue/expiry invalidation; rejection/timeout/malformed/I/O paths cannot reach success-evidence anchors; expected externally reachable responder set remains explicit.

Do not preserve stale string anchors or create checker-only closure around wrong runtime code.

**Gate:** semantic inventory/checker + full repository gate; commit/push.

**Continue immediately to C1:** yes.

### C1 — Explicit bounded carrier/source projection

**Status:** `PREAUTHORIZED_AFTER_E2`.

Add an explicit bounded carrier discriminator so current TCP and UDP pre-auth source domains cannot accidentally alias. Preserve family/address/port representation without textual/raw logging; add deterministic cross-carrier/family/address/port non-collision tests. Do not invent source-retention TTL/LRU/history limits.

**Continue immediately to C2:** yes.

### C2 — Terminal-source persistence policy checkpoint

**Status:** `ADR_CHECKPOINT_AFTER_C1`.

Re-read D019 against exact implementation. If literal no-reset-on-retry/reconnect/carrier-change semantics cannot coexist with bounded source-accounting memory without a new retention policy, write a compact ADR amendment request. Do not invent convenience numbers.

Only this policy-dependent lane should wait if a maintainer/reviewer decision is genuinely required; continue H -> I -> J independently.

### F — Full D019 adversarial/evidence-barrier matrix

**Status:** `PREAUTHORIZED_AFTER_C2_RESOLVED`.

Close source/global concurrency, staged bytes/packets/work, one-second windows, cumulative per-record work, memory, queue, response + inner 3x anti-amplification, idle/lifetime/100 ms deadline, arithmetic/backwards-clock failure, terminal non-revival, resolved retry/reconnect/carrier-transition semantics, cancellation/double cleanup and no-success-evidence-on-rejection.

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

Re-evaluate every remaining release/evidence row: already answered bounded question -> `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`; executable missing assertion -> `OPEN_READY` with exact action/dependencies/scope; otherwise exact implementation/environment/governance/review blocker.

Then end the audit-only loop. If no new HIGH/BLOCKER remains, select one concrete highest-value `BLOCKED_IMPLEMENTATION` runtime seam that can progress to local validation and then a truthful changed-hypothesis VPS question. Candidate families remain NAT/source endpoint change, migration-back, live key update or live PMTUD only when existing architecture supports a bounded path.

### K — One runtime-seam closure package

**Status:** `PREAUTHORIZED_AFTER_J_WHEN_ONE_CANDIDATE_IS_SELECTED`.

Treat implementation + local verification + evidence instrumentation for the selected seam as one coherent package. The agent may propose/select the smallest design inside existing architecture without reviewer pre-approval. Do not open several speculative seams at once.

**Continue immediately to L if local gate and exact-head CI make a truthful live row READY:** yes.

### L — One changed-hypothesis VPS evidence row + reconciliation

**Status:** `PREAUTHORIZED_AFTER_K_AND_EXACT_HEAD_GREEN`; only if a declared missing question is genuinely `READY_LIVE`.

Under standing authorization, run the minimum bounded self-owned client/VPS scenario needed to answer that one question; retain positive or negative evidence with exact code/binary/parameters/resource/cleanup provenance; reconcile release matrix/status. Do not repeat an unchanged failed scenario or promote one run to production/performance evidence.

## 24–48 hour output check

Recent repository motion is still dominated by security/reviewer infrastructure. E1A remains a genuine correctness blocker, but the design contract and a clean, current execution branch now exist. The next meaningful coding checkpoint must contain runtime/accounting implementation and tests, not another explanation of the blocker.

After the D019 lane closes (or C2 alone becomes a genuine policy wait), the queue explicitly returns to one runtime seam and one bounded real-network evidence question so the project produces a new executable path, real evidence conclusion, package/operator capability or genuine milestone/security gate closure rather than continuing audit-infrastructure self-reproduction.

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

If E1A still has no coding checkpoint after the clean execution branch and explicit local design authority above, classify the condition as external executor inactivity/coordination failure rather than inventing more repository design work. C2 may later become a genuine policy decision; if so, isolate only that lane and continue independent H/I/J work.