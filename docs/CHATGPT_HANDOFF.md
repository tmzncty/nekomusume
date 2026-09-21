# ChatGPT reviewer handoff — item-4 reconciliation front after PORT / CLI / BND closure group

## Current repository truth

- Current reachable reviewer anchor immediately before this handoff update is exact `031f7b814621bc7c2853f0bbadc9ff09bc2adbed` (`docs(review): complete current I4-BND Recovery challenge`). The current executable/source-test tree remains exact `42be53917ee58359ad86532a6aab0136f75047b9`; commits after it through `031f7b8` are documentation/review/handoff only.
- **H-I4-087 is CLOSED** at executable repair `d819d38559d60b6bd99df8a2453321809c046116`: both Session admission directions enforce the same aggregate `send.len() + recv.len()` record cap before mutation; send→receive, receive→send and opposite-direction slot release are regression-covered. Developer-reported clean exact-tree provenance is retained in `docs/notes/h-i4-087-provenance-d819d38-20260921.md`. The first full gate run recorded one startup/socket timing flake and the second full run passed; preserve that fact rather than rewriting it as an always-green first run.
- **H-I4-088 is CLOSED** at executable repair `26aa4e8036d61da7924d9fc5e587081d8a0f448f`: MemoryCarrier empty messages can no longer grow the peer queue without consuming a finite record slot; receiving a record releases the slot and the existing byte cap remains unchanged. Developer-reported clean exact-tree provenance is retained in `docs/notes/h-i4-088-provenance-26aa4e8-20260921.md`.
- **I4-PORT-01 is CLOSED** at executable test repair `42be53917ee58359ad86532a6aab0136f75047b9`: the five SIGTERM/process-lifecycle fixtures and `signal_term` are explicitly Unix-scoped rather than false cross-platform evidence. Its developer-reported clean exact-tree provenance is retained in `docs/notes/i4-port-01-provenance-42be539-20260921.md`.
- **I4-CLI-M is CLOSED with bounded no finding** at `docs/reviews/independent-i4-cli-machine-1b1b8c8-20260921.md` / reachable `882605a`. It challenged machine JSON/exit/process-result truth only; it does not stand in for human-output review.
- **I4-CLI-H has a developer bounded no-finding note** at reachable `536d59a545220bf48d4a9cd73821e572033229aa`. Preserve one qualification during reconciliation: the exact-current reachability gate does **not** parse a typed semantic object at the final human/exit branch; it uses `artifact.contains("\"reachable\":true")`. On the current one-case `reachability-matrix.v1` shape this was not falsified as a concrete defect, but the note's phrase “driven by the parsed result” is too broad. A future schema/multi-case or alternate `reachable` occurrence is a re-review trigger.
- **I4-BND developer review** landed at reachable `89d249b9f2b64a6cac5be9639743a86348dcb8bd`. It covered Session live/queued ownership, health/evidence vectors, failover/scheduler/MemoryCarrier and bounded CLI loops, and correctly leaves `SessionRuntime.events` retained-history capacity plus D019 outside automatic policy changes.
- The developer I4-BND note did not explicitly inspect current `neko-reliable::Recovery`, so reviewer completion `docs/reviews/independent-i4-bnd-recovery-current-89d249b-20260921.md` / reachable `031f7b8` independently challenged the current Recovery + executable ACK seam. No concrete resource-boundedness defect was found: packet/frame ownership is admission-bounded, runtime ACK input is wire-bounded to 32 ranges before conversion, numeric ACK intervals are not expanded by packet-number cardinality, and quiesce releases live packet/frame/high-water ownership. This was source-level reviewer work; **no reviewer-local command execution is claimed**.
- Current exact source/test tree `42be539` has developer-reported full `scripts/check.sh` / `git diff --check` / clean-tree provenance. For `d819d38`, GitHub combined-status and commit-workflow queries returned no hosted run/status; absence of hosted evidence is not a failure and does not replace developer-local provenance.
- H-I4-085/086/087/088, Candidate A/B, I4-FS1/FS2, I4-AD1/AD2, I4-PORT, I4-CLI-M and the R9 closure group remain closed unless exact-current owner truth changes or reconciliation discovers a false claim.
- The known `SessionRuntime.events` retained-state capacity question remains a maintainer/security policy gate. Do not invent history-size/TTL/LRU/capacity values. D019 source-retention/no-reset remains policy-frozen.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track merely because a VPS is still rented. Only a new code/instrumentation/hypothesis/path condition that creates a concrete unresolved real-network question may reopen a live lane.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: exact-current independent review/challenge -> focused deterministic tests where applicable -> smallest repair if a source-decided defect is demonstrated -> commit -> push -> clean exact-tree gate/provenance -> next slice. Reviewer cadence is only a check frequency; do not wait for the next reviewer once a slice is closed.

## READY_LOCAL 1 — item-4 factual reconciliation for the PORT / CLI / BND closure group

Reconcile exact-current review/evidence truth across:

- `docs/release-security-review-packet.md`;
- `docs/status.md`;
- `IMPLEMENTATION_PLAN.md`;
- the H-I4-085/086/087/088 provenance notes;
- current FS1/FS2, adapter, PORT, CLI-M, CLI-H and I4-BND review notes;
- `docs/reviews/independent-i4-bnd-recovery-current-89d249b-20260921.md`;
- current executable-owner commits (`38379b1`, `3acba04`, `d819d38`, `26aa4e8`, `42be539`) only where they support the specific claim being reconciled.

Required outcomes:

- preserve developer-local vs reviewer-source-review vs reviewer-local execution vs hosted CI vs live WAN vs performance conclusions as distinct evidence classes;
- explicitly qualify/supersede the CLI-H “parsed result” wording noted above without manufacturing a source repair when the current one-case schema remains unfalsified;
- preserve H-I4-087's first-run timing flake and second-run pass truthfully;
- do not mark item 4 complete merely because this closure group is coherent;
- do not change release/governance flags or D019/policy values.

This is the appropriate 3–4+ slice reconciliation point; do not split it into checker/schema filler.

## READY_LOCAL 2 — repository-wide 13-surface independent-review inventory / refill

After the reconciliation, inventory all implemented core surfaces against a **reachable, dedicated, bounded, sufficiently current** independent challenge:

1. `neko-reliable` UDP recovery: ACK range/future ACK/loss/retransmit/RTT/PTO/persistent congestion/Reno/fault simulation;
2. `CarrierState`: generation/validation/hysteresis/single-active/drain/fail/activate;
3. Concurrent Carrier Manager / health / migration-back;
4. FairScheduler / multi-stream / Session+stream flow-control accounting;
5. Memory/UDP/TCP adapter close/error/resource semantics;
6. `SessionRuntime` lifecycle/resource/window/DeliveryAck accounting;
7. observability projection/event/counter/high-water correctness;
8. package/reproducibility/operator scripts;
9. dependency/build/Cargo lock/features/build/native hooks/unsafe inheritance;
10. cross-platform CLI/process-test semantics;
11. CLI exit-code/JSON/human-output contract;
12. algorithmic resource boundedness under existing limits;
13. release-packet factual consistency/evidence boundary.

Re-open only if an owner changed materially, a prior review claim is now false, or a core implemented owner never received a dedicated challenge. A no-finding bounded challenge is legitimate item-4 support. Queue exhaustion is repository-wide, never inferred from one lane.

## READY_LOCAL 3 — release packet / status / implementation-plan evidence-boundary spot-check

Independently compare `docs/release-security-review-packet.md`, `docs/status.md`, `IMPLEMENTATION_PLAN.md` and exact-current review/provenance anchors. The packet is visibly an evolving evidence index and its header/coverage must not imply current review breadth it does not actually index. Challenge:

- stale exact-SHA coverage statements;
- developer/local-only SHA promotion;
- hosted-CI absence being presented as failure or success;
- historical live negatives being rewritten as current success;
- item 3 or item 4 being implied complete;
- policy-blocked retained-state questions being silently turned into implemented limits.

If only documentation truth is stale, repair the smallest factual text. Do not rewrite historical artifacts.

## READY_LOCAL 4 — dependency/build/native current-owner spot-check if inventory shows a gap

Only if READY_LOCAL 2 shows the surface is not already sufficiently current, re-challenge Cargo manifests/lock/features/build scripts/native hooks/unsafe inheritance against H-I4-085's corrected truth: `snow` is a normal production dependency with a build script, and the active `ring` closure includes `build.rs`/`links`/`cc`/native C-assembly surface. Do not alter dependency/default-feature/crypto selection merely to simplify evidence prose.

## READY_LOCAL 5 — package/reproducibility/operator current-owner spot-check if inventory shows a gap

Only if the inventory finds a real freshness/coverage gap, inspect the current package build/install/archive/lifecycle/repro helpers and their exact-tree provenance boundaries. Reuse existing negative and bounded operator evidence rather than rerunning the same VPS/package scenario. A checker-only documentation loop is not progress.

## READY_LOCAL 6 — observability + retained/live-state current-owner spot-check if inventory shows a gap

Only if materially changed owners or missing dedicated coverage justify it, challenge `neko-observe` projection/counter/high-water semantics plus retained/live-state distinction. Candidate B remains closed unless code changed. Keep `SessionRuntime.events` capacity as the existing policy gate; do not invent a numeric history cap to make item 4 look closed.

## READY_LOCAL 7 — final independent item-4 sweep after refill

Once the inventory-driven gaps are closed, perform a repository-wide independent sweep over the still-implemented core owners and evidence boundaries. Each no-finding closure must state exact owners, exact reachable anchor, challenge invariant, tests/commands actually run versus merely read, and exclusions. Any concrete correctness/security/evidence defect returns to repair priority before an item-4 completion claim.

If the sweep still leaves only policy/environment/release-authority gates, state those precisely. Do **not** write `queue exhausted` while an implemented core surface still lacks dedicated current review support.

## READY_LOCAL 8 — conditional live question only

Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved self-owned real-network question, classify it against `docs/standing-vps-lab-authorization.md` and `docs/vps-rental-window-priority.md`. Ordinary bounded self-owned TCP/UDP work inside standing authorization needs no per-run approval, but the authoritative current classification is still `READY_LIVE: none`.

## Surfaces deliberately not duplicated unless repository truth changes

- H-I4-087 Session aggregate queue accounting, H-I4-086 ACK-before-drain, current FS2 flow-control, terminal cleanup and prior R9 SessionRuntime terminal reviews are separate, already challenged seams; do not collapse them into one vague “Session reviewed” claim.
- MemoryCarrier H-I4-088 is repaired and independently re-challenged; UDP/TCP adapters have separate current bounded reviews. Re-open only on owner change or falsified claim.
- `neko-reliable` current algorithmic boundedness is now specifically challenged by `031f7b8`; prior Recovery correctness reviews remain useful for non-boundedness invariants but are not silently treated as having reviewed later owner changes.
- `neko-observe` has no material post-review owner change in this closure group; Candidate B remains closed.
- Process-resource sampler lifecycle/result-truth was repaired/reviewed through H-R9-082/083/084 and then challenged again by I4-CLI-M; do not restart that chain without a new counterexample.
- Release-packet boundaries were previously reconciled by Q10/Q11/Q12, but the current coherent PORT/CLI/BND group now warrants one factual reconciliation pass; after that, avoid per-commit packet churn.

## Stop / escalation conditions

Continue review/repair -> tests -> commit -> push -> next dependency-ready slice without waiting for reviewer cadence. Stop/escalate only for an unresolved BLOCKER/HIGH that cannot be auto-decided, core Session/Carrier/ACK/crypto/wire architecture change, D019 or another policy/value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
