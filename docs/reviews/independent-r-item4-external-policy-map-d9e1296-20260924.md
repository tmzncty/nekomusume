# R-ITEM4-EXTERNAL/POLICY MAP — exact-current residual release-item-4 gaps

**Repository anchor:** reachable `d9e12964a5db3d3f2d94e1527c5ada4a9ef26c40`.

**Product/test source anchor:** reachable `4511e4f147eb9511907bb4ec52cad687f113bb7a`. Repository movement after that anchor through this map is documentation/provenance/review navigation only.

**Result:** residual item-4 work is now classified without converting policy/external/environment gates into invented coding work. No new `READY_LIVE` question is created. This map is release-review support, not a security approval, production authorization, protocol freeze, RC decision, or release decision.

## Residual map

| Residual | Exact-current classification | Why / permitted next action |
|---|---|---|
| **H-I4-119 — ACK/PTO reset semantics** | **MAINTAINER / CORE SEMANTICS** | Repository/spec truth does not choose between reset-after-any-newly-acked-sent-packet and ack-eliciting-only reset. Keep current authorized baseline until a maintainer/spec decision; do not auto-patch or treat rollback as final policy. |
| **D019 source-retention / no-reset semantics** | **MAINTAINER / SECURITY POLICY** | D019 candidate budgets and counting domains are non-frozen; source-retention/reset semantics and any TTL/LRU/history/security-value change require explicit reviewed amendment. Existing bounded engineering controls remain reusable; no reviewer may invent values. |
| **`SessionRuntime.events` retained audit-history capacity** | **MAINTAINER / SECURITY NUMERIC POLICY** | Correctness owner is reviewed, but the retained audit/event history lacks an approved capacity bound. `SECURITY.md` requires bounded resource use; choosing a cap/history policy is a maintainer/security value decision, not an automatic code repair. |
| **RSEC-001 adversarial-load suitability / public-listener security promotion** | **MAINTAINER/SECURITY + RELEASE GATE** | Current bounded engineering-control reviews do not establish representative adversarial-load suitability or public-listener/security approval. Benchmark/load conditions that amount to capacity/security acceptance require maintainer choice; do not manufacture pressure targets. |
| **Persistent restart/rollback replay safety** | **BLOCKED DEPENDENCY / SECURITY-PROTOCOL-PERSISTENCE GATE** | In-memory replay/key-phase evidence does not prove persistence across restart/rollback. A durable persistence/rollback contract and normative security semantics must exist before a dependency-ready implementation/review slice can be defined. No current automatic repair is authorized. |
| **Signing / key custody / SBOM / publication trust** | **MAINTAINER / POLICY / RELEASE ENGINEERING GATE** | Package reproducibility/install/rollback evidence exists, but publication trust policy and key custody are intentionally unresolved. Do not select signing scheme, custody model, SBOM/publication policy, or public release process automatically. |
| **Independent external/security review acceptance** | **EXTERNAL / RELEASE AUTHORITY** | Internal bounded independent review support is extensive but is not an external security audit or final release/security approval. Acceptance criteria/authority are outside coding-agent discretion. |
| **Release item 3 — IPv6 row** | **BLOCKED ENVIRONMENT** | No owned IPv6 endpoint/path is currently available. Standing authorization would allow bounded owned IPv6 work if the environment becomes real; absence of environment is not a local coding defect. |
| **Release item 3 — repeated warm failover / periodic / HY2 frozen current lines** | **BLOCKED ORCHESTRATION CURRENT LINE / NO SAME-CLASS RETRY** | Existing negative evidence is retained and same-class retries are frozen unless code/instrumentation/configuration/hypothesis/path condition materially changes. Current repository truth creates no such new question. |
| **Release item 3 — general/public NAT/reachability and natural degradation matrix completion** | **BLOCKED DEPENDENCY / EVIDENCE GAP**, not `READY_LIVE` | Existing self-owned bounded observations answer narrower questions only. The current line has no new materially changed runtime/instrumentation hypothesis that would make another live run truthful and non-duplicative now. |
| **RC decision** | **EXTERNAL / RELEASE AUTHORITY** | `IMPLEMENTATION_PLAN.md` requires release items 1–4 before an explicit reviewed RC decision. Item 3 and item 4 remain incomplete; no agent may set `RELEASE_CANDIDATE=true`. |
| **Protocol freeze / release / production readiness / production authorization** | **EXTERNAL / RELEASE AUTHORITY** | These remain separate decisions and do not follow from implementation completeness or bounded evidence. `FREEZE=false`, `RELEASED=false`, `PRODUCTION_READY=false` remain authoritative. |

## What is still READY locally?

No new dependency-ready product-code repair emerges from the residual policy/external map itself. The current automatic reviewer work is therefore **review-support/history reconciliation**, not invented implementation:

1. rerun the exact-current thirteen-surface owner history after packet/map docs commits;
2. if any semantic product/test owner moved, immediately create the corresponding bounded review/repair lane;
3. if no semantic owner moved, keep current reusable reviews valid and perform the final repository-wide reconciliation before any queue-exhaustion statement;
4. if future code/instrumentation/hypothesis/path changes create a concrete new WAN question within standing authorization, then and only then reclassify the specific row as `READY_LIVE`.

## Live classification

**`READY_LIVE: none`.** The rented VPS remains valuable, but current authoritative rows either already have bounded answers, are frozen same-class negatives, lack environment, or require a material implementation/instrumentation/hypothesis/path change first. The rental window is not permission to repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track merely to generate activity.

## Evidence/release boundary

- Release item 3 remains incomplete.
- Release item 4 remains incomplete.
- `RELEASE_CANDIDATE=false`.
- `PRODUCTION_READY=false`.
- `FREEZE=false`.
- `RELEASED=false`.

This classification uses exact-current repository documents and reachable review/evidence anchors. No reviewer-local Rust/full-gate execution, hosted-CI success, fuzz, cross-platform run, WAN experiment, adversarial-load benchmark, performance result, penetration test, security approval or release action is claimed.

## Next lane

Proceed immediately to **R-FINAL-OWNER-HISTORY**: compare exact-current history against the latest dedicated review anchors for all thirteen required surfaces. Documentation-only movement does not invalidate semantic-owner reviews; any actual source/test semantic movement refills `READY_LOCAL` immediately.
