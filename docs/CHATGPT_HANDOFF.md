# ChatGPT reviewer handoff — I4-CLI-H human-output/stderr front

## Current repository truth

- Current reachable review anchor immediately before this handoff update is exact `882605a11830b7e1adff1356f903c51ff8ed2f75` (`docs(review): close I4-CLI-M machine contract`), whose inspected executable-owner tree is exact `1b1b8c8ff5e158e1b63bba99a1969da1069c700d` plus review-only documentation.
- **I4-PORT-01 is CLOSED** at executable test repair `42be53917ee58359ad86532a6aab0136f75047b9`; `1b1b8c8ff5e158e1b63bba99a1969da1069c700d` records developer-reported focused test + clean exact-tree `scripts/check.sh` / `git diff --check` provenance. The five POSIX signal/process-lifecycle tests and `signal_term` are explicitly `#[cfg(unix)]`; Linux `/proc` evidence remains Linux/POSIX evidence rather than a cross-platform runtime claim.
- **I4-CLI-M is CLOSED with bounded no finding** at `docs/reviews/independent-i4-cli-machine-1b1b8c8-20260921.md` / reachable commit `882605a`. The review challenged matrix probe JSON/exit truth, authenticated probe/lab outcome separation, periodic wrapper terminal status, repeated warm-failover batch status, and process-resource sampler/validator cleanup truth. It does not claim reviewer-local execution, hosted CI, WAN evidence, human-output review, or release approval.
- The current one-case matrix gate remains schema-coupled (`artifact.contains("\"reachable\":true")`), but exact-current `reachability-matrix.v1` has one case and one semantic `reachable` field; treat a future schema/multi-case change as a re-review trigger, not a current defect.
- H-I4-086, H-I4-087, H-I4-088, H-I4-085, I4-FS1/FS2, I4-AD1, I4-AD2a/AD2b, H-R9-080/081/082/083/084, Candidate A/B, R9-11A/B/C/D1/D2/D3/D4, R9-12, final independent R9, Q10/Q11/Q12 remain closed unless repository truth changes.
- The known `SessionRuntime.events` retained-state capacity question remains a maintainer/security policy gate. Do not invent history-size/TTL/LRU/capacity values. D019 source-retention/no-reset remains policy-frozen.
- Release items 3 and 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a new concrete unresolved real-network question.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: independent review/challenge -> focused deterministic tests when applicable -> smallest repair if a concrete source-decided defect is found -> commit -> push -> clean exact-tree gate/provenance -> next slice. Reviewer cadence is only a check frequency; do not wait for the next reviewer once a slice is closed.

## READY_LOCAL 1 — I4-CLI-H human-output / stderr contract

Read exact-current `crates/neko-cli/src/main.rs`, `crates/neko-cli/tests/probe.rs`, the freshly closed I4-CLI-M note, README/status wording, and only the process/script owners whose human/stderr surface is actually exposed.

Challenge independently from machine-output semantics:

- no positive `pass` / `ok` / success wording after a typed terminal failure or nonzero wrapper outcome;
- fixed reachability markers remain `喵~！` for human success and `喵呜呜呜呜…` for human failure only where that public wording is actually promised;
- stderr/error paths do not contaminate single-document machine stdout, and ordinary human output does not masquerade as structured evidence;
- signal/timeout/cleanup failure summaries remain truthful and do not print a later success line merely because an earlier stage succeeded;
- unsupported/usage failures keep their existing fail-closed exit semantics; do not broaden `--json` or invent a new CLI format as review filler.

If a current committed invariant determines the answer and a focused deterministic test disproves it, make the smallest repair + positive/negative regression, then run final pushed-SHA clean exact-tree gate/provenance. Otherwise persist a scope-precise no-finding note with owners, tests read/run, exclusions and exact reachable anchor, then continue immediately to I4-BND.

## READY_LOCAL 2 — I4-BND algorithmic resource boundedness

Re-check exact-current Recovery/Carrier/Session/CLI loops, maps, queues, retained/live state and peer-controlled iteration under **existing** limits. Do not run capacity-pressure/adversarial-load benchmarks and do not invent policy values. Explicitly separate implementation-bounded live/queued state from `SessionRuntime.events` retained-history capacity and D019, which remain maintainer/security policy gates.

Prefer mutation-sensitive source/test challenges of existing caps/iteration termination over new checker/schema/framework work. Any concrete source-decided defect gets the smallest repair/regression + exact-tree gate/provenance; otherwise persist a bounded no-finding note and continue.

## READY_LOCAL 3 — item-4 factual reconciliation after PORT/CLI/BND closure group

After I4-CLI-H and I4-BND, reconcile item-4 coverage and release/evidence facts across current review notes, executable repair anchors and provenance. Preserve valid no-finding notes; explicitly supersede false or stale claims; keep developer-local, reviewer-local, hosted, live and performance evidence distinct. Do not mark item 4 complete unless repository-wide independent-review truth supports it. This is factual reconciliation, not a release decision.

## READY_LOCAL 4 — repository-wide broad core-surface inventory/refill

Inventory all 13 required implemented core surfaces against reachable, dedicated, bounded independent review on the exact relevant owner tree:

1. reliable UDP recovery;
2. `CarrierState`;
3. Concurrent Carrier Manager / health / migration-back;
4. FairScheduler / multi-stream / flow-control accounting;
5. Memory/UDP/TCP adapters;
6. `SessionRuntime` lifecycle/resource/window/DeliveryAck;
7. observability projection/counters/high-water;
8. package/repro/operator scripts;
9. dependency/build/native/unsafe surface;
10. cross-platform CLI/process tests;
11. CLI exit/JSON/human contracts;
12. algorithmic resource boundedness;
13. release-packet factual/evidence boundary.

Re-open only surfaces whose owner changed materially or whose earlier review claim was falsified; add READY_LOCAL lanes for any implemented core owner still lacking a dedicated challenge. Queue exhaustion is repository-wide, never the result of one narrow no-finding sweep.

## READY_LOCAL 5 — release-packet / evidence-boundary consistency spot-check

Compare `docs/release-security-review-packet.md`, `docs/status.md`, `IMPLEMENTATION_PLAN.md`, current review/provenance notes and reachable owner commits. Challenge developer-local versus reviewer-local versus hosted versus live evidence, superseded findings, items 3/4 and policy gates. Do not convert missing hosted CI into failure and do not promote unpublished/local-only SHAs into shared exact-tree evidence.

## READY_LOCAL 6 — package/reproducibility/operator-script current-owner spot-check if inventory shows a coverage gap

Only if the inventory shows this surface lacks a sufficiently current dedicated challenge, inspect Cargo manifests/lock/features/build/native hooks plus package/operator scripts and local provenance helpers against current release claims. Reuse H-I4-085 truth about `snow`/`ring` build/native surface. Do not manufacture schema/checker/docs filler merely to create work.

## READY_LOCAL 7 — final independent item-4 sweep after current closure group

Once the above dependency-ready work is closed/refilled, perform one repository-wide independent challenge of remaining implemented core surfaces and evidence boundaries. A no-finding result is valid item-4 support only if scope/owners/commands/exclusions/exact reachable anchors are explicit. If a concrete defect appears, repair it before any item-4 completion claim.

## READY_LOCAL 8 — conditional live question only

Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved self-owned real-network question, classify it against `docs/standing-vps-lab-authorization.md` and VPS rental priority. Ordinary bounded self-owned TCP/UDP work inside standing authorization needs no per-run approval. Do not manufacture live work merely because the VPS is rented.

## Surfaces deliberately not duplicated unless repository truth changes

- `neko-reliable` Recovery, `CarrierState`, Concurrent Carrier Manager/migration-back and the R9 SessionRuntime terminal/resource surfaces have fresh independent challenges; H-I4-086/H-I4-087 plus the current FS2 note cover the distinct Session flow-control seam.
- MemoryCarrier is freshly closed after H-I4-088; UDP and TCP adapters are separately re-challenged and closed at `5f24cbf`/`e8fbc65` within their stated scopes.
- `neko-observe` has no material post-review owner change; Candidate-B closure remains applicable.
- `scripts/release` has no known material post-review owner change requiring synthetic repetition; process-resource sampler was separately repaired/reviewed through H-R9-082/083/084 and the current I4-CLI-M challenge.
- Release-packet boundaries were previously reconciled by Q10/Q11/Q12; revisit after this coherent PORT/CLI/BND closure group or a material owner change, not after each small commit.

## Stop / escalation conditions

Continue review/repair -> tests -> commit -> push -> next dependency-ready slice without waiting for reviewer cadence. Stop/escalate only for an unresolved BLOCKER/HIGH that cannot be auto-decided, core Session/Carrier/ACK/crypto/wire architecture change, D019/policy-value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
