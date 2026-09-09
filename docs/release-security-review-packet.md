# Independent release/security review packet

**Scope:** evidence index only. This is not an audit, security approval, production authorization, protocol freeze, or release decision.

**Evidence indexed and developer local gates rerun through reachable exact commit:** `188d5a5d7c0f87c5554b6673ad56284dfc7d7138`

The packet text is updated after that tested tree; it is not a self-referential commit attestation. Minimal developer local-CI provenance for the package source-state guard is retained in [`docs/local-ci-188d5a5-20260909.md`](local-ci-188d5a5-20260909.md); the earlier exact-`b3bfe97` local run is retained separately, and GitHub-hosted run `34298672407` is separate cross-evidence.

This packet is an evidence index, not a self-referential commit attestation; later packet-text commits may update the index without changing the tested tree.

## Review boundaries

- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.
- Bounded release item 3 and natural-loss evidence remain unchecked.
- Existing historical artifacts remain immutable; current exact-tree evidence is indexed without rewriting them.
- The packet indexes one bounded self-owned VPS package install/authenticated TCP+UDP smoke; this remains operator evidence, not release, production, public-reachability, or security approval.
- The exact `14be1c8` installed-package attempt remains an incomplete orchestration negative. The single changed-hypothesis exact-`7cebe6b` invocation is separately anchored by a sanitized 26-record result and provenance erratum; it is bounded operator evidence only.
- The exact `91a735c -> dc90c5f -> 91a735c` distinct-package rehearsal preserves external temporary identity/state for the bounded probe contract; its five-record sanitized anchor is retained, while experiment start/end timing was not retained; it is not arbitrary state-schema compatibility or production upgrade approval.

## Evidence index

| Area | Exact repository evidence | Boundary |
|---|---|---|
| Canonical corpus and executable identity | [`docs/spec/canonical-vector-review.v1.md`](spec/canonical-vector-review.v1.md), [`docs/status.md`](status.md) | Candidate/research identity only; no frozen interoperability or release claim |
| Negotiation, transcript binding, Noise/trust/authz | [`docs/specs/nekomusume-session-v0.md`](specs/nekomusume-session-v0.md), [`docs/research/security-threat-model.md`](research/security-threat-model.md), [`crates/neko-session/src/lib.rs`](../crates/neko-session/src/lib.rs) | Deterministic implementation evidence; independent security review remains absent |
| Wire/parser/fuzz | [`scripts/check.sh`](../scripts/check.sh), [`scripts/fuzz-smoke.sh`](../scripts/fuzz-smoke.sh) | Stable and fuzz smoke gates, not proof of production safety |
| Pre-auth/resource limits | [`docs/reviews/resource-abuse-evidence-2026-09-04.md`](reviews/resource-abuse-evidence-2026-09-04.md), [`docs/adr/m1-g0-preauth-resource-budget.md`](adr/m1-g0-preauth-resource-budget.md), [`crates/neko-crypto/src/lib.rs`](../crates/neko-crypto/src/lib.rs), [`crates/neko-cli/src/preauth.rs`](../crates/neko-cli/src/preauth.rs) | `ENGINEERING_CONTROLS_REVIEWED` for bounded non-policy controls; D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`; adversarial-load suitability and security/release/public-listener approval remain absent |
| ACK and carrier-feedback separation | [`docs/spec/m3-tcp-failover.md`](spec/m3-tcp-failover.md), [`crates/neko-carrier/src/lib.rs`](../crates/neko-carrier/src/lib.rs) | Loopback/candidate evidence; no public service claim |
| Failover/resume positive and negative boundaries | [`docs/era3-closure-2026-08-30.md`](era3-closure-2026-08-30.md), [`docs/status.md`](status.md) | Exact bounded samples and negatives remain scope-limited |
| Package lifecycle and archive validation | [`docs/decisions.md`](decisions.md), [`scripts/release/build-package.sh`](../scripts/release/build-package.sh), [`scripts/release/check-clean-source-test.sh`](../scripts/release/check-clean-source-test.sh), [`scripts/release/smoke-package.sh`](../scripts/release/smoke-package.sh), [`scripts/release/smoke-package-test.sh`](../scripts/release/smoke-package-test.sh), [`docs/package-operator-current-tree-f19ad28-20260909.md`](package-operator-current-tree-f19ad28-20260909.md), [`docs/package-operator-vps-a9928c8-20260909.md`](package-operator-vps-a9928c8-20260909.md), [`docs/package-operator-distinct-version-91a735c-dc90c5f-20260909.md`](package-operator-distinct-version-91a735c-dc90c5f-20260909.md) | Exact-tree reproducibility/operator evidence plus pre-build clean-source enforcement and fail-closed archive shape/type/path/layout/archived-mode/checksum regressions; no signing/key-custody/SBOM, arbitrary state-schema compatibility, deployment, release, or production approval |
| Operator readiness/shutdown/cleanup | [`docs/status.md`](status.md), [`crates/neko-cli/tests/probe.rs`](../crates/neko-cli/tests/probe.rs), [`docs/package-operator-lifecycle-negative-14be-20260909.md`](package-operator-lifecycle-negative-14be-20260909.md), [`docs/package-operator-lifecycle-7cebe6b-20260909.md`](package-operator-lifecycle-7cebe6b-20260909.md) | Local deterministic tests and the changed-hypothesis exact-`7cebe6b` installed-package VPS run cover ordered `READY -> DRAINING -> STOPPED`, listener release, same-address TCP/UDP rebind, and authenticated post-restart exchange; this remains bounded operator evidence, not service-manager or production approval |
| VPS/reachability matrix | [`docs/reachability-matrix.md`](reachability-matrix.md), [`docs/era4-ledger-2026-08-30.json`](era4-ledger-2026-08-30.json), [`docs/status.md`](status.md) | Bounded endpoint-rebind, migration-back and key-update observations exist; IPv6 is `BLOCKED_ENVIRONMENT`; repeated warm failover is frozen at its orchestration negative; live PMTUD requires a separate accepted wire/security design gate |
| HY2 methodology/result boundary | [`docs/status.md`](status.md), [`docs/era4-ledger-2026-08-30.json`](era4-ledger-2026-08-30.json), [`scripts/bench/validate-hy2-owned-lab.py`](../scripts/bench/validate-hy2-owned-lab.py) | Exact `13da094` is `BLOCKED_HARNESS_CURRENT_LINE_HY2` at typed `unknown / client_started`; no complete pair or performance result; same-class retry frozen. Exact `61a6490` remains historical local-preflight evidence only |
| Exact-current item-4 factual review support | [`docs/reviews/release-item4-subgates-20260909.md`](reviews/release-item4-subgates-20260909.md) | Developer-prepared exact-tree facts only; not independent review, security approval, RC, release, or production authorization |
| Unresolved release/security findings | [`docs/spec/m5-release-readiness-gate.md`](spec/m5-release-readiness-gate.md), [`IMPLEMENTATION_PLAN.md`](../IMPLEMENTATION_PLAN.md), [`ROADMAP.md`](../ROADMAP.md) | Governance gate remains open; no release promotion |

## Current classifications

The machine-readable closure is the reviewed navigation source: [`docs/era4-ledger-2026-08-30.json`](era4-ledger-2026-08-30.json). It distinguishes `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION` from `OPEN_READY`, `BLOCKED_DEPENDENCY`, `BLOCKED_IMPLEMENTATION`, `BLOCKED_ENVIRONMENT`, `BLOCKED_ORCHESTRATION_CURRENT_LINE`, and `GOVERNANCE_GATE`. `OPEN_READY` requires a specific unresolved question, `evidence_needed`, a concrete `next_action`, satisfied dependencies, and an explicit local/VPS scope; a row whose declared dependency is blocked or governance-gated cannot be open-ready. It is an opportunity classification, not authorization to execute a row. `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION` rows must not be re-run merely because more testing is possible.

Current actionable next row: **none for live execution**. The next safe action is an independent maintainer/security review of this packet and the linked evidence; standing authorization and technical readiness remain separate, and this packet does not authorize or de-authorize VPS/live work.
