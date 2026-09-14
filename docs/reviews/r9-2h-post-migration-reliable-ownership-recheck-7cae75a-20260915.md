# R9-2H bounded recheck — exact `7cae75a`

**Reviewed source/test revision:** `7cae75a5c72a049e34349f8735acd8de7266ed3f`

**Review type:** independent bounded correctness/evidence recheck of the current R9-2H reliable-UDP failover process path. This is not a release/security approval, WAN result, protocol freeze, or production claim.

## Repository truth

- `main` is exact `7cae75a` at review time.
- Open PRs: none.
- GitHub-hosted `stable checks`: success on exact `7cae75a`.
- GitHub-hosted `nightly decode fuzz smoke`: success on exact `7cae75a`.
- Hosted checks are supplementary evidence only; final R9-2H developer-local exact-tree provenance is still required after the remaining source/test closure.
- No new WAN/VPS experiment occurred; `READY_LIVE: none` remains controlling.

## Review of new developer sequence

### `88ffa5f` — tests/evidence support

Useful partial process coverage landed for reversed Session DeliveryAck order, migration-back reservation and incomplete settlement. The migration-back assertion is only a narrow negative string check and does not prove post-promotion reliable ownership.

### `676dca3` — implementation

**ACCEPT.** Pending logical ACK buffering prevents a later Session DeliveryAck from advancing the cumulative Session watermark across an earlier unconfirmed range. Pending ownership is bounded by the already-bounded outstanding record set; no new retention policy is introduced.

### `bf80428` / `442f60c` — implementation + test

**ACCEPT_WITH_EVIDENCE_FOLLOWUP.** Applied logical confirmation diagnostics now follow mutation order and carry exact stream/offset for the direct path. `442f60c` also adds a real built-binary reversed-order process path, but its assertions remain too permissive for final R9-2H closure.

### `7cae75a` — implementation

**ACCEPT for H-R9-014.** Buffered pending-ACK application now emits exact `stream + offset + buffered=true`, and logical outstanding/pending ownership is explicitly required to be empty before Carrier-only settlement.

## Closed findings

### H-R9-014 — CLOSED

Buffered applied Session DeliveryAck evidence now carries the same exact logical identity class as the direct path. The pre-settlement logical-owner invariant is explicit.

## OPEN HIGH — H-R9-015 post-migration reliable-owned Data bypasses `ReliableUdpRuntime`

The current `--reliable-udp + --migration-back` path reserves the final logical record correctly before migration-back, but after the manager authorizes return to UDP the client sends that record using only:

```text
ProcessMessage::Data
 -> SecureSession::seal_unreliable
 -> UdpSocket::send_to
```

The post-return path does **not** call the existing reliable-UDP runtime's congestion/recovery ownership (`can_send` / `on_packet_sent`), and its receive helper is invoked with `rt = None`. The server's post-return recovery owner likewise emits only the Session `DeliveryAck`; it does not register the received packet with `server_rt` and does not emit the canonical Carrier packet ACK for that post-return Data.

Therefore the final Data record after migration-back is no longer actually reliable-UDP-owned. This violates the current R9/P2 contract and makes the existing migration-back process test insufficient: it proves only that the record was not sent too early, not that it was later sent by the correct reliable owner.

This is a mechanically repairable correctness/ownership defect under current architecture. It does not require a new Session/Carrier/ACK/wire policy decision.

### Smallest repair contract

Preserve the existing Session/Carrier layering and current wire/crypto grammar.

After successful `migrate_back_to_udp` + `apply_migration_back`, when the reserved final record is sent on UDP under `--reliable-udp`:

1. reuse the existing `ReliableUdpRuntime` instance;
2. require congestion admission before send;
3. atomically register packet number / bytes / stable frame identity / bounded retransmit plaintext with `on_packet_sent` before socket send;
4. if admission/registration fails, do not send the datagram;
5. on the server, after authenticated Data reaches `SessionRuntime::receive`, register that packet with the server reliable-UDP receive tracker and emit the canonical authenticated Carrier packet ACK, separately from Session `DeliveryAck`;
6. on the client, run the post-return receive/settlement through the same bounded classifier principles: exact Session confirmation and Carrier ACK are independent, share a finite absolute deadline and malformed budget, and success requires both logical ownership retired and post-return recovery `in_flight == 0`;
7. keep packet feedback distinct from Session delivery evidence.

Do not add a new packet-ACK architecture, do not create a second Session ledger, and do not invent new capacity/TTL/LRU policy.

## OPEN acceptance-critical evidence gaps

### M-R9-010 — P1 reversed-order process assertions remain too weak

The current built-binary test uses broad `contains/find/rfind` checks. It does not prove exact event cardinality/identity and can be satisfied by unrelated `offset` fields elsewhere in the diagnostic stream.

Strengthen the real process regression so it proves, from the relevant structured event lines:

- client exit success;
- exactly one buffered event for `stream=1, offset=16, watermark=0`;
- exactly two applied `r9_udp_delivery_ack_validated` events;
- first applied = `stream=1, offset=0, buffered=false`;
- second applied = `stream=1, offset=16, buffered=true`;
- no `r9_udp_delivery_ack_covered` shortcut for those records;
- logical outstanding/pending ownership empty before settlement;
- final relevant recovery settlement reaches zero.

For symmetric evidence, add `stream` to `r9_udp_delivery_ack_buffered` and make direct applied evidence explicitly state `buffered=false`; these are diagnostic truthfulness changes only.

### M-R9-011 — P2 migration-back process proof is currently only a pre-promotion negative

The current test only asserts that a specific string for legacy `udp_uncertain_range_sent` at offset 32 is absent. It does not prove the reserved record is later recovery-tracked and sent by the post-promotion reliable owner.

After H-R9-015 is repaired, P2 must prove exact record identity (`stream=1, offset=32` for the current fixture):

- no pre-promotion Recovery ownership;
- no legacy uncertain-owner send;
- `udp_migrated_back` occurs before post-return reliable tracking/send;
- exactly one post-promotion reliable ownership registration/send for the reserved record;
- Session DeliveryAck is exact and independent from Carrier ACK;
- post-return reliable recovery settles to zero before success.

### M-R9-012 — P3 persistent malformed budget lacks the required real-process discriminator

Source structure now carries one `&mut malformed` counter across classifier returns, including valid Carrier feedback, but the required built-binary sequence is not yet present:

```text
malformed authenticated plaintext #1
malformed authenticated plaintext #2
canonical Carrier ACK
malformed authenticated plaintext #3
```

The third malformed item must hit the existing `MAX_POST_HANDSHAKE_MALFORMED` boundary despite the intervening valid Carrier ACK. Termination must be typed, finite and non-spinning. A bounded test-only server seam may generate authenticated malformed plaintext; do not change the numeric limit.

### P4 incomplete-settlement terminality — KEEP GREEN

The existing process regression must remain controlling: nonzero remaining in-flight is terminal, emits incomplete, never emits settled, and never feeds downstream health/failover as success.

## Final R9-2H gate

After H-R9-015 plus P1/P2/P3/P4 all exist on one reachable source/test SHA, the developer must run and persist clean exact-tree local provenance:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, both exit codes, and clean initial/final tree. No new decoder/parser/crypto-framing change means no additional fuzz obligation beyond the repository rules; hosted fuzz remains supplementary.

Only then may R9-2H close and R9-3 begin.

## Queue continuity

Do not stop after this repair. The existing R9-3 through R9-12 and Q10/Q11/Q12 queue remains pre-authorized and dependency-ordered. No VPS run is READY before the local R9 chain earns a new specific `READY_LIVE` row.

Governance remains unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`; D019 and other policy/authority gates remain separate and non-blocking for this mechanically determined R9 work.
