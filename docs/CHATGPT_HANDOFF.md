# Nekomusume ChatGPT Handoff

Checked at: 2026-09-06 21:02 Asia/Shanghai
Repository main HEAD reviewed: `16fb6003aab5f9c6a1894353ae10c49fe04f91d3`
Work-branch HEAD additionally reviewed: `d271a99a2ab26abbcb146c411ba0fde697395abe`
Previous reviewer handoff commit: `16fb6003aab5f9c6a1894353ae10c49fe04f91d3`
Previous checked implementation HEAD: `d271a99a2ab26abbcb146c411ba0fde697395abe`

## What changed

No new coding-agent implementation has landed since `d271a99`. The previous reviewer handoff already marked E1A `STALLED_IMPLEMENTATION` and gave the coding agent explicit proposal authority plus a concrete staged one-logical-record accounting contract. Exact `16fb600` Rust CI run `34026138137` completed `success`.

The branch topology itself is now the only remaining coordination friction worth removing:

- `main` is current at `16fb600` and contains the latest reviewer contract plus the autonomous-proposal rules in `AGENTS.md`;
- `work/continue-20260904` is still at `d271a99`, diverged from merge base `fdbcae7`;
- the unique `d271a99` coding change touches only `docs/preauth-responder-inventory.v1.json` and `scripts/check-preauth-responder-inventory.py` and is partial E2 checker/inventory work; it does **not** contain E1A runtime/accounting implementation.

Therefore preserving `d271a99` does not require making branch reconciliation a prerequisite for E1A. Continuing to treat that old work branch as the active implementation base would create needless coordination cost.

The repository still has a real HIGH/open deterministic defect: all four TCP pre-auth responder paths need staged input/work accounting before attacker-controlled length interpretation/allocation. There is no CI, environment, VPS, authorization, credential or core-architecture blocker. The minimal accounting extension is within the existing D019 design boundary.

## Review verdict

**STALLED_IMPLEMENTATION / EXECUTE NOW — start E1A from current `origin/main` (or a fresh coding branch based on current `origin/main`), implement the staged single-record accounting closure as one coherent engineering package, then carry the useful `d271a99` E2 inventory change forward only when E2 begins. Do not wait for another reviewer interval or branch-merge ceremony.**

No administrator decision is required. No new numeric security policy is required. No wire/Noise/Session/Carrier semantic change is required. No VPS run is useful until this deterministic correctness lane closes.

## Branch recovery rule

The coding agent is explicitly authorized to stop using the stale `work/continue-20260904` tip as its active base.

Preferred recovery:

1. `fetch` current remote state;
2. create/reset a **new coding branch from current `origin/main`** using a non-destructive new branch name, or work from a clean current-main checkout if that is the established local workflow;
3. implement E1A there;
4. when E2 starts, cherry-pick or manually reapply only the still-useful semantic parts of `d271a99` after checking them against the repaired E1A code;
5. never force-push or delete the old `d271a99` branch merely for cleanliness.

`d271a99` is retained evidence/history, not an execution dependency. If its checker assumptions conflict with the repaired E1A design, update/reapply them during E2 rather than bending E1A to preserve stale checker strings.

Do **not** create a proposal-only commit as a substitute for implementation. A short proposal may live in commit notes or implementation comments, but the first new coding checkpoint should contain real staged-accounting code and tests.

## Proposal authority for E1A

The coding agent may choose the exact local API shape without prior reviewer approval, provided all protected invariants below hold.

Preferred shape:

```text
inner PreauthBudget
  begin_input_record(header_bytes)
      -> InnerInputRecordPermit
  extend_input_record(&mut permit, additional_bytes)
  complete_input_record(permit)

process/source/global admission
  begin_input_record(id, header_bytes, header_work, now)
      -> ProcessInputRecordPermit
  extend_input_record(&mut permit, bytes, work, now)
  complete_input_record(permit, now)

ListenerAdmission
  composes both into one TcpInputReservation
```

Equivalent state-machine designs are acceptable if they make these properties structural rather than caller convention:

- packet/record ownership is charged exactly once;
- staged extensions add bytes/work without another packet charge;
- per-record work is cumulative across stages;
- extension/completion after terminal rejection is impossible;
- inner/outer accounting cannot advance independently into contradictory states;
- no new policy values are introduced.

Choose the smallest design with the least mutable state and strongest fail-closed tests, implement it, and let the next reviewer challenge the landed design.

## RSEC-001E1A — HIGH/open implementation contract

Invariant:

```text
one TCP frame
= one D019 input packet/record ownership
+ cumulative staged bytes/work
```

Required receive order for every real TCP pre-auth responder:

1. source/state admitted before pre-auth framing work;
2. bounded raw read of the fixed four-byte length header;
3. begin one logical input record, charging four header bytes + one packet/record + conservative header work **before** interpreting the attacker-controlled length;
4. decode/check the declared length;
5. extend/reserve the same logical record for declared body bytes + conservative protected work **before** body allocation/read;
6. allocate/read body only after reservation succeeds;
7. truncated body, EOF, timeout, oversize, arithmetic failure, expired state or I/O failure after reservation fails closed, terminalizes the logical pre-auth state and does not refund attacker-caused conservative charge;
8. complete body may proceed to negotiation/Noise parsing only under the already-reserved accounting;
9. consume/complete the one-record reservation exactly once.

Protected invariants:

- header/body stages never count as two D019 packets;
- `max_work_per_packet` applies cumulatively to the whole record, not independently per stage;
- existing inner per-state anti-amplification/input bound remains at least as strict;
- existing source/global byte, packet and work ceilings are not weakened;
- response anti-amplification still sees one charged input packet for one TCP frame;
- no new numeric ceilings or frame limits;
- no wire-format change;
- inner rejection terminalizes outer logical state as the accepted D1 contract requires;
- conservative reservation after attacker-declared length is not refunded merely because the body is truncated.

Required migrations in the same closure package:

- ordinary TCP probe;
- periodic TCP responder;
- multistream TCP responder;
- failover TCP responder.

No real TCP pre-auth responder may retain `read complete frame -> charge_input` semantics after E1A.

Minimum deterministic tests:

- fragmented 4-byte header;
- zero-length body;
- normal header + body => exactly one packet/record;
- body extension does not increment packet count;
- header + body work shares one cumulative per-record ceiling;
- exact work ceiling succeeds; max+1 fails terminally;
- oversize declared length rejects before body allocation after header ownership charge;
- truncated/EOF/timeout after body reservation retains conservative charge and terminalizes state;
- inner and outer exhaustion cannot leave a reusable ticket;
- overflow/backwards clock/expired state fails closed;
- permit cannot extend/complete twice;
- anti-amplification still observes one input packet;
- rejection cannot reach auth/readiness/Session/PathValidated/Delivery/ACK/authz-equivalent success evidence.

Run targeted tests, `scripts/check.sh`, `git diff --check`, and fuzz smoke if production untrusted-input parser/wire behavior materially changes. Commit and push a coherent E1A closure package, then continue immediately to E2.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- A1 absolute response-I/O deadline, B1 queue ownership/expiry and D1 terminal rejection remain accepted closed subfindings unless a concrete regression appears.
- Exact `16fb600` has green CI but is reviewer documentation only; it adds no runtime, WAN or performance evidence.
- `d271a99` is useful partial E2 inventory hardening only and is intentionally no longer a prerequisite for E1A.
- Existing 16 KiB per-state reservation bounds state-owned memory but is not staged input/work accounting.
- Existing `PreauthBudget::charge_input` and process `charge_input` couple packet ownership to each call, which is why a staged single-record primitive is necessary.
- No WAN/VPS evidence is added by this coordination work.
- Historical WAN/HY2/failover/periodic positive and negative evidence remains immutable at its exact commit boundary.
- Protected identity, credentials, private endpoints and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a multi-hour rolling queue. The coding agent owns ordinary local design choices and may propose/refine API shapes itself. Complete one coherent slice -> tests/gates -> commit -> push -> immediately continue to the next dependency-ready slice. Reviewer acknowledgement is not required between slices.

### E1A — Staged one-record TCP accounting + four-responder migration

**Status:** `READY_LOCAL / STALLED_IMPLEMENTATION`; immediate highest priority.

Start from current `origin/main` or a fresh branch from it. Implement the complete contract above; do not spend another execution cycle only restating the API gap.

**Commit/push gate:** real accounting primitive + all four TCP migrations + deterministic tests + required repository gates.

**Continue immediately to E2:** yes.

### E2 — Semantic responder inventory/evidence-barrier closure

**Status:** `PREAUTHORIZED_AFTER_E1A`; `d271a99` may be cherry-picked/reapplied here if still semantically correct.

Rebuild/strengthen the inventory against the repaired implementation:

- every TCP responder anchors staged charge before negotiation/Noise parse;
- every UDP responder anchors bounded raw receive -> charge -> protocol parse;
- pending owner proves reserve-before-store and exactly-once cancel/dequeue/expiry invalidation;
- rejection/timeout/malformed/I/O paths cannot reach success-evidence anchors;
- expected externally reachable responder set remains explicit.

Do not preserve stale string anchors merely because they existed in `d271a99`. Fix uncovered runtime seams rather than documenting around them.

**Commit/push gate:** semantic inventory/checker passes against exact repaired call sites + full repository gate.

**Continue immediately to C1:** yes.

### C1 — Explicit bounded carrier/source projection

**Status:** `PREAUTHORIZED_AFTER_E2`.

Add an explicit bounded carrier discriminator so TCP and UDP pre-auth source domains cannot alias accidentally. Preserve family/address/port representation without textual/raw logging. Add deterministic non-collision tests. Do not invent terminal-source retention TTL/LRU/history limits.

**Continue immediately to C2:** yes.

### C2 — Terminal-source persistence policy checkpoint

**Status:** `ADR_CHECKPOINT_AFTER_C1`.

Re-read D019 and adjacent decisions against the repaired implementation. If literal no-reset-on-retry/reconnect/carrier-change semantics cannot coexist with bounded source-accounting memory without a new retention policy, write a compact ADR amendment request and external-wait **only this policy lane**. Do not invent convenience numbers.

**If C2 waits:** continue H -> I -> J independently.

### F — Full D019 adversarial/evidence-barrier matrix

**Status:** `PREAUTHORIZED_AFTER_C2_RESOLVED`.

Close source/global concurrency, staged bytes/packets/work, one-second windows, per-record cumulative work, memory, queue, response + 3x anti-amplification, idle/lifetime/100 ms deadline, arithmetic/clock failure, terminal non-revival, resolved reconnect/carrier-transition semantics, cancellation/double cleanup and no-success-evidence-on-rejection.

**Commit/push gate:** full local gate; exact-head CI green before G.

### G — Exact-tree D019/security evidence closure

**Status:** `PREAUTHORIZED_AFTER_F`.

Independently re-read exact implementation/tests and reconcile resource-abuse review, release-security packet, `docs/status.md` and closure navigation. RSEC-001 may close as an implementation finding only if E1A/E2/C1/C2/F are truly satisfied. Independent external/two-person security review remains a separate release gate.

**Continue immediately to H if no new HIGH/BLOCKER:** yes.

### H — Compatibility/freeze-boundary review

**Status:** `READY_LOCAL_AFTER_G`; also independent fallback during C2 wait.

Audit corpus-v1 freeze vs global protocol non-freeze, current/current negotiation, unsupported/future rejection, downgrade/transcript/resume/replay boundaries and stale wording. Add regression only for a concrete defect.

**Continue immediately to I:** yes.

### I — Package/operator/evidence-provenance review

**Status:** `READY_LOCAL_AFTER_H`; fallback during C2 wait.

Verify existing package lifecycle, build identity, cleanup, evidence manifests and exact-head references without reading protected identity material. Do not rerun already-sufficient VPS/package work merely for freshness.

**Continue immediately to J:** yes.

### J — Reclassify release opportunities and return to milestone-output work

**Status:** `READY_LOCAL_AFTER_I`.

Re-evaluate each release/evidence row:

- bounded question already answered -> `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`;
- specific dependency-ready missing assertion -> `OPEN_READY` with exact action/scope;
- implementation/environment/governance dependency missing -> exact blocker.

Then deliberately terminate the audit-only loop. If D019 has no new HIGH/BLOCKER, select a concrete missing runtime seam that can progress to local validation and then, where truthful, one changed-hypothesis VPS evidence row under standing authorization. Candidate families remain NAT/source endpoint change, migration-back, live key update or live PMTUD only when their prerequisites are genuinely implementable without speculative architecture expansion.

Do not manufacture live traffic if no honest `READY_LIVE` row exists.

### K — Runtime-seam closure package

**Status:** `PREAUTHORIZED_AFTER_J_WHEN_ONE_CANDIDATE_IS_SELECTED`.

For the single highest-value `BLOCKED_IMPLEMENTATION` row selected in J, treat implementation + local verification + evidence instrumentation as one closure package. The agent may propose the smallest runtime design inside existing architecture and proceed without reviewer pre-approval. Do not open several speculative runtime seams at once.

**Continue immediately to L if local gate makes a truthful live row READY:** yes.

### L — One changed-hypothesis VPS evidence row + reconciliation

**Status:** `PREAUTHORIZED_AFTER_K_AND_EXACT_HEAD_GREEN`; execute only when a specific declared missing question becomes `READY_LIVE`.

Under standing authorization, run the minimum bounded self-owned client/VPS scenario needed to answer that question, retain positive or negative evidence with provenance/resource/cleanup boundaries, then reconcile the release matrix/status. Do not repeat an unchanged failed scenario and do not turn one run into a production/performance claim.

## 24-48 hour output check

Recent work remains dominated by security accounting, inventory and review infrastructure. E1A is still a genuine HIGH correctness blocker, so it must close; however the reviewer has now removed branch reconciliation as an excuse/dependency and supplied a concrete execution contract.

The next coding progress should be a real implementation closure, not another planning/checker-only checkpoint. After D019 closure (or C2 isolation plus independent safe fallback work), the queue explicitly returns to one runtime seam and one bounded real-evidence question.

A successful next phase should produce at least one of:

- a new executable runtime path;
- a newly answered bounded real-network question;
- a concrete package/operator capability;
- closure of a genuine security/release/milestone gate.

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

E1A is implementation-ready and no longer depends on the stale work branch. C2 may later become a genuine policy decision; if it does, isolate that lane and continue H/I/J rather than blocking the repository.