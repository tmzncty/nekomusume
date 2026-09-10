# ChatGPT reviewer handoff — close DeliveryLedger lane and repair matrix-probe operator contract

## Reviewed state

- Previous reviewer-owned handoff: exact `ecad9aa045c24978fb6ea9ac19ba549d9764156a` (`docs(handoff): reject globally stale advanced duplicates`).
- Previous reviewed developer implementation/test head: exact `5270ea17af297ae43683dbe46201f755bbbae48a` (`fix: keep advanced duplicate context evidence idempotent`).
- Current developer implementation/test head reviewed this cycle: exact `0f8f19251bdb08c9179265ff26ed432fd07b894d` (`fix: reject globally stale advanced duplicates`).
- Current developer documentation/evidence head reviewed this cycle: exact `c87519df3fb0a2f9067d23e1ca3aed60ca2af24f` (`docs: close global advanced duplicate rollback`).
- Default `main` before this reviewer update was exact `c87519df3fb0a2f9067d23e1ca3aed60ca2af24f`, exactly two commits ahead of the previous reviewer handoff.
- New sequence classification: one Session implementation/test repair (`0f8f192`) plus one docs/local-CI/release-evidence reconciliation (`c87519d`). No research-only fixture, reconstruction, or VPS/WAN experiment landed in this sequence.
- GitHub exposes no hosted combined-status records for exact `0f8f192`. This is not a blocker and must not trigger Actions polling.

## Review verdict

### ACCEPT — global stale advanced-duplicate rollback HIGH is closed on exact `0f8f192`

The developer implemented the minimum shape requested by the previous handoff:

- the component-wise Session context rule is now available as a pure, non-mutating `validate_context` step;
- mutating insertion/confirmation paths still commit the accepted ledger-global context through the existing `context_ok` step;
- a fully covered `InFlight`, `Uncertain`, or `Confirmed` duplicate validates against the ledger-global context before any idempotent success;
- a globally stale request returns `OldEpoch` even when the covered advanced segment itself still stores that stale context;
- a globally admissible request that differs from any covered advanced segment's evidence context returns `InvalidMigration`;
- only globally admissible, context-exact, byte-identical covered advanced duplicates return the existing state idempotently without changing segments, bytes, context, topology, or watermark.

The new disjoint A/B regression matrix exercises `InFlight`, `Uncertain`, and `Confirmed`: A retains `(1,0,1)`, B advances ledger-global context to `(1,1,2)`, stale A duplicate rejects as `OldEpoch`, and globally current-but-segment-mismatched A duplicate rejects as `InvalidMigration`, with exact snapshots preserved.

This closes the concrete rollback escape reported in the prior handoff. No Session/Carrier/ACK/wire/crypto architecture change was introduced.

### ACCEPT — exact-tree developer-local closure and docs reconciliation are internally consistent

The persisted note `docs/local-session-global-context-duplicate-0f8f192-20260910.md` records developer-local exact-tree validation of reachable implementation/test commit `0f8f19251bdb08c9179265ff26ed432fd07b894d`:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` -> exit `0`;
- `git diff --check` -> exit `0`;
- UTC `2026-09-10T01:11:28Z` -> `2026-09-10T01:13:23Z`;
- Linux x86_64, Rust stable `1.98.0`;
- initial/final source tree clean.

`docs/spec/m0-session-state.md`, `docs/specs/nekomusume-session-v0.md`, the release packet, and item-4 factual support now all describe the same advanced-duplicate/context boundary and anchor the developer-local gate to exact `0f8f192` where appropriate. The packet remains an evidence index, not independent review/security approval/release.

### FINAL BOUNDED DELIVERYLEDGER ADJACENT REVIEW — no new BLOCKER/HIGH found; lane is closed

I re-read exact-current `DeliveryLedger::insert -> mark_in_flight -> mark_uncertain -> confirm_received -> watermark` and the focused regressions against the current written candidate contract. Within that bounded contract I found no new demonstrable BLOCKER/HIGH after `0f8f192`:

- rejected insertions validate before committing ledger-global context;
- advanced novel-byte overlap remains fail closed;
- advanced exact duplicates are global-context-valid and evidence-context-exact before idempotent success;
- `confirm_received` validates state, segment context, then ledger-global context before atomically updating segment evidence and watermark;
- packet feedback remains orthogonal to logical delivery confirmation.

Do **not** extend this into another generic interval/checker campaign. In particular, do not redefine watermark contiguity, delivery-ACK encoding, or migration policy without a new concrete contradiction. The bounded DeliveryLedger hardening lane is complete and should remain closed unless a newly observed defect appears.

## New concrete local finding

### MEDIUM / READY_LOCAL — `probe --matrix` misclassifies semantic argument errors as reachability failures

The next smallest real operator-output defect is in the existing local-only reachability command.

Repository contract:

- `docs/reachability-matrix.md` says `neko probe --matrix` exits `0` for reachable, `1` for a completed failed probe, and `2` for invalid arguments;
- the mode is local-loopback only and intentionally refuses public/WAN targets.

Exact-current implementation:

- `matrix_probe` parses syntactic arguments, then calls `reachability::run(...)`;
- `reachability::run` folds `validate(...)` failures — non-loopback target, IP-family mismatch, port 0, timeout outside 1-5000 ms, payload outside 1-1200 bytes — into a normal artifact with `reachable=false`;
- `matrix_probe` then prints the ordinary failure cat/result and exits `1` because it only inspects the generated JSON for `"reachable":true`.

Therefore semantic **invalid arguments are currently reported as completed reachability failures**, contradicting the documented exit-code contract and making evidence classification less truthful.

This is not a network-security expansion. Fix it locally before any new WAN work.

#### Required invariant

- invalid/scope-rejected matrix arguments -> exit `2`, no reachability result claim;
- syntactically and semantically valid probe that completes unreachable/refused/timeout -> exit `1` with the existing failed-case human/JSON boundary;
- valid reachable local probe -> exit `0`;
- non-loopback/public target remains rejected before network probing;
- ordinary authenticated `probe` behavior remains unchanged when `--matrix` is absent;
- no raw socket, ICMP, public-WAN, third-party, production, route/firewall/DNS/proxy/tunnel/qdisc behavior is added.

Preferred minimum shape: validate semantic arguments in `matrix_probe` before calling the operation, or return a small typed validation/result value from `reachability` so exit classification does not depend on treating validation failure as a network observation. Do not build a generic CLI framework.

### LOW / READY_LOCAL WITH THE SAME OPERATOR PACKAGE — matrix mode is poorly discoverable

`probe --matrix` is documented in `docs/reachability-matrix.md` and dispatched by `main`, but the top-level `USAGE` text does not explain the matrix form/options. While touching this operator surface, add one concise help line for the local-only matrix mode and its `--target / --transport / --ip-version / --timeout-ms / --bytes / --json` shape. Keep canonical command identity as `probe`; do not invent a second command.

### LOW / ACCEPT_WITH_BOUNDS — populate the existing observation timestamp field if it stays schema-compatible

`schema/reachability-matrix.v1.json` already requires `observed_at_unix_ms` and permits an integer or null; exact-current `reachability::run` always emits null. Because this is explicitly an evidence artifact, the developer may populate that existing field with a bounded wall-clock Unix-millisecond observation timestamp in the same package, provided:

- schema version remains `reachability-matrix.v1` and no existing field meaning is changed;
- clock acquisition failure remains fail-closed or truthfully null rather than fabricating a value;
- tests assert a sensible bracket/range rather than a hardcoded instant.

This is optional within the package; the exit-code defect above is mandatory.

### DEFER — do not silently redefine `payload_bytes`

Current v1 artifact construction uses `payload_bytes` differently across TCP-connect and UDP-response paths. That deserves a future semantics review, but changing the meaning or shape of a versioned artifact can become a canonical-meaning migration. Do **not** opportunistically rename/reinterpret it in this slice. If exact-current callers require a stronger payload-semantics claim, raise a separate bounded proposal with compatibility consequences first.

## Local-CI-first rule

For the next coherent matrix-operator implementation commit, push the implementation/test tree first, then validate that exact SHA from a clean temporary worktree/clone:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # empty
```

Persist minimum provenance only after the final implementation/test SHA for the coherent package is green: exact SHA, commands, distinct UTC start/end timestamps, exits, host/OS/arch, stable Rust version, and initial/final clean-tree state. If red, repair the concrete failure; do not unchanged-rerun until lucky.

No fuzz is required unless the work unexpectedly changes wire decoder/parser/crypto framing. Developer-local persisted CI, reviewer source review, reviewer-executed CI, and GitHub-hosted CI remain separate evidence classes. Hosted CI is optional cross-evidence and must never become a wait condition.

## Rolling queue — execute continuously in dependency order

Known concrete work is smaller than a truthful 6-12 hour backlog right now. Do **not** manufacture hours. The items below provide seven concrete dependency-ordered slices; after they close, immediately refill the queue through the proposal/implementation loop in H-I rather than entering watcher mode.

### A. READY_LOCAL — repair matrix semantic exit classification

Goal: make the documented 0/1/2 operator contract executable.

Files/concepts: `crates/neko-cli/src/reachability.rs`, `crates/neko-cli/src/main.rs`, `docs/reachability-matrix.md`.

Protected boundary: local-loopback only; semantic invalid input is not a network observation.

Implementation: choose the smallest typed/pre-validation shape that distinguishes invalid input from completed failed reachability without changing ordinary authenticated `probe`.

Continue immediately to B.

### B. READY_LOCAL — direct process regression matrix for 0/1/2 outcomes

Add compact executable coverage for:

- non-loopback target -> exit `2` before probe;
- IP-version mismatch -> exit `2`;
- port `0`, timeout `0/5001`, bytes `0/1201` -> exit `2`;
- valid local TCP target with no listener/refused -> completed failure exit `1`;
- valid reachable local TCP listener -> exit `0`;
- valid UDP loopback responder -> exit `0`;
- human success/failure cat strings and JSON `reachable` result remain aligned with the exit class.

Use bounded local sockets only. Do not add sleeps as correctness synchronization when a listener/readiness primitive is available.

Continue immediately to C.

### C. READY_LOCAL — make matrix mode discoverable without inventing a new command

Update top-level help/USAGE so `probe --matrix` and its local-only option shape are visible. Extend the existing dispatch/help test rather than creating a new documentation checker. Preserve `probe` as the canonical command in capabilities output.

Continue immediately to D.

### D. OPTIONAL READY_LOCAL — fill `observed_at_unix_ms` truthfully

If the implementation remains small and schema-compatible, emit the existing v1 integer timestamp for completed matrix observations and cover it with a bracket/range test. If doing so requires changing schema meaning/version or introduces surprising portability policy, DEFER it and continue; do not block A-C.

Continue immediately to E.

### E. READY_LOCAL — exact-tree gate and concise provenance

After A-D are in one final pushed implementation/test SHA, run the required exact-tree local gate. Preserve a real red result and repair it if necessary. Save one concise local provenance note only after green.

Continue immediately to F.

### F. BOUNDED DOC/RELEASE RECONCILIATION — update only changed operator facts

Reconcile `docs/reachability-matrix.md`, and only if materially affected `docs/status.md`, `docs/release-security-review-packet.md`, or item-4 factual support. Do not re-anchor unrelated Session/RSEC/VPS evidence and do not promote local-loopback matrix behavior into WAN/release/security evidence.

Continue immediately to G.

### G. REVIEW-CLOSURE / READY_LOCAL — close this operator-contract lane if clean

Perform one bounded adjacent review of matrix dispatch -> validation -> socket operation -> artifact/human output -> exit code. Fix only a concrete correctness/evidence contradiction. If clean, explicitly stop extending this lane.

Continue immediately to H.

### H. LOCAL OUTPUT SELECTION — refill with 1-3 real outputs, then choose autonomously

Re-read exact-current `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, release packet, and the implementation touched by candidate outputs. Produce 1-3 concrete dependency-ready proposals and choose the smallest safe one without waiting for the next reviewer hour.

Prefer, in order:

1. a demonstrated runtime/correctness/security defect with a concrete call site;
2. an advertised operator/runtime behavior lacking direct executable evidence;
3. a named release-evidence question answerable locally without capacity/security claim inflation.

Each proposal must name the observed contradiction/missing behavior, owner file/API, protected invariant, minimal positive/negative evidence, and stop condition. Do not select D019 TTL/LRU/history policy, signing/key-custody policy, SBOM publication policy, prior-release interoperability without a frozen prior release, service/production mutation, Experimental Track carriers, or generic checker work merely to create backlog.

Continue immediately to I.

### I. READY_LOCAL / ROLLING VISIBLE OUTPUT — implement, exact-tree close, and repeat

Implement the chosen H output, run the exact-tree local gate, reconcile only changed facts, then repeat H -> I while concrete safe work remains. If coherent work keeps finishing in 10-30 minutes with good test/evidence quality, enlarge the next coherent package rather than stopping after every tiny commit. Queue exhaustion must be real and documented, not a stale-handoff/watcher artifact.

### J. CONDITIONAL VPS OUTPUT — only if exact-current truth creates a new live question

Current repository truth remains `READY_LIVE: none`. Standing VPS authorization is valid but is not a reason to duplicate old evidence. Only execute a VPS run if a new implementation/instrumentation change creates a named unresolved real-network question with satisfied dependencies.

Otherwise do not unchanged-rerun HY2, repeated warm failover, periodic/soak, package lifecycle, distinct A -> B -> A, already-answered endpoint migration/key update, IPv6 without a real owned IPv6 path, or live PMTUD before its separate authenticated wire/security design gate.

### K. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

The source-accounting retention/no-reset conflict remains a maintainer/security-policy question. Do not invent TTL, LRU/history capacity, external authority, or a weaker reset rule. This policy lane does not block dependency-independent local correctness/release work above.

## Stop conditions

Stop and escalate only for an unresolved BLOCKER/HIGH that cannot be safely repaired from existing semantics, a required change to core Session/Carrier/ACK/crypto/wire architecture, destructive/canonical-meaning migration, action outside standing authorization, production impact, new credentials/server/third-party permission, benchmark conditions requiring maintainer value judgment, D019 policy decision, real repository breakage, runtime/tool-budget exhaustion, or genuine queue exhaustion.

Otherwise: coherent slice -> exact-tree local gate -> commit/push -> immediately continue to the next pre-authorized dependency-ready slice.
