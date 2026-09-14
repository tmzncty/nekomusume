# R9-2H post-return dual-settlement recheck — exact `34ae142`

**Reviewed source/test revision:** `34ae1426ec6af8dfed161f1fcc7ba3e9e7797236` (`fix(cli): post-return dual settlement + server packet-ACK ownership (H-R9-015)`).

**Review type:** independent bounded source/evidence recheck of the H-R9-015 post-migration reserved-record path. This is not a local exact-tree gate, security audit, WAN result, release decision, or protocol freeze.

## Verdict

**H-R9-015A/B implementation shape: ACCEPT.** No new correctness defect was found in the bounded source scope below.

**R9-2H acceptance: NOT YET CLOSED.** Exact process evidence P1-P4 and developer-local clean exact-tree provenance remain required before R9-3. The coding agent is pre-authorized to complete those slices and continue immediately through the existing R9-3..R9-12 and Q10/Q11/Q12 queue without waiting for reviewer cadence.

## What `34ae142` closes in source

### Server post-return Carrier ACK ownership

After authenticated `open_unreliable`, exact `ProcessMessage::Data` decode, correct Session identity, and successful `SessionRuntime::receive`, reliable mode now:

1. derives the post-return packet identity from the received SecureSession record sequence;
2. calls the existing `server_rt.on_packet_received(pn, true)` owner;
3. emits the Session `DeliveryAck` separately;
4. polls the existing Recovery ACK ranges and sends the canonical authenticated `RecordType::Ack` separately.

Malformed, unauthenticated, wrong-peer, wrong-Session, non-Data, or Session-rejected input does not reach the new packet-received/Carrier-ACK block.

The packet-identity extraction is consistent with the existing crypto record contract: `SecureSession::seal` emits `sequence:u64be || Noise ciphertext`; `open` reads that same sequence, uses it as the Noise transport nonce, validates AEAD + record context, and only then advances replay state. The client post-return send registers that exact first-eight-byte sequence in `ReliableUdpRuntime`, so the server ACK names the same packet identity without introducing a second packet-number scheme.

### Client dual settlement

The post-return client now keeps one absolute two-second deadline and one malformed counter while waiting until both independent domains are complete:

```text
post_outstanding.is_empty()
AND
ReliableUdpRuntime.in_flight() == 0
```

A Carrier ACK may arrive before or after the Session DeliveryAck. Carrier feedback is applied through the existing runtime; Session confirmation is applied only to the exact reserved record. Timeout or receive/classification failure before both domains settle is terminal.

This preserves the committed layering: UDP packet ACK cannot prove Session delivery, and Session DeliveryAck cannot retire Recovery ownership.

## Hosted cross-evidence

GitHub-hosted checks on exact `34ae142` are both successful:

- `stable checks` — success;
- `nightly decode fuzz smoke` — success.

These are supplementary cross-evidence only. No developer-local clean exact-tree provenance for `34ae142` is accepted by this note.

## Remaining acceptance-critical evidence

### P1 — exact reversed logical-ACK evidence

The existing built-process regression still relies on broad substring/order assertions. Strengthen it to parse exact structured events and prove one buffered `stream=1, offset=16, watermark=0`, then direct applied `offset=0, buffered=false`, then buffered applied `offset=16, buffered=true`, each exactly once, with no covered shortcut and final Recovery settlement zero.

### P2 — exact reserved post-migration ownership and dual settlement

The current `reliable_udp_migration_back_reserves_final_record` originated as a negative reservation test: it permits a nonzero client exit and only rejects one legacy `udp_uncertain_range_sent` shape for offset 32. It does not prove the source behavior added by `940b17f`/`34ae142`.

Strengthen the real built-binary test so it requires successful client completion and exact structured evidence that:

1. reserved `stream=1, offset=32` has no pre-promotion Recovery/send ownership;
2. `udp_migrated_back` precedes exactly one `r9_udp_post_return_sent` for that exact record;
3. server accepts the authenticated post-return record and emits exactly one Session DeliveryAck plus one independent Carrier packet ACK obligation;
4. client applies the exact Session DeliveryAck once and Carrier ACK feedback through Recovery;
5. either ACK arrival order is accepted;
6. success occurs only after post-return `in_flight == 0`.

A dedicated final structured `r9_udp_post_return_settled` event carrying exact stream/offset and `remaining_in_flight=0` is acceptable if needed to make the evidence unambiguous; it is evidence only, not new protocol semantics.

### P3 — operation-wide malformed budget across valid Carrier feedback

Still require one authenticated receive operation with:

```text
malformed #1 -> malformed #2 -> canonical Carrier ACK -> malformed #3
```

The existing malformed ceiling must trigger on #3; the valid Carrier ACK must not reset the counter. No new numeric policy.

### P4 — incomplete settlement remains terminal, including post-return

Preserve the existing incomplete-settlement negative and extend the same truth boundary to post-return dual settlement: suppress/miss one evidence domain, require nonzero terminal outcome, no settled/success claim, and no downstream health/failover continuation based on incomplete Recovery ownership.

## Exact-tree gate before R9-3

After P1-P4 land together on one reachable pushed source/test SHA, record developer-local clean exact-tree provenance for:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, exit codes, and clean initial/final tree. No new decoder/parser/crypto-framing change was introduced by `34ae142`, so this review adds no new fuzz obligation beyond hosted supplementary evidence.

## Release/live boundary

No status/release promotion follows. Item 3 and item 4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`; `READY_LIVE: none` remains authoritative until the full local R9 chain is green and independently reviewed.
