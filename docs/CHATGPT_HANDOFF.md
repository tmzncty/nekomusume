# ChatGPT reviewer handoff — P2 stall root-caused; repair replay ownership then continue deep R9 queue

## Current truth

- Developer/source tip reviewed: exact `64ead5fb157d7c96da9994e593e8ef67b24a881f`, which reverts the guessed P2 recovery retry from `802ef95`.
- Reviewer finding: `docs/reviews/r9-2h-p2-owner-count-mismatch-64ead5f-20260915.md`; reviewer tip before this refresh is `89729f44b97aea7e50303d6e0a6f7e90e69afc56`.
- **ACCEPT `64ead5f`** as disposition of M-R9-009. Do not reintroduce the 500 ms / six-attempt policy or resend identical authenticated recovery ciphertext as a workaround.
- Hosted checks on `64ead5f`: stable success; nightly decode fuzz smoke success. These are supplementary only. P2-P4 still need developer-local clean exact-tree provenance after closure.
- Open PRs: none. No new WAN/VPS experiment. `READY_LIVE: none`.
- Item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

## Accepted R9-2H facts — do not reopen without contradictory evidence

- Out-of-order Session DeliveryAck buffering cannot cumulatively confirm earlier unconfirmed bytes; applied ACK evidence carries exact stream/offset and follows mutation order.
- One operation-wide malformed budget and absolute application deadline; Carrier ACK stays Carrier-local; incomplete Recovery settlement is terminal.
- Post-migration reserved Data is reliable-UDP owned; server creates acknowledgement obligations only after accepted authenticated Session Data; client success requires Session confirmation plus Recovery `in_flight()==0`.
- P1 is accepted at `76c3128`: the built process non-vacuously exercises reversed logical ACK order and settles Recovery to zero.

# READY_LOCAL front

## H-R9-016 — HIGH: client/server TCP replay-count mismatch

The P2 stall is deterministic and is **not** a missing UDP retransmission problem.

For `count=3 + reliable_udp + migration/recovery`:

- client: `uncertain_end = count - 1 = 2`, `uncertain_start = 2`, so `uncertain_count = 0`; nothing enters `FailoverController::track_uncertain`;
- client after TCP promotion: `tcp_records = failover.tcp_resend().len() = 0`;
- server: `tcp_records = count - 2 = 1`;
- server starts `udp_recovery_owner_started` only after consuming that TCP Data loop.

The server therefore waits for one TCP application frame the client will never send and does not reach the already-bound UDP recovery owner in time.

### Smallest repair

No retries, sleeps, new deadlines, new protocol messages or policy values. Make server replay expectation use the same committed ownership partition as the client:

```text
uncertain_end   = recovery_enabled ? count - 1 : count
uncertain_start = reliable_udp ? 2 : 1
tcp_records     = saturating_sub(uncertain_end, uncertain_start)
```

A tiny shared helper is preferred if it prevents drift; do not build a framework. Pin `count=3`:

| reliable UDP | recovery | expected TCP replay |
|---|---|---:|
| false | false | 2 |
| false | true | 1 |
| true | false | 1 |
| true | true | 0 |

This is current-semantics correctness repair, not architecture change.

## P2 — mandatory positive reserved post-migration dual settlement

After H-R9-016, make `reliable_udp_migration_back_reserves_final_record` non-vacuous:

1. require client and server success and retain both logs;
2. require client `udp_recovery_challenge_sent` -> server `udp_recovery_owner_started` -> server `udp_recovery_validated` -> client `udp_recovery_validated` -> migration-back promotion;
3. exactly one post-return reliable send for stream 1 / offset 32, never on the pre-promotion uncertain path;
4. promotion precedes reliable ownership/send;
5. accepted server Session receive for offset 32 precedes creation/emission of both Session DeliveryAck and Carrier ACK; add at most one narrow receive-success event if needed;
6. exact Session confirmation for offset 32 is applied;
7. independent post-return Carrier ACK is applied;
8. post-return terminal Recovery evidence is `remaining_in_flight=0` before success;
9. exercise Session-ACK-first and Carrier-ACK-first with a bounded test-only send-order seam, not sleeps;
10. neither acknowledgement domain substitutes for the other.

Do not change Session/Carrier/ACK/wire architecture.

## P3 — persistent malformed budget

Within one reliable receive/settlement operation drive:

```text
malformed #1 -> malformed #2 -> canonical Carrier ACK -> malformed #3
```

Malformed #3 must hit existing `MAX_POST_HANDSHAKE_MALFORMED`; Carrier ACK must not reset the counter; termination typed, finite, non-spinning. Do not change the numeric limit.

## P4 — incomplete settlement remains terminal

Preserve the existing incomplete-settlement negative and extend it to the post-migration reserved owner by suppressing one acknowledgement domain. Require nonzero terminal result, no settled/success claim, and no downstream health/failover continuation. Reuse existing budgets.

## R9-2H gate

After H-R9-016 + P2-P4 land on one reachable source/test SHA, persist developer-local clean exact-tree provenance:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, exit codes and clean initial/final tree. No decoder/parser/framing change is required by this front; do not invent fuzz work. Then continue immediately to R9-3.

# Continuous queue — keep depth, do not wait for reviewer cadence

1. **R9-3 Data-loss recovery:** suppress reliable-owned Data post-admission; cwnd admission before suppression; PTO after deadline; fresh packet number/nonce retransmit with stable Session/frame identity; exactly-once Session delivery; independent ACK domains; final Recovery zero or typed bounded failure.
2. **R9-4 ACK-loss + delayed original/reorder:** suppress Carrier ACK, retransmit, then release delayed original; one logical delivery; Session dedup; fresh packet numbers/nonces; truthful loss/PTO; settled recovery.
3. **R9-5 tamper/future-ACK/malformed negatives:** no recovery/Session mutation from tamper; future/never-sent ACK atomic rejection; stale/duplicate feedback fabricates no evidence; malformed finite/panic-free.
4. **R9-6 pacing/cwnd/plaintext ownership:** all sends/retransmits consult congestion admission; refusal commits no logical/recovery ownership; one bounded retransmit plaintext owner; teardown releases state.
5. **R9-7 process/result truth:** Data, Carrier ACK, Session ACK, PTO/retransmit, recovery, Session delivery, malformed budget and final outcome remain distinct and emitted only after claimed success.
6. **R9-8 authenticated warm TCP standby:** negotiation + Noise trust/authz + resume/readiness binding + resource admission; no application Data before promotion.
7. **R9-9 resolved UDP health -> hysteresis -> real TCP promotion:** resolved outcomes drive health; recoverable loss stays UDP; PTO-only samples cannot erase later loss; only ready TCP can promote.
8. **R9-10 uncertain Session replay UDP -> TCP:** replay genuine uncertain Session range over promoted TCP; draining UDP gets no new Data; exact-once Session dedup; no TCP packet ACK.
9. **R9-11 timeout/shutdown/cleanup matrix.**
10. **R9-12 coherent exact-tree gate + independent bounded review; BLOCKER/HIGH -> smallest repair + regression + re-gate.**
11. **Q10 observability reconciliation** using existing surfaces only.
12. **Q11 factual status/release reconciliation**; only independently reviewed cross-process R9 + real TCP promotion may create a specific `READY_LIVE` row.
13. **Q12 one changed-hypothesis self-owned VPS run** only after Q11 creates that row; preserve negative evidence and cleanup, no unchanged retry.

# VPS opportunity

**Not READY — implementation dependency.** Unlock: H-R9-016 -> positive P2 -> P3/P4 -> clean provenance -> R9-3..R9-12 -> Q10/Q11. Standing authorization already covers the eventual bounded self-owned TCP/UDP run.

# Separate non-blocking policy gates

`SessionRuntime.events` retention; D019 source retention/no-reset; RSEC-001 capacity/adversarial-load suitability; signing/key custody/SBOM/publication trust; previous frozen-release interoperability; final RC/freeze/release/production authority. Do not invent policy values while working R9.
