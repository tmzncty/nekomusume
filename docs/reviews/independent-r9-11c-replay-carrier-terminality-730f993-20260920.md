# Independent bounded R9-11C review — retained replay / Carrier-generation terminality

**Source/test anchor:** exact developer tree `730f993c687683f00f82859cbb7587d9fd6f5b84`.
**Developer review inspected:** `a2a1e6cf6b2890a836cc9d6271061897730585cb`.

## Scope challenged

Read the exact-current `FailoverController` replay owner, `CarrierManager` promotion/migration-back gates, and `ConcurrentCarrierManager` generation/readiness/assignment/replay lifecycle together with D005/D064, `docs/carrier-architecture.md`, and the Session-v0 evidence boundary. The challenge was limited to retained replay identity/bytes and Carrier-generation terminality. Executable process/socket cleanup remains R9-11D; D019 and all retention/capacity policy values remain out of scope.

## Independent challenge

No concrete defect was found on this bounded surface.

- `FailoverController::tcp_resend()` derives replay directly from the retained `uncertain` map and clones the exact `(DataId, bytes)` entries. `confirm(id)` is the explicit retention-ending sink; packet feedback is not consulted there. Exact duplicate receive is `Ok(false)`, while conflicting bytes for the same identity fail closed.
- `ConcurrentCarrierManager::fail()` checks capacity, marks every range owned by the failed active generation `uncertain`, then clears active ownership and marks that generation `Failed`. `replay_uncertain()` requires a new active owner and replays the stored stable `LogicalRangeId` plus stored bytes before rebinding ownership. `finish_drain()` similarly replays only the still-unconfirmed old-owner ranges after the drain deadline.
- Generation replacement is fail closed: `register()` accepts a greater generation only after the previous generation is `Failed`; `path`/`path_mut` reject stale and mismatched generations; readiness rejects `Failed`, `Draining`, and `Active` states; activation requires `Warm`.
- The migration-back path checks active generation, explicit validation, health, score margin, and hold before mutation. Rejected candidates do not acquire active ownership merely from packet/readiness observations.
- Capacity/conflict checks occur before replay-owner mutation in the reviewed manager paths, so a rejected transition does not silently discard retained bytes or manufacture switch/delivery success evidence.

The developer note at `a2a1e6c` names the reachable deterministic regressions that exercise these seams. I also checked the exact owner implementations rather than accepting that note as proof by itself.

## Evidence boundary

This is source/test reasoning over reachable GitHub state; **no reviewer-local command execution is claimed**. The developer-local clean exact-tree provenance already recorded for exact `730f993` remains developer-reported provenance. GitHub exposes no commit-status/workflow-run record for `730f993` through the connected status endpoints; absence of hosted status is not treated as failure. No decoder/parser/crypto-framing code changed, so no new fuzz run is required by this review.

This review does **not** re-prove every executable caller that invokes `confirm`; that caller/evidence coupling was handled in the preceding R9 process/replay slices and remains a target of the final independent R9 review. Within R9-11C's ownership/generation scope, no BLOCKER/HIGH was found.

**R9-11C: bounded independent no-finding support accepted.** `READY_LIVE: none` remains unchanged; continue immediately to R9-11D.
