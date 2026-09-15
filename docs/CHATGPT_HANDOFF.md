# ChatGPT reviewer handoff — P1 accepted; P2 is READY_LOCAL and implementation-stagnant

## Current repository truth

- Default branch before this refresh: exact `097201dc6af9ae05f6183accdd12cc659c45e363` (`docs(handoff): accept P1 and advance R9-2H to P2`).
- Latest developer source/test SHA remains exact `76c31284f10920728f6f5609cefa665cf1c3a68b` (`test(cli): P1 exact acceptance — client success + structured order proof`). No developer source/test commit has landed since the P1 acceptance handoff.
- Independent bounded P1 acceptance remains `docs/reviews/r9-2h-p1-accept-76c3128-20260915.md` (reviewer commit `908bc10`).
- Hosted GitHub checks on exact `76c3128`: `stable checks` success; `nightly decode fuzz smoke` success. Hosted checks remain supplementary only.
- Open PRs: none. No new WAN/VPS experiment. `READY_LIVE: none` remains authoritative.
- Governance unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

## Implementation-stagnation classification

P2 has remained dependency-ready across multiple reviewer intervals with a green exact source/test anchor and no CI, repository-integrity, authorization, environment, architecture or policy blocker. Per `AGENTS.md` §3.2, this is **implementation stagnation**, not a reason to wait for another reviewer.

The coding agent is explicitly pre-authorized to start P2 now, then execute P3, P4, persist the final R9-2H exact-tree local provenance, and continue immediately through R9-3..R9-12 and Q10/Q11/Q12 without waiting for reviewer cadence. Reviewer cadence is not a work-ticket boundary.

Do not spend another cycle re-reading this handoff and reporting no trigger. The current source already contains the H-R9-015 ownership path; P2 is an end-to-end evidence/test closure around that path, with only minimal test-only ordering/evidence seams if needed.

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

Current source already has the required ownership shape:

- after `migrate_back_to_udp` + `apply_migration_back`, client builds the reserved post-return Data, runs reliable-UDP `can_send`, calls `on_packet_sent` with stable `FrameId(post_record.offset)`, then sends the authenticated UDP datagram;
- server authenticates/decodes the post-return Data and successfully calls `SessionRuntime::receive` before creating either acknowledgement domain;
- in reliable mode the server then calls `server_rt.on_packet_received`, sends the independent Session DeliveryAck, and emits the canonical Carrier packet ACK from `poll_outgoing_ack`;
- client post-return receive waits until both logical outstanding is retired and `rt.in_flight()==0`, and accepts either Session or Carrier acknowledgement through the existing bounded demux helper.

Primary test owner: `crates/neko-cli/tests/probe.rs::reliable_udp_migration_back_reserves_final_record`.

The current test is **not acceptance evidence**. It deliberately ignores both process exit statuses and only proves that reserved offset 32 is absent from one legacy `udp_uncertain_range_sent` string. It does not prove the already-landed ownership/settlement path end-to-end.

### Required P2 closure

Strengthen that existing built-binary process test, adding only the minimum test-only reordering/evidence seam needed, so exact reserved fixture record `stream=1, offset=32` proves all of the following:

1. **Both processes succeed.** Assert client success and server success; a nonzero exit is no longer acceptable for this positive P2 case.
2. **No pre-promotion reserved ownership.** Offset 32 must not be Recovery-tracked/sent before successful migration-back, and must never use the legacy uncertain owner.
3. **Promotion precedes reliable send.** Structured `migrated_back_to_udp`/`udp_migrated_back` evidence must precede exactly one `r9_udp_post_return_sent` carrying offset 32.
4. **Server acceptance precedes both acknowledgement obligations.** The authenticated/correct-Session Data must be accepted by `SessionRuntime::receive` before the server emits either the Session DeliveryAck or the Carrier packet ACK. If current diagnostics cannot prove that ordering, add one evidence-only structured event immediately after successful receive, e.g. `r9_udp_post_return_received` with exact stream/offset. Do not create a new logging framework.
5. **Exact Session confirmation.** Client must show the post-return Session confirmation for stream 1 / offset 32 was actually applied. If needed add one evidence-only event immediately after successful `delivery.delivery_ack`, carrying exact stream/offset; do not infer this from a generic success line.
6. **Independent Carrier settlement.** Client must independently apply the post-return Carrier ACK and reach `rt.in_flight()==0` before success. Prefer one terminal structured event containing `remaining_in_flight=0` if existing output is ambiguous.
7. **Both arrival orders.** Exercise Session-ACK-first and Carrier-ACK-first. The production path must remain unchanged; use a bounded test-only server seam that swaps only acknowledgement send order (for example a test-only `--reverse-post-return-ack-order` flag) rather than sleeps/timing races.
8. **No domain collapse.** Session DeliveryAck must never substitute for Carrier ACK, and Carrier ACK must never mutate Session confirmation state.

Recommended test shape: one small helper in `probe.rs` runs the same three-record migration-back fixture twice, once in default order and once with the bounded reorder seam, then parses exact structured events and asserts the ordering/state predicates above. Avoid brittle whole-line substring assumptions when structured event identity is available.

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

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, both exit codes, and clean initial/final tree. No decoder/parser/crypto-framing change is required by the current evidence-only front; do not invent a new fuzz obligation.

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

**Not READY — implementation/evidence dependency.** Unlock chain: P2/P3/P4 + clean exact-tree provenance -> R9-3..R9-12 -> Q10/Q11. Standing authorization already covers the eventual bounded self-owned TCP/UDP run after a specific `READY_LIVE` row exists.

# Non-blocking policy/authority gates

Keep separate from this mechanically determined R9 chain:

- `SessionRuntime.events` retention (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 capacity/adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability;
- final RC/freeze/release/production authority.

Do not invent policy values while working R9.
