# ChatGPT reviewer handoff — P2 both-process success accepted; exact causal evidence still open

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `348997060579336ba846125ee0f9aadaa8accd97` (`test(cli): P2 both-process success + server milestones + settled-on-runtime (M-R9-008)`).
- Reviewer checkpoint: `docs/reviews/reviewer-r9-p2-both-process-3489970-20260915.md`.
- GitHub-hosted Rust CI for exact `3489970`: `stable checks` SUCCESS (`bash scripts/check.sh`) and `nightly decode fuzz smoke` SUCCESS. Hosted CI is cross-evidence only; the required final R9-2 developer-local clean exact-tree provenance is still absent.
- Open PRs: none at this review.
- No new WAN/VPS experiment.
- `READY_LIVE: none`; item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continue every dependency-ready slice below without waiting for reviewer cadence. Reviewer cadence is not a work-ticket length.

## Accepted progress — do not revert

### H-R9-019 exact controlled TCP replay identity — CLOSED

Controlled fallback replays from the ownership partition beginning at `uncertain_start`; do not reintroduce positional `skip(1)` replay selection.

### M-R9-008 dedicated post-return settlement evidence — ACCEPT_WITH_BOUNDARIES

`r9_udp_post_return_settled` remains after the bounded post-return owner has retired the exact logical expectation and drained `ReliableUdpRuntime::in_flight()` to zero. Exact `3489970` additionally emits this reliable-settlement event only when a recovery owner exists; owner absence can no longer masquerade as `remaining_in_flight=0`.

### Positive P2 both-process completion — ACCEPT

The existing count=4 migration-back fixture now requires both client and server process success. Do not restore the earlier permissive client-nonzero/server-status-discard behavior.

### Previously highlighted candidates A/B — CLOSED on current HEAD

- `Recovery::on_ack` rejects a largest ACK above `largest_sent` (and ACKs against a never-sent Recovery) before RTT/loss/PTO mutation.
- `neko-observe::record_datagrams` treats `queue_dropped` as a subset of generic `dropped`, emitting `queue_full` only for that subset and `terminal` for the remainder.

Do not reopen these without a new reproducer.

# READY_LOCAL front — finish exact P2 causal evidence in the existing fixture

The exact `3489970` commit message says the P2 test asserts the server-side milestone chain, but repository truth does not: the current test obtains `server_log` and requires `server_status.success()`, then performs only client-log milestone assertions. Do not treat the commit-message sentence as executable evidence.

Tighten `reliable_udp_migration_back_reserves_final_record`; do not create a parallel scenario. In the same count=4 bounded run require:

1. client success and server success (already present);
2. exactly one TCP application replay for stream 1 / offset 32; no replay of offsets 0, 16 or reserved 48;
3. client `udp_recovery_challenge_sent` before client `udp_recovery_validated` and `udp_migrated_back`;
4. server `udp_recovery_owner_started` before server `udp_recovery_validated`;
5. the server post-return acknowledgement branch is reached only after successful authenticated `SessionRuntime::receive` for stream 1 / offset 48;
6. server `udp_return_delivery_ack_sent` for offset 48 before `udp_return_packet_ack_sent` for the post-return Carrier owner;
7. exactly one client `r9_udp_post_return_sent` for stream 1 / offset 48;
8. exact client Session confirmation for offset 48;
9. client `r9_udp_return_packet_ack` with `applied=true`;
10. client `r9_udp_post_return_settled` for stream 1 / offset 48 / `remaining_in_flight=0`.

Use exact event-line fields and order. Existing TCP replay diagnostics expose positional `seq`; if that is not sufficient to prove logical ownership, add only the smallest `stream`/`offset` fields to the existing `tcp_delivery_ack_validated` / `tcp_delivery_ack_sent` events and assert exact counts/identity. Do the same for existing post-return events only where needed. The server ACK-sent source branch is already gated by successful `SessionRuntime::receive`, so do not invent a redundant second data-accepted channel unless exact existing fields cannot make the branch unambiguous.

Correct the stale P2 comment saying "With 3 records" while the fixture uses count=4.

# R9-2 continuation — execute without waiting

## R9-2I-E — post-return acknowledgement-order seam

The same bounded post-return receive owner must succeed in both authenticated arrival orders:

- Session DeliveryAck -> Carrier packet ACK;
- Carrier packet ACK -> Session DeliveryAck.

Use one bounded test-only ordering seam. Both runs must reach the same `r9_udp_post_return_settled` terminal event. No sleep-based ordering, duplicate ACK implementation, second receive owner or fresh per-order deadline.

## R9-2I-A — automatic-health replay exact-identity regression

The automatic-health branch still reduces exact `FailoverController::tcp_resend()` ownership to `.len()` and reconstructs a contiguous replay slice. Add a focused regression comparing actual replay `(DataId, payload)` identities against the exact controller ownership.

- If current contiguous partitioning is exactly equivalent, retain the regression as bounded no-finding evidence.
- If a mismatch is produced, consume exact ownership rather than cardinality.

Do not redesign failover architecture.

## P3 — persistent malformed budget across valid Carrier feedback

Within one reliable receive/settlement operation drive exactly:

```text
malformed #1 -> malformed #2 -> canonical Carrier ACK -> malformed #3
```

Malformed #3 must hit the existing `MAX_POST_HANDSHAKE_MALFORMED`; the valid Carrier ACK must not reset the operation-wide malformed counter. Termination must be typed, finite and non-spinning. Do not change the numeric limit or create a second settlement budget.

## P4 — incomplete settlement remains terminal

Suppress one acknowledgement domain at a time for the post-migration reserved owner and require:

- nonzero/typed terminal result;
- no `r9_udp_in_flight_settled`, `r9_udp_post_return_settled`, or equivalent success while Recovery remains in flight or exact logical confirmation remains outstanding;
- no downstream health/failover continuation from an unproven settled state;
- no fresh deadline after the operation-wide deadline expires.

## R9-2 closure gate

After exact P2 + both acknowledgement orders + automatic exact-identity regression + P3/P4 are all on one reachable source/test SHA, persist developer-local clean exact-tree provenance:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, command exit codes and clean initial/final tree. Run the pinned decoder fuzz commands only if decoder/parser/crypto framing changed. Hosted CI remains additional cross-evidence.

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

The earlier item-4 sweep already challenged the pre-R9 reliable engine, CarrierState/CarrierManager, scheduler/flow accounting, adapters, SessionRuntime, observability, package/reproducibility, dependency/build, CLI portability/output, algorithmic boundedness/validators, wire/parser and PLPMTUD/FEC/disabled-gate candidates.

The new cross-process R9 integration surface is not covered by those historical notes and still requires its own bounded independent review at R9-12. Item 4 therefore remains incomplete.

# VPS opportunity

**Not READY — implementation/evidence + independent-review dependency.** Rental-window priority is acknowledged, but running the current candidate would not answer a truthful new WAN question. Unlock sequence:

```text
complete R9-2 -> R9-3..R9-12 -> Q10/Q11 -> specific READY_LIVE row -> Q12
```

If Q11 creates a changed-hypothesis `READY_LIVE` question, standing authorization already covers the bounded self-owned TCP/UDP VPS run. Permission is not the blocker.

# Separate non-blocking policy gates

`SessionRuntime.events` retention; D019 source retention/no-reset; RSEC-001 adversarial-load/capacity suitability; signing/key custody/SBOM/publication trust; previous frozen-release interoperability; final RC/freeze/release/production authority.

Do not invent policy values while working R9, and do not let these independent gates starve dependency-ready R9 implementation/review work.
