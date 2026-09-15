# ChatGPT reviewer handoff — controlled replay identity accepted; positive R9 P2 is the READY_LOCAL front

## Current repository truth

- Latest developer-owned source/test commit: exact `6e19a08f2e7e2fc9b747810b1c70cbf01162ed31` (`fix(cli): TCP replay identity from uncertain_start partition (H-R9-019) + P2 count=4 fixture`).
- Reviewer checkpoint: `docs/reviews/reviewer-r9-replay-identity-6e19a08-20260915.md` (added by the docs-only descendant immediately after review).
- GitHub-hosted checks on exact `6e19a08`: `stable checks` SUCCESS and `nightly decode fuzz smoke` SUCCESS. Hosted checks are cross-evidence only; they do not replace the final developer-local clean exact-tree gate.
- Open PRs: none at the last reviewed repository state. No new WAN/VPS experiment.
- `READY_LIVE: none`; item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continue through the READY_LOCAL queue below without waiting for reviewer cadence.

## H-R9-019 controlled replay identity — CLOSED; do not revert

The concrete controlled-path defect is repaired: TCP replay now starts at `uncertain_start`, not positional index 1.

For the positive P2 shape:

```text
count=4
bytes=16
reliable_udp=true
migration_back=true

records[0] offset 0  -> reliable UDP owned
records[1] offset 16 -> reliable UDP owned
records[2] offset 32 -> genuine uncertain range; exactly one TCP replay
records[3] offset 48 -> reserved for post-migration reliable UDP
```

The controlled replay set is therefore offset 32; offsets 0/16 and reserved offset 48 are excluded.

### Automatic-health exact-identity support remains

The automatic-health branch still reduces `failover.tcp_resend()` to `.len()` and reconstructs the replay slice from `uncertain_start`. On the current CLI path, the tracked uncertain set is contiguous and no pre-replay confirmation mutates it, so the reviewer did not reproduce a current automatic-mode mis-selection at exact `6e19a08`. This is not a current HIGH, but the exact `(DataId, payload)` ownership returned by `tcp_resend()` is still not consumed and the focused exact-identity regression is absent. Keep this as R9 integration review-support before R9-12; do not generalize equal cardinality into a future ownership contract.

# READY_LOCAL front — make positive migration-back P2 real, not conditional

The `count=4` fixture now has a satisfiable ownership shape, but the current process test is still not positive evidence. It discards server status/log, accepts nonzero client exit, and gates all post-return assertions behind `if client_log.contains("r9_udp_post_return_sent")`.

## R9-2I-C — non-vacuous positive P2 (first action)

In `reliable_udp_migration_back_reserves_final_record`:

1. retain `server_status` and `server_log`;
2. require both client and server exit success;
3. require exactly one TCP application replay at offset 32 and no TCP replay of offsets 0, 16 or 48;
4. remove the conditional acceptance around `r9_udp_post_return_sent`;
5. require the exact causal chain:
   - client `udp_recovery_challenge_sent`;
   - server `udp_recovery_owner_started`;
   - server `udp_recovery_validated`;
   - client `udp_recovery_validated`;
   - client migration promotion (`udp_migrated_back` or the exact current event);
   - exactly one post-return reliable UDP send at offset 48;
   - server authenticated Session receive accepts offset 48 before either acknowledgement-domain send;
   - server Session DeliveryAck for offset 48;
   - server Carrier packet ACK;
   - client exact Session confirmation for offset 48;
   - client Carrier ACK `applied=true`;
   - final post-return Recovery `in_flight==0`.
6. Assert exact structured events/fields, not broad substring coincidence.

If this becomes red, classify the first missing structured milestone and repair only that transition. Do not add sleeps, retries, new deadlines or alternate evidence sources merely to obtain PASS.

## R9-2I-D — dedicated post-return settlement evidence

After the post-return receive owner has proven both exact logical confirmation and Carrier recovery settlement, emit one diagnostic-only terminal event, e.g.:

```text
r9_udp_post_return_settled
  stream=1
  offset=48
  logical_outstanding=0
  remaining_in_flight=0
```

P2 must require this exact event. Never emit it on timeout, malformed-budget exhaustion, missing Session confirmation or nonzero Recovery in-flight state.

## R9-2I-E — acknowledgement-domain order seam

The same post-return owner must succeed with both acknowledgement arrival orders after authenticated Session receive:

- Session DeliveryAck -> Carrier packet ACK;
- Carrier packet ACK -> Session DeliveryAck.

Use one bounded test-only ordering seam. Both runs must reach the same terminal settlement event. No sleeps, extra retry loop, fresh deadline or duplicate ACK implementation.

## R9-2I-A remainder — automatic replay exact ownership regression

Before R9-2 closure, add a focused regression that compares automatic-health TCP replay identities/payloads with the exact entries returned by `FailoverController::tcp_resend()`. If the current contiguous-partition implementation remains equivalent, record that no-finding result; if a mismatch is produced, repair by consuming exact DataId/payload ownership rather than only the count. Do not redesign failover architecture.

# P3 — persistent malformed budget across valid Carrier feedback

Within one reliable receive/settlement operation drive exactly:

```text
malformed #1 -> malformed #2 -> canonical Carrier ACK -> malformed #3
```

Malformed #3 must hit the existing `MAX_POST_HANDSHAKE_MALFORMED`; the valid Carrier ACK must not reset the operation-wide malformed counter. Termination must be typed, finite and non-spinning. Do not change the numeric limit or create a second settlement budget.

Prefer a bounded authenticated test injection seam; the Carrier ACK must be canonical and handled by the current reliable demux owner.

# P4 — incomplete settlement remains terminal

Suppress one acknowledgement domain at a time for the post-migration reserved owner and require:

- nonzero/typed terminal result;
- no `r9_udp_in_flight_settled`, `r9_udp_post_return_settled`, or equivalent success while Recovery remains in flight or exact logical confirmation remains outstanding;
- no downstream health/failover continuation from an unproven settled state;
- no fresh deadline after the operation-wide deadline expires.

# R9-2 closure gate

After positive P2 + acknowledgement-order coverage + automatic exact-identity regression + P3/P4 are all on one reachable source/test SHA, persist developer-local clean exact-tree provenance:

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
5. **R9-7 process/result truth:** Data, Carrier ACK, Session ACK, PTO/retransmit, recovery, Session delivery, malformed budget and final outcome stay distinct and are emitted only after the claimed transition.
6. **R9-8 authenticated warm TCP standby:** negotiation + Noise trust/authz + resume/readiness binding + resource admission; no application Data before promotion.
7. **R9-9 resolved UDP health -> hysteresis -> real TCP promotion:** resolved packet outcomes drive health; recoverable loss stays UDP; PTO-only samples cannot erase later loss; only ready TCP can promote.
8. **R9-10 uncertain Session replay UDP -> TCP:** replay the exact genuine uncertain Session range over promoted TCP; draining UDP gets no new Data; exact-once Session dedup; no TCP packet ACK.
9. **R9-11 timeout/shutdown/cleanup matrix.**
10. **R9-12 coherent exact-tree gate + dedicated independent bounded review of the new cross-process reliable-UDP integration surface.** Any BLOCKER/HIGH returns to smallest repair + regression + re-gate.
11. **Q10 observability reconciliation** using existing surfaces only.
12. **Q11 factual status/release reconciliation.** Only independently reviewed cross-process R9 + real TCP promotion may create a specific `READY_LIVE` row.
13. **Q12 one changed-hypothesis self-owned VPS run** only after Q11 creates that row; preserve negative evidence and cleanup; no unchanged retry.

# Core-surface review inventory

The earlier item-4 sweep already challenged the pre-R9 reliable engine, CarrierState/CarrierManager, scheduler/flow accounting, adapters, SessionRuntime, observability, package/reproducibility, dependency/build, CLI portability/output, boundedness/validators, wire/parser and PLPMTUD/FEC/disabled-gate candidates. The new cross-process R9 integration surface is not covered by those historical notes and still requires its own bounded independent review at R9-12.

# VPS opportunity

**Not READY — implementation/evidence + independent-review dependency.** The rental-window priority is acknowledged, but running the current candidate would not answer a truthful new WAN question. Unlock sequence: positive non-vacuous P2 -> acknowledgement-order + automatic identity support -> P3/P4 -> exact-tree provenance -> R9-3..R9-12 -> Q10/Q11. If Q11 creates a specific changed-hypothesis `READY_LIVE` row, standing authorization already covers the bounded self-owned TCP/UDP VPS run; permission is not the blocker.

# Separate non-blocking policy gates

`SessionRuntime.events` retention; D019 source retention/no-reset; RSEC-001 adversarial-load/capacity suitability; signing/key custody/SBOM/publication trust; previous frozen-release interoperability; final RC/freeze/release/production authority. Do not invent policy values while working R9.
