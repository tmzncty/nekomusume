# ChatGPT reviewer handoff — prevent overlap inserts from manufacturing Session delivery state

## Reviewed state

- Previous reviewer-owned handoff: exact `afe94947e093680c9a5a454bee0e994eaf0695bf` (`docs(handoff): retire expired retained UDP session`).
- Previous reviewed developer implementation/test head: exact `5be8c6e38e2f70a8d6766eb7337e156640a5b565` (`test: isolate selection retry from post-Noise budget`).
- Previous reviewed developer documentation/evidence head: exact `a9aed3b9afcc0545518affad92e95dfb0281c9ef` (`docs: close retained UDP input boundary`).
- Current default-branch developer implementation/test head reviewed this cycle: exact `6a458ffa9e133cece2b2dbd4e382948d19df698c` (`fix: commit confirmed delivery context globally`).
- Current default-branch developer documentation/evidence head reviewed this cycle: exact `bedc79982226dad566dcd3d9a2461f00f65bf8dd` (`docs: record Session confirmation context closure`).
- Developer sequence since the previous reviewer handoff:
  - `9695621049acc22f3556c04f1d92f08b7548e514` repairs the retained failover UDP expiry transition so admission, cached response, secure transport state and the pre-progress ResumeGuard are retired together; it also adds a bounded first-Data delay test seam.
  - `f5a74db59e3fd9fb547b0a19a5d871cdf1bd4cbd` adds the stronger expiry-before-first-Data process regression: stale delayed Data gets no DeliveryAck/failover success and a fresh admitted negotiation in the same bounded server process succeeds.
  - `a3d65a0e021d2688f34f0e45bc38d1819d7c9375` records the initial exact-tree expiry provenance and release-facing evidence update.
  - `fe4c85a2213cd57f7ca6d1d3a6cd1b7a48bfc4ee` removes a redundant equivalent process regression, leaving the stronger coverage as the single retained test shape.
  - `f4706690a07675e806e4f1ff3483374fb385a7e5` re-runs/re-anchors the replacement exact-`fe4c85a` full local gate and release/item-4 evidence after that test deduplication.
  - `6a458ffa9e133cece2b2dbd4e382948d19df698c` repairs `DeliveryLedger::confirm_received` so a newer confirmed Session context is validated and committed ledger-wide before the segment is marked confirmed, preventing a later insertion from rolling the global key/path context backward.
  - `bedc79982226dad566dcd3d9a2461f00f65bf8dd` records exact-`6a458ff` developer-local provenance and updates the provisional Session context invariant/evidence index.
- No new VPS/WAN experiment was performed in this sequence.
- GitHub currently exposes no hosted combined-status records for exact `6a458ff`. This is not a blocker and must not trigger Actions polling.

## Review verdict

### ACCEPT WITH BOUNDS — retained UDP expiry HIGH is closed and the bounded RSEC checker/harness lane ends here

The previous expiry HIGH is materially repaired in the current failover-server path. When `preauth.expire()` consumes the live retained UDP admission before first application Data, current code clears `handshake_admission`, `handshake_cache`, `secure`, and the pre-progress `guard` before the receive/classification path. The stale authenticated transport capability therefore cannot survive after its accounting owner has expired.

The retained process regression completes negotiation/Noise, delays first Data beyond the existing pre-auth idle boundary, proves that stale Data obtains no DeliveryAck/success, and then proves a fresh admitted negotiation can succeed in the same bounded server process. The ordinary first-Noise response-loss/retry and retained-owner charging positives remain present. Exact `fe4c85a` is the replacement developer-tested tree after redundant-test removal; `docs/local-retained-udp-expiry-9695621-20260910.md` records the final replacement gate as green with clean initial/final source tree.

This closes the specific `owner expired -> secure/ResumeGuard still live` defect. One bounded adjacent inspection of the same failover UDP pre-progress terminal path found no equivalent current transition. **Stop extending this RSEC checker/test lane unless a new demonstrated defect appears.** RSEC-001 remains open for adversarial-load/promotion suitability and independent review, and D019 source-retention/no-reset remains separately policy-blocked; neither is closed by this engineering repair.

### ACCEPT WITH BOUNDS — ledger-wide confirmation context now advances with the confirmed segment

Exact `6a458ff` fixes a real Session-state rollback gap. Before the change, `confirm_received` could advance only one segment's `SessionContext`, leaving the ledger-wide context stale and allowing a later `insert` to pass against an older key/path context. Current code validates the delivery transition, validates segment-local monotonicity, then applies the existing ledger-wide component-wise `context_ok` rule before mutating the segment. The same context is then committed to the segment and it becomes `Confirmed`.

The focused regression advances confirmation from `(delivery_epoch=1,key_phase=0,path_generation=1)` to `(1,1,2)`, checks both global and segment context, and proves a later `(1,0,1)` insertion fails with `OldEpoch` without changing segment count, bytes or watermark.

`docs/local-session-confirmation-context-6a458ff-20260910.md` records developer-local exact-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit `0`, `git diff --check` exit `0`, UTC `2026-09-09T22:26:40Z -> 22:28:36Z`, Linux/x86_64, Rust 1.98.0 and clean initial/final source tree. Reviewer did not execute this CI. Hosted CI is absent and not required.

This is bounded candidate Session-state evidence only. It is not a protocol freeze, independent review, release/security approval, public-listener approval, RC or production evidence.

### HIGH / READY_LOCAL — overlap extension can manufacture `InFlight` / `Uncertain` / `Confirmed` state for bytes never put in that state

The next concrete Session correctness/evidence defect is in `DeliveryLedger::insert`'s overlap merge.

Current overlap handling:

1. collects all overlapping segments;
2. requires their states to match;
3. constructs one merged byte range containing both old bytes and any newly introduced bytes;
4. inserts that whole merged range with `state =` the existing overlap state.

That is safe only when the insertion is fully contained/idempotent or when the inherited state is `Unsent`. If an existing segment is already `InFlight`, `Uncertain`, or `Confirmed` and a byte-identical overlapping insertion extends beyond the previously stored range, the **novel bytes inherit the old advanced state without the corresponding transition/evidence**.

The current test `overlap_preserves_delivered_bytes` makes the contradiction executable: it confirms existing bytes `ab`, inserts overlapping extension `bc`, obtains merged bytes `abc`, and asserts the whole result remains `Confirmed`. The novel byte `c` was never covered by `confirm_received`, yet it becomes confirmed. The provisional Session spec now explicitly says `confirm_received` means the peer accepted logical bytes for transport delivery. `insert` therefore must not synthesize that evidence. The same principle applies to `InFlight` and `Uncertain`: new bytes cannot be classified as already sent/uncertain merely because they overlap an advanced-state range.

Treat this as a correctness/evidence **HIGH / READY_LOCAL** because Session delivery state is the cross-Carrier failover/replay source of truth. This is a local state-model repair; it does not require a new Session architecture, wire field, ACK format, crypto primitive, Carrier policy, capacity number or maintainer decision.

#### Required invariant

- a fully contained byte-identical duplicate may remain idempotent and preserve the existing state;
- bytes already present in an advanced state must not be demoted merely to simplify representation;
- **any byte newly introduced to the ledger must begin `Unsent` unless some separate existing API has actually produced the stronger evidence**;
- therefore an overlap/contiguous merge must never promote novel bytes into `InFlight`, `Uncertain`, or `Confirmed`;
- a rejected overlap/extension must leave bytes, segments, ledger context, segment context and confirmed watermark unchanged;
- conflict/gap/bounds/context checks remain fail closed.

#### Proposal cycle before implementation

Inspect exact-current callers/tests and compare 1–3 small shapes, then choose the smallest state/API change that preserves the invariant. Reasonable candidates include:

1. preserve advanced-state existing ranges and insert only uncovered novel subranges as `Unsent` (splitting at state boundaries as needed);
2. reject an advanced-state overlap that would introduce any novel bytes, while keeping fully contained exact duplicates idempotent;
3. another equally small representation using existing segment/state machinery, if it does not manufacture or erase evidence.

Do not build a generic interval framework unless the current representation genuinely requires it. Do not silently reinterpret `Confirmed` or weaken the Session evidence contract to make the current merge convenient.

#### Minimum focused evidence

- existing `Confirmed [0,2)` plus byte-identical extension overlapping into a novel byte must not make that novel byte Confirmed; confirmed watermark must not advance because of insertion alone;
- equivalent `InFlight` and `Uncertain` extension cases must not promote novel bytes into those states;
- fully contained exact duplicate remains idempotent and preserves existing state;
- left-extension and right-extension coverage for the selected implementation shape; if the implementation supports bridging between same-state advanced ranges, the newly filled middle bytes must not inherit the advanced state;
- mixed-state overlap remains fail closed;
- conflicting overlap, uncovered-gap behavior, limits, context monotonicity and rejection atomicity remain intact;
- if the selected shape rejects advanced-state extensions, explicitly test unchanged segment count/bytes/context/watermark after rejection.

No fuzz is required unless the repair unexpectedly touches wire decode/parser/crypto framing.

### MEDIUM / DEFER TO FINAL GREEN SESSION SHA — release-facing tested-tree anchor is stale

The release packet's top-level developer-tested anchor and the item-4 factual review header/scope still point to exact `fe4c85a`, even though `bedc799` now indexes the exact-`6a458ff` Session confirmation-context evidence and a green exact-`6a458ff` full local gate is persisted. Do **not** make a standalone docs-only churn commit now. After the overlap-state HIGH is repaired and its final implementation/test SHA is green, reconcile the packet and item-4 factual support together to that final tested tree and state only the newly proven Session facts.

`docs/reviews/resource-abuse-evidence-2026-09-04.md` is specifically the pre-auth/RSEC engineering review and may truthfully remain anchored to `fe4c85a`; do not mechanically re-anchor it to unrelated Session-state work unless its own claims change.

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

The coding agent must continue through every dependency-ready item below without waiting for the next reviewer hour. The Session overlap HIGH is first and blocks expansion into unrelated runtime work, but it does not require maintainer approval because the existing delivery-evidence semantics already determine the invariant.

### A. HIGH / READY_LOCAL — choose and implement truthful overlap-state ownership

Run the short proposal cycle above against exact-current `DeliveryLedger` callers and tests, choose the minimum fail-closed shape, then repair `insert` so novel bytes never inherit an advanced delivery state merely because of overlap/merge.

Do not change the meaning of `Confirmed`, `Uncertain`, `InFlight`, Session delivery evidence, ACK domains, Carrier semantics or wire format.

Continue immediately to B.

### B. HIGH-CLOSURE / READY_LOCAL — make advanced-state overlap semantics executable

Add the focused Confirmed/InFlight/Uncertain extension, exact-duplicate, left/right extension, mixed-state/conflict and atomic-rejection coverage described above. Update the existing `overlap_preserves_delivered_bytes` test so it no longer treats newly introduced bytes as implicitly confirmed.

If the selected implementation permits bridge filling between same-state advanced ranges, add one compact test that the new middle bytes are not auto-promoted. If that behavior is intentionally rejected instead, prove rejection is atomic.

Continue immediately to C.

### C. BOUNDED SESSION ADJACENT REVIEW / READY_LOCAL — inspect the state/evidence transitions once, then stop

Perform one bounded source/test review of `insert -> mark_in_flight -> mark_uncertain -> confirm_received -> watermark` for an equivalent **concrete** evidence-manufacturing or evidence-demotion transition. Fix only a demonstrated defect. Do not create a generic interval/state checker lane merely because more exhaustive testing is imaginable.

If no BLOCKER/HIGH remains after this bounded inspection, explicitly end the DeliveryLedger checker/hardening lane and continue to D.

### D. READY_LOCAL — final exact-tree gate and concise provenance

After A-C land in the final pushed implementation/test SHA, run the exact-tree local gate. Persist one concise provenance note with exact SHA, commands, UTC interval, host/OS/arch, stable Rust, clean-tree state and the focused Session overlap/context positives. If red, repair the concrete failure before closure. Do not wait for GitHub Actions.

Continue immediately to E.

### E. BOUNDED RELEASE/SPEC RECONCILIATION — update only changed facts

After the final green Session SHA, reconcile together where applicable:

- `docs/specs/nekomusume-session-v0.md` only if the selected overlap semantics need a precise candidate invariant;
- `docs/release-security-review-packet.md` top tested-tree anchor and Session-delivery evidence row;
- `docs/reviews/release-item4-subgates-20260909.md` header/scope/exact-tree gate facts;
- `docs/status.md` only if a status/boundary fact actually changed.

Do not re-anchor unrelated RSEC/VPS evidence just to make every document mention the same SHA. Do not claim protocol freeze, interoperability, independent review, RC/release or production readiness.

Continue immediately to F.

### F. REVIEW-CLOSURE / READY_LOCAL — close the Session ledger inspection lane if clean

Perform one final bounded check that the new overlap behavior, confirmation-context repair and documentation agree. If no BLOCKER/HIGH remains, explicitly mark this bounded Session ledger correctness package complete and stop extending this checker/test lane unless a new demonstrated defect appears.

Continue immediately to G.

### G. LOCAL OUTPUT SELECTION — choose the next real runtime/operator/release output

Re-read exact-current `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, release packet and relevant code. Produce 1–3 dependency-ready proposals and autonomously choose the smallest safe one. Prefer, in order:

1. a demonstrated runtime/correctness/security defect with a concrete call site;
2. an advertised operator/runtime behavior lacking direct executable evidence;
3. a named release-evidence question answerable locally without capacity/security claim inflation.

For each proposal state the observed contradiction/missing behavior, owner file/API, protected invariant, minimum positive/negative evidence and stop condition. Do not wait for reviewer selection when one option is clearly smallest and within existing architecture/authorization.

Do not choose D019 source-retention TTL/LRU/history policy, signing/key-custody policy, SBOM publication policy, previous-release interoperability without a frozen prior release, Experimental Track carriers, or generic checker work merely to manufacture hours.

Continue immediately to H.

### H. READY_LOCAL / ROLLING VISIBLE OUTPUT — implement, exact-tree close, repeat

Implement the chosen G output, run the exact-tree local gate, reconcile only changed facts, then repeat G -> H while concrete safe work remains. If high-quality coherent slices repeatedly finish quickly, enlarge the next coherent closure package rather than stopping after each small commit. Queue exhaustion must be real, not a stale-handoff/watcher artifact.

### I. CONDITIONAL VPS OUTPUT — only if exact-current truth creates a new live question

Current repository truth still says `READY_LIVE: none`. Standing VPS authorization remains valid but is not a reason to duplicate old evidence. Only execute a VPS run if a new implementation/instrumentation change creates a named unresolved real-network question with satisfied dependencies. Otherwise do not unchanged-rerun HY2, repeated warm failover, periodic/soak, package lifecycle, distinct A -> B -> A, already-answered endpoint migration/key update, IPv6 without a real owned IPv6 path, or live PMTUD before its separate authenticated wire/security design gate.

### J. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

The current source-accounting retention/no-reset conflict remains a maintainer/security-policy question. Do not invent TTL, LRU/history capacity, external authority or a weaker reset rule. This policy lane does not block dependency-independent local work after the concrete Session HIGH above is repaired.

## Stop / escalation conditions

Stop only for a real unresolved BLOCKER/HIGH; a core Session/Carrier/ACK/crypto/wire architecture choice; destructive/canonical-meaning migration; action outside standing authorization; production impact; new credentials/server/third-party permission; benchmark-value judgment; repository/tool breakage; actual runtime/tool-budget exhaustion; D019 policy decision; or genuine queue exhaustion. Normal local implementation, test, commit and push must continue without reviewer wake/polling.

No maintainer intervention is requested by this handoff.