# ChatGPT reviewer handoff — reject globally stale advanced duplicates

## Reviewed state

- Previous reviewer-owned handoff: exact `318460442e00fe1ac094b9276cf1c7b8c17a03cf` (`docs(handoff): keep advanced duplicates context-idempotent`).
- Previous reviewed developer implementation/test head: exact `1a562ef61ca7293575e9ae85fa6d758d32c3740e` (`fix: reject advanced overlap extensions`).
- Previous reviewed developer documentation/evidence head: exact `019021a5a6b5667e5b4a3aabc25ee22910d32e5b` (`docs: record Session overlap-state closure`).
- Current developer implementation/test head reviewed this cycle: exact `5270ea17af297ae43683dbe46201f755bbbae48a` (`fix: keep advanced duplicate context evidence idempotent`).
- Current developer documentation/evidence head remains exact `019021a5a6b5667e5b4a3aabc25ee22910d32e5b`; no new exact-tree provenance/release reconciliation has landed for `5270ea17` yet.
- Default `main` before this reviewer update was exact `5270ea17af297ae43683dbe46201f755bbbae48a`, exactly one developer commit ahead of the previous reviewer handoff.
- No VPS/WAN experiment was performed in this sequence.
- GitHub exposes no hosted combined-status records for exact `5270ea17`. This is not a blocker and must not trigger Actions polling.

## Review verdict

### ACCEPT WITH BOUNDS — `5270ea17` closes the silent advanced-segment context rewrite in the ordinary case

The developer selected the minimum fail-closed/equality shape for fully covered advanced duplicates. For `InFlight`, `Uncertain`, or `Confirmed` bytes:

- a same-context, fully contained byte-identical duplicate returns the existing state without replacing segments or mutating ledger/segment context;
- a newer/different context that does not match the advanced segment evidence rejects with `InvalidMigration`;
- the prior novel-byte advanced extension/bridge rejection remains intact;
- compact tests now cover the three advanced states, a multi-segment duplicate, old/newer-context cases, and confirmation-owned context.

This is directionally correct and removes the prior path where a duplicate could directly relabel an advanced segment to the caller-supplied newer context.

However, this commit has only developer-reported focused `cargo test -p neko-session` evidence in the commit message. There is **no persisted exact-tree `scripts/check.sh` provenance for `5270ea17`**, and the release packet/item-4 factual review remain anchored to exact `1a562ef`. Do not claim full closure from `5270ea17`; produce the replacement green SHA below first.

### HIGH / READY_LOCAL — same-segment context equality can bypass a newer ledger-global context

The final bounded adjacent review found one concrete rollback escape in exact-current `DeliveryLedger::insert`.

Current advanced-duplicate logic performs the ledger-global rollback check only inside the branch where an overlapped advanced segment's stored context differs from the requested context. If every overlapped segment context equals the requested context, the function returns success immediately.

That is insufficient because a segment can legitimately retain an older context while the **ledger-wide** context has advanced due to a disjoint segment or a confirmation elsewhere.

Concrete model:

1. segment A exists at context `(delivery_epoch=1,key_phase=0,path_generation=1)` and is moved to `InFlight`, `Uncertain`, or `Confirmed`;
2. disjoint segment B advances the ledger-wide context to `(1,1,2)` without rewriting A's stored evidence context;
3. an exact byte-identical duplicate of A arrives carrying A's old `(1,0,1)` context;
4. every overlapped segment context equals the request, so exact-current code returns success before validating that the request regresses relative to the ledger-global `(1,1,2)` context.

This violates the existing written invariant that old/regressing duplicate context remains fail closed, and the M0 candidate context rule that regressions are `OldEpoch`. It also contradicts the implementation comment that rollback is validated against the ledger.

Treat this as **HIGH / READY_LOCAL**. It is a local Session state-model defect; it does not require a wire/ACK/Carrier/crypto architecture change or maintainer policy decision.

#### Required invariant

For a fully contained byte-identical advanced duplicate:

- first validate the requested context against the ledger-wide current context **without mutating it**;
- if the request regresses relative to ledger-global context, reject with `OldEpoch` atomically even if the overlapped segment itself still stores that old context;
- if the request is globally admissible but differs from any overlapped advanced segment's stored evidence context, reject atomically with `InvalidMigration`; do not rebind advanced evidence through `insert`;
- only if the request is globally admissible and exactly matches all overlapped advanced segment contexts may the duplicate succeed idempotently;
- success must leave global context, segment context/state/topology, byte count and watermark unchanged;
- `Unsent` overlap/extension keeps the existing mutating monotonic-context behavior;
- `confirm_received` remains the explicit transition that may advance confirmation evidence context.

#### Preferred minimum implementation shape

Avoid duplicating migration logic. Refactor the existing `context_ok` behavior into a pure validator plus the current mutating commit step, for example conceptually:

- `validate_context(&self, context) -> Result<(), LedgerError>` implements the existing component-wise rollback / delivery-epoch migration rules without changing `self.context`;
- mutating insertion/confirmation paths call the pure validator and then assign the accepted ledger-global context where they already do today;
- advanced duplicate idempotence calls only the pure validator before the same-segment-context equality check.

An equivalent smaller implementation is acceptable if it demonstrably cannot drift from `context_ok`. Do not invent a generic interval framework or a new migration policy.

#### Minimum executable evidence

For each of `InFlight`, `Uncertain`, and `Confirmed`:

- create segment A at `(1,0,1)`;
- advance only ledger-global context to `(1,1,2)` through a disjoint valid segment/transition while A retains `(1,0,1)`;
- duplicate A at `(1,0,1)` -> `OldEpoch`, exact atomic snapshot unchanged;
- duplicate A at `(1,1,2)` -> `InvalidMigration`, because the request is globally current but A's advanced evidence remains owned by `(1,0,1)`;
- ordinary same-context advanced duplicate when ledger-global context is not ahead -> success/idempotent;
- preserve the `5270ea17` multi-segment duplicate, newer/different-context negatives and exact `1a562ef` novel-byte extension/bridge negatives;
- preserve a valid `confirm_received` newer-context positive.

Snapshot checks should cover global context, segment contexts/states/topology, bytes and confirmed watermark. No fuzz is required unless the repair unexpectedly changes wire decoder/parser/crypto framing.

### MEDIUM / DEFER TO FINAL GREEN REPLACEMENT SHA — Session docs/release evidence still need one factual reconciliation

`docs/spec/m0-session-state.md` still contains the stale blanket wording that same-byte duplicate **and overlap** are idempotent. Current implementation intentionally distinguishes:

- fully contained exact duplicate idempotence, subject to context admissibility/evidence ownership;
- `Unsent` same-byte overlap/extension merging;
- fail-closed advanced-state overlap when novel bytes would be introduced.

After the HIGH is repaired and the final replacement implementation/test SHA is green, reconcile this wording once. Update `docs/specs/nekomusume-session-v0.md` only if a short sentence is needed to make advanced duplicate context ownership explicit.

The release packet and `docs/reviews/release-item4-subgates-20260909.md` remain correctly anchored to exact `1a562ef` for now. Re-anchor them once, after the final replacement SHA has real exact-tree local provenance. Do not mechanically move unrelated RSEC/VPS evidence.

## Local-CI-first rule

Do **not** spend a provenance/docs commit on `5270ea17` first. Repair the HIGH in a replacement coherent developer SHA, push it, then validate that exact SHA from a clean temporary worktree/clone:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # empty
```

Persist minimum provenance only after that final replacement SHA is green: exact SHA, commands, distinct UTC start/end timestamps, exits, host/OS/arch, stable Rust version, and initial/final clean-tree state. If red, preserve and repair the concrete failure; do not unchanged-rerun until lucky.

Developer-local persisted CI, reviewer-executed checks and GitHub-hosted CI remain separate evidence classes. Hosted CI is optional cross-evidence and must never become a wait condition.

## Rolling queue — execute continuously in dependency order

The coding agent must continue through every dependency-ready item below without waiting for the next reviewer hour. The rollback HIGH is first and blocks unrelated expansion, but current repository semantics are sufficient to repair it locally.

### A. HIGH / READY_LOCAL — validate ledger-global context before advanced duplicate success

Implement the minimum pure/non-mutating ledger-global context validation described above. Preserve the current equality-based evidence ownership rule and old error distinctions where possible: global rollback -> `OldEpoch`; globally admissible but advanced-segment context mismatch -> `InvalidMigration`; exact admissible equality -> idempotent success.

Continue immediately to B.

### B. HIGH-CLOSURE / READY_LOCAL — make the disjoint-segment rollback escape executable

Add the A/B disjoint-context regression matrix for `InFlight`, `Uncertain`, and `Confirmed`, with exact atomic snapshots. Preserve existing duplicate/extension/bridge/confirmation positives and negatives.

Continue immediately to C.

### C. FINAL BOUNDED SESSION ADJACENT REVIEW / READY_LOCAL — inspect once, then stop this lane

Perform **one final** bounded source/test review of `insert -> mark_in_flight -> mark_uncertain -> confirm_received -> watermark`, limited to a demonstrable contradiction with the current written evidence/context contract. Fix only a concrete BLOCKER/HIGH found in that pass.

Do not redefine watermark contiguity, general interval semantics, migration policy, or build a generic checker. If no BLOCKER/HIGH remains, explicitly end the DeliveryLedger hardening lane and continue to D.

### D. READY_LOCAL — final replacement exact-tree gate and concise provenance

After A-C land in the final pushed implementation/test SHA, run the exact-tree local gate and persist one concise provenance note. If red, repair the concrete failure before any closure claim. Do not wait for GitHub Actions.

Continue immediately to E.

### E. BOUNDED SPEC/RELEASE RECONCILIATION — update only changed facts

After the final green replacement SHA, reconcile together where applicable:

- `docs/spec/m0-session-state.md` — narrow the stale blanket overlap-idempotence statement;
- `docs/specs/nekomusume-session-v0.md` — only if advanced duplicate context ownership needs an explicit candidate sentence;
- `docs/release-security-review-packet.md` — tested-tree anchor and Session evidence row;
- `docs/reviews/release-item4-subgates-20260909.md` — header/scope/exact-tree factual gate evidence;
- `docs/status.md` — only if a status/boundary fact actually changed.

Do not claim protocol freeze, interoperability, independent review, RC/release, public-service safety or production readiness. Do not re-anchor unrelated RSEC/VPS evidence.

Continue immediately to F.

### F. REVIEW-CLOSURE / READY_LOCAL — close the DeliveryLedger lane if clean

Verify that advanced novel-byte rejection, duplicate context handling, ledger-global rollback, confirmation-owned context, tests and candidate docs agree. If no BLOCKER/HIGH remains, explicitly mark this bounded DeliveryLedger correctness package complete and **stop extending this checker/test lane unless a new demonstrated defect appears**.

Continue immediately to G.

### G. LOCAL OUTPUT SELECTION — propose 1–3 real runtime/operator/release outputs

Re-read exact-current `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, release packet and relevant implementation. Produce 1–3 dependency-ready proposals and autonomously choose the smallest safe one. Prefer:

1. a demonstrated runtime/correctness/security defect with a concrete call site;
2. an advertised operator/runtime behavior lacking direct executable evidence;
3. a named release-evidence question answerable locally without capacity/security claim inflation.

For each proposal state the observed contradiction/missing behavior, owner file/API, protected invariant, minimum positive/negative evidence and stop condition. Do not wait for reviewer selection when one option is clearly smallest and within existing architecture/authorization.

Do not choose D019 source-retention TTL/LRU/history policy, signing/key-custody policy, SBOM publication policy, previous-release interoperability without a frozen prior release, Experimental Track carriers, or generic checker work merely to manufacture hours.

Continue immediately to H.

### H. READY_LOCAL / ROLLING VISIBLE OUTPUT — implement, exact-tree close, repeat

Implement the chosen G output, run the exact-tree local gate, reconcile only changed facts, then repeat G -> H while concrete safe work remains. If coherent slices repeatedly finish in 10–30 minutes with good test/evidence quality, enlarge the next closure package instead of stopping after each small commit. Queue exhaustion must be real, not a stale-handoff/watcher artifact.

### I. CONDITIONAL VPS OUTPUT — only if exact-current truth creates a new live question

Current repository truth remains `READY_LIVE: none`. Standing VPS authorization is still valid but is not a reason to duplicate old evidence. Only execute a VPS run if a new implementation/instrumentation change creates a named unresolved real-network question with satisfied dependencies.

Otherwise do not unchanged-rerun HY2, repeated warm failover, periodic/soak, package lifecycle, distinct A -> B -> A, already-answered endpoint migration/key update, IPv6 without a real owned IPv6 path, or live PMTUD before its separate authenticated wire/security design gate.

### J. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

The source-accounting retention/no-reset conflict remains a maintainer/security-policy question. Do not invent TTL, LRU/history capacity, external authority or a weaker reset rule. This policy lane does not block dependency-independent local correctness/release work above.

## Stop conditions

Stop and escalate only for an unresolved BLOCKER/HIGH that cannot be safely repaired from existing semantics, a required change to core Session/Carrier/ACK/crypto/wire architecture, destructive/canonical-meaning migration, action outside standing authorization, production impact, new credentials/server/third-party permission, benchmark conditions requiring maintainer value judgment, D019 policy decision, real repository breakage, runtime/tool-budget exhaustion, or genuine queue exhaustion.

Otherwise: coherent slice -> exact-tree local gate -> commit/push -> immediately continue to the next pre-authorized dependency-ready slice.
