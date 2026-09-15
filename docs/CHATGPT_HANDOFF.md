# ChatGPT reviewer handoff — server ownership repair exposed client replay drift; restore green tree then close P2-P4

## Current truth

- Developer/source tip reviewed: exact `97cbbfbc55adacf28724d37f38f8cfd98eb42a57` (`fix(cli): server TCP replay count derives from shared ownership partition (H-R9-016)`).
- Independent bounded recheck: `docs/reviews/r9-2h-replay-partition-regression-97cbbfb-20260915.md`.
- Reviewer report commit before this handoff: exact `e63138c31624c59a9bec08798277a04fab63f500`.
- Hosted checks on developer exact `97cbbfb`: **nightly decode fuzz smoke SUCCESS; stable checks FAILURE**. Stable fails inside `bash scripts/check.sh` in exactly two `neko-cli` process tests: `reliable_udp_failover_settles_packet_acks_to_zero_in_flight` and `reliable_udp_reversed_ack_order_confirms_in_order`.
- Open PRs: none. No new WAN/VPS experiment. `READY_LIVE: none`.
- Item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

## Accepted R9-2H facts — preserve unless contradicted

- Out-of-order Session DeliveryAck buffering cannot cumulatively confirm earlier unconfirmed bytes; exact applied evidence is stream/offset-bearing and ordered by actual Session mutation.
- One operation-wide malformed budget and one absolute application deadline; Carrier ACK remains Carrier-local; incomplete Recovery settlement is terminal.
- Post-migration reserved Data is reliable-UDP owned; server acknowledgement obligations arise only after accepted authenticated Session Data; client success requires exact Session confirmation plus Recovery `in_flight()==0`.
- P1 logic itself remains accepted: reversed logical ACK order buffers offset 16, applies offset 0, drains offset 16, and Carrier settlement reaches zero. The current process test is red only because a later controlled-TCP replay ownership mismatch causes `BrokenPipe`.
- H-R9-016 server-side formula is directionally correct for the migration/recovery case. **Do not revert it and do not reintroduce guessed retries/sleeps.**

# READY_LOCAL front — BLOCKER/HIGH before any R9 expansion

## H-R9-017 — HIGH correctness / exact-tree-gate blocker: controlled client TCP replay still ignores reliable-UDP ownership

The prior H-R9-016 repair brief was incomplete. The developer implemented the requested server partition correctly, but the non-automatic client still uses the legacy replay rule.

Current ownership partition is already committed on both sides:

```text
uncertain_start = reliable_udp ? 2 : 1
uncertain_end   = recovery_enabled ? count - 1 : count
owned_tcp_replay_count = saturating_sub(uncertain_end, uncertain_start)
```

At exact `97cbbfb`, the server now expects `owned_tcp_replay_count`, but the client still does:

```text
automatic_health_failover == true  => failover.tcp_resend().len()
automatic_health_failover == false => count - 1
```

For `count=3`, `reliable_udp=true`, `recovery=false`:

```text
server expects 1 TCP replay
client sends 2 TCP records
```

The extra record is offset 16, already owned by reliable UDP and already exactly Session-confirmed before fallback. The server completes after one expected TCP record; the client's second `write_frame` then hits `BrokenPipe`. This is exactly the hosted stable failure in both red process tests.

### Smallest repair

- Keep the server `97cbbfb` partition.
- Change only the client **non-automatic / controlled fallback** replay count from `count - 1` to the same `owned_tcp_replay_count`.
- Prefer one tiny shared helper/calculation used by server expected count and client controlled fallback so the four-mode table cannot drift. Do not create a framework.
- Automatic-health replay remains `failover.tcp_resend().len()` because that is the actual tracked uncertain set; add a deterministic assertion/test that its covered fixture ownership agrees with the same partition.
- No new retries, sleeps, deadlines, wire messages, capacity values, Session semantics or ACK semantics.

Pin `count=3`:

| reliable UDP | recovery/migration reservation | expected TCP replay |
|---|---|---:|
| false | false | 2 |
| false | true  | 1 |
| true  | false | 1 |
| true  | true  | 0 |

### Immediate gate after H-R9-017

Run at minimum the two currently red process tests, then the normal clean exact-tree gate. Do not advance to new R9 functionality while `scripts/check.sh` is red.

Required preservation:

- `reliable_udp_failover_settles_packet_acks_to_zero_in_flight` succeeds with one TCP replay for the remaining uncertain offset 32, not two;
- `reliable_udp_reversed_ack_order_confirms_in_order` succeeds and retains exact two logical confirmations + Recovery zero settlement;
- non-reliable controlled fallback still replays two records at `count=3`;
- no TCP packet-ACK layer is introduced.

# P2 — mandatory positive post-migration reserved-record dual settlement

The existing `reliable_udp_migration_back_reserves_final_record` happened to pass in hosted CI at `97cbbfb`, but it is still **not acceptance evidence**: the source explicitly allows nonzero client exit and only asserts full post-return evidence conditionally if `r9_udp_post_return_sent` appears.

After H-R9-017 restores a green tree, make P2 non-vacuous:

1. require both client and server success and retain both logs;
2. require client `udp_recovery_challenge_sent` -> server `udp_recovery_owner_started` -> server `udp_recovery_validated` -> client `udp_recovery_validated` -> migration-back promotion;
3. exactly one post-return reliable send for stream 1 / offset 32, never on the pre-promotion uncertain path;
4. promotion precedes reliable ownership/send;
5. accepted server Session receive for offset 32 precedes creation/emission of both Session DeliveryAck and Carrier ACK; add at most one narrow accepted-receive diagnostic if needed;
6. exact Session confirmation for offset 32 is applied;
7. independent post-return Carrier ACK is applied;
8. post-return terminal Recovery evidence is `remaining_in_flight=0` before success;
9. exercise Session-ACK-first and Carrier-ACK-first with a bounded test-only ordering seam, never sleeps;
10. neither acknowledgement domain substitutes for the other.

Do not change Session/Carrier/ACK/wire architecture.

# P3 — persistent malformed budget

Within one reliable receive/settlement operation, drive exactly:

```text
malformed #1 -> malformed #2 -> canonical Carrier ACK -> malformed #3
```

Malformed #3 must hit the existing `MAX_POST_HANDSHAKE_MALFORMED`; Carrier ACK must not reset the counter; termination typed, finite and non-spinning. Do not change the numeric limit.

# P4 — incomplete settlement remains terminal

Preserve the existing incomplete-settlement negative and extend it to the post-migration reserved owner by suppressing one acknowledgement domain. Require nonzero terminal result, no settled/success claim, and no downstream health/failover continuation. Reuse existing budgets.

# R9-2H closure gate

After H-R9-017 + positive P2 + P3/P4 are on one reachable source/test SHA, persist developer-local clean exact-tree provenance:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, exit codes and clean initial/final tree. The H-R9-017 front itself changes no decoder/parser/framing grammar; do not invent extra fuzz work. Hosted CI is cross-evidence only.

Then **continue immediately** without waiting for reviewer cadence.

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
12. **Q11 factual status/release reconciliation**; only independently reviewed cross-process R9 + real TCP promotion may create a specific `READY_LIVE` row.
13. **Q12 one changed-hypothesis self-owned VPS run** only after Q11 creates that row; preserve negative evidence and cleanup; no unchanged retry.

# VPS opportunity

**Not READY — implementation dependency.** Current exact tree is red and P2 is still conditional. Unlock: H-R9-017 -> green tree -> positive P2 -> P3/P4 -> clean exact-tree provenance -> R9-3..R9-12 -> Q10/Q11. Standing authorization already covers the eventual bounded self-owned TCP/UDP experiment; lack of permission is not the blocker.

# Separate non-blocking policy gates

`SessionRuntime.events` retention; D019 source retention/no-reset; RSEC-001 capacity/adversarial-load suitability; signing/key custody/SBOM/publication trust; previous frozen-release interoperability; final RC/freeze/release/production authority. Do not invent policy values while working R9.
