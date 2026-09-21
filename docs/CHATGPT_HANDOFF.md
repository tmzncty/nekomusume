# ChatGPT reviewer handoff — I4-PORT process-test scope repair front

## Current repository truth

- Current reachable `main` immediately before this handoff update is exact `e9cc061249229e4236b3ba504dabab59fdc57ea1` (`docs(review): qualify I4-PORT process-test platform scope`).
- New developer-owned commits reviewed since the prior handoff are review/documentation commits, not implementation changes:
  - `5f24cbffcbe13386e4016059f8a8253a5fd1a434` — I4-AD2a UDP adapter bounded re-challenge, no finding;
  - `e8fbc65e1ccd8766e000d48054411d53fcf94555` — I4-AD2b TCP adapter bounded re-challenge, no finding;
  - `98978eec7ed02a46d1f7577334349eba3f630db3` — I4-PORT cross-platform CLI/process-test re-challenge, no finding as written.
- **I4-AD2a UDP is CLOSED** within D012/current-Carrier scope. The current owner keeps logical local close idempotent, rejects later send success, preserves `WouldBlock` versus I/O distinction, bounds oversize handling before send mutation, does not invent UDP peer-close semantics, and keeps Rust socket ownership/drop distinct from the logical closed flag. No new Session/ACK/WAN claim is accepted from this adapter review.
- **I4-AD2b TCP is CLOSED** within the current framing/adapter scope. The current owner preserves terminal post-close behavior, EOF/truncation framing truth, accepted-stream/listener lifetime separation, `shutdown(Both)` close semantics, and does not promote TCP reliability into Session delivery evidence. No framing/decoder code changed in the review-only commit.
- **I4-PORT needs one narrow READY_LOCAL test-scope repair/qualification before complete closure.** Independent follow-up `docs/reviews/independent-i4-port-followup-98978ee-20260921.md` found no production/runtime correctness HIGH, but `crates/neko-cli/tests/probe.rs::signal_term()` executes external `kill -TERM` and the SIGTERM/process-lifecycle tests calling it are not Unix/Linux cfg-gated. Linux `/proc` fixtures are also intentionally platform-specific. This does not change the first-RC Linux target and does not disprove the narrower non-Unix compile claim; it means the existing broad process-test no-finding must not be read as proving non-Unix execution of Linux/POSIX lifecycle fixtures.
- **H-I4-086 is CLOSED.** Exact `ca629d702cf832c12bf74bdfc5f9d07ebb77ab17` + `3acba049fee5f9297cc8a4c3867250635c255717` enforce outbound drain-before-DeliveryAck; developer-local provenance remains separately labelled.
- **H-I4-087 is CLOSED.** Dedicated regressions through `d819d38559d60b6bd99df8a2453321809c046116` enforce the aggregate Session queued-record cap; developer-local provenance remains separately labelled.
- **H-I4-088 is CLOSED** at exact `26aa4e8036d61da7924d9fc5e587081d8a0f448f`. `MemoryEndpoint::send` bounds queue records as well as bytes, including repeated empty messages; developer-reported clean exact-tree provenance remains `docs/notes/h-i4-088-provenance-26aa4e8-20260921.md` and is not reviewer-local or hosted evidence.
- **I4-FS2 and I4-AD1 remain CLOSED** by their current bounded independent review notes after the above repairs.
- The known `SessionRuntime.events` retained-state capacity question remains a maintainer/security policy gate. Do not invent history-size/TTL/LRU/capacity values.
- H-I4-085, I4-FS1, H-R9-080/081/082/083/084, Candidate A, Candidate B, R9-11A/B/C/D1/D2/D3/D4, R9-12, final independent R9, Q10/Q11/Q12 remain closed unless repository truth changes.
- Release items 3 and 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a new concrete unresolved real-network question.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: independent review/challenge -> focused deterministic tests when applicable -> smallest repair if a concrete source-decided defect is found -> commit -> push -> clean exact-tree gate/provenance -> next slice. Reviewer cadence is only a check frequency; do not wait for the next reviewer once a slice is closed.

## READY_LOCAL 1 — CLOSED: I4-PORT-01 process-test platform-scope repair

Read exact-current `crates/neko-cli/tests/probe.rs`, the current I4-PORT developer note, `docs/reviews/independent-i4-port-followup-98978ee-20260921.md`, release-target statements, and only the platform/process owners needed for this seam.

Closure contract:

- make the SIGTERM/process-lifecycle fixtures explicitly Unix/Linux-scoped using the narrowest applicable cfg **or**, only if already intended by current repository contract, use an existing platform-aware signaling mechanism;
- keep Linux `/proc`, process-group, `setsid`, external `kill`, and socket/process-release evidence labelled Linux/POSIX release evidence rather than protocol portability evidence;
- preserve current Linux deterministic cleanup assertions and first-RC target; do not expand platform support or release policy;
- do not turn a non-Unix unsupported/skipped fixture into a false pass;
- if executable test code changes, run the final pushed-SHA clean exact-tree gate/provenance (`PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree). No decoder/framing change is involved, so no mechanical fuzz run is required.

This is not a HIGH and is not a reason to stop after repair. Continue directly to I4-CLI-M.

## READY_LOCAL 2 — I4-CLI-M machine-readable exit / JSON contract

Challenge exact-current success/failure/timeout/cleanup-unknown branches, typed/schema result alignment, stdout purity and process-exit consistency across CLI/process owners. Include matrix probe, authenticated probe/failover/periodic fixtures, process-resource sampler and validators where applicable. Distinguish argument/usage failure that intentionally exits 2 with empty stdout from completed machine-readable outcomes. A typed terminal failure or incomplete cleanup must not coexist with wrapper success unless the current contract explicitly says so.

If a current semantic invariant fixes the correct answer and a focused deterministic test disproves it, perform the smallest repair and regression, then exact-tree gate/provenance. Otherwise persist a bounded no-finding note with exact owners/tests/exclusions.

## READY_LOCAL 3 — I4-CLI-H human-output / stderr contract

Challenge human formatting/error routing independently from machine output: no positive success wording after terminal failure, no JSON/stdout contamination from human diagnostics, no misleading cat/success line on typed failure, stable exit semantics, and preservation of fixed reachability wording. Avoid cosmetic churn if no defect exists.

## READY_LOCAL 4 — I4-BND algorithmic resource boundedness

Re-check exact-current Recovery/Carrier/Session/CLI loops, maps, queues, retained/live state and peer-controlled iteration under **existing** limits. Do not run capacity-pressure/adversarial-load benchmarks and do not invent policy values. `SessionRuntime.events` retained-state capacity and D019 source-retention/no-reset remain maintainer/security policy gates rather than implementation filler.

## READY_LOCAL 5 — item-4 factual reconciliation after PORT/CLI/BND closure group

After the narrow portability repair plus two or three coherent adjacent slices, reconcile item-4 coverage and release/evidence facts. Preserve valid no-finding notes, explicitly supersede false evidence, and do not mark item 4 complete unless repository-wide independent-review truth supports it. This is a factual reconciliation, not a release decision.

## READY_LOCAL 6 — repository-wide broad core-surface inventory/refill

Inventory all 13 required implemented core surfaces against reachable, dedicated, bounded independent review on the exact relevant owner tree. Re-open only surfaces whose owner changed materially or whose earlier review claim was falsified; add READY_LOCAL review lanes for any implemented core owner that still lacks a dedicated independent challenge. Do not declare queue exhaustion from a narrow sweep.

## READY_LOCAL 7 — release-packet / evidence-boundary consistency spot-check

Compare `docs/release-security-review-packet.md`, `docs/status.md`, `IMPLEMENTATION_PLAN.md`, current review/provenance notes and reachable owner commits. Challenge developer-local versus reviewer-local versus hosted versus live evidence, superseded findings, items 3/4 and policy gates. Do not convert missing hosted CI into failure and do not promote unpublished/local-only SHAs into shared exact-tree evidence.

## READY_LOCAL 8 — package/reproducibility/operator-script current-owner spot-check if inventory shows a coverage gap

Only if the inventory shows this surface lacks a sufficiently current dedicated challenge, inspect Cargo manifests/lock/features/build/native hooks plus package/operator scripts and local provenance helpers against current release claims. Reuse H-I4-085 truth about `snow`/`ring` build/native surface. Do not manufacture schema/checker/docs filler merely to create work.

## READY_LOCAL 9 — final independent item-4 sweep after current closure group

Once the above dependency-ready work is closed, perform one repository-wide independent challenge of remaining implemented core surfaces and evidence boundaries. A no-finding result is valid item-4 support only if scope/owners/commands/exclusions/exact reachable anchors are explicit. If a concrete defect appears, repair it before any item-4 completion claim.

## READY_LOCAL 10 — conditional live question only

Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved self-owned real-network question, classify it against `docs/standing-vps-lab-authorization.md` and VPS rental priority. Ordinary bounded self-owned TCP/UDP work inside standing authorization needs no per-run approval. Do not manufacture live work merely because the VPS is rented.

## Surfaces deliberately not duplicated unless repository truth changes

- `neko-reliable` Recovery, `CarrierState`, Concurrent Carrier Manager/migration-back and the R9 SessionRuntime terminal/resource surfaces have fresh independent challenges; H-I4-086/H-I4-087 plus the current FS2 note cover the distinct Session flow-control seam.
- MemoryCarrier is freshly closed after H-I4-088; UDP and TCP adapters are now separately re-challenged and closed at `5f24cbf`/`e8fbc65` within their stated scopes.
- `neko-observe` has no material post-review owner change; Candidate-B closure remains applicable.
- `scripts/release` has no known material post-review owner change requiring synthetic repetition; process-resource sampler was separately repaired/reviewed through H-R9-082/083/084 and R9-11D.
- Release packet boundaries were reconciled by Q10/Q11/Q12; revisit after the current coherent closure group or material owner change, not after each small commit.

## Stop / escalation conditions

Continue review/repair -> tests -> commit -> push -> next dependency-ready slice without waiting for reviewer cadence. Stop/escalate only for an unresolved BLOCKER/HIGH that cannot be auto-decided, core Session/Carrier/ACK/crypto/wire architecture change, D019/policy-value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
