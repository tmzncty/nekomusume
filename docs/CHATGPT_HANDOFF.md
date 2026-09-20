# ChatGPT reviewer handoff — H-R9-081 FRONT HIGH; R9-11B not closed

## Current repository truth

- Current reviewer repository anchor before this handoff update: reachable exact `eeb49e44788413a5287dc1660b9c31244c7f9745`, which records independent finding H-R9-081 in `docs/reviews/independent-r9-11b-pathrecovery-quiesce-health-freshness-6fe8ae8-20260920.md`.
- Exact current source/test tree under review remains developer exact `052b6eab40f8e4257d4480866b21f64ce3e5230c`; commits `4f9dfff`, `a83b898`, `3adcbd1`, `6fe8ae8`, and `eeb49e4` are docs/review/handoff-only.
- **H-R9-080 remains CLOSED** at exact `052b6ea`: the SessionRuntime terminal-cleanup regressions now cross the send-drain boundary, preserve documented lifetime survivor `total_bytes`, and include the required cancel post-terminal mutator negative. Developer-reported clean exact-tree provenance for exact `052b6ea`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-20T01:50:16Z -> 2026-09-20T01:55:23Z, Linux x86_64, rustc 1.98.0. Treat this as developer-local provenance, not reviewer-local execution.
- **R9-11A SessionRuntime terminal cleanup remainder remains CLOSED as bounded no-finding review support** at exact `a83b89850b75d6ff405c004b8186856987f5cd9e`; `SessionRuntime.events` capacity remains `POLICY_BLOCKED_RESOURCE_BOUND` and no policy value was invented.
- Developer exact `6fe8ae83f3e213e0978bed5cf2e723e739857c0f` added a docs-only R9-11B no-finding note. GitHub-hosted Rust CI run `35484901555` completed `success` on that exact SHA. **Do not treat the R9-11B no-finding note as closure:** H-R9-081 below falsifies one of its explicit quiesce/fresh-health claims on the same source/test tree.
- **H-R9-081 OPEN / HIGH — PathRecovery quiesce manufactures a fresh health observation.** Exact-current `PathRecovery::quiesce()` sets `health_epoch = None`, while `fresh_health_sample()` returns `None` only when `health_epoch == Some(outcome_epoch)`. Therefore `resolved outcome -> quiesce() -> fresh_health_sample()` produces `Some(HealthSample)` without any post-quiesce resolved outcome; an unconsumed pre-quiesce resolved delta can also be projected after quiesce. This contradicts the method freshness contract and the R9-11B invariant that quiesce must not create fresh health evidence.
- Scope boundary for H-R9-081: exact-current outer `ReliableUdpRuntime::teardown()` still sets `torn_down`, and `ReliableUdpRuntime::poll_health()` returns `RuntimeEvent::Idle` before calling the recovery bridge. This finding does **not** establish a live/executable false fallback after outer teardown; it is a public `PathRecovery` freshness/correctness defect plus release-review closure-truth gap.
- H-R9-079 source repair remains accepted at exact `271e512c01cefcfc40745aa41725bbd3e2585bb6`. H-R9-077/H-R9-078 and H-R9-075/H-R9-076 remain closed at their prior reachable anchors.
- Candidate A (`Recovery::on_ack` future/never-sent largest) remains closed by the current pre-mutation guard/regressions. Candidate B (`record_datagrams` mixed queue/terminal projection) remains closed by current mixed-delta/reason regressions.
- Sampler determinism HIGH remains closed at exact `e021b27cffe4a87de185757a8d47b8685f462bb6` with bounded release-after-observation synchronization and persisted provenance.
- `READY_LIVE: none`. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track absent a new code/instrumentation/hypothesis/path condition producing a concrete unresolved self-owned real-network question.
- Release item 3 and item 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- No reviewer-local test execution is claimed for this pass. Repository/source truth came from GitHub; developer-local clean exact-tree gates remain first-class and hosted CI is supplemental.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency. Do not wait for the next reviewer after closing a slice.

## READY_LOCAL 1 — FRONT HIGH H-R9-081 PathRecovery quiesce / health freshness

Read exact-current `PathRecovery::{quiesce,fresh_health_sample}` and the independent finding at `eeb49e4`.

Repair contract:

1. Add a focused deterministic direct-`PathRecovery` regression that creates a real resolved outcome, calls `quiesce()`, and requires the immediate `fresh_health_sample()` to be `None`.
2. Cover both preconditions: the old health outcome was already consumed, and a pre-quiesce resolved delta remained unconsumed. Neither may become fresh merely because quiesce occurred.
3. Preserve lifetime diagnostics (`packets_sent`, `packets_lost`, RTT/PTO and existing outcome/resolved history). Do not erase truthful history to make the test pass.
4. If the public `PathRecovery` remains reusable after quiesce, a later strictly post-quiesce resolved outcome may become fresh, but its interval delta must start at the quiescent boundary and must not replay pre-quiesce resolved loss. Use the smallest repair consistent with exact-current semantics; marking the current outcome consumed and aligning interval-delta baselines at quiesce is an acceptable minimal shape if tests confirm it.
5. Do not redesign Session/Carrier/ACK/crypto/wire architecture and do not change policy values.
6. Run focused `neko-carrier` tests, then for the final pushed developer SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and verify a clean tree with exact SHA, UTC start/end, exit codes, OS/arch, and stable Rust version. No decoder/framing change is involved, so do not mechanically run fuzz.

After the repair is pushed and gated, continue immediately into R9-11B remainder; do not wait for reviewer cadence.

## READY_LOCAL 2 — R9-11B Recovery / reliable-UDP terminal ownership remainder

Re-run the bounded independent challenge on the repaired exact tree. Keep the already-inspected ownership claims, but explicitly re-challenge the repaired quiesce/freshness seam instead of inheriting `6fe8ae8`'s no-finding conclusion.

Challenge:

- PathRecovery live sent map/outstanding copies/Reno charge/retransmit plaintext/packet-frame map/receiver ACK obligations are zero or explicitly terminal-inert after teardown;
- quiesce preserves only documented lifetime diagnostics, never stale retransmission eligibility or a fresh health outcome;
- post-teardown send/retransmit/receive/apply-ACK/PTO/health/control-plane mutators cannot manufacture fresh positive evidence;
- abort/teardown ordering cannot orphan retained plaintext or make aborted packets ACK-valid;
- future/never-sent ACK remains fail-closed before RTT/loss/PTO mutation.

Concrete defect -> smallest repair + positive/negative regression + exact-tree local gate. No defect -> scope-precise independent bounded no-finding note and continue immediately.

## READY_LOCAL 3 — R9-11C retained Session replay / Carrier-generation terminality

Challenge `FailoverController`, `ConcurrentCarrierManager`, retained `(DataId, bytes)` ownership and migration-back across failure/drain/terminalization:

- unresolved Session replay is not erased merely because UDP/path recovery terminates;
- confirmed replay identities disappear only after required Session logical proof;
- failed/retired generations cannot become active or gain ownership without a fresh legal transition/generation;
- migration-back cannot resurrect stale failed-generation ownership;
- Carrier packet feedback never substitutes for Session confirmation.

Do not change D064 single-active/warm semantics, D019, replay-retention policy values or core architecture.

## READY_LOCAL 4 — R9-11D executable process/socket lifecycle and result truth

Challenge automatic-health failover/migration-back executable paths plus TCP/UDP process fixtures:

- success, explicit failure, timeout, shutdown and post-return terminal paths release sockets/process-owned resources no longer owned;
- listener/process cleanup is proved by deterministic owner/socket/process evidence, not inferred from zero exit;
- lifecycle transitions cannot create false READY/success after terminal failure/stop;
- shutdown/result JSON/human output/exit status remain mutually consistent;
- cleanup failure remains failure/unknown, never promoted to success.

Do not open a live lane unless this local review creates a new real-network-only question.

## READY_LOCAL 5 — R9-12 final exact-tree provenance

After coherent R9-11 repair/review stabilizes, persist developer-local provenance for the final pushed developer source/test SHA: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`; `git diff --check`; clean tree; exact pushed SHA; UTC start/end; exit codes; OS/arch; stable Rust. Run pinned decode fuzz only if the coherent group actually touches wire decoder/parser/crypto framing. Hosted CI remains supplemental.

## READY_LOCAL 6 — dedicated final independent R9 bounded review

On the final R9 source/test tree, independently re-challenge Recovery, receiver ACK ownership, Session logical proof, Carrier readiness/promotion, uncertain retention, TCP replay/dedup, migration-back, process result truth and terminal cleanup. No-finding bounded review is valid item-4 support. Concrete defect converts immediately to repair; do not claim R9 closure first.

## READY_LOCAL 7 — Q10 factual reconciliation

Reconcile release packet/status claims against exact reachable R9 review/provenance anchors. Update only factual indexes/boundaries that changed. Do not mark item 4 complete or promote local evidence into WAN/security/release claims.

## READY_LOCAL 8 — Q11 release-evidence boundary reconciliation

Recheck item 3/WAN rows, historical negatives and `READY_LIVE`. Current authoritative classification remains `READY_LIVE: none`; only a new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question may open a live lane.

## READY_LOCAL 9 — Q12 governance/release-state reconciliation

Confirm D019 and other policy/value gates remain separate and RC/production/freeze/release authority has not been inferred from engineering completion. Do not select TTL/LRU/history/capacity/security numbers, signing/key-custody/SBOM/publication policy, destructive migration, frozen-release policy or release authority.

## READY_LOCAL 10 — repository-wide item-4 refill

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

## READY_LOCAL 11 — conditional live question only

Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved self-owned real-network question, classify it against `docs/standing-vps-lab-authorization.md` and `docs/vps-rental-window-priority.md`. Ordinary bounded self-owned TCP/UDP work inside standing authorization needs no per-run approval. Do not manufacture live work merely because the VPS is rented.

## Stop / escalation conditions

Continue implementation/review -> tests -> commit -> push -> next dependency-ready slice without waiting for reviewer cadence. Stop/escalate only for an unresolved BLOCKER/HIGH that cannot be auto-decided, core Session/Carrier/ACK/crypto/wire architecture change, D019/policy-value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
