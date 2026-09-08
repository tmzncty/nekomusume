# Nekomusume ChatGPT Handoff

Checked at: 2026-09-08 22:00 Asia/Shanghai
Previous reviewer main: exact `799b65bb6a60b49811b5b2f2fa17a71ae828a387` (`docs(handoff): review preauth adversarial closure boundary`).
Previous checked coding HEAD: exact `d120e74b47df22959a23a39b031b10c7b360f3e1` (`test: close bounded preauth abuse evidence`).
Repository default `main` before this handoff: exact `256666676624578d72e5dc80fb8c3c13f41e521c` (`merge: integrate accepted closure lineage`).
Current execution branch `work/e1a-staged-accounting-20260907`: exact `f4404257520e9a014ac4e785b0ab9a97f8aaf794` (`merge: integrate reviewer closure lineage`).
The pre-handoff main and current work branch have the same tree (`4e3a3aec74d46295cbb303b70661d6560f5a102a`); the long-lived implementation/reviewer truth split is therefore closed. Do not create more coordination-only merge commits for tiny follow-ups.

New substantive coding commits reviewed this cycle:

- exact `b2fe1a30ccf6f20357a95c018db989836925c310` — `fix: close warm failover contract gaps`;
- exact `8a67be3b5ddfa0fdaf1374d1dd4de66e8cd7d55a` — `fix: redact preauth accounting diagnostics`.

Exact GitHub Actions:

- `b2fe1a3` Rust CI run `34230666694` — success;
- `8a67be3` Rust CI run `34231492179` — success;
- integrated default-main exact `2566666` Rust CI run `34232112189` — success.

## What changed and review verdict

`b2fe1a3` closes the queued **local-only** repeated-warm-failover contract cleanup: required command token/value failures and malformed/non-finite/out-of-range startup timeout are explicitly owned by `startup_setup`; the Draft 2020-12 regression again walks every retained repeated-warm-failover `result.json`; ROADMAP/status now describe the current repeated-failover line as an orchestration negative with same-class retry frozen absent a material new hypothesis. This does **not** authorize or justify another repeated-failover VPS run.

`8a67be3` closes the important non-policy RSEC repair from the previous review. `ProcessPreauthAdmission` no longer exposes its internal source/state keys through derived `Debug`; the custom `Debug` surface is aggregate-only and the adversarial test asserts the complete permitted shape. The resource-abuse note now names exact `d120e74` as the tested tree, distinguishes inherited from newly-added evidence, and correctly states that release/reopen demonstrates the unresolved D019 terminal source-retention conflict rather than satisfying it. The pending failover responder is also marked as a persisted pending owner in the inventory/checker.

**Verdict:** `ACCEPT_b2fe1a3_LOCAL_RWFDIAG_CLOSURE; ACCEPT_8a67be3_RSEC_PROVENANCE_AND_REDACTED_OBSERVABILITY_REPAIR_AS_BOUNDED_LOCAL_EVIDENCE; ACCEPT_INTEGRATED_MAIN_2566666_AS_THE_CURRENT_TRUTH; KEEP_D019_SOURCE_RETENTION_POLICY_BLOCKED; DO_NOT_GROW_MORE_REPEATED-FAILOVER_DIAGNOSTICS; ADVANCE_TO_THE_EXISTING_HY2_DIRECT-UNLOCK_SEAM_AND_AT_MOST_ONE_MATERIALLY_CHANGED_SELF-OWNED_FAIR-PAIR_ATTEMPT; KEEP_TWO_SMALL_STATIC/EVIDENCE-DEBT_REPAIRS_BEHIND_OR_PARALLEL_TO_THE_OUTWARD_LANE.`

There is no new Session/Carrier/ACK/crypto/wire semantic change, no destructive migration, no production action, and no new numeric security policy in the accepted commits. This is active progress, not `STALLED_IMPLEMENTATION`.

## Reviewer findings

### RSEC-PRIV/PROV — CLOSED for the bounded non-policy claim

The previous false assurance is repaired. The permitted `ProcessPreauthAdmission` debug representation now contains only aggregate counts: live states, source count, memory, queue depth, and window input/work/response totals. Source tuple bytes, state IDs, limits, timestamps and per-source keys are not rendered. The test asserts the exact aggregate format rather than merely searching for dotted-decimal text.

The developer note’s exact-tree provenance is also corrected to exact `d120e74`, and the source-reopen test is explicitly evidence of the D019 policy conflict, not a security-closure claim.

This supports bounded local engineering review only. It does not make RSEC-001 fully closed and does not authorize public listener promotion.

### RSEC-RETENTION — unchanged MAINTAINER POLICY CHECKPOINT

The current implementation removes a source entry when its final live admission state is released; a later admission from that source can therefore start fresh source-lifetime accounting. The repository’s unapproved amendment request already records the conflict with literal D019 and the competing policy shapes.

Do **not** invent a TTL, LRU/history size, epoch, capacity, or stable external accounting authority; do not silently weaken D019. Until an approved decision exists, represent the eventual review result as at most `ENGINEERING_EVIDENCE_CLOSED / SOURCE_RETENTION_POLICY_BLOCKED`.

This checkpoint blocks only full RSEC/D019 security closure. It does not block HY2, package/operator, release-evidence, or other independent engineering lanes.

### RSEC-INVENTORY-002 — MEDIUM / pending-owner checker is better but still too global

Exact `8a67be3` correctly marks `failover_udp_pending` as a persisted pending owner and requires reserve/store/cancel plus an expiry-cleanup anchor. However, the current checker locates several of those anchors with whole-file `text.find(...)` / whole-file membership checks. For a lifecycle whose producer (`enqueue` + `pending = Some(...)`) and consumer (`pending.take()` / dequeue / release / expiry) naturally live in different regions of the same large `main.rs`, this can false-pass if the relevant lifecycle changes while another identical string remains elsewhere.

This is a static-review precision gap, **not** evidence of a runtime leak and not a reason to hold the VPS/outward lane.

Minimum later repair: make the manifest identify the exact producer/lifecycle/consumer regions (or equivalently narrow anchors), then prove reserve -> persisted store in the producer and consume/cancel/expiry cleanup in the correct pending lifecycle. Keep it specific to this ownership invariant; do not build a generic static-analysis framework.

### RWFSCHEMA-LEGACY-001 — LOW/MEDIUM / preserve one frozen malformed historical digest without widening the contract

The corpus-wide schema regression exposes one historical retained result whose `diagnostic.sha256` value is not canonical 64-hex even though the field is named SHA-256. `b2fe1a3` preserves that immutable artifact with an exact-value schema exception rather than accepting arbitrary malformed digests. Other inspected diagnostic-bearing retained results carry normal full SHA-256 values.

Do not rewrite the historical artifact. Add only a compact regression/comment proving: the exact frozen legacy value remains accepted for backward compatibility, an arbitrary new non-64-hex value is rejected, and newly generated diagnostic records still require canonical 64-hex SHA-256. This is evidence-schema debt, not a reason for another WAN run.

### HY2-DIRECT-UNLOCK — READY and now the highest-value outward lane

The retained exact `3d54585` live artifact predates the current typed sanitized client-diagnostic seam. It contains a valid `nekomusume-1` success followed by `hy2-1` `client_exit`, 0 HY2 application bytes, no complete pair/summary, and therefore no performance conclusion.

Current exact-main code already has a materially stronger observation path:

- `run_client` captures the HY2 client log separately from the timed/raw application output;
- `validate-hy2-owned-lab.py` can emit bounded sanitized private diagnostics with `tls/auth/config/path/readiness/unknown` categories;
- prose can classify a subsystem but cannot promote lifecycle success;
- only strict typed monotonic stage evidence can advance `last_success_stage` beyond the harness baseline;
- current local tests exercise diagnostic redaction, bounds, private-bundle identity, category handling, no-prose stage promotion, and the first-pair blocked-result shape.

That instrumentation materially changes what one failed `hy2-1` attempt can tell us compared with exact `3d54585`. Therefore, after one focused exact-tree local confirmation, standing authorization permits **at most one** bounded self-owned fair-pair attempt with the existing equal workload/security/lifecycle/resource contract. Do not add capture or a generic diagnostic framework unless the existing current-main seam demonstrably cannot distinguish anything useful.

## Evidence / claim boundaries

- Integrated default main before this handoff is exact `2566666`; Rust CI run `34232112189` is green.
- Exact `b2fe1a3` and `8a67be3` are local implementation/test/docs evidence; neither adds new WAN evidence or performance conclusions.
- Repeated warm failover remains `BLOCKED_ORCHESTRATION_CURRENT_LINE`; exact `f17b648`/`9999622` already consumed the post-attribution WAN attempt. No same-class retry without a concrete material setup hypothesis.
- Endpoint rebinding exact `a8f49fc`/`7405da4`, migration-back exact `5d6582c`/`f024458`, and key update exact `2f4f59a`/`69d0ed9` retain only their previously accepted bounded claims.
- HY2 still has **no complete fair pair and no performance conclusion**. Existing/production Hysteria service must not be modified; the harness uses only temporary experimental HY2 under the standing authorization.
- IPv6 remains environment-blocked. Live PLPMTUD remains implementation/design blocked; do not invent wire fields or numeric policy to force it READY.
- Release/security flags remain `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`.

## Rolling Work Queue

The queue is continuous and dependency ordered. A green CI run, one commit, or a reviewer interval is not a stop condition while another dependency-satisfied slice remains.

### A — Confirm the existing HY2 direct-diagnostic seam on the integrated exact tree

**Status:** `READY_LOCAL`, outward/VPS-priority prerequisite.

Goal: prove the current production harness—not a synthetic replacement—still carries a nonzero HY2 client exit into the bounded typed diagnostic path.

Primary files: `scripts/bench/compare-hy2-owned-lab.sh`, `scripts/bench/compare-hy2-owned-lab-test.sh`, `scripts/bench/validate-hy2-owned-lab.py`.

Required work:

1. start from/fetch current reviewer `main`; avoid a handoff-only coordination merge;
2. run the existing focused HY2 owned-lab tests and the normal repository gate;
3. confirm the production `run_client hy2` path uses the HY2 client log as diagnostic input, preserves exact payload/resource/lifecycle symmetry, writes only bounded/redacted public metadata plus the protected private diagnostic bundle, and still suppresses comparative summary on an incomplete pair;
4. if a tiny bug prevents this existing seam from functioning, repair only that bug and its focused regression; otherwise make no instrumentation commit merely to manufacture activity;
5. require exact-head green CI before live execution.

No production Hysteria modification, no new credentials, no security weakening, no capture required by default.

**Continue immediately to B when green:** yes.

### B — One materially changed self-owned HY2 fair-pair attempt

**Status:** `PREAUTHORIZED_AFTER_A_GREEN`.

This is the current VPS opportunity. Standing authorization already covers the temporary self-owned Nekomusume-vs-HY2 run.

Use the existing contract: same owned client/VPS, same endpoint/route window, same 1200-byte exact payload contract, same security class, fresh transport lifecycle per timed sample, same resource sampler, current bounded run count/time limits, pinned temporary HY2 v2.9.3, and cleanup verification. Do not touch the existing production Hysteria service.

The run must preserve, not hide, any failure. If `hy2-1` fails again, retain the typed sample and sanitized diagnostic category/last-success boundary. That negative is useful even with zero complete pairs. If the pair succeeds, collect only the contract-defined paired samples/statistics; one bounded result is not a superiority claim.

Exactly one attempt for this changed instrumentation/hypothesis. A same-class failure after this run freezes the line again unless later code/config/path evidence creates a genuinely new hypothesis.

**Continue immediately to C:** yes.

### C — HY2 evidence/status reconciliation

**Status:** `READY_AFTER_B`.

If B is negative, state the narrowest trustworthy boundary: harness/setup, readiness, path, TLS/auth, client runtime, or still unknown. Do not infer stage success from stderr prose. Preserve cleanup truth separately from root-cause classification.

If B yields complete pairs, validate the artifact and report raw paired samples, failures, median/P95 and resources according to the existing schema. Do not write “faster/better” from one bounded experiment and do not let HY2 become a single release gate.

Update ROADMAP/status/implementation-plan only where the current truth actually changes. Commit/push and continue.

### D — Tighten the one pending-owner lifecycle static invariant

**Status:** `READY_LOCAL`, MEDIUM; may follow C or run while no live dependency is available.

Primary files: `docs/preauth-responder-inventory.v1.json`, `scripts/check-preauth-responder-inventory.py`, exact current failover pending code.

Goal: replace whole-file accidental-anchor matching with a small exact lifecycle proof for the persisted failover UDP pending owner.

Protected invariants: no runtime behavior change unless a real defect is discovered; no new generic audit framework; no numeric-policy change.

Test the checker against at least one negative fixture/mutation shape where an unrelated matching anchor elsewhere in the file cannot satisfy the pending owner’s producer/consumer/expiry contract.

Commit/push and continue.

### E — Make the frozen repeated-failover digest exception explicit

**Status:** `READY_LOCAL`, LOW/MEDIUM.

Keep the historical artifact immutable. Add a narrow schema/test comment or fixture proving the exact legacy digest exception is historical-only, arbitrary malformed new digests are rejected, and new generated diagnostics require canonical 64-hex SHA-256. Do not add a new schema version or migration unless actually necessary.

No WAN run follows this slice.

### F — Exact-tree partial RSEC engineering review

**Status:** `READY_LOCAL_AFTER_D`; independent of the D019 policy choice.

Review the **non-policy** RSEC pieces on the exact integrated tree: source projection, every real responder admission surface, charge-before-parse, exact response charging and rollback, source/global concurrent reservation, queue ownership, expiry/release/error cleanup, rate-window rejection, and redacted aggregate observability. Prefer existing deterministic/process tests; add only a test that closes a concrete uncovered invariant.

If these engineering pieces close, reconcile status/review to an explicit partial state such as `ENGINEERING_EVIDENCE_CLOSED / SOURCE_RETENTION_POLICY_BLOCKED`. Do not mark RSEC-001 fully closed, do not alter candidate numeric limits, and do not change release flags.

### G — Select the next real release-evidence/operator output lane

**Status:** `READY_AFTER_C_OR_F`.

Re-read the bounded release matrix after the HY2/RSEC reconciliation. Prefer a real package/operator/VPS evidence task that is dependency-ready and not already answered. Do not return to repeated-failover diagnostics, speculative FEC/0-RTT/multipath, or live PLPMTUD unless a new observed problem/design dependency actually makes them the next justified slice.

If no truthful VPS-only task remains READY, choose the earliest independent release-engineering task rather than manufacturing network activity.

## Maintainer/security-policy escalation boundary

No new maintainer decision was created this cycle. The one existing genuine policy checkpoint remains D019 terminal source retention / full RSEC-001 closure. The coding agent and reviewer must not choose retention TTL/LRU/history numbers, weaken no-reset semantics, or introduce a new stable accounting authority without an approved decision.

That checkpoint does **not** stop A-G. Standing VPS authorization remains valid for B after A is green, and no additional per-run permission is required.
