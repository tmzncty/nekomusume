# ChatGPT reviewer handoff — `7cae75a` closes buffered ACK identity; post-migration reliable ownership is HIGH

## Current repository truth

- Latest developer source/test SHA reviewed: exact `7cae75a5c72a049e34349f8735acd8de7266ed3f` (`fix(cli): buffered applied ACK carries stream identity + pre-settlement invariant (H-R9-014)`).
- Reviewer bounded recheck: `docs/reviews/r9-2h-post-migration-reliable-ownership-recheck-7cae75a-20260915.md` (reviewer commit `922c001`).
- Hosted checks on exact `7cae75a`: `stable checks` success; `nightly decode fuzz smoke` success. Hosted checks are supplementary only, not developer-local exact-tree provenance.
- Open PRs: none. No new WAN/VPS experiment occurred. `READY_LIVE: none` remains authoritative.
- Governance unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The coding agent is explicitly pre-authorized to repair the HIGH below, finish R9-2H process evidence/provenance, and then continue through R9-3..R9-12 and Q10/Q11/Q12 without waiting for reviewer cadence.

## Accepted progress through `7cae75a`

The developer sequence since the earlier R9-2H checkpoint is meaningful and accepted within its bounded scope:

- `88ffa5f`: useful tests/evidence support for reversed logical ACK order, migration-back reservation and incomplete settlement, but not sufficient for closure by itself.
- `676dca3`: H-R9-011 repaired — a later Session DeliveryAck is buffered and cannot cumulatively confirm an earlier unconfirmed range.
- `bf80428`: H-R9-012 repaired — applied evidence follows actual Session mutation order.
- `442f60c`: direct applied Session ACK evidence carries exact `stream + offset`, and a built-binary reversed-order path exists.
- `7cae75a`: **H-R9-014 CLOSED** — buffered applied ACK evidence now also carries exact `stream + offset + buffered=true`; logical `outstanding + pending_acks` must be empty before Carrier-only settlement.
- Prior R9-2H closures remain controlling: one operation-wide malformed count; one absolute application deadline; Carrier packet ACK remains Carrier-local; incomplete Carrier settlement is terminal/nonzero and cannot feed downstream health/failover as success.

# OPEN HIGH — H-R9-015 post-migration reliable-owned Data bypasses ReliableUdpRuntime

The current `--reliable-udp + --migration-back` path reserves the final logical record before migration-back, but after `CarrierManager::migrate_back_to_udp` and `FailoverController::apply_migration_back` authorize return to UDP, the client sends that final record with only:

```text
ProcessMessage::Data
 -> SecureSession::seal_unreliable
 -> UdpSocket::send_to
```

It does **not** pass through the existing reliable-UDP congestion/recovery owner (`can_send` / `on_packet_sent`), and the post-return receive helper is invoked with `rt=None`.

On the server, the post-return recovery owner authenticates the Data and feeds `SessionRuntime::receive`, then emits only the Session `DeliveryAck`; it does not register the packet with `server_rt` and does not emit the canonical Carrier packet ACK for that post-return Data.

Therefore the reserved final Data record is not actually reliable-UDP-owned after migration-back. Existing P2 only proves that it was not sent too early; it does not prove correct later ownership. This is a correctness/ownership HIGH under current committed architecture, not a policy question.

## H-R9-015 smallest repair contract

Preserve Session-above-Carrier layering and the current wire/crypto grammar.

For the post-migration final record when `--reliable-udp` is active:

1. reuse the existing client `ReliableUdpRuntime`;
2. require `can_send` before committing the send;
3. register packet number / bytes / stable `FrameId(record.offset)` / bounded retransmit plaintext with `on_packet_sent` **before** socket send;
4. if congestion admission or reliable ownership registration fails, do not send;
5. on the server, only after authenticated Data reaches `SessionRuntime::receive`, register that exact received packet with the server reliable-UDP tracker and emit the canonical authenticated Carrier packet ACK, separately from the Session `DeliveryAck`;
6. on the client, post-return receive/settlement must preserve the same bounded classifier principles: Session DeliveryAck and Carrier packet ACK are independent, one finite absolute deadline and malformed budget cover the operation, and success requires both logical ownership retired and post-return Recovery `in_flight == 0`;
7. packet ACK must never stand in for Session delivery evidence.

Do not introduce a new ACK architecture, a second Session ledger, TTL/LRU/capacity values, or migration policy.

# R9-2H evidence front — execute continuously after H-R9-015

## R9-2H-P1 — make reversed-order built-process evidence exact

The current process test is real but its assertions are too broad: generic `contains/find/rfind` on `offset` can be satisfied by unrelated diagnostic fields.

Strengthen the relevant structured event evidence so the real client process proves:

1. successful client exit;
2. exactly one `r9_udp_delivery_ack_buffered` with `stream=1`, `offset=16`, `watermark=0`;
3. exactly two `r9_udp_delivery_ack_validated` applied events;
4. applied event 1 = `stream=1`, `offset=0`, `buffered=false`;
5. applied event 2 = `stream=1`, `offset=16`, `buffered=true`;
6. each exact applied identity occurs once;
7. no `r9_udp_delivery_ack_covered` shortcut for these records;
8. logical outstanding/pending ownership is empty before Carrier settlement;
9. final relevant Recovery settlement reaches `remaining_in_flight=0`.

For symmetric evidence, add `stream` to `r9_udp_delivery_ack_buffered` and explicitly emit `buffered=false` on the direct applied event. These are evidence-truthfulness changes only.

## R9-2H-P2 — prove exact reserved post-migration ownership

After H-R9-015 repair, strengthen `reliable_udp_migration_back_reserves_final_record` so the exact reserved final record (current fixture: `stream=1`, `offset=32`) proves all of:

- no pre-promotion Recovery ownership;
- no legacy pre-promotion `udp_uncertain_range_sent` ownership;
- `udp_migrated_back` precedes its reliable tracking/send;
- exactly one post-promotion reliable ownership registration/send for that exact record;
- Session DeliveryAck is exact and independent from Carrier packet ACK;
- post-return Recovery settles to zero before success.

Use exact record identity and ordered relevant diagnostics, not aggregate counts or a single negative substring.

## R9-2H-P3 — persistent malformed budget across valid Carrier feedback

Source structure now carries one `&mut malformed` counter across classifier returns, but the required real-process discriminator is still absent.

Drive one reliable receive/settlement operation with authenticated sequence:

```text
malformed #1
malformed #2
canonical Carrier ACK
malformed #3
```

Malformed #3 must hit the existing `MAX_POST_HANDSHAKE_MALFORMED` ceiling. The valid Carrier ACK must not reset the operation-wide malformed count. Termination must be typed, finite and non-spinning. A bounded test-only server seam may emit authenticated malformed plaintext; do not change the numeric limit.

## R9-2H-P4 — keep incomplete settlement terminal

Preserve `reliable_udp_incomplete_settlement_fails_not_settled`: remaining in-flight != 0 is nonzero/terminal, emits incomplete, never emits `r9_udp_in_flight_settled`, and never feeds downstream health/failover from a false premise.

## R9-2H-GATE — exact pushed-tree developer-local provenance

After H-R9-015 and P1-P4 all land on one reachable source/test SHA, run in a safe clean worktree and persist:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, both exit codes, and clean initial/final tree. No decoder/parser/crypto-framing change => do not invent a new fuzz obligation; hosted fuzz remains supplementary.

R9-2H closes only when one reachable tree proves exact reversed logical confirmation, exact direct+buffered Session applied identity, bounded pending ownership, one operation-wide malformed budget, one absolute deadline, pre- and post-migration reliable ownership, independent Session + Carrier settlement, migration-back reserved ownership and incomplete-settlement terminality.

**Then continue immediately to R9-3. Do not wait for reviewer cadence.**

# Continuous queue after R9-2H

Preserve this deep queue; do not collapse it after one repair.

## R9-3 — process Data-loss recovery

Add bounded deterministic post-admission suppression of one/periodic reliable-owned Data packet. Prove congestion admission precedes suppression; PTO fires only after deadline; retransmission uses a fresh authenticated packet number/nonce with stable Session/frame identity; application delivery is exactly once; Carrier ACK and Session DeliveryAck settle independently; final Recovery drains to zero or produces an explicit bounded incomplete/error.

## R9-4 — ACK-loss + reorder / delayed original

Cover Carrier packet ACK emitted then suppressed, retransmitted replacement before delayed original, and delayed original after replacement. Require one logical Session delivery, Session-owned duplicate suppression, no conflict, fresh packet numbers/nonces, truthful ACK-loss/PTO counters and final settlement.

## R9-5 — tamper / future ACK / malformed-feedback negatives

Prove tampered Data creates no Carrier ACK obligation or Session receive; tampered ACK creates no recovery mutation; future/never-sent ACK is typed rejected atomically; stale/duplicate ACK fabricates no RTT/loss/Session evidence; malformed feedback is finite and panic-free.

## R9-6 — pacing / cwnd / plaintext-owner atomicity

Prove initial and retransmit sends consult congestion admission; refusal consumes no Session byte space and commits no packet/plaintext/recovery owner; retransmit plaintext has one bounded owner; pacing deadlines are finite; teardown releases ownership.

## R9-7 — truthful process observability/result contract

Keep Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, resolved recovery, Session delivery, malformed-budget, cleanup and final-outcome evidence domains distinct. Counters/events increment only after the action they claim succeeds.

## R9-8 — actual authenticated warm TCP standby

Reuse existing TCP connect/accept, canonical negotiation, fresh Noise trust/authz, resume/readiness tuple+generation+delivery-epoch binding and resource admission. Warm/standby carries no application Data before atomic promotion.

## R9-9 — resolved UDP health -> hysteresis -> real TCP promotion

Fresh resolved UDP outcomes drive Carrier health. Recoverable loss stays on UDP; PTO-only observation cannot erase later resolved loss; distinct bad resolved intervals cross committed hysteresis and promote only to an actually ready TCP standby. Invalid/unready standby yields typed failure.

## R9-10 — uncertain Session replay across UDP -> TCP

At promotion replay at least one genuinely uncertain logical Session range over promoted TCP. Draining UDP accepts no new application Data. Receiver deduplicates by Session/stream/offset and application bytes remain exactly once. Do not add TCP packet ACK.

## R9-11 — timeout / shutdown / cleanup matrix

Cover setup/application/PTO/single-owner receive deadline, controlled stop, failed standby promotion, malformed/tampered input, listener/socket/process cleanup and same-address rebind where supported.

## R9-12 — coherent exact-tree gate + independent bounded review

Independently challenge Session-above-Carrier layering, exact Session ACK evidence, authenticated packet identity/Carrier ACK, bounded plaintext/pending-ACK ownership, single receive-owner classification + operation-wide malformed/deadline ownership, PTO/pacing/cwnd, fresh health/hysteresis, real warm TCP readiness/promotion, uncertain Session replay/dedup, result truthfulness and cleanup/no-public-exposure. BLOCKER/HIGH -> smallest repair + regression + re-gate + continue; LOW/NOTE does not halt progression.

## Q10 — observability reconciliation

Integrate only genuinely new R9 evidence into existing `neko-observe`/result surfaces. Do not create a second logging framework.

## Q11 — factual status/release reconciliation

Reconcile `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md` and the release packet only for evidence actually earned by the final R9 tree. Only a green independently reviewed cross-process R9 plus real TCP promotion may create a new specific `READY_LIVE` row.

## Q12 — one changed-hypothesis self-owned VPS run

Only after Q11 creates a specific `READY_LIVE` question, execute exactly one minimal bounded self-owned client<->VPS run under standing authorization. Record exact commit/binary/parameters, authenticated recovery/Session/switch evidence and cleanup. Preserve negative evidence; no unchanged same-class retry.

# VPS opportunity

**Not READY — implementation/evidence dependency.** Current unlock chain: H-R9-015 -> R9-2H P1/P2/P3/P4 + local provenance -> R9-3..R9-12 -> Q10/Q11. Standing authorization already covers the eventual bounded self-owned TCP/UDP run once a specific `READY_LIVE` row exists; do not ask for generic WAN permission.

# Non-blocking policy/authority gates

Keep separate from this mechanically determined R9 chain:

- `SessionRuntime.events` retention (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 capacity/adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability;
- final RC/freeze/release/production authority.

Do not invent policy values while working R9.
