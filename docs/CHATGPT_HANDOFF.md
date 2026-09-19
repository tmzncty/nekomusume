# ChatGPT reviewer handoff — H-R9-078 closed at 442a805; R9-11 lifecycle continues

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `442a805` (`fix(session): restore #[test] attributes lost in H-R9-078 edit`), on top of `dd7afc6a1af69f5722250fa475fef805fd330c31` (`fix(session): H-R9-078 repeated cancel is idempotent on terminal Error`) and `e3dca29babfaf330931f26aa3697aa3cd3c98545`.
- Latest independent reviewer support: exact `ae65dba44d302b02cdfb02a8429bc077c1533ff1`, `docs/reviews/independent-r9-11-session-cancel-terminality-4129276-20260920.md`.
- **H-R9-077 remains CLOSED** at exact `e3dca29`: `ReliableUdpRuntime::teardown()` replaces its `PacketAckTracker` with a fresh empty tracker for the same generation, so observed ranges/largest observation/pending ACK obligation no longer survive terminalization.
- Developer-local clean exact-tree provenance for exact `442a805`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-19T20:57:46Z → 2026-09-19T21:02:29Z, Linux x86_64, rustc 1.98.0. This remains developer-reported local evidence, not reviewer-local execution.
- **H-R9-078 closed:** `SessionRuntime::cancel()` on an already-`Error` runtime is now idempotent `Ok(())` — no new terminal `Error` event, no re-clear, no retained-state growth, matching the already-`Closed` path. `repeated_cancel_is_idempotent_no_new_error_events` proves exactly one `Error` event and no growth on repeated cancel.
- **Sampler determinism HIGH remains CLOSED** at exact `e021b27cffe4a87de185757a8d47b8685f462bb6`; its child/resource observation uses a true bounded release-after-observation handshake rather than a fixed sleep. Persisted developer-local exact-tree provenance and hosted CI remain historical supporting evidence only.
- **R9-10B remains closed** at exact `70e87f086b0bbd2d13cd9bae073cb1cdb4c5abd4`; automatic-health TCP replay constructs replayed records from the authoritative retained `(DataId, bytes)` returned by `FailoverController::tcp_resend()`.
- **R9-10C remains independently bounded no-finding closed** at reviewer exact `a1de16f5568951fdc5e99846436f918ed8d99ed2`; full lifecycle/resource terminalization remains R9-11.
- **H-R9-075/H-R9-076 remain closed**: SessionRuntime rejects forward-gap DeliveryAck atomically, and resumed-session fixtures apply earlier logical proof before later proofs/confirm.
- R9-7 remains independently bounded no-finding closed at `cdac663e4d2649c8f87fc2263766616dc9b4bbe9`; R9-8 at `7c87ac675a381154ae7fceca987e7f2f106c26cf`; R9-9 at `b4a527a1fff994ee0836893f850c36eadb609eb1`.
- Candidate A (`Recovery::on_ack` future/never-sent largest) remains closed by the current pre-mutation guard and regressions. Candidate B (`record_datagrams` mixed queue/terminal drop projection) remains closed by mixed-delta/reason regressions.
- `READY_LIVE: none`. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track absent a new code/instrumentation/hypothesis/path condition producing a concrete unresolved real-network question.
- Release item 3 and item 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- No reviewer-local test execution is claimed for this pass; direct container access could not resolve GitHub, so source/repository truth came from the GitHub repository API/connector. Developer-local exact-tree gates remain first-class; hosted CI is supplemental.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency. **H-R9-078 is now the front correctness/resource HIGH; close it before widening downstream implementation/review.** After closing it, continue immediately through the preserved queue below without waiting for another reviewer pass.

## READY_LOCAL 1 — FRONT HIGH: H-R9-078 repeated cancel mutates terminal Error state

Independent finding: `ae65dba44d302b02cdfb02a8429bc077c1533ff1`, `docs/reviews/independent-r9-11-session-cancel-terminality-4129276-20260920.md`.

Exact-current owner facts:

- `SessionRuntime::check()` rejects `cancelled || state == Error` before ordinary mutators can change state/evidence.
- `tick()` treats `Error` as terminal and is observationally inert.
- the first `cancel()` sets `cancelled=true`, enters `RuntimeState::Error`, clears runtime-owned delivery/window state and emits one `RuntimeEventKind::Error`.
- `cancel()` special-cases only `Closed`; repeated `cancel()` while already `Error` re-clears state and appends another retained `Error` event each time.
- `events` is lifetime-retained diagnostic history and its capacity remains a separate policy-blocked question. H-R9-078 does **not** choose a cap: the defect is that a terminal session can continue manufacturing new retained events after terminalization.

Smallest accepted repair shape:

1. make `cancel()` on already-`Error` either explicitly idempotent `Ok(())` with no new mutation/event (matching the existing Closed fast-path) or fail closed using an existing committed terminal/cancelled error, also without mutation/event; do not invent a new error code;
2. focused regression must populate at least one owned runtime surface, perform the first cancel, prove exactly one Error event and cleared runtime ownership, then invoke cancel repeatedly and prove event count/state/ownership/lifetime diagnostics remain unchanged;
3. preserve ordinary post-Error mutators failing before delivery/window/success evidence mutation;
4. do not fold the known `SessionRuntime.events` capacity policy gate into this repair and do not select TTL/LRU/history-size/capacity numbers;
5. focused `neko-session` tests, then final pushed-SHA developer-local exact-tree gate: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, persisted exact GitHub-resolvable SHA + UTC start/end + exit codes + OS/arch + stable Rust. No decoder/parser/crypto-framing change means no mechanical fuzz requirement.

After repair/provenance, continue directly to R9-11A remainder.

## READY_LOCAL 2 — R9-11A SessionRuntime terminal cleanup remainder

Read exact-current `SessionRuntime` owners/tests after H-R9-078 and challenge remote close, cancel, idle timeout and graceful-close deadline as separate terminal paths.

Required checks:

- send/recv queues, dedup history, confirmed watermarks, per-stream/session send/receive window accounting and queued-byte ownership are released on every terminal path that owns them;
- every post-Closed and post-Error mutating API is fail-closed or explicitly idempotent by committed semantics;
- rejected post-terminal calls append no false success/delivery/window evidence;
- inspect remaining non-lifetime state (including stream/timer ownership) rather than assuming `clear_runtime_state()` is exhaustive; lifetime facts may remain only where comments/tests intentionally define them as such;
- `SessionRuntime.events` retained-state capacity remains `POLICY_BLOCKED_RESOURCE_BOUND`; do not invent a number.

Concrete defect -> smallest repair + positive/negative regression + exact-tree gate. No defect -> scope-precise independent bounded no-finding note and continue immediately.

## READY_LOCAL 3 — R9-11B Recovery / reliable-UDP terminal ownership re-challenge

Independently re-challenge combined terminal state after H-R9-077:

- PathRecovery live sent map/outstanding copies/Reno charge/retransmit plaintext/packet-frame map/receiver ACK obligations are zero or explicitly terminal-inert;
- quiesce preserves only documented lifetime diagnostics, never stale retransmission eligibility or fresh health outcome;
- post-teardown send/retransmit/receive/apply-ACK/PTO/health/control-plane mutators cannot manufacture fresh positive evidence;
- abort/teardown ordering cannot orphan retained plaintext or make aborted packets ACK-valid;
- future/never-sent ACK remains fail-closed before RTT/loss/PTO mutation.

No finding -> independent bounded note; finding -> smallest repair.

## READY_LOCAL 4 — R9-11C retained Session replay and Carrier-generation terminality

Challenge `FailoverController`, `ConcurrentCarrierManager`, retained `DataId`/bytes ownership and migration-back across path failure/drain/terminalization:

- unresolved retained Session replay is not silently erased merely because UDP/path recovery terminates;
- confirmed replay identities are removed only after required Session logical proof;
- failed/retired generations cannot become active or receive new ownership without a fresh legal transition/generation;
- migration-back cannot resurrect stale failed-generation ownership;
- Carrier packet feedback never substitutes for Session confirmation.

Do not change D064 single-active/warm semantics or replay-retention policy values.

## READY_LOCAL 5 — R9-11D executable process/socket lifecycle and result truth

Challenge automatic-health failover/migration-back executable paths plus TCP/UDP process fixtures:

- success, explicit failure, timeout, shutdown and post-return terminal paths release sockets/process-owned resources no longer owned;
- listener/process cleanup is proved by deterministic owner/socket/process evidence, not inferred from zero exit;
- lifecycle transitions cannot create false READY/success after terminal failure/stop;
- shutdown/result JSON/human output/exit status remain mutually consistent;
- cleanup failure remains failure/unknown, never promoted to success.

Do not open a live lane unless this local review creates a new real-network-only question.

## READY_LOCAL 6 — R9-12 final exact-tree provenance

After coherent R9-11 repair/review stabilizes, run/persist developer-local provenance for the final pushed developer source/test SHA:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- `git diff --check`
- clean tree
- exact pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust

Run pinned decode fuzz only if the coherent group actually touches wire decoder/parser/crypto framing. Hosted CI is supplemental.

## READY_LOCAL 7 — dedicated final independent R9 bounded review

On the final R9 exact source/test tree, independently re-challenge Recovery, receiver ACK ownership, Session logical proof, Carrier readiness/promotion, uncertain retention, TCP replay/dedup, migration-back, process result truth and terminal cleanup. A no-finding bounded review is valid item-4 support. A concrete defect converts immediately to repair; do not claim R9 closure first.

## READY_LOCAL 8 — Q10 factual reconciliation

Reconcile release packet/status claims against exact reachable R9 review/provenance anchors. Update only factual indexes/boundaries that changed. Do not mark item 4 complete or promote local evidence into WAN/security/release claims.

## READY_LOCAL 9 — Q11 release-evidence boundary reconciliation

Recheck item 3/WAN rows, historical negatives and `READY_LIVE`. Current authoritative classification remains `READY_LIVE: none`; only a new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question may open a live lane.

## READY_LOCAL 10 — Q12 governance/release-state reconciliation

Confirm D019 and other policy/value gates remain separate and RC/production/freeze/release authority has not been inferred from engineering completion. Do not select TTL/LRU/history/capacity/security numbers, signing/key-custody/SBOM/publication policy, destructive migration, frozen-release policy or release authority.

## READY_LOCAL 11 — repository-wide item-4 refill

Item 4 remains incomplete. Repeat the broad core-surface inventory rather than declaring queue exhaustion from one R9 sweep. Prefer implemented core surfaces lacking a dedicated reachable independent bounded challenge on the current/relevant tree, or whose semantics changed since the prior challenge:

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

## READY_LOCAL 12 — conditional live question only

Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved self-owned real-network question, classify it against `docs/standing-vps-lab-authorization.md` and `docs/vps-rental-window-priority.md`. Ordinary bounded self-owned TCP/UDP work inside standing authorization needs no per-run approval. Do not manufacture live work merely because the VPS is rented.

## Stop / escalation conditions

Continue implementation/review -> tests -> commit -> push -> next dependency-ready slice without waiting for reviewer cadence. Stop/escalate only for an unresolved BLOCKER/HIGH that cannot be auto-decided, core Session/Carrier/ACK/crypto/wire architecture change, D019/policy-value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
