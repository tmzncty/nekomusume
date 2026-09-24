# Release item-4 factual reconciliation — recovery + CarrierState current sweep

**Exact reconciliation anchor:** `44fd3d69edd9802a73721a10596e0f19ac04f404`

This is a factual boundary check after the coherent H-I4-116/117/118 + R-REC + R-CS review group. It does not grant release/security approval and does not replace the per-tree execution provenance.

## Current facts

- H-I4-116 (Reno zero-loss collapse), H-I4-117 (runtime persistent-congestion wiring), and H-I4-118 (stale ACK time-thresholding newer packets) have reachable repair/test commits and reachable developer-local exact-tree provenance notes. Their local evidence remains developer-reported local evidence; it is not reviewer-local execution or hosted CI by implication.
- The current recovery refill now has dedicated bounded challenges for ACK validity, loss/retransmit ownership, RTT/PTO/persistent congestion, aggregate positive-loss Reno behavior, deterministic fault simulation, and the repaired stale-ACK frontier.
- R-CS-1 at exact `270265f` found no concrete defect in generation / validation / hysteresis under the committed M0 model. R-CS-2 at exact `259fb58` found no concrete defect in the bounded M0 single-active / drain / fail / activate state machine and explicitly does not project that old model onto the richer D064 runtime manager.
- R-CM-DIFF at exact `44a77cb` found no moved concurrent-manager / health / migration-back semantic owner that invalidates the existing dedicated reviews; post-anchor carrier changes are in separately reviewed MemoryCarrier and Recovery/Reno/PTO seams.

## Packet / status / plan boundary

`docs/release-security-review-packet.md` remains conservative: it is explicitly an evolving evidence index, keeps item-3 and independent-review limitations, and does not claim the recent review notes make item 4 complete. Its older `Developer-reviewed factual coverage` marker therefore understates later review work rather than creating a false positive release claim. A broad packet-index rewrite is deferred until a larger evidence group closes; do not churn it per small slice.

`IMPLEMENTATION_PLAN.md` remains authoritative that release item 3 and item 4 are unchecked. `docs/status.md` still keeps the research/candidate boundaries and the independent release flags false. No current repository fact supports changing `RELEASE_CANDIDATE`, `PRODUCTION_READY`, `FREEZE`, or `RELEASED`.

The authoritative live classification remains `READY_LIVE: none`; the standing self-owned VPS authorization remains available but creates no reason to repeat frozen/answered live rows without a new code/instrumentation/hypothesis/path-condition question.

## Open item-4 lanes retained

1. package/reproducibility/dependency-build owner-diff reuse after the recent CLI/process/recovery changes;
2. observability + carrier-adapter owner-diff reuse;
3. FairScheduler / multi-stream / SessionRuntime current-owner diff and targeted re-challenge where moved;
4. CLI exit-code / JSON / human-output + cross-platform process/boundedness reuse against latest owner tree;
5. repaired Recovery owner refill after H-I4-116/117/118 (target only changed seams not already independently challenged);
6. broad repository 13-surface inventory/refill;
7. final packet/status/evidence-boundary reconciliation only after those lanes materially close.

**Result:** no new BLOCKER/HIGH and no release-stage transition. Queue is not exhausted.