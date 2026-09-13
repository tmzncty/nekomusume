# Independent R9-2 repair recheck — exact `2dcb731`

**Reviewed source revision:** `2dcb7316849a0ead04dfd6d4b96cd53480e085d2`.

**Current descendant at review time:** `cc6fb959411f8f54ef4ad64bd84df649f874f430` adds developer-local provenance only; no source/test change after `2dcb731`.

**Scope:** the prior findings H-R9-001..003 / M-R9-004 from `independent-r9-cross-process-f849fe9-20260914.md`, current `crates/neko-cli/src/main.rs`, current R9 process regression, exact-tree provenance, and the interaction with the pre-existing migration-back reserved-record ownership rule.

This is a bounded correctness/evidence review, not a security audit, WAN result, RC decision, protocol freeze, or release approval.

## Evidence state

- `docs/local-gate-2dcb731-20260914.md` records a developer-local clean detached exact-tree gate for `2dcb731`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean initial/final tree, Linux x86_64 / rustc 1.98.0.
- Current descendant `cc6fb95` has GitHub-hosted `stable checks` and `nightly decode fuzz smoke` green. Hosted checks are separate cross-evidence.
- No WAN/VPS result is introduced by this sequence.

## Verdict

**R9-2 remains blocked.** The repair is partial and useful; do not revert it. H-R9-002 and H-R9-003 are closed by source changes, but H-R9-001 remains present. M-R9-004 is only partially addressed and still does not prove two independent Session confirmations. A new ownership defect appears when `--reliable-udp` is combined with the existing recovery/migration-back mode.

## H-R9-001 — OPEN: authenticated receive demux still swallows the second Session DeliveryAck

**Severity:** HIGH.

Current `recv_udp_delivery_ack` still takes one `expected: &OutboundRecord`. It returns only for a Session `DeliveryAck` matching that one record. With `rt` present, every other authenticated plaintext enters the Carrier-ACK branch; if it is not a canonical Carrier ACK, the code still `continue`s.

Therefore the second reliable record's Session `DeliveryAck` is never applied to `SessionRuntime`:

- if it arrives before record 1's Session ACK, the helper consumes and discards it;
- if it arrives after the helper returns, the later settlement loop only attempts Carrier-ACK decoding and does not apply Session DeliveryAck.

The caller still invokes `delivery.delivery_ack(...)` only for `udp_record` (record 1). Thus current source cannot establish independent logical confirmation for both reliable-owned records.

The same branch still silently discards `ReliableUdpRuntime::apply_ack` errors (`let _ = ...`) and authenticated non-ACK/control/malformed plaintext bypasses the existing `MAX_POST_HANDSHAKE_MALFORMED` bound whenever `rt` is present.

### Required repair

Replace the single-record R9 receive behavior with one bounded authenticated demux over a bounded outstanding logical-record set/map:

1. canonical Carrier `RecordType::Ack` -> decode -> `apply_ack`; rejected ACK is typed/observable and mutation-free;
2. Session `ProcessMessage::DeliveryAck` -> exact `(session, stream, offset, len)` match against outstanding logical records -> apply `SessionRuntime::delivery_ack` exactly once;
3. unexpected authenticated control/malformed plaintext -> bounded rejection/diagnostic and malformed budget accounting.

Carrier ACK and Session delivery confirmation remain separate evidence domains.

## H-R9-002 — CLOSED for the ordinary R9-2 no-recovery path

**Severity:** previous HIGH, closed with boundary below.

`2dcb731` moves the legacy direct uncertain send from index 1 to index 2 when `--reliable-udp` is enabled, so `records[1]` no longer has both a reliable owner and the legacy direct owner in the ordinary no-recovery R9-2 path.

This closure does **not** cover the new migration-back interaction described in H-R9-005.

## H-R9-003 — CLOSED

**Severity:** previous HIGH, closed.

`server_rt.on_packet_received` now occurs only after authenticated plaintext decodes as `ProcessMessage::Data`, the Session id matches `7001`, and `SessionRuntime::receive` succeeds. Authenticated non-Data/control/malformed input no longer enters the outgoing R9 packet-ACK tracker through this path.

## M-R9-004 — OPEN/PARTIAL: regression still does not prove two Session confirmations

**Severity:** MEDIUM evidence-integrity.

The new process assertions prove that offset 16 is sent through the reliable runtime and is not also emitted as the legacy `udp_uncertain_range_sent` offset-16 owner. They do **not** prove the full required multi-record contract.

Still missing durable assertions/evidence for:

- two distinct reliable-owned logical offsets, including the first record and offset 16;
- two independently validated/applied Session `DeliveryAck`s;
- both packet owners retired by Carrier ACKs;
- zero PTO/retransmit in the no-loss baseline;
- zero Session conflict and preferably zero duplicate;
- a negative that fails if record-2 Session ACK is swallowed.

Because H-R9-001 is still present, the stronger two-confirmation assertion should currently fail or expose the defect.

## H-R9-005 — NEW: `--reliable-udp` consumes the migration-back reserved final record early

**Severity:** HIGH for R9 progression into warm-fallback/migration-back ownership.

The pre-existing failover logic defines:

- `uncertain_end = records.len() - 1` when recovery/migration-back is enabled;
- the final record is deliberately left unassigned until the manager authorizes return to UDP.

`2dcb731` changes the reliable-mode uncertain path to `skip(2)` but leaves the count as `take(uncertain_end.saturating_sub(1))`, and unconditionally selects `uncertain_idx = 2` for the direct uncertain send.

When `--reliable-udp` and recovery/migration-back are both enabled, this can include and directly send the final reserved record before migration-back authorization. For a three-record run, `uncertain_end == 2`, yet index 2 is still selected and sent. That violates the existing reserved-record ownership boundary and would invalidate later R9-8/R9-10 evidence.

### Required repair

Derive explicit indices rather than reusing the old `- 1` count:

- `reliable_owned_end = min(2, records.len())` in reliable mode (otherwise legacy start 1);
- `uncertain_start = reliable_owned_end` in reliable mode, legacy 1 otherwise;
- `uncertain_end` remains the existing exclusive end that reserves the final record when recovery is enabled;
- track/send a legacy uncertain record only when `uncertain_start < uncertain_end`;
- iterator count must be `uncertain_end.saturating_sub(uncertain_start)`.

Add a focused combined-mode regression proving the final migration-back record is not sent/tracked before the post-promotion return-to-UDP milestone.

## R9 progression consequence

Do not start R9-3 Data-loss fault injection yet. First close H-R9-001, H-R9-005, and the remaining M-R9-004 evidence gap, then re-run a clean exact pushed-tree gate. After that, continue immediately through R9-3..R9-12 without waiting for reviewer cadence.

`READY_LIVE` remains none. Policy gates (`SessionRuntime.events`, D019, RSEC-001, signing/SBOM/release authority) remain separate and do not block these mechanically determined R9 repairs.
