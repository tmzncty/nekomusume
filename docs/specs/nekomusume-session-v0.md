# Nekomusume Session v0 — Normative Specification Placeholder

**Protocol research: 伊冯；Security research: 阿米娅；Transport research: 符玄；Plan & implementation: 庄方宜；Coordination/Review: 佩丽卡**
**检索日期：2026-08-26**
**Status: normative-source scaffold for specification work; not an implementation and not a wire-freeze by itself.**

## Authority and status

This file is the single project normative-source entry point for Nekomusume Session v0. It is intentionally a specification gate: no field, error code, cryptographic suite, or acknowledgement semantics are considered frozen until the relevant section is completed, reviewed, and linked from an ADR. Research notes under `docs/research/` are non-normative. `README.md` and `ROADMAP.md` are indexes. `docs/decisions.md` records decision status. `docs/design-handoff.md` is historical and non-normative.

A future amendment must preserve a versioned history and update golden vectors, fuzz expectations, and the review record. External standards are cited from RFC Editor; their mechanisms must not be mistaken for Nekomusume compliance.

## M0 normative gates

The following gates must be completed before this document can claim a frozen v0 wire specification:

1. **Identity and model:** define `Session`, `Carrier`, `Path`, `active_path_epoch`, and the authenticated binding between Session identity, epoch, and path challenge transcript.
2. **Wire:** define magic, version, record/frame types, canonical integer encoding, byte order, lengths, maximum sizes, and deterministic errors for truncation, unknown version/type, overflow, and invalid nesting.
3. **Evidence domains:** keep `packet_feedback`, `session_delivery`, and `path_validated` distinct. Define which evidence is authenticated and which state transition it permits.
4. **Delivery proof:** select whether the Session ACK proves `received`, `delivered`, or `effect`; packet feedback alone must never promote logical delivery.
5. **Replay:** define stream/offset/range identity, overlap conflict behavior, bounded deduplication, and `unsent`/`in_flight`/`uncertain`/`confirmed` transitions.
6. **Security:** after a reviewed ADR selects TLS 1.3 over custom records or a Noise pattern and a mature implementation, define nonce, key phase/update, identity, replay and resource limits. Until then, no cryptographic suite is frozen.
7. **Test contract:** require at least 20 deterministic golden vectors, deterministic state-machine tests, and a decode fuzz target whose oracle forbids panic, out-of-bounds access, unbounded allocation, and impossible-length acceptance. T-FR-01 remains a named failover/replay test plan, not an M0 live-network result.

## Explicit non-goals

This v0 gate does not specify QUIC, MPTCP, SCTP, DCCP, TCP packet acknowledgements, concurrent UDP/TCP striping, congestion-control parameters, 0-RTT, production proxy behavior, or a license choice. Those require separate decisions and evidence.

## Change control

A change is normative only when this file, its ADR, affected vectors/tests, and `docs/research/review-record-2026-08-26.md` are updated together and reviewed by Coordination/Review. Until all gates are complete, implementation work must describe this file as provisional rather than claim protocol conformance.
