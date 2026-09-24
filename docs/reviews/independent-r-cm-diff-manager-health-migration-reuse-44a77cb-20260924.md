# R-CM-DIFF — concurrent Carrier Manager / health / migration-back owner-diff reuse

**Exact current repository anchor:** `44a77cb7f4abc124606ba77e56edc74cacccf395`

**Prior independent anchors reused:**

- `a14cf471b6ef97442f36419ee8df3648037a5ef4` — bounded `CarrierManager` / health / migration-back review after switch-margin overflow repair;
- `5e73aead6d16506a6b299a459928b9c001022405` (source/test anchor `d97a536414a78ff44ef80b1a6d428ddf96ae6e22`) — integrated resolved-outcome -> health -> warm promotion / switch-ordering review;
- `a2a1e6cf6b2890a836cc9d6271061897730585cb` — retained replay / Carrier-generation terminality review.

## Diff challenge

I re-read the exact-current carrier owner and the post-`a2a1e6cf` path history before reusing those results. The material `crates/neko-carrier/src/lib.rs` changes after that review family are in other bounded owners: MemoryCarrier empty-record accounting and the reliable-UDP Recovery/Reno/PTO/persistent-congestion seams challenged by H-I4-116/H-I4-117/R-REC-4. The current commit history/search does not show a later semantic rewrite of `ConcurrentCarrierManager`, its Warm-only activation/single-active assignment contract, `CarrierHealth` decision rules, D064 readiness tuple checks, or migration-back validation/generation/health-margin/hold gates.

The exact-current source remains consistent with the earlier challenged claims:

- one active Session-data owner; warm candidates do not independently own new application data;
- readiness remains separate from packet health and Session delivery evidence;
- fail/promotion errors are explicit rather than converted into successful fallback;
- stale/mismatched generation/session/readiness observations cannot authorize promotion;
- migration-back still requires independent validation plus committed generation, health-margin and hold gates;
- terminal runtime gating prevents post-teardown readiness/activation mutation;
- retained uncertain/replay identity remains Session-owned rather than erased by Carrier teardown.

No current moved owner invalidates those prior bounded no-finding conclusions, and no concrete new manager/health/migration-back defect was found by the diff challenge.

## Evidence boundary

This is an owner-diff/reuse review, not a new execution record. I did not run reviewer-local Rust/full-gate commands and do not reclassify the prior developer-local or hosted evidence. The post-`a2a1e6cf` reliable-UDP changes are **not** declared covered by this note; they were separately challenged by R-REC and H-I4-116/117/118.

No new live-network question is created. `READY_LIVE: none`, release items 3/4, and all release/governance flags remain unchanged.

## Next dependency-ready lane

After three coherent slices (R-CS-1, R-CS-2, R-CM-DIFF), perform the scheduled **release packet / item-4 factual reconciliation** before consuming more reused lanes. Then continue package/build owner-diff reuse, observability/adapters, FairScheduler/Session, CLI/boundedness, recovery repaired-owner refill, and repository-wide 13-surface refill.