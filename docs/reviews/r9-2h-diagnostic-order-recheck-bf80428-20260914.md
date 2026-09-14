# R9-2H diagnostic-order recheck — exact `bf80428`

## Scope and repository truth

Reviewed developer source commit: exact `bf80428e78e99c6fccc523d68b59b0b396b462a3` (`fix(cli): emit ACK validated in mutation order — no reversed evidence (H-R9-012)`). Its parent is reviewer handoff exact `3e238b9097518fa018c02c2f839336f868363098`. The developer commit changes only `crates/neko-cli/src/main.rs`; no process-test file or developer-local exact-tree provenance file lands with it.

GitHub-hosted checks on exact `bf80428` are both green: `stable checks` and `nightly decode fuzz smoke`. They are supplementary cross-evidence only and do not substitute for the required developer-local clean exact-tree `scripts/check.sh` + `git diff --check` provenance. Open PRs remain none. No WAN/VPS experiment occurred. `READY_LIVE: none` remains authoritative.

Release/governance boundaries are unchanged: item 3 incomplete, item 4 incomplete, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`.

## Accepted source progress — H-R9-012 mutation/evidence ordering component is repaired

The current source now emits the current in-order Session DeliveryAck's validated diagnostic immediately after its successful `SessionRuntime::delivery_ack` mutation and before draining any buffered later ACK. Buffered ACKs are then applied and emitted one by one in actual mutation order. This closes the specific inversion found at exact `676dca3`: for the reversed two-record shape, record 0 mutates first and its current-record diagnostic is emitted before the buffered record 1 mutation/diagnostic.

The repair does not alter Session/Carrier layering, ACK architecture, wire grammar, crypto framing, capacity policy, the operation-wide malformed counter, the single absolute application deadline, or terminal incomplete Carrier settlement.

## OPEN HIGH — H-R9-013: first applied Session ACK is still not exactly attributable in process evidence

R9-2H-P1 requires **offset-bearing** built-process evidence proving that exact offset 0 applies first and buffered exact offset 16 applies second.

At current `bf80428`, the first successful logical confirmation still emits the legacy event:

```text
udp_delivery_ack_validated
```

with `ciphertext_bytes` only. It does **not** include `stream` or `offset`. Only subsequent reliable confirmations use `r9_udp_delivery_ack_validated` with an explicit `offset`.

Therefore the ordering bug in source has been repaired, but the process evidence still cannot directly state that the first successful Session mutation was exact offset 0. Inferring that identity from the server fault seam or from the remaining outstanding set is weaker than the explicit P1 contract and would make the closure depend on test knowledge rather than truthful client mutation evidence.

This is an evidence-integrity HIGH for R9-2H closure, not a Session semantic defect.

### Smallest repair contract

Do not remove or reinterpret the legacy event if existing consumers rely on it. In reliable mode, emit one explicit offset-bearing applied event for **every** successful Session DeliveryAck mutation, including the first one. A safe minimal shape is:

- retain `udp_delivery_ack_validated` for compatibility if needed;
- additionally emit `r9_udp_delivery_ack_validated` containing exact `stream` and `offset` immediately after every successful reliable Session mutation;
- for buffered-drain applications, keep `buffered=true` (or equivalent typed attribution) and emit only after that mutation succeeds;
- observed/buffered evidence must remain distinct from applied/validated evidence.

Before transitioning from logical confirmation to Carrier settlement, explicitly verify that both `outstanding` and `pending_acks` are empty; if not, fail typed rather than continuing. This adds no capacity policy because both owners derive from the already-bounded reliable logical set.

## Remaining discriminating R9-2H closure work

### P1 — real built-binary reversed logical ACK regression

No developer test commit after the prior reviewer recheck adds the required process regression; `bf80428` is source-only. Add a built-binary `failover-server` / `failover-client` test using the existing `--reverse-ack-order` seam. It must parse/assert real client diagnostics and prove:

1. exact offset 16 ACK is observed first;
2. offset 16 is buffered while Session watermark remains 0;
3. exact offset 0 applied/validated event is explicit and appears first among successful Session mutations;
4. buffered exact offset 16 applied/validated event appears second;
5. both exact confirmations occur once; logical outstanding + pending ownership is empty before success;
6. Carrier packet ACK remains Carrier-local and never substitutes for Session confirmation;
7. successful Recovery settlement reaches zero in-flight;
8. no duplicate/conflict application delivery is fabricated.

A test that asserts only process success, only event presence, or inferred offset identity is insufficient.

### P2 — strengthen migration-back reserved-record ownership

The existing `reliable_udp_migration_back_reserves_final_record` still proves only a narrow negative log shape for the reserved final offset. Strengthen it to prove the exact reserved final record is not Recovery-tracked and not sent by the legacy pre-promotion owner, then is tracked/sent only by the explicit post-promotion owner after return authorization. Use exact offset/record identity, not aggregate counts or one negative string.

### P3 — persistent malformed budget across valid Carrier feedback

Add a real built-process regression for one reliable receive/settlement operation:

```text
malformed #1 -> malformed #2 -> canonical Carrier ACK -> malformed #3
```

Malformed #3 must hit the existing `MAX_POST_HANDSHAKE_MALFORMED` ceiling. The valid Carrier ACK must not reset the operation-wide malformed count. The result must be typed, finite and non-spinning. Do not add or change a numeric policy.

### P4 — preserve incomplete settlement terminality

Keep `reliable_udp_incomplete_settlement_fails_not_settled` green and semantically unchanged: nonzero remaining in-flight is terminal/nonzero, emits incomplete, never emits settled, and never continues into downstream health/failover from a false premise.

### Final exact-tree developer-local gate

After H-R9-013 and P1-P4 are present on one reachable source/test SHA, run from a clean worktree and persist:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, both exit codes, and clean initial/final tree. No extra fuzz obligation is created unless wire decoder/parser/crypto framing code changes.

## Queue decision

R9-2H is **not closed** at exact `bf80428`. R9-3 remains blocked by H-R9-013/P1-P3/final local provenance, all mechanically repairable under current committed semantics. This is not a policy, WAN authorization, or environment blocker.

After R9-2H closes, continue without waiting through R9-3 Data-loss/PTO recovery, R9-4 ACK-loss/reorder, R9-5 tamper/future-ACK negatives, R9-6 pacing/cwnd/plaintext ownership, R9-7 truthful process observability, R9-8 real authenticated warm TCP standby, R9-9 resolved-UDP health/hysteresis/promotion, R9-10 uncertain Session replay over TCP, R9-11 timeout/shutdown/cleanup, R9-12 coherent exact-tree independent review, Q10 observability reconciliation, Q11 factual status/release reconciliation, and only then Q12 one changed-hypothesis bounded self-owned VPS run if Q11 creates a specific `READY_LIVE` row.

Non-blocking policy/authority gates remain separate: `SessionRuntime.events` retention, D019, RSEC-001 capacity/adversarial-load suitability, signing/key custody/SBOM/publication trust, previous frozen-release interoperability, and final RC/freeze/release/production authority.
