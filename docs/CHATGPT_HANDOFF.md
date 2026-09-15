# ChatGPT reviewer handoff — P2 retry experiment reviewed; positive P2 closure remains READY_LOCAL

## Current repository truth

- Current reviewer tip before this handoff refresh: exact `763b5e9bb3d2613d62e5a4423eefa2e55ccc08d1` (`docs(review): keep P2 open after recovery retry experiment`). Its parent developer source/test SHA is exact `802ef95fcc4e74be84d7e4aedb47c792c1498aad` (`fix(cli): retry UDP recovery challenge across bounded deadline (P2 partial)`).
- New developer change since the prior reviewer handoff: `802ef95` changes only `crates/neko-cli/src/main.rs`; it does not strengthen `crates/neko-cli/tests/probe.rs::reliable_udp_migration_back_reserves_final_record`, which remains the conditional exact-`605ab06` P2 test.
- Independent bounded recheck: `docs/reviews/r9-2h-p2-retry-802ef95-20260915.md` at reviewer exact `763b5e9`.
- Hosted GitHub checks on exact `802ef95`: `stable checks` success; `nightly decode fuzz smoke` success. Hosted checks are supplementary only; this partial slice has no new persisted developer-local exact-tree provenance.
- Open PRs: none. No new WAN/VPS experiment. `READY_LIVE: none` remains authoritative.
- Governance unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

## Reviewer verdict on `802ef95`

**PARTIAL / NOT ACCEPTANCE.** The retry experiment may help diagnose the migration-back fixture stall, but P2 is still not a positive built-binary proof.

The source now re-sends the same sealed UDP recovery ReadinessRequest after 500 ms receive timeouts, stopping at the earlier of the existing roughly-three-second absolute recovery deadline or six send attempts. The existing P2 process test is unchanged: it still ignores both process exit statuses, still permits the fixture never to reach post-return, and still gates all positive assertions behind optional `if r9_udp_post_return_sent` execution.

### M-R9-009 — ad-hoc recovery retry policy must not become accepted semantics

The current handoff explicitly prohibited inventing a new retry numeric policy. Exact `802ef95` nevertheless adds a new 500 ms cadence and six-attempt cap in the production failover client.

It also re-sends the exact same authenticated ciphertext. `SecureSession::open_unreliable` advances a replay window after successful authentication/context validation; therefore, if the server accepts one request but its response is lost, subsequent copies of that same sealed record are replay-rejected. This is not a general authenticated request-retry contract and must not be treated as one.

The comment that the server may still be in the TCP readiness loop is not itself proof that retransmission is required: the UDP socket is already bound and a tiny local datagram can queue until the recovery owner reads it. Before retaining any production retry behavior, identify the actual first missing stage from process evidence.

**Required disposition:** either remove the guessed production retry workaround and solve deterministic P2 orchestration with a bounded test-only seam, or replace it with a separately coherent recovery-probe behavior that reuses existing accepted deadline constants and has deterministic replay/duplicate/late-response regressions. Do not keep the new 500 ms / six-attempt policy merely because hosted CI is green.

## Accepted R9-2H progress — do not reopen absent contradictory evidence

- H-R9-011: out-of-order later Session DeliveryAck is buffered and cannot cumulatively confirm earlier unconfirmed bytes.
- H-R9-012/H-R9-013/H-R9-014: applied Session ACK evidence follows actual mutation order; direct and buffered applied evidence carries exact `stream + offset`, buffered application marks `buffered=true`; logical `outstanding + pending_acks` retires before Carrier-only settlement.
- One operation-wide malformed counter and one absolute application deadline; Carrier packet ACK remains Carrier-local; incomplete Recovery settlement is terminal and cannot feed downstream health/failover as success.
- `940b17f` + `34ae142`: the reserved final record after successful migration-back is owned by the existing reliable-UDP runtime; server creates Carrier ACK obligation only after authenticated/opened/decoded/correct-Session accepted Data; client completion requires both exact Session confirmation and Recovery `in_flight()==0`; Session DeliveryAck and Carrier ACK remain separate evidence domains.
- **P1 ACCEPT at `76c3128`:** the built client exits success; exactly one buffered offset-16/watermark-0 event is present; the covered shortcut is absent; exactly two applied confirmation events occur in real mutation order (offset 0 then buffered offset 16); terminal structured Recovery settlement reports `remaining_in_flight=0`.

# R9-2H evidence front — READY_LOCAL, execute continuously

## P2 — exact reserved post-migration ownership + dual settlement — CURRENT FRONT

### Exact current owners

Primary implementation owner: `crates/neko-cli/src/main.rs`.

Current source already has the intended post-return ownership shape:

- after `migrate_back_to_udp` + `apply_migration_back`, client builds the reserved post-return Data, runs reliable-UDP `can_send`, calls `on_packet_sent` with stable `FrameId(post_record.offset)`, then sends the authenticated UDP datagram;
- server authenticates/decodes the post-return Data and successfully calls `SessionRuntime::receive` before creating either acknowledgement domain;
- in reliable mode the server then calls `server_rt.on_packet_received`, sends the independent Session DeliveryAck, and emits the canonical Carrier packet ACK from `poll_outgoing_ack`;
- client post-return receive waits until both logical outstanding is retired and `rt.in_flight()==0`, accepting either Session or Carrier acknowledgement through the existing bounded demux helper.

Primary test owner: `crates/neko-cli/tests/probe.rs::reliable_udp_migration_back_reserves_final_record`.

At current HEAD that test is still conditional and therefore not acceptance evidence. Treat exact `802ef95` as a diagnostic implementation experiment only.

### P2 closure sequence

1. **Make the built-binary test non-vacuous before claiming success.** Preserve both client and server statuses and full logs. Require client success and server success.
2. **Pin the actual recovery-owner handoff.** Use existing structured stages to determine the first missing point: client `udp_recovery_challenge_sent`; server `udp_recovery_owner_started`; server `udp_recovery_validated`; client `udp_recovery_validated`; migration-back promotion. Add at most one narrow evidence-only event if the existing stages cannot distinguish the stall. Do not add another logging framework.
3. **Dispose M-R9-009.** Do not preserve a new 500 ms / six-attempt production retry policy without a separately reviewed need and regression. Prefer the existing accepted readiness/deadline constants and minimal state, or a test-only deterministic orchestration seam when the problem is fixture timing.
4. **Post-return is mandatory.** Require exactly one `r9_udp_post_return_sent` for exact `stream=1, offset=32`; no conditional `if present` acceptance.
5. **No pre-promotion reserved ownership.** Offset 32 must not be Recovery-tracked/sent before successful migration-back and must never use the legacy uncertain owner.
6. **Promotion precedes reliable send.** Structured migration-back evidence must precede reserved reliable ownership/send.
7. **Server receive precedes both ACK obligations.** Authenticated/correct-Session post-return Data must be accepted by `SessionRuntime::receive` before the Session DeliveryAck and Carrier ACK are created/emitted. Add one narrow exact `stream/offset` receive-success diagnostic if required to prove ordering.
8. **Exact Session confirmation.** Client must show the post-return Session confirmation for `stream=1, offset=32` was actually applied; generic delivery-success text is insufficient.
9. **Independent Carrier settlement.** Client must separately apply the post-return Carrier ACK and emit a post-return-scoped terminal event with `remaining_in_flight=0` before success. Earlier recovery-settlement text cannot satisfy this assertion.
10. **Both arrival orders.** Exercise Session-ACK-first and Carrier-ACK-first through a bounded test-only send-order seam, not timing sleeps. Both runs must execute the same production migration-back, reliable ownership, Session receive and dual-settlement code.
11. **No domain collapse.** Session DeliveryAck cannot substitute for Carrier settlement and Carrier ACK cannot mutate Session confirmation state.

Do not change Session/Carrier/ACK/wire architecture. Do not add a new capacity, TTL, retry, history or security numeric policy.

## P3 — persistent malformed budget across valid Carrier feedback

Drive one reliable receive/settlement operation with authenticated sequence:

```text
malformed #1
malformed #2
canonical Carrier ACK
malformed #3
```

Malformed #3 must hit the existing `MAX_POST_HANDSHAKE_MALFORMED` ceiling. The valid Carrier ACK must not reset the operation-wide malformed counter. Termination must be typed, finite and non-spinning. A bounded test-only server seam is allowed; do not change the numeric limit.

## P4 — preserve incomplete settlement terminality, including post-return

Keep the existing `reliable_udp_incomplete_settlement_fails_not_settled` negative: remaining in-flight != 0 is terminal/nonzero, emits incomplete, never emits `r9_udp_in_flight_settled`, and cannot feed downstream health/failover from a false premise.

Extend the same truth boundary to the post-migration reserved-record owner by suppressing/missing one of the two independent evidence domains. Require nonzero terminal outcome, no post-return settled/success claim, and no downstream continuation based on incomplete ownership. Reuse existing finite deadlines/budgets; no new capacity policy.

## R9-2H exact-tree gate

After P2-P4 land on one reachable source/test SHA, persist developer-local clean exact-tree provenance from a safe worktree:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, both exit codes, and clean initial/final tree. No decoder/parser/crypto-framing change is required by the current evidence front; do not invent a new fuzz obligation.

Then continue immediately to R9-3.

# Continuous queue after R9-2H

Preserve this deep queue; do not collapse it after one green test.

1. **R9-3 process Data-loss recovery:** deterministic post-admission suppression of one/periodic reliable-owned Data packet; cwnd admission before suppression; PTO only after deadline; retransmit with fresh authenticated packet number/nonce and stable Session/frame identity; exactly-once application delivery; independent Carrier ACK and Session DeliveryAck settlement; final Recovery zero or explicit bounded incomplete/error.
2. **R9-4 ACK-loss + reorder/delayed original:** suppress Carrier ACK, retransmit before delayed original, then delayed original; one logical Session delivery, Session-owned duplicate suppression, fresh packet numbers/nonces, truthful loss/PTO counters, final settlement.
3. **R9-5 tamper/future-ACK/malformed negatives:** tampered Data creates no Carrier ACK obligation or Session receive; tampered ACK creates no recovery mutation; future/never-sent ACK rejected atomically; stale/duplicate feedback fabricates no RTT/loss/Session evidence; malformed feedback finite/panic-free.
4. **R9-6 pacing/cwnd/plaintext-owner atomicity:** initial and retransmit sends consult congestion admission; refusal commits no Session byte space or recovery/plaintext owner; one bounded retransmit plaintext owner; finite pacing deadlines; teardown releases ownership.
5. **R9-7 process observability/result truth:** Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, resolved recovery, Session delivery, malformed-budget, cleanup and final-outcome evidence remain distinct; events increment only after the action they claim succeeds.
6. **R9-8 actual authenticated warm TCP standby:** existing TCP connect/accept + canonical negotiation + fresh Noise trust/authz + resume/readiness tuple/generation/delivery-epoch binding + resource admission; warm carries no application Data before atomic promotion.
7. **R9-9 resolved UDP health -> hysteresis -> real TCP promotion:** fresh resolved outcomes drive health; recoverable loss stays UDP; PTO-only observation cannot erase later resolved loss; committed hysteresis promotes only to genuinely ready TCP standby; invalid/unready standby typed failure.
8. **R9-10 uncertain Session replay UDP -> TCP:** replay at least one genuinely uncertain Session range over promoted TCP; draining UDP accepts no new application Data; Session/stream/offset dedup keeps application bytes exactly once; no TCP packet ACK.
9. **R9-11 timeout/shutdown/cleanup matrix:** setup/application/PTO/single-owner receive deadline, controlled stop, failed standby promotion, malformed/tampered input, listener/socket/process cleanup and same-address rebind where supported.
10. **R9-12 coherent exact-tree gate + independent bounded review:** challenge Session-above-Carrier layering, exact Session ACK evidence, authenticated packet identity/Carrier ACK, bounded plaintext/pending ownership, single receive-owner classification/deadline/malformed ownership, PTO/pacing/cwnd, health/hysteresis, warm TCP readiness/promotion, uncertain replay/dedup, result truthfulness and cleanup/no-public-exposure. BLOCKER/HIGH -> smallest repair + regression + re-gate + continue.
11. **Q10 observability reconciliation:** integrate only genuinely new R9 evidence into existing observability/result surfaces; no second logging framework.
12. **Q11 factual status/release reconciliation:** update status/plan/roadmap/release packet only for earned evidence. Only green independently reviewed cross-process R9 plus real TCP promotion may create a specific `READY_LIVE` row.
13. **Q12 one changed-hypothesis self-owned VPS run:** only after Q11 creates a specific `READY_LIVE` question, execute exactly one minimal bounded self-owned client<->VPS run under standing authorization; record exact commit/binary/parameters, recovery/Session/switch evidence and cleanup; preserve negative evidence and do not unchanged-retry.

# VPS opportunity

**Not READY — implementation/evidence dependency.** Unlock chain: positive P2 + P3/P4 + clean exact-tree provenance -> R9-3..R9-12 -> Q10/Q11. Standing authorization already covers the eventual bounded self-owned TCP/UDP run after a specific `READY_LIVE` row exists.

# Non-blocking policy/authority gates

Keep separate from this mechanically determined R9 chain:

- `SessionRuntime.events` retention (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 capacity/adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability;
- final RC/freeze/release/production authority.

Do not invent policy values while working R9.
