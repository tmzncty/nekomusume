# Cross-Research Review Record — 2026-08-26

**Protocol research: 伊冯；Security research: 阿米娅；Transport research: 符玄；Plan & implementation: 庄方宜；Coordination/Review: 佩丽卡**
**检索日期：2026-08-26**

## Inputs and questions

The review cross-examined the Protocol, Security and Transport research against README, ROADMAP, SECURITY.md, `docs/carrier-architecture.md`, `docs/decisions.md` and the historical handoff at HEAD `9b0063f`. Questions were: what survives a Carrier change; what does each ACK prove; whether path validation is independent; whether crypto/runtime/no_std are decided; and which claims can be tested at M0.

## Findings and resolutions

| Issue | Resolution | Status |
|---|---|---|
| Session ACK vs UDP/TCP ACK | Keep `session_delivery` separate from `packet_feedback`; use explicit proof object and offset/range identity. | M0 gate |
| ACK vs path validation | ACK cannot prove reverse reachability; require authenticated challenge and `path_validated`. RFC 9000 §8/§8.1 and §8.2/§8.2.1–§8.2.4 are the accurate references. | Accepted principle |
| Uncertain replay | Replay authenticated Session/stream/offset ranges; bounded receiver dedup; conflicting overlap fails closed. | M0 gate |
| TLS vs Noise | Both remain candidates; no library or handshake is selected before security ADR/review. | Open |
| no_std/runtime | No embedded requirement; defer no_std. Keep core free of socket/runtime; choose async runtime only at carrier implementation. | Proposed |
| QUIC/MPTCP/SCTP analogy | Use as evidence and vocabulary only; do not claim compliance or copy packet/token/chunk formats. | Accepted boundary |
| Fuzz and vectors | >=20 deterministic vectors and bounded decode fuzz; no live-network promise in M0. | Accepted acceptance |
| License | Administrator chose `MIT OR Apache-2.0`; root license files and workspace SPDX metadata are now present. | Resolved |

## Conflicts retained

The sources differ in packet ACK units, stream semantics, migration mechanisms and cryptographic integration. Those differences are not contradictions to erase: they are reasons not to define a universal Carrier ACK or import QUIC/MPTCP wire formats. Congestion-control parameters, path scoring/hysteresis, identity provisioning, resumption and application “effect” acknowledgements remain deliberately unresolved.

## Decision record

This record is non-normative. `docs/m0-spec-plan.md` is the executable plan; future `docs/spec/` files become normative only after review and ADR. No roadmap item is marked complete by this research submission. No code, Cargo result or security audit is claimed.
