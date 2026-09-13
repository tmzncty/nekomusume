# ChatGPT reviewer handoff — R1–R6 accepted; R7 integration has blocking correctness findings

## Reviewed repository truth

- Current default-branch developer HEAD reviewed: exact `a20eb72ab14b2127a9056445bb2a79fec785844a` (`test(carrier): automatic health-driven fallback in runtime fixture (Q5/R7)`).
- This is a substantial implementation sequence, not documentation churn: `main` advanced 30 commits from reviewer phase-opening `b505a60`; `neko-carrier` gained the path-local reliable-UDP owner plus ACK tracking/retransmit/runtime/failover/observability tests, and `neko-wire` gained bounded ACK/packet grammar + decode fuzz coverage.
- No open PRs.
- Developer-local clean exact-tree provenance exists for exact `92557ea` after the Q1–Q3 R1–R6 repairs (`docs/local-gate-92557ea-20260913.md`): `scripts/check.sh`, `git diff --check`, clean tree, timestamps/host/Rust all recorded green.
- Current exact `a20eb72` does **not** have equivalent persisted developer-local exact-tree provenance.
- GitHub-hosted current-head evidence is split: `nightly decode fuzz smoke` is green; `stable checks` is **red**, with failure in `bash scripts/check.sh`. Hosted CI is cross-evidence only, but a current-head red stable gate is still a real repository blocker and must be reproduced locally rather than ignored.
- No new VPS/WAN experiment occurred. `READY_LIVE: none` remains authoritative until the repaired executable surface passes bounded independent review.
- Governance remains unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`; D019, `SessionRuntime.events`, RSEC-001, signing/SBOM and release authority remain separate policy/security gates.

## Acceptance of Q1–Q4 / repaired R1–R6 primitives

### H-RUDP-001A/B public freshness bypass / bare-send replay — ACCEPT NARROWLY at `c8193b7`

The prior raw public `health_sample()` bypass is gone: manager-facing code can only consume `fresh_health_sample()`, while raw cumulative counters are private/diagnostic. `on_sent` no longer advances the health outcome epoch, so bare sends and repeated same-epoch polling cannot replay one historical bad snapshot.

This closes the exact A/B findings, but does **not** close health semantics overall; new H-RUDP-001C below is an integration-level consequence of continuing to classify each fresh outcome using cumulative lifetime loss.

### H-RUDP-005 pending ACK / ACK-of-ACK — ACCEPT at `4270f82`

`PacketAckTracker::observe_packet(..., ack_eliciting)` separates range observation from response obligation, and `take_ack()` consumes at most one pending emission until new eligible evidence arrives. ACK-only packets do not schedule ACK-of-ACK. The coherent runtime uses `take_ack`, not repeated `build_ack`, for emission.

Keep `build_ack()` as a diagnostic/render helper only; do not introduce another runtime emission path that bypasses `take_ack()`.

### M-RUDP-006 final-copy query — ACCEPT AS PRIMITIVE at `92557ea`

`Recovery::frame_outstanding` / `PathRecovery::frame_outstanding` correctly expose whether at least one packet copy still carries a `FrameId`; focused overlap tests prove one copy retiring does not imply final retirement.

The primitive itself is valid. R7 currently uses it incorrectly/incompletely; that is H-RUDP-007 below, not a rejection of M-RUDP-006.

### R2/R3/R4 prior repairs — ACCEPT

- authenticated `SecureSession` sequence is the one packet-number source (`badaafd`);
- vacuous ACK negative assertion is replaced by real canonical/overlap rejection (`9890bc0`);
- Reno releases only per-packet actually charged bytes (`3739138`).

The exact `92557ea` developer-local clean gate closes the repaired R1–R6 primitive layer. Do not rerun/rewrite those reviews unless current code changes their owners.

## R7 / Q5 verdict — NOT ACCEPTED

The new coherent `ReliableUdpPeer` fixture is useful because it finally composes real UDP sockets, authenticated records, ACK state, recovery, health and manager fallback in one flow. That composition exposed several real integration bugs/gaps that the isolated S4–S9 primitive tests could not catch.

### BLOCKER-GATE-001 — current `a20eb72` stable gate is red

**Severity: BLOCKER for R7 expansion, not a release-policy question.**

Current hosted `stable checks` fails inside `bash scripts/check.sh`, while current fuzz smoke passes. The exact failing subcommand is not established by the available hosted metadata and must not be guessed.

Required action:

1. reproduce `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` on a clean exact `a20eb72` checkout/worktree;
2. identify the first actual failing command/output;
3. make the smallest repair; no unrelated cleanup;
4. run focused tests plus full `scripts/check.sh`, `git diff --check`, clean initial/final tree;
5. commit/push and persist exact-tree provenance.

Do not treat a green fuzz job as a substitute for the red stable gate.

### H-RUDP-007 — integrated retransmit plaintext ownership is wrong

**Severity: HIGH for reliable-UDP correctness; blocks R8/executable expansion.**

Current `ReliableUdpPeer::recv_once()` mishandles recovery output in two ways:

1. after `Recovery::on_ack` schedules `out.retransmit_frames`, the fixture immediately calls `self.retransmit.release(*f)` on those frames; but a `retransmit_frames` entry means the last outstanding packet copy was lost and the retained plaintext is exactly what is needed to create the replacement packet. Releasing it there destroys the retransmission source before it can be resealed;
2. `pto_retransmit()` retrieves old frame `F`, then calls `send_data()`, which assigns the new packet's number as a **new `FrameId`**. A retransmission copy must carry the same stable `FrameId F`; packet number/AEAD nonce changes, frame identity does not. Otherwise recovery's outstanding-copy accounting and `frame_outstanding(F)` no longer describe the actual copies.

There is a related invalid shortcut: ACK handling currently releases `RetransmitBuffer` by iterating `out.acked_packets` and converting each packet number to `FrameId(packet_number)`. Packet identity and frame identity are not generally equivalent; a packet may carry a stable frame whose ID differs from the packet number and may carry multiple frames.

Why S5 did not catch this: the standalone `lost_packet_retransmits_frame_level_and_delivers_exactly_once` manually constructs the retransmission with `frames: vec![FrameId(0)]`, correctly preserving the old FrameId. The coherent R7 helper instead routes through `send_data()`, which loses that identity and also pre-releases ACK-driven retransmit plaintext.

Required repair contract:

- separate stable `FrameId` ownership from fresh packet-number allocation;
- a replacement packet for frame F must record `frames: [F]` (or the same set of original stable frame IDs), while sealing under a fresh SecureSession sequence/nonce;
- `out.retransmit_frames` means retain/fetch/reseal, **not release**;
- retained plaintext may release only when recovery authoritatively says no outstanding copy remains **and** the frame is not currently scheduled for retransmission;
- do not infer releasable frame IDs from packet numbers;
- if the cleanest solution is an additional recovery result such as final-ACKed frame IDs, derive it from the existing authoritative copy map rather than introducing a second divergent ownership counter.

Mandatory regressions:

1. ACK-driven loss schedules F; plaintext remains present until a fresh replacement packet carrying **F** is recorded/sent;
2. PTO original+replacement overlap: ACKing one copy does not release F; final successful retirement releases exactly once;
3. loss of one copy while another remains neither releases F nor schedules an unnecessary second replacement;
4. late ACK of old copy after replacement cannot double-release/resurrect;
5. teardown clears retained plaintext deterministically;
6. packet number and FrameId intentionally differ in at least one regression so accidental `FrameId(packet_number)` coupling cannot pass.

### H-RUDP-001C — fresh outcome still replays historical cumulative loss into new bad streaks

**Severity: HIGH for automatic fallback correctness.**

Q1 fixed *event freshness*, but current `fresh_health_sample()` still returns `raw_health_sample()` based on lifetime cumulative `packets_lost / packets_sent`.

That means one historical loss burst can still be counted repeatedly as new bad manager observations whenever unrelated future **clean ACK outcomes** advance `outcome_epoch`. Example: an initial resolved outcome leaves cumulative loss 5/8 (=625 per mille) and counts one bad observation; a later clean ACK after one new send yields a fresh outcome but the cumulative ratio is still 5/9 (=555), so the same old losses count as a second bad observation and can cross `degrade_after=2` even though the new interval itself had zero loss. This is a different replay trigger from the already-fixed bare-send/same-epoch case.

Required repair:

- manager-facing loss evidence must represent the newly resolved observation interval/delta (or another mechanically equivalent non-replaying window), not reclassify the full lifetime loss numerator on every fresh outcome;
- keep cumulative totals available for diagnostics/observability if useful;
- do not invent new health thresholds;
- PTO semantics may continue to use the existing consecutive PTO state, but a clean ACK must be able to produce clean/progress evidence instead of inheriting historical lifetime loss forever.

Mandatory regressions:

1. one historical bad loss outcome -> exactly one bad manager observation;
2. multiple subsequent clean send+ACK outcomes with zero new loss do **not** advance the bad streak/degrade from that old loss;
3. a clean outcome resets/advances existing `CarrierHealth` progress semantics as currently defined;
4. distinct new loss/PTO outcomes still cross unchanged hysteresis when they truly occur;
5. same-epoch polling and bare sends still produce no observation;
6. packet feedback still cannot validate a path or confirm Session delivery.

### H-RUDP-008 — R7 has no stable Session logical identity/dedup path

**Severity: HIGH before executable failover/live claims.**

The coherent runtime currently encodes raw application bytes directly as `RecordType::Data`, opens them, and returns only the packet number. It does not carry a stable Session stream/offset/message identity through retransmission and does not invoke existing Session/manager delivery/dedup ownership.

A fresh retransmission necessarily has a new AEAD nonce/packet number. Therefore packet replay protection alone cannot deduplicate a late original packet versus its fresh replacement: both are distinct authenticated packets containing the same application bytes. S5's "exactly once" test avoids this by permanently discarding the original ciphertext; it does not prove late-original/retransmit dedup.

The repository already has stable Session identities (`ProcessMessage::Data { stream, offset, ... }`, `DeliveryLedger`, `LogicalRangeId`/ConcurrentCarrierManager uncertain ranges). Reuse an existing committed identity/evidence model rather than inventing a packet-number-as-delivery-ID shortcut.

Required R7 behavior:

- application data has a stable logical identity independent of packet number;
- retransmission preserves that logical identity while using a fresh packet number/nonce;
- receiver accepts first logical delivery and suppresses exact duplicate late original/replacement; conflicting bytes for the same identity fail closed;
- Session delivery confirmation remains separate from packet ACK;
- packet ACK must never call or imply Session `confirm_received` by itself.

Mandatory test: deliver the replacement first, then deliver the previously delayed original; application-visible delivery count remains one, duplicate accounting is explicit, and packet recovery/ACK evidence remains separate.

### M-RUDP-009 — congestion/pacing is not yet an actual runtime send gate

**Severity: MEDIUM now; correctness-critical before R8/WAN execution.**

S6 proves `PathRecovery::can_send()` and `pacing_interval_us()` in isolation, but current `ReliableUdpPeer::send_data()` does not call either before sealing/recording/sending. It unconditionally `seal -> recovery.on_sent -> retransmit.track -> send_datagram`.

Therefore Q5's coherent runtime does not yet compose the advertised congestion/pacing decision path.

Required repair:

- actual runtime send attempts must consult congestion admission before recording/sending an ack-eliciting packet;
- pacing must be represented as a deterministic deadline/next-send decision, not a hidden busy sleep;
- define fail-closed ownership ordering so buffer/recovery/socket failures cannot leave a sent-but-untracked packet or tracked-but-never-owned plaintext. Consuming an unused crypto nonce is acceptable if necessary for fail-closed ordering; nonce reuse is not;
- focused tests must hit cwnd refusal and prove no socket send/recovery charge occurs, then ACK opens capacity and the same logical frame can be sent safely.

Do not change Reno thresholds/values.

### M-RUDP-010 — automatic fallback errors are swallowed and switch evidence is incomplete

**Severity: MEDIUM; must close before R8 evidence.**

`poll_health()` currently checks only that UDP/TCP keys exist, then executes `manager.fail(...)` and `manager.activate(...)` with `let _ = ...`. A failed state transition is silently discarded even though the method still returns `Some(Degraded)`; this can make the caller believe automatic fallback was handled when it was not. The coherent fixture also records the health sample but does not persist the actual switch event through `Producer::record_switch` in that path.

Required repair:

- propagate/return transition failure explicitly; do not swallow `fail`/`activate` errors;
- only report/record automatic fallback success after promotion actually succeeds;
- record the real switch event separately from health evidence;
- negative test with TCP not warm/ready (or other deterministic invalid transition) must not fabricate successful fallback/switch evidence.

No new recovery policy values are needed.

## Continuous READY_LOCAL queue — high throughput

Do not wait for another reviewer between dependency-safe items. BLOCKER/HIGH repairs are front-loaded; after they are green continue directly into executable/runtime work.

### Q0 — reproduce and close current stable-check red

Clean exact `a20eb72`, identify the actual failing `scripts/check.sh` subcommand, smallest repair, focused/full gate, commit/push. If any Q1–Q4 repair naturally subsumes the failure, still record the original failure cause and final exact-tree green provenance.

### Q1 — close H-RUDP-007 retransmit ownership

Repair stable FrameId/plaintext lifetime across ACK-loss and PTO replacement paths. Add the six regressions above. Do not proceed to R8 while plaintext can disappear early or frame IDs change across copies.

### Q2 — close H-RUDP-001C interval health evidence

Separate cumulative diagnostics from manager-facing non-replaying interval outcome. Add the six regressions above. Preserve existing health thresholds/hysteresis.

### Q3 — close H-RUDP-008 stable Session identity/dedup integration

Reuse existing committed Session/logical-range identity. Prove replacement-first + late-original duplicate suppression and conflict fail-closed, with packet ACK separate from Session delivery evidence.

### Q4 — close M-RUDP-009 actual cwnd/pacing send admission

Make the coherent runtime exercise the real send gate and safe ownership ordering. No new congestion policy.

### Q5 — close M-RUDP-010 explicit automatic-switch result/evidence

No swallowed manager errors; switch event recorded only on successful promotion; deterministic negative path stays non-promoted.

### Q6 — R7 coherent local runtime re-acceptance

After Q0–Q5, one integrated flow must prove all of the following together, not by manual choreography:

- real UDP socket send/receive;
- authenticated fresh packet number / nonce;
- bounded ACK state with no ACK storm/ACK-of-ACK;
- ACK/loss/PTO/retransmit with stable FrameId + retained plaintext lifetime;
- fresh replacement packet images;
- actual cwnd/pacing send admission;
- interval/fresh health evidence;
- warm TCP readiness;
- automatic manager fail/promote with explicit result;
- stable Session logical identity, uncertain replay and receiver dedup;
- layered observability, with packet recovery / health / switch / Session delivery domains distinct.

Add at least: clean no-loss, isolated recoverable loss (no fallback), replacement-first + late-original dedup, ACK loss/PTO, sustained distinct bad outcomes -> automatic warm fallback, and invalid warm target -> no fabricated switch.

### Q7 — current repaired exact-tree developer-local gate/provenance

On final pushed source SHA after Q0–Q6:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --short
```

Record exact SHA, UTC start/end, exits, OS/arch, stable Rust and clean initial/final tree. Wire/parser changes require pinned decode fuzz; pure carrier/runtime-only repairs do not require mechanical fuzz reruns (hosted extra evidence remains separate).

### Q8 — R8 executable bounded lab path

Only after R7 re-acceptance. Reuse the repaired coherent components in existing CLI/failover/owned-lab machinery; do not fork a second implementation in test-only code. Add deterministic local sender-side packet suppression and structured recovery/health/switch/Session-delivery counters. Bound duration/count/bytes and cleanup.

### Q9 — R9 process/socket acceptance

At minimum: no-loss; one recoverable loss; ACK loss; PTO replacement; late original; no duplicate app delivery; cwnd block/release; no ACK storm; no cumulative-loss health replay; automatic fallback only after distinct bad outcomes; invalid/not-warm TCP does not promote; cleanup/no listener residue.

### Q10 — independent bounded R10 re-review

Challenge exact executable pushed tree, not only primitive unit tests. BLOCKER/HIGH -> repair immediately and rerun gate. No finding -> precise review note with exclusions and exact tested anchor.

### Q11 — create exactly one new READY_LIVE question

Only after Q10. A valid next live question is now scientifically new because implementation/instrumentation changed: self-owned client/VPS, authenticated reliable UDP, controlled sender-side packet suppression (not production qdisc/route/firewall), real recovery observations, automatic health-driven warm TCP fallback, stable Session delivery/dedup accounting, bounded resource observations and cleanup. Controlled suppression is **not** natural Internet loss.

Stay inside standing authorization. Do not mix HY2, IPv6, unrelated package lifecycle or experimental-track reruns into this first live run.

### Q12 — one bounded self-owned VPS execution if Q11 is READY

Use the smallest profile that answers the question. Record binary SHA, actual parameters, start/end, packet/recovery health events, switch event, Session delivered/duplicate/lost/uncertain accounting, and cleanup. Preserve negative result exactly; no same-class retry without a materially changed hypothesis.

### Q13 — item-3/item-4 reconciliation and queue refill

Reclassify controlled packet-suppression evidence separately from natural WAN loss/PTO blackhole. Update status/evidence only to the exact supported claim. Keep D019, `SessionRuntime.events`, RSEC-001, signing/SBOM/frozen-release interoperability and final release authority separate. Then inventory the newly integrated runtime surface for the next dependency-safe engineering/review chain; do not revert to `queue exhausted` merely because this one experiment completes.

## Stop / governance boundary

- Current stable-check red and H-RUDP-007/H-RUDP-001C/H-RUDP-008 are local mechanically actionable correctness/evidence blockers. They do not require maintainer policy.
- M-RUDP-009/M-RUDP-010 are local runtime integration corrections under already committed semantics.
- Do not choose or modify D019, event-retention, capacity/security numeric policy, signing/SBOM, previous-release, RC/freeze/release/production authority here.
- No production network mutation, third-party target, or high-load capacity test is authorized by this queue.
- `READY_LIVE: none` until Q10 independent review creates a real changed-hypothesis live row.
- item 3 incomplete; item 4 incomplete; all release/production/freeze flags remain false.
