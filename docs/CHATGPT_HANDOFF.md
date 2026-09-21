# ChatGPT reviewer handoff — post-reconciliation item-4 independent gaps

## Current repository truth

- Current reachable reviewer anchor before this handoff update is exact `45c2890029c4bf46855dc8ea4df987d0cc335e64` (`docs(review): inventory current item-4 independent coverage gaps`). The exact executable/source-test tree remains `42be53917ee58359ad86532a6aab0136f75047b9`; later commits through `45c2890` are review/evidence/handoff documentation only.
- The PORT/CLI/BND factual reconciliation landed at `588ca372827f7cbe00cdb9086cb0901ff7a11904`, but one reconciliation sentence incorrectly upgraded the current reachability gate into a typed parse. Reviewer repair `7197abf037988adf3a4ac097429dc047b5f52f33` corrects that factual claim: exact `matrix_probe` calls `reachability::run(...)` and then uses `artifact.contains("\"reachable\":true")` for both human wording and exit status. The current one-case `reachability-matrix.v1` shape has not been falsified as a concrete defect, but the stronger “parsed result” wording is superseded and a future schema/multi-case/alternate-`reachable` occurrence is a re-review trigger.
- Repository-wide 13-surface inventory is recorded at `docs/reviews/reviewer-item4-13-surface-inventory-7197abf-20260921.md` / reachable `45c2890`. It is a source/spec/review-note inventory; **no reviewer-local commands/tests are claimed**.
- **H-I4-086 is CLOSED** at executable source/test repair `ca629d702cf832c12bf74bdfc5f9d07ebb77ab17` + `3acba049fee5f9297cc8a4c3867250635c255717`: DeliveryAck cannot confirm queued-but-undrained bytes; terminal sent bookkeeping is released.
- **H-I4-087 is CLOSED** at executable repair/provenance anchor `d819d38559d60b6bd99df8a2453321809c046116`: both Session admission directions enforce the same aggregate `send.len() + recv.len()` record cap before mutation. Preserve the provenance fact that the first full gate run saw one startup/socket timing flake and the second full run passed.
- **H-I4-088 is CLOSED** at executable repair `26aa4e8036d61da7924d9fc5e587081d8a0f448f`: MemoryCarrier empty messages consume a finite record slot derived from the already-committed queue bound; receiving one record releases that slot. Current independent MemoryCarrier re-challenge is `docs/reviews/independent-i4-ad1-memorycarrier-rechallenge-d0f4c55-20260921.md`.
- **I4-FS2 is CLOSED as a bounded current independent challenge** at `docs/reviews/independent-i4-fs2-session-flow-control-11cdb08-20260921.md`; it explicitly supersedes the falsified portions of the earlier developer no-finding note after H-I4-086/087.
- **I4-PORT-01 executable repair is present** at `42be539`: `signal_term` and the affected SIGTERM/process-lifecycle/socket fixtures are explicitly `#[cfg(unix)]`. Developer-reported clean exact-tree provenance exists, but the independent follow-up at `98978ee` predates that repair and explicitly requested a post-repair closure. Therefore cross-platform/process-test surface 10 still has one genuine independent READY gap.
- **I4-CLI-M is CLOSED with bounded independent no finding** at `docs/reviews/independent-i4-cli-machine-1b1b8c8-20260921.md`; that note explicitly excludes human wording and says to continue to an independent I4-CLI-H challenge. The later I4-CLI-H note at `536d59a` is developer-authored, so human-output/stderr surface 11 still has one genuine independent READY gap.
- Current algorithmic boundedness is supported piecemeal by the developer I4-BND review (`89d249b`), reviewer Recovery completion (`031f7b8`), current independent Session FS2 and Memory AD1 reviews. A bounded cross-surface synthesis under **existing limits only** remains legitimate item-4 support; do not turn it into a capacity-pressure/adversarial benchmark and do not invent policy values.
- The release packet reconciliation row landed at `dd166b3`; standalone reconciliation note landed at `588ca` and was corrected at `7197abf`. A dedicated independent post-reconciliation packet/status/plan evidence-boundary spot-check remains READY.
- Candidate A/B, current CarrierState/CarrierManager, unchanged observability, unchanged package/operator owners, H-I4-085 dependency/build owner truth, and previously closed R9 seams remain closed unless exact-current owner truth changes or a new counterexample falsifies the claim.
- The known `SessionRuntime.events` retained-history capacity question remains a maintainer/security policy gate. Do not invent history-size/TTL/LRU/capacity values. D019 source-retention/no-reset remains policy-frozen.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm-failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD, or Experimental Track merely because a VPS remains rented. Only new code/instrumentation/hypothesis/path condition creating a concrete unresolved real-network question may reopen a live lane.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: exact-current independent challenge -> focused deterministic tests when applicable -> smallest repair if a source-decided defect is demonstrated -> commit -> push -> exact-tree local gate/provenance for executable changes -> next slice. Reviewer cadence is only a check frequency; do not wait for the next reviewer once a slice is closed.

## READY_LOCAL 1 — I4-PORT post-`42be539` independent closure

Read exact-current `crates/neko-cli/tests/probe.rs`, applicable portability/release claims, `docs/reviews/independent-i4-port-followup-98978ee-20260921.md`, executable repair `42be539`, and its developer-local provenance.

Challenge the repair contract independently:

- every test/helper that depends on Unix `kill`, signal semantics, process-group behavior, `/proc`, or Linux/POSIX process/socket-lifecycle assumptions must either be explicitly scoped to an appropriate platform or be supported by a real platform-aware implementation;
- Unix-only lifecycle evidence must not be promoted into non-Unix execution evidence or protocol portability;
- cfg scoping must not accidentally remove platform-independent machine/CLI coverage;
- current first-RC Linux target and release policy must not be broadened.

If no counterexample exists, write one scope-precise independent no-finding closure naming inspected owners, exact reachable anchor, source/tests read, commands actually run versus not run, and exclusions. If a concrete unguarded owner remains, make the smallest test/evidence repair and gate the final pushed executable SHA. No decoder/framing change => no mechanical fuzz run.

## READY_LOCAL 2 — I4-CLI-H independent human-output / stderr / exit challenge

This is not satisfied by the developer note at `536d59a` and is explicitly left open by `independent-i4-cli-machine-1b1b8c8-20260921.md`.

Read exact-current `matrix_probe`, `reachability::run`, CLI error/fail paths, relevant process tests, README/ROADMAP/D007 output claims, the developer I4-CLI-H note, and the corrected reconciliation at `7197abf`.

Challenge:

- `pass: 喵~！` versus `fail: 喵呜呜呜呜…` must agree with exit status and the one-case machine artifact;
- stderr/usage/config failures must not print a later human success line or contaminate machine stdout;
- do **not** describe the final reachability branch as typed parsing: exact-current truth is the literal substring gate `artifact.contains("\"reachable\":true")`;
- determine whether README/ROADMAP/D007 wording is a truthful description of the current emitted line format; if only prose is stale and current code/tests establish the intended current contract, make the smallest factual correction rather than inventing a new CLI format;
- a schema/multi-case or alternate `reachable` occurrence remains a future re-review trigger unless an exact-current counterexample exists now.

If no concrete defect is found, persist a bounded independent no-finding note; do not manufacture a source refactor solely to make “parsed result” prose true.

## READY_LOCAL 3 — I4-BND current independent synthesis under existing limits

Independently reconcile current resource boundedness across the owners most recently changed or re-reviewed:

- `SessionRuntime` queued/live/sent-unacked ownership after H-I4-086/087;
- MemoryCarrier byte + empty-record ownership after H-I4-088;
- current `neko-reliable::Recovery` packet/frame/ACK-range ownership after the `031f7b8` boundedness challenge;
- already-bounded Carrier health/evidence vectors and CLI loop counts only as needed to test cross-surface claims.

Challenge algorithmic growth and release-on-pop/ACK/terminal/quiesce semantics using existing committed limits. Do **not** run or request a capacity-pressure/adversarial-load benchmark and do not choose TTL/LRU/history-size/capacity/security values. `SessionRuntime.events` and D019 remain policy gates. If current independent notes already prove a sub-invariant, cite/reuse them rather than duplicating tests.

## READY_LOCAL 4 — post-reconciliation release packet / status / plan evidence-boundary spot-check

Independently compare exact-current:

- `docs/release-security-review-packet.md`;
- `docs/status.md`;
- `IMPLEMENTATION_PLAN.md`;
- `docs/notes/i4-reconciliation-port-cli-bnd-20260921.md` after `7197abf`;
- H-I4-085/086/087/088 and I4-PORT provenance notes;
- current independent FS2/AD1/CLI-M/Recovery reviews and the results of READY_LOCAL 1-3.

Challenge stale SHA/breadth statements, developer-local evidence being promoted to reviewer-local/hosted evidence, hosted absence being interpreted as success/failure, historical live negatives being rewritten as current success, item 3/4 being implied complete, and policy-blocked retained-state questions being turned into implemented limits. Repair only current factual/index text; do not rewrite historical artifacts.

## READY_LOCAL 5 — final item-4 technical sweep / refill

After READY_LOCAL 1-4, repeat the broad 13-surface inventory against the then-current owners. Re-open only a materially changed owner, a falsified prior claim, or an implemented core surface still lacking a dedicated current bounded independent challenge. A no-finding bounded review is valid item-4 support. Do **not** call the queue exhausted from a narrow lane.

This sweep should explicitly verify that the current repairs/reviews have not created a cross-layer contradiction among Session delivery evidence, Carrier packet feedback, resource accounting, process/result truth, or package/release claims. Any concrete correctness/security/evidence defect returns to repair priority before further closure work.

## READY_LOCAL 6 — final item-4 security/release-boundary classification

If the technical sweep has no unresolved automatic repair, classify the remaining release-item-4 boundaries precisely without deciding them:

- D019/source-retention policy;
- `SessionRuntime.events` retained-history capacity choice;
- adversarial-load/capacity suitability if still outside bounded semantic review;
- cryptanalysis/independent security-audit exclusions;
- release item 3 environment/evidence dependencies;
- maintainer RC/freeze/release/production authority.

Do not convert any of these into invented values, and do not let them block independent READY local work that remains.

## READY_LOCAL 7 — final independent item-4 conclusion packet

Only after the inventory-driven gaps are closed, produce one concise current conclusion that names exact reachable anchors, evidence classes, unresolved policy/environment/authority gates, and whether item 4 is technically ready for a maintainer decision. This is a factual synthesis, not the RC/freeze/release decision itself. Avoid checker/schema/doc churn; if the exact-current packet/status already says the truth, a no-finding conclusion is sufficient.

## READY_LOCAL 8 — conditional live question only

Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved self-owned real-network question, classify it against `docs/standing-vps-lab-authorization.md` and `docs/vps-rental-window-priority.md`. Ordinary bounded self-owned TCP/UDP work inside standing authorization needs no per-run approval, but the authoritative current classification remains `READY_LIVE: none`.

## Conditional refill triggers — not immediate duplicate work

- Dependency/build/native: re-open only if Cargo manifests/lock/features/build hooks/native/unsafe owners change or H-I4-085 corrected truth is falsified. Do not alter dependency/default-feature/crypto selection merely to simplify evidence prose.
- Package/repro/operator: re-open only on a material helper/package owner change or a new provenance contradiction. Do not rerun identical VPS/package scenarios for freshness alone.
- Observability: re-open only on a material `neko-observe` owner change or a new counterexample. Candidate B remains closed.
- Process-resource sampler H-R9-082/083/084: do not restart without a new counterexample; I4-CLI-M already re-challenged result/cleanup truth.

## Stop / escalation conditions

Continue review/repair -> tests -> commit -> push -> next dependency-ready slice without waiting for reviewer cadence. Stop/escalate only for an unresolved BLOCKER/HIGH that cannot be auto-decided, core Session/Carrier/ACK/crypto/wire architecture change, D019 or another policy/value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
