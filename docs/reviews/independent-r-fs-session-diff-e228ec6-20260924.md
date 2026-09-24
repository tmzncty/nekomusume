# R-FS/SESSION-DIFF — FairScheduler / multistream / SessionRuntime flow-accounting owner-diff review

**Exact source/control-flow anchor:** `e228ec61a25a54e77de9b5ba38a4fda4393c1191`.

**Review class:** bounded independent GitHub source/owner-diff review. This is not reviewer-local Rust execution, developer-local provenance for this anchor, hosted CI, WAN evidence, fuzz evidence, or performance evidence.

## Reuse baseline

The dependency-ready baseline is the completed I4-FS2 / multistream + flow-control review chain through `982ee6da427698ac73e7e199650873255ecef4bb`, including the H-I4-086/H-I4-087 repairs that separated Session delivery acknowledgement from drain and bounded aggregate queued-record ownership. The earlier dedicated FairScheduler challenge is also retained for the scheduler-only invariants.

## Owners / claims challenged

- `crates/neko-carrier/src/lib.rs::FairScheduler`: stream-count, per-stream byte, session byte, overflow and interactive-vs-bulk fairness accounting;
- `crates/neko-session/src/lib.rs`: SessionRuntime lifecycle/resource/window/DeliveryAck accounting and the Session-vs-carrier evidence boundary;
- `crates/neko-cli/tests/multistream.rs`: whether later process-harness hardening changed the semantic multistream/flow-control oracle rather than only child ownership/deadline mechanics;
- `docs/specs/nekomusume-session-v0.md`: Session confirmation remains logical transport-delivery evidence, not application side-effect or packet-feedback evidence.

## Owner-diff result

**No finding in this bounded lane.**

- `982ee6da..e228ec6` does **not** modify `crates/neko-session/src/lib.rs`. The SessionRuntime / delivery-accounting implementation reviewed by the FS2 closure therefore remains the exact semantic owner.
- The exact-current `FairScheduler` implementation was compared with the `982ee6da` tree and is unchanged: open-stream admission remains bounded by `max_streams`; enqueue performs checked per-stream and aggregate-session byte addition before mutation; empty data is rejected; dequeue subtracts exactly the selected frame from both accounting domains; round-robin cursor state is retained; and the existing `INTERACTIVE_BURST` gate still forces a queued bulk stream to become preferred after the committed interactive burst without inventing a new policy value.
- Later edits in `crates/neko-carrier/src/lib.rs` are in independently challenged Recovery/Reno/PTO, CarrierState, manager/health/migration and adjacent test-support owners, not the `FairScheduler` implementation.
- `crates/neko-cli/tests/multistream.rs` has changed since the FS2 closure, but the later changes are the separately reviewed process-ownership/deadline/readout hardening chain (H-I4-106/H-I4-107/H-I4-115 class). They do not change the committed Session/stream accounting contract or promote process exit into Session delivery evidence.
- The provisional Session v0 evidence boundary remains unchanged: Session confirmation and carrier packet/path observations remain separate domains.

## Evidence boundaries / exclusions

- This note does not claim the historical FS2 gate ran on `e228ec6`; it establishes semantic-owner reuse only.
- It does not choose new queue/window/capacity numbers and does not touch D019 or any release/security policy.
- No decoder/parser/crypto framing owner changed; fuzz is not requested.
- No new real-network question is created. **`READY_LIVE: none`.**

## Classification

`R-FS/SESSION-DIFF: CLOSED — bounded no-finding / prior dedicated evidence remains reusable for unchanged semantic owners.`

Continue to the current CLI/output/process + algorithmic-boundedness owner-diff lane; release items 3/4 remain incomplete and repository-wide queue exhaustion is not established.
