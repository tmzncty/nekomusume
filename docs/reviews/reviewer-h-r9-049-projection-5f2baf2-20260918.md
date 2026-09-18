# Independent bounded review — H-R9-049 projection closure

Date: 2026-09-18

## Exact reviewed anchors

- Developer source/test anchor: `5f2baf2a29962521e4060697a063138dcaa7b9cb` (`fix(cli): H-R9-049 projection discriminator — carrier_retired_fields helper emits every retired packet`).
- Lower-layer typed-result discriminator: `a92d07630835b63504278e4653aaa9590894598b` (`test(reliable): H-R9-049 multi-retirement discriminator — one ACK range retires all outstanding`).
- Handoff observed after the repair: `458fc6b62b5ed73f20f9456eb1eee116998b169c`.

## Claim challenged

A single canonical Carrier ACK can retire more than one outstanding Recovery packet copy. The process evidence projection must therefore emit one positive `r9_udp_return_packet_ack` identity for every packet in the typed `acked_packets` result. It must not collapse a multi-retirement result to a representative `.last()` packet.

## Owners inspected

- `crates/neko-reliable/src/lib.rs`
  - `Recovery::on_ack`
  - `RecoveryResult::acked_packets`
  - `one_ack_range_retires_multiple_outstanding_packets`
- `crates/neko-cli/src/main.rs`
  - `carrier_retired_fields`
  - the post-return `UdpAcknowledgement::Carrier` continuation in `failover_client`
  - `carrier_retired_fields_projects_every_retired_packet`
- `docs/CHATGPT_HANDOFF.md`
- applicable evidence-domain boundaries in `docs/specs/nekomusume-session-v0.md`, `docs/carrier-architecture.md`, and `docs/decisions.md`.

## Challenge result

No new defect found in this bounded seam.

The lower layer has a deterministic discriminator in which one ACK range retires three outstanding packet identities and returns the complete ordered `acked_packets == [1, 2, 3]` set. The CLI projection now routes the exact typed `acked_packets` slice through `carrier_retired_fields`, which maps every element to its own positive `applied=true`, `packet_number=<pn>`, `retired=true` field. The runtime iterates every returned field and emits one `r9_udp_return_packet_ack` diagnostic per entry. The helper regression independently exercises a three-element set and a singleton, so reverting the projection to `.last()` or another one-element collapse would make the helper discriminator fail.

This composition is sufficient for the H-R9-049 property being challenged: complete typed retirement set at the Recovery owner, plus a runtime-used projection whose unit oracle preserves one output per typed identity. It does not rely on UDP log ordering and does not merge Carrier packet retirement with Session DeliveryAck.

## Validation/evidence boundary

- Developer-local exact-tree provenance recorded in the handoff for exact `5f2baf2`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean pushed tree, Linux x86_64, rustc 1.98.0.
- GitHub-hosted Rust CI run `35301009924` for exact `5f2baf2` completed successfully. Its `stable checks` job ran `bash scripts/check.sh`; its nightly job ran the pinned decode fuzz build and 30-second smoke. Hosted CI is cross-evidence only.
- The reviewer did not execute a separate local checkout/gate in this bounded review and does not relabel hosted CI as reviewer-local or developer-local evidence.

## Explicit exclusions

This review does **not** close:

- H-R9-041 exact encoded-wire retransmit-admission refusal regression;
- H-R9-042 deterministic just-before-PTO-deadline negative;
- downstream R9-4 through R9-12 integration lanes;
- release item 3 or release item 4 as a whole;
- any live WAN, performance, production, RC, freeze, release, signing, SBOM, key-custody, D019, or capacity-policy question.

No wire decoder/parser/crypto framing code changed in `5f2baf2`; no new fuzz requirement is created by this review itself.

## Disposition

**H-R9-049: independently bounded no-finding / closed at developer anchor `5f2baf2`.**

The dependency-ready queue remains non-empty. Continue immediately with H-R9-041, then H-R9-042, followed by the preserved R9-4…R9-12, dedicated independent R9 review, and Q10/Q11/Q12 factual reconciliation lanes. `READY_LIVE` remains `none` unless later code/instrumentation/hypothesis changes create a concrete unresolved real-network question.
