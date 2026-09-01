# Nekomusume ChatGPT Handoff

Checked at: 2026-09-01 14:00 Asia/Shanghai
Reviewed implementation HEAD: `ed08b644b6cc88ca2b322fd09b3f971d604f791c`
Previous reviewed implementation HEAD: `ecb8729a01761cb62ee889fa17e6c50790006d4f`
Previous reviewer handoff commit: `f1949bfc4d2079e5f6a7415499d4b14cf26955cd`

## What changed

One substantive coding-agent commit landed after the previous reviewer handoff:

- `ed08b64` — **automatic-health failover candidate + deterministic loopback tests + current-code VPS negative evidence; not yet a truthful release-evidence PASS**.

Useful work in this commit:

- `CarrierHealthEvidence` can expose the current path record and `FailoverController` has a health-triggered adapter candidate;
- `failover-server` gained an explicit off-by-default bounded `--cease-udp-replies-after` application-level degradation seam;
- `failover-client` gained a distinct `--automatic-health-failover` path rather than renaming the old controlled-stop fixture;
- deterministic loopback tests prove below-threshold health states do not switch, recovery hysteresis exists, malformed post-cessation traffic is ignored by the later health loop, TCP resume uses authenticated DeliveryAck, and final receiver bytes are complete;
- the old controlled-stop path remains separate;
- exact-head GitHub Rust CI run #85 is green for `ed08b64`.

The commit also preserves a historical prior-branch VPS PASS separately from the re-port result. The **current exact-code VPS row is negative**: experiment `automatic-health-ecb8729-20260901-r5` negotiated/authenticated successfully, but before the health-degradation seam the client consumed a stale/duplicate post-handshake UDP datagram at the first DeliveryAck boundary and failed closed with `unauthenticated UDP delivery acknowledgement`. Therefore the current implementation has not yet produced real-socket health-driven UDP->TCP resume evidence.

This negative row is valuable: it exposed a real UDP retry/admission boundary that loopback timing did not reproduce.

## Review verdict

**CONTINUE WITH REQUIRED REPAIRS — automatic-health local candidate is useful, but Primary A is not complete and must not be promoted to VPS release evidence yet.**

There are four concrete correctness/evidence-contract problems to close before another automatic-failover VPS PASS can be claimed:

1. **Post-handshake duplicate admission:** the client currently assumes the first same-peer datagram after sending application record 1 is the encrypted DeliveryAck. Real timing can leave a duplicate cached Noise response queued after `hs.finish()`. The existing retry test proves server state is not reset, but does not prove a delayed duplicate cannot cross into the application receive phase.
2. **Fabricated health telemetry:** the automatic path turns every receive miss into `HealthSample { rtt_us: 100_000, loss_per_mille: 1000, pto: 3 }`. Those RTT/loss/PTO values were not measured. D064 permits timeout/probe failure as a health observation, but does not permit inventing unrelated metrics.
3. **Accepted-policy drift:** D064 fixes the initial `k_failure = 3`, while this runner uses `HealthLimits::default().fail_after = 4` and the process test asserts threshold 4. The executable release-evidence path must either use the accepted D064 profile or explicitly amend the decision through the normal governance path; do not silently substitute 4.
4. **Selection/reason ownership drift:** D064 says Carrier Manager is the sole owner of active-path selection and fixes stable switch reason codes (`udp_blackhole`, `udp_path_degraded`, ...). `udp_health_at()` currently mutates `FailoverController.active` directly and the runner reports `authenticated_delivery_ack_timeout` as the switch reason. That string may remain an underlying diagnostic cause, but it is not an accepted manager switch reason.

A fifth evidence limitation must stay explicit: the current TCP connection is created only after the health decision, so this path is **cold fallback**, not the D064 warm-standby recovery class. Do not label it warm merely because the server TCP listener already exists.

No maintainer decision is required. These are implementation/spec-alignment repairs within the accepted architecture.

## Evidence boundaries

### Accepted from `ed08b64`

- The explicit server-side UDP reply-cessation seam is bounded, off by default, and does not modify firewall/route/qdisc or production services.
- The automatic path is distinct from the old controlled-stop path.
- Local deterministic evidence proves a `HealthState::Failed` adapter can trigger the existing failover bookkeeping and that healthy/recovered states do not trigger it.
- Exact-head Rust CI #85 is green.
- The current-code cross-host failure is preserved as negative evidence with cleanup rather than overwritten by an older PASS.
- The failure is not evidence of natural WAN loss; it happened at the post-handshake/application admission boundary before the intended degradation seam.

### Not accepted as release evidence yet

- No current exact-head real-socket automatic health failover PASS exists.
- No truthful measured RTT/loss/PTO sample exists for the automatic path; the current constants are synthetic placeholders and must not appear as measured WAN telemetry.
- The process runner does not currently implement the D064 `k_failure=3` policy.
- `FailoverController` currently performs the active-carrier mutation instead of consuming a Carrier-Manager-owned switch decision.
- The emitted `authenticated_delivery_ack_timeout` is a diagnostic cause, not one of the accepted D064 switch reason codes.
- Current recovery is cold, not warm.
- `udp_health_at()` is given synthetic `sample_index * 100_000` times and initializes `failure_started_us` only when state is already Failed, so its `last_recovery_latency_us` is not a truthful real recovery measurement for this path.
- `ROADMAP.md` real `UDP degradation / TCP fallback` therefore remains unchecked.
- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` remain correct. `CANONICAL_CORPUS_V1_FROZEN=true` is still corpus-specific only.

## Work Package — repair the real retry/health boundary, then immediately spend the VPS window on replacement evidence

Execute A -> B -> C in dependency order. This is intentionally a thick package: do not stop after one helper if the next repair/test/evidence step is already READY.

### Primary A — Make the automatic health path semantically truthful and robust to delayed UDP handshake retries

**Goal**

Turn `ed08b64` from a useful loopback candidate into a truthful current-code automatic-degradation path: pre-data handshake retransmissions cannot poison the application receive phase; health state is driven by explicit observed events rather than invented telemetry; D064 owns threshold/reason/selection semantics; and timing/recovery evidence uses real monotonic events.

#### A1 — Add a bounded post-handshake UDP admission helper

The first application DeliveryAck receive must no longer be a single `recv_from()` + immediate `open_unreliable()` assumption.

Implement the smallest bounded helper/loop that, until the existing application deadline:

- accepts only datagrams from the expected peer;
- recognizes and idempotently ignores **exact duplicate protocol-retransmission artifacts that the client can identify from the completed negotiation/Noise exchange** (for example the exact cached Noise response bytes, and any exact duplicate negotiation response if that can remain queued by the current retry path);
- never treats those duplicates as Session delivery, path validation, health progress, or a new handshake;
- never accepts arbitrary unauthenticated bytes as a DeliveryAck;
- permits arbitrary malformed/unadmitted same-peer datagrams to be ignored only under an explicit small count/rate bound and the same absolute application deadline, with deterministic diagnostics; exhaustion fails closed rather than spinning forever;
- returns only a successfully authenticated, exact-semantic Session `DeliveryAck` for the expected record, or a bounded terminal error.

Do not “fix” this by sleeping, flushing the socket blindly, disabling retries, or accepting any decrypt failure as harmless. The exact stale retransmission must be recognizable as old protocol traffic; unrelated attacker garbage remains untrusted.

Add a deterministic regression that deliberately delays a duplicate Noise response so it arrives **after** `hs.finish()` and before the first application DeliveryAck. The real application DeliveryAck must still validate, and the duplicate must not reset negotiation, Noise, ResumeGuard, Session, path generation, or delivery state.

Also test wrong-peer traffic and bounded malformed same-peer traffic at this boundary.

#### A2 — Separate health-event truth from measured telemetry

Do not encode a timeout as fake `rtt_us/loss_per_mille/pto`.

Use the smallest API consistent with the existing carrier model, for example an explicit health observation/event path (`progress`, `probe/application timeout`, independently measured sample) or an equivalent adapter. Exact API naming is implementation-owned, but these invariants are required:

- a bounded authenticated DeliveryAck/readiness timeout can count as one failed health observation;
- authenticated progress counts as one good observation and resets/recovers according to the accepted hysteresis contract;
- measured `HealthSample` fields remain reserved for values actually measured/derived by a documented measurement algorithm;
- JSON/evidence must distinguish event/cause from optional measured RTT/loss/PTO; do not serialize invented numbers simply because the legacy `HealthSample` struct requires them;
- health evidence remains bounded in count and paths.

If the current `CarrierHealth` internals require a small `observe_good/observe_failure` seam in addition to `observe(HealthSample)`, add that rather than overloading fake sample values. Do not redesign the whole manager.

#### A3 — Reconcile D064 threshold and stable reason codes

For this release-evidence path use the accepted D064 initial policy:

```text
k_failure = 3 consecutive failed probes/observations
one miss != switch
```

The current generic `HealthLimits::default()` uses 4. Do not silently claim that as D064. Prefer one of these narrow repairs:

- make the D064 failover profile explicit in the runner/manager while leaving unrelated legacy defaults intact; or
- change the shared default only if repository evidence shows it is intended to represent D064 everywhere and all affected tests/specs are updated consistently.

A threshold-crossing manager event must use an accepted stable reason code. For the explicit reply-cessation seam, `udp_path_degraded` is the conservative default unless the experiment truly establishes a blackhole. Keep `authenticated_delivery_ack_timeout` as a lower-level diagnostic cause if useful, not as the manager switch reason.

Tests must prove 1 and 2 misses do not switch, the 3rd accepted failed observation reaches the expected state/decision, progress resets the counter per policy, and stale/wrong generation cannot mutate active selection.

#### A4 — Restore Carrier Manager ownership of the switch

D064 says the Carrier Manager owns active-path selection. The current `FailoverController::udp_health_at()` directly changes its own `active` carrier.

Bridge the existing components without creating another state machine:

- CarrierHealth/CarrierHealthEvidence owns bounded health observations and transitions;
- Carrier Manager owns the actual path promotion/failure decision, generation and stable reason;
- FailoverController (or the existing Session failover bookkeeping seam) may retain uncertain/replay/dedup state and execute replay **after** a manager decision, but it must not independently decide which carrier becomes active.

If the current `CarrierManager` API lacks exactly one operation needed for active UDP failure -> eligible TCP promotion, add the smallest generation-aware API and deterministic tests. Preserve single-active ownership and reject stale generation before mutation.

Do not wholesale rewrite all historical manager/failover types in this slice; add one explicit ownership bridge and document which type owns which fact.

#### A5 — Make recovery timing/classification truthful

Use actual monotonic runner timestamps, not `sample_index * 100_000`, for evidence fields such as:

- first failed observation / degradation start when defined;
- `failure_decided_at`;
- TCP connect/auth/resume-ready;
- first accepted resumed Session data;
- recovery latency.

Do not infer wall-clock synchronization between hosts; relative monotonic client-side intervals are enough for this bounded row.

Until TCP is authenticated/validated before the failure decision, classify the run as `cold`. If a true warm fallback prerequisite can be added narrowly under the accepted ADR, it may be a later follow-up; do not block this repair merely to redesign the whole warm-readiness protocol.

#### A6 — Deterministic closure and full local gate

At minimum add/retain tests for:

1. delayed duplicate Noise response crosses into the application receive window and is safely ignored as an exact stale retransmission;
2. wrong-peer / arbitrary malformed traffic never becomes DeliveryAck or health progress and remains bounded;
3. D064 1/2/3 failure threshold behavior;
4. authenticated progress recovery/reset;
5. manager is sole active-path decision owner;
6. stale/wrong generation cannot switch;
7. stable manager reason is `udp_path_degraded` for the controlled cessation seam;
8. cold recovery is labeled cold;
9. uncertain range replay + exact duplicate idempotence + conflicting duplicate fail-closed remain intact;
10. old controlled-stop fixture remains behaviorally separate.

Run targeted tests, `cargo fmt --all -- --check`, workspace check/test/clippy locked gates, `bash scripts/check.sh`, and `git diff --check`. Run fuzz smoke only if parser/wire behavior changes or the normal gate requires it.

**Primary A completion definition:** current code no longer fails on delayed duplicate handshake responses at the application boundary; no synthetic health telemetry is presented as measured evidence; D064 threshold/reason/manager ownership are enforced; local deterministic/full gates pass; exact-head CI is green or any failure is investigated.

### Follow-up B — Replacement VPS automatic-degradation row on the repaired exact commit

**Dependency:** Primary A green locally.

Immediately use the rented VPS under standing authorization. Run one bounded self-owned cross-host IPv4 application-level reply-cessation -> manager-driven TCP resume row on the exact repaired commit.

Keep the scenario small and reproducible (for example count 3, small payload, one session, <=10 s) unless the existing harness requires another bounded profile.

Record:

- experiment ID, exact commit and release binary SHA-256 on both hosts;
- owned path classification and actual ports/bounds;
- negotiation/Noise retry diagnostics including any stale duplicate ignored at the application boundary;
- exact failure-observation count and D064 threshold 3;
- manager transition with stable reason;
- cold/warm classification (truthfully cold unless TCP was already authenticated/validated before decision);
- monotonic failure-decided, TCP-ready/resume and first-resumed-data times + recovery interval;
- confirmed/uncertain/replayed/duplicate/missing Session bytes/records from real state;
- receiver final application records/bytes;
- CPU/RSS/FD/owned-socket observations where the direct-child sampler is applicable;
- client/server exits and cleanup verification.

Preserve another failure as evidence. If the repaired exact scenario fails for a **new** cause, do not mechanically rerun unchanged; record the new blocker and proceed to Follow-up C only if it is independent.

If this row passes, `ROADMAP.md` may mark only the narrow controlled self-owned application-degradation -> TCP fallback evidence that was actually proved. Do not call it arbitrary natural-WAN blackhole behavior.

### Follow-up C — Keep the VPS productive with an independent READY evidence row

**Dependency:** A local correctness is green. B may PASS or may be blocked by a new environment-specific condition.

Select the highest-value independent VPS task that does not depend on the failing automatic path:

1. **Preferred:** produce the first fair HY2 v2.9.3 paired application sample using the already pinned artifact and forwarding seam, if the equal-application Nekomusume command/wrapper is now implementable from the generic authenticated TCP path without weakening safety guards.
2. Otherwise build the genuine periodic authenticated real-socket runner and take one 5-minute self-owned UDP session sample, if it reuses the corrected post-handshake admission path and does not inherit an unresolved A defect.
3. Otherwise take a release-relevant package/readiness/resource row only if it answers a currently open question and current package contents changed materially.

For HY2, preserve the existing production Hysteria service and credentials entirely. Use disposable experimental config/cert/auth, fresh high ports and explicit cleanup. Fix equal application payload/hash, same client/VPS and close time window, route/MTU metadata, one stream/load, same run count/timeout, and truthful security-class wording. Report raw samples, median/P95/failures and comparable CPU/RSS/FD/application bytes. No superiority claim from the first sample.

For a periodic session, one run must stay <=10 minutes and be a genuine single open authenticated Session with periodic exchanges, not repeated short probes pretending to be longevity.

## Completion gates for this batch

- Exact-head CI for the repaired commit is green or any failure is investigated.
- No stale negotiation/Noise retransmission can be mistaken for an application DeliveryAck.
- Arbitrary unauthenticated UDP cannot become Session delivery or health progress.
- Health state is driven by truthful observations, not fabricated RTT/loss/PTO values.
- D064 `k_failure=3` is the executable release-evidence policy or an explicit reviewed decision changes it.
- Carrier Manager, not FailoverController, owns active-path selection.
- The controlled reply-cessation switch uses an accepted stable manager reason.
- Recovery timing/classification is measured from real monotonic events and labeled cold/warm truthfully.
- One replacement self-owned VPS row is attempted after meaningful code change; negative evidence is retained.
- If an independent VPS task is READY after the repair, rental time is used rather than spent on unrelated local polish.
- No third-party targets, production network mutation, >10-minute run, >256 MiB application traffic, >32 sessions, or long-lived experimental daemon.
- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` remain unchanged.

## Do not expand into

- disabling UDP/Noise retries merely to hide the stale-response bug;
- treating every decrypt failure as benign;
- fabricating loss/RTT/PTO values for convenience;
- changing D064 policy silently;
- adding a fourth failover/timeout state machine;
- calling a post-failure TCP connection `warm`;
- natural-WAN or production claims from the controlled application-level cessation seam;
- 0-RTT, enabled FEC, striping/aggregation or exotic carriers without an observed-problem gate;
- third-party targets, scanning, production firewall/route/DNS/proxy/tunnel/qdisc changes;
- weakening the existing loopback-only HY2 harness guard; use a separate self-owned-VPS orchestrator if needed.

## Fallback

If the stale post-handshake datagram cannot be classified safely from the current protocol bytes without adding an ambiguous heuristic, stop only that receive-path repair and preserve a minimal reproducer. Use the existing transcript/handshake cache facts to design an explicit bounded retransmission identifier or phase-admission rule; do not guess by packet length or timing.

While that design is being resolved, independent generic TCP/HY2 comparison preparation or package/resource evidence may continue if it does not touch the ambiguous UDP admission path and remains inside standing authorization.

## Questions requiring maintainer decision

none.
