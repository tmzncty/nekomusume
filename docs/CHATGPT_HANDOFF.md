# ChatGPT reviewer handoff — tear down expired retained UDP pre-auth session before evidence

## Reviewed state

- Previous reviewer-owned handoff: exact `c0b33de958e6eb77a295974852671ffbaa56cd81` (`docs(handoff): charge post-Noise preauth input before RSEC closure`).
- Previous reviewed developer implementation/test head: exact `976f90b738315aa7d925b54282a000bcdb139007` (`test: close cached preauth retry ownership`).
- Previous reviewed developer documentation/evidence head: exact `8f7d5a39f4889c2a5fcde7af4fbc341174dc556b` (`docs: close cached preauth retry boundary`).
- Current default-branch developer implementation/test head reviewed this cycle: exact `5be8c6e38e2f70a8d6766eb7337e156640a5b565` (`test: isolate selection retry from post-Noise budget`).
- Current default-branch developer documentation/evidence head reviewed this cycle: exact `a9aed3b9afcc0545518affad92e95dfb0281c9ef` (`docs: close retained UDP input boundary`).
- Developer sequence since the previous reviewer handoff:
  - `17b39e34da87b671865a9749196f376d9b46d5af` moves retained-owner UDP input/work charging to the common owned-peer receive path before cached-message comparison or AEAD open and removes the cached-branch double charge.
  - `e32a1138a746998ed9278b40d914b9dc185866fa` adds focused same-owner cached/non-matching accounting and first-excess refusal evidence. Its first exact-tree full gate was red because an independent selection-loss fixture mixed an unrelated late post-Noise hello into the deliberately small four-packet budget.
  - `5be8c6e38e2f70a8d6766eb7337e156640a5b565` removes only that unrelated late-hello injection from the selection-loss fixture, preserving its selection retry/recovery purpose; this is the replacement green tested tree.
  - `a9aed3b9afcc0545518affad92e95dfb0281c9ef` persists exact-`5be8c6e` developer-local provenance and re-anchors the resource review, release packet and item-4 factual support.
- No new VPS/WAN experiment was performed in this sequence.
- GitHub currently exposes no hosted combined-status records for exact `5be8c6e`. This is not a blocker and must not trigger Actions polling.

## Review verdict

### ACCEPT WITH BOUNDS — common retained-owner input charge and truthful exact-tree local closure

The previous post-Noise input/work charging HIGH is materially repaired for the live-owner path. While the retained failover UDP `handshake_admission` exists, every datagram from the owned peer is now charged once before cached first-Noise comparison or `open_unreliable`; a byte-identical cached retry performs only the response charge after that common input charge. The first valid authenticated application Data still releases the owner/cache, so later normal authenticated traffic remains outside pre-auth accounting.

The focused model test now combines one cached retry plus non-matching datagrams under one four-packet input ceiling and proves the first excess input terminalizes the state. The process positive continues to prove first Noise-response loss/retry and records exactly one retained-input charge for the cached retry plus one for the first non-matching authenticated Data.

`docs/local-retained-udp-input-5be8c6e-20260910.md` records developer-local exact-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit `0`, `git diff --check` exit `0`, UTC `2026-09-09T21:18:07Z -> 21:19:59Z`, Linux/x86_64, Rust 1.98.0, and clean initial/final source tree. Exact `e32a113` is retained as the red integration attempt rather than being rewritten as green. Reviewer did not execute this CI. Hosted CI is absent and is not required.

These facts remain bounded engineering evidence only. They do not establish adversarial-load or production-capacity suitability, independent security approval, public-listener approval, RC/release/production readiness, or D019 source-retention policy closure.

### HIGH / READY_LOCAL — expiry drops the accounting owner but leaves the authenticated pre-progress UDP session able to produce DeliveryAck

The retained-owner repair still has a concrete terminal-lifecycle escape.

At the top of the failover-server loop, `preauth.expire()` removes expired process states. When the returned IDs contain `handshake_admission`, current code sets only:

- `handshake_admission = None`;
- `handshake_cache = None`.

It does **not** clear `secure` or the pre-progress `ResumeGuard`.

The same loop then skips fresh negotiation because `secure.is_none()` is false and enters the existing `if let Some((ref mut ss, peer)) = secure` receive path. Because `handshake_admission` is now `None`, the new common charge block is skipped. A later datagram from that peer can therefore reach `open_unreliable`; if it is a valid `ProcessMessage::Data`, current code can admit it into `SessionRuntime` and emit a DeliveryAck even though the pre-auth owner already expired.

That directly conflicts with D019's existing candidate contract: idle/lifetime expiry tears down the bounded pre-auth state without response evidence, and timeout/exhaustion must produce no Delivery/PathValidated/ACK-equivalent evidence. It also means the current broad statement that pre-progress failover UDP input is charge-before-work until authenticated progress is too strong across the expiry transition.

Treat this as a correctness/security/evidence **HIGH / READY_LOCAL** for the existing bounded contract. It is not a production exploit claim and does not require a new Session, Carrier, ACK, crypto, Noise-pattern or wire decision.

#### Required invariant

If the retained failover UDP pre-auth owner expires before the first valid authenticated application Data, the associated pre-progress authenticated transport state must become terminal before any later peer datagram can reach AEAD/classification or create delivery/path/ACK evidence. Expiry must not turn `handshake_admission: Some` into an uncharged `secure: Some` path.

#### Preferred minimal implementation shape

Use current state and existing D019 deadlines; do not add a new timeout number.

- when `preauth.expire()` reports the live `handshake_admission` ID, clear the complete pre-progress UDP handshake/session capability tied to that owner, not only the ticket/cache;
- at minimum retire `handshake_admission`, `handshake_cache`, `secure`, and the pre-progress `guard` before the receive/classification path can run;
- reset only clearly session-local test/diagnostic state needed to permit a fresh bounded negotiation; do not reset source-lifetime accounting or invent a new retention policy;
- a datagram from the expired peer must require a fresh admitted negotiation rather than reuse the expired secure context;
- no expired-session Data may cause `runtime.receive`, DeliveryAck, PathValidated, resume admission or equivalent success evidence;
- if a nearby budget-exhaustion path can be made state-scoped with the same small teardown helper, that is acceptable, but do not create a generic lifecycle framework merely for this slice.

An equivalent smaller shape is acceptable if it proves the same terminal-before-evidence invariant and preserves ordinary first-Noise retry/recovery.

#### Minimum focused evidence

- deterministic or process evidence that establishes a failover UDP session through negotiation/Noise but delays first application Data past the current pre-auth idle deadline;
- prove the expired session cannot obtain a DeliveryAck, cannot resume through the stale guard, and cannot continue through the old `secure` object;
- prove cleanup leaves no orphan pre-auth state/cache/owner;
- if the server is intentionally reusable during its bounded experiment window, prove a fresh admitted negotiation can still succeed after the expired pre-progress attempt; otherwise document the exact process-lifetime boundary without inflating the claim;
- preserve first Noise-response loss/retry recovery and the exact-once retained-input charge positive;
- preserve expiry/error cleanup and responder inventory truth.

No fuzz is required unless the repair touches decoder/parser/crypto framing rather than only ownership/lifecycle ordering.

### LOW / SAME-SLICE CLEANUP — stale selection-loss test comment

The selection-loss fixture no longer enables the late post-auth UDP hello seam, but one nearby comment still says that both the duplicate pending hello and a late post-auth hello are being proved. Correct that comment when touching the test; do not create a standalone docs/test commit for it.

## Evidence boundary after the HIGH

Exact `5be8c6e` remains valid developer-local evidence for common charging while the owner is live. It does not prove the expiry transition is terminal. `docs/reviews/resource-abuse-evidence-2026-09-04.md`, `docs/release-security-review-packet.md`, and `docs/reviews/release-item4-subgates-20260909.md` may retain exact `5be8c6e` historically, but their broad current engineering-control wording must be narrowed or re-anchored only after the replacement implementation/test SHA is green.

RSEC-001 remains open for adversarial-load/promotion suitability and independent review even after this engineering defect is repaired. D019 source-retention/no-reset policy remains separately `SOURCE_RETENTION_POLICY_BLOCKED`.

## Local-CI-first rule

For implementation/test slices below, push the coherent developer SHA first, then validate that exact SHA in a clean temporary worktree/clone:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # empty
```

Persist minimum provenance only after the final replacement SHA is green: exact SHA, commands, distinct UTC start/end timestamps, exits, host/OS/arch, stable Rust version, and initial/final clean-tree state. If red, preserve and repair the concrete failure; do not unchanged-rerun until lucky. Developer-local persisted CI, reviewer-executed CI and hosted CI remain separate evidence classes. Hosted CI is optional cross-evidence and must never become a wait condition.

## Rolling queue — execute continuously in dependency order

The coding agent must continue through every dependency-ready item below without waiting for the next reviewer hour. The HIGH is first and blocks expansion into unrelated runtime work, but it does not require maintainer approval because the expiry and evidence-barrier semantics already exist.

### A. HIGH / READY_LOCAL — retire the complete expired pre-progress UDP capability

Implement the minimal current-main repair above. Reuse the existing expiry result and current pre-auth owner/session variables. No new TTL, capacity, retention history, wire field, crypto primitive, Session semantic or carrier architecture is authorized.

Continue immediately to B.

### B. HIGH-CLOSURE / READY_LOCAL — expiry-before-first-Data regression

Add the smallest stable evidence that makes the expiry transition executable. Prove stale secure/guard state cannot create application delivery or resume evidence after owner expiry, cleanup is complete, and the ordinary cached Noise retry/recovery positive remains intact. Correct the stale selection-loss comment in this same coherent test slice.

Perform exactly one bounded source inspection of the final failover UDP pre-progress terminal paths for an equivalent `owner gone, secure path still live` transition. Fix only a concrete equivalent; do not extend the checker/harness lane generically.

Continue immediately to C.

### C. READY_LOCAL — final exact-tree gate and concise provenance

After A-B land in the final pushed implementation/test SHA, run the exact-tree local gate. Persist one concise provenance note with exact SHA, commands, UTC interval, host/OS/arch, stable Rust, clean-tree state and focused expiry/retained-owner positives. If red, repair the concrete failure before closure. Do not wait for GitHub Actions.

Continue immediately to D.

### D. BOUNDED RELEASE/SECURITY RECONCILIATION — re-anchor only proven facts

Update together after the final green developer SHA:

- `docs/reviews/resource-abuse-evidence-2026-09-04.md`;
- `docs/release-security-review-packet.md`;
- `docs/reviews/release-item4-subgates-20260909.md`;
- responder inventory / concise provenance only if their facts changed.

Allowed claim: exact bounded local evidence exists that retained failover UDP input is charged before work while ownership is live and owner expiry retires the associated pre-progress secure/resume capability before delivery evidence.

Forbidden claims: D019 resolved, RSEC-001 fully closed, candidate limits production-suitable, independent review complete, public listener approved, RC/release/production ready.

Continue immediately to E.

### E. REVIEW-CLOSURE / READY_LOCAL — end this RSEC engineering inspection lane if clean

Perform one bounded adjacent review of the repaired failover UDP pre-progress lifecycle for charge ordering, expiry/error cleanup, stale secure/guard ownership, evidence production and stale tested-tree claims. Fix concrete defects found. If no BLOCKER/HIGH remains, explicitly mark this bounded RSEC engineering inspection package complete and **stop extending this checker/test lane unless a new demonstrated defect appears**.

Continue immediately to F.

### F. LOCAL OUTPUT SELECTION — choose the next real runtime/operator/release output

Re-read exact-current `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, release packet and relevant code. Produce 1-3 dependency-ready proposals and autonomously choose the smallest safe one. Prefer, in order: a demonstrated runtime/correctness/security defect with a concrete call site; an advertised operator/runtime behavior lacking direct executable evidence; or a named release-evidence question answerable locally without capacity/security claim inflation.

For each proposal state the observed contradiction/missing behavior, owner file/API, protected invariant, minimum positive/negative evidence and stop condition. Do not wait for reviewer selection when one option is clearly smallest and within existing architecture/authorization.

Do not choose D019 source-retention TTL/LRU/history policy, signing/key-custody policy, SBOM publication policy, previous-release interoperability without a frozen prior release, Experimental Track carriers, or generic checker work merely to manufacture hours.

Continue immediately to G.

### G. READY_LOCAL / ROLLING VISIBLE OUTPUT — implement, exact-tree close, repeat

Implement the chosen F output, run the exact-tree local gate, reconcile only changed facts, then repeat F -> G while concrete safe work remains. If high-quality coherent slices repeatedly finish quickly, enlarge the next coherent closure package rather than stopping after each small commit. Queue exhaustion must be real, not a stale-handoff/watcher artifact.

### H. CONDITIONAL VPS OUTPUT — only if exact-current truth creates a new live question

Current repository truth still says `READY_LIVE: none`. Standing VPS authorization remains valid but is not a reason to duplicate old evidence. Only execute a VPS run if a new implementation/instrumentation change creates a named unresolved real-network question with satisfied dependencies. Otherwise do not unchanged-rerun HY2, repeated warm failover, periodic/soak, package lifecycle, distinct A -> B -> A, already-answered endpoint migration/key update, IPv6 without a real owned IPv6 path, or live PMTUD before its separate authenticated wire/security design gate.

### I. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

The current source-accounting retention/no-reset conflict remains a maintainer/security-policy question. Do not invent TTL, LRU/history capacity, external authority or a weaker reset rule. This policy lane does not block dependency-independent local work after the concrete HIGH above is repaired.

## Stop / escalation conditions

Stop only for a real unresolved BLOCKER/HIGH; a core Session/Carrier/ACK/crypto/wire architecture choice; destructive/canonical-meaning migration; action outside standing authorization; production impact; new credentials/server/third-party permission; benchmark-value judgment; repository/tool breakage; actual runtime/tool-budget exhaustion; D019 policy decision; or genuine queue exhaustion. Normal local implementation, test, commit and push must continue without reviewer wake/polling.

No maintainer intervention is requested by this handoff.