# ChatGPT reviewer handoff — positive P2 is non-vacuous on the client, but R9-2 is still open

## Current repository truth

- Latest developer-owned source/test commit: exact `ee2156420cf98bb0638c18b56ad3f32077ede225` (`test(cli): positive P2 — migration-back post-return reliable ownership + dual settlement`).
- Latest runtime source change remains exact `6e19a08f2e7e2fc9b747810b1c70cbf01162ed31`; `ee21564` changes only `crates/neko-cli/tests/probe.rs`.
- Reviewer checkpoint: `docs/reviews/reviewer-r9-positive-p2-ee21564-20260915.md`.
- GitHub-hosted checks on exact `ee21564`: `stable checks` SUCCESS and `nightly decode fuzz smoke` SUCCESS. Hosted checks are cross-evidence only; they do not replace the final developer-local clean exact-tree gate.
- Open PRs: none at this review.
- No new WAN/VPS experiment.
- `READY_LIVE: none`; item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continue the READY_LOCAL queue below without waiting for reviewer cadence. Reviewer cadence is not a work-ticket length.

## Accepted progress — do not revert

### H-R9-019 exact controlled TCP replay identity — CLOSED

The controlled fallback path replays from `uncertain_start`, not positional index 1. For the positive count=4 ownership shape:

```text
records[0] offset 0  -> reliable UDP owned
records[1] offset 16 -> reliable UDP owned
records[2] offset 32 -> genuine uncertain range; sole TCP replay
records[3] offset 48 -> reserved for post-migration reliable UDP
```

Do not reintroduce `skip(1)` replay selection.

### `ee21564` positive-P2 test — PARTIAL_ACCEPT

The prior fully conditional acceptance is gone. The fixture now requires client success plus client-side UDP recovery validation, migration-back, the post-return reliable send at offset 48, a Session DeliveryAck observation, an applied Carrier packet ACK observation, and a zero-in-flight diagnostic. A run in which post-return never executes can no longer satisfy the test.

This is useful evidence progress. It is **not** R9-2 closure yet.

# READY_LOCAL front — finish positive P2 causally and exactly

## R9-2I-C1 — both-process positive P2 and exact ownership evidence (first action)

`reliable_udp_migration_back_reserves_final_record` still discards `server_status` and `server_log`. Tighten the existing fixture; do not create a second scenario.

Require all of the following from the same bounded run:

1. client exit success;
2. server exit success;
3. exactly one TCP application replay: stream 1 / offset 32;
4. no TCP replay of offsets 0, 16 or reserved offset 48;
5. client `udp_recovery_challenge_sent`;
6. server `udp_recovery_owner_started`;
7. server `udp_recovery_validated`;
8. client `udp_recovery_validated`;
9. client `udp_migrated_back`;
10. exactly one client `r9_udp_post_return_sent` for offset 48;
11. server authenticated Session receive accepts offset 48 before either acknowledgement-domain send;
12. server `udp_return_delivery_ack_sent` for offset 48;
13. server `udp_return_packet_ack_sent` for the post-return Carrier packet;
14. client exact Session confirmation for offset 48;
15. client `r9_udp_return_packet_ack` with `applied=true`;
16. post-return Recovery terminates with `in_flight==0`.

Use exact structured-event fields/order relationships where the runtime already exposes them. Do not satisfy this with broad unrelated substring matches. Remove the duplicate identical `r9_udp_post_return_sent` assertion currently in the test. Correct the stale count/comment text while touching the fixture.

If this goes red, classify the first missing structured milestone and repair only that transition. Do not add sleeps, fresh deadlines, retries, or alternate evidence channels just to obtain PASS.

## R9-2I-D — dedicated post-return terminal settlement event

The runtime currently waits for both `post_outstanding.is_empty()` and Recovery `in_flight()==0`, but there is no post-return-specific terminal evidence event. Add one diagnostic-only event after both conditions are true, for example:

```text
r9_udp_post_return_settled
  stream=1
  offset=48
  logical_outstanding=0
  remaining_in_flight=0
```

The exact name may follow existing naming style, but P2 must require the event. Never emit it on timeout, malformed-budget exhaustion, missing Session confirmation, rejected Carrier ACK, or nonzero Recovery in-flight state.

This is an evidence boundary; do not change Session/Carrier/ACK semantics.

## R9-2I-E — post-return acknowledgement-order seam

The same post-return bounded receive owner must succeed in both authenticated acknowledgement orders:

- Session DeliveryAck -> Carrier packet ACK;
- Carrier packet ACK -> Session DeliveryAck.

Use one bounded test-only ordering seam. Both runs must reach the same post-return terminal settlement event. No sleep-based ordering, duplicate ACK implementation, second receive owner, or fresh per-order deadline.

## R9-2I-A remainder — automatic-health replay exact-identity regression

The automatic-health branch still reduces `FailoverController::tcp_resend()` to `.len()` and reconstructs a contiguous replay slice. Current code has not reproduced a mismatch, but exact `(DataId, payload)` ownership is discarded.

Add a focused regression comparing emitted TCP replay identities/payloads with exact `tcp_resend()` ownership. If the current contiguous-partition implementation is exactly equivalent, record the no-finding result. If a mismatch is produced, repair by consuming exact DataId/payload ownership rather than only cardinality. Do not redesign failover architecture.

# P3 — persistent malformed budget across valid Carrier feedback

Within one reliable receive/settlement operation drive exactly:

```text
malformed #1 -> malformed #2 -> canonical Carrier ACK -> malformed #3
```

Malformed #3 must hit the existing `MAX_POST_HANDSHAKE_MALFORMED`; the valid Carrier ACK must not reset the operation-wide malformed counter. Termination must be typed, finite and non-spinning. Do not change the numeric limit or create a second settlement budget.

Prefer a bounded authenticated test seam; the Carrier ACK must be canonical and handled by the current demux owner.

# P4 — incomplete settlement remains terminal

Suppress one acknowledgement domain at a time for the post-migration reserved owner and require:

- nonzero/typed terminal result;
- no `r9_udp_in_flight_settled`, `r9_udp_post_return_settled`, or equivalent success while Recovery remains in flight or exact logical confirmation remains outstanding;
- no downstream health/failover continuation from an unproven settled state;
- no fresh deadline after the operation-wide deadline expires.

# R9-2 closure gate

After positive P2 + both acknowledgement orders + automatic exact-identity regression + P3/P4 are all on one reachable source/test SHA, persist developer-local clean exact-tree provenance:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, command exit codes and clean initial/final tree. Run pinned decoder fuzz only if decoder/parser/crypto framing changed. Hosted CI remains additional cross-evidence.

Then continue immediately without waiting for reviewer cadence.

# Continuous queue — preserve depth

1. **R9-3 Data-loss recovery:** suppress reliable-owned Data post-admission; cwnd admission before suppression; PTO after deadline; fresh packet number/nonce retransmit with stable Session/frame identity; exactly-once Session delivery; independent ACK domains; final Recovery zero or typed bounded failure.
2. **R9-4 ACK-loss + delayed original/reorder:** suppress Carrier ACK, retransmit, release delayed original; one logical delivery; Session dedup; fresh packet numbers/nonces; truthful loss/PTO; settled recovery.
3. **R9-5 tamper/future-ACK/malformed negatives:** tamper mutates neither recovery nor Session; future/never-sent ACK atomic rejection; stale/duplicate feedback fabricates no evidence; malformed finite/panic-free.
4. **R9-6 pacing/cwnd/plaintext ownership:** every send/retransmit consults congestion admission; refusal commits no logical/recovery ownership; one bounded retransmit plaintext owner; teardown releases state.
5. **R9-7 process/result truth:** Data, Carrier ACK, Session ACK, PTO/retransmit, recovery, Session delivery, malformed budget and final outcome remain distinct and are emitted only after the claimed transition.
6. **R9-8 authenticated warm TCP standby:** negotiation + Noise trust/authz + resume/readiness binding + resource admission; no application Data before promotion.
7. **R9-9 resolved UDP health -> hysteresis -> real TCP promotion:** resolved packet outcomes drive health; recoverable loss stays UDP; PTO-only samples cannot erase later loss; only ready TCP can promote.
8. **R9-10 uncertain Session replay UDP -> TCP:** replay the exact genuine uncertain Session range over promoted TCP; draining UDP gets no new Data; exact-once Session dedup; no TCP packet ACK.
9. **R9-11 timeout/shutdown/cleanup matrix.**
10. **R9-12 coherent exact-tree gate + dedicated independent bounded review of the new cross-process reliable-UDP integration surface.** Any BLOCKER/HIGH returns to smallest repair + regression + re-gate.
11. **Q10 observability reconciliation** using existing surfaces only.
12. **Q11 factual status/release reconciliation.** Only independently reviewed cross-process R9 + real TCP promotion may create a specific `READY_LIVE` row.
13. **Q12 one changed-hypothesis self-owned VPS run** only after Q11 creates that row; preserve negative evidence and cleanup; no unchanged retry.

# Core-surface review inventory

The earlier item-4 sweep already challenged the pre-R9 reliable engine, CarrierState/CarrierManager, scheduler/flow accounting, adapters, SessionRuntime, observability, package/reproducibility, dependency/build, CLI portability/output, boundedness/validators, wire/parser and PLPMTUD/FEC/disabled-gate candidates.

The new cross-process R9 integration surface is not covered by those historical notes and still requires its own bounded independent review at R9-12. Item 4 therefore remains incomplete even though the historical pre-R9 core-surface inventory is broad.

# VPS opportunity

**Not READY — implementation/evidence + independent-review dependency.** The rental-window priority is acknowledged, but running the current candidate would not answer a truthful new WAN question. Unlock sequence:

```text
complete R9-2 -> R9-3..R9-12 -> Q10/Q11 -> specific READY_LIVE row -> Q12
```

If Q11 creates a changed-hypothesis `READY_LIVE` question, standing authorization already covers the bounded self-owned TCP/UDP VPS run. Permission is not the blocker.

# Separate non-blocking policy gates

`SessionRuntime.events` retention; D019 source retention/no-reset; RSEC-001 adversarial-load/capacity suitability; signing/key custody/SBOM/publication trust; previous frozen-release interoperability; final RC/freeze/release/production authority.

Do not invent policy values while working R9, and do not let these independent gates starve dependency-ready R9 implementation/review work.
