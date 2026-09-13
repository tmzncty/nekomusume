# ChatGPT reviewer handoff — reliable-UDP R7 advancing; two HIGH integration findings remain

## Reviewed repository truth

- Current default-branch developer HEAD reviewed: exact `50c4a9e69d479c34085217be2360fb3981084398` (`refactor(carrier): drop dead raw_health_sample after interval-delta bridge`).
- Since the previous reviewer handoff at `f7fdd1f`, the developer landed substantive source/test work, not docs churn:
  - `e19cd6d` promotes the coherent socket-agnostic `ReliableUdpRuntime` orchestrator into `neko-carrier`;
  - `a19d1a4` closes the prior rustfmt stable-gate failure and `cec861f` persists developer-local exact-tree provenance for that repaired R7 tree;
  - `0bd4270` separates stable `FrameId` from fresh packet number and changes manager-facing health from lifetime-cumulative loss to an interval delta;
  - `50c4a9e` removes the now-dead raw health helper.
- Current GitHub-hosted `stable checks` and `nightly decode fuzz smoke` are both green on exact `50c4a9e`. These are cross-evidence only.
- There is **no persisted developer-local clean exact-tree provenance for exact `50c4a9e` yet**. The newest persisted developer-local source-tree gate remains earlier (`a19d1a4` / `cec861f`). Final repaired R7 acceptance still requires a new pushed exact-tree local gate/provenance.
- No open PRs and no new VPS/WAN experiment in this sequence.
- Governance remains unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`; D019, `SessionRuntime.events`, RSEC-001, signing/SBOM and release authority remain separate policy/security gates.
- `READY_LIVE` remains none until the coherent local runtime is corrected, independently challenged, and promoted into the executable CLI/lab path.

## Reviewer verdict on new developer work

### BLOCKER-GATE-001 — CLOSED

The prior current-head red `scripts/check.sh` was reproduced as a rustfmt formatting failure in the R7 runtime test. `a19d1a4` makes the smallest formatting repair; `cec861f` records the clean exact-tree developer-local gate. Current hosted stable/fuzz checks are also green at `50c4a9e`.

Do not reopen this unless a new gate fails.

### H-RUDP-007 stable FrameId / replacement ownership — DIRECTION ACCEPTED, closure incomplete because of H-RUDP-011

`0bd4270` fixes the central identity error from the previous handoff:

- packet number / AEAD nonce is fresh per transmission;
- `FrameId` is supplied independently and remains stable across replacement packets;
- `packet_frames` records real packet -> frame ownership instead of reconstructing `FrameId(packet_number)`;
- `retransmit_frames` no longer causes immediate plaintext release;
- `pto_probe()` returns `(FrameId, plaintext)` and `on_retransmit_sent()` records the same stable frame under a new packet number;
- a frame is released after packet retirement only when recovery reports no outstanding copy and it is not currently scheduled for retransmission.

The existing regressions deliberately use FrameIds different from packet numbers and retain overlapping copies until final retirement. This closes the **identity/copy-count logic** of H-RUDP-007.

However, the integrated retained-plaintext owner is still not fail-closed or truly bounded; H-RUDP-011 below therefore blocks full ownership acceptance and R8.

### H-RUDP-001C lifetime-loss replay — PARTIALLY CLOSED; new H-RUDP-001D found

`0bd4270` correctly stops the previously identified pattern where an old cumulative lifetime loss ratio is reclassified as bad on every later clean resolved outcome. A clean outcome after old loss can now produce zero new-loss evidence and reset the bad streak.

But the new delta uses `packets_sent since last health consumption` as the denominator. That denominator is not aligned with when old packets are later **resolved** lost. A PTO sample can consume the send denominator before those packets are declared lost; a later ACK/loss outcome can then have `delta_sent = 0` and `delta_lost > 0`, causing `checked_div(0).unwrap_or(0)` to report **zero loss for a real newly resolved loss event**. H-RUDP-001D below must close this before automatic fallback is accepted.

## New / still-open findings

### H-RUDP-011 — retained plaintext capacity is bypassable and send ownership is non-transactional

**Severity: HIGH for resource/retransmission correctness; blocks R7 acceptance and R8.**

Current `ReliableUdpRuntime::on_packet_sent` does:

1. `recovery.on_sent(...)` — mutates sent/recovery/Reno state;
2. `let _ = self.retransmit.track(frame, frame_plaintext)` — **discards Capacity / FrameTooLarge / Conflict**;
3. unconditionally clones the plaintext into a separate `frame_plaintext: BTreeMap<FrameId, Vec<u8>>`.

This creates two independent plaintext owners. `RetransmitBuffer` is explicitly bounded to 64 frames / 8192 bytes, but the second `frame_plaintext` map has no corresponding byte/frame cap. A too-large frame or full retransmit buffer can therefore fail `track()` while the runtime still records the packet as sent and still stores an unrestricted plaintext copy. That is both a resource-bound bypass and an inconsistent recovery state: a packet can exist in recovery even though the authoritative bounded retransmit owner rejected it.

`on_retransmit_sent` similarly records a replacement packet without proving at the API boundary that the stable frame is still retained.

Required repair contract:

- use **one authoritative bounded plaintext owner**; prefer removing the duplicate `frame_plaintext` map and fetching probe bytes from `RetransmitBuffer`, unless a second map has a demonstrated non-duplicative purpose;
- never ignore `RetransmitBuffer::track` errors;
- initial-send ownership must be transactional/fail-closed: capacity/conflict/oversize rejection leaves no sent recovery packet, no Reno charge, no packet->frame map entry, and no leaked plaintext;
- if plaintext reservation happens before `recovery.on_sent`, roll it back exactly when the send-record step rejects; do not release an already-existing identical frame accidentally;
- a replacement send must require an existing retained stable frame (or equivalent explicit ownership proof) before recovery records a new copy;
- no public API path may create an outstanding recovery copy whose plaintext cannot be resealed if later scheduled;
- teardown must clear the single bounded owner and packet->frame map deterministically.

Mandatory regressions:

1. frame larger than plaintext bound -> rejection is atomic; recovery in-flight/bytes-in-flight/packet map/retained bytes all unchanged;
2. max-frame/max-byte capacity first excess -> same atomic rejection;
3. same FrameId + conflicting bytes -> same atomic rejection;
4. recovery-side rejection after a successful plaintext reservation rolls back only the newly reserved ownership;
5. replacement for an unretained/retired FrameId fails before recording a packet;
6. successful original + overlapping retransmit still retains exactly one plaintext copy and releases it only after authoritative final retirement;
7. repeated failure cannot make retained plaintext exceed configured bounds.

Do **not** fix this by inventing larger limits; preserve the existing bounds.

### H-RUDP-001D — health loss delta is keyed to sends, not resolved outcomes

**Severity: HIGH for automatic degradation/fallback correctness.**

`fresh_health_sample()` currently computes:

`delta_sent = packets_sent - last_health_sent`

`delta_lost = packets_lost - last_health_lost`

and reports `delta_lost / delta_sent`.

That only works when send and loss-resolution intervals coincide. They need not.

Counterexample that must become a regression:

1. send several packets;
2. fire PTO and consume the fresh health sample — this advances `last_health_sent` to include those still-unresolved packets;
3. later receive an ACK that newly declares some of those old packets lost, with no intervening send;
4. now `delta_lost > 0` but `delta_sent == 0`; current code reports `loss_per_mille = 0` even though the newly resolved outcome contains real loss. If the ACK also resets `pto_count`, the manager-facing sample can look entirely clean.

Required repair:

- manager-facing loss must be based on **newly resolved packet outcomes**, not send-time deltas;
- mechanically acceptable shape: accumulate `resolved_packets` and `resolved_lost` from `RecoveryResult.acked_packets + lost_packets` and consume deltas from those counters; or aggregate equivalent per-outcome numerators/denominators until the next health sample;
- a PTO-only outcome may produce PTO health evidence with zero resolved packet-loss denominator; it must not consume future loss denominator for still-unresolved packets;
- keep lifetime sent/lost counters separately for observability/diagnostics;
- preserve existing thresholds/hysteresis and same-epoch/bare-send freshness rules.

Mandatory regressions:

1. send -> PTO health sample -> later ACK declares old packets lost with no new send: fresh sample reports non-zero newly resolved loss;
2. one old bad resolved outcome followed by genuinely clean resolved ACK outcomes does not replay old loss;
3. ACK with 1 acked + N newly lost packets uses exactly that newly resolved set as the interval denominator;
4. PTO-only samples remain distinct and cannot erase a later loss outcome;
5. same-epoch polling and bare sends still emit none;
6. packet feedback still cannot validate a path or confirm Session delivery.

### H-RUDP-008 — stable Session logical identity / receiver dedup remains open

**Severity: HIGH before executable failover/live claims.**

The coherent reliable-UDP runtime still needs a stable logical Session data identity independent of fresh packet numbers. Reuse existing committed Session / `ProcessMessage::Data` / stream+offset / manager logical-range semantics rather than inventing packet-number delivery identity.

Required behavior remains:

- retransmission preserves logical identity while packet number/nonce changes;
- replacement-first then late-original produces one application-visible logical delivery, explicit duplicate accounting, no conflict;
- conflicting bytes for the same logical identity fail closed;
- packet ACK is never Session delivery confirmation.

### M-RUDP-009 — actual cwnd/pacing send admission remains open

**Severity: MEDIUM now; blocks executable R8/WAN evidence.**

Current `ReliableUdpRuntime::on_packet_sent` records/charges a send directly and does not itself enforce `can_send()` or a deterministic pacing deadline. The isolated Reno test is not enough.

Close after H-RUDP-011 so ownership ordering and congestion admission are solved together:

- actual runtime send admission checks cwnd before any socket/recovery send record;
- pacing is represented as a deterministic next-send/deadline decision, not hidden sleep;
- cwnd refusal is atomic: no packet/recovery charge/plaintext ownership/socket effect;
- after ACK opens capacity, the same logical frame can proceed safely;
- do not change Reno policy values.

### M-RUDP-010 — automatic fallback transition errors/evidence remain open

**Severity: MEDIUM; blocks R8 evidence.**

Current `poll_health()` still discards `manager.fail(...)` and `manager.activate(...)` errors with `let _ = ...` and returns `WarmFallback` whenever health reaches Degraded and both path records merely exist. This can fabricate fallback success when TCP is not actually warm/activatable.

Required repair remains:

- return/propagate transition failure explicitly;
- report `WarmFallback` only after real fail + promotion success;
- expose/record the actual switch event separately from health evidence;
- deterministic invalid standby state must produce no successful switch event.

## Continuous READY_LOCAL queue — keep feeding the agent

Do not wait for the next reviewer between dependency-safe slices. The queue is intentionally deep; preserve still-valid later work rather than collapsing it after each repair.

### Q1 — close H-RUDP-011 single bounded transactional plaintext owner

Implement the exact contract/regressions above. This is first because H-RUDP-008/Q4 both depend on correct frame/plaintext ownership.

### Q2 — close H-RUDP-001D resolution-aligned interval health

Replace send-delta denominator with resolution-aligned accounting and add all six regressions. Preserve cumulative diagnostics separately.

### Q3 — close H-RUDP-008 stable Session logical identity + late-original dedup

Reuse existing logical Session identity; replacement-first + delayed original must deliver once, conflict fail closed, packet ACK remain separate.

### Q4 — close M-RUDP-009 actual congestion/pacing admission

Integrate cwnd + deterministic pacing decision into the coherent runtime with atomic ownership ordering.

### Q5 — close M-RUDP-010 explicit automatic switch result/evidence

No swallowed manager errors. Real switch result/event only after actual successful promotion.

### Q6 — R7 coherent local runtime re-acceptance

One integrated flow, not manual choreography, must prove together:

- real UDP socket send/receive in the coherent local fixture/path;
- authenticated fresh packet number / nonce;
- bounded canonical ACK state; no ACK storm / ACK-of-ACK;
- ACK/loss/PTO/retransmit with stable FrameId and one bounded transactional plaintext owner;
- replacement packets use fresh nonce but stable logical Session identity;
- actual cwnd/pacing admission;
- resolution-aligned fresh health evidence;
- warm TCP readiness;
- automatic manager fail/promote with explicit success/failure result;
- uncertain replay + receiver logical dedup/conflict handling;
- recovery / health / switch / Session delivery observability remain distinct.

Minimum scenarios: no-loss; isolated recoverable loss without fallback; ACK loss/PTO replacement; replacement-first + late-original dedup; one historical loss then clean recovery; PTO-sample-before-later-loss; sustained distinct bad resolved outcomes -> successful warm fallback; invalid warm target -> no fabricated switch; plaintext capacity refusal -> zero partial state.

### Q7 — final pushed exact-tree developer-local provenance

After Q1–Q6 reach one coherent source SHA:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --short
```

Record exact pushed SHA, UTC start/end, exits, OS/arch, stable Rust and clean initial/final tree. If wire/parser/framing changes again, also run the repository-pinned decode fuzz build/run; pure carrier/session/runtime-only work does not require a mechanical fuzz rerun. Hosted CI remains extra cross-evidence.

### Q8 — independent bounded R7 review

Challenge the final exact source tree specifically for:

- plaintext/resource transactional ownership;
- resolved-outcome health math;
- stable logical identity/dedup;
- ACK/Session evidence separation;
- cwnd/pacing admission;
- automatic-switch truthful result/evidence;
- nonce freshness/retransmit semantics;
- bounded retained state.

Concrete defect -> smallest repair + regressions + new exact-tree provenance, then continue. No defect -> precise no-finding note.

### Q9 — R8 executable CLI / bounded lab path

Only after R7 re-acceptance. Reuse the same repaired `ReliableUdpRuntime` / Session components in existing CLI/failover/lab machinery; no second test-only implementation. Add bounded deterministic sender-side packet suppression/loss injection and structured recovery/health/switch/Session-delivery counters.

### Q10 — R9 process/socket acceptance

At minimum cover: no-loss; one recoverable packet loss; ACK loss; PTO replacement; replacement-first + late original; no duplicate logical delivery; conflicting logical duplicate fail-closed; sustained packet-recovery degradation -> automatic warm TCP fallback; invalid standby -> no false switch; cleanup/listener/process residue zero.

### Q11 — release/status reconciliation and READY_LIVE decision

Only after Q9/Q10. Update `docs/status.md`, `IMPLEMENTATION_PLAN.md`, release packet and handoff truthfully. If the executable path now creates a materially new real-network question, classify an exact READY_LIVE row. Do not inherit old application-reply-cessation evidence as proof of packet-recovery fallback.

### Q12 — changed-hypothesis self-owned VPS evidence

Only if Q11 creates READY_LIVE and standing authorization covers the exact run. Use the minimum bounded profile that distinguishes:

- real packet recovery without fallback;
- real packet loss/PTO causing truthful UDP degradation;
- same logical Session warm TCP promotion;
- uncertain replay/dedup;
- duplicate/lost application bytes;
- failure-decision-to-first-resumed-data timing;
- CPU/RSS/FD/socket observations when available;
- cleanup.

This is a genuinely new hypothesis/instrumentation path, not a same-class rerun of historical reply-cessation/warm-failover negatives.

### Q13 — item-3/item-4 evidence reconciliation after new live result

Preserve negative evidence and exact boundaries. Do not change RC/freeze/release/production flags automatically. Refill the next engineering queue from what the new evidence actually exposes.

## Separate policy/security gates — do not let them stall Q1–Q13

Still unresolved and unchanged:

- `SessionRuntime.events` retained-state policy;
- D019 source-retention/no-reset policy;
- RSEC-001 representative adversarial-load/capacity suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen release / final independent security-release judgment;
- RC/freeze/release/production authority.

Do not invent these decisions while implementing reliable-UDP runtime integration, but do not use them as a reason to idle on the dependency-safe queue above.

## Stop conditions

Stop only for a newly discovered unresolved BLOCKER/HIGH that prevents safe continuation, a required core Session/Carrier/ACK/crypto/wire architecture decision not already determined by committed semantics, destructive migration, action beyond standing authorization, production/third-party/new credentials, or actual repository/tool breakage.

Otherwise: implement -> focused tests -> full required local gate -> commit/push -> immediately continue to the next dependency-ready slice.