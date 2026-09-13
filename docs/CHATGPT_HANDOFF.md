# ChatGPT reviewer handoff — candidate/measurement sweep accepted; repository-wide local queue exhausted

## Reviewed repository truth

- Default branch before this refresh: exact `b6fac41c782f303e4ef98ec5a076ba41816d9381` (`docs(review): corrected item-4 core inventory including candidate/measurement lanes`).
- Previous handoff content was stale: its benchmark P95 repair and PLPMTUD/FEC/disabled-gate review lanes have all been completed.
- The only new source/test change after the prior deep-sweep handoff is reachable exact `757e0b6c056f0bac39c1fb00e24d54cab55f3f72` (`fix(bench): align P95 order statistic with repository netns convention`). Later commits through `b6fac41` are review/evidence/index documentation only.
- Exact `757e0b6` has developer-local clean exact-tree provenance in `docs/local-gate-757e0b6-20260913.md`: `cargo test -p neko-bench`, `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and clean initial/final tree all passed. GitHub-hosted `stable checks` and `nightly decode fuzz smoke` also passed and remain extra cross-evidence only.
- No VPS/WAN experiment occurred in this sequence. Open pull requests: none.
- Governance remains unchanged: item 3 incomplete, item 4 incomplete, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, D019 policy-blocked, `SessionRuntime.events` remains `POLICY_BLOCKED_RESOURCE_BOUND`, and `READY_LIVE: none`.

## Reviewer verdict on new work

### Benchmark P95 repair `757e0b6` — ACCEPT

The prior measurement-consistency MEDIUM is closed.

`neko-bench::stat` now uses the same repository order-statistic convention as `scripts/bench/run-netns.sh`: `p95 = xs[round((n - 1) * 0.95)]` on sorted successful samples. Deterministic tests pin `n = 1, 2, 20, 100, 1000`; failed operations remain excluded from latency distributions while being counted separately; all-failure/empty distributions remain non-panicking. `docs/era4-i-performance.md` now states the convention.

The independent re-close `docs/reviews/independent-bench-reclose-757e0b6-20260913.md` also rechecked iteration/sample accounting, failure exclusion, crypto sample setup/nonce progression, units/timed boundaries, and the no-HY2-superiority claim. No remaining defect in that bounded scope.

### PLPMTUD candidate review `76b14da` — ACCEPT

Dedicated independent bounded review found no defect in the socket-free candidate: config/base/max bounds, u16-edge search arithmetic, one-outstanding-probe and probe-budget ordering, checked probe-ID exhaustion, exact generation/id/size ACK binding, stale/duplicate/wrong-size atomic rejection, timeout/upper-bound reduction, generation reset, bounded blackhole fallback, and EMSGSIZE lowering semantics all match the committed candidate contract.

This does not create a live PMTUD path or a `READY_LIVE` question.

### Disabled XOR FEC candidate review `167f1db` — ACCEPT

Dedicated independent bounded review found no defect in config/block/symbol bounds before allocation, inclusive block-id identity space, byte-exact XOR parity/single-loss recovery, multi-loss fail-closed behavior, duplicate/out-of-range missing-symbol handling, reorder independence, or evidence-domain separation. FEC remains disabled; no enablement/performance claim follows.

### Disabled-gate enforcement review `6e04684` — ACCEPT

Current executable/config/CLI surfaces expose no 0-RTT/early application-data path and no concurrent heterogeneous UDP+TCP application striping path. Warm/standby readiness/control traffic remains distinct from application striping. This is enforcement verification only; disabled features remain disabled.

### Release indexing and corrected inventory — ACCEPT_WITH_BOUNDARIES

Exact `4e21520` correctly adds the benchmark/candidate lanes to `docs/release-security-review-packet.md` without changing release flags or claiming security approval.

Exact `b6fac41` supersedes the earlier premature `e10b6f` exhaustion inventory. The corrected inventory now includes PLPMTUD, FEC, benchmark P95 closure and disabled-gate enforcement.

I independently reverse-checked the inventory against:

- all eight workspace crates (`neko-bench`, `neko-carrier`, `neko-cli`, `neko-crypto`, `neko-observe`, `neko-reliable`, `neko-session`, `neko-wire`);
- current `docs/status.md` candidate rows, including reliable UDP, unreliable datagram, PLPMTUD, FEC, manager, 0-RTT and concurrent-multipath gates;
- release/package/dependency/CLI/validator/benchmark tooling already covered by named broader reviews;
- canonical corpus, HY2 methodology, pre-auth/resource-abuse and wire/parser independent review surfaces;
- current open PR set (none).

No additional implemented/candidate core surface lacking a dedicated bounded challenge was found.

## Repository-wide local queue status — genuinely exhausted for current semantics

This exhaustion decision is repository-wide, not the earlier narrow pre-auth proposal sweep.

There is currently:

- no unresolved mechanically repairable correctness/security/evidence defect;
- no implemented/candidate core surface lacking a dedicated independent bounded challenge;
- no READY review-support lane left by the corrected inventory;
- no open PR requiring review/merge;
- no dependency-ready live question (`READY_LIVE: none`).

The external coding agent should synchronize to current `main` and remain idle **unless one of the resume triggers below occurs**. Do not manufacture another checker/schema/framework/docs sweep solely to keep the agent busy, and do not re-review unchanged owners for volume.

## Remaining real gates / stop conditions

### 1. `SessionRuntime.events` retained-state bound — `POLICY_BLOCKED_RESOURCE_BOUND`

`SECURITY.md` requires per-connection/global memory, CPU and rate bounds. `SessionRuntime.events: Vec<RuntimeEvent>` is the one known retained collection without a hard lifetime bound.

This is not silently repairable from current semantics because the repository does not decide what happens when event retention reaches a limit: evict-oldest with explicit drop evidence, external drain/consumer plus fallback cap, hard logging refusal, or another retention contract. Reusing an unrelated queue constant would still invent retention semantics. Do not choose a numeric cap, TTL/LRU, or drop policy autonomously.

A maintainer/security decision on retention semantics makes this local slice READY immediately; then implement the smallest bounded design with deterministic overflow/drop tests and exact-tree provenance.

### 2. D019 source-retention / no-reset policy

Still a maintainer policy/value decision. Do not invent TTL, LRU/history-size, external-retention authority, or weakened no-reset semantics.

### 3. RSEC-001 representative adversarial-load / capacity suitability

Engineering controls are independently bounded-reviewed, but representative pressure/capacity suitability remains unestablished. Selecting meaningful load/concurrency/capacity conditions is a maintainer/security judgment and may exceed ordinary standing authorization when the purpose becomes pressure/capacity testing. Do not create a pseudo-capacity result from ordinary bounded unit/process tests.

### 4. Item 3 / environment / frozen live lines

Item 3 remains incomplete. Current repository truth still has no `READY_LIVE` row: natural-loss/long-lived/HY2/IPv6 and related current lines are blocked/frozen or already sufficient only for narrower bounded questions. Standing authorization remains valid, but authorization alone does not create a new scientific question.

Do not mechanically rerun HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint migration, key update, IPv6, PLPMTUD or Experimental Track work. A materially new implementation/instrumentation/configuration/hypothesis/path condition may create a new READY live row later.

### 5. Release-policy / authority gates

Signing, key custody, SBOM/publication trust, previous-frozen-release interoperability, final independent security/release judgment, RC, protocol/release freeze, release and production readiness remain separate decisions/dependencies. No local review result changes them automatically.

## Item-4 status

The bounded **local independent-review support inventory is now exhausted and internally reconciled**, but item 4 itself remains unchecked because its own stated scope includes resource/abuse-limit judgment and independent release/security judgment, while `SessionRuntime.events`, D019 and RSEC-001/final judgment remain open.

Do not mark item 4 complete merely because every current executable core surface has been bounded-reviewed.

## Resume triggers

Resume continuous execution immediately if repository truth gains any of:

1. a new developer source/test commit or open PR;
2. a concrete regression/failure contradicting an accepted review;
3. a new implemented/candidate core surface not covered by the corrected inventory;
4. a maintainer decision resolving `SessionRuntime.events` retention semantics or D019;
5. maintainer-selected RSEC-001 pressure/capacity conditions within authorized scope;
6. a materially changed live hypothesis/instrumentation/path/environment creating a genuine `READY_LIVE` row;
7. a release-policy decision that turns signing/SBOM/frozen-release interoperability into executable work.

On a new mechanically determined defect: smallest repair -> bounded regression -> commit/push -> clean exact pushed-tree `scripts/check.sh` + `git diff --check` + provenance -> continue through all dependency-safe follow-ups without waiting for reviewer cadence.

## Live / release boundary

- item 3 incomplete;
- item 4 incomplete;
- `RELEASE_CANDIDATE=false`;
- `PRODUCTION_READY=false`;
- `FREEZE=false`;
- `RELEASED=false`;
- D019 remains policy-blocked;
- `SessionRuntime.events` remains `POLICY_BLOCKED_RESOURCE_BOUND`;
- `READY_LIVE: none` remains authoritative.

Standing VPS authorization remains valid and unchanged. Current lack of live work is a question/dependency classification, not a missing-permission problem.
