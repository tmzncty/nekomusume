# ChatGPT reviewer handoff — H-R9-081 interval-delta oracle HIGH; R9-11B remains open

## Current repository truth

- Current reachable reviewer anchor is exact `375443310bd4f76d1dc69dfc88475a6064fb70ae`, which records `docs/reviews/independent-r9-11b-h-r9-081-interval-delta-gap-c52efef-20260920.md`.
- Current developer source/test tree is exact `c52efef3069c8b5ba8430ec4fee6657365f9ee92` on top of `4f40b2d` / `18a67a7` / `3202796`. Its new tests correctly cover consumed pre-quiesce freshness and post-quiesce reuse/fresh-once behavior; the source repair in `4f40b2d` remains accepted.
- **H-R9-081 is CLOSED** at exact `730f993c687683f00f82859cbb7587d9fd6f5b84` (on top of `c52efef`/`4f40b2d`/`18a67a7`/`3202796`): `post_quiesce_outcome_is_fresh_once_then_consumed` now creates a real pre-quiesce resolved loss (sends 0-7, ACKs 0+7 → packets 1-6 resolved as loss), then asserts the post-quiesce clean ACK produces `loss_per_mille == 0` — pre-quiesce loss survives diagnostics but never leaks into the post-quiesce interval delta. Developer-local clean exact-tree provenance for exact `730f993`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-20T05:50:25Z → 2026-09-20T05:55:22Z, Linux x86_64, rustc 1.98.0.
- **H-R9-080 remains CLOSED** at exact `052b6eab40f8e4257d4480866b21f64ce3e5230c`. **R9-11A remains CLOSED** as bounded independent no-finding support at exact `a83b89850b75d6ff405c004b8186856987f5cd9e`; `SessionRuntime.events` remains `POLICY_BLOCKED_RESOURCE_BOUND` and no retention cap/TTL/LRU/history value has been invented.
- Developer exact `6fe8ae83f3e213e0978bed5cf2e723e739857c0f` is a docs-only R9-11B no-finding note and cannot close R9-11B until H-R9-081 is fully closed and the remainder is re-challenged on the repaired exact tree.
- Developer-local clean exact-tree provenance for exact `c52efef` is recorded as `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-20T04:50:27Z -> 2026-09-20T04:55:32Z, Linux x86_64, rustc 1.98.0. The GitHub connector currently exposes no hosted workflow run/combined status for exact `c52efef`; absence of hosted status is not treated as failure. No reviewer-local execution is claimed.
- Candidate A (`Recovery::on_ack` future/never-sent largest) remains closed by the pre-mutation guard/regressions. Candidate B (`record_datagrams` mixed queue/terminal projection) remains closed by the current mixed-delta/reason regressions.
- Sampler determinism HIGH remains closed at exact `e021b27cffe4a87de185757a8d47b8685f462bb6` with bounded release-after-observation synchronization and persisted provenance.
- `READY_LIVE: none`. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question.
- Release item 3 and item 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency. Do not wait for the next reviewer after closing a slice.

## READY_LOCAL 1 — FRONT HIGH H-R9-081 post-quiesce interval-delta oracle

Read exact-current `PathRecovery::{on_sent,on_ack,quiesce,fresh_health_sample}`, the original finding at `eeb49e4`, closure-gap note at `16a6413`, and reviewer anchor `3754433`.

The source repair in `4f40b2d` is accepted. Complete the remaining mutation-sensitive regression contract:

1. Keep the existing consumed/unconsumed freshness regressions.
2. In the post-quiesce reuse case, first create a **real pre-quiesce resolved loss** (not merely an outstanding packet) and prove lifetime loss diagnostics are nonzero and survive quiesce.
3. After quiesce, send/resolve clean new work and inspect the returned `HealthSample`: the first post-quiesce interval must be derived only from post-quiesce resolved counters; specifically a clean interval must report `loss_per_mille == 0` despite nonzero lifetime/pre-quiesce loss.
4. The new post-quiesce outcome remains fresh exactly once and repeated polling returns `None`.
5. Preserve public reusable `PathRecovery`; do not add a terminal flag or redesign Recovery/health, Session/Carrier/ACK/crypto/wire semantics or policy values.
6. If this regression exposes a source bug, make the smallest exact-current-semantic repair. Otherwise test-only closure is sufficient.
7. Run focused `neko-carrier` tests, then for the final pushed developer SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and verify a clean tree with exact SHA, UTC start/end, exit codes, OS/arch, and stable Rust version. No decoder/framing change is involved, so do not mechanically run fuzz.

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
