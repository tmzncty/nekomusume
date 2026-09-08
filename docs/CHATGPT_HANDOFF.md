# Nekomusume ChatGPT Handoff

Checked at: 2026-09-08 10:00 Asia/Shanghai
Repository main HEAD reviewed: `2d0e8b0e04597e97ca6cee8b1bf445468956ea30`
Previous reviewer handoff commit: `2d0e8b0e04597e97ca6cee8b1bf445468956ea30`
Previous checked implementation/evidence HEAD: `759cb1fe8a3f3a9cd8b7789595d291497412608f`
Current execution branch: `work/e1a-staged-accounting-20260907` at `759cb1fe8a3f3a9cd8b7789595d291497412608f`
Historical partial-E2 branch retained: `work/continue-20260904` at `d271a99a2ab26abbcb146c411ba0fde697395abe`

## What changed

No coding-agent implementation or evidence commit landed after the previous review. `main` contains only the previous reviewer handoff `2d0e8b0`; the execution branch still points to the prior integrated implementation tree `759cb1f` and has not reconciled the latest handoff.

Exact reviewer-head CI is green:

- `main` exact `2d0e8b0` Rust CI run `34172107640` — `success`.

The repository, CI, standing VPS authorization, current environment model, and core Session/Carrier/ACK/crypto/wire architecture expose no blocker to the migration-back slice. Since the previous review, two normal coding-agent wake/resume opportunities have passed without a migration-back coding checkpoint. This now meets the durable `AGENTS.md` stagnation rule.

This is therefore **STALLED_IMPLEMENTATION**, not a reason to invent another checker, branch, approval gate, or documentation-only task. The migration-back contract below is deliberately more concrete so the coding agent can implement without waiting for another design answer.

## Review verdict

**CONTINUE_STALLED_IMPLEMENTATION_WITH_CONCRETE_RECOVERY_CONTRACT — fetch/reconcile current `main`, then implement the bounded UDP recovery -> migration-back path now.**

No HIGH/BLOCKER correctness or security defect was found in existing accepted code. C2 terminal-source retention remains an explicit release/security policy limitation and does not block runtime/WAN work. Do not spend the next execution interval on compatibility prose, package re-audit, responder inventory, or repeated-failover harness work while migration-back is READY.

No administrator action is required.

## Review findings

### MIGBACK-001 — READY / implementation stall — manager semantics exist, live reverse path does not

The reusable manager contract is already implemented and tested:

- TCP is active at path 2 / generation 1 after current failover promotion;
- `CarrierManager::migrate_back_to_udp(MigrationCandidate)` accepts only a different path at the **current active generation**;
- explicit validation, independently healthy score, score margin, and hold gate are required;
- rejected candidates leave active ownership unchanged;
- successful migration changes exactly one active owner and resets the migration hold.

The missing seam is in the live failover command. Current runtime establishes/authenticates UDP, fails over to authenticated/resume-validated TCP, replays uncertain application records, then stops. The post-handshake UDP server loop currently understands application `Data` only; it has no post-TCP authenticated recovery-control exchange and no post-return application proof.

This gap is local implementation work, not an architecture decision.

### MIGBACK-002 — concrete preferred implementation family

The coding agent retains proposal authority, but the smallest reviewed family is now explicit. Prefer it unless code work reveals a concrete invariant violation.

**Do not add a new wire message merely for this lab seam.** Reuse the already authenticated `ProcessMessage::ReadinessRequest` / `ReadinessResponse` control envelope as the bounded recovery challenge transport, while keeping semantic domains distinct:

- `ReadinessRequest/Response` is only the authenticated control carrier;
- successful AEAD opening + exact peer + exact tuple + fresh challenge response is the bounded **recovery-path challenge evidence**;
- `MigrationCandidate.validated=true` is derived only after that fresh challenge completes;
- this must not be documented as “readiness == PathValidated” or “packet ACK == validation”;
- health/score remains a separate observation from validation.

An equivalent typed helper/state-machine over the existing authenticated UDP record is acceptable. Adding a new `ProcessMessage` kind or changing Session/wire semantics is **not** the default solution and would require a fresh justification.

**Recovered UDP identity:** reuse UDP `PathId(1)` only as a **new generation 1 candidate** after TCP path 2 / generation 1 is active. Never revive generation 0 evidence. Before migration, overwrite/refresh the UDP health observation from the new recovery attempt; do not reuse the old pre-failure sample. If the agent finds a concrete reason that the current manager representation cannot safely distinguish this, it may use another bounded path id without waiting, but must still use generation 1 and fresh evidence.

### MIGBACK-003 — bounded runtime sequence

Add an explicit experimental opt-in such as `--migration-back` to the existing failover client/server. Preserve all old failover behavior when the flag is absent.

Recommended constraints for this experimental mode:

- require `--automatic-health-failover` so manager ownership is real rather than simulated;
- require at least three application records because the proof needs one UDP-primary record, at least one TCP-resumed record, and one post-return UDP record;
- do not change global protocol limits or security policy values.

For `--migration-back`, execute this sequence:

1. UDP path 1 / generation 0 authenticates and confirms the first application record exactly as today.
2. The controlled application-level UDP reply-cessation seam produces the existing bounded failure observation.
3. TCP path 2 / generation 1 is authenticated, resume-validated, resource-admitted, and atomically becomes sole active owner.
4. Replay/confirm only the **middle** application records over TCP. Reserve the final logical record for after migration-back. The final record is not tracked as UDP-uncertain because it has not yet been assigned/sent.
5. After TCP application replay completes, send one fresh bounded authenticated UDP recovery challenge using the already-established UDP secure channel. Bind it to Session 7001, UDP target path 1, **generation 1**, delivery epoch 1, and a fresh challenge id. The server may answer this control challenge after the scripted application-reply cessation because it is recovery control, not an application DeliveryAck.
6. The server must answer only after the prior TCP resume path has actually reached the accepted resumed/active state for this experiment. Wrong peer, malformed/tampered record, wrong session/path/generation/epoch, replayed challenge, or unadmitted runtime state must fail closed and must not enable UDP application replies.
7. On the client, only an authenticated exact-tuple fresh response from the expected UDP peer creates the recovery-validation proof. Measure the challenge RTT separately and derive a fresh bounded `HealthSample`; do not infer healthy state merely from the validation boolean.
8. Feed the fresh UDP health sample to the manager. Call the existing migration gate without sending new UDP application data. If the first eligible call returns `HoldGate`, retain TCP as active and continue only the bounded control/hold progression; do not fake the hold counter or send application data early. With the current runtime manager `min_hold_events=1`, one hold rejection followed by the next eligible attempt is sufficient.
9. Only after `migrate_back_to_udp(...) == Ok(true)` may the client send the reserved final application record over UDP. Send no further new application data over TCP after that point.
10. The server enables post-recovery UDP application DeliveryAck only after the accepted recovery challenge. The old `--cease-udp-replies-after` fault must remain truthful: it ceased application replies until this explicit recovery-control transition; do not silently reset the old counter before validation.
11. Validate the final UDP `DeliveryAck` through the existing Session runtime and finish bounded cleanup.

Suggested diagnostics, names flexible but semantics required:

- `udp_recovery_challenge_sent`;
- `udp_recovery_validated` with path/generation and bounded RTT, not secret material;
- `udp_migration_hold` when applicable;
- `udp_migrated_back` with old/new active path and generation;
- `udp_post_return_delivery_ack_validated`;
- one final accounting row preserving attempted/confirmed/uncertain/replayed/duplicate/lost/conflicting counts.

### MIGBACK-004 — runtime negative coverage without test proliferation

Existing `CarrierManager` unit tests already cover old generation, future generation, unvalidated, unhealthy, score-margin, hold, and same-active rejection. Do not duplicate all of those at process level.

Add only runtime-specific negative proof that the new seam could otherwise violate:

- a bad/tampered/wrong-tuple or replayed UDP recovery challenge/response cannot emit `udp_migrated_back`;
- failed recovery leaves TCP active and the reserved final UDP application record is not sent;
- no UDP post-return DeliveryAck success is emitted before migration gate success.

Add a positive real-loopback process test proving event order and the final UDP DeliveryAck. Preserve old failover tests unchanged unless their assertions must account for the opt-in branch.

### PLAN-DRIFT-001 — MEDIUM / deferred into next evidence reconciliation

`IMPLEMENTATION_PLAN.md` still contains stale wording that repeated warm failover needs the old exact-`07545f0` Python-runner-entry fix. Later `9fd2411` and changed-hypothesis `a117086` already entered the structured outer runner; the truthful blocker is bounded inner-collector diagnostic observability. Do not create a standalone docs task for this. Repair it in the migration-back evidence/status reconciliation package after visible runtime progress.

### RSEC-001E2-GUARD — RETAIN CLOSED at `655df00`

Do not reopen responder checker/inventory infrastructure absent a concrete regression.

### RSEC-001E1A — RETAIN CLOSED at `164731d` / current lineage

Staged one-logical-record TCP accounting remains accepted across all real TCP pre-auth responder handshakes.

### RSEC-001C1 — RETAIN CLOSED at `f7e2cf1`

Carrier-aware pre-auth source projection remains accepted and bounded.

### RSEC-001C2 — POLICY LIMITATION at `f066af5`

Terminal-source retention remains unresolved. Do not invent TTL/LRU/history/epoch/eviction values. Conservative bounded cleanup plus explicit literal-D019 non-compliance remains the current release/security boundary. It does not block migration-back, VPS recovery evidence, endpoint migration, or live PMTUD.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- Exact `759cb1f` is the accepted integrated implementation/evidence tree before this reviewer-only handoff.
- Exact `2d0e8b0` is reviewer coordination only and has green Rust CI; it adds no runtime/WAN/performance semantics.
- Exact `69d0ed9` remains one bounded self-owned VPS synchronized key-update observation, not a reliability rate or dynamic rekey protocol.
- Existing controlled failover evidence uses **application-level UDP reply cessation**. A future restoration under `--migration-back` is likewise a scripted application-level recovery seam, not natural path recovery, PTO blackhole recovery, or general middlebox evidence.
- Authenticated recovery challenge evidence, path validation, health score, and Session DeliveryAck must remain distinct evidence domains even when one bounded runtime composes them.
- Historical failover/repeated-failover/HY2/periodic evidence remains immutable at its exact commit boundaries.
- Standing VPS authorization covers bounded self-owned migration/recovery work once the local runtime is exact-green; no new WAN approval is required.
- Protected identities, SSH private keys, credentials, private endpoints, and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

Coordination remains reviewer `:00`, coding-agent wake/resume `:20`; neither is a work-duration limit. If work is running, do not interrupt it. Finish a coherent closure package -> gates -> commit -> push -> immediately consume the next dependency-ready package. One commit, one nominal hour, one reviewer interval, a reviewer-only main commit, or ordinary CI pending is not a stop condition.

### Q0 — Reconcile reviewer handoff and begin work

**Status:** `READY_LOCAL`; coordination only.

Fetch `origin/main` and normally merge/reconcile the reviewer handoff into the work branch. Do not force-push or discard implementation history. `docs/CHATGPT_HANDOFF.md` remains read-only to the coding agent.

Do not stop after the merge commit. Continue immediately to C in the same execution.

### C — Implement bounded UDP recovery -> migration-back runtime

**Status:** `READY_LOCAL`; immediate highest-value slice.

Implement `MIGBACK-002` through `MIGBACK-004` above in the existing failover command. Preserve the default non-migration behavior. No new wire message, no new crypto primitive, no striping, and no production network changes.

**Gate:** focused unit/process tests, real loopback sockets, `./scripts/check.sh`, `git diff --check`; fuzz only if untrusted parser/wire decoding changes.

**Commit/push:** one coherent runtime + tests checkpoint. Do not split helper, diagnostics, and process proof into separate micro-tickets unless a real failure forces it.

**Continue immediately to D:** yes.

### D — Exact local real-socket fallback -> recovery -> return proof

**Status:** `PREAUTHORIZED_AFTER_C`.

Prove, with actual local TCP/UDP sockets:

- UDP primary app confirmation;
- scripted application-level failure -> TCP active;
- bounded uncertain replay/dedup on TCP;
- fresh authenticated UDP generation-1 recovery challenge;
- separate fresh health sample;
- hold gate crossed while TCP remains active;
- atomic UDP migration-back;
- no concurrent new application data on both carriers;
- final post-return UDP application DeliveryAck;
- deterministic shutdown/cleanup.

Exact-head CI must be green before live VPS execution. If CI is pending, consume an independent fallback only if it does not alter the migration runtime semantics.

**Continue immediately to E after exact-head green:** yes.

### E — One materially distinct self-owned VPS migration-back observation

**Status:** `PREAUTHORIZED_AFTER_D_GREEN`.

Use the smallest workload that crosses the full sequence, preferably 3–5 small records, one scripted failure and one scripted recovery, self-owned client/VPS only, temporary unprivileged listeners, <=10 minutes, <=256 MiB, <=32 sessions, no production route/firewall/DNS/proxy/tunnel/qdisc changes.

Retain exact git/binary identity, actual parameters, start/end/duration, scripted fault/recovery class, carrier/generation transitions, recovery challenge/health/hold/migration events, attempted/confirmed/uncertain/replayed/duplicate/lost/conflicting counts, cheap CPU/RSS/FD/socket observations when already available, and explicit cleanup.

**Claim boundary:** this proves only a bounded scripted application-level fallback/recovery/migration-back observation. It is not natural failure/recovery or a reliability rate.

**Negative rule:** preserve the first meaningful negative at its exact stage. No unchanged retry; require a code/instrumentation/config/hypothesis/path change first.

**Continue immediately to F:** yes.

### F — Reconcile migration-back evidence and planning truth

**Status:** `PREAUTHORIZED_AFTER_E`.

Update only rows actually answered by C–E in `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `docs/status.md`, and exact evidence references. In the same commit repair `PLAN-DRIFT-001` so repeated warm failover is classified by the later inner-collector diagnostic boundary rather than stale `07545f0` runner-entry wording.

One bounded positive remains one observation. A retained negative is also valid evidence and must not be rewritten into success.

**Validation:** repository consistency gate + `git diff --check`.

**Continue immediately to G:** yes.

### G — Next VPS-unlock seam: endpoint migration or live PMTUD

**Status:** `READY_LOCAL_AFTER_F`; proposal authority applies.

Choose the smaller high-value missing runtime seam after reading current implementation:

1. authenticated endpoint/source migration if the owned environment can create a genuine endpoint change without production route mutation; otherwise
2. integrate existing authenticated PLPMTUD state into a live probe/ACK path without trusting unauthenticated ICMP; otherwise
3. another actual `BLOCKED_IMPLEMENTATION` release row with a short path to self-owned VPS evidence.

Do not choose FEC, 0-RTT, striping, heterogeneous aggregation, or exotic carriers merely for queue depth.

**Continue immediately to H:** yes.

### H — Local proof + one bounded VPS observation for G

**Status:** `PREAUTHORIZED_AFTER_G_GREEN`.

Local real-socket proof -> exact-head green -> one minimal materially distinct self-owned VPS observation if truthfully `READY_LIVE`. Preserve negative evidence and cleanup; no unchanged retry.

**Continue immediately to I:** yes.

### I — Repeated warm-failover diagnostic fallback

**Status:** `READY_LOCAL_FALLBACK`; only if C/G are genuinely blocked or while exact CI is pending and work is independent.

Do not rerun `9fd2411` or `a117086` unchanged. The next useful local change is bounded sanitized inner-collector failure categorization in outer structured evidence, followed by synthetic/dry verification. Only after that changed instrumentation may one materially changed VPS repeated-failover attempt be considered.

This remains behind missing runtime capabilities so harness work does not become the project again.

### J — Independent security/release-debt fallback

**Status:** `READY_LOCAL_FALLBACK`; must never displace READY runtime/VPS work.

While C2 remains unresolved, cover only deterministic debt independent of source retention or one exact-tree release-navigation consolidation after visible runtime progress. Reuse existing tests. Do not reopen pre-auth checker infrastructure.

## Completion gates

The synchronized key-update closure remains complete for its bounded question and is not to be rerun for freshness.

The next visible closure is migration-back. C–F are complete only when:

- the opt-in live runtime has a fresh current-generation UDP recovery proof after TCP becomes active;
- validation and health evidence remain separate;
- manager hold/margin gates are actually exercised;
- single-active ownership is preserved;
- one final new application record succeeds on UDP only after migration;
- local real sockets and exact-head CI are green;
- one bounded VPS observation or exact retained negative is archived when dependency-ready;
- authoritative status/planning is reconciled without inflating scripted recovery into natural recovery;
- release/freeze/production flags remain unchanged.

The broader queue remains active through G–J unless a real stop condition occurs.

## Do not expand into

- a new recovery wire message when the existing authenticated control envelope suffices;
- dynamic rekey-control work merely because fixed-schedule key update succeeded;
- public or production listener deployment;
- new source-retention TTL/LRU/history/epoch/eviction policy without reviewed authority;
- FEC/0-RTT/UDP+TCP striping/heterogeneous aggregation/exotic carriers without an observed-problem gate;
- unchanged repeats of historical failover/repeated-failover/HY2/key-update evidence;
- reading, printing, copying, hashing, uploading, or committing protected identity / SSH private-key contents;
- production route/firewall/DNS/proxy/tunnel/qdisc changes;
- third-party targets or scanning;
- release/RC/freeze/production promotion.

## Questions requiring maintainer decision

None for the current migration-back / VPS / next-runtime queue.

C2 terminal-source retention remains a future release/security policy choice. Until approved, bounded cleanup plus explicit literal-D019 non-compliance remains in force and does not block independent runtime/WAN work.
