# Nekomusume ChatGPT Handoff

Checked at: 2026-09-06 17:57 Asia/Shanghai
Repository main HEAD reviewed: `5a8d358b3172944d4ac32f2b9251b910397a0832`
Work-branch HEAD additionally reviewed: `d271a99a2ab26abbcb146c411ba0fde697395abe`
Previous checked reviewer handoff: `bc0f55068fe994a284cc37258537af71fb90e9ac`
Previous checked implementation HEAD: `d271a99a2ab26abbcb146c411ba0fde697395abe`

## What changed

No new coding-agent implementation has landed since `d271a99`. Two reviewer/coordination commits landed on `main`:

- `bc0f550` — turns E1A from an abstract ordering requirement into an explicit staged one-logical-record accounting contract;
- `5a8d358` — updates `AGENTS.md` so coding agents have proposal authority, must break implementation stagnation, and should optimize for deliverable closures rather than endless checker/review infrastructure.

Exact `5a8d358` Rust CI run `34021330112` completed `success`.

The refs remain diverged from merge base `fdbcae7`:

- `main` is ahead with reviewer/agent-contract work;
- `work/continue-20260904` still retains the useful coding commit `d271a99` but has not integrated current `main`.

This is now a real `STALLED_IMPLEMENTATION` coordination condition. E1A has remained the immediate HIGH/open slice across multiple reviewer/execution cycles, there is no CI/environment/authorization blocker, and the missing accounting API can be implemented inside the existing D019/Session/Carrier architecture. The coding agent must not keep reporting “existing API cannot express staged reservation” as a terminal blocker: adding the minimal staged accounting primitive is the task.

## Review verdict

**STALLED_IMPLEMENTATION / CONTINUE_WITH_REQUIRED_FIXES — integrate the branch history, choose and implement a minimal staged single-record accounting design, migrate all four TCP pre-auth responders, then continue the existing closure queue without waiting for reviewer approval.**

No administrator decision is required for E1A. No new security numeric policy is required. No wire/Noise/Session/Carrier semantic change is required. No VPS work is useful until this deterministic correctness lane closes.

## Proposal authority for the coding agent

The coding agent is explicitly authorized to propose and choose the implementation shape for E1A without prior reviewer approval, provided it preserves the invariants below and does not change core protocol architecture or invent new policy numbers.

A short proposal is enough. Prefer one of these shapes or an equivalent safer shape:

### Preferred: typed one-record permit

At the inner budget layer:

```text
PreauthBudget::begin_input_record(header_bytes)
    -> InnerInputRecordPermit

PreauthBudget::extend_input_record(&mut permit, additional_bytes)
    -> ()

PreauthBudget::complete_input_record(permit)
    -> ()
```

At the process/source/global layer:

```text
ProcessPreauthAdmission::begin_input_record(id, header_bytes, header_work, now)
    -> ProcessInputRecordPermit

ProcessPreauthAdmission::extend_input_record(&mut permit, bytes, work, now)
    -> ()

ProcessPreauthAdmission::complete_input_record(permit, now)
    -> ()
```

`ListenerAdmission` composes both permits into one `TcpInputReservation` so callers cannot advance one layer without the other.

The permit owns **one packet/record charge exactly once**. Extensions add bytes/work only. Per-record work is cumulative across every stage.

### Acceptable alternative

A staged record object may be held inside `AdmissionTicket`/process state rather than returned as a separate crypto-layer permit, but it must still make double packet ownership, double completion, extension-after-terminal and fresh per-stage work ceilings mechanically impossible.

Avoid untyped booleans or caller-maintained arithmetic if a typed ownership object can encode the lifecycle.

The coding agent should choose the smallest design with the least new mutable state and easiest deterministic fail-closed tests, implement it, and let the next reviewer challenge the result. Do not wait for a separate design approval.

## RSEC-001E1A — HIGH/open implementation contract

Invariant:

```text
one TCP frame
= one D019 input packet/record ownership
+ cumulative staged bytes/work
```

Required receive order for every real TCP pre-auth responder:

1. source/state already admitted;
2. bounded raw read of exactly the fixed 4-byte length header;
3. begin one logical input record, charging 4 header bytes + one packet/record + conservative header work **before** interpreting the length;
4. decode/check the attacker-controlled u32 length;
5. extend/reserve the same logical record for declared body bytes + conservative body/protected work **before** body allocation/read;
6. allocate/read body only after reservation succeeds;
7. truncated body, EOF, timeout, oversize, arithmetic failure, expired state or I/O failure after reservation fails closed, terminalizes the logical pre-auth state, and does not refund attacker-caused conservative charge;
8. complete body may proceed to negotiation/Noise parsing only under the already-reserved accounting;
9. consume/complete the one-record reservation exactly once.

Protected invariants:

- header/body stages never count as two D019 packets;
- `max_work_per_packet` applies cumulatively to the whole record, not independently per stage;
- existing per-state/source/global byte, packet and work ceilings are not weakened;
- response anti-amplification continues to see one charged input packet for one TCP frame;
- no new numeric ceilings or frame limits are introduced;
- no wire-format change;
- inner accounting failure terminalizes outer state as D1 requires;
- outer accounting failure cannot resurrect/refund a reusable ticket merely for symmetry.

Required migrations:

- ordinary TCP probe;
- periodic TCP responder;
- multistream TCP responder;
- failover TCP responder.

No current TCP pre-auth responder may retain the old `read complete frame -> charge_input` order.

Minimum deterministic tests:

- fragmented 4-byte header;
- zero-length body;
- normal header + body => exactly one packet/record;
- body extension does not increment packet count;
- header + body work shares one cumulative per-record ceiling;
- exact work ceiling succeeds, max+1 fails terminally;
- oversize declared length rejects before body allocation after header ownership charge;
- truncated/EOF/timeout after body reservation retains conservative charge and terminalizes state;
- inner and outer exhaustion cannot leave a reusable ticket;
- overflow/backwards clock/expired state fails closed;
- permit cannot extend/complete twice;
- anti-amplification still sees one input packet;
- no auth/readiness/Session/PathValidated/Delivery/ACK/authz-equivalent success evidence on rejection.

Run targeted tests, `scripts/check.sh`, and `git diff --check`; run fuzz smoke if production untrusted-input parser/wire behavior materially changes. Commit and push a coherent E1A closure package.

**Continue immediately to E2: yes.**

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- A1 absolute response-I/O deadline, B1 queue ownership/expiry and D1 terminal rejection remain accepted closed subfindings unless a concrete regression appears.
- `d271a99` is useful partial E2 inventory hardening only; it is not E1A closure and is not on current `main` history yet.
- Existing 16 KiB per-state reservation is useful memory bounding, not staged input/work accounting.
- Existing `PreauthBudget::charge_input` / process `charge_input` semantics couple one packet to each call and therefore cannot simply be called twice for header/body.
- No WAN/VPS evidence is added by these coordination commits.
- Historical WAN/HY2/failover/periodic evidence remains immutable at its exact commit boundary.
- Protected identity, credentials, private endpoints and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a multi-hour rolling queue. The coding agent may propose/refine local API shapes itself. Complete one coherent slice -> tests/gates -> commit -> push -> immediately continue to the next dependency-ready slice. Reviewer acknowledgement is not required between slices.

### Q0 — Reconcile current branch divergence

**Status:** `READY_LOCAL`; do first if still divergent.

Integrate current `origin/main` and retained `d271a99` into the coding branch/history without force-push or loss of reviewer-owned files. Ordinary merge/cherry-pick-safe reconciliation is acceptable. Do not discard `d271a99`; do not edit `docs/CHATGPT_HANDOFF.md` from the coding-agent side.

**Continue immediately to E1A:** yes.

### E1A — Staged one-record TCP accounting + four-responder migration

**Status:** `READY_LOCAL / STALLED_IMPLEMENTATION`; highest priority.

Use the contract above. The agent may choose the minimal typed API design itself and implement immediately.

**Continue immediately to E2:** yes.

### E2 — Semantic responder inventory/evidence-barrier closure

**Status:** `PREAUTHORIZED_AFTER_E1A`; `d271a99` is retained partial progress.

Update the responder inventory/checker to assert semantic ordering on the repaired implementation:

- every TCP responder anchors staged charge before negotiation/Noise parse;
- every UDP responder anchors bounded raw receive -> charge -> protocol parse;
- pending owner proves reserve-before-store and exactly-once cancel/dequeue/expiry invalidation;
- rejection/timeout/malformed/I/O paths cannot reach success-evidence anchors;
- expected externally reachable responder set remains explicit.

Fix real uncovered code rather than documenting around it. Full gate, commit, push.

**Continue immediately to C1:** yes.

### C1 — Explicit bounded carrier/source projection

**Status:** `PREAUTHORIZED_AFTER_E2`.

Add an explicit carrier discriminator so current TCP and UDP pre-auth source domains cannot alias accidentally. Preserve family/address/port representation without textual/raw logging. Add deterministic non-collision tests. Do not invent terminal-source retention TTL/LRU/history limits.

**Continue immediately to C2:** yes.

### C2 — Terminal-source persistence policy checkpoint

**Status:** `ADR_CHECKPOINT_AFTER_C1`.

Re-read D019 and adjacent decisions against the exact repaired implementation. If literal no-reset-on-retry/reconnect/carrier-change semantics cannot coexist with bounded source-accounting memory without a new retention policy, write a compact ADR amendment request and external-wait **only this lane**. Do not invent convenience numbers.

**If C2 waits:** continue H -> I -> J independently.

### F — Full D019 adversarial/evidence-barrier matrix

**Status:** `PREAUTHORIZED_AFTER_C2_RESOLVED`.

Close the deterministic matrix: source/global concurrency; staged input bytes/packets/work; global windows; per-record cumulative work; state/global memory; queue; response + inner 3x anti-amplification; idle/lifetime/100 ms deadline; arithmetic/clock failures; terminal non-revival; resolved retry/reconnect/carrier transition; cancellation/double cleanup; and no success evidence on rejection.

Full gate, commit, push; exact-head CI green before G.

### G — Exact-tree D019/security evidence closure

**Status:** `PREAUTHORIZED_AFTER_F`.

Independently re-read exact implementation/tests and reconcile resource-abuse review, release-security packet, `docs/status.md`, and closure navigation. RSEC-001 may close as an implementation finding only if E1A/E2/C1/C2/F are truly satisfied. Independent external/two-person review remains a separate release gate.

**Continue immediately to H if no new HIGH/BLOCKER:** yes.

### H — Compatibility/freeze-boundary review

**Status:** `READY_LOCAL_AFTER_G`; also independent fallback during C2 wait.

Audit corpus-v1 freeze vs global protocol non-freeze, current/current negotiation, unsupported/future rejection, downgrade/transcript/resume/replay boundaries, and stale wording. Add regressions only for concrete defects.

### I — Package/operator/evidence-provenance review

**Status:** `READY_LOCAL_AFTER_H`; fallback during C2 wait.

Verify existing package lifecycle, build identity, cleanup, evidence manifests and exact-head references without reading protected identity material. Do not rerun already-sufficient VPS/package work merely for freshness.

### J — Reclassify release opportunities and return to milestone-output work

**Status:** `READY_LOCAL_AFTER_I`.

Re-evaluate each release/evidence row:

- bounded question already answered -> `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`;
- specific dependency-ready missing assertion -> `OPEN_READY` with exact action/scope;
- implementation/environment/governance dependency missing -> exact blocker.

Then deliberately end the audit-only loop. If D019 has no new HIGH/BLOCKER, choose the highest-value dependency-ready milestone/output lane. Prefer a concrete missing runtime seam that can progress to local verification and then one changed-hypothesis VPS evidence row under standing authorization. Candidate families include NAT/source endpoint change, migration-back, live key update or live PMTUD only when their prerequisites can be implemented without speculative architecture expansion.

Do not manufacture live traffic if no honest `READY_LIVE` row exists.

## 24-48 hour output check

Recent work has been dominated by security accounting, inventory and review infrastructure. E1A remains a real security blocker, so it must close; however this lane must terminate in an exact-tree D019 closure rather than spawning indefinite new checker/docs slices.

After E1A/E2/C1/C2/F/G closure (or after C2 is isolated as external-wait and H/I are done), the queue must return to a deliverable runtime/evidence lane. A successful next phase should produce at least one of:

- a new executable runtime path;
- a newly answered bounded real-network question;
- a concrete package/operator capability;
- closure of a genuine release/milestone gate.

Pure checker/docs churn is not sufficient by itself unless it directly closes the current security gate.

## Completion gates

D019/RSEC-001 implementation closure requires:

- one TCP frame remains one D019 packet/record under staged accounting;
- header charged before attacker-controlled length interpretation;
- body bytes/work reserved before allocation/body read;
- per-record work cumulative across stages;
- incomplete reserved bodies remain charged and terminal;
- all real TCP responders use the shared staged helper;
- semantic responder inventory/evidence barriers are machine checked;
- carrier/source projection explicit and bounded;
- terminal-source persistence policy reviewed rather than invented;
- complete deterministic adversarial matrix and exact-head CI green;
- security/release prose does not outrun exact implementation;
- governance flags unchanged.

## Do not expand into

- protocol/wire/Noise/Session/Carrier redesign for E1A;
- new numeric D019/security/source-retention limits without reviewed ADR work;
- VPS/load tests as a substitute for deterministic accounting;
- renewed HY2 work without a changed missing-question hypothesis;
- speculative FEC/0-RTT/striping/multipath/exotic carriers;
- public/production listener deployment;
- reading/hashing/copying/modifying/committing protected identity/secrets/private endpoint material;
- RC/freeze/release/production promotion.

## Questions requiring maintainer decision

None at this review point.

E1A is implementation-ready and explicitly permits coding-agent proposal/choice inside the existing architecture. C2 may later become a genuine policy decision; if it does, isolate that lane and continue independent work rather than blocking the whole repository.
