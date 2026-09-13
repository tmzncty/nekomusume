# ChatGPT reviewer handoff — reconcile SessionRuntime evidence, then continue deep item-4 core review

## Reviewed repository truth

- Current default branch before this refresh: exact `4b66a444941f85a93865d209f3a6b1f9e850cc14` (`docs(release): index 2026-09-13 deep item-4 core review sweep`).
- The previous handoff is stale: after it, the developer/reviewer sequence completed the SessionRuntime review/repair, package/release-script review, dependency/build-surface review, CLI portability/output review, and one release-packet reconciliation.
- Latest source/test change is reachable exact `9697ee7a0045b39c6ef46c1951fefea486ccf13b` (`fix(session): symmetric terminal cleanup on idle-timeout and close-deadline`). GitHub-hosted `stable checks` and `nightly decode fuzz smoke` are green for that exact SHA. Hosted CI is extra cross-evidence only, not a replacement for developer-local exact-tree provenance.
- No new VPS/WAN experiment occurred. Governance remains unchanged: item 3 incomplete, item 4 incomplete, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, D019 policy-blocked, `READY_LIVE: none`.

## Accepted progress since the previous handoff

The high-throughput core review strategy is working and must continue. Preserve these completed surfaces unless their source changes:

1. reliable UDP recovery / future and never-sent ACK rejection (`531c82d`, `02b6eaa` plus independent close notes);
2. `CarrierState` generation / validation / hysteresis;
3. CarrierManager / migration-back, including switch-margin overflow and negative-margin repairs (`a14cf47`, `91f8cd8`);
4. FairScheduler / flow-control accounting;
5. observability projection/buffer/drop/schema repairs through exact `8e11de0` and independent re-close;
6. carrier adapter close/error/resource review;
7. package/reproducibility release-script review at `8e11de0` — ACCEPT_AS_BOUNDED_SUPPORT;
8. dependency/build-surface review at `8e11de0` — ACCEPT_AS_BOUNDED_SUPPORT;
9. CLI portability + machine/human/exit-code review at `8e11de0` — ACCEPT_AS_BOUNDED_SUPPORT.

These remain bounded item-4 support, not a security audit or release approval.

## EVIDENCE-INTEGRITY HIGH — SessionRuntime review must be reconciled

The independent note `docs/reviews/independent-session-runtime-8e11de0-20260913.md` contains a concrete false no-finding for its claimed exact source anchor. It says that at exact `8e11de0`, idle-timeout and close-deadline `tick()` already cleared `received`, `confirmed`, per-stream inflight/window maps, session-level window counters, and queues consistently with `close_remote()` / `cancel()`.

That statement is contradicted by the later source repair exact `9697ee7`: the repair correctly records that the two `tick()` terminal paths at `8e11de0` cleared only `send`, `recv`, and `queued_bytes`, retaining dedup history, confirmation watermarks, and window/inflight accounting until object drop. Exact `9697ee7` adds shared `clear_runtime_state()` plus focused idle-timeout and close-deadline regressions.

Do **not** rewrite the historical exact-`8e11de0` review note as though it had been correct. Supersede/reconcile it truthfully.

A second evidence-integrity issue is the current release-packet wording that the listed repair SHAs, including `9697ee7`, each have their own exact-tree gate. GitHub currently exposes green hosted checks for exact `9697ee7`, but this reviewer pass did not find a repository-persisted developer-local clean exact-tree provenance note for that repair. Do not infer that no local run happened; the narrower problem is that the shared repository surface does not currently reproduce that local-gate claim.

### MUST_EXECUTE_LOCAL 1 — close exact `9697ee7` provenance

On a clean checkout/worktree of pushed exact `9697ee7` run at minimum:

```bash
cargo test -p neko-session
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --short
```

Persist sanitized provenance: exact SHA, commands, UTC start/end, exit codes, OS/arch, stable Rust version, initial/final clean-tree state. Do not include secrets, private endpoint topology, or unnecessary absolute paths.

If the local gate fails, repair the actual failure first and move the tested-tree anchor to the new reachable pushed source SHA. Do not preserve `9697ee7` as a green local anchor if it was not green.

### MUST_EXECUTE_LOCAL 2 — truthful SessionRuntime evidence reconciliation

After the exact source gate is green:

1. Add one concise reconciliation/supersession note that identifies the incorrect exact-`8e11de0` cleanup sentence and points to the reachable corrected source/test SHA.
2. Update `docs/release-security-review-packet.md` and, where useful, `docs/reviews/release-item4-subgates-20260909.md` so they do not present the old cleanup no-finding as accepted current truth.
3. Keep the historical note immutable except for an explicit superseded/quarantined marker only if the repository's documentation practice requires it; do not rewrite its historical test commands/timestamps or pretend it described a different tree.
4. State hosted CI and developer-local exact-tree evidence separately.
5. No release flag changes follow.

This HIGH is evidence correctness, not a reason to reopen already-closed reliable/Carrier/observability work.

## POLICY/SECURITY GAP — SessionRuntime event retention requires maintainer choice

The SessionRuntime review also found a distinct retained-state issue that remains real on current source: `SessionRuntime.events: Vec<RuntimeEvent>` grows on every event and is not drained or capped, while `SECURITY.md` requires per-connection/global memory, CPU and rate bounds.

The existence of **some** bound/retention mechanism follows from the committed security boundary, but the repository does not currently choose the policy shape or value. Do not invent a numeric cap, TTL, LRU/history size, ring capacity, or evidence-loss semantics. Plausible policy families include a bounded ring with explicit dropped-event evidence, an explicit drain/consumer contract with a hard fallback cap, or another reviewed bounded retention design; selecting among them is maintainer/security policy.

Classify this as `POLICY_BLOCKED_RESOURCE_BOUND`, keep item 4 open, and continue all independent non-dependent work below. Do not let this one policy choice idle the agent.

## Deep pre-authorized queue after HIGH closure

The repository-wide inventory still has substantive implemented surfaces without a dedicated deep independent challenge. Continue through these without reviewer wait.

### REVIEW_LOCAL 3 — DeliveryLedger deep independent challenge

Primary owner: the delivery-ledger half of `crates/neko-session/src/lib.rs` plus `docs/specs/nekomusume-session-v0.md` and existing ledger tests/evidence.

Challenge, independently from the prior developer review:

- insertion/merge/bridge behavior and byte-exact overlap;
- `Unsent -> InFlight -> Uncertain/Confirmed` ownership;
- ledger-global context monotonicity and atomic rejection;
- watermark advancement under out-of-order confirmation;
- per-stream/global resource accounting and offset/reorder/overflow bounds;
- advanced-state exact duplicates versus novel-byte overlap;
- no packet/path observation manufacturing Session delivery evidence.

Concrete contradiction -> smallest repair/regression. No finding -> dedicated bounded no-finding note with exact reachable source anchor.

### REVIEW_LOCAL 4 — ProcessMessage / Resume / readiness codec boundary

Primary owner: `ProcessMessage`, `ResumeWireBinding`, `ReadinessRequest/Response` encode/decode in `neko-session`, plus direct CLI call sites/tests.

Challenge:

- exact length and trailing-byte rejection;
- `usize`/`u64` conversion and offset/length overflow;
- malformed `admitted` values;
- Data/DeliveryAck/Resume semantic separation;
- session/path-generation/delivery-epoch tuple preservation;
- no decode success silently creating authentication, admission, delivery, or readiness evidence;
- deterministic round-trip and unknown-kind/version failure.

If code changes touch this external byte decoder/framing surface, use repository pinned decode fuzz policy as applicable; do not broaden wire semantics.

### REVIEW_LOCAL 5 — bounded unreliable `DatagramRuntime`

Primary owner: `DatagramRuntime` / `DatagramCounters` and `docs/spec/m2-unreliable-datagram.md`.

Challenge:

- admitted/opened/dropped/oversize/queue-drop counter relationships;
- closed-state behavior and queue release;
- payload/queue bounds before allocation;
- no ACK/retransmit/order/delivery-evidence promotion;
- mixed reliable/unreliable resource separation where implemented;
- saturating counters cannot fabricate a stronger claim.

The prior observability mixed-drop fix is downstream projection; do not treat it as a substitute for reviewing the runtime itself.

### REVIEW_LOCAL 6 — crypto integration/API boundary, no cryptanalysis

Primary owner: `neko-crypto` and directly relevant call sites/tests.

This is a dedicated independent implementation/API challenge, not cipher/Noise cryptanalysis and not dependency suitability approval. Check current implemented semantics for:

- Noise pattern/prologue/transcript binding actually used at both roles;
- trust/authz before protected data admission;
- direction/context/epoch/key-phase separation;
- nonce uniqueness/exhaustion and replay/old-phase rejection;
- synchronized/unsynchronized key update state transitions;
- secret-safe error/debug/display surfaces;
- any API shape that permits current committed invariants to be bypassed without changing architecture.

D019/persistent replay policy and cryptographic primitive selection remain outside this lane.

### REVIEW_LOCAL 7 — `neko-bench` local evidence semantics

Primary owner: `crates/neko-bench/src/main.rs` plus any schema/docs that consume `era4-i-performance.v1`.

Challenge measurement/evidence correctness, not speed:

- iteration clamp/bounds and allocation proportionality;
- whether failed operations are included in latency distributions and whether docs/schema state that truthfully;
- median/P95 indexing semantics and empty/failure behavior;
- crypto setup/nonces do not accidentally invalidate repeated samples;
- benchmark names/units/claims match what is measured;
- scheduler/recovery samples do not overclaim end-to-end behavior;
- no superiority or WAN claim from local microbenchmark.

Do not redesign benchmarking or run comparative WAN performance unless a new concrete live question becomes READY.

### REVIEW_LOCAL 8 — remaining CLI subcommands / command-specific truth boundaries

The recent CLI portability review was broad but static. Inventory major subcommands/runners for command-specific factual boundaries not already independently challenged, especially capability/probe/failover/periodic/multistream/key-update/package-facing output paths. Look for success/READY before actual completion, partial-result loss, exit-code inconsistency, secret-bearing debug/error paths, and inconsistent config-validation-before-side-effect ordering.

Do not redo already-closed pre-auth responder inventory or TCP EOF/RST semantics absent changed source.

### REVIEW_LOCAL 9 — static algorithmic resource boundedness cross-sweep

Across implemented core owners, search for:

- external-magnitude loops where retained state is bounded but work is not;
- allocation/clone before validated length/count bounds;
- retries without bounded progress/deadline;
- counters/arithmetic saturation or overflow that can invert a security/selection gate;
- maps/queues/history that exceed declared hard limits;
- diagnostic/evidence work that scales with attacker-controlled magnitudes.

Do not convert this into RSEC-001 pressure/capacity testing and do not choose new policy values. Record policy-dependent gaps separately.

### REVIEW_LOCAL 10 — benchmark/result validators and evidence-prefix preservation

Review deterministic/netns/result-validator paths not already exhausted by the HY2 methodology review. Challenge schema/producer consistency, partial-prefix preservation, failure classification, cleanup truth, sample-count/median/P95 suppression for incomplete data, and immutable negative artifacts. Avoid re-reviewing the already closed HY2 key-exclusivity/schema defect unless source changed.

### REVIEW_SUPPORT 11 — second deep-sweep reconciliation

After roughly 3–4 additional coherent slices (or sooner if another HIGH appears), update the release packet/subgate index so accepted independent reviews and repairs have reachable anchors and precise exclusions. `docs/status.md` changes only if a capability/governance status actually changes.

Do not self-declare item 4 complete merely because more sub-surfaces have been reviewed.

### REVIEW_LOCAL 12 — broad repository-wide core inventory before any `queue exhausted`

Inventory every implemented core crate and release tool against the independent-review index. Only declare queue exhaustion when all substantial implemented core surfaces have a dedicated bounded challenge or an explicit reason they are already subsumed, no concrete repairable defect remains, no READY review-support exists, no READY live question exists, and remaining items are genuinely policy/environment/external-release-authority gates.

## Queue discipline

- The evidence-integrity HIGH is first. Close it without waiting for hosted CI.
- After that, continue REVIEW_LOCAL 3 onward immediately; the policy-blocked event-retention question does not block them.
- Preserve prior accepted no-finding reviews; do not redo them unless their owner source changes.
- A bounded no-finding independent challenge of a previously unreviewed implemented core surface is valid item-4 support, not filler.
- Any concrete existing-semantics defect becomes smallest repair -> regression -> pushed exact-tree local gate/provenance -> continue.
- Preserve a deep queue; do not collapse the handoff to one ticket after each small commit.

Stop only for unresolved correctness/security/evidence BLOCKER/HIGH that contaminates downstream work, core architecture/policy decision, destructive/canonical migration, action outside standing authorization, production/third-party action, new credentials/server permission, maintainer-valued pressure/capacity conditions, repository breakage, or actual runtime/tool exhaustion.

## Live / release boundary

- item 3 incomplete;
- item 4 incomplete;
- `RELEASE_CANDIDATE=false`;
- `PRODUCTION_READY=false`;
- `FREEZE=false`;
- `RELEASED=false`;
- D019 remains maintainer policy/value;
- `SessionRuntime.events` retention is `POLICY_BLOCKED_RESOURCE_BOUND` pending maintainer/security choice;
- RSEC-001 representative adversarial-load/capacity suitability remains unestablished and is not converted into a local pressure test;
- signing/key custody/SBOM/publication trust and previous-frozen-release interoperability remain separate gates/dependencies;
- `READY_LIVE: none` remains authoritative.

Standing VPS authorization remains valid, but current repository truth creates no new dependency-ready live question. Do not mechanically rerun HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint migration, key update, IPv6, PLPMTUD or Experimental Track work.
