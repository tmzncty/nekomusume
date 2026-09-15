# ChatGPT reviewer handoff — H-R9-017 accepted; R9-2H advances to positive P2/P3/P4 closure

## Current truth

- Developer/source tip reviewed: exact `422c16b106b5568cb87f81419b45941508c2d4a4` (`fix(cli): controlled-fallback TCP replay uses ownership partition (H-R9-017)`).
- Independent bounded recheck: `docs/reviews/r9-2h-controlled-replay-recheck-422c16b-20260915.md`.
- Reviewer report commit before this handoff: exact `0f392bb1e97bcae5c2f4ec9f2097d905b72cccc5`.
- Hosted checks on developer exact `422c16b`: `stable checks` SUCCESS; `nightly decode fuzz smoke` SUCCESS. These are cross-evidence only; final R9-2H source/test closure still requires persisted developer-local clean exact-tree provenance.
- Open PRs: none. No new WAN/VPS experiment. `READY_LIVE: none`.
- Item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

## H-R9-017 — CLOSED

The controlled/non-automatic client replay count now uses the same committed ownership partition as the server:

```text
uncertain_start = reliable_udp ? 2 : 1
uncertain_end   = recovery_enabled ? count - 1 : count
controlled_tcp_replay = saturating_sub(uncertain_end, uncertain_start)
```

Automatic-health mode still uses the actual tracked uncertain set from `failover.tcp_resend()` and is unchanged.

For `count=3`, the formulas yield the intended controlled replay cardinalities:

| reliable UDP | recovery / migration reservation | TCP replay |
|---|---|---:|
| false | false | 2 |
| false | true  | 1 |
| true  | false | 1 |
| true  | true  | 0 |

This closes the client/server drift that caused the extra reliable-owned replay and `BrokenPipe`. Do not revert the server/client ownership partition and do not reintroduce guessed retries/sleeps/deadlines.

## Accepted R9-2H facts — preserve unless contradicted

- Out-of-order Session DeliveryAck buffering cannot cumulatively confirm earlier unconfirmed bytes; exact applied evidence is stream/offset-bearing and ordered by actual Session mutation.
- One operation-wide malformed budget and one absolute application deadline; Carrier ACK remains Carrier-local; incomplete Recovery settlement is terminal.
- Post-migration reserved Data is reliable-UDP owned; server acknowledgement obligations arise only after accepted authenticated Session Data; client success requires exact Session confirmation plus Recovery `in_flight()==0`.
- P1 is accepted: reversed logical ACK order buffers offset 16, applies offset 0, drains offset 16, and Carrier settlement reaches zero on a successful built-binary path.
- H-R9-015 post-return source shape is accepted: the reserved final record is Recovery-owned after migration-back, and server/client retain separate Session-ACK and Carrier-ACK obligations.

# READY_LOCAL front — finish R9-2H acceptance evidence

## P2 — mandatory positive post-migration reserved-record dual settlement

Current `reliable_udp_migration_back_reserves_final_record` is still **not acceptance evidence**. The test enforces the negative pre-promotion reservation invariant, but its positive post-return assertions are conditional on already seeing `r9_udp_post_return_sent`; the source explicitly documents that the fixture may stall before that path. A passing test can therefore remain vacuous about post-migration reliable ownership.

Make P2 non-vacuous without changing Session/Carrier/ACK/wire architecture:

1. require both client and server exit success and retain both logs;
2. require the causal chain client `udp_recovery_challenge_sent` -> server `udp_recovery_owner_started` -> server `udp_recovery_validated` -> client `udp_recovery_validated` -> migration-back promotion;
3. exactly one post-return reliable send for stream 1 / offset 32, never on the pre-promotion uncertain path and never over TCP;
4. promotion must precede reliable-UDP Recovery ownership / socket send;
5. accepted server Session receive for offset 32 must precede creation/emission of both Session DeliveryAck and independent Carrier ACK; add at most one narrow accepted-receive diagnostic if needed;
6. exact Session confirmation for offset 32 must be applied;
7. independent post-return Carrier ACK must be applied to Recovery;
8. terminal post-return Recovery evidence must be `remaining_in_flight=0` before client success;
9. exercise **Session-ACK-first** and **Carrier-ACK-first** using a deterministic bounded test-only ordering seam, never sleeps or timing luck;
10. suppressing either acknowledgement domain must prevent success; neither domain substitutes for the other.

If the existing recovery-challenge path still cannot deterministically reach promotion, first identify the exact current source/test mismatch and make the smallest semantics-preserving deterministic fixture repair. Do not revive the reverted retry/sleep experiment and do not weaken positive acceptance into conditional assertions.

## P3 — persistent malformed budget across Carrier feedback

Within one reliable receive/settlement operation, drive exactly:

```text
malformed #1 -> malformed #2 -> canonical Carrier ACK -> malformed #3
```

Malformed #3 must hit the existing `MAX_POST_HANDSHAKE_MALFORMED`; the valid Carrier ACK must not reset the operation-wide malformed counter. Termination must be typed, finite and non-spinning. Do not change the numeric limit and do not create a second malformed budget for settlement.

## P4 — incomplete settlement remains terminal

Preserve the existing incomplete-settlement negative and extend it to the post-migration reserved owner. Suppress one acknowledgement domain at a time and require:

- nonzero/typed terminal outcome;
- no `r9_udp_in_flight_settled` or equivalent settled/success claim while `in_flight()!=0` or exact logical confirmation remains outstanding;
- no downstream health/failover continuation based on an unproven settled state;
- no fresh deadline after the operation-wide deadline has expired.

Reuse existing budgets and owner state.

## R9-2H closure gate

After positive P2 + P3/P4 are on one reachable source/test SHA, persist developer-local clean exact-tree provenance:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, exit codes and clean initial/final tree. Do not use hosted CI as a substitute. Only run the pinned decoder fuzz flow if the resulting source changes decoder/parser/crypto framing; do not mechanically fuzz unchanged framing for every process-test edit.

Then continue immediately without waiting for reviewer cadence.

# Continuous queue — preserve depth

1. **R9-3 Data-loss recovery:** suppress reliable-owned Data post-admission; cwnd admission before suppression; PTO after deadline; fresh packet number/nonce retransmit with stable Session/frame identity; exactly-once Session delivery; independent ACK domains; final Recovery zero or typed bounded failure.
2. **R9-4 ACK-loss + delayed original/reorder:** suppress Carrier ACK, retransmit, then release delayed original; one logical delivery; Session dedup; fresh packet numbers/nonces; truthful loss/PTO; settled recovery.
3. **R9-5 tamper/future-ACK/malformed negatives:** no recovery/Session mutation from tamper; future/never-sent ACK atomic rejection; stale/duplicate feedback fabricates no evidence; malformed finite/panic-free.
4. **R9-6 pacing/cwnd/plaintext ownership:** all sends/retransmits consult congestion admission; refusal commits no logical/recovery ownership; one bounded retransmit plaintext owner; teardown releases state.
5. **R9-7 process/result truth:** Data, Carrier ACK, Session ACK, PTO/retransmit, recovery, Session delivery, malformed budget and final outcome remain distinct and emitted only after claimed success.
6. **R9-8 authenticated warm TCP standby:** negotiation + Noise trust/authz + resume/readiness binding + resource admission; no application Data before promotion.
7. **R9-9 resolved UDP health -> hysteresis -> real TCP promotion:** resolved outcomes drive health; recoverable loss stays UDP; PTO-only samples cannot erase later loss; only ready TCP can promote.
8. **R9-10 uncertain Session replay UDP -> TCP:** replay genuine uncertain Session range over promoted TCP; draining UDP gets no new Data; exact-once Session dedup; no TCP packet ACK.
9. **R9-11 timeout/shutdown/cleanup matrix.**
10. **R9-12 coherent exact-tree gate + independent bounded review.** Any BLOCKER/HIGH returns to smallest repair + regression + re-gate.
11. **Q10 observability reconciliation** using existing surfaces only.
12. **Q11 factual status/release reconciliation.** Only independently reviewed cross-process R9 + real TCP promotion may create a specific `READY_LIVE` row.
13. **Q12 one changed-hypothesis self-owned VPS run** only after Q11 creates that row; preserve negative evidence and cleanup; no unchanged retry.

# Core-surface review inventory reminder

The earlier repository-wide item-4 sweep already independently challenged the existing `neko-reliable`, CarrierState/CarrierManager, scheduler/flow accounting, adapters, SessionRuntime, observability, package/reproducibility, dependency/build, CLI portability/output, boundedness/validators, wire/parser and candidate PLPMTUD/FEC/disabled-gate surfaces. The **new R9 cross-process reliable-UDP integration surface is not covered by those older no-finding notes** and must receive its own bounded independent review at R9-12. Do not mistake the old inventory for coverage of new integration code.

# VPS opportunity

**Not READY — implementation dependency / independent-review dependency.** Unlock: positive P2 -> P3/P4 -> clean exact-tree provenance -> R9-3..R9-12 -> Q10/Q11. Once Q11 creates a specific changed-hypothesis `READY_LIVE` row, standing authorization already covers the bounded self-owned TCP/UDP VPS run; lack of permission is not the blocker.

# Separate non-blocking policy gates

`SessionRuntime.events` retention; D019 source retention/no-reset; RSEC-001 capacity/adversarial-load suitability; signing/key custody/SBOM/publication trust; previous frozen-release interoperability; final RC/freeze/release/production authority. Do not invent policy values while working R9.
