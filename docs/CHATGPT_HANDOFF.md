# Nekomusume ChatGPT Handoff

Checked at: 2026-09-08 21:00 Asia/Shanghai
Repository default `main` before this handoff: exact `910dbda7659dd35d15794706a0e91a9d840350da` (`docs(handoff): freeze repeated failover line and advance closure`).
Previous checked implementation/evidence HEAD: exact `99996226815a58a99270b16c1bd62cd1773e3bed`.
Current execution branch: `work/e1a-staged-accounting-20260907` at exact `d120e74b47df22959a23a39b031b10c7b360f3e1` (`test: close bounded preauth abuse evidence`), parent exact `9999622`.
New coding commit under review: exact `d120e74b47df22959a23a39b031b10c7b360f3e1`.
Exact GitHub Actions: main `910dbda` run `34224762205` — `success`; work exact `d120e74` run `34224960719` — `success`.
Current main/work topology is still diverged from merge base exact `1b5625d4f7c0c7063684690805fe5fda662a41c8`; the work branch is 8 commits ahead / 4 behind the current reviewer main. Do not create repeated coordination-only merges.

## What changed

Exact `d120e74` adds two deterministic `ListenerAdmission` adversarial tests and one developer evidence note. One test fills the existing same-source UDP pre-auth state ceiling, rejects the plus-one reservation, releases all tickets and observes zero live state/memory. The other exhausts the existing per-ticket input budget, verifies subsequent response/input rejection, releases the ticket and then proves the same source can be admitted again. Exact-head CI is green.

These tests are useful current-implementation evidence, but the commit does **not** close RSEC-001 and its new evidence note is not yet trustworthy as written. Review found one evidence-provenance error, one privacy-observability defect, and—most importantly—the new “reopens source” test directly demonstrates an already documented D019 policy conflict that cannot be solved by the coding agent inventing a retention TTL/LRU/history size.

The 24–48 hour visible-output check remains healthy: migration-back/recovery, authenticated endpoint rebinding, promotion-ordering protection and a truthful repeated-failover negative already exist. RSEC work is now a genuine release/security blocker, not audit-infrastructure filler, but it must be split into the parts that engineering can close and the source-retention policy checkpoint that requires maintainer judgment.

## Review verdict

**ACCEPT_EXACT_`d120e74`_TESTS_AS_BOUNDED_CURRENT_IMPLEMENTATION_EVIDENCE; REJECT_ITS_CURRENT_EXACT-TREE/REDACTION_CLOSURE_CLAIMS; MARK_D019_TERMINAL_SOURCE_RETENTION_AS_A_MAINTAINER_POLICY_CHECKPOINT; REQUIRE_ONE_BOUNDED_NON-POLICY_PRIVACY/PROVENANCE/INVENTORY_REPAIR; FINISH_THE_ALREADY-QUEUED_RWFDIAG_LOCAL_CLOSURE; INTEGRATE_ONCE; THEN RETURN_TO_AN_OUTWARD/HY2 LANE OR PARTIAL_RSEC_RECONCILIATION.**

This is active coding progress, not `STALLED_IMPLEMENTATION`. There is no new Session/Carrier/ACK/crypto/wire change, no destructive migration and no production action in exact `d120e74`. The source-retention subproblem, however, would require a security-policy/ADR decision and therefore is outside autonomous proposal authority. That subproblem must not stop independent work.

## Reviewer findings

### RSEC-PROV-001 — HIGH / new evidence note names the wrong exact tree

`docs/notes/resource-abuse-adversarial-9999622.md` says its exact tree is `99996226815a58a99270b16c1bd62cd1773e3bed`, but the two tests it cites (`adversarial_same_source_reservations_are_bounded_and_redacted` and `adversarial_input_budget_rejects_before_parse_and_reopens_source`) are introduced only by child commit exact `d120e74`. The note also records test counts from the post-change tree.

Minimum repair: make the note identify exact `d120e74` (or a later child containing the same code) as the tested tree and distinguish inherited evidence from tests newly introduced by this commit. Renaming the file is optional; content truth is mandatory. Do not rewrite historical WAN artifacts.

### RSEC-PRIV-001 — HIGH for security/privacy closure / current “redacted” test is false assurance

`source_key()` intentionally projects carrier + address family + raw IP octets + source port into a bounded binary key. That projection is acceptable as internal accounting state only under the current candidate design. However, `ProcessPreauthAdmission` and its per-state structs currently derive `Debug`, and the new test formats the entire process with `format!("{:?}", admission.process)` then checks only that dotted text `198.51.100.7` is absent.

Derived `Debug` exposes the source key as a byte vector, from which the exact IPv4/port tuple is recoverable. Therefore absence of dotted-decimal text does not prove redaction and conflicts with D018’s requirement that raw/unreviewed network identifiers not cross the diagnostic/log boundary.

Preferred minimal repair: either implement a custom redacted `Debug` for `ProcessPreauthAdmission`/reachable state that emits only aggregate bounded counters and never source/state keys, or remove that formatting surface from the proof and expose a narrow aggregate redacted snapshot API. Use the smaller fail-closed shape. The negative test should validate the exact permitted output shape, not merely search for one textual spelling of an address. No new numeric policy is needed.

This finding blocks claiming “secret-safe diagnostics” for RSEC closure. It does not invalidate bounded authenticated research experiments that do not emit this debug representation.

### RSEC-RETENTION-001 — MAINTAINER POLICY CHECKPOINT / RSEC-001 cannot close yet

The new test `adversarial_input_budget_rejects_before_parse_and_reopens_source` truthfully proves current behavior: after terminal release, the same source can be admitted again. That is consistent with current code because `ProcessPreauthAdmission::release` removes a source entry when its last live state is gone.

But D019 literally says relevant source accounting is not reset by retry/reconnect/carrier change/error and that cleanup must not reopen admission after the source/lifetime ceiling has been reached. The repository already contains `docs/adr/m1-g0-preauth-source-retention-amendment-request.md`, status `Review required — not approved`, explaining exactly this conflict and explicitly forbidding invention of a retention TTL, LRU/history bound or equivalent numeric policy.

Therefore exact `d120e74` is valuable evidence **of the unresolved policy conflict**, not evidence that RSEC-001 is closed. Keep the test, but document its boundary. The coding agent must not choose among bounded retention, a separate stable accounting authority, weakening D019 to live-state lifetime, or intentionally retaining this as a release blocker. That selection requires maintainer/security-policy judgment.

Project-wide work continues: only the D019 source-retention closure lane is policy-blocked.

### RSEC-INVENTORY-001 — MEDIUM / pending-owner static review coverage is weaker than the runtime ownership

Current runtime expires and invalidates a queued pending UDP negotiation before continuing, and the new branch has no demonstrated runtime leak. However, current `docs/preauth-responder-inventory.v1.json` marks `failover_udp_pending` as `pending_owner=false`, so the checker does not require reserve/store/cancel ownership anchors for that surface even though pending admission/queue state survives across loop iterations.

An older non-active branch contains a stronger pending-ownership anchor concept, but do **not** cherry-pick stale history wholesale. Re-read exact current `main.rs` and port only the invariant: the static inventory/checker must prove reservation before store plus expiry/cancellation cleanup for each path that owns persisted pending pre-auth state. Keep this small; it is independent review coverage, not a new runtime framework.

### RWFDIAG-005/006/007 — still open locally; no WAN retry

Exact `d120e74` does not touch the queued repeated-warm-failover cleanup. Missing required command token/value and malformed/non-finite/out-of-range startup timeout still need explicit `startup_setup`; corpus-wide Draft 2020-12 validation over all retained repeated-warm-failover `result.json` files still needs restoring; status/ROADMAP current-state wording still needs reconciliation.

The repeated-warm-failover WAN line remains frozen: exact `f17b648`/`9999622` already consumed the materially changed retry and repeated the same `startup_setup` category/hash/size. The local cleanup is **not** a new setup hypothesis and must not be followed by another same-class VPS attempt.

### RSEC-001 — remains HIGH for release/security/public-listener promotion, now explicitly split

Current deterministic evidence covers substantial implementation behavior: source tuple projection, existing source/global state limits, queue/max-plus-one behavior, response permits, expiry/release tests and several charge-before-parse call sites. Exact `d120e74` adds useful adversarial coverage. But closure must now be represented as:

- **engineering-reviewable/non-policy part:** redacted observability, exact responder coverage, charge ordering, atomic rollback, global/source rate windows, expiry/release/error cleanup;
- **policy-blocked part:** terminal source-accounting semantics after the last live state disappears.

Do not mark RSEC-001 fully closed until the policy checkpoint is resolved by an approved ADR/security decision. Do not change candidate numeric limits merely to satisfy tests.

## Evidence / claim boundaries

- Default main before this handoff is exact `910dbda`; exact-main Rust CI `34224762205` is green.
- Current work exact `d120e74` has exact-head Rust CI `34224960719` green.
- Exact `d120e74` is local deterministic implementation/test evidence only; it contains no new VPS/WAN observation and no performance/reachability conclusion.
- The new resource-abuse note’s current exact-tree wording is wrong until repaired; do not cite it as exact-`9999622` security evidence.
- Current release/security flags remain `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`.
- Exact `f17b648`/`9999622` repeated-warm-failover negative remains frozen against unchanged retry.
- Endpoint rebinding exact `a8f49fc`/`7405da4`, migration-back exact `5d6582c`/`f024458`, and key-update exact `2f4f59a`/`69d0ed9` retain only their previously accepted bounded claims.
- HY2 still has no complete fair pair or performance conclusion. Existing production Hysteria service must not be modified.
- IPv6 remains environment-blocked. Live PLPMTUD remains implementation/design blocked; do not invent wire fields or numeric policy.
- Protected identities, credentials, private endpoints and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This queue is continuous and dependency ordered. A commit, reviewer interval or green CI is not a stop condition while another safe dependency-satisfied slice remains.

### A — Repair the new RSEC evidence truth/privacy surface

**Status:** `READY_LOCAL`, HIGH. Proposal authority applies except to source-retention policy.

Goal: keep the useful exact-`d120e74` adversarial tests while removing false security/provenance claims.

Primary files: `crates/neko-crypto/src/lib.rs` and/or `crates/neko-cli/src/preauth.rs`, `docs/notes/resource-abuse-adversarial-9999622.md`, `docs/preauth-responder-inventory.v1.json`, `scripts/check-preauth-responder-inventory.py`.

Protected invariants: no D019 numeric change; no Session/Carrier/ACK/crypto/wire semantic change; source key stays an internal bounded accounting projection; no raw endpoint appears in debug/log evidence; no historical artifact rewrite.

Required behavior/tests:

1. correct the developer note’s exact tested tree/provenance;
2. replace the weak dotted-IP Debug assertion with a redacted aggregate output whose permitted fields are explicit and source keys cannot be reconstructed;
3. keep the reopen-source test but state that it demonstrates the unresolved D019 terminal-retention policy conflict rather than closure;
4. re-read exact current failover pending ownership and strengthen the inventory/checker only enough to prove reserve -> persist -> expiry/cancel cleanup ordering.

Commit and push. Do **not** invent TTL/LRU/history limits or weaken D019 in code/docs.

**Continue immediately to B:** yes.

### B — Finish the already-authorized repeated-failover local contract cleanup

**Status:** `READY_LOCAL`, MEDIUM.

Primary files: `scripts/bench/run-live-warm-failover-cycle.py`, its focused tests, repeated runner schema tests.

Required behavior/tests:

1. missing required token/value -> explicit `startup_setup`;
2. malformed/non-finite/out-of-range server-start timeout -> explicit `startup_setup`;
3. restore Draft 2020-12 validation across every retained repeated-warm-failover `result.json` plus the generated fine-category negative;
4. preserve `evidence_serialization` ownership for malformed/identity/cardinality evidence and preserve earlier-primary-over-cleanup regression.

This is local-only. **No repeated-failover VPS rerun.**

Commit/push and continue to C.

### C — Full gate + current-state reconciliation

**Status:** `PREAUTHORIZED_AFTER_A_B`.

Run focused tests, `./scripts/check.sh`, `git diff --check`; fuzz only if parser/wire decode code changes. Require green exact-head CI.

Then make one compact developer-owned status/review reconciliation:

- repeated warm failover = current-line orchestration negative, same-class retry frozen absent material new hypothesis;
- remove stale “next seam is inner categorization” / obsolete `BLOCKED_DIAGNOSTICS` wording where it describes the current repeated-failover line;
- RSEC current truth = substantial local implementation/adversarial evidence, but privacy/provenance fixes are exact-tree facts and terminal source retention is `POLICY_CHECKPOINT` / not closed;
- resource-abuse review must not call the source-retention requirement satisfied merely because release/reopen behavior is tested.

Do not set RC/freeze/production/release flags.

**Continue immediately to D:** yes.

### D — Integrate the accepted closure lineage to default main once

**Status:** `PREAUTHORIZED_AFTER_C_GREEN`.

Merge current reviewer `main` into the execution branch while preserving reviewer ownership/content of `docs/CHATGPT_HANDOFF.md`, resolve only real conflicts, run the normal gate, then integrate the accepted implementation/evidence lineage to default `main` without history rewriting. Observe exact-main CI.

This should end the long-lived main/work truth split. Do not create additional coordination-only merges for tiny follow-ups.

**Continue immediately to E after green:** yes.

### E — HY2 direct-unlock proposal or explicit skip

**Status:** `READY_AFTER_D`; outward/VPS-priority lane.

Re-read the latest exact-tree HY2 fair-pair harness/artifact around `hy2-1 client_exit`. Propose 1–3 minimal shapes and choose the smallest security-neutral observation that can distinguish harness/setup from actual HY2 client/runtime failure while preserving application/security/lifecycle/resource symmetry.

Allowed: bounded typed client-exit/stderr classification or equivalent existing-harness observation. Forbidden: generic benchmark diagnostic framework, workload/security asymmetry, production Hysteria changes, new credentials/permissions.

If no small seam truthfully changes the hypothesis, record/retain the blocker and skip F; continue to G. Do not manufacture work.

### F — At most one materially changed self-owned HY2 fair-pair attempt

**Status:** `PREAUTHORIZED_ONLY_IF_E_MATERIALLY_CHANGES_THE_HYPOTHESIS_AND_EXACT_HEAD_IS_GREEN`.

Standing authorization already covers one bounded self-owned Nekomusume-vs-HY2 attempt. Use the existing equal workload/security/lifecycle/resource contract, temporary experimental HY2 only, cleanup required. Preserve a failure or slower Nekomusume result exactly. No superiority claim from one sample or incomplete pair.

Same-class failure without a new hypothesis freezes this line; no mechanical retry.

**Continue immediately to G:** yes.

### G — Exact-tree partial RSEC reconciliation and next dependency-satisfied output lane

**Status:** `READY_LOCAL_AFTER_D`; can run if E is skipped or after F.

Perform the independent exact-tree review of the **non-policy** RSEC pieces only: source projection, charge-before-parse and exact-response charging, every real responder integration, concurrent source/global reservations and rollback, rate-window rejection, expiry/release/error cleanup, and redacted aggregate observability. Prefer existing deterministic/process tests; add only tests that close a concrete uncovered invariant.

If those engineering pieces close, update the resource-abuse review/status to an explicit partial state such as `ENGINEERING_EVIDENCE_CLOSED / SOURCE_RETENTION_POLICY_BLOCKED`; do **not** mark RSEC-001 fully closed and do not change release flags. Re-read the release matrix and select the next real dependency-satisfied output/VPS lane rather than returning to checker growth.

## Maintainer/security-policy escalation boundary

One maintainer decision is now genuinely required **only to close D019 terminal source retention / full RSEC-001**. The existing amendment request already enumerates the policy shapes; reviewer/coding agent must not choose a retention TTL/LRU/history size, weaken the no-reset requirement, or introduce a stable external accounting authority without an approved decision. Until then, leave that sublane policy-blocked and continue A-G wherever independent.

No maintainer action is required for the other queued slices. Standing VPS authorization remains valid for F if and only if E produces a material, truthful changed hypothesis.
