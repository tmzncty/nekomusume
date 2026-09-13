# ChatGPT reviewer handoff — close benchmark measurement semantics, then finish remaining independent candidate inventory

## Reviewed repository truth

- Current default branch before this reviewer refresh: exact `335246340bb764e458ca89c75ec92a35ed38fc6e` (`docs(release): index second-half 2026-09-13 deep item-4 sweep`).
- Previous handoff `ef1338b9965ce9f640531a10d02604c98b88f017` is stale and is superseded by this queue.
- The sequence after `ef1338b` closed the prior SessionRuntime evidence-integrity HIGH, then completed dedicated independent bounded reviews for DeliveryLedger, ProcessMessage/Resume/readiness codec, DatagramRuntime, crypto integration/API, CLI command truth boundaries, and static resource/result-validator surfaces.
- The only source/test change in that sequence is reachable exact `01b24a07ef81e9427d14f953fbf04b239b4015ce` (`fix(bench): exclude failed ops from median/P95 latency distribution`). It correctly stops failed operations from depressing successful-sample latency statistics. GitHub-hosted `stable checks` and `nightly decode fuzz smoke` are green for exact `01b24a0`; hosted CI is extra cross-evidence only.
- Exact `3352463` now indexes the second-half deep sweep in `docs/release-security-review-packet.md`.
- No VPS/WAN experiment occurred. Governance remains unchanged: item 3 incomplete, item 4 incomplete, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, D019 policy-blocked, `SessionRuntime.events` remains `POLICY_BLOCKED_RESOURCE_BOUND`, and `READY_LIVE: none`.

## Accepted progress — preserve unless owner source changes

The following deep independent surfaces are now accepted as bounded item-4 support and must not be mechanically re-reviewed:

- reliable UDP recovery / future and never-sent ACK rejection;
- CarrierState generation/validation/hysteresis;
- CarrierManager / migration-back / margin-domain repairs;
- FairScheduler / flow-control accounting;
- observability projection/ring/drop/schema closure;
- carrier adapters;
- SessionRuntime lifecycle/resource/terminal-cleanup, with the historical false cleanup sentence explicitly superseded and exact `9697ee7` developer-local provenance recorded;
- package/reproducibility/release scripts;
- dependency/build surface;
- CLI portability/output and command-specific truth boundaries;
- DeliveryLedger;
- ProcessMessage / Resume / readiness codec;
- DatagramRuntime;
- crypto integration/API (not cryptanalysis);
- static algorithmic retained-state boundedness + result-validator cross-sweep.

These are not a security audit or release approval. The known `SessionRuntime.events` retention policy gap remains open and policy-blocked; do not invent its cap/TTL/ring/drop semantics.

## MEDIUM — `neko-bench` P95 convention is inconsistent with existing repository benchmark semantics

Primary owner: `crates/neko-bench/src/main.rs::stat`, with `docs/era4-i-performance.md` and `scripts/bench/run-netns.sh` as current evidence/measurement contracts.

Exact `01b24a0` correctly changed latency distributions to successful samples only. However, current Rust `stat()` computes:

```rust
p95: xs.get(n.saturating_mul(95) / 100)
```

while the existing netns benchmark computes P95 from sorted non-null samples using:

```text
round((length - 1) * 0.95)
```

For 100 successful samples this selects different order statistics (Rust index 95 versus netns index 94); for 1000 it is 950 versus 949. The repository does not currently document a deliberate reason for two P95 conventions. This is measurement/evidence consistency, not performance tuning.

### MUST_EXECUTE_LOCAL 1 — prove and repair percentile semantics

1. Add focused deterministic tests around several sample counts (include at least 1, 2, 20, 100, 1000) that make the selected order statistic unambiguous.
2. Preserve the already-correct exact-`01b24a0` behavior:
   - failed operations increment `failures`;
   - failed timings never enter median/P95;
   - empty successful distribution remains non-panicking and reports the documented zero/null-equivalent local convention;
   - per-operation `iterations` remains the successful sample count unless an existing committed schema contradicts it.
3. Align `neko-bench` P95 with the already-existing repository convention used by `run-netns.sh`, unless a stronger current committed benchmark specification proves a different convention. Do not invent a third percentile definition.
4. Keep median semantics consistent with current repository convention; do not broaden this into a statistics framework.
5. Update `docs/era4-i-performance.md` only as needed to state the chosen order-statistic convention precisely enough to reproduce it.

This is bench-only measurement code. No decoder/crypto framing changes are expected, so no fuzz rerun is required solely for this repair.

### MUST_EXECUTE_LOCAL 2 — exact-tree closure for final benchmark source

After the percentile source shape is final, commit/push it and validate that exact pushed source SHA in a clean checkout/worktree:

```bash
cargo test -p neko-bench
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --short
```

Persist sanitized developer-local provenance with exact reachable SHA, commands, UTC start/end, exits, OS/arch, stable Rust version, and initial/final clean-tree state. Hosted CI may be cited separately if available; do not substitute it for the local anchor.

If investigation proves current Rust P95 is already the explicitly committed intended convention and `run-netns.sh` is the inconsistent owner instead, repair the inconsistent owner rather than forcing Rust to match. The invariant is one reproducible repository convention, not a predetermined implementation.

### REVIEW_LOCAL 3 — independent benchmark re-close on final source

On the final reachable source anchor, independently challenge `neko-bench` evidence semantics:

- requested iteration clamp versus successful sample count;
- median/P95 exact order-statistic behavior for small/large `n`;
- failure exclusion and empty/all-failure behavior;
- crypto setup/nonce use across repeated samples;
- units/names match actual timed operation boundaries;
- scheduler/recovery microbench rows do not imply end-to-end/WAN behavior;
- output cannot be read as HY2 superiority evidence.

Concrete defect -> smallest repair/regression/local gate. No finding -> one dedicated bounded no-finding note.

## Remaining genuine independent candidate lanes

The broad inventory now covers nearly all release-relevant core crates. The following implemented candidates still have executable state but no dedicated independent bounded review visible in the accepted review index. Continue without reviewer wait after benchmark closure.

### REVIEW_LOCAL 4 — PLPMTUD candidate state model

Primary owner: `neko-reliable::Plpmtud` and `docs/spec/m2-plpmtud.md`.

Challenge only the current socket-free candidate semantics:

- base/max MTU and configuration validation;
- binary-search arithmetic near `u16` edges;
- one outstanding probe and total probe-limit ordering;
- checked probe-ID exhaustion and atomic failure;
- ACK binding to exact generation/id/size;
- stale/duplicate/wrong-size rejection without mutation;
- timeout/retry/upper-bound reduction;
- generation reset discards stale evidence;
- blackhole fallback cannot become path-failure evidence;
- `on_emsgsize` cannot raise confirmed MTU or fabricate authenticated probe evidence.

Do not connect PLPMTUD to live sockets/ICMP or create a live experiment from this review. Concrete existing-semantics defect -> smallest repair/regression. No finding -> dedicated bounded review note.

### REVIEW_LOCAL 5 — disabled XOR FEC candidate

Primary owner: `FecConfig` / `FecBlock` in `neko-reliable` and `docs/spec/m2-fec.md`.

FEC remains disabled; this lane does **not** enable it. Challenge:

- block/symbol/config bounds before allocation/copy;
- block-id/max-block semantics against the committed candidate spec/tests;
- parity/recovery byte exactness;
- single-loss recovery only, multi-loss fail-closed;
- duplicate/out-of-range missing-symbol handling;
- reorder independence where claimed;
- no FEC result becomes ACK, Session delivery, congestion, or enablement evidence.

Do not add adaptive FEC, performance tuning or enablement policy.

### REVIEW_LOCAL 6 — disabled-gate enforcement (`0-RTT`, concurrent heterogeneous multipath)

Review the executable/config/CLI surfaces against the already-committed disabled gates:

- no early application data / 0-RTT path is accidentally reachable;
- no ordinary config/CLI path silently enables concurrent UDP+TCP application striping or heterogeneous aggregation;
- warm/standby readiness/control traffic is not misclassified as application striping;
- disabled-gate docs/status claims match executable capability discovery/output.

This is enforcement verification, not a proposal to implement the disabled features. If there is no executable enable path, record bounded no-finding support and stop there.

### REVIEW_SUPPORT 7 — post-benchmark/candidate reconciliation

After benchmark closure plus roughly two of REVIEW_LOCAL 4–6, update the release packet/subgate index once with reachable anchors and precise exclusions. The second-half `3352463` reconciliation already indexes DeliveryLedger/codec/Datagram/crypto/CLI/boundedness; do not duplicate that work.

Only add the final benchmark repair/review/provenance and genuinely new candidate reviews.

### REVIEW_LOCAL 8 — repository-wide independent-review inventory

Before any future `queue exhausted`, compare every current implemented/candidate crate/tool/status row against the independent-review index and classify each as:

- dedicated independent bounded challenge complete;
- legitimately subsumed by a named broader review;
- disabled/policy/environment-only and no executable review question;
- still missing a concrete independent challenge.

Do not invent review work after all executable surfaces are actually covered. If the inventory shows no remaining concrete local review/repair, no new live question, and all remaining gates are policy/environment/external authority, then queue exhaustion may finally be recorded truthfully.

## Policy/external gates that remain outside autonomous repair

Do not let these block independent work above, but do not invent answers:

- `SessionRuntime.events` retention bound: `POLICY_BLOCKED_RESOURCE_BOUND`;
- D019 source-retention/no-reset policy;
- RSEC-001 representative adversarial-load/capacity suitability and maintainer-selected pressure conditions;
- item 3 natural-loss/long-lived/HY2/IPv6/environment evidence where current lines are frozen or blocked;
- signing, key custody, SBOM/publication trust;
- previous-frozen-release interoperability dependency;
- independent final security/release judgment;
- RC/freeze/release/production authority.

## Live / release boundary

- item 3 incomplete;
- item 4 incomplete;
- `RELEASE_CANDIDATE=false`;
- `PRODUCTION_READY=false`;
- `FREEZE=false`;
- `RELEASED=false`;
- `READY_LIVE: none` remains authoritative.

Standing VPS authorization remains valid, but current repository truth creates no new dependency-ready live question. Do not mechanically rerun HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint migration, key update, IPv6, PLPMTUD or other Experimental Track work.

## Queue discipline

- MEDIUM benchmark measurement consistency is first; it does not require maintainer input.
- Continue REVIEW_LOCAL 3–6 sequentially or in non-conflicting independent lanes; source changes always move subsequent tested-tree anchors forward.
- Preserve completed reviews; do not re-review unchanged owners for volume.
- A bounded no-finding review of an actually unreviewed executable candidate is valid item-4 support.
- Stop only for unresolved BLOCKER/HIGH that contaminates downstream work, core architecture/policy choice, destructive/canonical migration, action outside standing authorization, production/third-party action, new credentials/server permission, maintainer-valued pressure/capacity conditions, repository breakage, or actual runtime/tool exhaustion.
