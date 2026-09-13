# ChatGPT reviewer handoff — R1–R6 reviewed; health freshness and ACK-emission HIGHs block R7/runtime

## Reviewed repository truth

- Current default-branch developer HEAD reviewed: exact `ecb4902853e8974931d130d595ab187abdebae40` (`feat(carrier): bounded retransmission frame ownership (R6)`).
- New developer sequence after the prior reviewer handoff `7039261`:
  - `16c17f6` — attempted H-RUDP-001 fresh-health repair;
  - `badaafd` — single packet-number source = authenticated `SecureSession` sequence;
  - `9890bc0` — replaced the vacuous ACK negative assertion;
  - `3739138` — per-packet Reno charged-byte accounting;
  - `d8f53e0` — developer-local exact-tree provenance for R1–R4 at `3739138`;
  - `a43f69d` — receiver-side `PacketAckTracker` (R5);
  - `ecb4902` — bounded `RetransmitBuffer` (R6).
- No open PRs.
- Exact `3739138` has persisted developer-local clean exact-tree provenance (`scripts/check.sh`, `git diff --check`, clean initial/final tree, Rust/OS/time recorded). R5/R6 do **not yet** have equivalent persisted developer-local exact-tree provenance on their final pushed source tree.
- At review time, hosted decode fuzz on exact `ecb4902` is green; hosted stable checks are still in progress. Hosted checks remain cross-evidence only and are not a substitute for the required developer-local exact-tree gate.
- No new WAN/VPS result occurred. `READY_LIVE: none` remains authoritative.
- Governance remains unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`; D019, `SessionRuntime.events`, and RSEC-001 remain separate policy/security gates.

## Verdict on R1–R4

### R2 M-RUDP-002 — ACCEPT, with one stale-comment cleanup

`PathRecovery` no longer owns a second packet-number allocator; the actual authenticated `SecureSession` sequence/AEAD nonce is the live packet-number source. This removes the prior divergence footgun without changing crypto semantics.

Minor cleanup: the top-level `PathRecovery` rustdoc still says it owns "packet-number allocation" even though R2 removed that ownership. Correct that wording when touching the owner next; this is documentation drift, not a blocker.

### R3 M-RUDP-003 — ACCEPT

The vacuous `|| true` negative is gone. Encoder rejection of adjacent/noncanonical ranges and raw-overlap decode rejection are now real executable assertions.

### R4 M-RUDP-004 — ACCEPT

Per-packet `charged` accounting closes the reproduced Reno under-drain: non-ack-eliciting packets carry zero congestion charge and cannot retire another packet's bytes-in-flight. The `charged` map is naturally bounded by the recovery engine's bounded in-flight packet set and removes entries on ACK/loss retirement.

### R1 H-RUDP-001 — **NOT CLOSED**

`16c17f6` improves the S8 fixture, but the fresh-evidence guarantee is still bypassable in current public API and still counts unresolved sends as fresh health evidence.

#### H-RUDP-001A — public cumulative `health_sample()` still permits exact snapshot replay

**Severity: HIGH for automatic health/fallback correctness.**

Current `PathRecovery` exposes both:

- `fresh_health_sample(&mut self) -> Option<HealthSample>`; and
- public `health_sample(&self) -> HealthSample`.

The latter still returns the cumulative `packets_lost / packets_sent` snapshot with no freshness guard. The current unit test `recovery_evidence_maps_to_health_sample_without_session_delivery` demonstrates the bypass explicitly: it computes one `bad = r.health_sample()` and calls `CarrierHealth::observe(..., bad)` repeatedly until the path becomes `Degraded`.

Therefore the invariant "one recovery evidence state can advance the health streak at most once" is not enforced by the type/API boundary. A future runtime can accidentally call the old public method and recreate the original false fallback.

**Required repair:** make the health-to-manager bridge structurally freshness-enforcing. The simplest acceptable shape is to make the raw cumulative snapshot private/non-bridge (or return a diagnostic type that cannot be fed directly as manager evidence) while the public runtime-facing API only yields consumable fresh observations. Existing observability may read raw counters/RTT separately; do not require manager health to accept replayable snapshots.

#### H-RUDP-001B — `on_sent` advances `evidence_epoch` before any new loss/ACK/PTO outcome

**Severity: HIGH for automatic health/fallback correctness.**

`on_sent` increments `evidence_epoch`. After any historical cumulative loss, each subsequent send can therefore make `fresh_health_sample()` return `Some(...)` even if no new ACK, loss declaration, or PTO occurred. Because the sample is cumulative, the historical loss ratio can remain above the degradation threshold and repeated sends + polls can advance the bad-observation streak without distinct new bad outcomes.

This is the same class of evidence replay through a different trigger: "new sent packet" is new traffic, but it is not a new resolved loss observation.

**Required repair:** base manager-health emission on resolved recovery observation intervals/deltas, not merely any mutation of the recovery object. Do not invent new thresholds. A minimal shape may track sent/lost/PTO deltas since the last consumed health interval and only make a manager observation available after a feedback/timeout outcome. The exact implementation may differ, but these tests are mandatory:

1. one bad ACK/loss outcome -> at most one bad manager observation;
2. repeated raw polls -> no additional manager observation;
3. repeated `on_sent` calls with no new ACK/loss/PTO outcome -> no additional bad streak advancement from historical loss;
4. a genuinely new clean feedback interval can exercise existing recovery semantics rather than replaying the old cumulative loss forever;
5. distinct new bad feedback/PTO intervals may eventually cross the unchanged existing hysteresis;
6. packet feedback still cannot validate a Path or confirm Session delivery.

Do **not** enter R7/R8 process/runtime work until H-RUDP-001A/B are closed and focused tests demonstrate the public API cannot bypass freshness.

## New finding H-RUDP-005 — ACK tracker can emit ACK forever and cannot suppress ACK-of-ACK

**Severity: HIGH for executable reliable-UDP runtime liveness/resource behavior.**

R5's `PacketAckTracker` is a useful bounded range accumulator, but its current API is not yet a safe ACK-emission state machine:

- `observe_packet(generation, packet_number)` has no `ack_eliciting`/record-class input;
- `build_ack(ack_delay_us)` returns `Some(AckPayload)` forever once *any* packet has been observed;
- building/sending an ACK does not consume or clear a pending-ACK obligation.

A real event loop that polls `build_ack()` can therefore resend the same ACK indefinitely without new receiver evidence. More seriously, an authenticated ACK-only packet has its own `SecureSession` sequence/packet number; the current tracker cannot distinguish it from an ack-eliciting DATA/control packet. If the runtime observes every successfully opened record as the R5 rustdoc currently suggests, ACK packets can trigger ACKs of ACKs and create an ACK ping-pong/resource loop.

This is not a wire-format redesign. The repository already has `SentPacket::ack_eliciting`, so the required semantic distinction exists.

### Required repair before R7

Give receiver ACK state an explicit bounded pending/consume contract. Acceptable minimal shape:

- packet reception may record packet number/range independently;
- only an ack-eliciting authenticated packet creates/refreshes an ACK emission obligation;
- expose a `take_ack`/`pending_ack` equivalent that yields at most one emission for the current new ack-eliciting evidence and then returns `None` until new eligible evidence arrives;
- an ACK-only/non-ack-eliciting packet must not by itself create a new ACK obligation;
- malformed/tampered/wrong-generation input remains unable to mutate ACK state;
- duplicate/reorder behavior remains deterministic and bounded.

Required regressions:

1. DATA/other ack-eliciting authenticated packet -> one pending ACK;
2. repeated ACK polling with no new eligible packet -> `None` / no second send obligation;
3. authenticated ACK-only packet -> no ACK-of-ACK obligation;
4. ACK-only packet may be retained in ranges only if the chosen existing candidate semantics require it, but it still must not schedule a response by itself;
5. duplicate/reordered eligible packets remain bounded/canonical;
6. 33rd fragmented range/capacity behavior is deterministic, atomic, and does not silently fabricate an ACK or Session evidence.

Do not create a second ACK protocol/version or add a new policy threshold.

## R6 acceptance boundary — primitive useful, final-copy retirement still unresolved

`RetransmitBuffer` is acceptable as a bounded plaintext ownership primitive: idempotent same-id tracking, conflict on divergent bytes, explicit capacity, fetch, release and teardown clear are sensible.

However the handoff's R6 requirement also said release must be correct with overlapping original/retransmit copies. That integration property is **not yet proven**.

`neko-reliable::Recovery` already counts overlapping copies in `outstanding_frames`, but `RecoveryResult` currently exposes only `acked_packets`, `lost_packets`, `retransmit_frames`, and byte totals. On the ACK path, `Recovery::release_frame(...)` decrements the copy count but discards the boolean that tells whether the last outstanding copy disappeared. Therefore a future runtime cannot safely infer from `acked_packets` alone when a `RetransmitBuffer` frame is finally releasable without duplicating packet->frame ownership state.

### M-RUDP-006 — provide one authoritative final-frame-retirement signal

**Severity: MEDIUM now; becomes correctness-critical in R7.**

Before composing R7, add the smallest authoritative signal from recovery/path ownership to retransmit ownership. Prefer exposing "frame no longer has any outstanding packet copy" from the existing recovery state rather than maintaining a second divergent copy counter in the runtime.

Required overlap regression:

1. original packet carries frame F;
2. PTO schedules F and a fresh retransmission packet carrying F is recorded while original remains outstanding;
3. ACK of only one copy must **not** release F's plaintext;
4. ACK/retirement of the final outstanding copy releases F exactly once;
5. loss of one copy while another remains must not schedule a duplicate retransmission or release F early;
6. duplicate ACK/late old-copy evidence must not double-release;
7. teardown clears retained plaintext deterministically.

## Continuous READY_LOCAL queue — revised

External coding agent should continue immediately, but **repair the two HIGHs before building the coherent event loop**. Do not wait for the next reviewer between dependency-safe slices.

### Q1 — close H-RUDP-001A/B completely

Enforce freshness at the public health bridge and remove `on_sent`-only replay of historical loss. Add the six regressions above. Commit/push, focused tests, then continue.

### Q2 — close H-RUDP-005 ACK emission / ACK-of-ACK

Add explicit ack-eliciting + pending/consume semantics to `PacketAckTracker` without changing wire version or thresholds. Commit/push, continue.

### Q3 — close M-RUDP-006 final-copy retransmit ownership

Expose authoritative final-frame retirement from existing recovery state and couple it to `RetransmitBuffer` in focused tests. Commit/push, continue.

### Q4 — exact-tree gate/provenance for repaired R1–R6 surface

On the final pushed source tree after Q1–Q3:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
clean initial/final tree
```

Persist SHA, UTC start/end, exits, OS/arch, stable Rust, clean-tree state. If wire/parser bytes/code changed, run the pinned decode fuzz; if only carrier runtime semantics changed, do not mechanically require fuzz. Hosted checks are extra only.

### Q5 — R7 coherent local runtime/event-loop fixture

Only after Q1–Q4 are green. Compose one actual bounded flow:

- UDP socket send/receive;
- authenticated packet number;
- ACK tracker with no replay/ACK-of-ACK;
- ACK application, RTT/loss/PTO;
- retransmit plaintext ownership + fresh reseal + final-copy retirement;
- congestion/pacing;
- freshness-enforced health bridge;
- existing warm TCP readiness;
- automatic manager degrade/fail/promote;
- Session uncertain replay/dedup;
- observability.

No manual repeated snapshot feeding. No manually calling manager transitions as a substitute for the runtime path.

### Q6 — R8 executable bounded lab path

Reuse existing CLI/failover/owned-lab machinery where practical. The executable path must reuse Q5 components rather than duplicate logic in CLI. Keep deterministic lab-only packet suppression, structured recovery/switch/application counters, bounded duration/count/bytes and cleanup.

### Q7 — R9 local process/socket acceptance

At minimum: clean no-loss, isolated recoverable loss, sustained distinct bad outcomes -> automatic fallback, ACK loss/PTO, uncertain/replayed/duplicate/lost accounting, no ACK storm, no health replay, resource cleanup.

### Q8 — R10 independent bounded re-review

Challenge exact pushed executable tree. BLOCKER/HIGH -> repair; no finding -> precise accepted note and exact-tree provenance.

### Q9 — R11 one new READY_LIVE experiment

Only after Q8. Create exactly one controlled self-owned client/VPS question: authenticated reliable UDP under controlled sender-side suppression, real recovery observations, automatic health-driven warm TCP fallback, application delivery accounting and cleanup. No production qdisc/route/firewall changes. Controlled suppression is not natural Internet loss.

### Q10 — item-3/item-4 reconciliation and refill

Reclassify with exact evidence. Keep natural-loss, performance-rate, D019, `SessionRuntime.events`, RSEC-001, signing/SBOM and release-authority boundaries distinct.

## Stop / governance boundary

- H-RUDP-001A/B and H-RUDP-005 are mechanically repairable local correctness HIGHs; they block R7+ expansion but do not require maintainer policy.
- M-RUDP-006 is a local ownership/API repair under already committed recovery semantics.
- Do not choose D019, event-retention, security-capacity numbers, signing/SBOM, RC/freeze/release/production policy in these repairs.
- `READY_LIVE: none` until the executable surface passes independent Q8 review.
- item 3 incomplete; item 4 incomplete; all release/production/freeze flags remain false.
