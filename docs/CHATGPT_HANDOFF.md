# ChatGPT reviewer handoff — P2 implementation stagnation breaker; preserve full R9 queue

## Current truth

- Default branch HEAD at this refresh: exact `f3f777c448a1529b653f4c0b5b31f375c1a681fa` (`docs(handoff): accept H-R9-017 and advance positive R9-2H evidence`).
- Latest developer/source-test commit remains exact `422c16b106b5568cb87f81419b45941508c2d4a4` (`fix(cli): controlled-fallback TCP replay uses ownership partition (H-R9-017)`). No developer source/test commit has landed after it.
- Hosted checks are green on both developer exact `422c16b` and current docs-only HEAD: `stable checks` SUCCESS and `nightly decode fuzz smoke` SUCCESS. Hosted checks remain cross-evidence only.
- Open PRs: none. No new WAN/VPS experiment. `READY_LIVE: none`.
- Item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

## Implementation-stagnation classification

P2 is dependency-ready, current exact-head CI is green, no external permission/environment blocker applies, and the queue behind it is deep. One reviewer cadence has passed without a new developer source/test commit. Per `AGENTS.md` this is implementation stagnation, not a reason to wait.

The handoff is therefore sharpened below to a concrete test/evidence ownership contract. The coding agent should implement/test/push continuously; do not wait for another reviewer pass.

## H-R9-017 — CLOSED; preserve

The controlled/non-automatic client replay count now uses the same committed ownership partition as the server:

```text
uncertain_start = reliable_udp ? 2 : 1
uncertain_end   = recovery_enabled ? count - 1 : count
controlled_tcp_replay = saturating_sub(uncertain_end, uncertain_start)
```

Automatic-health mode still uses the actual tracked uncertain set from `failover.tcp_resend()` and is unchanged.

For `count=3`, controlled replay remains `2 / 1 / 1 / 0` for `(reliable,recovery) = (F,F)/(F,T)/(T,F)/(T,T)`. Do not revert this partition and do not reintroduce guessed retries/sleeps/deadlines.

## Accepted R9-2H facts — preserve unless contradicted

- Out-of-order Session DeliveryAck buffering cannot cumulatively confirm earlier unconfirmed bytes; exact applied evidence is stream/offset-bearing and ordered by actual Session mutation.
- One operation-wide malformed budget and one absolute application deadline; Carrier ACK remains Carrier-local; incomplete Recovery settlement is terminal.
- Post-migration reserved Data is reliable-UDP owned; server acknowledgement obligations arise only after accepted authenticated Session Data; client success requires exact Session confirmation plus Recovery `in_flight()==0`.
- P1 is accepted: reversed logical ACK order buffers offset 16, applies offset 0, drains offset 16, and Carrier settlement reaches zero on a successful built-binary path.
- H-R9-015 source shape is accepted: the reserved final record is Recovery-owned after migration-back, and server/client retain separate Session-ACK and Carrier-ACK obligations.

# READY_LOCAL front — P2 first, then P3/P4

## P2-A — first action: make the existing process test non-vacuous before speculative runtime changes

The current source already contains the full nominal causal path:

- client emits `udp_recovery_challenge_sent`;
- server enters `udp_recovery_owner_started`, accepts the authenticated readiness request, emits `udp_recovery_validated`, and waits for exactly one bounded post-return application datagram;
- client validates recovery, passes the manager hold gate, applies migration-back, registers the reserved offset-32 Data in `ReliableUdpRuntime`, sends it, and waits for both Session DeliveryAck and Carrier ACK;
- server records the accepted post-return packet in `server_rt`, then emits separate Session DeliveryAck and canonical Carrier packet ACK.

The remaining acceptance gap is presently in the process test/evidence, not a proven missing runtime transition. Therefore **do not change runtime semantics first**.

In `reliable_udp_migration_back_reserves_final_record`:

1. retain `server_status` and `server_log` instead of discarding them;
2. require both client and server exit success — remove the current comment/behavior that treats a nonzero client exit as acceptable;
3. delete the conditional `if client_log.contains("r9_udp_post_return_sent")` acceptance block; P2 must fail if the post-return path is not reached;
4. require the causal milestone chain, using exact structured lines where possible:
   - client `udp_recovery_challenge_sent`;
   - server `udp_recovery_owner_started`;
   - server `udp_recovery_validated`;
   - client `udp_recovery_validated`;
   - client `udp_migrated_back` / `migrated_back_to_udp`;
   - exactly one client `r9_udp_post_return_sent` carrying offset `32`;
   - server accepted post-return Session Data before either acknowledgement-domain send;
   - server Session DeliveryAck sent for offset `32`;
   - server Carrier packet ACK sent;
   - client exact Session confirmation for offset `32`;
   - client Carrier ACK `applied=true`;
   - client post-return Recovery settlement zero;
5. preserve the negative ownership assertion: offset `32` must never appear on `udp_uncertain_range_sent` and must never be replayed over TCP.

Run this focused test against the **current source shape first**. If it is green, commit the strengthened positive evidence without inventing a runtime fix. If it is red, retain the full client/server logs and classify the first missing milestone; repair only that exact transition.

## P2-B — add an unambiguous post-return terminal evidence point

The current post-return loop exits only when `post_outstanding.is_empty()` and `rt.in_flight()==0`, but the existing P2 assertion for a broad `remaining_in_flight=0` can accidentally match the earlier pre-failover R9 settlement.

After the post-return dual-settlement loop succeeds, emit one dedicated structured event, for example:

```text
r9_udp_post_return_settled
  stream=1
  offset=32
  logical_outstanding=0
  remaining_in_flight=0
```

This event is diagnostic/evidence only; it must not alter Session, Carrier, ACK, timeout, congestion, or migration semantics. P2 must require this exact terminal event rather than a repository-wide substring match.

## P2-C — deterministic acknowledgement-domain order seam

Current server source emits the post-return Session DeliveryAck before the Carrier packet ACK. P2 requires both arrival orders without timing luck.

Add one **test-only ordering flag** on the failover server, e.g. `--test-post-return-carrier-ack-first`:

- default path: Session DeliveryAck first, Carrier packet ACK second;
- test flag: Carrier packet ACK first, Session DeliveryAck second;
- both datagrams may be created/sent only **after** authenticated post-return `SessionRuntime::receive` accepts offset 32;
- do not use sleeps, retry loops, fresh deadlines, or duplicated ACK logic to create the order;
- the client must succeed in both orders with the same exact terminal `r9_udp_post_return_settled` evidence.

The smallest acceptable implementation is to construct both already-existing acknowledgement datagrams after accepted Session receive and swap only their send order under the test flag.

## P2-D — if the strengthened current-shape test is red

Do not revive the reverted recovery-challenge retry/sleep experiment. Use the first missing structured milestone to localize the failure:

- no server `udp_recovery_owner_started` -> TCP resume/replay ownership boundary is still wrong;
- owner started but no server `udp_recovery_validated` -> challenge receive/auth/tuple path is wrong;
- server validated but client no `udp_recovery_validated` -> response emission/receive path is wrong;
- client validated but no migration promotion -> manager hold/generation state is wrong;
- promoted but no `r9_udp_post_return_sent` -> reserved-record ownership/cwnd registration is wrong;
- post-return sent but no server accepted receive -> post-return socket/auth/Session admission is wrong;
- server accepted but one ACK domain absent -> server acknowledgement ownership is wrong;
- both ACKs sent but client cannot settle -> bounded demux/Recovery settlement is wrong.

Make the smallest semantics-preserving repair for the first missing milestone, add the corresponding regression, push, then immediately continue P2.

## P3 — persistent malformed budget across Carrier feedback

Within one reliable receive/settlement operation, drive exactly:

```text
malformed #1 -> malformed #2 -> canonical Carrier ACK -> malformed #3
```

Malformed #3 must hit the existing `MAX_POST_HANDSHAKE_MALFORMED`; the valid Carrier ACK must not reset the operation-wide malformed counter. Termination must be typed, finite and non-spinning. Do not change the numeric limit and do not create a second malformed budget for settlement.

Prefer a bounded test-only server injection seam that emits this exact sequence after authentication; do not rely on sleeps/races. The Carrier ACK must be a real canonical ACK handled through the existing reliable demux owner.

## P4 — incomplete settlement remains terminal

Preserve the existing incomplete-settlement negative and extend it to the post-migration reserved owner. Suppress one acknowledgement domain at a time and require:

- nonzero/typed terminal outcome;
- no `r9_udp_in_flight_settled`, `r9_udp_post_return_settled`, or equivalent success claim while `in_flight()!=0` or exact logical confirmation remains outstanding;
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
