# Nekomusume implementation status

**Status source:** This table is the single repository status source for the G0 governance baseline. It describes only evidence present at the exact commit carrying this file; it is not a protocol or security approval.

**Baseline:** `candidate/g0-governance-status` (parent `a2b7705c128e17ff4afcda8a74bff23c8c7a98b1` before this change)

| ID | Area | Status | Evidence | Boundary / next gate |
|---|---|---|---|---|
| G0 | governance and security release gate | blocked | `docs/adr/m1-g0-noise-ik-candidate.md` | STOP remains; no protocol freeze, network exposure, production or security claim |
| workspace | Rust workspace and crate boundaries | implemented | `Cargo.toml` | Research engineering baseline only |
| wire-codec | deterministic bounded wire codec | candidate | `crates/neko-wire/src/lib.rs` | Candidate format; no frozen interoperability contract |
| session-model | in-memory Session delivery state | candidate | `crates/neko-session/src/lib.rs` | Candidate model; correctness gaps remain |
| carrier-model | in-memory Carrier/Path state | candidate | `crates/neko-carrier/src/lib.rs` | Candidate model; no live carrier or failover |
| cli | CLI scaffold | implemented | `crates/neko-cli/src/main.rs` | Scaffold only; no client/server/probe transport |
| normative-spec | Session v0 normative entry point | provisional | `docs/specs/nekomusume-session-v0.md` | Provisional and not frozen |
| crypto-handshake | authenticated handshake and AEAD | absent | `docs/adr/m1-g0-noise-ik-candidate.md` | Candidate ADR only; implementation is not authorized |
| preauth-admission | runtime pre-auth accounting | absent | `docs/adr/m1-g0-preauth-resource-budget.md` | Documentation-only candidate budget |
| live-udp | UDP socket carrier | absent | `ROADMAP.md` | Requires later gated implementation |
| live-tcp | TCP carrier and resume | absent | `ROADMAP.md` | Requires later gated implementation |
| reachability | probe / public-network experiments | blocked | `ROADMAP.md` | Explicitly out of scope; no public exposure |
| production | production deployment/readiness | blocked | `SECURITY.md` | Research-only repository; no production or security approval |

## Status vocabulary

- **implemented** — repository evidence exists and is exercised, without implying protocol/security readiness.
- **candidate** — executable or documented candidate exists, but semantics are not frozen or approved.
- **provisional** — a planning/specification entry point exists and remains explicitly non-normative/non-frozen.
- **absent** — no implementation evidence exists in this repository.
- **blocked** — deliberately prohibited by the current governance boundary until named gates and review pass.

A status change must update this table and the evidence links in the same commit. `implemented` never means “production-ready”, “secure”, “interoperable”, or “publicly deployable”.
