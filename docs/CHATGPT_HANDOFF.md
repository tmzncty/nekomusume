# ChatGPT reviewer handoff — H-R9-080 REOPENED: remaining SessionRuntime terminal-oracle contract

## Current repository truth

- Current developer handoff anchor reviewed this pass: reachable `main` exact `cbe8c8e75082416c9d7f0eead5892995651ee66e`.
- New developer-owned commits reviewed since the prior reviewer pass:
  - `eeec59f5b2f877ebb1dc2b9b1b569d5fc513b324` — strengthens `populated_runtime()` so positive send and receive accounting coexist before terminalization;
  - `fea90f4808c5de757b9a371a7e0465472035acb5` — formatting-only follow-up on the same test tree;
  - `cbe8c8e75082416c9d7f0eead5892995651ee66e` — developer handoff claiming H-R9-080 fully closed at `fea90f4`.
- Latest independent reviewer support: exact `99fcd04c632ab5e1ad18090172948505a2de99dd`, `docs/reviews/independent-r9-11-session-terminal-oracle-reopen-fea90f4-20260920.md`.
- **H-R9-079 source repair remains accepted** at exact `271e512c01cefcfc40745aa41725bbd3e2585bb6`: `SessionRuntime::clear_runtime_state()` clears queues, receive dedup, confirmation watermarks, per-stream/session flow-control accounting, queued bytes, stream ownership, and `close_deadline_ms` while retaining documented lifetime facts.
- **H-R9-080 is CLOSED** at exact `052b6eab40f8e4257d4480866b21f64ce3e5230c`: `populated_runtime()` now `pop_send()`s the first record across the send-drain boundary before `delivery_ack` — the confirmed range represents a record that crossed the queue while the second queued record remains positively in-flight send-accounting ownership. `total_bytes` is snapshotted and asserted unchanged after both terminal paths. The cancel/Error regression now proves a rejected post-terminal `queue_send`/`open_stream` adds no fresh evidence. Developer-local clean exact-tree provenance for exact `052b6ea`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-20T01:50:16Z → 2026-09-20T01:55:23Z, Linux x86_64, rustc 1.98.0.
- Developer-local clean exact-tree provenance reported for exact `fea90f4`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-20T00:51:06Z → 2026-09-20T00:56:05Z, Linux x86_64, rustc 1.98.0. Treat as developer-reported local provenance, not reviewer-local execution. GitHub commit-status/workflow lookup exposed no hosted status/run for exact `fea90f4`; absence is not a failure claim.
- **H-R9-077 remains CLOSED** at exact `e3dca29babfaf330931f26aa3697aa3cd3c98545`: reliable-UDP teardown releases receiver ACK ownership as well as sender recovery state.
- **H-R9-078 remains CLOSED** at exact `442a8058e4ae27f867bd27db632494a945ec30f2`: repeated cancel on terminal Error is idempotent and the focused regression is active.
- Sampler determinism HIGH remains closed at exact `e021b27cffe4a87de185757a8d47b8685f462bb6` with bounded release-after-observation handshake and persisted exact-tree provenance.
- H-R9-075/H-R9-076 remain closed; R9-7/R9-8/R9-9/R9-10B/R9-10C remain at their prior accepted reachable anchors.
- Candidate A (`Recovery::on_ack` future/never-sent largest) remains closed by the current pre-mutation guard/regressions. Candidate B (`record_datagrams` mixed queue/terminal projection) remains closed by mixed-delta/reason regressions.
- `READY_LIVE: none`. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD, or Experimental Track absent a new code/instrumentation/hypothesis/path condition producing a concrete unresolved real-network question.
- Release item 3 and item 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- No reviewer-local test execution is claimed for this pass. Repository/source truth came from the GitHub repository connector; the reviewer runtime did not have a materialized checkout. Developer-local exact-tree gates remain first-class; hosted CI is supplemental.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency. Because H-R9-080 is a current HIGH, close it before widening R9-11.

## READY_LOCAL 1 — FRONT HIGH: finish H-R9-080 survivor + cancel-negative oracle

Independent finding: exact `99fcd04c632ab5e1ad18090172948505a2de99dd`, `docs/reviews/independent-r9-11-session-terminal-oracle-reopen-fea90f4-20260920.md`.

Keep the current shared source cleanup and the positive send/receive accounting assertions. Repair only the remaining oracle contract:

1. In `populated_runtime()`, queue two sends, `pop_send()` exactly the first, then `delivery_ack()` only that first range. Preserve separate positive assertions for remaining `session_send_inflight`, per-stream `send_inflight`, session/per-stream receive-window ownership, non-empty `send`/`recv`/`received`/`confirmed`/`streams`, positive `queued_bytes`, and an armed close deadline.
2. Snapshot `total_bytes` (or another already-documented lifetime survivor) before terminalization. For first cancel/Error and remote close separately, assert runtime-owned mutable state clears while the survivor remains unchanged.
3. On the cancel/Error path, after repeated-cancel event-count idempotence, invoke one representative ordinary mutator such as `queue_send` or `open_stream`; assert the current typed terminal failure and unchanged observable-event count. Do not invent a new error code.
4. Keep the remote-close negative, idle-timeout and close-deadline controls green.
5. Do not change `SessionRuntime.events` retention policy, D019, capacity/TTL/LRU/history/security values, or Session/Carrier/ACK/wire/crypto semantics. No decoder/parser/crypto-framing change => no mechanical fuzz requirement.

Run focused `neko-session` tests, then final pushed source/test SHA developer-local exact-tree gate: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`; `git diff --check`; clean tree; exact reachable SHA; UTC start/end; exit codes; OS/arch; stable Rust.

After truthful closure, continue directly to R9-11A remainder without waiting for reviewer cadence.

## READY_LOCAL 2 — R9-11A SessionRuntime terminal cleanup remainder

On the repaired exact tree, independently challenge remote close, cancel, idle timeout and graceful-close deadline as separate terminal paths:

- every post-Closed/post-Error mutating API is fail-closed or explicitly idempotent by committed semantics;
- rejected post-terminal calls append no false success/delivery/window evidence;
- stream/timer ownership, queues, dedup, watermarks, flow-control counters and queued-byte ownership are terminal-clean;
- documented lifetime facts are the only intended survivors;
- `SessionRuntime.events` retained-state capacity remains `POLICY_BLOCKED_RESOURCE_BOUND`; do not invent a number.

Concrete defect -> smallest repair + positive/negative regression + exact-tree gate. No defect -> scope-precise independent bounded no-finding note and continue immediately.

## READY_LOCAL 3 — R9-11B Recovery / reliable-UDP terminal ownership re-challenge

Challenge combined terminal state after H-R9-077:

- PathRecovery live sent map/outstanding copies/Reno charge/retransmit plaintext/packet-frame map/receiver ACK obligations are zero or explicitly terminal-inert;
- quiesce preserves only documented lifetime diagnostics, never stale retransmission eligibility or fresh health outcome;
- post-teardown send/retransmit/receive/apply-ACK/PTO/health/control-plane mutators cannot manufacture fresh positive evidence;
- abort/teardown ordering cannot orphan retained plaintext or make aborted packets ACK-valid;
- future/never-sent ACK stays fail-closed before RTT/loss/PTO mutation.

No finding -> independent bounded note; finding -> smallest repair.

## READY_LOCAL 4 — R9-11C retained Session replay / Carrier-generation terminality

Challenge `FailoverController`, `ConcurrentCarrierManager`, retained `(DataId, bytes)` ownership and migration-back across failure/drain/terminalization:

- unresolved Session replay is not erased merely because UDP/path recovery terminates;
- confirmed replay identities disappear only after required Session logical proof;
- failed/retired generations cannot become active or gain ownership without a fresh legal transition/generation;
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

After coherent R9-11 repair/review stabilizes, persist developer-local provenance for the final pushed developer source/test SHA: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`; `git diff --check`; clean tree; exact pushed SHA; UTC start/end; exit codes; OS/arch; stable Rust. Run pinned decode fuzz only if the coherent group actually touches wire decoder/parser/crypto framing. Hosted CI is supplemental.

## READY_LOCAL 7 — dedicated final independent R9 bounded review

On the final R9 source/test tree, independently re-challenge Recovery, receiver ACK ownership, Session logical proof, Carrier readiness/promotion, uncertain retention, TCP replay/dedup, migration-back, process result truth and terminal cleanup. A no-finding bounded review is valid item-4 support. A concrete defect converts immediately to repair; do not claim R9 closure first.

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
