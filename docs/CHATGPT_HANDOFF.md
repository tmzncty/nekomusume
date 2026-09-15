# ChatGPT reviewer handoff — R9 P2 fixture-cardinality blocker; preserve full R9 queue

## Current truth

- Default branch before this refresh reached exact `412b0d2cc280534f29942294c3e295151d47610f` (`docs(review): identify deterministic P2 count=3 reachability blocker`).
- Latest developer/source-test commit remains exact `422c16b106b5568cb87f81419b45941508c2d4a4` (`fix(cli): controlled-fallback TCP replay uses ownership partition (H-R9-017)`). No developer source/test commit has landed after it.
- The prior docs-only exact `b8d256b4d541bc887ef16a9081c4e3941463c074` has hosted `stable checks` SUCCESS and `nightly decode fuzz smoke` SUCCESS. Hosted checks remain cross-evidence only.
- Open PRs: none. No new WAN/VPS experiment. `READY_LIVE: none`.
- Item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

## Implementation-stagnation classification

P2 remains dependency-ready and local/hosted tree health is not the blocker. The current P2 process fixture itself had a deterministic cardinality contradiction, identified below. This is a concrete local repair/evidence task, not a reason to wait for another reviewer cadence.

The coding agent should repair/test/push continuously, then proceed through P3/P4 and the preserved R9 queue without waiting.

## H-R9-017 — CLOSED; preserve

The controlled/non-automatic client replay count uses the same committed ownership partition as the server:

```text
uncertain_start = reliable_udp ? 2 : 1
uncertain_end   = recovery_enabled ? count - 1 : count
controlled_tcp_replay = saturating_sub(uncertain_end, uncertain_start)
```

Automatic-health mode still uses the actual tracked uncertain set from `failover.tcp_resend()` and is unchanged. Do not revert this partition and do not reintroduce guessed retries/sleeps/deadlines.

## H-R9-018 — HIGH / READY_LOCAL: current P2 `count=3` fixture cannot reach migration-back

The current `reliable_udp_migration_back_reserves_final_record` uses:

```text
count = 3
reliable_udp = true
recovery_enabled = true
```

Under the accepted ownership partition:

```text
uncertain_start = 2
uncertain_end   = count - 1 = 2
tcp_records    = 0
```

`tcp_active_sample` is initialized to `None` and is populated only inside the authenticated TCP application replay loop after a valid TCP DeliveryAck. The migration-back branch then requires:

```text
let tcp_active_sample = tcp_active_sample
    .unwrap_or_else(|| fail("missing authenticated TCP application RTT sample"));
```

before sending the UDP recovery challenge.

Therefore the existing `count=3` fixture is deterministically incapable of reaching `udp_recovery_challenge_sent`, regardless of whether the UDP recovery runtime itself is correct. This is an acceptance-fixture reachability defect, not evidence that retry/sleep/runtime semantics are missing.

Independent note: `docs/reviews/independent-r9-p2-fixture-cardinality-b8d256b-20260915.md`.

# READY_LOCAL front — close P2 with a satisfiable ownership shape

## P2-A — first action: make the migration-back fixture `count=4`

Do **not** change runtime semantics first. Change only the dedicated positive P2 process fixture from `count=3` to `count=4`, then update exact expected offsets.

With `bytes=16`, the accepted ownership partition becomes:

```text
records[0] offset 0   -> reliable UDP owned
records[1] offset 16  -> reliable UDP owned
records[2] offset 32  -> one genuine uncertain range replayed over TCP
records[3] offset 48  -> reserved for post-migration return to reliable UDP

uncertain_start = 2
uncertain_end   = 3
controlled_tcp_replay = 1
```

That single authenticated TCP replay supplies the already-existing `tcp_active_sample` without inventing a new health evidence source.

Update the P2 negative ownership assertion accordingly:

- reserved offset is now `48`, not `32`;
- offset `48` must never appear on pre-promotion `udp_uncertain_range_sent`;
- offset `48` must never be replayed over TCP;
- offset `32` is expected to be the one genuine TCP replay in this fixture.

Run the focused P2 test on this current runtime shape before any speculative runtime repair. If it is red, retain full client/server output and classify the first missing structured milestone.

## P2-B — make positive evidence non-vacuous

In `reliable_udp_migration_back_reserves_final_record`:

1. retain `server_status` and `server_log` instead of discarding them;
2. require both client and server exit success;
3. remove the conditional `if client_log.contains("r9_udp_post_return_sent")` acceptance block — P2 must fail if post-return is not reached;
4. require the exact causal milestone chain:
   - client `udp_recovery_challenge_sent`;
   - server `udp_recovery_owner_started`;
   - server `udp_recovery_validated`;
   - client `udp_recovery_validated`;
   - client `udp_migrated_back` / `migrated_back_to_udp`;
   - exactly one client `r9_udp_post_return_sent` carrying offset `48`;
   - server accepted post-return Session Data before either acknowledgement-domain send;
   - server Session DeliveryAck sent for offset `48`;
   - server Carrier packet ACK sent;
   - client exact Session confirmation for offset `48`;
   - client Carrier ACK `applied=true`;
   - client post-return Recovery settlement zero;
5. preserve the negative ownership assertion for reserved offset `48`.

Use exact structured event parsing/assertions where available; do not satisfy this with broad repository-wide substring matches.

## P2-C — add an unambiguous post-return terminal evidence point

The post-return loop exits only when `post_outstanding.is_empty()` and `rt.in_flight()==0`, but broad `remaining_in_flight=0` can match earlier settlement.

After successful post-return dual settlement, emit one dedicated diagnostic event, e.g.:

```text
r9_udp_post_return_settled
  stream=1
  offset=48
  logical_outstanding=0
  remaining_in_flight=0
```

This is evidence-only; it must not change Session, Carrier, ACK, timeout, congestion or migration semantics. P2 must require this exact event.

## P2-D — deterministic acknowledgement-domain order seam

Current server source sends post-return Session DeliveryAck before Carrier packet ACK. P2 must cover both arrival orders without timing luck.

Add one test-only server flag, e.g. `--test-post-return-carrier-ack-first`:

- default: Session DeliveryAck first, Carrier packet ACK second;
- flagged: Carrier packet ACK first, Session DeliveryAck second;
- both datagrams may be constructed/sent only **after** authenticated post-return `SessionRuntime::receive` accepts the reserved offset 48;
- do not use sleeps, retry loops, fresh deadlines or duplicated ACK logic to create the order;
- client must succeed in both orders with the same exact terminal `r9_udp_post_return_settled` evidence.

The smallest acceptable implementation is to create the two already-existing acknowledgement datagrams after accepted Session receive and swap only send order under the test flag.

## P2-E — if `count=4` is still red

Do not revive the reverted recovery-challenge retry/sleep experiment. Use the first missing structured milestone:

- no server `udp_recovery_owner_started` -> TCP replay/resume completion boundary is wrong;
- owner started but no server `udp_recovery_validated` -> challenge receive/auth/tuple path is wrong;
- server validated but client no `udp_recovery_validated` -> response emission/receive path is wrong;
- client validated but no migration promotion -> manager hold/generation state is wrong;
- promoted but no `r9_udp_post_return_sent` -> reserved-record ownership/cwnd registration is wrong;
- post-return sent but no server accepted receive -> post-return socket/auth/Session admission is wrong;
- server accepted but one ACK domain absent -> server acknowledgement ownership is wrong;
- both ACKs sent but client cannot settle -> bounded demux/Recovery settlement is wrong.

Repair only the first missing transition, add its regression, push, then continue P2 immediately.

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

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, exit codes and clean initial/final tree. Do not use hosted CI as a substitute. Only run pinned decoder fuzz if the resulting source actually changes decoder/parser/crypto framing.

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

The earlier repository-wide item-4 sweep independently challenged the pre-R9 `neko-reliable`, CarrierState/CarrierManager, scheduler/flow accounting, adapters, SessionRuntime, observability, package/reproducibility, dependency/build, CLI portability/output, boundedness/validators, wire/parser and candidate PLPMTUD/FEC/disabled-gate surfaces. The **new R9 cross-process reliable-UDP integration surface is not covered by those older no-finding notes** and must receive its own bounded independent review at R9-12.

# VPS opportunity

**Not READY — implementation dependency / independent-review dependency.** Unlock: positive P2 -> P3/P4 -> clean exact-tree provenance -> R9-3..R9-12 -> Q10/Q11. Once Q11 creates a specific changed-hypothesis `READY_LIVE` row, standing authorization already covers the bounded self-owned TCP/UDP VPS run; permission is not the blocker.

# Separate non-blocking policy gates

`SessionRuntime.events` retention; D019 source retention/no-reset; RSEC-001 capacity/adversarial-load suitability; signing/key custody/SBOM/publication trust; previous frozen-release interoperability; final RC/freeze/release/production authority. Do not invent policy values while working R9.
