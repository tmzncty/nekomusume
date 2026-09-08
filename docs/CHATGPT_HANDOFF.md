# Nekomusume ChatGPT Handoff

Checked at: 2026-09-08 11:00 Asia/Shanghai
Repository main HEAD reviewed: `f41c1db0641e9f088473cfa9f1f631d03b18b1fc`
Previous reviewer handoff commit: `f41c1db0641e9f088473cfa9f1f631d03b18b1fc`
Current execution branch: `work/e1a-staged-accounting-20260907` at exact `0130bce740a331260bec6606e159006e8fc3f847`
New implementation commit: `99f2ad95ae031311b5a8b8a5e21c6538237dfe6e` — `feat: add bounded UDP migration-back recovery`
Exact execution-head Rust CI: run `34180780984` — `success`
Historical partial-E2 branch retained: `work/continue-20260904` at `d271a99a2ab26abbcb146c411ba0fde697395abe`

## What changed

The migration-back stall is over. The coding agent implemented a real opt-in UDP recovery -> migration-back path and then merged the latest reviewer handoff into the work branch. Exact `0130bce` is four commits ahead of the prior integrated tree and has green exact-head CI.

Meaningful new runtime behavior exists:

- `FailoverController` can commit an already manager-authorized TCP -> UDP active-owner transition;
- the failover process can reserve one final application record for after return to UDP;
- after TCP fallback/replay, the runtime sends an authenticated UDP recovery-control request on generation 1;
- the existing `CarrierManager::migrate_back_to_udp` hold/generation/validation gate is exercised;
- a final post-return UDP application record and authenticated `DeliveryAck` are exercised on real loopback sockets;
- the positive loopback test now observes one TCP-resumed record and one post-return UDP record.

This is real progress and should be preserved. It is local deterministic/runtime evidence only; it is **not yet truthful VPS-ready recovery evidence** because the new path currently bypasses two important recovery-proof invariants and its final accounting is inaccurate.

## Review verdict

**CONTINUE_WITH_REQUIRED_RECOVERY_FIXES — accept the runtime direction and green CI, but do not run the VPS migration-back experiment yet. Repair fresh health evidence, exact recovery tuple/freshness, and final accounting; add one bounded runtime negative; then go directly to VPS.**

No administrator action is required. No new wire message, crypto primitive, numeric security policy, or production change is needed.

## Reviewer findings

### MIGBACK-005 — HIGH — live migration gate is fed fabricated health instead of measured recovery health

The new client calls `manager.observe(PathId(1), HealthSample { rtt_us: 100, loss_per_mille: 0, pto: 0 })` **before** the recovery challenge succeeds, and similarly installs a hard-coded TCP sample. After the authenticated response, `MigrationCandidate` again carries the same synthetic UDP health sample.

This defeats the intended separation between recovery validation and independent health/score. In a real VPS run, `validated=true` plus a fabricated 100 us / 0-loss sample could authorize migration even though the recovered path's actual observation does not support that score.

**Required repair:** derive the UDP recovery `HealthSample` from the fresh bounded recovery attempt. At minimum, measure a monotonic challenge send -> authenticated exact-response RTT and use that measured RTT in the candidate/manager observation; keep validation boolean and health sample separate. Do not install the recovered UDP healthy sample before the exact authenticated response. If a bounded retry policy is used, preserve truthful timing/loss semantics rather than hard-coding a perfect sample. Do not invent a new global health policy value.

The TCP comparison/active-path sample must also be based on already available live/runtime health information or a clearly typed bounded local observation, not an unexplained magic 500 us constant used only to force the score margin. If current manager APIs need a minimal typed helper to consume measured recovery observations, use proposal authority and implement it locally.

### MIGBACK-006 — HIGH — recovery responder does not enforce the exact Session tuple before enabling post-return UDP application replies

The UDP server recovery pattern currently captures `session` without requiring `SessionId(7001)`. Any authenticated `ReadinessRequest` with target path 1 / generation 1 / delivery epoch 1 is answered and then enables the bounded post-recovery UDP application-reply loop, even if the logical Session field is wrong.

The handoff contract required **exact peer + exact session/path/generation/epoch + fresh challenge** before recovery validation can enable post-return application replies.

**Required repair:** require Session 7001 explicitly on the server recovery request, and retain the exact tuple in the response. Wrong session/path/generation/epoch, malformed/tampered authentication, wrong peer, and replay/stale challenge must fail closed and must not enter the post-return application loop or emit a migration/recovery success diagnostic.

Use the existing unreliable-record replay protection where it already gives structural replay rejection, but do not assume it replaces the logical fresh-challenge contract. Keep one bounded outstanding recovery challenge or a similarly small typed freshness state. No new wire kind is needed.

### MIGBACK-007 — MEDIUM — the reserved post-return record is still counted/tracked as UDP-uncertain before it is ever assigned or sent

Before the new recovery branch, the client tracks every record after the first as `FailoverController` uncertain. The migration-back mode later withholds the final record from TCP and sends it only after successful return to UDP, but that final record was already placed in the uncertain set.

This contradicts the intended single-active assignment boundary: the reserved final record must remain **unassigned/unsent**, not UDP-uncertain, until migration succeeds. If recovery fails, an unsent record must not look like a previously assigned uncertain record.

**Required repair:** in migration-back mode, exclude the reserved final record from pre-failure uncertain tracking. Keep the old behavior unchanged for ordinary failover mode.

### MIGBACK-008 — MEDIUM — final accounting omits the confirmed post-return UDP record

The final `failover_accounting` row reports `confirmed_records = 1 + replayed_records` and corresponding bytes. In migration-back mode the post-return UDP record is separately authenticated and confirmed, so the row under-counts confirmed application delivery by one while `ordered_records_complete count={count}` claims the full count.

**Required repair:** make attempted/confirmed/uncertain/replayed/duplicate/lost/conflicting accounting agree with actual assignment and DeliveryAck events. For the 3-record positive shape, the expected semantic split is one initial UDP-confirmed record + one TCP replay/confirm + one newly assigned post-return UDP-confirmed record; the last record is not uncertain/replayed.

### MIGBACK-009 — MEDIUM evidence-order/test gap — recovery proof needs one explicit negative and truthful server ordering

The positive process test proves the new happy path but does not add the requested recovery-specific fail-closed negative. Also the server currently emits `tcp_resumed` after the recovery-control/post-return block, even though recovery is supposed to be enabled only after the TCP resume path is already accepted. Internally the server has processed authenticated TCP data first, but the emitted evidence order is misleading.

**Required repair:** move/emit the accepted TCP-resumed state before enabling the UDP recovery-control owner, or otherwise make the state transition explicit and testable. Add one bounded process negative that exercises a wrong tuple or tampered/replayed recovery proof and asserts:

- no `migrated_back_to_udp` / equivalent success event;
- TCP remains the active owner;
- the reserved final UDP application record is not sent;
- no post-return UDP `DeliveryAck` success is emitted.

Do not duplicate all `CarrierManager` unit negatives; one runtime-specific proof is enough once the exact-tuple bug is repaired.

## Accepted direction / boundaries

- Reusing authenticated `ProcessMessage::ReadinessRequest/Response` as a recovery-control **carrier** remains acceptable; do not add a new wire message solely for this experiment.
- Recovery validation, health score, Session DeliveryAck, and packet feedback remain separate evidence domains.
- Recovered UDP must be generation 1 after TCP path 2 / generation 1 is active; generation-0 evidence must not be revived.
- Single-active remains mandatory: no new UDP application record before manager migration succeeds; no new TCP application data after successful return.
- C2 terminal-source retention remains an explicit release/security policy limitation; do not invent TTL/LRU/history/epoch values. It does not block this runtime lane.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- The standing VPS authorization still covers bounded self-owned migration/recovery once the corrected exact head is green.
- Protected identities, SSH keys, credentials, private endpoints, and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

The coding agent owns ordinary implementation choices. Finish coherent closure -> focused/full gates -> commit -> push -> continue immediately. Reviewer cadence is not a work-duration limit.

### A — Repair migration-back proof invariants

**Status:** `READY_LOCAL`; highest priority.

On the existing work branch, repair `MIGBACK-005` through `MIGBACK-009` as one coherent closure package where practical.

**Gate:** focused carrier/process tests, real loopback sockets, `./scripts/check.sh`, `git diff --check`; fuzz is not required unless parser/wire decoding changes.

**Commit/push:** required. Do not stop after the commit if exact-head CI can be checked while other dependency-safe work continues.

**Continue immediately to B:** yes.

### B — Exact local fallback -> measured recovery -> migration-back proof

**Status:** `PREAUTHORIZED_AFTER_A`.

Prove on real local TCP/UDP sockets:

- UDP primary confirmation;
- scripted application reply cessation -> TCP active;
- only actually assigned uncertain data replayed over TCP;
- accepted TCP-resumed state precedes recovery handling;
- fresh authenticated exact-tuple generation-1 UDP recovery challenge;
- **measured** fresh recovery health, distinct from validation;
- hold gate while TCP remains active;
- atomic return to UDP;
- final previously-unassigned application record sent only after migration;
- authenticated final UDP DeliveryAck;
- internally consistent accounting and cleanup;
- one recovery-specific fail-closed negative.

Exact-head Rust CI must be green before VPS execution.

**Continue immediately to C after green:** yes.

### C — One bounded self-owned VPS migration-back observation

**Status:** `PREAUTHORIZED_AFTER_B_GREEN`.

Use the smallest workload that crosses the full sequence, preferably 3 small records, one scripted application-level failure and one scripted recovery. Stay within standing authorization: self-owned endpoints, temporary unprivileged listeners, <=10 minutes, <=256 MiB, <=32 sessions, bounded capture if needed, no production route/firewall/DNS/proxy/tunnel/qdisc changes.

Retain exact commit/binary identity, actual parameters, start/end/duration, scripted fault/recovery class, measured recovery RTT/health input, carrier/generation transitions, hold/migration events, attempted/confirmed/uncertain/replayed/duplicate/lost/conflicting counts, cheap resource observations when already available, and explicit cleanup.

**Claim boundary:** one bounded scripted application-level fallback/recovery/migration-back observation only; not natural path recovery, reliability rate, or production readiness.

**Negative rule:** preserve first meaningful negative and do not rerun unchanged.

**Continue immediately to D:** yes.

### D — Reconcile migration-back evidence and planning truth

**Status:** `PREAUTHORIZED_AFTER_C`.

Update only truthfully answered rows in `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `docs/status.md`, and exact evidence references. In the same commit repair stale repeated-failover wording: later `9fd2411` / `a117086` already reached the structured outer runner, so the remaining blocker is bounded inner-collector diagnostic observability, not the old `07545f0` Python-runner-entry boundary.

**Validation:** repository consistency gate + `git diff --check`.

**Continue immediately to E:** yes.

### E — Integrate reviewed migration-back lineage to main

**Status:** `PREAUTHORIZED_AFTER_D`.

Once A-D are internally consistent and exact-green, normally merge/reconcile implementation/evidence with reviewer `main`; preserve history, no force push, no rewriting retained negative evidence. `docs/CHATGPT_HANDOFF.md` stays reviewer-owned.

**Continue immediately to F:** yes.

### F — Next VPS-unlock runtime seam

**Status:** `READY_LOCAL_AFTER_E`; proposal authority applies.

Prefer, in order:

1. authenticated endpoint/source migration if the owned environment can create a genuine endpoint change without production-route mutation;
2. integrate existing authenticated PLPMTUD state into a live probe/ACK path without trusting unauthenticated ICMP;
3. another actual `BLOCKED_IMPLEMENTATION` release row with a short path to self-owned VPS evidence.

Do not choose FEC, 0-RTT, striping, heterogeneous aggregation, or exotic carriers merely for queue depth.

**Continue immediately to G:** yes.

### G — Local proof + one bounded VPS observation for F

**Status:** `PREAUTHORIZED_AFTER_F_GREEN`.

Local real-socket proof -> exact-head green -> one minimal materially distinct self-owned VPS observation if truthfully `READY_LIVE`; preserve negative evidence and cleanup; no unchanged retry.

### H — Repeated warm-failover diagnostic fallback

**Status:** `READY_LOCAL_FALLBACK`; only if A/F are genuinely blocked or while unrelated exact CI is pending.

Do not rerun `9fd2411` or `a117086` unchanged. The next useful local change is bounded sanitized inner-collector failure categorization in outer structured evidence, followed by synthetic/dry verification. Only after changed instrumentation may one materially changed VPS attempt be considered. Keep this behind missing runtime capabilities so harness work does not become the project again.
