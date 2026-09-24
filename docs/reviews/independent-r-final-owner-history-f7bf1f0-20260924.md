# R-FINAL-OWNER-HISTORY — exact-current thirteen-surface owner history at `f7bf1f0`

**Repository anchor:** reachable `f7bf1f0d2b55ab69834c119cfc6310c9a2dce0be`.

**Latest product/test source anchor:** reachable `4511e4f147eb9511907bb4ec52cad687f113bb7a`.

**Result:** repository-wide bounded **NO NEW SEMANTIC OWNER MOVEMENT / NO NEW READY_LOCAL REPAIR** across the thirteen required release-item-4 surfaces. This is a history/refill result, not item-4 closure, security approval, RC/freeze/release authority, or production readiness.

## History checked

The exact-current history from `4511e4f...` through `f7bf1f0...` contains review/provenance/handoff/release-packet maintenance plus the reviewed merge of PR #4 (`research: define middlebox and reachability simulation plan`). PR #4 changes only:

- `docs/carrier-architecture.md`, adding an explicitly research-scoped middlebox/network-behavior boundary and open research question;
- new `docs/research/middlebox-reachability-and-network-behavior.md`, an explicitly non-freezing research plan.

The `docs/carrier-architecture.md` blob at PR #4's historical base was identical to current main before the PR merge, so the merge does not discard a later main-side architecture edit. The new text explicitly preserves Session/Carrier separation, prefers ordinary carrier/path repair before traffic-appearance research, forbids third-party/access-control bypass work, and makes no release/security/production or current `READY_LIVE` claim. Its executable ideas are explicitly future/dependency-ready research slices and do not outrank current release/security gates.

No Rust source, integration/unit test, Cargo manifest/lock, build/native hook, CLI/process harness, package/operator script, wire/parser/crypto owner, or current release/status flag moved after `4511e4f...`.

## Thirteen required surfaces

| # | Surface | Exact-current result |
|---|---|---|
| 1 | `neko-reliable` UDP recovery — ACK range/future/stale ACK/loss/retransmit/RTT/PTO/persistent congestion/Reno/fault simulation | **CURRENT / REUSE.** Post-H-I4-120 Recovery delta independently re-challenged at `4511e4f...`; H-I4-119 remains an explicit maintainer/core semantics gate and is excluded from automatic repair. |
| 2 | `CarrierState` generation/validation/hysteresis/single-active/drain/fail/activate | **CURRENT / REUSE.** R-CS-1/R-CS-2 remain applicable; PR #4 adds only research context, no state-machine rule. |
| 3 | Concurrent Carrier Manager / health / migration-back | **CURRENT / REUSE.** Manager/health/migration-back semantic owners did not move. The middlebox plan does not change D064 ownership/readiness/single-active semantics. |
| 4 | FairScheduler / multi-stream / Session + stream flow-control accounting | **CURRENT / REUSE.** No owner movement. |
| 5 | Memory/UDP/TCP carrier adapter close/error/resource semantics | **CURRENT / REUSE.** No adapter owner movement. |
| 6 | `SessionRuntime` lifecycle/resource/window/DeliveryAck accounting | **CURRENT / REUSE** for reviewed correctness; retained `events` history capacity remains policy-gated, not an invented numeric repair. |
| 7 | `neko-observe` projection/event/counter/high-water correctness | **CURRENT / REUSE.** No owner movement. |
| 8 | package/reproducibility/operator scripts | **CURRENT / REUSE.** No owner movement; signing/key-custody/SBOM/publication remain separate policy/release gates. |
| 9 | dependency/build manifests/lock/features/build/native hooks/unsafe inheritance | **CURRENT / REUSE.** No owner movement. |
| 10 | cross-platform CLI/process-test semantics | **CURRENT / REUSE** with existing evidence-platform boundary; no owner movement. |
| 11 | CLI exit-code / JSON / human-output contract | **CURRENT / REUSE.** No owner movement. |
| 12 | algorithmic resource boundedness | **CURRENT / REUSE** for implemented numeric-free invariants; retained-history/capacity/security-value questions remain maintainer/policy gates. |
| 13 | release packet factual consistency / evidence boundary | **CURRENT / independently checked.** Packet reconciliation is current and `docs/reviews/independent-r-rpkt-check-cc89045-20260924.md` closed its bounded verification with no finding. |

## Open PR / issue truth

PR #4 was the sole open pull request discovered in this pass. Its two docs-only commits were independently read against current architecture/authorization/release boundaries and merged as squash commit `f7bf1f0...`. A fresh open-PR query immediately after the merge returned none.

Historical issues #1 and #2 remain open, but their checkbox/body state predates the current implementation/status ledger. Per repository precedence, current code/tests/`docs/status.md`/current plans outrank those stale issue checklists; they are not used to manufacture duplicate M0/reachability work.

## Refill consequence

No current source/test semantic movement creates a new READY_LOCAL implementation or bounded core-review lane. Current dependency-ready reviewer work is therefore the **final repository-wide reconciliation / queue-state check**, using:

- this exact-current thirteen-surface history;
- `docs/reviews/independent-r-item4-external-policy-map-d9e1296-20260924.md`;
- the current release/security packet + independent packet check;
- current `READY_LIVE: none` classification;
- open-PR truth after PR #4 merge.

If that final reconciliation confirms no concrete defect, no unreviewed implemented owner, no READY review-support lane, no READY live question, and all residuals are genuinely policy/external/environment/release-authority gated, then repository-wide queue exhaustion is a valid current-state conclusion. It must be immediately invalidated by any new product/test commit, new PR/review work, new concrete defect, or materially changed live hypothesis/path condition.

## Evidence boundary

This is exact-current GitHub source/history/review-ledger analysis. No reviewer-local Rust/full-gate execution, developer-local exact-tree gate on `f7bf1f0...`, hosted-CI success, fuzz, cross-platform execution, WAN run, adversarial-load benchmark, performance result, security audit, protocol freeze, RC/release or production action is claimed. PR #4 is docs/research-only, so no mechanical fuzz run is requested.

`READY_LIVE: none`; release items 3 and 4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
