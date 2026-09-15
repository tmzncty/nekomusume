# ChatGPT reviewer handoff — R9 TCP replay identity HIGH; positive P2 remains READY_LOCAL

## Current repository truth

- Default branch before this refresh reached exact `cd9a8d65ba660502e213a5265900d7b8fd24569f` (`docs(review): identify exact TCP replay identity drift in R9 P2`).
- Latest developer/source-test commit remains exact `422c16b106b5568cb87f81419b45941508c2d4a4` (`fix(cli): controlled-fallback TCP replay uses ownership partition (H-R9-017)`). No developer source/test commit has landed after it.
- Exact docs-only `62f7f108ff28f171675e773eded2f9644f32f57f` has hosted `stable checks` SUCCESS and `nightly decode fuzz smoke` SUCCESS. Hosted checks remain cross-evidence only; they do not replace developer-local exact-tree provenance.
- Open PRs: none. The historical `work/r9-2e-demux-20260914` branch is stale at `42f0c9f` and is not a current resume trigger.
- No new WAN/VPS experiment. `READY_LIVE: none`.
- Item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

## Stagnation classification

R9 remains dependency-ready. Green hosted checks, repository access and standing authorization are not blockers. The coding agent must continue implementation/test/push through the READY_LOCAL front and then the preserved R9 queue without waiting for reviewer cadence.

The previous handoff correctly identified that the dedicated migration-back P2 fixture with `count=3` cannot create a TCP application RTT sample. A deeper current-source review found a prior ownership defect that must be repaired **before** merely changing that fixture to count 4.

## H-R9-017 — CLOSED AS CARDINALITY ONLY; do not revert

The controlled/non-automatic client replay count now uses the same partition cardinality as the server:

```text
uncertain_start = reliable_udp ? 2 : 1
uncertain_end   = recovery_enabled ? count - 1 : count
controlled_tcp_replay = uncertain_end - uncertain_start
```

This fixes the earlier producer/consumer **count** drift. Preserve it. It did not, however, fix which logical records are selected for TCP replay.

# H-R9-019 — HIGH / READY_LOCAL: exact TCP replay identity is still wrong

Current client source computes the correct `tcp_records` count but still sends:

```rust
for record in records.into_iter().skip(1).take(tcp_records) {
    ...
}
```

That positional prefix ignores `uncertain_start` and ignores the exact ownership returned by `failover.tcp_resend()`.

### Deterministic P2 consequence

For the intended positive P2 shape:

```text
count=4
bytes=16
reliable_udp=true
recovery_enabled=true

records[0] offset 0  -> reliable UDP owned
records[1] offset 16 -> reliable UDP owned
records[2] offset 32 -> genuine uncertain range; exactly one TCP replay
records[3] offset 48 -> reserved for post-migration reliable UDP

uncertain_start = 2
uncertain_end   = 3
tcp_records     = 1
```

The current `skip(1).take(1)` loop replays offset **16**, not offset **32**. The actual uncertain record is skipped while an already reliable-UDP-owned range crosses onto TCP.

Therefore **do not treat “change P2 count 3 -> 4” as sufficient on the current source**.

### Automatic-health mode is also affected

`FailoverController::tcp_resend()` already returns the exact uncertain set as `(DataId, payload)` entries. The client currently throws that identity away and keeps only `.len()`, then sends the same guessed positional prefix. Equal cardinality is not equal ownership.

Independent note: `docs/reviews/independent-r9-tcp-replay-identity-62f7f10-20260915.md`.

# READY_LOCAL front — repair exact replay ownership, then close positive P2

## R9-2I-A — exact replay selection (first action)

Do not change Session, Carrier, ACK, wire, timing, retry or policy semantics. Repair only TCP replay selection.

1. Build one bounded explicit `tcp_replay_records` sequence before the TCP application replay loop.
2. **Controlled/non-automatic mode:** select exactly `records[uncertain_start..uncertain_end]`, or the equivalent `skip(uncertain_start).take(uncertain_end - uncertain_start)` iterator.
3. **Automatic-health mode:** preserve the exact `(DataId, payload)` values returned by `failover.tcp_resend()`; map each entry back to exactly one original logical record by stable identity and require byte-exact payload equality. Missing, duplicate or mismatched ownership must fail closed.
4. Iterate only this exact replay sequence. Remove the semantic dependence on `skip(1)`.
5. Preserve the reserved final-record exclusion and the H-R9-017 replay-count partition.
6. If no existing structured event precisely names the replayed logical range, add one diagnostic-only event at actual TCP send (stream, offset, len, controlled/automatic source). It must not mutate protocol state.

Required focused regressions before P2:

- reliable controlled fallback with `uncertain_start=2`: first TCP replay is record index 2, never index 1;
- automatic-health fallback: emitted/sent TCP ranges equal the exact `tcp_resend()` DataIds/payloads, not merely an equal-sized prefix;
- no reliable-UDP-owned or migration-back-reserved record appears in the TCP replay set.

Push this repair + regressions and continue immediately.

## R9-2I-B — make the positive migration-back P2 fixture satisfiable

After H-R9-019 is closed, change only the dedicated positive P2 process fixture from `count=3` to `count=4` and update exact offsets.

Expected ownership:

```text
0,16  -> reliable UDP
32    -> exactly one authenticated TCP replay
48    -> reserved final record for post-migration reliable UDP
```

Require exactly one TCP replay at offset 32. Offset 48 must never be replayed over TCP and must never appear on the pre-promotion `udp_uncertain_range_sent` path.

The one TCP replay supplies the existing `tcp_active_sample`; do not invent a new health evidence source and do not revive the reverted recovery-challenge retry/sleep experiment.

## R9-2I-C — positive P2 must be non-vacuous

In `reliable_udp_migration_back_reserves_final_record`:

1. retain `server_status` and `server_log`;
2. require client and server exit success;
3. remove the conditional acceptance around `r9_udp_post_return_sent`;
4. require the exact causal chain:
   - client `udp_recovery_challenge_sent`;
   - server `udp_recovery_owner_started`;
   - server `udp_recovery_validated`;
   - client `udp_recovery_validated`;
   - client migration promotion (`udp_migrated_back` / equivalent current exact event);
   - exactly one client post-return reliable send at offset 48;
   - server authenticated Session receive accepts offset 48 before either acknowledgement-domain send;
   - server Session DeliveryAck for offset 48;
   - server Carrier packet ACK;
   - client exact Session confirmation for offset 48;
   - client Carrier ACK `applied=true`;
   - final post-return Recovery `in_flight==0`.
5. Use exact structured event parsing/assertions rather than broad substring coincidence.

## R9-2I-D — dedicated post-return settled evidence

After the post-return receive owner proves both exact logical confirmation and Carrier recovery settlement, emit one diagnostic-only terminal event such as:

```text
r9_udp_post_return_settled
  stream=1
  offset=48
  logical_outstanding=0
  remaining_in_flight=0
```

P2 must require this exact event. Do not emit it on timeout, malformed exhaustion or partial settlement.

## R9-2I-E — deterministic acknowledgement-domain order seam

The post-return path must succeed in both acknowledgement arrival orders without timing luck.

Use one test-only server switch (or the already-equivalent current seam) that changes only send order **after** accepted authenticated Session receive:

- default: Session DeliveryAck then Carrier packet ACK;
- alternate: Carrier packet ACK then Session DeliveryAck.

Both runs must end with the same exact post-return settled event. No sleeps, extra retries, fresh deadlines or duplicated ACK logic.

## R9-2I-F — if positive count=4 is still red

Classify the first missing structured milestone and repair only that transition:

- no exact TCP offset-32 replay -> replay ownership selection still wrong;
- no server TCP acceptance/ack -> TCP resume/application boundary wrong;
- no server `udp_recovery_owner_started` -> TCP replay/resume completion boundary wrong;
- owner started, no server validation -> challenge receive/auth/tuple path wrong;
- server validated, client not validated -> response emission/receive wrong;
- client validated, no migration promotion -> manager hold/generation state wrong;
- promoted, no post-return send -> reserved-record/cwnd/recovery ownership wrong;
- post-return sent, no server accepted receive -> post-return socket/auth/Session admission wrong;
- server accepted, one ACK domain absent -> server acknowledgement ownership wrong;
- both ACKs sent, client cannot settle -> bounded demux/recovery settlement wrong.

Do not guess retry/sleep fixes before identifying the first missing state transition.

# P3 — persistent malformed budget across Carrier feedback

Within one reliable receive/settlement operation, drive exactly:

```text
malformed #1 -> malformed #2 -> canonical Carrier ACK -> malformed #3
```

Malformed #3 must hit the existing `MAX_POST_HANDSHAKE_MALFORMED`; the valid Carrier ACK must not reset the operation-wide malformed counter. Termination must be typed, finite and non-spinning. Do not change the numeric limit or create a second settlement budget.

Prefer a bounded authenticated test injection seam; the Carrier ACK must be canonical and handled by the current reliable demux owner.

# P4 — incomplete settlement remains terminal

Preserve/extend the incomplete-settlement negative to the post-migration reserved owner. Suppress one acknowledgement domain at a time and require:

- nonzero/typed terminal result;
- no `r9_udp_in_flight_settled`, `r9_udp_post_return_settled`, or equivalent success while Recovery remains in flight or exact logical confirmation remains outstanding;
- no downstream health/failover continuation from an unproven settled state;
- no new deadline after the operation-wide deadline expires.

# R9-2H/I closure gate

After H-R9-019 + positive P2 + P3/P4 are all on one reachable source/test SHA, persist developer-local clean exact-tree provenance:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, command exit codes and clean initial/final tree. Hosted CI is additional cross-evidence only. Run pinned decoder fuzz only if decoder/parser/crypto framing actually changed.

Then continue immediately without waiting for reviewer cadence.

# Continuous queue — preserve depth

1. **R9-3 Data-loss recovery:** suppress reliable-owned Data post-admission; cwnd admission before suppression; PTO after deadline; fresh packet number/nonce retransmit with stable Session/frame identity; exactly-once Session delivery; independent ACK domains; final Recovery zero or typed bounded failure.
2. **R9-4 ACK-loss + delayed original/reorder:** suppress Carrier ACK, retransmit, release delayed original; one logical delivery; Session dedup; fresh packet numbers/nonces; truthful loss/PTO; settled recovery.
3. **R9-5 tamper/future-ACK/malformed negatives:** tamper mutates neither recovery nor Session; future/never-sent ACK atomic rejection; stale/duplicate feedback fabricates no evidence; malformed finite/panic-free.
4. **R9-6 pacing/cwnd/plaintext ownership:** every send/retransmit consults congestion admission; refusal commits no logical/recovery ownership; one bounded retransmit plaintext owner; teardown releases state.
5. **R9-7 process/result truth:** Data, Carrier ACK, Session ACK, PTO/retransmit, recovery, Session delivery, malformed budget and final outcome stay distinct and are emitted only after the claimed state transition.
6. **R9-8 authenticated warm TCP standby:** negotiation + Noise trust/authz + resume/readiness binding + resource admission; no application Data before promotion.
7. **R9-9 resolved UDP health -> hysteresis -> real TCP promotion:** resolved packet outcomes drive health; recoverable loss stays UDP; PTO-only samples cannot erase later loss; only ready TCP can promote.
8. **R9-10 uncertain Session replay UDP -> TCP:** replay the exact genuine uncertain Session range over promoted TCP; draining UDP gets no new Data; exact-once Session dedup; no TCP packet ACK.
9. **R9-11 timeout/shutdown/cleanup matrix.**
10. **R9-12 coherent exact-tree gate + independent bounded review.** Any BLOCKER/HIGH returns to smallest repair + regression + re-gate.
11. **Q10 observability reconciliation** using existing surfaces only.
12. **Q11 factual status/release reconciliation.** Only independently reviewed cross-process R9 + real TCP promotion may create a specific `READY_LIVE` row.
13. **Q12 one changed-hypothesis self-owned VPS run** only after Q11 creates that row; preserve negative evidence and cleanup; no unchanged retry.

# Core-surface review inventory reminder

The earlier item-4 sweep challenged the pre-R9 reliable engine, CarrierState/CarrierManager, scheduler/flow accounting, adapters, SessionRuntime, observability, package/reproducibility, dependency/build, CLI portability/output, boundedness/validators, wire/parser and PLPMTUD/FEC/disabled-gate candidates. The **new cross-process R9 integration surface is not covered by those earlier notes** and still requires its own bounded independent review at R9-12.

# VPS opportunity

**Not READY — implementation/correctness + independent-review dependency.** Unlock sequence: H-R9-019 -> positive P2 -> P3/P4 -> exact-tree provenance -> R9-3..R9-12 -> Q10/Q11. If Q11 creates a specific changed-hypothesis `READY_LIVE` row, standing authorization already covers the bounded self-owned TCP/UDP VPS run; permission is not the blocker.

# Separate non-blocking policy gates

`SessionRuntime.events` retention; D019 source retention/no-reset; RSEC-001 adversarial-load/capacity suitability; signing/key custody/SBOM/publication trust; previous frozen-release interoperability; final RC/freeze/release/production authority. Do not invent policy values while working R9.
