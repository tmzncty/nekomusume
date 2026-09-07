# Nekomusume ChatGPT Handoff

Checked at: 2026-09-08 01:59 Asia/Shanghai
Repository main HEAD reviewed: `760681287ef47d9369e69d322b5bd838c2ae5b50`
Previous reviewer handoff commit: `760681287ef47d9369e69d322b5bd838c2ae5b50`
Current implementation branch: `work/e1a-staged-accounting-20260907` at exact `655df00b0b6439339ffccdf9a0962f5bb1c66b82`
Previous checked implementation HEAD: `655df00b0b6439339ffccdf9a0962f5bb1c66b82`
Historical partial-E2 branch retained: `work/continue-20260904` at `d271a99a2ab26abbcb146c411ba0fde697395abe`

## What changed

No new coding commit has appeared after exact `655df00`. The current implementation branch remains on the accepted E2 guard closure while `main` advanced only by the previous reviewer handoff `7606812`.

Exact CI remains green on both relevant tips:

- implementation exact `655df00` Rust CI run `34137385366` — `success`;
- reviewer/main exact `7606812` Rust CI run `34141124437` — `success`.

The implementation branch is behind current `main` only by the reviewer-owned handoff lineage. There is no CI, repository, standing-VPS-authorization, credential, environment or known core-architecture blocker preventing independent local work.

Two expected coding-agent continuation opportunities have now passed after the E2 closure handoff without a new coding checkpoint. This is therefore a coordination signal, not a new protocol finding. The previous B/C/D sequence was safe but too easy to interpret as three separate audit tickets that could each end with “no change.” That is not the intended execution semantics.

The queue is refreshed to remove that ambiguity: compatibility/package/release-matrix review is one **non-stopping outward closure sprint**. If an audit finds no defect requiring a commit, the agent must immediately continue within the same run. Unless exact-current evidence exposes a different genuinely smaller ready runtime seam, the default next implementation target is now **live synchronized key update over the existing periodic authenticated TCP Session**.

This default is selected because current code already has a real `SecureSession::update_key_phase()` primitive that rekeys both directions, advances authenticated `key_phase`, resets send nonce/replay state, rejects a second update, and has deterministic synchronized/unsynchronized fixture tests. The CLI still advertises `key-update` only as a fixture. The periodic runtime already provides one bounded authenticated TCP Session and repeated logical exchanges, so a fixed exchange-boundary update can become real-socket evidence without inventing a new wire message, new crypto primitive, new ACK semantics or new security-policy number.

## Review verdict

**CONTINUE_OUTWARD_WITH_DEFAULT_RUNTIME_TARGET — E1A/E2/C1 remain locally closed. Do not reopen pre-auth checker work. Treat B/C/D as one read-only/repair-if-needed sprint that may not stop merely because it produces no diff. Then implement live synchronized key update as the default runtime seam, take it through exact-head green CI, and—if the resulting declared question is READY_LIVE—run one bounded self-owned VPS experiment under standing authorization.**

C2 terminal-source retention remains an isolated release/security policy checkpoint. It does not block this queue. No administrator action is required to continue.

## Reviewer findings and retained boundaries

### RSEC-001E2-GUARD — CLOSED at `655df00`

Retain the accepted machine-checkable responder-ordering guard. Existing-pending UDP input is charged before duplicate classification and Noise processing; pending queue ownership remains reserve-before-store with terminal cleanup. Do not add another inventory/checker framework absent a concrete regression.

### RSEC-001E2-RUNTIME — CLOSED at `b041a64`

Retain one conservative charge for a matched existing-pending UDP datagram before duplicate-vs-Noise classification, without a second duplicate/non-duplicate charge.

### RSEC-001E1A — CLOSED at `164731d` / current implementation lineage

Retain staged one-logical-record TCP accounting across ordinary, periodic, multistream and failover responder handshakes. Header charge precedes attacker-controlled length interpretation; body reservation precedes allocation/read; malformed/incomplete ownership terminalizes; one frame remains one input packet/record.

### RSEC-001C1 — CLOSED at `f7e2cf1`

Carrier-aware source projection remains accepted and bounded; TCP/UDP pre-auth source domains do not accidentally alias.

### RSEC-001C2 — POLICY CHECKPOINT / RELEASE-SECURITY LIMITATION at `f066af5`

`docs/adr/m1-g0-preauth-source-retention-amendment-request.md` remains unresolved. Do not invent TTL/LRU/history/epoch/eviction numbers. Literal terminal-source no-reset compliance remains unclaimed.

If a later release/security conclusion requires an immediate disposition before a new retention policy is approved, the already pre-authorized conservative disposition is: preserve bounded in-process cleanup, explicitly document that literal D019 terminal-source no-reset is not implemented, and keep that as a release/security limitation rather than inventing a retention mechanism. This does not authorize claiming D019 compliance.

### OUTER-STALL-001 — COORDINATION — audit-only steps must not become stopping points

No implementation change has landed since E2 guard closure even though the repository and exact-head CI are healthy. B/C/D are therefore collapsed operationally: an audit that finds “no defect” is a result, not a reason to end the coding run. The same invocation should continue to the release-matrix decision and then the runtime seam.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- A1 response-I/O deadline, B1 queue ownership/expiry, D1 terminal rejection, E1A staged TCP accounting, E2 pending-UDP runtime/guard ordering and C1 carrier projection are accepted bounded implementation findings.
- C2 remains unresolved policy text and a release/security limitation; no literal terminal-source no-reset claim is allowed.
- Exact `655df00` has green CI but adds no WAN/VPS/performance evidence.
- `SecureSession::update_key_phase()` plus its local fixture tests are implementation/fixture evidence only. They do not yet prove a key update occurred inside a real authenticated socket Session.
- A future fixed-boundary periodic key-update run would prove only that both self-controlled peers can synchronously rekey an already authenticated bounded Session under that experimental schedule. It would **not** prove dynamic rekey negotiation, arbitrary peer-initiated update signaling, production key-rotation policy, public reachability or reliability rate.
- Historical WAN/HY2/failover/periodic positive and negative evidence remains immutable at exact commit boundaries.
- Standing VPS authorization remains valid; C2 is not a generic WAN blocker.
- Protected identity material, SSH private keys, credentials, private endpoint material and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a multi-hour pre-authorized queue. Coordination cadence is reviewer `:00`, coding agent expected wake/resume `:20`; neither clock is a work-duration limit. If work is already running, do not interrupt it. Finish a coherent package -> required gates -> commit -> push -> immediately consume the next dependency-satisfied package. No-diff audit result, one commit, one nominal hour, reviewer interval, or CI pending state is a stop condition by itself.

### Q0 — Reconcile reviewer-only branch lag

**Status:** `READY_LOCAL`; coordination only.

Fetch current `origin/main` and normally merge/reconcile `7606812` (and this newer handoff if present) into `work/e1a-staged-accounting-20260907`. Preserve all implementation history; no force-push. `docs/CHATGPT_HANDOFF.md` remains reviewer-owned/read-only to the coding agent.

If already reconciled, skip immediately.

**Continue immediately to A:** yes.

### A — Outward closure sprint: compatibility + package/provenance + release matrix

**Status:** `READY_LOCAL`; independent of C2. **This is one sprint, not three stopping tickets.**

**Goal / why now:** verify the old release-facing assumptions against the exact current implementation quickly, then select executable runtime work instead of returning to review-only mode.

#### A1 Compatibility/freeze

Audit current/current negotiation, unsupported/future rejection before data admission, exact selected-version transcript binding into Noise, resume/version/replay binding, and corpus-v1 content-addressed freeze vs global protocol non-freeze.

Add a regression/fix only for a concrete mismatch. Do not reopen frozen corpus bytes without correctness evidence. If all existing tests/contracts are sufficient, record that conclusion in the eventual closure/release update only if useful and continue immediately—do not stop to report “no changes.”

#### A2 Package/operator/provenance

Verify the existing bounded x86_64 package lifecycle evidence, install/readiness/smoke/upgrade/rollback mechanism, shutdown/listener/temp cleanup, canonical Git-blob/checksum manifests, exact-head CI references and stale release-packet links. Do not read protected identity material and do not rerun VPS/package work merely for freshness.

Distinguish “package lifecycle mechanism already demonstrated” from “exact-current release binary packaged.” Do not promote the former into the latter.

If no concrete defect exists, continue immediately.

#### A3 Release-opportunity recomputation

Re-evaluate every remaining release/evidence row from exact-current code/evidence:

- bounded question already answered -> `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`;
- executable specific missing assertion with dependencies satisfied -> `OPEN_READY` / `READY_LIVE` with exact `evidence_needed`, `next_action`, `requires`, `execution_scope`;
- missing implementation/environment/governance/review dependency -> exact blocker;
- C2 is only its specific source-retention release/security limitation;
- standing-authorized self-owned work must not be called “need WAN authorization.”

If a genuine `READY_LIVE` row already exists and has higher evidence value than the default live-key-update seam, it may go directly to C after recording the rationale. Otherwise continue immediately to B.

**Gate/commit:** no cosmetic commit is required merely for A1/A2 no-defect outcomes. If A3 changes an authoritative release matrix/status file, commit/push that coherent reclassification. Whether or not a commit is needed, **continue in the same run**.

**Continue immediately to B unless a higher-value READY_LIVE row is already selected:** yes.

### B — Default runtime seam: live synchronized key update in periodic TCP

**Status:** `READY_LOCAL_AFTER_A`; default choice unless A3 identifies a materially higher-value already-READY live row or exact-current code reveals a smaller dependency-safe seam.

**Goal / why now:** turn the existing synchronized `SecureSession::update_key_phase()` fixture/state transition into a real authenticated socket Session capability and immediately make it eligible for bounded VPS evidence.

**Existing primitives to reuse:**

- `SecureSession::update_key_phase()` rekeys outgoing/incoming Noise transport state, increments authenticated record-context `key_phase`, resets send nonce to zero and resets replay state;
- deterministic tests already prove synchronized update accepts new-phase data and rejects stale old-phase data, while unsynchronized update fails closed;
- `periodic-server` / `periodic-client` already run one bounded authenticated TCP Session with repeated request/DeliveryAck exchanges and deterministic count/duration limits.

**Preferred minimal design:** add an explicit bounded periodic experimental option such as `--key-update-after N` (exact spelling may differ) that both controlled endpoints receive from the harness/command line. `N` is an exchange boundary, not a new protocol/security policy. Require `1 <= N < count` when enabled. After exchange/ack `N` is successfully completed, both peers synchronously call `update_key_phase()` before the next application record. The server may commit immediately after sending the Nth authenticated ack; the client commits after successfully receiving/validating that ack; the next client record must therefore use phase 1.

Equivalent local design is acceptable if it is smaller and preserves the same invariants. Do **not** add a new wire control frame merely to make the experiment convenient.

**Protected invariants / evidence boundary:**

- one already-authenticated Session remains the same Session across the phase change;
- negotiation transcript, identity authorization, Session delivery/ACK semantics and Carrier architecture do not change;
- no 0-RTT or unauthenticated control path;
- no old-phase record accepted after commit;
- no second update beyond current `MAX_KEY_PHASE` support;
- an endpoint missing/misplacing the scheduled update fails closed rather than silently resynchronizing;
- diagnostics expose only secret-free phase/event/count information;
- the fixed experimental schedule is not claimed as a production rekey protocol or dynamic negotiation mechanism.

**Deterministic local tests:**

- disabled option preserves existing periodic behavior;
- invalid boundary 0 / `>= count` rejected;
- count around boundary: records before update phase 0, next record phase 1;
- exactly one update on each side;
- stale phase-0 ciphertext injected/replayed after boundary rejected;
- intentionally unsynchronized boundary fails closed and does not fabricate DeliveryAck/success;
- second update attempt rejected if directly exercised;
- setup/ack timeout and shutdown cleanup remain bounded;
- structured summary/diagnostic accurately states update attempted/committed and phase.

**Local proof:** bounded loopback/process test using real TCP sockets, one authenticated Session, records on both sides of the update boundary, no duplicate/missing application records, and cleanup verified.

**Gate:** targeted tests + `scripts/check.sh` + `git diff --check`; fuzz only if untrusted parser/wire decode changes (the preferred design should not require that). Commit and push. Check exact-head CI.

**Continue immediately to C after exact-head green CI:** yes.

### C — One bounded self-owned VPS live-key-update evidence run

**Status:** `PREAUTHORIZED_AFTER_B_GREEN_CI` if the implemented runtime question is truthfully READY_LIVE.

Use `docs/standing-vps-lab-authorization.md` and the existing local secret endpoint configuration. Do not read/print/copy private key material; use it only through the SSH client as already authorized. Verify the expected owned endpoint before deployment using non-secret host/UID identity checks.

Run the smallest periodic TCP profile that puts authenticated application exchanges on both sides of exactly one scheduled key update, comfortably within standing ceilings. No benchmark/load objective is needed.

Collect/retain:

- experiment ID;
- exact git commit and binary hash/size;
- exact count/bytes/duration/update boundary;
- client/server authentication and key-update structured events;
- application records/DeliveryAck evidence before and after phase change;
- duplicate/missing/conflict counts if available;
- bounded CPU/RSS/FD/socket observations when existing sampler makes them cheap;
- explicit listener/process/temp cleanup verification.

A positive run is a bounded single observation, not a reliability rate. A negative run must identify the exact failure stage; do not unchanged-retry.

**Continue immediately to D after cleanup:** yes.

### D — Reconcile live-key-update result

**Status:** `PREAUTHORIZED_AFTER_C`.

Update the release matrix/status/evidence at the same semantic boundary. State precisely what the run proves and does not prove. If the run exposes a real runtime defect, fix that defect first and only rerun when code/instrumentation/hypothesis has materially changed.

If the declared bounded question is answered, classify it sufficient rather than scheduling freshness reruns.

**Continue immediately to E:** yes.

### E — Next runtime/VPS opportunity

**Status:** `PREAUTHORIZED_AFTER_D`.

Recompute remaining historical `BLOCKED_IMPLEMENTATION` candidates. Preferred next families, in value/dependency order determined from exact current code, are:

- migration-back/recovery real socket path;
- endpoint/path migration if the owned environment can actually produce a meaningful path/source change;
- live PMTUD integration;
- another distinct release row already made READY by recent runtime work.

Compare 1–3 minimal shapes and select one without waiting if it does not change core Session/Carrier/ACK/crypto/wire semantics or invent policy. If one candidate requires maintainer architecture choice, select another independent candidate instead of stopping the project.

**Continue immediately to F when safe:** yes.

### F — Second bounded VPS question, only if materially distinct

**Status:** `PREAUTHORIZED_AFTER_E_GREEN_CI`.

Execute only if E creates a genuinely distinct dependency-ready question. Same standing authorization/evidence/cleanup rules as C. Never use many nominally different runs to bypass the 10-minute single-experiment ceiling.

**Continue immediately to G:** yes.

### G — Independent D019/security debt while C2 waits

**Status:** `READY_LOCAL_FALLBACK`; must not displace a READY runtime/VPS lane.

Cover only genuinely missing boundaries independent of terminal-source retention: concurrency ceilings, global windows, memory/queue/response/anti-amplification, idle/lifetime/100 ms response deadline, cancellation/double cleanup and no-success-evidence barriers. Reuse existing tests; add only distinct missing boundaries. Do not reopen staged responder/checker work.

Full D019/RSEC-001 compliance remains unclaimable while C2 is deferred.

### H — Exact-tree release/security navigation refresh

**Status:** `READY_AFTER_VISIBLE_RUNTIME_PROGRESS`.

After at least one new runtime/VPS bounded question is closed, reconcile `docs/reviews/resource-abuse-evidence-2026-09-04.md`, `docs/release-security-review-packet.md`, `docs/status.md`, ROADMAP/IMPLEMENTATION_PLAN stale labels and exact-head CI/evidence references. Do not use this documentation pass as a substitute for runtime progress.

Never promote RC/production/freeze/release automatically.

## 24–48 hour output check

Recent work produced genuine engineering closure: staged TCP accounting, all responder migrations, terminal ownership/cross-layer repair, carrier-aware source projection, isolated C2 policy tension, pending-UDP runtime ordering and the exact machine guard. That phase is done.

The next visible-output target is now explicit: **real authenticated periodic TCP Session key update, then one bounded self-owned VPS observation**. If the coding branch remains unchanged through another continuation window with healthy CI and no newly discovered technical blocker, treat that as external executor inactivity rather than producing another reviewer/checker layer.

## Completion gates for this outward phase

This phase has made meaningful progress when all are true:

- A exact-current audit/reclassification is performed without turning no-diff results into stop points;
- one historical fixture-only or implementation-blocked capability becomes a real runtime path;
- exact implementation HEAD is green;
- at least one newly enabled bounded self-owned VPS question is executed or truthfully classified with a concrete environment blocker;
- result provenance and cleanup are retained;
- no bounded single run is inflated into reliability/public/production/performance claims;
- C2 limitation remains honest and isolated;
- governance flags remain unchanged.

## Do not expand into

- another pre-auth responder checker/inventory framework absent regression;
- new terminal-source TTL/LRU/history/epoch/eviction numbers without reviewed policy;
- a new key-update wire/control protocol merely to run the bounded fixed-schedule experiment;
- public or production listener deployment;
- core Session/Carrier/ACK/Noise redesign unrelated to an observed blocker;
- unchanged historical HY2/repeated-failover/periodic retries;
- VPS load/stress testing as a substitute for a declared bounded question;
- speculative FEC/0-RTT/striping/multipath/exotic-carrier work without an observed problem;
- reading, hashing, copying, uploading or committing protected identity/SSH-secret material;
- release/RC/freeze/production promotion.

## Questions requiring maintainer decision

**None for the active outward queue.**

D019 terminal-source retention remains a deferred release/security policy decision. The active queue must continue without it. If a future release gate cannot proceed without a disposition, use the already authorized conservative limitation/deferral unless a reviewer/maintainer deliberately chooses to design and approve a new bounded retention policy.
