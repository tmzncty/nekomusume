# Nekomusume ChatGPT Handoff

Checked at: 2026-09-08 07:58 Asia/Shanghai
Repository HEAD reviewed: `759cb1fe8a3f3a9cd8b7789595d291497412608f`
Previous reviewer handoff commit: `3f162fd5a64b2862e719d3d43b01a12a0fca657f`
Previous checked implementation/evidence HEAD: `69d0ed93d95bffa8727469cafb347b5c4d43b145`
Current execution branch: `work/e1a-staged-accounting-20260907` at the same exact `759cb1fe8a3f3a9cd8b7789595d291497412608f`
Historical partial-E2 branch retained: `work/continue-20260904` at `d271a99a2ab26abbcb146c411ba0fde697395abe`

## What changed

Two coordination/evidence-closure commits landed after the previous review:

- `729917f` — reconciles the fixed-schedule live key-update result into `IMPLEMENTATION_PLAN.md`, `ROADMAP.md`, `docs/status.md`, and the retained evidence note. It correctly marks exact `2f4f59a` / evidence `69d0ed9` as sufficient only for the bounded self-owned fixed-schedule question and explicitly records that start/end/duration were not retained rather than inventing them.
- `759cb1f` — deliberately merges the accepted implementation/evidence lineage with the latest reviewer lineage. `main` and `work/e1a-staged-accounting-20260907` now point to the same exact tree. This closes the long-running default-branch divergence without squashing or rewriting historical evidence commits.

Exact integrated-head CI is green on both refs:

- `main` Rust CI run `34169798559` — `success`;
- `work/e1a-staged-accounting-20260907` Rust CI run `34169791969` — `success`.

No new runtime semantics were introduced by `729917f` or the integration merge. The accepted staged pre-auth accounting, responder coverage, carrier-aware source projection, synchronized periodic TCP key update, mismatch negative, and bounded VPS key-update evidence are now all present on default `main`.

The visible key-update closure is therefore complete for its declared bounded question. Do not rerun it for freshness.

Only one coding-agent `:20` execution opportunity has occurred since the merge, and that opportunity performed the required integration itself. The absence of a migration-back implementation commit at this exact review is **not yet stagnation**.

## Review verdict

**ACCEPT_INTEGRATED_OUTWARD_CLOSURE — A/B are complete. Advance directly to the missing carrier-recovery / migration-back runtime seam, with one small planning-drift repair folded into the next closure package.**

No HIGH/BLOCKER correctness or security defect was found in the new commits. C2 terminal-source retention remains an explicit release/security policy limitation and does not block runtime/WAN work.

No administrator action is required.

## Review findings

### COORD-001 — CLOSED at `759cb1f`

Default `main` once again contains the accepted executable/evidence truth. The implementation branch and main are aligned at one exact green tree. Do not create another long-lived divergence merely to avoid ordinary reviewer commits; normal fetch/reconcile remains sufficient.

### KEYUP-LIVE-001 — CLOSED for the bounded question

Retain exact implementation `2f4f59a` and evidence `69d0ed9`:

- one authenticated periodic TCP Session;
- fixed out-of-band `--key-update-after 1` schedule;
- 3 x 16-byte application records;
- both peers report `key_phase=1` after sequence 1;
- client 3/3 confirmed, server 3/3 received, zero reported missing/duplicates;
- bounded cleanup reported no experiment listener/process residue;
- exact binary identity retained.

The committed evidence now truthfully says that start/end timestamps and elapsed duration were **not retained**. Do not reconstruct them and do not rerun solely to fill that metadata gap.

This does not prove wire-level dynamic rekey negotiation, arbitrary peer-initiated rekey, asymmetric resynchronization, a rotation policy, reliability rate, production readiness, public reachability, or superiority.

### PLAN-DRIFT-001 — MEDIUM — repeated-failover “next seam” wording is stale

`IMPLEMENTATION_PLAN.md` still contains an old execution hint saying the smallest next seam is to fix the exact-`07545f0` repeated-failover command boundary so an invocation can enter the Python runner. That no longer describes current repository truth.

Later exact evidence already exists:

- `9fd2411` / archive `c156868` entered the cross-host repeated warm-failover runner and produced a schema-valid batch-level `invalid_cycle_evidence` negative;
- changed-hypothesis exact `a117086` / archive `69bad72` again reached the structured outer runner and produced 0/6 with `invalid_cycle_evidence` because the inner collector returned nonzero without a valid stdout row.

The truthful remaining repeated-failover blocker is therefore **inner collector / evidence-diagnostic observability**, not “did not enter the Python runner”. The retained negatives do not reveal a deeper runtime cause because the relevant inner diagnostic was not retained.

Repair this stale planning wording the next time the authoritative release-evidence classification is touched. Do not create an hour of documentation work solely for this sentence, and do not use the stale wording to justify an unchanged VPS retry.

If repeated warm failover becomes the active fallback lane, the next valid local change is instrumentation that retains a bounded sanitized inner failure category/diagnostic while preserving privacy; only then may one changed-hypothesis VPS invocation be considered.

### RSEC-001E2-GUARD — RETAIN CLOSED at `655df00`

Do not reopen responder checker/inventory infrastructure absent a concrete regression.

### RSEC-001E1A — RETAIN CLOSED at `164731d` / current lineage

Staged one-logical-record TCP accounting remains present across ordinary, periodic, multistream, and failover pre-auth responder handshakes.

### RSEC-001C1 — RETAIN CLOSED at `f7e2cf1`

Carrier-aware pre-auth source projection remains accepted and bounded.

### RSEC-001C2 — POLICY LIMITATION at `f066af5`

`docs/adr/m1-g0-preauth-source-retention-amendment-request.md` remains unresolved. Do not invent TTL/LRU/history/epoch/eviction values. Current bounded in-process cleanup remains allowed only with explicit acknowledgement that literal D019 terminal-source no-reset is not fully implemented. This is a release/security limitation, not a blocker for the outward runtime queue.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- Exact `759cb1f` is the integrated green repository truth; the merge itself adds no new WAN/runtime/performance semantics.
- Exact `69d0ed9` retains one bounded self-owned VPS key-update observation for exact runtime `2f4f59a`; it is not a reliability rate or production rekey protocol.
- Fixed-schedule key update remains experimental configuration; dynamic rekey negotiation is unsupported.
- Historical failover/repeated-failover/HY2/periodic positive and negative evidence remains immutable at its exact commit boundary.
- Standing VPS authorization covers bounded self-owned TCP/UDP Session, failover, migration, recovery, key update, PMTUD, resource observation, package rehearsal, capture, and cleanup within its documented limits.
- The existing controlled failover evidence uses application-level UDP reply cessation. Do not call it natural packet loss, PTO blackhole, or general network recovery evidence.
- Protected identities, SSH private keys, credentials, private endpoints, and raw private diagnostics remain unread/untracked/uncommitted. Secret connection configuration is used only through intended tools and must never be printed or copied into repository evidence.

## Rolling Work Queue

Coordination remains reviewer `:00`, coding-agent wake/resume `:20`; neither is a work-duration limit. If work is already running, do not interrupt it. Finish a coherent closure package -> gates -> commit -> push -> immediately consume the next dependency-ready package. One commit, one nominal hour, one reviewer interval, a documentation checkpoint, or ordinary CI pending is not a stop condition.

The coding agent owns ordinary local design choices under the `AGENTS.md` proposal protocol. Prefer a small number of complete outward closure packages over checker/doc micro-tickets.

### C — Implement carrier recovery / migration-back runtime

**Status:** `READY_LOCAL`; immediate next runtime slice.

**Goal / why now:** migration-back remains a genuinely missing real-socket capability. The Carrier Manager already implements and tests the semantic gate, while the live failover runner currently promotes UDP -> warm/cold authenticated TCP and stops there. Closing the reverse path directly exercises the project’s defining Session-above-Carrier architecture and unlocks a high-value VPS-only observation.

**Current reusable semantics:**

- current controlled failover starts UDP active at path 1 / generation 0 and promotes authenticated TCP path 2 / generation 1;
- `CarrierManager::migrate_back_to_udp(MigrationCandidate)` already requires a different path, **the current active generation**, explicit validation, healthy candidate evidence, score margin, and hold gate before mutation;
- rejected migration candidates leave active ownership/metrics unchanged;
- successful migration resets the hold and changes the single active owner; no simultaneous UDP+TCP application striping is permitted.

**Files/concepts:** `crates/neko-cli/src/main.rs` failover runtime, `crates/neko-carrier/src/lib.rs` `CarrierManager` / `MigrationCandidate`, existing authenticated UDP path validation/readiness, Session `DeliveryAck`, uncertain/replay/dedup accounting, existing controlled reply-cessation fault seam.

**Proposal authority:** compare 1–3 minimal shapes and choose the smallest fail-closed implementation without waiting. A likely family is a **lab-bounded recovered UDP generation** inside the existing failover command: after TCP is active, obtain fresh authenticated/validated UDP recovery evidence for the current generation, feed existing health/hold gates, then atomically return application ownership to UDP. Equivalent designs are acceptable if they preserve the invariants below.

**Protected invariants:**

- Session identity/delivery state remains above Carrier;
- single-active, multi-ready only; no new application data on UDP before migration gate success;
- the old failed UDP generation must not be revived as current evidence; recovery must be represented as current-generation validated evidence consistent with the existing manager contract;
- path validation remains distinct from packet health/score;
- no unauthenticated packet may trigger migration;
- no new wire/crypto/Session-ACK architecture merely to make the lab seam convenient;
- no production route/firewall/DNS/proxy/tunnel/qdisc change;
- preserve uncertain/replayed/dedup/confirmed accounting through both transitions.

**Fault/recovery seam:** the current `--cease-udp-replies-after` behavior is explicitly an application-level lab fault. If a recovery control is needed, add the smallest bounded experimental-only seam and label it truthfully. Do not claim natural network recovery from a scripted reply restoration.

**Validation:** focused deterministic manager/runtime tests, then real loopback sockets. Required negative coverage includes stale/old generation, unvalidated recovery, unhealthy/margin failure, hold gate, and proof that failed migration leaves TCP active and does not send new UDP application data.

**Gate:** `./scripts/check.sh` + `git diff --check`; fuzz only if untrusted parser/wire decoding changes.

**Commit/push:** one coherent runtime + tests slice.

**Continue immediately to D:** yes.

### D — Local real-socket fallback -> recovery -> migration-back proof

**Status:** `PREAUTHORIZED_AFTER_C`.

**Goal:** demonstrate the complete bounded state path with actual local TCP/UDP sockets before WAN use.

**Required behavior:**

1. authenticated UDP primary carries application data;
2. existing bounded application-level failure seam causes UDP -> TCP fallback under current rules;
3. uncertain data is replayed/deduplicated under existing Session delivery semantics;
4. UDP recovery is freshly authenticated/validated for the current generation;
5. health-margin + hold gate is actually crossed;
6. application ownership migrates back to UDP;
7. at no point do TCP and UDP both carry new application data as active owners;
8. post-return application `DeliveryAck` succeeds on UDP;
9. bounded shutdown/cleanup succeeds on positive and negative paths.

**Evidence/assertions:** active carrier/generation transitions, fallback/recovery/migration event ordering, attempted/confirmed/missing/duplicate/uncertain/replayed counts, and no false success on rejected recovery.

**Commit/push:** exact local proof and tests. Exact-head CI must be green before E.

**Continue immediately to E after exact-head green:** yes.

### E — One materially distinct self-owned VPS migration-back observation

**Status:** `PREAUTHORIZED_AFTER_D_GREEN` if the local path is truthfully ready.

**Priority:** highest rental-window task once D is green.

**Execution boundary:** self-owned client/VPS only, standing authorization, temporary unprivileged listeners, smallest workload that crosses fallback and return, <=10 minutes, <=256 MiB application traffic, <=32 sessions, bounded capture only if useful, no production network modification.

**Evidence:** exact git and binary identity, actual parameters, start/end/duration, scripted failure/recovery class, active carrier/generation events, fallback and migration-back boundaries, application attempted/confirmed/missing/duplicate/uncertain/replayed counts, cheap CPU/RSS/FD/socket observations when already available, and explicit cleanup.

**Claim boundary:** scripted application-level reply cessation/restoration is exactly that; it is not natural degradation, PTO blackhole, or general middlebox recovery evidence.

**Negative rule:** preserve the first meaningful negative at its exact stage. No unchanged retry. Repair code/instrumentation/config or change the hypothesis before another live attempt.

**Commit/push:** one bounded evidence checkpoint.

**Continue immediately to F:** yes.

### F — Reconcile migration-back evidence and authoritative release matrix

**Status:** `PREAUTHORIZED_AFTER_E`.

Update only the capability/evidence rows actually answered by C–E in `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `docs/status.md`, and the exact artifact references.

In this same reconciliation, repair `PLAN-DRIFT-001`: remove the stale claim that repeated warm failover still needs the old `07545f0` Python-runner-entry fix. Reclassify it according to the later exact `9fd2411` / `a117086` inner-collector/evidence boundary without erasing either negative.

One bounded positive remains one observation, not a reliability rate, production claim, or natural-network recovery result.

**Validation:** repository consistency gate + `git diff --check`.

**Commit/push:** one evidence/status closure.

**Continue immediately to G:** yes.

### G — Select the next VPS-unlock seam: endpoint migration or live PMTUD

**Status:** `READY_LOCAL_AFTER_F`; proposal authority applies.

Inspect current code and choose the smaller/high-value missing real-network seam:

1. endpoint/source migration if the owned environment can produce a genuine endpoint change without production route mutation; otherwise
2. integrate the existing authenticated PLPMTUD state into a live probe/ACK path without trusting unauthenticated ICMP; otherwise
3. another real `BLOCKED_IMPLEMENTATION` release row with a short path to VPS evidence.

Do not choose FEC, 0-RTT, striping, heterogeneous aggregation, or exotic carriers merely to fill queue depth.

**Commit/push:** runtime unlock + focused tests.

**Continue immediately to H:** yes.

### H — Local proof then one bounded VPS observation for G

**Status:** `PREAUTHORIZED_AFTER_G_GREEN`.

Use the same discipline: local real-socket proof, exact-head green, then one minimal materially distinct self-owned VPS observation if the question is truly `READY_LIVE`. Preserve negatives and cleanup; no unchanged retry.

**Continue immediately to I:** yes.

### I — Repeated warm-failover diagnostic repair fallback

**Status:** `READY_LOCAL_FALLBACK`; use only if C/G are genuinely architecture/environment blocked or while their exact CI is pending and this work is independent.

Do **not** rerun `9fd2411` or `a117086` unchanged. The next useful slice is local instrumentation only: retain a bounded sanitized inner collector failure category/diagnostic in the outer structured evidence so a future changed-hypothesis invocation can distinguish configuration/orchestration/runtime failure without exposing endpoint/credential material.

After the instrumentation change and local synthetic/dry verification, one materially changed self-owned VPS repeated-failover run is preauthorized if it answers the still-open repeated-resilience question. A new negative is valid evidence; no success is required.

This fallback is intentionally behind the missing migration-back capability so harness work does not become the main project again.

### J — Independent security/release debt fallback

**Status:** `READY_LOCAL_FALLBACK`; must never displace READY runtime/VPS work.

While C2 remains unresolved, cover only deterministic boundaries independent of terminal-source retention or one exact-tree release-navigation consolidation after visible runtime progress. Reuse existing tests. Do not reopen pre-auth checker infrastructure.

Optional key-update diagnostic precision belongs here only if no higher-value runtime/VPS work is ready.

## Completion gates

The integrated key-update closure is complete now:

- synchronized periodic key-update runtime exists and is exact-green;
- mismatch fail-closed negative exists;
- one bounded self-owned VPS observation is retained;
- authoritative status no longer classifies live key update as implementation-blocked;
- missing timestamps are explicitly recorded as not retained rather than invented;
- accepted implementation/evidence lineage is on default `main`;
- integrated exact-head CI is green;
- governance flags remain unchanged.

The next visible closure is migration-back: C–F are complete only when the real runtime path exists, local sockets prove the full fallback/recovery/return sequence, one bounded VPS observation or exact retained negative is archived when dependency-ready, authoritative status is reconciled, and evidence wording stays within the scripted-fault boundary.

The broader rolling queue remains active through G–J unless a real stop condition occurs.

## Do not expand into

- dynamic wire rekey-control work merely because fixed-schedule key update succeeded;
- public or production listener deployment;
- new source-retention TTL/LRU/history/epoch/eviction policy without reviewed authority;
- FEC/0-RTT/UDP+TCP striping/heterogeneous aggregation/exotic carriers without an observed-problem gate;
- unchanged repeats of historical failover/repeated-failover/HY2/key-update negatives or positives;
- reading, printing, copying, hashing, uploading, or committing protected identity / SSH private-key contents;
- production route/firewall/DNS/proxy/tunnel/qdisc changes;
- third-party targets or scanning;
- release/RC/freeze/production promotion.

## Questions requiring maintainer decision

None for the current migration-back / VPS / next-runtime queue.

C2 terminal-source retention remains a future release/security policy choice. Until a policy is approved, conservative bounded cleanup plus explicit literal-D019 non-compliance remains in force and does not block independent runtime/WAN work.
