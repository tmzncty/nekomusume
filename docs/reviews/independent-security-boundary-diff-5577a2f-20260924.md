# Independent security-boundary owner-diff / reuse review — `5577a2f`

**Repository anchor:** reachable `5577a2ff3c4fe1f99d04709a8ca6a7d5de9dc591`.

**Product/test source anchor:** `68d938279478b91661f786e830e99af83db457f1`. Commits after that source anchor through `5577a2f` are reviewer navigation/evidence documentation only and do not move product/test owners.

**Result:** bounded **NO-FINDING / REUSE** for the inspected current security engineering boundaries. This does not close D019, RSEC-001, the retained-history/resource-capacity policy gaps, signing/key-custody/SBOM/publication, release item 3, release item 4, security approval, RC, production readiness, freeze, or release.

## Owners and claims inspected

- `SECURITY.md` current research-prototype boundary and red lines: no home-grown crypto, no nonce reuse, unauthenticated clear control data cannot change connection state, bounded amplification/resources, no secret/plaintext logging, no arbitrary open proxy, and no production replacement before security/benchmark review.
- `docs/research/security-threat-model.md` trust/authentication, replay, path-validation, resource and logging research gates; it remains non-normative research input, not an audit or primitive/library approval.
- `docs/specs/nekomusume-session-v0.md` provisional Session evidence separation and governance boundary; packet/path feedback is not Session delivery evidence and the document remains unfrozen.
- `crates/neko-crypto/src/lib.rs` current Noise IK / trust-authz / transcript-context / nonce / replay / key-phase integration owner.
- `crates/neko-cli/src/preauth.rs` and production pre-auth call sites in `crates/neko-cli/src/main.rs` for admission/resource/rejection ownership.
- `docs/release-security-review-packet.md` current evidence-index boundaries and the latest reachable pre-auth/crypto/resource review notes.

## Exact owner-diff result

1. `SECURITY.md` has not moved since its 2026-08-25 M0 research-plan amendment (`381b198b8ef553f6d546ac468ad770e0a1b7f151`). Its current claims therefore predate and are constrained by the later bounded implementation/review evidence rather than silently expanding with it.
2. `crates/neko-crypto/src/lib.rs` has not moved after the dedicated independent crypto integration/API review at exact `9697ee7a0045b39c6ef46c1951fefea486ccf13b`; repository history shows its latest source change before that review. The prior review remains applicable to the current owner for Noise pattern/prologue binding, trust/authz before protected data, direction/context/key-phase separation, nonce exhaustion, replay/old-phase rejection, key update, secret-safe error surfaces and pre-allocation size bounds.
3. The latest dedicated pre-auth rejection/resource-accounting review is `docs/reviews/independent-preauth-rejection-accounting-d96aabe-20260922.md`. The later exact owner-diff review `docs/reviews/independent-r-preauth-diff-68d9382-20260924.md` already established that `crates/neko-cli/src/preauth.rs`, `crates/neko-crypto/src/lib.rs`, and production pre-auth call sites did not move through product/test source anchor `68d9382`. No product/test source moved after that anchor through this review.
4. The current Session v0 document remains provisional and does not promote packet/path observations to logical delivery. No current owner movement was found that would invalidate the existing trust/authentication/transcript/replay separation or create a new security-semantic implementation lane.

No concrete current-owner counterexample was found in this bounded owner/claim-diff pass, so there is no dependency-ready security code repair to manufacture.

## Known gates preserved — not findings invented by this pass

- **D019 source-retention/no-reset remains a maintainer/security policy gate.** Do not select TTL/LRU/history/capacity/security values or reset semantics here.
- **RSEC-001 remains HIGH for public-listener release/security promotion.** Representative adversarial-load suitability and the separate security/release decision remain absent; bounded engineering-control no-findings do not close it.
- `SECURITY.md` requires per-connection and global memory/CPU/rate bounds. The already-recorded `SessionRuntime.events` retained audit-history bound gap remains `POLICY_BLOCKED_RESOURCE_BOUND`; this review does not invent a capacity value to close it.
- Persistent restart/rollback replay safety remains outside the bounded crypto integration review and is not promoted by in-memory replay-window evidence.
- Signing, key custody, SBOM/publication trust, cryptanalysis/independent security audit, production/public-listener approval and release authority remain external/policy gates.
- H-I4-119 remains a separate **MAINTAINER / CORE ACK-PTO SEMANTICS GATE**. Nothing in the security owner diff resolves the disputed `pto_count` reset rule.

## Evidence boundary

This slice used current GitHub source/document contents plus reachable owner history and existing independent review anchors. It did **not** run reviewer-local Rust/full-gate commands, hosted CI, fuzz, WAN, cross-platform execution, adversarial-load/capacity benchmarks, penetration testing or performance experiments. It does not promote developer-local provenance into reviewer or hosted evidence.

No decoder/parser/crypto-framing owner changed in this slice, so no fuzz execution is requested. No code, protocol architecture, security numeric policy, release flag or live classification is changed.

`RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`; release items 3 and 4 remain incomplete; `READY_LIVE: none` remains unchanged.

## Queue consequence

`R-SECURITY-BOUNDARY-DIFF` closes as current-owner **NO-FINDING / REUSE**. `R-RPKT-CURRENT` remains the immediate factual-maintenance lane because `docs/release-security-review-packet.md` still lacks the latest H-I4-116/117/118 repair/provenance chain, H-I4-119 policy reclassification and current owner-diff/review references, and its older Session-delivery row still says “no independent review” despite later bounded independent Session reviews. After packet maintenance, rerun the thirteen-surface owner inventory; any real new developer source movement refills READY_LOCAL immediately.