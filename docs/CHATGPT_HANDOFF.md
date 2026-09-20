# ChatGPT reviewer handoff — R9-11B closed; R9-11C retained replay / Carrier-generation terminality is front

## Current repository truth

- Exact developer source/test tree remains `730f993c687683f00f82859cbb7587d9fd6f5b84`.
- **H-R9-081 is CLOSED** at `730f993`: the strengthened regression creates nonzero resolved pre-quiesce loss, then proves the first clean post-quiesce resolved interval reports `loss_per_mille == 0`, is fresh exactly once, and leaves lifetime loss diagnostics intact. Developer-local clean exact-tree provenance for exact `730f993`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-20T05:50:25Z -> 2026-09-20T05:55:22Z, Linux x86_64, rustc 1.98.0. No reviewer-local execution is claimed.
- **R9-11B is CLOSED as bounded independent no-finding support** at reviewer anchor `3b3a9182a27cc61df22250d370b87a0b50e9d253`, reviewing exact `730f993`. The accepted review covers Recovery/PathRecovery/ReliableUdpRuntime teardown ownership, receiver ACK obligations, retransmit plaintext, Reno charge, post-teardown mutators, future/never-sent ACK fail-closed behavior, and H-R9-081's quiescent health boundary. It does not close retained Session replay / Carrier-generation terminality (R9-11C) or process/socket lifecycle (R9-11D).
- **H-R9-080 remains CLOSED** at exact `052b6eab40f8e4257d4480866b21f64ce3e5230c`. **R9-11A remains CLOSED** as bounded independent no-finding support at exact `a83b89850b75d6ff405c004b8186856987f5cd9e`; `SessionRuntime.events` remains `POLICY_BLOCKED_RESOURCE_BOUND` and no retention cap/TTL/LRU/history value has been invented.
- Candidate A (`Recovery::on_ack` future/never-sent largest) remains closed by the pre-mutation guard/regressions. Candidate B (`record_datagrams` mixed queue/terminal projection) remains closed by the current mixed-delta/reason regressions.
- Sampler determinism HIGH remains closed at exact `e021b27cffe4a87de185757a8d47b8685f462bb6` with bounded release-after-observation synchronization and persisted provenance.
- `READY_LIVE: none`. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question.
- Release item 3 and item 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency. Do not wait for the next reviewer after closing a slice.

## READY_LOCAL 1 — FRONT: R9-11C retained Session replay / Carrier-generation terminality

Challenge exact-current `FailoverController`, `ConcurrentCarrierManager`, retained `(DataId, bytes)` ownership and migration-back across failure/drain/terminalization. Read applicable Session/Carrier spec and the current R9 review chain before modifying code.

Required invariants:

1. unresolved Session replay is not erased merely because UDP/path recovery terminates;
2. confirmed replay identities disappear only after the required Session logical delivery proof — Carrier packet feedback never substitutes for Session confirmation;
3. failed/retired generations cannot become active, own replay ranges, or gain new readiness without a fresh legal transition/generation;
4. migration-back cannot resurrect stale failed-generation ownership or bypass validation/health/hold gates;
5. duplicate/conflicting replay identity behavior remains deterministic and bounded; no retained bytes are silently substituted from positional/original records when the retained replay-set identity differs;
6. cleanup/partial-promotion negatives remain fail-closed and do not manufacture success/switch/delivery evidence.

For each concrete defect: smallest exact-current-semantic repair -> positive/negative regression -> focused tests -> final pushed developer SHA clean exact-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree and provenance. No decoder/framing fuzz unless this slice actually changes decoder/parser/crypto framing. No defect -> scope-precise bounded independent no-finding note and continue immediately to R9-11D.

Do not change D064 single-active/warm semantics, D019, replay-retention policy values, Session/Carrier/ACK/crypto/wire architecture, or any frozen/policy value.

## READY_LOCAL 2 — R9-11D executable process/socket lifecycle and result truth

Challenge automatic-health failover/migration-back executable paths plus TCP/UDP process fixtures:

- success, explicit failure, timeout, shutdown and post-return terminal paths release sockets/process-owned resources no longer owned;
- listener/process cleanup is proved by deterministic owner/socket/process evidence, not inferred from zero exit;
- lifecycle transitions cannot create false READY/success after terminal failure/stop;
- shutdown/result JSON/human output/exit status remain mutually consistent;
- cleanup failure remains failure/unknown, never promoted to success.

Do not open a live lane unless this local review creates a new real-network-only question.

## READY_LOCAL 3 — R9-12 final exact-tree provenance

After coherent R9-11 repair/review stabilizes, persist developer-local provenance for the final pushed developer source/test SHA: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`; `git diff --check`; clean tree; exact pushed SHA; UTC start/end; exit codes; OS/arch; stable Rust. Run pinned decode fuzz only if the coherent group actually touches wire decoder/parser/crypto framing. Hosted CI remains supplemental.

## READY_LOCAL 4 — dedicated final independent R9 bounded review

On the final R9 source/test tree, independently re-challenge Recovery, receiver ACK ownership, Session logical proof, Carrier readiness/promotion, uncertain retention, TCP replay/dedup, migration-back, process result truth and terminal cleanup. No-finding bounded review is valid item-4 support. Concrete defect converts immediately to repair; do not claim R9 closure first.

## READY_LOCAL 5 — Q10 factual reconciliation

Reconcile release packet/status claims against exact reachable R9 review/provenance anchors. Update only factual indexes/boundaries that changed. Do not mark item 4 complete or promote local evidence into WAN/security/release claims.

## READY_LOCAL 6 — Q11 release-evidence boundary reconciliation

Recheck item 3/WAN rows, historical negatives and `READY_LIVE`. Current authoritative classification remains `READY_LIVE: none`; only a new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question may open a live lane.

## READY_LOCAL 7 — Q12 governance/release-state reconciliation

Confirm D019 and other policy/value gates remain separate and RC/production/freeze/release authority has not been inferred from engineering completion. Do not select TTL/LRU/history/capacity/security numbers, signing/key-custody/SBOM/publication policy, destructive migration, frozen-release policy or release authority.

## READY_LOCAL 8 — repository-wide item-4 refill

Item 4 remains incomplete. Repeat the broad core-surface inventory rather than declaring queue exhaustion from R9 alone. Prefer implemented core surfaces lacking a dedicated reachable independent bounded challenge on the current/relevant tree, or whose semantics changed since the prior challenge:

1. `neko-reliable` ACK ranges / future-unsent / loss-retransmit / RTT-PTO / persistent congestion / Reno / fault simulation;
2. `CarrierState` generation / validation / hysteresis / single-active / drain-fail-activate;
3. concurrent manager / health / migration-back;
4. FairScheduler / multi-stream / Session+stream flow-control accounting;
5. Memory/UDP/TCP carrier close/error/resource semantics;
6. SessionRuntime lifecycle/resource/window/DeliveryAck accounting;
7. observability event/counter/high-water projection;
8. package/reproducibility/operator scripts;
9. Cargo manifests/lock/features/build/native hooks/unsafe inheritance;
10. cross-platform CLI/process-test semantics;
11. CLI exit-code / JSON / human-output contract;
12. algorithmic resource boundedness without capacity-pressure benchmarking or invented policy values;
13. release-packet factual consistency/evidence boundaries.

No-finding independent review is valid item-4 support. Do not count one narrow no-finding as repository-wide exhaustion.

## READY_LOCAL 9 — conditional live question only

Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved self-owned real-network question, classify it against `docs/standing-vps-lab-authorization.md` and `docs/vps-rental-window-priority.md`. Ordinary bounded self-owned TCP/UDP work inside standing authorization needs no per-run approval. Do not manufacture live work merely because the VPS is rented.

## Stop / escalation conditions

Continue implementation/review -> tests -> commit -> push -> next dependency-ready slice without waiting for reviewer cadence. Stop/escalate only for an unresolved BLOCKER/HIGH that cannot be auto-decided, core Session/Carrier/ACK/crypto/wire architecture change, D019/policy-value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
