# ChatGPT reviewer handoff — current item-4 closure queue after `87b371d`

## Current repository truth

- Reviewer re-read current `main` after the prior handoff at `c8bc0f5c4b85dd53b586f7ae3a1a958dfab29912`. The developer advanced by 36 commits through `87b371d0d28b7a3df2c105770f1420e61eb4c349`, then reviewer finding `cb893259e1f54230ed78d8a080d58563c6bc18a3` was added. **Synchronize to current `main`; do not use the older handoff anchor as an idle reason.**
- The current executable/source-test tree is exact `42be53917ee58359ad86532a6aab0136f75047b9`. Later commits are review/evidence/reconciliation documentation only. No decoder/parser/crypto-framing owner changed in this closure group.
- **H-I4-086 remains CLOSED** at `ca629d702cf832c12bf74bdfc5f9d07ebb77ab17` + `3acba049fee5f9297cc8a4c3867250635c255717`: `DeliveryAck` cannot confirm queued-but-undrained bytes; sent/drained bookkeeping is terminal-owned and released.
- **H-I4-087 remains CLOSED** after exact-current source review. `queue_send()` and `receive()` both gate admission on aggregate `send.len() + recv.len()` before mutation; `pop_send()` / `pop_receive()` release queue ownership. The repair landed at `14e2f520a0fc9d41bcfcb1bd480758008a1dd4ea`; developer exact-tree provenance/closure is anchored by `d819d38559d60b6bd99df8a2453321809c046116`. Preserve the provenance fact that one startup/socket timing flake preceded a clean rerun; do not rewrite that as hosted/reviewer CI.
- **H-I4-088 remains CLOSED** after exact-current source review at executable repair `26aa4e8036d61da7924d9fc5e587081d8a0f448f`: MemoryCarrier empty messages consume a finite queue-record slot derived from the already committed queue bound and dequeue releases it. `docs/reviews/independent-i4-ad1-memorycarrier-rechallenge-d0f4c55-20260921.md` remains the current independent bounded Memory owner review. Do not invent a new record-cap policy value merely to make the type prettier.
- **I4-FS2 remains CLOSED** as a bounded current independent challenge at `docs/reviews/independent-i4-fs2-session-flow-control-11cdb08-20260921.md`, which supersedes the falsified pre-H-I4-086/087 developer claims.
- **I4-PORT-01 executable repair is present** at `42be539`: POSIX `kill` and the affected `/proc`/signal/process-lifecycle fixtures are explicitly `#[cfg(unix)]`. Developer post-repair review `5e9b4b22bbfe8d336026fee238cd5ae22c282f6e` is useful input, but reviewer still owes one exact-current independent acceptance/challenge rather than accepting its self-description mechanically.
- **I4-CLI-M remains CLOSED** at `docs/reviews/independent-i4-cli-machine-1b1b8c8-20260921.md`. Exact-current `matrix_probe` still uses the literal predicate `artifact.contains("\"reachable\":true")` for human branch and exit status; do not call this typed/schema parsing.
- Developer subsequently produced I4-CLI-H (`e3522d1`), I4-BND synthesis (`f61b201`) and final 13-surface sweep (`87b371d`) no-finding notes. These are review inputs, not automatic reviewer acceptance.
- **New current finding (non-HIGH): I4-CLI-H factual output-contract drift**, recorded at `docs/reviews/reviewer-i4-cli-h-output-contract-drift-20260921.md` / reachable `cb893259`. Current code and `ROADMAP.md` use complete human lines `pass: 喵~！` / `fail: 喵呜呜呜呜…`, while `README.md` says the human output is frozen but shows the bare cat strings, and `docs/carrier-architecture.md` / accepted D007 retain the older bare wording. Therefore the developer CLI-H “all prose matches emitted format” no-finding is not accepted yet. This is presently a factual/contract-doc inconsistency, not a demonstrated executable correctness defect.
- Consequently the `87b371d` statement that the repository is genuinely queue-exhausted is **not accepted yet**. Queue exhaustion must be re-evaluated only after the current CLI-H inconsistency and the remaining independent acceptance lanes below are closed.
- Candidate A/B, CarrierState/CarrierManager, unchanged observability, unchanged package/operator owners, H-I4-085 dependency/build truth, and the closed R9 process/result seams remain closed unless exact-current owner truth changes or a new counterexample falsifies a claim.
- `SessionRuntime.events` retained-history capacity remains a maintainer/security policy gate. Do not invent history-size/TTL/LRU/capacity values. D019 source-retention/no-reset remains policy-frozen.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track merely because the VPS is still rented. Reopen live only for a new concrete unresolved network question created by new code/instrumentation/hypothesis/path condition.

The external coding agent must continuously execute dependency-ready work: exact-current challenge -> focused deterministic tests when applicable -> smallest source-decided repair -> commit -> push -> clean exact-tree gate/provenance for executable changes -> next slice. Reviewer cadence is only a check frequency; do not wait for the next reviewer once a slice is closed.

## READY_LOCAL 1 — I4-CLI-H exact output-contract reconciliation + independent re-check

Read exact-current `crates/neko-cli/src/main.rs::matrix_probe`, relevant `probe.rs` tests, `README.md`, `ROADMAP.md`, D007 in `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/reviews/dev-i4-cli-h-independent-20260921.md`, and `docs/reviews/reviewer-i4-cli-h-output-contract-drift-20260921.md`.

Challenge and resolve only the exact-current contradiction:

- human success/failure and exit status must be driven by the same current one-case predicate;
- `--json` machine stdout must stay free of the human line; config/usage failures must not later print success;
- do not promote the literal substring gate into typed parsing;
- establish from current implementation/tests/reachable history whether the intended complete frozen human line includes `pass:` / `fail:`. If current executable/tests plus a reachable superseding contract decide that question, make the smallest factual prose correction; **do not refactor the CLI solely to make old prose true**;
- if reconciling D007 would actually change an accepted design/value choice rather than clarify stale wording, stop only that sub-question as a maintainer/design gate and continue independent lanes below.

After any docs-only correction, do not fabricate a source-test gate. If executable code changes, run the required clean exact-tree gate/provenance. No decoder/framing change => no mechanical fuzz run.

## READY_LOCAL 2 — I4-PORT post-`42be539` reviewer-independent acceptance

Use `42be539`, exact-current `crates/neko-cli/tests/probe.rs`, `docs/reviews/dev-i4-port-post-42be539-20260921.md`, the older independent portability review and release/platform claims.

Independently challenge that every Unix `kill` / signal / process-group / `/proc` dependent owner is appropriately scoped; Unix-only lifecycle evidence is not promoted to non-Unix execution evidence; and platform-independent CLI/process coverage was not accidentally cfg-elided. `std::process::Child::kill()` by itself is cross-platform and is not a reason to gate a test. If no counterexample exists, persist a scope-precise reviewer no-finding acceptance; if one exists, smallest repair + focused regression + exact-tree gate.

## READY_LOCAL 3 — I4-BND reviewer-independent synthesis under existing committed limits

Treat `docs/reviews/dev-i4-bnd-synthesis-20260921.md` as an input, not the conclusion. Reconcile exact-current:

- `SessionRuntime` queued/received/sent-unacked ownership after H-I4-086/087;
- MemoryCarrier payload-byte + empty-record ownership after H-I4-088;
- `neko-reliable::Recovery` / `AckRanges` ownership using the existing independent Recovery review;
- Carrier health/scheduler and bounded CLI loop owners only where needed for cross-surface claims.

Challenge growth and release-on-pop/ACK/terminal/quiesce semantics using **existing** limits only. Do not perform capacity-pressure/adversarial-load benchmarking and do not choose a new TTL/LRU/history-size/capacity/security value. `SessionRuntime.events` remains an explicit policy gate, not a false no-finding.

## READY_LOCAL 4 — post-reconciliation packet/status/plan evidence-boundary challenge

Independently compare exact-current `docs/release-security-review-packet.md`, `docs/status.md`, `IMPLEMENTATION_PLAN.md`, release reconciliation notes, H-I4-085/086/087/088 provenance, I4-PORT provenance, current independent FS2/AD1/CLI-M/Recovery reviews, and results of lanes 1-3.

Reject stale/broadened claims: developer-local evidence must not become reviewer-local or hosted evidence; absent hosted CI is neither success nor failure; historical live negatives are not current positive evidence; item 3/4 must remain open unless their actual gates close; policy-blocked retained-state questions must not be converted into implemented numerical limits. Repair only current factual/index text, never historical artifacts.

## READY_LOCAL 5 — repository-wide 13-surface item-4 sweep / refill

After lanes 1-4, repeat the broad 13-surface inventory against then-current owners. Reopen a surface only for a material owner change, a falsified prior claim, or an implemented core surface still lacking a dedicated bounded independent challenge. Explicitly check cross-layer consistency among Session delivery evidence, Carrier feedback, resource accounting, process/result truth, CLI contracts and release evidence.

Only if the broad inventory finds **no** unresolved automatic defect, no unreviewed core owner and no legitimate local review-support lane may it classify the local technical queue as exhausted. A narrow no-finding is never enough.

## READY_LOCAL 6 — item-4 security/release-boundary classification

If the technical sweep is clean, classify without deciding:

- D019/source-retention policy;
- `SessionRuntime.events` retained-history capacity choice;
- adversarial-load/capacity suitability outside bounded semantic review;
- cryptanalysis / independent security-audit exclusions;
- release item 3 environment/evidence dependencies;
- maintainer RC/freeze/release/production authority.

Do not invent values or convert exclusions into completed security evidence.

## READY_LOCAL 7 — final independent item-4 conclusion packet

Only after inventory-driven local gaps are closed, produce one concise current conclusion naming exact reachable source/review/provenance anchors, evidence classes, unresolved policy/environment/authority gates, and whether item 4 is technically ready for a maintainer decision. This is factual synthesis only; it is not the RC/freeze/release decision.

## READY_LOCAL 8 — conditional live question only

Only if a new code/instrumentation/hypothesis/path condition creates a concrete unresolved self-owned real-network question, classify it under `docs/standing-vps-lab-authorization.md` and `docs/vps-rental-window-priority.md`. Ordinary bounded self-owned TCP/UDP work inside standing authorization needs no per-run permission. Current authoritative classification remains `READY_LIVE: none`.

## Conditional refill triggers — not duplicate work

- Dependency/build/native: reopen only on manifest/lock/feature/build-hook/native/unsafe owner change or falsification of H-I4-085 corrected truth.
- Package/repro/operator: reopen only on material owner change or new provenance contradiction; do not rerun identical VPS/package scenarios for freshness.
- Observability: reopen only on `neko-observe` owner change or a new counterexample; Candidate B remains closed.
- Reliable UDP Candidate A: reopen only if Recovery ACK owner changes or a new future/unsent-ACK counterexample appears.
- Process-resource H-R9-082/083/084: do not restart without a new counterexample; current result/cleanup truth has already been independently challenged.

## Stop / escalation conditions

Continue review/repair -> tests -> commit -> push -> next dependency-ready slice without waiting for reviewer cadence. Stop/escalate only for an unresolved BLOCKER/HIGH that cannot be automatically decided, a core Session/Carrier/ACK/crypto/wire architecture change, D019 or another true policy/value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
