# Nekomusume ChatGPT Handoff

Checked at: 2026-09-09 00:00 Asia/Shanghai
Previous reviewer main: exact `180f5fe056348e15cd917eeefa1e1cc1a8eb00bc` (`docs(handoff): accept closure integration and advance HY2 evidence`).
Repository default `main` at review start: exact `6a69ab881e233ee71e2efb4fb79a00e72593d537` (`merge: preserve reviewer handoff during RSEC reconciliation`).
The old execution branch `work/e1a-staged-accounting-20260907` remains at exact `f4404257520e9a014ac4e785b0ab9a97f8aaf794` and is now stale relative to main; do not merge it back merely for coordination. Start/fetch current `main` for new work.

New commits since the previous reviewer handoff:

- exact `0b9acbabf5ea13ad6a481f7ac20ad716873d4ed1` — `docs: reconcile partial preauth security evidence`;
- exact `6a69ab881e233ee71e2efb4fb79a00e72593d537` — merge preserving this reviewer-owned handoff while integrating `0b9acba`.

There is **no new implementation/test/WAN evidence commit** in this interval. The compare from exact `180f5fe` to exact `6a69ab8` changes only `docs/reviews/resource-abuse-evidence-2026-09-04.md` and `docs/status.md`.

Exact GitHub Actions for current main:

- Rust CI run `34243187885` — `success`;
- `stable checks` job — `success`, including `bash scripts/check.sh`;
- `nightly decode fuzz smoke` — `success`.

Because `scripts/check.sh` includes `scripts/bench/compare-hy2-owned-lab-test.sh` and `scripts/bench/validate-hy2-owned-lab-test.py`, the previously queued HY2 local prerequisite is already satisfied on exact current main. Do **not** manufacture another local-only commit just to re-prove that gate.

## Review verdict

**ACCEPT_CURRENT_MAIN_`6a69ab8`_AS_REPOSITORY_TRUTH_AND_ACCEPT_ITS_GREEN_GATE; REJECT_`0b9acba`_`ENGINEERING_EVIDENCE_CLOSED`_LABEL_AS_PREMATURE/INTERNALLY_CONTRADICTORY; KEEP_D019_SOURCE_RETENTION_POLICY_BLOCKED; MARK_HY2_DIRECT-UNLOCK_AS_`STALLED_IMPLEMENTATION`_COORDINATION_SIGNAL_BECAUSE_THE_READY_OUTWARD_SLICE_HAS_REMAINED_UNEXECUTED_WITHOUT_AN_EXTERNAL_BLOCKER; REPAIR_THE_RSEC_CLAIM_DRIFT_AS_A_SMALL_FIRST_SLICE_AND_THEN_EXECUTE_EXACTLY_ONE_PREAUTHORIZED_SELF-OWNED_HY2_ATTEMPT_FROM_CURRENT_GREEN_MAIN; DO_NOT_GROW_MORE_DIAGNOSTIC_FRAMEWORK.`

There is no new Session/Carrier/ACK/crypto/wire semantic change, no destructive migration, no production action, and no new numeric security policy in the new commits.

## Reviewer findings

### RSEC-CLAIM-003 — HIGH evidence/provenance drift in exact `0b9acba`

The new reconciliation header says `ENGINEERING_EVIDENCE_CLOSED` for the non-policy pre-auth controls on the integrated `2566666` / `8a67be3` lineage. That label is not yet supported by the review document beneath it.

The same file still says:

- `RSEC-001 — Pre-auth admission is integrated; independent review/load evidence remains absent`;
- the required evidence to close the promotion finding includes independent review of source projection, charge ordering, response accounting and all listener coverage plus bounded adversarial concurrency/rate/expiry evidence;
- the evidence matrix still says RSEC-001 HIGH remains open for independent review/adversarial evidence;
- the document still names exact `bb9e268...` as its `Reviewed tree` and says `scripts/check.sh` passed on exact `bb9e268`, not on the newly claimed reconciliation tree.

Current main CI proves current-tree automated gates are green, but it is not an independent security review and does not retroactively change the review document's exact-tree provenance. The earlier reviewer queue also still had the pending-owner static-review precision repair before the exact-tree partial RSEC review.

Minimum correction: keep the useful implementation evidence, but change the current state to something truth-preserving such as:

`ENGINEERING_CONTROLS_PRESENT / INDEPENDENT_REVIEW_OPEN / SOURCE_RETENTION_POLICY_BLOCKED`

or equivalent wording that does **not** say the engineering evidence is closed. Update the top reconciliation, `docs/status.md`, and any directly conflicting sentence together. Do not rewrite historical evidence or change release flags.

This is a documentation/evidence correctness HIGH, not a newly discovered runtime vulnerability. Close it quickly and continue to the outward lane; do not let it expand into another review framework.

### RSEC-RETENTION — unchanged maintainer/security-policy checkpoint

D019 terminal source-accounting retention after the last live state disappears remains unresolved. Current code can release the final source state and later admit the same source with fresh source-lifetime accounting. The retained amendment request already records the policy alternatives and forbids the agent/reviewer from inventing a retention TTL, LRU/history size, epoch capacity or silently weakening D019.

This blocks full RSEC/D019 closure only. It does **not** block HY2, bounded self-owned VPS work, package/operator evidence, or other independent engineering lanes.

### HY2-GATE-001 — local prerequisite is already GREEN on exact current main

The previous handoff asked for one focused confirmation that the production HY2 harness carries a nonzero HY2 client exit into the bounded typed diagnostic path before spending another VPS attempt.

Current exact `6a69ab8` already satisfies that gate:

- `scripts/check.sh` invokes `compare-hy2-owned-lab-test.sh` and `validate-hy2-owned-lab-test.py`;
- exact-main CI run `34243187885` passed `scripts/check.sh`;
- production `run_client hy2` captures the HY2 client process log separately as `hy2-client-$run_no.log` and passes that file into `make-sample` as `--client-diagnostics`;
- the validator produces bounded/redacted private diagnostics with categories `tls/auth/config/path/readiness/unknown`;
- prose can classify a subsystem but cannot promote lifecycle success; only strict typed monotonic stage evidence can advance `last_success_stage`;
- incomplete first-pair results retain the valid prefix and suppress comparative summary.

One boundary must stay explicit: the production `run_client` path currently does **not** pass a `--diagnostic-stage-evidence` file, so a live HY2 failure will normally retain `last_success_stage=client_started` unless a future materially different harness supplies strict typed stage evidence. A category such as `tls` or `path` is therefore a sanitized diagnostic classification, **not proof that all earlier lifecycle stages succeeded**.

No additional instrumentation is required merely to make the next bounded attempt useful. If the next run still yields `unknown`, preserve that negative honestly and freeze the line again; do not add generic capture/debug infrastructure unless a later concrete question justifies it.

### HY2-STATUS-DRIFT-002 — MEDIUM, reconcile with the next evidence commit

`docs/status.md` still describes the HY2 current line using exact `61a6490` local port-range preflight and `BLOCKED_ORCHESTRATION_CURRENT_LINE_HY2`, while `ROADMAP.md` / `IMPLEMENTATION_PLAN.md` also retain later exact `3d54585` facts: valid `nekomusume-1` success followed by `hy2-1 client_exit`, no complete pair and no comparison.

Do not spend a standalone docs-only commit on this before the live attempt. Reconcile the authoritative status, ROADMAP and implementation plan in the same closure commit after the one changed-hypothesis run, preserving all historical negatives.

### HY2-DIRECT-UNLOCK — `STALLED_IMPLEMENTATION` coordination signal, not a maintainer blocker

The HY2 local prerequisite has remained at the front of the rolling queue across consecutive reviewer opportunities, current exact-main CI is green, standing authorization is explicit, and there is no repository/external blocker. The intervening work was an out-of-order RSEC documentation reconciliation rather than progress on the READY outward slice.

Treat this as `STALLED_IMPLEMENTATION` for coordination purposes: re-read the current harness, use the already-green exact tree, and execute the already-authorized next step. Do **not** respond by adding another design note, another diagnostic layer, or another coordination merge.

The agent may choose the smallest execution shape itself. No API proposal is needed unless the current harness demonstrably cannot run.

### RSEC-INVENTORY-002 — MEDIUM, still open behind the outward lane

The persisted `failover_udp_pending` owner is now represented in the inventory, but several checker anchors are still whole-file `find`/membership matches. An unrelated identical string elsewhere in `main.rs` could satisfy the static proof after the relevant lifecycle regressed.

Later minimum repair: narrow producer/store/consumer/expiry ownership to the actual pending lifecycle region and add one negative mutation/fixture showing an unrelated matching anchor cannot satisfy the owner contract. Do not create a generic static-analysis framework.

### RWFSCHEMA-LEGACY-001 — LOW/MEDIUM, still local-only

Keep the one frozen historical malformed digest accepted by exact-value compatibility only; add a compact regression proving that arbitrary new non-64-hex digests remain rejected and newly generated diagnostics still require canonical 64-hex SHA-256. Do not rewrite the historical artifact and do not run WAN afterward.

## Evidence / claim boundaries

- Current default main is exact `6a69ab881e233ee71e2efb4fb79a00e72593d537`; exact-main Rust CI `34243187885` is green.
- New exact `0b9acba` / `6a69ab8` are documentation/merge commits only. They add no runtime, test, WAN, security-audit or performance evidence.
- Repeated warm failover remains frozen against same-class retry absent a material setup hypothesis.
- Endpoint rebinding exact `a8f49fc`/`7405da4`, migration-back exact `5d6582c`/`f024458`, and key-update exact `2f4f59a`/`69d0ed9` retain only their previously accepted bounded claims.
- HY2 still has no complete fair pair and no performance conclusion. Exact `3d54585` is the latest retained live prefix known to the current plan/ROADMAP before the next attempt: `nekomusume-1` success, `hy2-1 client_exit`, zero complete pairs/comparative summary.
- Existing/production Hysteria service must not be modified. Only the temporary pinned HY2 v2.9.3 experiment is in scope.
- IPv6 remains environment-blocked. Live PLPMTUD remains design/implementation blocked; do not invent wire fields or numeric policy to force it READY.
- Release/security flags remain `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`.

## Rolling Work Queue

The queue is continuous and dependency ordered. A green CI run, one commit, or a reviewer interval is not a stop condition while dependency-satisfied work remains.

### A — Correct the premature RSEC closure label without expanding the review surface

**Status:** `READY_LOCAL`, HIGH evidence correctness; intentionally small.

Goal: remove the exact `0b9acba` internal contradiction while preserving all useful pre-auth implementation/test evidence and the D019 policy checkpoint.

Primary files: `docs/reviews/resource-abuse-evidence-2026-09-04.md`, `docs/status.md`.

Required behavior:

1. do not claim `ENGINEERING_EVIDENCE_CLOSED` while independent exact-tree review is still open and the review document itself says it is absent;
2. preserve the distinction between implementation evidence, independent review, numeric-policy suitability and D019 retention policy;
3. do not change candidate numeric limits or release flags;
4. no new checker/harness framework and no historical artifact rewrite.

Run normal local gate, commit/push, then continue immediately to B. This slice should be one bounded truth-reconciliation commit, not a new review project.

### B — Execute exactly one materially changed self-owned HY2 fair-pair attempt

**Status:** `PREAUTHORIZED_NOW`; the local gate is already green on exact `6a69ab8`.

This is the highest-value current VPS opportunity. Standing authorization covers it; no per-run maintainer permission is needed.

Use the existing contract without changing production services:

- self-owned client and VPS only;
- temporary experimental listener/processes only;
- pinned HY2 v2.9.3 artifact;
- same 1200-byte exact payload contract and current bounded run count/time budget;
- same security class, fresh client transport lifecycle per sample, same resource sampler and cleanup verification;
- same route/time-window fairness as far as the existing harness controls it;
- do not touch the existing production Hysteria service;
- exactly one live attempt for this changed diagnostic hypothesis.

If `hy2-1` fails, preserve the typed row, diagnostic category, baseline/typed `last_success_stage`, resource/cleanup truth and valid Nekomusume prefix. Do not infer successful TLS/auth/path stages from prose category alone.

If complete pairs exist, preserve raw paired samples and contract-defined median/P95/failure/resource output. One bounded run is not a superiority claim.

After any same-class failure, freeze the line again unless a later material code/config/path/instrumentation change creates a genuinely new hypothesis.

**Continue immediately to C:** yes.

### C — Reconcile HY2 evidence and stale status/plan truth

**Status:** `READY_AFTER_B`.

Primary files: new/retained HY2 artifact, `docs/status.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`.

Requirements:

- retain old `61a6490`, `3d54585` and any earlier negatives as history;
- make authoritative current status name the newest exact attempt and its narrow evidence boundary;
- remove contradictory simultaneous `BLOCKED_DIAGNOSTICS` / older-current-line wording where it no longer describes the latest attempt;
- incomplete pair => no summary/performance claim;
- complete pairs => raw samples/median/P95/failures/resources only, no “faster/better” conclusion from one bounded run;
- cleanup/postcheck facts remain separate from root-cause attribution.

Commit/push and continue.

### D — Tighten the one persisted pending-owner static invariant

**Status:** `READY_LOCAL`, MEDIUM; after C unless B is temporarily unavailable for a real environment reason.

Primary files: `docs/preauth-responder-inventory.v1.json`, `scripts/check-preauth-responder-inventory.py`, current failover pending lifecycle in `crates/neko-cli/src/main.rs`.

Goal: make the checker prove reserve -> persisted store -> consume/cancel/expiry cleanup for the actual `failover_udp_pending` owner, not merely find identical strings somewhere in the file.

Minimum test: a negative fixture/mutation in which an unrelated matching anchor elsewhere exists but the target lifecycle anchor is missing must fail.

No runtime behavior change unless the tighter proof discovers a real defect; no generic static-analysis framework.

### E — Make the frozen repeated-failover digest exception explicit

**Status:** `READY_LOCAL`, LOW/MEDIUM.

Keep the historical artifact immutable. Add one narrow regression proving the exact legacy digest remains accepted, an arbitrary malformed new digest is rejected, and generated diagnostics require canonical 64-hex SHA-256.

No WAN run follows this slice.

### F — Perform the actual exact-tree partial RSEC independent review

**Status:** `READY_LOCAL_AFTER_D`; D019 policy remains independently blocked.

Review the current exact integrated tree, not the old `bb9e268` review tree. Check source projection, every real responder admission surface, charge-before-parse, exact response charging/rollback, source/global reservations, persisted queue ownership, expiry/release/error cleanup, rate-window rejection and aggregate redacted observability.

Use existing deterministic/process/adversarial tests first. Add only a concrete missing negative test; do not create another audit framework.

Only after this review may the repository use an engineering-review state stronger than `INDEPENDENT_REVIEW_OPEN`. Even then, D019 source retention remains `SOURCE_RETENTION_POLICY_BLOCKED` until a maintainer/security-policy decision exists. Do not mark RSEC-001 fully closed or change release flags.

### G — Select the next real VPS/operator/release-evidence output lane

**Status:** `READY_AFTER_C_OR_F`.

Re-read the release evidence matrix after HY2/RSEC reconciliation. Prefer one dependency-ready result that is difficult to reconstruct after the VPS rental ends: package install/upgrade/rollback/operator lifecycle, a genuinely unanswered bounded real-socket row, or another directly release-relevant VPS observation.

Do not return to unchanged repeated-failover experiments, speculative FEC/0-RTT/multipath, or live PLPMTUD without a newly observed problem/design dependency.

If no truthful VPS-only task remains READY, choose the earliest independent release-engineering/operator task instead of manufacturing network activity.

## Maintainer/security-policy escalation boundary

The one genuine current maintainer checkpoint remains D019 terminal source-retention semantics / full RSEC-001 closure. The reviewer and coding agent must not choose a retention TTL, LRU/history size, capacity, external stable-accounting authority, or weakening of no-reset semantics without an approved decision.

That checkpoint does **not** stop A-G. The HY2 attempt in B is already within standing authorization and needs no additional permission.
