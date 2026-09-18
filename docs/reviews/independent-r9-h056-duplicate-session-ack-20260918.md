# Independent R9 review — H-R9-056 duplicate Session DeliveryAck misclassified as malformed

**Classification:** HIGH — local correctness/evidence defect in the dependency-ready R9-4 ACK-loss/retransmission path.

**Exact inspected repository anchor:** `431e79724effe7fe3f824532e00bedda5b6bcca5`

**Scope:** bounded source-level challenge of the current cross-process post-return reliable-UDP owners for R9-4: the server post-return receive/ACK continuation, `SessionRuntime::receive`, the client `recv_udp_delivery_ack` demux, `SessionRuntime::delivery_ack`, and the current packet-ACK tracker semantics. No wire/parser/crypto-framing change is proposed by this note.

## Finding

The exact-current server and client disagree about a legitimate semantic duplicate Session `DeliveryAck` created by Carrier-ACK loss followed by a retransmission.

The deterministic path is:

1. The original post-return reliable UDP `Data` reaches the server. The server calls `SessionRuntime::receive`, records the packet in the reliable-UDP ACK tracker, constructs the exact Session `DeliveryAck`, and polls the canonical Carrier ACK obligation.
2. R9-4 withholds that first legitimate Carrier ACK long enough to force PTO, while the Session `DeliveryAck` is still delivered. The client therefore removes the matching logical record from its bounded `outstanding` set and applies the one real logical confirmation; Recovery ownership for the packet remains outstanding.
3. PTO retransmits the same stable logical record under a fresh packet number / authenticated record sequence. On the server, `SessionRuntime::receive` deliberately accepts the byte-identical `(stream, offset, data)` duplicate as `DuplicateDedup` and returns `Ok(())` without enqueueing a second application delivery. The surrounding server owner nevertheless correctly constructs and sends another freshly authenticated Session `DeliveryAck` for that duplicate arrival.
4. On the client, `recv_udp_delivery_ack` recognizes a Session `DeliveryAck` only when it still matches an entry in `outstanding`. After step 2 that set no longer contains the already-confirmed record. The exact duplicate ACK from step 3 is therefore classified as `unexpected_logical_ack`, increments the operation-wide `malformed` budget, and repeated legitimate retransmission feedback can terminate with `UDP delivery acknowledgement malformed bound exceeded`.

This is not the crypto replay case: a replay of the same authenticated envelope still must fail. The problematic ACK is freshly sealed by the real server in response to a newly received retransmission of already-deduplicated logical bytes.

It is also inconsistent with the current Session owner. `SessionRuntime::delivery_ack` treats `end == confirmed_watermark` as a zero-delta idempotent acknowledgement: it performs no second window release / `AckReleased` / `Resumed` transition. The cross-process demux currently prevents that committed Session semantics from being represented and instead spends the malformed budget.

The current `reliable_demux_bounds_an_authenticated_unexpected_logical_ack` regression does not justify this behavior for the exact duplicate case: its logical ACK is for an unrelated offset (`4096`) and its repeated ciphertexts additionally exercise crypto replay rejection. It does not test a freshly sealed exact ACK for the just-confirmed bounded record.

## Why this is HIGH

R9-4 is specifically the legitimate Carrier-ACK-loss + PTO/retransmission challenge. The existing server deduplicates retransmitted application data correctly, but the client can label the corresponding legitimate semantic Session ACK as malformed. That makes successful recovery consume an adversarial/malformed-resource budget and allows ordinary repeated loss/reordering to turn authenticated, valid duplicate feedback into a terminal failure. It also makes diagnostics claim malformed input where the current Session state says the ACK is an idempotent zero-delta duplicate.

## Required repair / discriminator

Keep Session DeliveryAck and Carrier ACK as separate evidence domains and keep crypto envelope replay rejection unchanged.

Use the smallest bounded current-operation classification that can distinguish:

- an exact freshly authenticated Session ACK for the already-confirmed current logical range: accepted-empty / stale logical feedback, zero second confirmation transition, zero malformed-budget charge; from
- a wrong session/stream/range/length or otherwise unadmitted logical ACK: existing bounded negative / malformed behavior.

Prefer reusing the real `SessionRuntime` confirmation state or an equivalent bounded exact current-record context. Do **not** add unbounded ACK history, TTL/LRU/capacity policy, or a parallel delivery architecture.

Add a deterministic cross-process R9-4 regression using the real post-return owners:

1. deliver the original post-return Data;
2. withhold exactly its first legitimate Carrier ACK while still sending the Session DeliveryAck;
3. force a real PTO + fresh-PN retransmission of the same logical bytes;
4. let the server `DuplicateDedup` that retransmission and emit a freshly authenticated duplicate Session DeliveryAck;
5. release the delayed original / retransmission Carrier feedback in a deterministic reorder;
6. prove exactly one logical Session confirmation transition and exactly-once application delivery;
7. prove the exact duplicate Session ACK is classification-only and does not increment the malformed budget;
8. bind each positive Carrier retirement to a packet actually retired by Recovery, allow only semantically valid accepted-empty/stale Carrier feedback, and require no retransmit after lifecycle resolution;
9. finish with zero Recovery in-flight ownership and exactly-once release of retained retransmit plaintext.

The regression must fail if the current `unexpected_logical_ack -> malformed += 1` behavior is restored for the exact already-confirmed range.

## Evidence boundary

This review is source-derived against the reachable pushed anchor above. Reviewer-local execution is **not claimed** in this note. Existing developer-local and GitHub-hosted gates for earlier exact trees remain separate evidence and do not test this R9-4 path. No live/WAN question is created by this finding; `READY_LIVE: none` remains appropriate until local correctness closes.
