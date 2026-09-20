# ChatGPT reviewer handoff — I4 adapter / portability review queue

## Current repository truth

- Current reachable `main` immediately before this handoff update is exact `cd182ade1365f7aefb48b66c73cb49ed9989f175` (`docs(review): close I4-FS2 after H-I4-086/087 repairs`).
- **H-I4-086 is CLOSED.** Source repair exact `ca629d702cf832c12bf74bdfc5f9d07ebb77ab17` plus companion tests exact `3acba049fee5f9297cc8a4c3867250635c255717` enforce the outbound drain boundary: `delivery_ack` cannot confirm queued-but-undrained bytes, cannot cross a queued suffix, and cannot borrow another stream's drained boundary. Developer-reported clean exact-tree provenance is retained in `docs/notes/h-i4-086-provenance-3acba04-20260921.md`.
- **H-I4-087 is CLOSED.** Source repair begins at exact `14e2f520a0fc9d41bcfcb1bd480758008a1dd4ea`; dedicated regressions and source/test closure anchor exact `d819d38559d60b6bd99df8a2453321809c046116`. Both send and receive admission now enforce the one aggregate queued-record cap before mutation. Developer-reported clean exact-tree provenance is retained in `docs/notes/h-i4-087-provenance-d819d38-20260921.md` (`check.sh` exit 0 on the recorded rerun, `git diff --check` exit 0, clean tree, Linux x86_64, rustc 1.98.0 stable). No hosted status/workflow run was visible for exact `d819d38`; absence of hosted evidence is not a failure.
- **I4-FS2 is CLOSED as a bounded independent current-owner challenge** at review note `docs/reviews/independent-i4-fs2-session-flow-control-11cdb08-20260921.md`, anchored to reachable exact `11cdb08e7bc56e105dadb0092c7636a4ca346b3d` plus the H-I4-086/087 executable owner history. The pre-repair no-finding portions of `docs/reviews/dev-i4-fs2-multistream-flow-control-20260921.md` are superseded. No additional concrete source-decided defect was found in stream/session byte-window interaction, aggregate queue accounting, queued/drained/sent-unacked ownership, reject-before-mutate, duplicate/terminal behavior, cross-stream isolation, receive-window ownership, or Session-delivery versus Carrier-feedback separation.
- The known `SessionRuntime.events` retained-state capacity question remains the existing maintainer/security policy gate; do not invent a history-size/TTL/LRU/capacity value. Successful-ACK/duplicate idle-liveness refresh semantics were not source-decided by I4-FS2 and were not silently redefined.
- H-I4-085 and I4-FS1 remain closed at their reachable anchors. H-R9-080/081/082/083/084, Candidate A, Candidate B, R9-11A/B/C/D1/D2/D3/D4, R9-12, final independent R9, Q10/Q11/Q12 remain closed unless repository truth changes.
- Release items 3 and 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: exact-current review -> focused deterministic challenge/tests -> smallest repair if source-decided -> commit -> push -> clean exact-tree local gate/provenance for changed source/test/evidence trees -> next slice. Reviewer cadence is only a check frequency; do not wait for the next reviewer after a slice closes.

## READY_LOCAL 1 — I4-AD1 MemoryCarrier close/error/resource re-challenge

Challenge developer note `9288b83` against exact-current MemoryCarrier source/tests and applicable D011/current Carrier contract. Explicitly challenge:

- queued data accepted before close: whether the documented endpoint/peer close ordering preserves or discards it exactly as specified;
- local close idempotence and peer-visible close/error truth;
- queue byte/record accounting on send, receive, close and error paths;
- post-close send/receive false-success and mutation-before-error risks;
- concurrency/lock/poison/error behavior where the owner exposes it;
- logical `close()` semantics versus ordinary Rust ownership/drop semantics.

Use focused deterministic tests to falsify the current claims. Repair only a concrete source-decided defect; otherwise persist a bounded no-finding note naming inspected owners, commands/tests, exclusions and reachable anchor. No WAN work is implied.

## READY_LOCAL 2 — I4-AD2a UDP adapter close/error/resource re-challenge

Independently challenge the exact-current UDP adapter rather than treating UDP/TCP as one interchangeable abstraction. Cover connected/unconnected assumptions actually implemented, local logical close, OS socket lifetime/drop, send/recv after close, oversize/error paths, nonblocking behavior, listener/socket release and same-address rebind evidence where current local process tests exist. Distinguish adapter semantics from process-resource-sampler evidence. No new live run unless a genuinely new unresolved real-network question appears.

## READY_LOCAL 3 — I4-AD2b TCP adapter close/error/resource re-challenge

Challenge exact-current TCP framing/adapter ownership independently: logical close versus `shutdown`/EOF/object drop, buffered/pre-close data, partial/read framing errors, send/recv after terminal state, peer close ordering, accepted-stream/listener lifetime, FD release and local deterministic rebind/process tests. Do not infer Session delivery from TCP reliability and do not duplicate packet ACK semantics.

## READY_LOCAL 4 — I4-PORT cross-platform CLI/process-test semantics

Challenge developer note `11678b2` against current platform guards, Unix identity/permission checks, Linux `/proc`, signals/process groups/`setsid`, socket/process samplers, temp/runtime cleanup, and deterministic unsupported-platform behavior. Separate protocol portability from Linux-only release/process evidence. If a command is intentionally Linux-only, unsupported platforms must fail truthfully rather than appearing supported.

## READY_LOCAL 5 — I4-CLI-M machine-readable exit / JSON contract

Challenge success/failure/timeout/cleanup-unknown branches, typed/schema result alignment, stdout purity and process-exit consistency across current CLI/process owners. Include matrix probe, authenticated probe/failover/periodic fixture surfaces, process-resource sampler and validators where applicable. Typed failure or incomplete cleanup must never coexist with wrapper success unless that separation is explicitly documented. Do not silently redefine versioned JSON field meaning.

## READY_LOCAL 6 — I4-CLI-H human-output / stderr contract

Challenge human formatting/error routing independently from machine output: no positive success wording after terminal failure, no JSON/stdout contamination, no misleading cat/success line on typed failure, and stable exit semantics. Preserve the project’s fixed human reachability wording where applicable; avoid cosmetic churn if no defect exists.

## READY_LOCAL 7 — I4-BND algorithmic resource boundedness

Re-check current Recovery/Carrier/Session/CLI loops, maps, queues, retained/live state and peer-controlled iteration under existing limits. This is code-path boundedness review, not a capacity-pressure benchmark. Do not invent TTL/LRU/history/capacity/security values. `SessionRuntime.events` retained-state capacity and D019 source-retention/no-reset remain maintainer/security policy gates and must be classified rather than decided.

## READY_LOCAL 8 — item-4 factual reconciliation after the next coherent closure group

After MemoryCarrier + UDP adapter + TCP adapter + portability (or an equivalent 3–4 coherent slice group), reconcile item-4 coverage and release/evidence facts. Preserve valid no-finding notes, explicitly supersede false/stale evidence, update the release packet only where factual boundaries changed, and do not mark item 4 complete unless repository-wide independent-review truth supports it.

## READY_LOCAL 9 — repository-wide refill / uncovered current owners

Inventory all implemented core surfaces against reachable dedicated bounded review on the exact relevant owner tree. Refill materially changed or still-unreviewed owners. At minimum re-evaluate the 13-surface inventory in the reviewer contract; do not duplicate unchanged `neko-observe`, package/release owners or fresh R9 surfaces merely for count, but equally do not declare queue exhaustion while a current implemented core surface lacks a dedicated independent challenge.

## READY_LOCAL 10 — release-packet / evidence-boundary consistency spot-check

After the coherent adapter/portability group, compare `docs/release-security-review-packet.md`, `docs/status.md`, `IMPLEMENTATION_PLAN.md`, current review notes and exact reachable source owners. Challenge only factual consistency and provenance boundaries: developer-local versus reviewer-local versus hosted versus live evidence, superseded findings, release-item 3/4 state, and policy-gated exclusions. Do not turn this into prose churn or a release decision.

## READY_LOCAL 11 — conditional live question only

Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved self-owned real-network question, classify it against `docs/standing-vps-lab-authorization.md` and `docs/vps-rental-window-priority.md`. Ordinary bounded self-owned TCP/UDP work inside standing authorization needs no per-run approval. Do not manufacture live work merely because the VPS is rented.

## Surfaces deliberately not duplicated unless repository truth changes

- `neko-reliable` Recovery, `CarrierState`, Concurrent Carrier Manager/migration-back and the R9 SessionRuntime terminal/resource surfaces have fresh independent challenges on current/relevant owners; H-I4-086/H-I4-087 plus the new I4-FS2 note cover the distinct current flow-control/resource-accounting seam.
- `neko-observe` has no material post-review owner change; Candidate-B closure remains applicable.
- `scripts/release` has no material post-review owner change requiring synthetic repetition; process-resource sampler was separately repaired/reviewed through H-R9-082/083/084 and R9-11D.
- Release-packet factual/evidence boundaries were reconciled by Q10/Q11/Q12; revisit after this next coherent closure group or a material owner change, not after every tiny commit.

## Stop / escalation conditions

Continue review/repair -> tests -> commit -> push -> next dependency-ready slice without waiting for reviewer cadence. Stop/escalate only for an unresolved BLOCKER/HIGH that cannot be auto-decided, core Session/Carrier/ACK/crypto/wire architecture change, D019/policy-value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage. Normal bounded review/no-finding work and source-decided repairs are pre-authorized local work.
