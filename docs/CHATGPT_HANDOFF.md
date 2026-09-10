# ChatGPT reviewer handoff — keep advanced Session duplicates context-idempotent

## Reviewed state

- Previous reviewer-owned handoff: exact `4ea103ba98931aa7f579a362d21a4a0d1a2331d8` (`docs(handoff): prevent overlap delivery-state promotion`).
- Previous reviewed developer implementation/test head: exact `6a458ffa9e133cece2b2dbd4e382948d19df698c` (`fix: commit confirmed delivery context globally`).
- Previous reviewed developer documentation/evidence head: exact `bedc79982226dad566dcd3d9a2461f00f65bf8dd` (`docs: record Session confirmation context closure`).
- Current developer implementation/test head reviewed this cycle: exact `1a562ef61ca7293575e9ae85fa6d758d32c3740e` (`fix: reject advanced overlap extensions`).
- Current developer documentation/evidence head reviewed this cycle: exact `019021a5a6b5667e5b4a3aabc25ee22910d32e5b` (`docs: record Session overlap-state closure`).
- Default `main` before this reviewer update was exact `019021a5a6b5667e5b4a3aabc25ee22910d32e5b`, exactly two commits ahead of the previous reviewer handoff.
- Developer sequence since the previous reviewer handoff:
  - `1a562ef61ca7293575e9ae85fa6d758d32c3740e` chooses the minimum fail-closed overlap shape: byte-identical overlap that would introduce novel bytes into an `InFlight`, `Uncertain`, or `Confirmed` range rejects with `InvalidMigration`; contained duplicates and `Unsent` merges remain supported. Focused tests cover left/right extensions and an advanced same-state bridge.
  - `019021a5a6b5667e5b4a3aabc25ee22910d32e5b` records developer-local exact-tree provenance, updates the provisional Session overlap invariant, and re-anchors the release packet/item-4 factual support to exact `1a562ef`.
- No new VPS/WAN experiment was performed in this sequence.
- GitHub exposes no hosted combined-status records for exact `1a562ef`. This is not a blocker and must not trigger Actions polling.

## Review verdict

### ACCEPT WITH BOUNDS — the previous novel-byte delivery-state promotion HIGH is closed

Exact `1a562ef` materially fixes the prior contradiction. `DeliveryLedger::insert` now computes whether the requested interval is already covered by existing same-state bytes. When the overlapping state is `InFlight`, `Uncertain`, or `Confirmed`, any insertion that would add a novel byte rejects before `context_ok`, segment replacement, byte accounting, or watermark mutation. `Unsent` overlap/extension keeps the existing merge behavior.

The focused tests cover right and left extension for all three advanced states and a same-state advanced bridge; rejection preserves segment count, bytes, global context and confirmed watermark. The selected repair is deliberately representationally conservative: it rejects advanced-state extension rather than splitting state intervals, and it does not change wire, ACK format, Carrier, crypto, capacity, or migration policy.

`docs/local-session-overlap-state-1a562ef-20260910.md` records developer-local exact-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit `0`, `git diff --check` exit `0`, UTC `2026-09-09T23:11:35Z -> 23:13:31Z`, Linux/x86_64, Rust 1.98.0, and clean initial/final source tree. Reviewer did not execute this CI. Hosted CI is absent and not required.

This closes only the **novel bytes inherit advanced state** defect. It is bounded candidate Session-state evidence, not a protocol freeze, independent review, RC/release, interoperability proof, public-listener approval, or production evidence.

### HIGH / READY_LOCAL — a fully contained advanced duplicate can still rewrite delivery context without the evidence-producing transition

The bounded adjacent review found one concrete Session evidence mutation that remains in exact-current `DeliveryLedger::insert`.

The new advanced-extension guard only rejects when `existing_coverage < end`. A fully contained byte-identical duplicate therefore proceeds through the ordinary merge path. That path then calls `self.context_ok(context)?`, removes the existing segment(s), and inserts the merged segment with **the caller-supplied `context`** while preserving the old advanced `state`.

Consequently, a segment that is already `Confirmed` at context `(delivery_epoch=1,key_phase=0,path_generation=1)` can receive an exact duplicate insertion at `(1,1,2)`: the insertion can advance the ledger-wide context and replace the still-`Confirmed` segment with the newer context even though `confirm_received` was never called for that newer context. The same shape can rebind `InFlight` or `Uncertain` bytes without an explicit delivery-state transition.

That conflicts with current repository truth:

- provisional Session v0 says fully contained exact duplicates are idempotent;
- `docs/spec/m0-session-state.md` likewise describes same-byte duplicates as idempotent;
- the current confirmation-context invariant specifically makes `confirm_received` validate and atomically commit a newer confirmation context to the ledger and segment.

`insert` must not become a second implicit evidence-producing context transition merely because bytes are duplicates. Treat this as **HIGH / READY_LOCAL** because Session delivery/context state is the cross-Carrier failover/replay source of truth. It is a local state-model repair; it does not require a new wire field, ACK domain, crypto primitive, Carrier architecture, capacity number, or maintainer policy decision.

#### Required invariant

- Same-context, fully contained byte-identical advanced duplicates remain supported and must be state/context/bytes/watermark idempotent.
- An old/regressing context on a duplicate must remain fail closed; do not turn replay/rollback evidence into unconditional success merely to make duplicates idempotent.
- A duplicate carrying a newer admissible context must **not** silently advance the context attached to `InFlight`, `Uncertain`, or `Confirmed` bytes.
- In particular, `Confirmed` segment context may advance only through the existing explicit confirmation semantics; duplicate insertion alone cannot manufacture confirmation under a newer key/path/delivery context.
- If the implementation chooses rejection for an advanced duplicate whose context differs, the rejection must leave global context, segment context/state, bytes, segment topology, and watermark unchanged.
- `Unsent` overlap/extension behavior may continue to rebind/merge under the existing monotonic context rule because those bytes do not yet carry stronger delivery evidence.
- No rejected duplicate may demote or split existing advanced evidence merely to simplify the fix.

#### Short proposal cycle before implementation

Inspect exact-current callers/tests and compare 1–3 minimal shapes, then choose the smallest fail-closed one. Reasonable candidates are:

1. for a fully covered advanced duplicate, run a **pure** context admissibility check and return the existing state without replacing segments or mutating ledger/segment context;
2. require context equality for advanced duplicates and reject a differing context atomically, while retaining same-context idempotence and the existing old-context fail-closed rule;
3. only if exact-current callers demonstrate a real need to rebind already-advanced bytes, propose a separate explicit typed transition API. That would be a larger semantic choice and should be DEFERRED unless repository truth proves it necessary.

Prefer 1 or 2. Do not build a generic interval framework or silently weaken the meaning of `Confirmed`.

#### Minimum focused evidence

- same-context exact duplicate of `Confirmed` remains successful/idempotent with unchanged segment and ledger context;
- exact duplicate of `Confirmed` carrying newer key/path context cannot mutate confirmation context through `insert` alone;
- equivalent `InFlight` and `Uncertain` cases cannot silently rewrite their segment context;
- a regressing/old context duplicate remains fail closed and atomic;
- if a duplicate spans multiple same-state advanced segments, it must not collapse/relabel their contexts/evidence as an incidental side effect; rejecting that shape is acceptable for the current minimal representation;
- prior left/right advanced-extension and bridge tests remain green;
- `Unsent` overlap/extension, mixed-state rejection, conflict/bounds checks, global context monotonicity, byte count and watermark behavior remain intact;
- `confirm_received` with a valid newer context still advances ledger + segment context atomically.

No fuzz is required unless the repair unexpectedly touches wire decode/parser/crypto framing.

### MEDIUM / DEFER TO FINAL GREEN SESSION SHA — candidate documentation overstates generic overlap idempotence

`docs/spec/m0-session-state.md` still says, without qualification, that same-byte duplicate **and overlap** are idempotent. Exact `1a562ef` intentionally rejects advanced-state overlaps that introduce novel bytes, while provisional Session v0 now states the narrower truthful rule. Reconcile the M0 candidate text after the final green Session SHA so it distinguishes:

- fully contained exact duplicate idempotence;
- `Unsent` same-byte overlap/extension merging;
- fail-closed advanced-state extension with novel bytes.

Do not make a standalone docs-churn commit for this while the context-idempotence HIGH is open.

The current release packet and item-4 factual review are correctly anchored to exact `1a562ef`; after the HIGH is fixed, re-anchor them once to the final tested implementation/test SHA and state only facts that the final tests prove. The pre-auth/resource review remains independently anchored to its own RSEC tested tree and should not be mechanically moved for unrelated Session changes.

## Local-CI-first rule

For implementation/test slices below, push the coherent developer SHA first, then validate that exact SHA in a clean temporary worktree/clone:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # empty
```

Persist minimum provenance only after the final replacement SHA is green: exact SHA, commands, distinct UTC start/end timestamps, exits, host/OS/arch, stable Rust version, and initial/final clean-tree state. If red, preserve and repair the concrete failure; do not unchanged-rerun until lucky. Developer-local persisted CI, reviewer-executed CI and hosted CI remain separate evidence classes. Hosted CI is optional cross-evidence and must never become a wait condition.

No fuzz is required for the current pure Session-state slice unless implementation unexpectedly changes wire decode/parser/crypto framing.

## Rolling queue — execute continuously in dependency order

The coding agent must continue through every dependency-ready item below without waiting for the next reviewer hour. The context-idempotence HIGH is first and blocks unrelated expansion, but the current repository semantics are sufficient to resolve it without maintainer approval.

### A. HIGH / READY_LOCAL — choose and repair advanced-duplicate context ownership

Run the short proposal cycle above against exact-current `DeliveryLedger` callers/tests. Implement the minimum shape that preserves old-context rejection while preventing `insert` from silently rebinding advanced delivery evidence to a newer context.

Do not change the meaning of Session confirmation, ACK domains, Carrier semantics, crypto, wire format, or delivery evidence.

Continue immediately to B.

### B. HIGH-CLOSURE / READY_LOCAL — make context-idempotence executable

Add the focused same-context/newer-context/older-context duplicate matrix for `InFlight`, `Uncertain`, and `Confirmed`, plus any compact multi-segment duplicate case required by the selected implementation. Preserve the exact-`1a562ef` novel-byte extension/bridge negatives and the confirmation-context positive.

Continue immediately to C.

### C. BOUNDED SESSION ADJACENT REVIEW / READY_LOCAL — inspect this evidence lane once, then stop

Perform one bounded source/test review of `insert -> mark_in_flight -> mark_uncertain -> confirm_received -> watermark`, limited to a **demonstrable contradiction with the written current evidence/context contract**. Fix only a concrete defect. Do not redefine watermark contiguity, interval semantics, or migration policy merely because an alternative model is imaginable.

If no BLOCKER/HIGH remains after this pass, explicitly end the DeliveryLedger checker/hardening lane and continue to D.

### D. READY_LOCAL — final exact-tree gate and concise provenance

After A-C land in the final pushed implementation/test SHA, run the exact-tree local gate. Persist one concise provenance note with exact SHA, commands, UTC interval, host/OS/arch, stable Rust, clean-tree state, and the focused Session duplicate/overlap/context positives and negatives. If red, repair the concrete failure before closure. Do not wait for GitHub Actions.

Continue immediately to E.

### E. BOUNDED SPEC/RELEASE RECONCILIATION — update only changed facts

After the final green Session SHA, reconcile together where applicable:

- `docs/specs/nekomusume-session-v0.md` only if the selected duplicate-context semantics need a precise candidate sentence;
- `docs/spec/m0-session-state.md` to remove the stale blanket overlap-idempotence wording;
- `docs/release-security-review-packet.md` tested-tree anchor and Session evidence row;
- `docs/reviews/release-item4-subgates-20260909.md` header/scope/exact-tree gate facts;
- `docs/status.md` only if a status/boundary fact actually changes.

Do not re-anchor unrelated RSEC/VPS evidence. Do not claim protocol freeze, interoperability, independent review, RC/release, public-service safety or production readiness.

Continue immediately to F.

### F. REVIEW-CLOSURE / READY_LOCAL — close the bounded Session ledger lane if clean

Check that advanced extension rejection, duplicate context handling, ledger-wide confirmation context, tests and candidate documentation agree. If no BLOCKER/HIGH remains, explicitly mark this bounded Session ledger correctness package complete and stop extending this checker/test lane unless a new demonstrated defect appears.

Continue immediately to G.

### G. LOCAL OUTPUT SELECTION — propose 1–3 real runtime/operator/release outputs

Re-read exact-current `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, release packet and relevant implementation. Produce 1–3 dependency-ready proposals and autonomously choose the smallest safe one. Prefer, in order:

1. a demonstrated runtime/correctness/security defect with a concrete call site;
2. an advertised operator/runtime behavior lacking direct executable evidence;
3. a named release-evidence question answerable locally without capacity/security claim inflation.

For each proposal state the observed contradiction/missing behavior, owner file/API, protected invariant, minimum positive/negative evidence and stop condition. Do not wait for reviewer selection when one option is clearly smallest and within existing architecture/authorization.

Do not choose D019 source-retention TTL/LRU/history policy, signing/key-custody policy, SBOM publication policy, previous-release interoperability without a frozen prior release, Experimental Track carriers, or generic checker work merely to manufacture hours.

Continue immediately to H.

### H. READY_LOCAL / ROLLING VISIBLE OUTPUT — implement, exact-tree close, repeat

Implement the chosen G output, run the exact-tree local gate, reconcile only changed facts, then repeat G -> H while concrete safe work remains. If high-quality coherent slices repeatedly finish in 10–30 minutes, enlarge the next coherent closure package rather than stopping after each small commit. Queue exhaustion must be real, not a stale-handoff/watcher artifact.

### I. CONDITIONAL VPS OUTPUT — only if exact-current truth creates a new live question

Current repository truth still says `READY_LIVE: none`. Standing VPS authorization remains valid but is not a reason to duplicate old evidence. Only execute a VPS run if a new implementation/instrumentation change creates a named unresolved real-network question with satisfied dependencies. Otherwise do not unchanged-rerun HY2, repeated warm failover, periodic/soak, package lifecycle, distinct A -> B -> A, already-answered endpoint migration/key update, IPv6 without a real owned IPv6 path, or live PMTUD before its separate authenticated wire/security design gate.

### J. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

The current source-accounting retention/no-reset conflict remains a maintainer/security-policy question. Do not invent TTL, LRU/history capacity, external authority or a weaker reset rule. This policy lane does not block dependency-independent local correctness/release work above.

## Stop conditions

Stop and escalate only for an unresolved BLOCKER/HIGH that cannot be safely repaired from existing semantics, a required change to core Session/Carrier/ACK/crypto/wire architecture, destructive/canonical-meaning migration, action outside standing authorization, production impact, new credentials/server/third-party permission, benchmark conditions requiring maintainer value judgment, D019 policy decision, real repository breakage, runtime/tool-budget exhaustion, or genuine queue exhaustion.

Otherwise: coherent slice -> exact-tree local gate -> commit/push -> immediately continue to the next pre-authorized dependency-ready slice.
