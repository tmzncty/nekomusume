# ChatGPT reviewer handoff — HY2 typed negative accepted; reconcile current-line truth

## Reviewed state

- Previous reviewer baseline: `312879e153c3dcfdab60398638f8cc11ec69e910`.
- Pre-handoff default `main`: `e50b9ddda2d206c4b2fcee4e3d13b544f07d036a`.
- New coding/documentation commits reviewed this round:
  - `13da094f4e2541cc2d047113cc1002120817ced9` — `docs: correct preauth review state`.
  - `e50b9ddda2d206c4b2fcee4e3d13b544f07d036a` — `docs: retain typed HY2 diagnostic negative`.
- Exact `e50b9dd` GitHub Actions is green: stable `scripts/check.sh` and nightly decode fuzz smoke both succeeded.
- No active work branch is ahead of `main`; `work/e1a-staged-accounting-20260907` is stale/behind and must not be treated as current task truth or merged for coordination alone.

## Review verdict

### ACCEPT — RSEC claim correction

`13da094` correctly retracts the premature `ENGINEERING_EVIDENCE_CLOSED` wording. Current honest boundary is:

- `ENGINEERING_CONTROLS_PRESENT`;
- `INDEPENDENT_REVIEW_OPEN`;
- `SOURCE_RETENTION_POLICY_BLOCKED`.

Full RSEC/D019 closure is **not** claimed. The source-retention conflict in `docs/adr/m1-g0-preauth-source-retention-amendment-request.md` remains a maintainer/security-policy checkpoint. Agent/reviewer must not invent TTL/LRU/history/capacity values, silently weaken D019, or turn cleanup-reset behavior into a compliance claim.

### ACCEPT — one bounded HY2 real-VPS negative, not a comparison result

`e50b9dd` retains a materially changed, standing-authorized self-owned HY2 attempt against exact tested tree `13da094` after exact-head CI was green. The retained evidence is narrow:

- same bounded 1200-byte application payload and existing fair-pair harness;
- temporary Hysteria v2.9.3 on the owned VPS; production Hysteria untouched;
- `nekomusume-1` completed the application exchange;
- `hy2-1` exited before application bytes, so zero complete Nekomusume↔HY2 pair exists;
- typed diagnostic is `category=unknown`, `last_success_stage=client_started`, `last_success_source=harness`;
- bounded private diagnostic body remains untracked; tracked output carries only bounded/hash metadata;
- local and remote cleanup were verified;
- therefore there is **no median/P95, performance, superiority, production, or general HY2 reliability conclusion**.

The current HY2 client shape (`insecure: true` plus `pinSHA256` for the temporary self-signed certificate) is compatible with upstream Hysteria's documented certificate-pinning model; do not create a security-config blocker merely because `insecure` appears in the temporary lab config.

### HIGH — current-line truth drift remains in authoritative docs

The new HY2 artifact is acceptable, but current planning/status text is internally inconsistent:

- one current row already records exact `13da094` as `BLOCKED_HARNESS_CURRENT_LINE`, with `unknown/client_started`;
- later present-tense text still calls exact `61a6490` the **current** HY2 line and labels it `BLOCKED_ORCHESTRATION_CURRENT_LINE_HY2`;
- `IMPLEMENTATION_PLAN.md` still has an unchecked current-line item pointing at the old `61a6490` local preflight.

This is evidence/status truth drift, not a new runtime defect. Fix it as one small reconciliation slice. Historical dated records for `61a6490` must remain historical facts; do not rewrite old evidence to pretend it never happened.

### HY2 lane state after this review

`FROZEN_NO_RETRY` for the same-class fair-pair attempt. The `unknown/client_started` negative is now preserved. No new generic HY2 diagnostic framework and no additional VPS retry are authorized by mere curiosity. A future retry requires a **concrete new hypothesis plus material code/config/instrumentation/path change** that can distinguish something the current artifact cannot.

## Design / proposal protocol

The external coding agent remains an active designer, not a passive ticket executor. For local implementation/test shapes inside the current ADRs it may compare 1–3 minimal designs and choose the one with the least new state/API/policy and easiest fail-closed validation, then implement/test/commit/push without waiting for reviewer preapproval.

This autonomy does **not** extend to new Session/Carrier/ACK/crypto/wire semantics, numeric security policy, destructive migration, production-network changes, third-party targets, or D019 source-retention policy. Those remain real escalation boundaries.

## Rolling queue

### A. HIGH / READY_LOCAL — reconcile HY2 current-line truth

**Goal:** make current authoritative planning text agree with the retained `13da094` HY2 negative.

**Why now:** the runtime/evidence lane already produced a new real-VPS result; current docs now disagree about which result is current.

**Files:** `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md`; touch dated Era-4 ledger files only if an existing repository consistency rule truly requires a current-state pointer update, and never erase their explicit historical anchor.

**Protected invariant:** exact `61a6490` remains a historical preflight/orchestration negative; exact `13da094` is the current bounded HY2 line and is only `BLOCKED_HARNESS_CURRENT_LINE` with `unknown/client_started`; no completed pair/performance claim.

**Behavior:** remove or reword present-tense contradictions so old rows are explicitly historical. Do not mutate the HY2 artifact and do not rerun HY2.

**Tests/gates:** focused grep/current-state consistency as useful, `scripts/check.sh`, `git diff --check`, exact-head CI.

**Commit/push:** yes. Immediately continue to B when green.

### B. MEDIUM / READY_LOCAL — tighten pre-auth pending-owner checker locality

**Goal:** make static ownership evidence fail closed within each inventoried responder region rather than pass because the same anchor string appears elsewhere in `main.rs`.

**Why now:** `failover_udp_pending` and `failover_udp_new` are persisted pending owners; current runtime inventory is correct, but checker locality is weaker than its claim.

**Files:** `scripts/check-preauth-responder-inventory.py`, focused checker tests/fixtures if needed, and `docs/preauth-responder-inventory.v1.json` only if a current anchor must become responder-local.

**Protected invariant:** no pre-auth numeric policy or runtime behavior changes; this is review-precision only.

**Behavior:** region-scope admission-owner, success/expiry/rejection cleanup, reserve/store/cancel ownership anchors. Add at least one negative where an identical anchor exists outside the responder region and must not satisfy the responder.

**Tests/gates:** focused checker tests, `scripts/check.sh`, `git diff --check`, exact-head CI.

**Commit/push:** yes. Immediately continue to C.

### C. MEDIUM / READY_LOCAL — lock the legacy repeated-failover digest exception

**Goal:** prove the schema's one exact historical digest exception does not open a class of malformed digests.

**Why now:** the schema intentionally preserves one frozen historical value while new evidence should remain strict 64-hex.

**Files:** prefer `scripts/bench/run-repeated-warm-failover-test.py`; change `schema/repeated-warm-failover.v1.json` only if the test exposes an actual schema defect.

**Protected invariant:** do not rewrite retained historical artifacts and do not reopen the frozen repeated-warm-failover VPS line.

**Behavior/tests:** Draft 2020-12 must accept the exact frozen legacy digest, accept a normal 64-hex digest, and reject at least one arbitrary sibling malformed digest. Keep corpus-wide retained-artifact validation.

**Gates:** focused test, `scripts/check.sh`, `git diff --check`, exact-head CI.

**Commit/push:** yes. Immediately continue to D.

### D. HIGH SECURITY REVIEW / READY_LOCAL except D019 policy — exact-tree partial RSEC review

**Goal:** independently review the integrated exact tree for the non-policy parts of pre-auth resource accounting.

**Why now:** controls and adversarial tests exist, but independent exact-tree review is still explicitly open.

**Files:** current `crates/neko-cli/src/preauth.rs`, responder call sites, `crates/neko-cli/tests/probe.rs`, responder inventory/checker, `docs/reviews/resource-abuse-evidence-2026-09-04.md`, relevant D018/D019 ADR text.

**Protected invariants:** charge before expensive parse/auth work; response bytes accounted; all pre-auth listener surfaces covered; TCP/UDP source domains non-colliding; redacted observability; bounded concurrency/queue/memory behavior. Do not invent source-retention policy.

**Review behavior:** check source projection, admission/charge ordering, release/expiry/rejection cleanup, responder coverage, adversarial limit tests, and evidence provenance against the **current exact tree**. If a concrete engineering defect is found, repair it locally with deterministic tests and continue. If non-policy controls pass, state only a bounded result such as `ENGINEERING_CONTROLS_REVIEWED`; full RSEC/D019 remains `SOURCE_RETENTION_POLICY_BLOCKED` until maintainer policy exists.

**Tests/gates:** focused tests plus `scripts/check.sh`, `git diff --check`, exact-head CI.

**Commit/push:** yes if review/evidence or repair changes repository truth. Immediately continue to E; do not idle waiting for D019 policy.

### E. READY_LOCAL / VISIBLE OUTPUT — current-tree package/operator closure

**Goal:** produce or revalidate a user/operator-visible package lifecycle on the materially changed current tree, rather than spend the next cycle only on checkers/docs.

**Why now:** substantial runtime work landed after the older N5 package evidence. Package/operator evidence is a concrete deliverable and does not require D019 policy resolution.

**Files:** `scripts/release/build-package.sh`, `scripts/release/smoke-package.sh`, current release/package docs/evidence only as necessary.

**Protected invariant:** isolated/temp install target only; no destructive host migration, no production service replacement, no production route/firewall/DNS/proxy changes.

**Behavior:** first determine whether existing N5 evidence truly covers the current package contract. If current package/runtime changes are material, build exact current tree, verify manifest/binary hash/capabilities and bounded authenticated TCP/UDP smoke from an isolated install. If an A→B→A upgrade/rollback can be done with the existing isolated harness without production impact, include it and verify state permissions/cleanup. If exact current package behavior is already demonstrably unchanged and existing evidence is sufficient, record the exact-tree rationale and skip redundant execution rather than manufacturing activity.

**Tests/gates:** package build/smoke, cleanup, `scripts/check.sh`, exact-head CI.

**Commit/push:** commit new evidence only if a genuinely new run/closure is produced. Continue to F.

### F. READY_LOCAL — reconcile release/evidence matrix after B–E

**Goal:** update current status/plan only for evidence actually obtained and make the remaining gates explicit.

**Why now:** prevents local review/package work from being inflated into WAN/release conclusions.

**Files:** `docs/status.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, release-review packet only if its current-tree claims changed.

**Protected invariant:** local tests/package smoke ≠ WAN validation; bounded VPS evidence ≠ release evidence; HY2 and repeated-failover frozen negatives remain frozen; D019 remains a policy blocker.

**Behavior:** reconcile current labels and exact-tree provenance; no new checker/harness unless a concrete consistency gate is broken.

**Tests/gates:** `scripts/check.sh`, `git diff --check`, exact-head CI.

**Commit/push:** yes if current repository truth changes. Continue to G only if a real dependency-ready outward question exists.

### G. NEXT OUTPUT SELECTION — choose one truly unanswered dependency-ready runtime/operator/VPS question

**Goal:** keep the queue pointed at system output rather than infrastructure self-reproduction.

**Selection rule:** re-read current status/code/tests after F. Prefer a VPS-only question if there is a non-frozen, standing-authorized row whose answer cannot be reconstructed locally and a concrete hypothesis exists. Otherwise choose one bounded local runtime/wire/session/crypto question that is still genuinely open. Do not blindly execute stale 2026-08-30 ledger classifications; revalidate them against current tree truth first.

**Protected boundaries:** no unchanged HY2/repeated-failover rerun; no FEC/0-RTT/striping/multipath/exotic carrier merely because a roadmap row exists; no third-party target; no production network mutation; no new security numbers.

**Tests/gates:** define a bounded question, implement/test if needed, exact-head green before any WAN action, preserve cleanup/evidence class honestly.

**Commit/push:** yes for real implementation/evidence. Continue while READY work remains; do not stop at one nominal slice or reviewer interval.

## VPS / evidence priority

- Existing standing authorization continues to cover bounded self-owned client↔VPS TCP/UDP/Session/diagnostic/benchmark/capture/cleanup and the already-approved HY2 comparison shape.
- **HY2 same-class retry is frozen now** because the changed-instrumentation attempt has already produced `unknown/client_started`; another run without a new concrete hypothesis would be mechanical repetition.
- Repeated warm failover remains frozen under the same no-material-change rule.
- If a future material hypothesis is implemented and exact-head gates are green, do not invent a per-run authorization blocker; execute the bounded self-owned experiment within the standing limits.
- Never extend standing authorization to third-party scanning/targets, production route/firewall/DNS/proxy/tunnel/qdisc changes, destructive migration, or other explicitly excluded privileged/long/high-volume actions.

## Visible-output check

The last 24–48 hours are not checker-only: the repository gained endpoint-rebinding runtime + real VPS evidence, migration-back runtime + real VPS evidence, periodic/key-update VPS evidence, and now a materially changed HY2 real-VPS negative with typed diagnostics. Therefore the current small checker/security slices are justified as bounded cleanup, but after them the queue deliberately returns to package/operator/runtime output.

## Maintainer/admin boundary

Only the pre-existing D019 source-retention policy remains a known maintainer/security-value checkpoint in this queue. It blocks full D019/RSEC closure, but **does not block** B–G engineering/evidence work. No administrator action is required for the current ready slices.
