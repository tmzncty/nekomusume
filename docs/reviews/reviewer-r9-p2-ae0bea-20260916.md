# Reviewer checkpoint — R9-2 P2 exact evidence at `ae0bea8`

Exact reviewed developer-owned source/test anchor: `ae0bea87c45b8197cae6e4e6aa8268e171f54389` (`test(cli): P2 C1-C2 exact identity/cardinality — TCP replay seq2 offset32 + post-return send once after migrated`). Parent reviewer handoff: `60d64b7fe3db9b65813dbe463ab64cfcbcdba6c5`.

## Classification

This commit is evidence-oriented implementation + process-test work, not a Session/Carrier/ACK/wire semantic change:

- `crates/neko-cli/src/main.rs`: adds secret-free stream/offset fields at the already-existing client/server TCP DeliveryAck evidence points.
- `crates/neko-cli/tests/probe.rs`: tightens the existing count=4 migration-back P2 fixture for TCP replay identity/cardinality and post-return-send cardinality/order.

No decoder/parser/crypto framing source changed.

## Exact-tree cross-evidence

GitHub-hosted Rust CI on exact `ae0bea87c45b8197cae6e4e6aa8268e171f54389` is green:

- `stable checks`: SUCCESS; hosted job ran `bash scripts/check.sh`.
- `nightly decode fuzz smoke`: SUCCESS.

This is hosted cross-evidence only. It is **not** the required final developer-local clean exact-tree R9-2 provenance (`scripts/check.sh`, `git diff --check`, clean initial/final tree, exact OS/arch/Rust/UTC metadata).

Open PRs at review time: none.

## P2 progress accepted narrowly

### C1 — materially improved, not yet mechanically complete

The existing fixture now requires exactly one client `tcp_delivery_ack_validated` and exactly one server `tcp_delivery_ack_sent`; the client line is bound to stream 1 / offset 32, and the server line is bound to offset 32. It also excludes client TCP-ACK evidence for offsets 0, 16 and 48.

Remaining exact oracle work is small: bind both selected lines mechanically to `seq=2`, bind the server line to stream 1 as well, and keep one-only cardinality. No transport change is needed.

### C2 — materially improved, not yet mechanically complete

The fixture now requires exactly one `r9_udp_post_return_sent`, requires offset 48, and proves `udp_migrated_back < r9_udp_post_return_sent`. Earlier assertions already prove first-observed `challenge < validated < migrated`.

Remaining exact oracle work: make the selected challenge/validated/migrated milestones one-only rather than first-occurrence-only, then assert the full strict chain with the one post-return send. Keep the negative ownership boundary: offset 48 must not appear in uncertain/TCP/pre-promotion reliable ownership evidence.

### C3/C4 — still open

The current server post-return diagnostics still emit `udp_return_delivery_ack_sent` and `udp_return_packet_ack_sent` without enough exact identity to bind both one-only events mechanically to the post-return owner. The client `udp_return_delivery_ack_validated` likewise lacks stream/offset/len, while `r9_udp_return_packet_ack` records only `applied`.

The current test proves both ACK-domain event names occur before `r9_udp_post_return_settled`, but does not yet establish exact one-only domain identity/cardinality. This remains the next dependency-ready evidence slice.

## Bounded independent review — automatic-health TCP replay ownership

### Invariant challenged

When automatic health failover promotes TCP, the records replayed on TCP must be exactly the controller-owned uncertain logical identities and payloads; a count-only reconstruction must not silently select a different record.

### Inspected owners

- `crates/neko-carrier/src/lib.rs`: `DataId`, `FailoverController::track_uncertain`, `tcp_resend`, `confirm` and focused failover tests.
- `crates/neko-cli/src/main.rs`: Session record construction, `uncertain_start/uncertain_end`, controller tracking, automatic-health failure boundary and the TCP replay loop.
- Existing count=4 migration-back process fixture and exact-SHA hosted stable gate.

### Result: bounded no finding on the current reachable CLI semantics

`FailoverController::track_uncertain` stores an immutable snapshot `DataId -> Vec<u8>`, rejects a same-id/different-payload conflict, and `tcp_resend()` clones that exact ordered map. In the current failover CLI, the controller uncertain set is constructed once from exactly the contiguous immutable slice `records[uncertain_start..uncertain_end]`, with `DataId(record.offset)` and `record.data`. The same slice is later used for TCP replay; `tcp_resend().len()` therefore selects the same current identities, and each replayed record is immediately confirmed by the same `DataId(record.offset)`.

The automatic-health observation path does not partially retire that set before TCP promotion: permitted matching UDP delivery progress at the degradation boundary is treated as an unexpected terminal condition rather than silently confirming a subset. Thus the specific stale handoff concern — that current reachable automatic-health behavior can already contain a sparse/non-contiguous controller set while positional replay chooses a different record — is not reproduced at this anchor.

Do **not** manufacture a helper/schema/refactor solely to preserve `(DataId,payload)` when current semantics already prove equivalence. Re-open this finding if a future change permits partial controller confirmation/removal before replay, sparse uncertain membership, a different payload source, or multi-stream identity mapping where offset alone is no longer a sufficient caller-chosen `DataId`.

This no-finding review does not generalize to hypothetical future sparse sets; it closes only the current READY_LOCAL challenge.

## Next dependency-ready queue

1. Finish P2 C1/C2 exact cardinality/identity with the existing fixture.
2. Add only the minimum post-return diagnostic identity required for exact C3/C4 one-only assertions.
3. Challenge both post-return ACK arrival orders through the same bounded receive owner/deadline/budget.
4. Tighten the two existing P4 ACK-domain suppression negatives.
5. Run final developer-local R9-2 exact-tree provenance.
6. Continue immediately through R9-3..R9-12 and then the dedicated independent cross-process R9 review / factual release reconciliation.

## Release/live boundary

No new WAN/VPS question was created by this commit or review. `READY_LIVE: none` remains authoritative. Release item 3 and item 4 remain incomplete; no RC/freeze/release/production authority changes are justified.
