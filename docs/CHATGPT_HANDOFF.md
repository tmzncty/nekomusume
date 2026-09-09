# ChatGPT reviewer handoff — charge post-Noise pre-auth input before RSEC closure

## Reviewed state

- Previous reviewer-owned handoff: exact `4d23eb4e8e025d1c645bf42ca39007194831db9f` (`docs(handoff): bind cached preauth retries before RSEC closure`).
- Previous reviewed developer implementation/test head: exact `21e42af49a9bbe639aefa3dca887e17b03cad07c` (`test: observe bounded preauth recovery`).
- Previous reviewed developer documentation/evidence head: exact `f9511f33cc8a3f777026e5a7ab78a37142d303c4` (`docs: record bounded preauth process recovery`).
- Current default-branch developer implementation/test head reviewed this cycle: exact `976f90b738315aa7d925b54282a000bcdb139007` (`test: close cached preauth retry ownership`).
- Current default-branch developer documentation/evidence head reviewed this cycle: exact `8f7d5a39f4889c2a5fcde7af4fbc341174dc556b` (`docs: close cached preauth retry boundary`).
- Developer sequence since the previous reviewer handoff:
  - `10e5d4cb147e8e6ea3cf7d612ed99fbe61052211` reconciles the legacy mutable fuzz-seed corpus with the separate frozen canonical-vector corpus and corrects negotiation-integration scope wording; it does not widen release/freeze claims.
  - `cb00f5d1ba2804de7e041fb5039268ed30c12ebd` retains the original failover UDP admission after server-side Noise completion, exact-charges byte-identical cached first-Noise retries, sends them through the existing bounded UDP response helper, suppresses the old post-auth delayed duplicate seam, and releases/cache-clears on first valid authenticated application Data.
  - `976f90b738315aa7d925b54282a000bcdb139007` adds same-owner cached-response ceiling evidence, extends the responder inventory to the cached failover UDP surface, makes the malformed-process source-domain claim executable by holding eight sender sockets simultaneously and asserting eight unique source ports, and asserts temporary identity cleanup.
  - `8f7d5a39f4889c2a5fcde7af4fbc341174dc556b` persists exact-`976f90b` developer-local provenance and reconciles release/security evidence anchors.
- No new VPS/WAN experiment was performed in this sequence.
- GitHub currently exposes no hosted combined-status records for exact `976f90b`. This is not a blocker and must not trigger Actions polling.

## Review verdict

### ACCEPT WITH BOUNDS — cached response ownership, source-domain repair, and exact-tree developer-local gate

The previous cached-response escape is materially repaired: the original source-owned admission is no longer released immediately after server-side Noise completion, byte-identical cached first-Noise retries exact-charge input and response under that same owner, the response goes through the bounded response permit/deadline helper, and the owner/cache are cleared on first valid authenticated application Data. The old opt-in delayed post-auth duplicate is now suppressed rather than emitting pre-auth material after authenticated progress.

The focused model regression proves four charged/suppressed cached response attempts share one source-owned response packet ceiling and the next response is refused. The malformed-process observation now keeps eight local UDP senders alive simultaneously, asserts eight unique source ports before sending one five-byte malformed datagram from each, retains one later authenticated 16-byte recovery exchange, and asserts identity cleanup. These are bounded local engineering observations only, not same-source saturation, adversarial-load suitability, production-capacity evidence, security approval, or D019 policy closure.

`docs/local-cached-preauth-retry-976f90b-20260910.md` records developer-local exact-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit `0`, `git diff --check` exit `0`, UTC `2026-09-09T20:24:29Z -> 20:26:22Z`, Linux/x86_64, Rust 1.98.0, and clean initial/final source tree. Reviewer did not execute this CI. Hosted CI is absent and is not required.

### HIGH / READY_LOCAL — non-matching post-Noise datagrams bypass input/work charging while the retained pre-auth owner is live

The previous HIGH is not fully closed because the newly retained admission is used only for the byte-identical cached first-Noise branch.

Current `failover_server` shape on `main` after `secure.is_some()`:

1. receive a UDP datagram from the established peer;
2. if it byte-matches the cached first Noise message, use `handshake_admission` to `charge_input`, then charge/send the cached response;
3. otherwise, emit the diagnostic and call `ss.open_unreliable(&buf[..n])` directly;
4. only after a valid `ProcessMessage::Data` is opened/accepted does the code release `handshake_admission` and clear `handshake_cache`.

Therefore, while `handshake_admission` is still deliberately live and the peer has not yet demonstrated authenticated application progress, a non-matching datagram can reach AEAD open/classification without any `ListenerAdmission::charge_input` / source-lifetime work charge. Malformed, duplicate-but-not-byte-identical, random ciphertext, or valid encrypted non-Data input in this retained pre-auth ownership window can consume crypto/parser work outside the source/global input/work counters.

This conflicts with the D019 candidate contract that input bytes/packets, including malformed and duplicate input, are charged before parse/work and that an operation unable to charge fails closed. It also makes the current broad resource-review wording that all inventoried pre-auth input/work is charge-before-parse too strong.

Treat this as a correctness/security/evidence **HIGH / READY_LOCAL** for the bounded pre-auth accounting contract. It is not a new wire, Noise pattern, Session, Carrier, ACK, or crypto-design decision, and it is not a production exploit claim. Do not expand unrelated runtime work until it is closed.

#### Required invariant

While the failover UDP `handshake_admission` owner remains live, every datagram from that owned peer must consume the applicable source/global input and work accounting exactly once **before** cached-message classification or `open_unreliable`. Only after the first valid authenticated application Data causes owner release may subsequent normal authenticated traffic leave the pre-auth accounting domain.

#### Preferred minimal implementation shape

Keep the current retained-owner design and move the input/work charge to the common top of the owned-peer receive path while `handshake_admission.is_some()`:

- after peer equality is checked, obtain the retained admission and exact-charge the datagram once before comparing it with `handshake_cache` or calling `ss.open_unreliable`;
- the byte-identical cached retry then reuses that already-charged input and performs only the existing exact response charge/send; do not double-charge input;
- non-matching input may proceed to AEAD open/classification only after the common charge succeeds;
- first valid authenticated application Data still releases the owner exactly once and clears cached retry capability;
- expiry, error, budget exhaustion and experiment termination remain fail-closed and must not silently fall back to an uncharged path;
- do not migrate normal post-progress DeliveryAck traffic into pre-auth accounting.

An equivalent smaller shape is acceptable if it proves the same charge-once-before-work invariant without new policy numbers or new architecture.

#### Minimum focused evidence

- preserve first UDP Noise-response loss/retry recovery;
- prove the cached duplicate is input-charged exactly once, not twice;
- prove at least one non-matching post-Noise/pre-Data datagram consumes the same retained owner’s input/work budget before AEAD open/classification;
- prove the first operation beyond the configured input/work ceiling fails closed before `open_unreliable` work and produces no Delivery/PathValidated/ACK-equivalent evidence;
- preserve owner/cache cleanup on first valid authenticated Data, expiry/error and normal experiment termination with zero orphan live state;
- keep the responder inventory truthful after the call-order change.

No fuzz is required unless the change actually touches wire decoder/parser/crypto framing rather than only call ordering/accounting.

## Evidence boundary after the HIGH

The exact-`976f90b` local green gate remains valid developer-local evidence for that tree, but it does not close this newly identified post-Noise input/work charging gap. `docs/reviews/resource-abuse-evidence-2026-09-04.md`, `docs/release-security-review-packet.md`, and `docs/reviews/release-item4-subgates-20260909.md` may continue to index exact `976f90b` historically, but their broad current engineering-control claims must be re-anchored/narrowed only after the replacement implementation/test SHA is green.

RSEC-001 remains open for adversarial-load/promotion suitability and independent review even after this engineering gap is repaired. D019 source-retention/no-reset policy remains separately blocked.

## Local-CI-first rule

For implementation/test slices below, push the coherent developer SHA first, then validate that exact SHA in a clean temporary worktree/clone:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # empty
```

Persist minimum provenance only after the final replacement SHA is green: exact SHA, commands, distinct UTC start/end timestamps, exits, host/OS/arch, stable Rust version, and initial/final clean-tree state. If red, preserve and repair the concrete failure; do not unchanged-rerun until lucky. Developer-local persisted CI, reviewer-executed CI and hosted CI remain separate evidence classes. Hosted CI is optional cross-evidence and must never become a wait condition.

## Rolling queue — execute continuously in dependency order

The coding agent must continue through every dependency-ready item below without waiting for the next reviewer hour. The HIGH is first and blocks expansion into unrelated runtime work, but it does not require maintainer approval because the accounting invariant and candidate numbers already exist.

### A. HIGH / READY_LOCAL — common-charge every retained-owner UDP datagram before work

Implement the minimal current-main repair above. Use the existing `handshake_admission`, existing `charge_input` API, existing candidate limits and current release-on-authenticated-Data transition. No new capacity, TTL, LRU/history, wire field, crypto primitive, Session semantic or carrier architecture is authorized.

Continue immediately to B.

### B. HIGH-CLOSURE / READY_LOCAL — exact-once and non-matching input regression

Add focused tests that distinguish cached-match input from non-matching post-Noise/pre-Data input and prove common-owner exact-once charging, over-limit fail-closed behavior before AEAD work, and cleanup. Preserve the existing loss/retry positive and ordinary authenticated application success.

Perform exactly one bounded source inspection of the final failover UDP pre-auth lifecycle for equivalent pre-progress input/work bypasses. Fix concrete equivalents only; do not create another generic checker framework.

Continue immediately to C.

### C. READY_LOCAL — final exact-tree gate and concise provenance

After A-B land in the final pushed implementation/test SHA, run the exact-tree local gate. Persist one concise provenance note with exact SHA, commands, UTC interval, host/OS/arch, stable Rust, clean-tree state, focused charge-once tests and only sanitized aggregate observations. If red, repair the failure before closure. Do not wait for GitHub Actions.

Continue immediately to D.

### D. BOUNDED RELEASE/SECURITY RECONCILIATION — re-anchor only proven facts

Update together after the final green developer SHA:

- `docs/reviews/resource-abuse-evidence-2026-09-04.md`;
- `docs/release-security-review-packet.md`;
- `docs/reviews/release-item4-subgates-20260909.md`;
- the concise local provenance note and responder inventory if needed.

Allowed claim: exact bounded local evidence exists that the retained failover UDP pre-auth owner charges every pre-progress peer datagram before classification/AEAD work and cached responses remain exact-charged under that owner.

Forbidden claims: D019 resolved, RSEC-001 fully closed, candidate limits production-suitable, independent review complete, public listener approved, RC/release/production ready.

Continue immediately to E.

### E. REVIEW-CLOSURE / READY_LOCAL — close this RSEC engineering lane and stop extending the harness

Perform one bounded adjacent review of the repaired failover UDP pre-auth lifecycle for charge ordering, exact-once accounting, release/expiry/error cleanup, stale tested-tree claims and accidental policy invention. Fix concrete defects found. If no BLOCKER/HIGH remains, explicitly mark this bounded RSEC engineering package complete and **stop extending this checker/test lane unless a new demonstrated defect appears**.

Continue immediately to F.

### F. LOCAL OUTPUT SELECTION — choose the next real runtime/operator/release output

Re-read exact-current `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, release packet and relevant code. Produce 1-3 dependency-ready proposals and autonomously choose the smallest safe one. Prefer, in order, a demonstrated runtime/correctness/security defect with a concrete call site, an advertised operator/runtime behavior lacking direct executable evidence, or a named release-evidence question answerable locally without capacity/security claim inflation.

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