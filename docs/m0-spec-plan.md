# M0 Specification and Implementation Plan

**Protocol research: 伊冯；Security research: 阿米娅；Transport research: 符玄；Plan & implementation: 庄方宜；Coordination/Review: 佩丽卡**
**检索日期：2026-08-26**
**Status: executable plan; not an implemented milestone.**

## Normative governance

External standards are cited from RFC Editor. Once implementation begins, the versioned document [`docs/specs/nekomusume-session-v0.md`](specs/nekomusume-session-v0.md) is the only Nekomusume normative source; README and ROADMAP are indexes, while `docs/decisions.md` records decision status and links to the spec/ADR. Research documents are non-normative. Every normative change requires an ADR, updated vectors, and review; historical `docs/design-handoff.md` is explicitly non-normative and retained for context only.

## Minimum model

- `Session`: authenticated logical identity, epoch, streams, delivery state.
- `Carrier`: UDP/TCP/etc. mechanism and carrier-local feedback.
- `Path`: concrete Carrier instance, address and `active_path_epoch`.

Three evidence domains must remain typed and separate: `packet_feedback`, `session_delivery`, `path_validated`.

## Proof object boundary

M0 must state whether an ACK proves `received` (peer parsed bytes), `delivered` (peer handed data to the logical stream), or `effect` (application acted). These are not interchangeable. Only the selected proof can promote `in_flight` to `confirmed`; packet feedback alone cannot. Unknown or unauthenticated control state never changes delivery state.

## M0 decisions and gates

1. Create a virtual Cargo workspace with explicit resolver and stable toolchain policy; no false implementation claims.
2. Define `neko-wire`, `neko-session`, `neko-carrier`, and `neko-cli` boundaries. Keep codec/state deterministic and independent of socket/runtime.
3. Freeze wire magic/version/type/length, canonical integer encoding, maximum sizes, truncation/unknown-version/type errors.
4. Freeze Session stream/offset/range state, duplicate/conflict rules, `active_path_epoch`, and path challenge contract.
5. Choose TLS 1.3 vs Noise only through a security-reviewed ADR; do not write crypto before that gate. License selection likewise requires maintainer decision and SPDX/manifest consistency.
6. Add at least 20 deterministic golden encode/decode vectors covering valid, boundary and invalid cases.
7. Add decode fuzz target with oracle: no panic, OOB, unbounded allocation, or acceptance of impossible lengths. T-FR-01 is the transport failover/replay test plan, not an M0 live-network claim.

## Acceptance

M0 acceptance is documentation/spec review plus deterministic implementation tests once code is authorized: vector count >=20; all proof/state transitions tested; uncertain replay is idempotent and bounded; path validation is independent; no runtime/socket dependency in core. Fuzz smoke is nightly-only and must preserve failing artifacts. No cargo result may be claimed until code exists.

## Planned files

`Cargo.toml`; `crates/neko-wire`, `crates/neko-session`, `crates/neko-carrier`, `crates/neko-cli`; `docs/spec/m0-wire-format.md`; `docs/spec/m0-session-state.md`; `tests/vectors/`; `fuzz/`; `.github/workflows/ci.yml`; `deny.toml`; ADRs for runtime/no_std, handshake and license. This document does not create them.

## Proposed commit sequence

1. `docs: record standards-grounded M0 decisions`
2. `build: add Rust workspace and core crate boundaries`
3. `feat(wire): define v0 codec limits and golden vectors`
4. `feat(session): add delivery state and replay dedup tests`
5. `test(fuzz): add decode smoke and dependency governance`

Every commit body must include actual `Verification:` output and the research-contributor trailers requested by the maintainer. Co-authored-by is omitted without real identities.
