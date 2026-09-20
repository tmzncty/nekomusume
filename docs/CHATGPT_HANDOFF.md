# ChatGPT reviewer handoff — H-R9-081 closure oracle HIGH; R9-11B remains open

## Current repository truth

- Current reachable reviewer anchor is exact `16a641394b49fd36cb406ea3f16b13ee2ae4da33`, which records the independent H-R9-081 closure-oracle finding in `docs/reviews/independent-r9-11b-h-r9-081-closure-gap-3202796-20260920.md`.
- Current source/test repair tree is developer exact `320279601ed7a414dfef81e667aad6b30526907e` (`4f40b2d` source repair + `18a67a7` focused regression + `3202796` adjacent-test attribute restoration). Later commits through this handoff are review/handoff-only.
- **H-R9-080 remains CLOSED** at exact `052b6eab40f8e4257d4480866b21f64ce3e5230c`: the SessionRuntime terminal-cleanup regressions cross the send-drain boundary, preserve documented `total_bytes`, and include the required cancel post-terminal mutator negative. Developer-local exact-tree provenance remains recorded as green; do not reinterpret it as reviewer-local execution.
- **R9-11A remains CLOSED as bounded independent no-finding review support** at exact `a83b89850b75d6ff405c004b8186856987f5cd9e`; `SessionRuntime.events` remains `POLICY_BLOCKED_RESOURCE_BOUND`, and no cap/TTL/LRU/history value has been invented.
- Developer exact `6fe8ae83f3e213e0978bed5cf2e723e739857c0f` added a docs-only R9-11B no-finding note, but it cannot itself close R9-11B because H-R9-081 falsified one of its explicit quiesce/fresh-health claims.
- **H-R9-081 is CLOSED** at exact `c52efef3069c8b5ba8430ec4fee6657365f9ee92` (on top of `4f40b2d`/`18a67a7`/`3202796`): `quiesce()` marks outcome consumed (`health_epoch = Some(outcome_epoch)` + aligned `last_health_*`), and the committed regressions now cover all required shapes — `quiesce_cannot_manufacture_fresh_health_from_pre_quiesce_outcome` (unconsumed), `quiesce_consumed_outcome_stays_consumed` (consumed pre-quiesce stays consumed), and `post_quiesce_outcome_is_fresh_once_then_consumed` (post-quiesce outcome fresh exactly once, interval deltas at quiescent boundary). Developer-local clean exact-tree provenance for exact `c52efef`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-20T04:50:27Z → 2026-09-20T04:55:32Z, Linux x86_64, rustc 1.98.0.
- GitHub-hosted Rust CI run `35488510885` completed `success` on exact `3202796`. Developer-local clean exact-tree provenance for exact `3202796` is recorded as `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-20T04:12:17Z -> 2026-09-20T04:17:40Z, Linux x86_64, rustc 1.98.0. Hosted CI is supplemental; no reviewer-local execution is claimed.
- Outer `ReliableUdpRuntime::teardown()` still sets `torn_down`, and `poll_health()` returns `RuntimeEvent::Idle` before consulting `PathRecovery`; H-R9-081 does not establish an executable false fallback after teardown.
- Candidate A (`Recovery::on_ack` future/never-sent largest) remains closed by the pre-mutation guard/regressions. Candidate B (`record_datagrams` mixed queue/terminal projection) remains closed by the current mixed-delta/reason regressions.
- Sampler determinism HIGH remains closed at exact `e021b27cffe4a87de185757a8d47b8685f462bb6` with bounded release-after-observation synchronization and persisted provenance.
- `READY_LIVE: none`. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question.
- Release item 3 and item 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency. Do not wait for the next reviewer after closing a slice.

## READY_LOCAL 1 — FRONT HIGH H-R9-081 closure-oracle completion

Read exact-current `PathRecovery::{on_sent,on_ack,on_pto,quiesce,fresh_health_sample}`, the original finding at `eeb49e4`, and the closure-gap note at `16a6413`.

The source fix in `4f40b2d` is currently accepted. Complete the missing regression contract only:

1. Keep the existing **unconsumed** pre-quiesce outcome regression.
2. Add an **already-consumed** case: create a real resolved outcome, consume it once with `fresh_health_sample()`, call `quiesce()`, and require the immediate next `fresh_health_sample()` to be `None`; lifetime diagnostics must remain unchanged.
3. Add a **post-quiesce reuse** case: after the quiescent boundary, send and resolve genuinely new work; require exactly one fresh health sample for that new resolved outcome and `None` on repeated polling. The interval result must use only post-quiesce resolved deltas; pre-quiesce loss/history may survive diagnostically but must not leak into the new interval.
4. Do not add a terminal flag or redesign Recovery/health, Session/Carrier/ACK/crypto/wire semantics. Do not change policy values.
5. Run focused `neko-carrier` tests, then for the final pushed developer SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and verify a clean tree with exact SHA, UTC start/end, exit codes, OS/arch, and stable Rust version. No decoder/framing change is involved, so do not mechanically run fuzz.

After this is pushed and gated, continue immediately into R9-11B remainder; do not wait for reviewer cadence.

## READY_LOCAL 2 — R9-11B Recovery / reliable-UDP terminal ownership remainder

Re-run the bounded independent challenge on the repaired exact tree; do not inherit `6fe8ae8`'s no-finding conclusion without re-challenging H-R9-081.

Challenge:

- PathRecovery sent map/outstanding copies/Reno charge/retransmit plaintext/packet-frame map/receiver ACK obligations are zero or explicitly terminal-inert after teardown;
- quiesce preserves documented lifetime diagnostics but neither stale retransmission eligibility nor fresh pre-quiesce health evidence;
- post-teardown send/retransmit/receive/apply-ACK/PTO/health/control-plane paths cannot manufacture fresh positive evidence;
- `abandon_sent` / `abandon_retransmit` after teardown must remain observationally inert even though they are not separately `torn_down`-gated today; a concrete state/evidence mutation is a defect, mere absence of a redundant guard is not;
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
