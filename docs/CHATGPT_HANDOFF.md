# ChatGPT reviewer handoff — `940b17f` fixes client post-return ownership; H-R9-015 remains HIGH on server + dual settlement

## Current repository truth

- Latest developer source/test SHA reviewed: exact `940b17f5450314acae1b87dd63440b540185395c` (`fix(cli): post-migration reliable-UDP ownership for reserved record (H-R9-015)`).
- Independent bounded recheck: `docs/reviews/r9-2h-post-migration-reliable-ownership-recheck-940b17f-20260915.md` (reviewer commit `057673d`).
- Hosted GitHub checks on exact `940b17f`: `stable checks` success; `nightly decode fuzz smoke` success. Hosted checks are supplementary only; no developer-local exact-tree provenance for this source SHA is accepted yet.
- Open PRs: none. No new WAN/VPS experiment. `READY_LIVE: none` remains authoritative.
- Governance unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The coding agent is explicitly pre-authorized to close the remaining mechanically determined H-R9-015 pieces, finish R9-2H P1-P4 + exact-tree provenance, then continue through R9-3..R9-12 and Q10/Q11/Q12 without waiting for reviewer cadence.

## Accepted progress through `940b17f`

Prior R9-2H closures remain accepted:

- H-R9-011: out-of-order later Session DeliveryAck is buffered and cannot cumulatively confirm earlier unconfirmed bytes.
- H-R9-012/H-R9-013/H-R9-014: applied Session ACK evidence follows actual mutation order and carries exact `stream + offset`; buffered applied evidence carries `buffered=true`; logical `outstanding + pending_acks` must retire before Carrier-only settlement.
- one operation-wide malformed counter; one absolute application deadline; Carrier packet ACK remains Carrier-local; incomplete Recovery settlement is terminal and cannot feed downstream health/failover as success.

`940b17f` additionally closes the **client send-registration half** of H-R9-015:

- after successful migration-back, the reserved final UDP record reuses the existing client `ReliableUdpRuntime`;
- `can_send` is checked before ownership commit;
- `on_packet_sent` registers packet number / bytes / stable `FrameId(record.offset)` / retained plaintext before socket send;
- the post-return receive helper now receives `rt.as_mut()` instead of `None`.

Do not regress those facts.

# OPEN HIGH — H-R9-015 is only partially repaired

## H-R9-015A — post-return server still emits no Carrier packet ACK

The existing primary reliable-UDP server path already shows the correct committed layering: after authenticated Data successfully reaches `SessionRuntime::receive`, it derives the received packet number, calls `server_rt.on_packet_received(pn, true)`, emits the Session `DeliveryAck`, and separately obtains `server_rt.poll_outgoing_ack(0)` to encode/seal/send a Carrier `RecordType::Ack`.

The post-migration recovery owner still does only:

```text
authenticated post-return Data
 -> SessionRuntime::receive
 -> Session DeliveryAck
 -> migration_back_complete=true
```

It does **not** call `server_rt.on_packet_received` for that packet and does not emit the canonical Carrier ACK. Therefore the client Recovery packet registered by `940b17f` has no truthful server-side Carrier-ACK obligation.

### Smallest repair

Reuse the existing primary reliable-UDP server pattern in the post-return owner. Only after authentication + exact Data decode + successful `SessionRuntime::receive`:

1. derive the actual received packet number from the authenticated datagram framing;
2. call `server_rt.on_packet_received(pn, true)`;
3. emit the canonical authenticated Carrier packet ACK via `poll_outgoing_ack(0)` / existing ACK codec;
4. emit the Session `DeliveryAck` separately;
5. malformed/tampered/unadmitted/non-Data input creates neither Session delivery evidence nor Carrier ACK obligation.

Do not invent a second ACK format or packet tracker.

## H-R9-015B — post-return client is order-sensitive and does not require dual settlement

The client currently calls `recv_udp_delivery_ack(...)` once and treats the first typed result as follows:

- `Session` -> apply `SessionRuntime::delivery_ack` and continue;
- `Carrier` -> fail (`post-return path requires a Session DeliveryAck, got a Carrier ACK`).

Once H-R9-015A emits both independent acknowledgements, either authenticated datagram may arrive first. A valid Carrier ACK arriving before the Session DeliveryAck is not a protocol error.

Also, after accepting the Session DeliveryAck, the current path does not require `rt.in_flight() == 0` before success. A missing/suppressed Carrier ACK can therefore leave Recovery ownership live while the command continues.

### Smallest repair

Under the existing post-return absolute deadline and malformed budget, run one bounded classifier owner until both are true:

```text
exact reserved Session DeliveryAck applied once
AND
post-return ReliableUdpRuntime.in_flight() == 0
```

Carrier ACK and Session DeliveryAck may arrive in either order. Apply/reject Carrier feedback through the existing runtime with typed evidence; apply Session confirmation only to the exact reserved record. Neither evidence domain may retire the other. Timeout/malformed-bound/receive failure with either domain incomplete is typed terminal/nonzero and emits no success/settled claim.

No Session/Carrier/ACK/crypto/wire architecture change and no new TTL/LRU/capacity value is required.

# R9-2H evidence front — execute continuously after H-R9-015

## R9-2H-P1 — exact reversed-order built-process evidence

Strengthen the real built-binary reversed ACK test so structured evidence proves all of:

1. successful client exit;
2. exactly one `r9_udp_delivery_ack_buffered` with `stream=1`, `offset=16`, `watermark=0`;
3. exactly two applied `r9_udp_delivery_ack_validated` events;
4. direct applied = `stream=1`, `offset=0`, `buffered=false`;
5. buffered applied = `stream=1`, `offset=16`, `buffered=true`;
6. each exact identity occurs once;
7. no `r9_udp_delivery_ack_covered` shortcut for these two records;
8. logical outstanding/pending ownership is empty before Carrier settlement;
9. final relevant Recovery settlement reaches `remaining_in_flight=0`.

Use parsed/exact structured event assertions, not generic substring ordering that unrelated diagnostics can satisfy.

## R9-2H-P2 — exact reserved post-migration ownership

Strengthen `reliable_udp_migration_back_reserves_final_record` for the exact reserved fixture record (`stream=1`, current offset `32`) to prove:

- no pre-promotion Recovery ownership;
- no legacy pre-promotion `udp_uncertain_range_sent` ownership;
- `udp_migrated_back` precedes reliable tracking/send;
- exactly one post-promotion reliable ownership registration/send for that exact record;
- server records the exact authenticated post-return packet and emits one independent Carrier ACK obligation;
- exact Session DeliveryAck is independently applied once;
- arrival order of Carrier ACK vs Session DeliveryAck does not affect correctness;
- post-return Recovery reaches `in_flight=0` before command success.

## R9-2H-P3 — persistent malformed budget across valid Carrier feedback

Drive one reliable receive/settlement operation with authenticated sequence:

```text
malformed #1
malformed #2
canonical Carrier ACK
malformed #3
```

Malformed #3 must hit the existing `MAX_POST_HANDSHAKE_MALFORMED` ceiling. The valid Carrier ACK must not reset the operation-wide malformed counter. Termination must be typed, finite and non-spinning. A bounded test-only server seam may emit authenticated malformed plaintext; do not change the numeric limit.

## R9-2H-P4 — preserve incomplete settlement terminality

Keep `reliable_udp_incomplete_settlement_fails_not_settled`: remaining in-flight != 0 is nonzero/terminal, emits incomplete, never emits `r9_udp_in_flight_settled`, and never feeds downstream health/failover from a false premise. Extend the same truth boundary to the post-migration reserved-record owner.

## R9-2H-GATE — exact pushed-tree developer-local provenance

After H-R9-015A/B and P1-P4 all land on one reachable source/test SHA, use a safe clean worktree and persist:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, both exit codes, and clean initial/final tree. No decoder/parser/crypto-framing change => do not invent a new fuzz obligation; hosted fuzz remains supplementary.

R9-2H closes only when one reachable tree proves exact reversed logical confirmation, direct+buffered Session applied identity, bounded pending ownership, one operation-wide malformed budget/deadline, pre/post-migration reliable ownership, independent Session + Carrier settlement, exact reserved ownership and incomplete-settlement terminality.

**Then continue immediately to R9-3. Do not wait for reviewer cadence.**

# Continuous queue after R9-2H

Preserve this deep queue. Do not collapse it after one repair.

## R9-3 — process Data-loss recovery

Add bounded deterministic post-admission suppression of one/periodic reliable-owned Data packet. Prove congestion admission precedes suppression; PTO fires only after deadline; retransmission uses a fresh authenticated packet number/nonce with stable Session/frame identity; application delivery is exactly once; Carrier ACK and Session DeliveryAck settle independently; final Recovery drains to zero or produces explicit bounded incomplete/error.

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

**Not READY — implementation/evidence dependency.** Unlock chain: H-R9-015A/B -> R9-2H P1/P2/P3/P4 + local provenance -> R9-3..R9-12 -> Q10/Q11. Standing authorization already covers the eventual bounded self-owned TCP/UDP run once a specific `READY_LIVE` row exists; do not ask for generic WAN permission.

# Non-blocking policy/authority gates

Keep separate from this mechanically determined R9 chain:

- `SessionRuntime.events` retention (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 capacity/adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability;
- final RC/freeze/release/production authority.

Do not invent policy values while working R9.
