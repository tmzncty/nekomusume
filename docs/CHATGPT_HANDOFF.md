# Nekomusume ChatGPT Handoff

Checked at: 2026-09-08 07:00 Asia/Shanghai
Repository main HEAD reviewed: `49c4fc127c5750f1e91b2da30612f3be6c08fc93`
Previous reviewer handoff commit: `49c4fc127c5750f1e91b2da30612f3be6c08fc93`
Current implementation/evidence branch: `work/e1a-staged-accounting-20260907` at exact `69d0ed93d95bffa8727469cafb347b5c4d43b145`
Previous checked implementation HEAD: `2f4f59a9887dbcde2973175d3f7abf30f8edaa15`
Historical partial-E2 branch retained: `work/continue-20260904` at `d271a99a2ab26abbcb146c411ba0fde697395abe`

## What changed

One meaningful evidence commit landed after the previous review:

- `69d0ed9` — **retains one bounded self-owned VPS synchronized periodic TCP key-update observation** for exact implementation `2f4f59a`. This is evidence/docs only; it does not change runtime, wire, crypto, Session, Carrier or release semantics.

The retained observation records:

- administrator-controlled VPS identity verified by non-secret hostname/UID/architecture checks;
- authenticated periodic TCP Session;
- 3 application records x 16 bytes = 48 application bytes;
- fixed `--key-update-after 1` schedule;
- exact release binary size `1,208,216` bytes and SHA-256 `6be15273b2ce4b3e241d800d15d8eefd334d6d37615bae733c5bff4a64a89a43`;
- both peers emitted secret-free `key_phase=1` after sequence 1;
- client `attempted=3 confirmed=3 missing=0 duplicates=0`;
- server `received=3 confirmed=3 duplicates=0`;
- bounded cleanup found no experiment listener on TCP 40080 and no remaining Nekomusume experiment process; the remote experiment path and temporary local identity material were removed.

Exact-head CI is green:

- implementation/evidence exact `69d0ed93d95bffa8727469cafb347b5c4d43b145` — Rust CI run `34165984858`, `success`;
- reviewer main exact `49c4fc127c5750f1e91b2da30612f3be6c08fc93` — Rust CI run `34165251936`, `success`.

This closes the previously `READY_LIVE_NOW` bounded question. Do **not** rerun the same key-update scenario for freshness. The result is one correctness observation, not a reliability rate or production rekey protocol.

The authoritative planning/status files on the implementation branch are now stale in a concrete way: `IMPLEMENTATION_PLAN.md` still classifies live key update as `BLOCKED_IMPLEMENTATION`, and its opportunity section still says `READY_LIVE: none`, despite exact-green runtime plus retained VPS evidence. That must be reconciled before moving the accepted lineage onto default `main`.

## Review verdict

**ACCEPT_BOUNDED_LIVE_KEY_UPDATE — advance immediately to authoritative classification reconciliation, then integrate the accepted implementation/evidence lineage onto default `main`, then continue outward to carrier recovery/migration-back.**

No HIGH/BLOCKER correctness, security or evidence defect was discovered in `69d0ed9`. The evidence language is appropriately narrow and does not claim dynamic peer-negotiated rekey, arbitrary resynchronization, reliability, public reachability, performance superiority, RC or production readiness.

C2 terminal-source retention remains an isolated release/security policy limitation and does not block this runtime/VPS queue. No administrator action is required.

## Review findings and retained boundaries

### KEYUP-LIVE-001 — ACCEPTED — bounded self-owned VPS synchronized key update

The exact `2f4f59a` runtime, retained by evidence commit `69d0ed9`, now has both local/process evidence and one self-owned VPS observation:

- one already-authenticated periodic TCP Session crosses exactly one fixed synchronized phase boundary;
- application/DeliveryAck traffic continues after the boundary;
- all 3/3 bounded application records complete in this observation;
- no missing or duplicate application record is reported;
- both endpoints independently report the same phase transition;
- cleanup is recorded as zero experiment listener/process residue.

This is sufficient for the narrow release-matrix question “can two controlled peers synchronously rekey this already-authenticated bounded periodic TCP Session at a fixed out-of-band schedule on the observed self-owned path?”

It does **not** prove:

- a wire-level key-update negotiation/control message;
- arbitrary peer-initiated rekey;
- recovery from asymmetric update without failure;
- dynamic resynchronization;
- long-term key-rotation policy;
- interaction with carrier transition;
- reliability rate, production readiness, public reachability or superiority.

Do not repeat this exact scenario unless a new question, code/configuration change, instrumentation change or materially different path condition exists.

### KEYUP-EVIDENCE-002 — MEDIUM — experiment duration/start-end metadata not retained in the committed summary

`docs/standing-vps-lab-authorization.md` asks every public experiment to be associated with start/end time, and the previous handoff asked to retain actual duration. `artifacts/periodic-key-update-vps-2f4f59a/evidence.md` records count, bytes, update boundary, binary identity, client/server outcomes and cleanup, but not explicit start/end or elapsed duration.

This is an evidence-completeness gap, **not** a reason to rerun the successful scenario.

During the next classification commit:

- if the coding agent still has trustworthy structured/local experiment timestamps from this exact run, append only those exact values to the evidence note;
- otherwise explicitly state `duration/start-end not retained` and keep the result bounded to the facts already committed;
- never reconstruct or invent timestamps from commit time, shell history or guesswork.

Do not delay integration or the next runtime seam solely to manufacture missing duration metadata.

### COORD-001 — ACTIVE — default main materially trails accepted executable truth

`main` at `49c4fc1` is reviewer/navigation lineage. The implementation/evidence branch at `69d0ed9` is 18 commits ahead and 3 reviewer commits behind from merge base `74abbd3`; it carries accepted E1A/E2/C1 work, local live-key-update runtime, mismatch negative and the VPS evidence.

This divergence is now large enough that default `main` must be reconciled at the next meaningful boundary. Do not keep accumulating runtime features only on the long-lived work branch.

After authoritative key-update classification is corrected, perform one deliberate **normal non-force integration** onto default `main`. Preserve exact historical commits/evidence boundaries and preserve the latest reviewer-owned `docs/CHATGPT_HANDOFF.md` when resolving that file.

### KEYUP-RUNTIME-001 — RETAINED ACCEPTED

Retain exact `0b293f7` synchronized periodic TCP runtime and `2f4f59a` mismatch negative. No new wire control frame, crypto primitive, Session/Carrier/ACK semantic or security-policy number was introduced.

The event-specific mismatch assertions described by previous reviews remain optional MEDIUM diagnostic hardening only. They do not block runtime/VPS/integration work.

### RSEC-001E2-GUARD — CLOSED at `655df00`

Retain the responder-ordering/evidence guard. Do not reopen checker infrastructure absent a concrete regression.

### RSEC-001E1A — CLOSED at `164731d` / current lineage

Retain staged one-logical-record TCP accounting across ordinary, periodic, multistream and failover responder handshakes.

### RSEC-001C1 — CLOSED at `f7e2cf1`

Carrier-aware pre-auth source projection remains accepted and bounded.

### RSEC-001C2 — POLICY LIMITATION at `f066af5`

`docs/adr/m1-g0-preauth-source-retention-amendment-request.md` remains unresolved. Do not invent TTL/LRU/history/epoch/eviction values. For the current bounded research implementation, preserve bounded in-process cleanup and keep literal D019 terminal-source no-reset as an explicit unimplemented release/security limitation. This does not claim full D019 compliance and does not block independent runtime/WAN evidence.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- Exact `69d0ed9` has green CI and adds retained self-owned VPS evidence for exact implementation `2f4f59a`; it adds no new implementation semantics.
- The live key-update result is a **single bounded correctness observation** on one controlled path, not a reliability rate or production rekey protocol.
- Fixed schedule synchronization remains out-of-band experimental configuration; no dynamic rekey negotiation may be implied.
- Historical WAN/HY2/failover/periodic positive and negative evidence remains immutable at exact commit boundaries.
- Standing VPS authorization continues to cover bounded self-owned TCP/UDP Session, key-update, failover, migration, recovery, PMTUD, resource observation, package rehearsal and cleanup within documented limits.
- A failed future WAN run must be retained at its exact stage; no unchanged retry is permitted.
- Protected identity material, SSH private keys, credentials, private endpoint material and raw private diagnostics remain unread/untracked/uncommitted. Secret connection configuration is used only through intended tools and must never be printed or copied into repository evidence.

## Rolling Work Queue

Coordination cadence remains reviewer `:00`, coding-agent wake/resume `:20`; neither is a work-duration limit. If work is already running, do not interrupt it. Finish a coherent closure package -> gates -> commit -> push -> immediately consume the next dependency-ready package. One commit, nominal hour, reviewer interval, documentation checkpoint or ordinary CI wait is not a stop condition.

The coding agent owns ordinary local design choices under `AGENTS.md` proposal authority. The queue below is deliberately organized as outward closure packages rather than checker/doc micro-tickets.

### A — Reconcile live-key-update evidence into authoritative status

**Status:** `READY_LOCAL`; immediate next slice.

**Goal / why now:** repository truth must stop calling live key update `BLOCKED_IMPLEMENTATION` now that exact-green runtime and one retained VPS observation exist.

**Files/concepts:** `IMPLEMENTATION_PLAN.md`, `ROADMAP.md`, `docs/status.md`, and `artifacts/periodic-key-update-vps-2f4f59a/evidence.md` only if trustworthy exact timestamps/duration are still available.

**Protected invariants/evidence boundary:** update only the narrow fixed-schedule synchronized-key-update capability/evidence classification. Do not mark the whole release matrix complete. Do not promote RC/production/freeze/release. Do not rewrite historical negatives.

**Required behavior:**

- replace stale `live key update = BLOCKED_IMPLEMENTATION` wording with a bounded evidence classification tied to exact `2f4f59a` / `69d0ed9`;
- remove stale `READY_LIVE: none` only if current truth now has other ready rows; otherwise explicitly say the key-update row is answered and re-evaluate remaining rows;
- distinguish local/runtime implementation, one live self-owned observation and unsupported dynamic rekey semantics;
- if exact start/end/duration from the already-completed run is still available, append it; otherwise state it was not retained, without rerun or invention.

**Validation:** `./scripts/check.sh` + `git diff --check`; documentation/status consistency checks already included in the repository gate.

**Commit/push:** one coherent evidence/status commit on the implementation branch.

**Continue immediately to B:** yes.

### B — Integrate accepted implementation/evidence lineage into default main

**Status:** `PREAUTHORIZED_AFTER_A_GREEN`; repository-truth closure.

**Goal / why now:** default `main` should again contain the executable implementation and accepted evidence, not only reviewer navigation.

**Files/concepts:** full accepted `work/e1a-staged-accounting-20260907` lineage plus current `origin/main`.

**Protected invariants:**

- no force-push;
- no history rewrite or squash that destroys exact evidence boundaries;
- no dropping accepted E1A/E2/C1/key-update commits;
- latest reviewer-owned `docs/CHATGPT_HANDOFF.md` wins on that path if a merge conflict occurs;
- governance flags unchanged.

**Required behavior:** fetch current `origin/main`, perform one deliberate normal reconciliation/merge, run full repository gate, push integrated default branch according to existing repository workflow, and verify exact integrated-head CI.

**Validation:** `./scripts/check.sh` + `git diff --check` before push; exact integrated `main` CI must be green before a later VPS run that depends on integrated code.

**Commit/push:** ordinary merge/integration, no force.

**Continue immediately to C:** yes; if CI is pending, independent C design/local work may begin, but do not run VPS from an unverified integrated head when that exact head is the intended experiment candidate.

### C — Select and specify the next outward runtime seam: carrier recovery / migration-back

**Status:** `READY_LOCAL_AFTER_B`; proposal authority applies.

**Goal / why now:** migration-back is still `BLOCKED_IMPLEMENTATION` for real-socket evidence even though Carrier Manager already has validated generation + health-margin + hold-gate state logic. It directly exercises Nekomusume’s defining Session-above-Carrier purpose and is higher value than cosmetic key-update assertions.

**Default target:** one authenticated Session starts with UDP primary and warm authenticated TCP fallback, performs a bounded failure-driven promotion to TCP using existing uncertain/replay/dedup semantics, then after UDP recovery satisfies the existing validated generation/health-margin/hold gate and migrates application ownership back to UDP without dual-active data transmission.

**Files/concepts:** existing failover/resume runtime in `crates/neko-cli`, Carrier Manager/migration-back state in `crates/neko-carrier`, existing DeliveryAck/uncertain/dedup evidence model and process tests.

**Protected invariants:** Session remains above Carrier; `single-active, multi-ready`; no simultaneous UDP+TCP application striping; no new wire/crypto/Session-ACK architecture unless a genuine decision is discovered; no production network changes.

**Proposal behavior:** compare 1–3 minimal implementation shapes, choose the smallest fail-closed path that reuses existing runtime and manager semantics, then implement without waiting for reviewer approval. If migration-back unexpectedly requires a core architecture decision, record the exact conflict and immediately choose another independent outward candidate rather than stopping the whole project.

**Commit/push:** runtime seam + focused tests.

**Continue immediately to D:** yes.

### D — Local real-socket migration-back proof

**Status:** `PREAUTHORIZED_AFTER_C`.

**Goal:** prove the selected recovery/migration-back runtime with actual local sockets before spending WAN time.

**Required behavior:**

- establish authenticated UDP primary and warm/ready TCP fallback;
- inject only the existing bounded application-level or isolated test failure mechanism; do not claim natural packet-loss detection if the injection is application-level;
- promote TCP under existing failover rules;
- recover/revalidate UDP generation under existing manager gates;
- migrate application ownership back to UDP only after the gate;
- preserve uncertain/replayed/dedup/DeliveryAck accounting;
- never have two active owners carrying new application data;
- bounded shutdown/cleanup on success and failure.

**Validation:** focused process/real-socket tests + `./scripts/check.sh` + `git diff --check`; fuzz only if untrusted parser/wire decode changes.

**Commit/push:** exact implementation head with green CI before E.

**Continue immediately to E:** yes after exact-head green.

### E — One materially distinct bounded self-owned VPS migration-back observation

**Status:** `PREAUTHORIZED_AFTER_D_GREEN` if the local runtime path is truthfully ready.

**Goal / why now:** capture a second high-value rental-window-only behavior result: failure-driven fallback followed by recovery and migration-back in one bounded controlled Session.

**Execution boundary:** self-owned client/VPS only, standing authorization, temporary unprivileged listeners, smallest workload that crosses fallback and recovery, <=10 minutes, <=256 MiB, <=32 sessions, no production route/firewall/DNS/proxy/tunnel/qdisc modification.

**Evidence:** exact git/binary identity, actual parameters, failure injection class, fallback/recovery/migration timestamps/events, active carrier/generation transitions, application attempted/confirmed/missing/duplicate/uncertain/replayed counts, cheap process/socket resource observations when already available, start/end/duration, explicit cleanup.

**Claim boundary:** if the failure injection is controlled application-layer reply cessation, say exactly that; do not promote it to natural degradation/PTO-blackhole evidence.

**Negative rule:** preserve the first meaningful negative. No unchanged retry; repair the discovered code/harness/config issue or change the hypothesis before another run.

**Commit/push:** bounded evidence checkpoint.

**Continue immediately to F:** yes.

### F — Reconcile migration-back evidence and release matrix

**Status:** `PREAUTHORIZED_AFTER_E`.

**Goal:** update only the exact bounded capability/evidence rows answered by E.

**Files/concepts:** `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `docs/status.md`, exact new artifact/evidence references.

**Protected boundary:** one successful bounded observation is not a reliability rate, production readiness or general natural-network recovery claim. Negative evidence retains its exact blocker category.

**Validation:** repository consistency gate + `git diff --check`.

**Commit/push:** evidence/status closure.

**Continue immediately to G:** yes.

### G — Select the next VPS-unlock seam: endpoint/path migration or live PMTUD

**Status:** `READY_LOCAL_AFTER_F`; proposal authority applies.

**Priority choice:** inspect current code and choose the smaller/high-value real-network seam:

1. endpoint/source migration if the owned environment can create a genuine endpoint change without production route mutation; otherwise
2. integrate existing authenticated PLPMTUD state into a live probe/ACK path without trusting unauthenticated ICMP; otherwise
3. another genuinely `BLOCKED_IMPLEMENTATION` release row with a bounded path to VPS evidence.

Do not choose Experimental Track work merely to fill queue depth. Do not implement exotic carriers, FEC, 0-RTT or aggregation absent an observed problem.

**Commit/push:** one runtime-unlock slice + focused tests.

**Continue immediately to H:** yes.

### H — Local proof then one bounded VPS observation for the selected seam

**Status:** `PREAUTHORIZED_AFTER_G_GREEN`.

Follow the same discipline: local real-socket proof first, exact-head green, then one minimal materially distinct self-owned VPS observation if the question is truly `READY_LIVE`. Preserve negative evidence and cleanup; no unchanged retry.

**Continue immediately to I:** yes.

### I — Independent security/release debt fallback

**Status:** `READY_LOCAL_FALLBACK`; must never displace READY runtime/VPS work.

While C2 remains unresolved, cover only genuinely missing deterministic boundaries independent of terminal-source retention, or one exact-tree release-navigation consolidation after visible runtime progress. Reuse existing tests. Do not reopen pre-auth checker infrastructure. Do not let docs/checker work become the main lane again.

The optional key-update mismatch event-precision assertions may be done here only if no higher-value runtime/VPS work is dependency-ready.

## Completion gates

The key-update outward closure is complete when all are true:

- synchronized periodic authenticated TCP key update remains locally green;
- exact implementation/evidence head CI is green;
- one bounded self-owned VPS key-update observation is retained with exact implementation/binary/parameters/result/cleanup;
- authoritative status no longer says live key update is `BLOCKED_IMPLEMENTATION`;
- evidence explicitly distinguishes fixed experimental schedule from dynamic/production rekey protocol;
- accepted implementation/evidence lineage is reconciled onto default `main`;
- governance flags remain unchanged.

The first three are now satisfied at `69d0ed9`; A/B close the remaining repository-truth gates. Do not repeat the key-update VPS run merely to add more samples.

The broader outward queue remains active through C–I unless a real stop condition occurs.

## Do not expand into

- new wire rekey-control frames solely to generalize the fixed-boundary experiment;
- production/public listener deployment;
- new source-retention TTL/LRU/history/epoch/eviction policy without reviewed authority;
- FEC/0-RTT/striping/heterogeneous multipath/exotic carriers without an observed-problem gate;
- unchanged repeats of historical failed WAN/HY2/repeated-failover/key-update lines;
- reading/printing/copying/committing protected identity or SSH private-key contents;
- production route/firewall/DNS/proxy/tunnel/qdisc changes;
- third-party targets or scanning;
- release/RC/freeze/production promotion.

## Questions requiring maintainer decision

None for the current classification/integration/runtime/VPS queue.

C2 terminal-source retention remains a future release/security policy choice. Until a new policy is approved, conservative bounded cleanup plus explicit literal-D019 non-compliance remains in force and does not block this queue.
