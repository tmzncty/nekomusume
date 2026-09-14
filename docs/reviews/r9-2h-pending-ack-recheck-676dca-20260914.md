# R9-2H pending logical-ACK recheck — exact `676dca3`

## Scope and repository truth

Reviewed developer source commit: exact `676dca307f00ad36de69600c8a330ab58d60524f` (`fix(cli): R9-2 pending logical-ACK buffer — no cumulative over-confirmation (H-R9-011)`). The commit changes only `crates/neko-cli/src/main.rs`; no new built-binary regression or developer-local exact-tree provenance file landed with it.

GitHub-hosted checks on exact `676dca3` are both green: `stable checks` and `nightly decode fuzz smoke`. These are supplementary cross-evidence only and do not substitute for the required developer-local clean exact-tree `scripts/check.sh` + `git diff --check` provenance.

Open PRs: none. No WAN/VPS experiment occurred. `READY_LIVE: none` remains authoritative. Release item 3 and item 4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.

## Accepted source progress — H-R9-011 state mutation is repaired in the current two-record reliable receive path

The new `pending_acks: Vec<OutboundRecord>` prevents a later exact Session DeliveryAck from immediately advancing the cumulative Session watermark over an earlier unconfirmed range. A recognized ACK is applied only when `record.offset == delivery.confirmed_watermark(record.stream)`; a later offset is retained pending, and pending ACKs are applied only after the watermark becomes contiguous.

This preserves the existing Session core semantics rather than changing `SessionRuntime::delivery_ack`, and it does not alter wire or crypto framing. Pending ownership is naturally bounded by the already-bounded reliable-owned logical set: records move from the helper's `outstanding` set into `pending_acks`; the change does not introduce a new TTL/LRU/capacity policy.

The existing operation-wide malformed counter, one absolute `application_deadline`, typed Carrier `apply_ack` outcome, and terminal incomplete-settlement check remain present. H-R9-010B and M-R9-009 therefore stay closed on source inspection.

## OPEN HIGH — H-R9-012: structured logical-ACK evidence is emitted in the opposite order from actual Session application

The current source applies an in-order current ACK, increments `logical_confirmations`, then immediately drains now-contiguous buffered ACKs and emits `r9_udp_delivery_ack_validated` for each buffered record. Only after that drain loop does it emit the validated event for the current ACK.

For the required reversed-order case (`record 1 offset=16` arrives before `record 0 offset=0`):

1. record 1 is observed and correctly emitted as `r9_udp_delivery_ack_buffered` at watermark 0;
2. record 0 arrives and `SessionRuntime::delivery_ack(offset=0)` is applied first;
3. buffered record 1 is then applied second, and its `r9_udp_delivery_ack_validated` event is emitted immediately;
4. only after the pending drain does the code emit the validated event for record 0.

The state transition order is therefore correct, but the structured evidence states the reverse application order. The special first-confirmation `udp_delivery_ack_validated` event can also disappear in this reversed case because `logical_confirmations` has already become 2 before the current-record diagnostic is selected.

This is an evidence-integrity HIGH because R9-2H-P1 specifically requires offset-bearing process diagnostics to prove that record 0 exact confirmation applies before the buffered record 1 exact confirmation. Current output cannot truthfully prove that ordering even though the state mutation was repaired.

### Smallest repair

Do not change Session, Carrier, ACK, wire, crypto, or capacity semantics. After a current exact ACK successfully calls `delivery.delivery_ack`, emit its applied/validated diagnostic immediately, before draining `pending_acks`. Then drain and emit each buffered ACK's applied event in the actual order of successful Session mutation. Keep `observed/buffered` distinct from `applied/validated`. A final successful reliable receive should also assert/emit that no logical pending evidence remains before Carrier settlement is treated as complete.

## Remaining discriminating closure work

### P1 — real built-binary reversed logical ACK

The existing `88ffa5f` test commit did not add a built-binary `--reverse-ack-order` process regression. Add one against the repaired source. It must prove, from real server/client output:

- offset 16 ACK is observed first and buffered while watermark is 0;
- it does not advance Session confirmation;
- exact offset 0 confirmation applies exactly once and is logged as applied first;
- only then buffered exact offset 16 confirmation applies exactly once and is logged second;
- logical outstanding + pending state is empty only after both exact confirmations;
- Carrier packet ACK remains Carrier-local and cannot advance Session confirmation;
- successful Recovery settlement reaches zero in-flight;
- no duplicate/conflict application delivery is created.

A regression that checks only process success, only event presence, or the current inverted diagnostic order is insufficient.

### P2 — strengthen migration-back reservation ownership

Keep `reliable_udp_migration_back_reserves_final_record`, but make it prove the exact reserved record is not Recovery-tracked before post-promotion authorization, is not sent by the legacy pre-promotion uncertain path, and is tracked/sent only by the explicit later post-promotion owner. The current negative assertion against one exact `udp_uncertain_range_sent` log shape is only partial evidence.

### P3 — persistent malformed budget across Carrier feedback

Through the real built process path drive one reliable receive/settlement operation with:

`malformed #1 -> malformed #2 -> canonical Carrier ACK -> malformed #3`

Malformed #3 must hit the same existing `MAX_POST_HANDSHAKE_MALFORMED` ceiling. The valid Carrier ACK must not reset the counter. Termination must be typed and finite.

### P4 — preserve incomplete settlement terminality

Keep `reliable_udp_incomplete_settlement_fails_not_settled` green: nonzero remaining in-flight must emit the incomplete marker, return nonzero, never emit the settled marker, and never fall through into health/failover on a false premise.

### Final exact-tree gate

After H-R9-012 and P1-P4 land on one reachable source/test SHA, persist one developer-local clean exact-tree gate containing at least:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, exit codes, and clean initial/final tree. No additional fuzz obligation is created unless wire decoder/parser/crypto framing code changes.

## Queue decision

R9-2H is **not closed** at `676dca3`. R9-3 remains blocked only by the mechanically repairable H-R9-012/P1-P3/evidence-gate front; this is not a policy or WAN authorization blocker.

After R9-2H closes, continue immediately through R9-3 Data-loss/PTO recovery, R9-4 ACK-loss/reorder, R9-5 tamper/future-ACK negatives, R9-6 pacing/cwnd/plaintext ownership, R9-7 truthful process observability, R9-8 real authenticated warm TCP standby, R9-9 resolved-UDP health/hysteresis/promotion, R9-10 uncertain Session replay over TCP, R9-11 timeout/shutdown/cleanup, R9-12 coherent exact-tree independent review, then Q10 observability reconciliation, Q11 factual status/release reconciliation, and only then Q12 one changed-hypothesis bounded self-owned VPS run if Q11 creates a specific `READY_LIVE` row.

Non-blocking policy/authority gates remain separate: `SessionRuntime.events` retention, D019, RSEC-001 capacity suitability, signing/key custody/SBOM/publication trust, previous frozen-release interoperability, and final RC/freeze/release/production authority.
