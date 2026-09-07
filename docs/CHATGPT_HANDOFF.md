# Nekomusume ChatGPT Handoff

Checked at: 2026-09-08 05:58 Asia/Shanghai
Repository main HEAD reviewed: `f8cc5632d16ac8c73212ff8444d26b530907aff7`
Previous reviewer handoff commit: `f8cc5632d16ac8c73212ff8444d26b530907aff7`
Current implementation branch: `work/e1a-staged-accounting-20260907` at exact `2f4f59a9887dbcde2973175d3f7abf30f8edaa15`
Previous checked implementation HEAD: `2f4f59a9887dbcde2973175d3f7abf30f8edaa15`
Historical partial-E2 branch retained: `work/continue-20260904` at `d271a99a2ab26abbcb146c411ba0fde697395abe`

## What changed

No new coding, experiment, evidence, or authoritative-status commit landed after the previous review.

Repository truth is otherwise healthy:

- implementation exact `2f4f59a9887dbcde2973175d3f7abf30f8edaa15` — Rust CI run `34158757820`, `success`;
- reviewer main exact `f8cc5632d16ac8c73212ff8444d26b530907aff7` — Rust CI run `34161927259`, `success`;
- the accepted synchronized periodic TCP key-update runtime remains at `0b293f7` with the mismatch negative at `2f4f59a`;
- there is still **no retained self-owned VPS/WAN key-update observation** on GitHub.

The previous handoff intentionally classified the extra mismatch-event assertions as **MEDIUM evidence hardening, not a correctness/security/architecture blocker**, while the VPS rental policy makes a dependency-ready real-network observation the higher-value asset. At least one coding-agent wake/resume opportunity has now passed after that handoff without a live result, and the prior coding opportunity had already received the same “small negative then live” direction.

This is now **STALLED_OUTWARD_EXECUTION**, not a missing-design, CI, credential, authorization, or repository blocker. The reviewer is removing the optional local assertion refinement from the critical path so it cannot consume another rental-window cycle.

## Review verdict

**EXECUTE_LIVE_NOW — exact `2f4f59a` is sufficiently green and bounded for one minimal self-owned synchronized key-update correctness observation. Do not perform another local-only refinement before the live run. Preserve the under-specific mismatch assertion as a later local evidence-hardening task.**

The accepted local positive path already proves an authenticated real TCP Session crosses the synchronized fixed boundary and continues application/DeliveryAck traffic. `2f4f59a` adds a fail-closed schedule-mismatch negative. The remaining event-specific mismatch assertions improve diagnostic precision but are not needed to make one same-boundary, bounded, self-owned VPS observation truthful.

C2 terminal-source retention remains an isolated release/security policy limitation and does not block runtime/VPS work. No administrator action is required.

## Review findings and retained boundaries

### KEYUP-RUNTIME-001 — ACCEPTED — synchronized local real-socket key update

Retain `0b293f7` and exact-green `2f4f59a` lineage:

- one authenticated periodic TCP Session crosses exactly one configured synchronized key-phase update;
- server rekeys only after writing the Nth authenticated DeliveryAck;
- client rekeys only after authenticating/applying that ack;
- subsequent application records remain authenticated and the positive process test confirms all 3/3 records;
- existing `SecureSession::update_key_phase()` is reused;
- no new wire control frame, crypto primitive, Session/Carrier/ACK semantic, or security-policy number exists.

This is local/process evidence only. It is not WAN evidence, a reliability rate, production rekey negotiation, arbitrary peer-initiated rekey, or dynamic resynchronization.

### KEYUP-RUNTIME-002 — ACCEPTED AS BOUNDED NEGATIVE, diagnostic precision still open

`2f4f59a` proves that intentionally different valid update schedules do not fabricate an all-records-success result and both processes fail boundedly. That is sufficient as a narrow fail-closed negative for the current fixed-schedule experiment.

A later local-only hardening may additionally assert the precise event sequence:

- first old-phase exchange succeeds;
- server emits `periodic_server_key_update seq=1 key_phase=1`;
- client does not reach its later phase-change event;
- the first incompatible post-boundary exchange is where success stops.

Those assertions remain useful, but **they are no longer a prerequisite for the same-boundary VPS observation**. Do not let evidence cosmetics outrank a rental-window-only experiment.

### COORD-001 — MEDIUM — default main still trails accepted implementation lineage

The active work branch carries accepted E1A/E2/C1/key-update runtime work while default `main` is still dominated by reviewer handoffs. Do not merge every hourly handoff. After the live-key-update result and its authoritative classification are committed, perform one deliberate non-force integration of accepted implementation/evidence lineage onto default `main`, preserving the latest reviewer-owned `docs/CHATGPT_HANDOFF.md` on conflict.

This is repository-truth closure, not release promotion.

### RSEC-001E2-GUARD — CLOSED at `655df00`

Retain the responder-ordering/evidence guard. Do not reopen checker infrastructure absent a concrete regression.

### RSEC-001E1A — CLOSED at `164731d` / current lineage

Retain staged one-logical-record TCP accounting across ordinary, periodic, multistream, and failover responder handshakes.

### RSEC-001C1 — CLOSED at `f7e2cf1`

Carrier-aware pre-auth source projection remains accepted and bounded.

### RSEC-001C2 — POLICY LIMITATION at `f066af5`

`docs/adr/m1-g0-preauth-source-retention-amendment-request.md` remains unresolved. Do not invent TTL/LRU/history/epoch/eviction values. For the current bounded research implementation, preserve bounded in-process cleanup and keep literal D019 terminal-source no-reset as an explicit unimplemented release/security limitation. This does not claim full D019 compliance and does not block independent runtime/WAN evidence.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- Exact `2f4f59a` has green CI and includes local process/runtime evidence plus a bounded mismatch negative; it adds no WAN/performance/reliability evidence.
- A self-owned VPS positive can prove only that two controlled peers synchronously rekey an already authenticated bounded Session at the configured out-of-band boundary while application/DeliveryAck traffic continues on that observed path.
- A positive does **not** prove arbitrary peer-initiated rekey, dynamic resynchronization, long-term rotation policy, public reachability, reliability rate, production readiness, or superiority.
- A negative is retained at its exact stage; no unchanged retry is permitted.
- Historical WAN/HY2/failover/periodic evidence remains immutable at exact commit boundaries.
- Standing VPS authorization explicitly covers bounded self-owned key-update, failover, migration, recovery, resource observation, and cleanup within the documented limits.
- Protected identity material, SSH private keys, credentials, private endpoint material, and raw private diagnostics remain unread/untracked/uncommitted. Use secret endpoint/SSH configuration only through intended tools; never print/copy private-key contents.

## Rolling Work Queue

Coordination cadence remains reviewer `:00`, coding-agent wake/resume `:20`; neither is a work-duration limit. If work is already running, do not interrupt it. Finish a coherent closure package -> gates -> commit -> push -> immediately consume the next dependency-ready package. A single commit, nominal hour, reviewer interval, no-diff audit, or ordinary CI wait is not a stop condition.

The coding branch does not need to merge every reviewer-only main commit. `git fetch origin` and read `git show origin/main:docs/CHATGPT_HANDOFF.md`; reconcile histories only at a meaningful implementation/evidence integration boundary.

### A — Execute one bounded self-owned VPS synchronized key-update observation now

**Status:** `READY_LIVE_NOW`; highest rental-window priority.

**Goal / why now:** obtain evidence that cannot be reconstructed after the rented VPS disappears. Local same-boundary implementation and exact-head CI are already sufficient for this bounded correctness question.

**Files/concepts:** existing periodic client/server command, existing secret endpoint/SSH configuration, current implementation exact `2f4f59a`, small evidence note/artifact after the run.

**Protected invariants/evidence boundary:** same authenticated Session; exactly one fixed synchronized update boundary; no new wire negotiation; no production service changes; no claim beyond one bounded observed path; secrets are used only through intended tools and are never printed or committed.

**Execution behavior:**

- verify the controlled target using non-secret identity/hostname/system metadata before deployment;
- use the smallest profile that crosses one boundary, e.g. 3–5 records, small payload, update after 1 or 2;
- use temporary unprivileged listener(s) only;
- do not modify production firewall/route/DNS/proxy/tunnel/qdisc or unrelated services;
- collect auth, key-update, application and DeliveryAck events before/after the boundary;
- collect cheap CPU/RSS/FD/socket observations only if the existing sampler already provides them; do not delay the run to build new metrics tooling;
- cleanup and verify no experiment listener/process/temp residue.

**Validation/evidence:** retain experiment ID, exact git commit, binary hash/size, actual count/bytes/duration/update boundary, client/server results, attempted/confirmed/missing/duplicate counts, relevant key-update events, and explicit cleanup status.

**Negative rule:** preserve the exact negative and cleanup. No unchanged retry; a retry requires changed code/instrumentation/config/hypothesis/path condition.

**Commit/push:** commit the bounded evidence/result after cleanup. Do not rewrite historical artifacts.

**Continue immediately to B:** yes.

### B — Reconcile live-key-update evidence and authoritative classifications

**Status:** `PREAUTHORIZED_AFTER_A`.

**Goal / why now:** remove stale repository claims that still call live key update `BLOCKED_IMPLEMENTATION` after runtime code and live evidence exist.

**Files/concepts:** `IMPLEMENTATION_PLAN.md`, `ROADMAP.md`, `docs/status.md`, and a small exact evidence note/artifact only where materially changed.

**Protected boundary:** update the narrow capability/evidence classification only. Do not promote RC/production/freeze/release; explicitly state what the run does not prove.

**Behavior/validation:** preserve historical negative artifacts unchanged; link the exact run/commit/parameters/cleanup; run documentation/status consistency gates plus `scripts/check.sh`/`git diff --check` when the implementation branch contains the update.

**Commit/push:** one coherent evidence/status checkpoint.

**Continue immediately to C:** yes.

### C — Integrate accepted implementation/evidence lineage into default main

**Status:** `PREAUTHORIZED_AFTER_B_GREEN`; repository-truth closure.

**Goal / why now:** default `main` must once again contain the executable truth rather than only reviewer navigation.

**Files/concepts:** full accepted work-branch lineage plus latest reviewer handoff.

**Protected invariants:** no force-push, no history rewrite, no squash that destroys exact historical evidence boundaries; latest reviewer-owned handoff wins on handoff conflict; governance flags unchanged.

**Behavior/validation:** fetch latest `origin/main`, perform one deliberate normal reconciliation, run full `scripts/check.sh` + `git diff --check`, push integrated head, require exact integrated-head CI green.

**Commit/push:** ordinary merge/integration according to existing repository workflow.

**Continue immediately to D:** yes.

### D — Harden mismatch event assertions only if still useful

**Status:** `READY_LOCAL_AFTER_C`; optional evidence-quality closure, not a runtime/VPS gate.

**Goal / why now:** make the existing mismatch negative prove the exact failure boundary without delaying rental-window evidence.

**Files/concepts:** `crates/neko-cli/tests/probe.rs` only unless stronger assertions expose a real runtime defect.

**Protected invariants:** no silent resynchronization, no success evidence for rejected post-boundary record, bounded shutdown, positive same-boundary path remains green.

**Behavior/validation:** add the narrow event assertions described in KEYUP-RUNTIME-002; targeted tests + `scripts/check.sh` + `git diff --check`; no fuzz unless parser/wire code changes.

**Commit/push:** coherent test-only checkpoint if the assertions add real value. If the live run or current logs already make the boundary unambiguous and another higher-value READY runtime task exists, this slice may remain deferred.

**Continue immediately to E:** yes.

### E — Select the next outward runtime seam

**Status:** `READY_LOCAL_AFTER_C`; may run before D if D is merely cosmetic. Proposal authority applies.

**Goal / why now:** convert another `BLOCKED_IMPLEMENTATION` release row into an executable real-socket path while VPS time remains.

**Default candidate:** **carrier recovery / migration-back over real sockets**, because manager-state logic already exists and a real recovery observation directly tests the project’s carrier-agnostic purpose.

**Alternatives if exact code is materially smaller/higher-value:** endpoint/path migration that can be produced without production route changes; live PMTUD integration on an authenticated path; another newly dependency-ready release row.

**Protected invariants:** Session remains above Carrier; single-active semantics remain intact; no new wire/crypto/ACK architecture unless a genuine decision is raised; no speculative multipath/FEC/0-RTT expansion.

**Behavior:** compare 1–3 minimal local shapes, choose the smallest fail-closed design under current ADRs, implement directly without waiting for reviewer approval. If one candidate needs a new architecture/policy decision, choose another independent candidate rather than stopping the project.

**Commit/push:** runtime seam + focused tests.

**Continue immediately to F:** yes.

### F — Local real-socket proof for selected recovery/migration seam

**Status:** `PREAUTHORIZED_AFTER_E`.

**Goal:** exercise selected behavior with actual sockets before spending WAN time.

**Files/concepts:** smallest process/loopback or netns path; reuse existing Session/Carrier Manager/evidence machinery.

**Protected boundary:** prove only the exact recovery/migration behavior; no generic harness project.

**Behavior/validation:** verify authenticated application continuity, migration/recovery events, uncertain/dedup semantics where applicable, failure boundaries, and cleanup. Targeted tests + full gate; fuzz only if untrusted parser/wire decode changes.

**Commit/push:** exact green implementation head required before G if G depends on it.

**Continue immediately to G:** yes.

### G — One materially distinct bounded VPS recovery/migration observation

**Status:** `PREAUTHORIZED_AFTER_F_GREEN` if truthfully `READY_LIVE`.

**Goal / why now:** obtain second high-value rental-window-only behavior result.

**Behavior:** one minimal self-owned run under standing authorization; preserve exact parameters/events/resources/cleanup. Do not split a >10-minute scenario into nominal runs. Do not modify production network configuration. Do not mix performance work with CPU-heavy build/fuzz.

**Negative rule:** preserve and classify; repair a discovered runtime/harness defect before any changed-hypothesis retry.

**Commit/push:** bounded evidence checkpoint.

**Continue immediately to H:** yes.

### H — Reconcile second runtime/VPS result

**Status:** `PREAUTHORIZED_AFTER_G`.

**Goal:** update authoritative matrix/status only at the semantic boundary actually observed.

**Behavior:** if the bounded question is answered, mark that narrow question sufficient rather than rerunning for freshness; if negative, retain exact blocker category and changed-hypothesis condition.

**Commit/push:** evidence/status closure with governance unchanged.

**Continue immediately to I:** yes.

### I — Independent security/release debt fallback

**Status:** `READY_LOCAL_FALLBACK`; must never displace READY runtime/VPS work.

While C2 remains unresolved, cover only genuinely missing deterministic boundaries independent of terminal-source retention or do one exact-tree release-navigation consolidation after visible runtime progress. Reuse existing tests. Do not reopen pre-auth checker infrastructure and do not allow docs/checker work to become the primary lane again.

## Completion gates

The current key-update outward closure is complete when:

- synchronized periodic authenticated TCP key update remains locally green;
- exact implementation-head CI is green;
- one bounded self-owned VPS key-update observation is retained with exact commit/parameters/result/cleanup, positive or negative;
- authoritative classification no longer calls live key update `BLOCKED_IMPLEMENTATION` once the implementation/live evidence exists;
- evidence language distinguishes fixed experimental schedule from dynamic/production rekey protocol;
- accepted implementation/evidence lineage is reconciled onto default `main`;
- governance flags remain unchanged.

The extra event-specific mismatch assertions are desirable diagnostic hardening but are **not** part of the live-run prerequisite gate.

The broader queue remains active through D–I unless a real stop condition occurs.

## Do not expand into

- new wire rekey-control frames solely for the fixed-boundary experiment;
- production/public listener deployment;
- new source-retention TTL/LRU/history/epoch/eviction policy without reviewed authority;
- FEC/0-RTT/striping/heterogeneous multipath/exotic carriers without an observed-problem gate;
- unchanged repeats of historical failed WAN/HY2/repeated-failover lines;
- reading/printing/copying/committing protected identity or SSH private-key contents;
- production route/firewall/DNS/proxy/tunnel/qdisc changes;
- third-party targets or scanning;
- release/RC/freeze/production promotion.

## Questions requiring maintainer decision

None for the current key-update/runtime/VPS/integration queue.

C2 terminal-source retention remains a future release/security policy choice. Until a new policy is approved, conservative bounded cleanup plus explicit literal-D019 non-compliance remains in force and does not block this queue.
