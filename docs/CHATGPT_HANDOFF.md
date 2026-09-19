# ChatGPT reviewer handoff — H-R9-079 terminal stream/timer ownership HIGH; R9-11 lifecycle continues

## Current repository truth

- Current reviewer anchor before this handoff: reachable `main` exact `26462bb31617486b793ee7d90e8dcf5176552bc9`; latest developer-owned source/test commit reviewed is exact `442a8058e4ae27f867bd27db632494a945ec30f2`, on top of `dd7afc6a1af69f5722250fa475fef805fd330c31` and `e3dca29babfaf330931f26aa3697aa3cd3c98545`.
- Latest independent reviewer support: exact `26462bb31617486b793ee7d90e8dcf5176552bc9`, `docs/reviews/independent-r9-11-session-terminal-stream-timer-ownership-ee62338-20260920.md`.
- **H-R9-077 remains CLOSED** at exact `e3dca29`: `ReliableUdpRuntime::teardown()` releases receiver ACK ownership as well as sender-side recovery state.
- **H-R9-078 remains CLOSED** at exact `442a805`: repeated `SessionRuntime::cancel()` on terminal `Error` is idempotent and the focused regression is an active `#[test]`. Developer-local clean exact-tree provenance for exact `442a805` is recorded as `scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-19T20:57:46Z → 2026-09-19T21:02:29Z, Linux x86_64, rustc 1.98.0. GitHub-hosted Rust CI run `35468597623` for exact `442a805` completed successfully; hosted CI remains supplemental.
- **H-R9-079 is CLOSED** at exact `271e512c01cefcfc40745aa41725bbd3e2585bb6`: `clear_runtime_state()` now clears `streams` (per-stream mutable state/next_send/next_receive) and disarms `close_deadline_ms` — live-session ownership does not survive terminalization. `idle_timeout_releases_all_runtime_owned_state` and `close_deadline_releases_all_runtime_owned_state` now assert `streams.is_empty()` and `close_deadline_ms.is_none()`. Developer-local clean exact-tree provenance for exact `271e512`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-19T21:50:56Z → 2026-09-19T21:55:52Z, Linux x86_64, rustc 1.98.0.
- **Sampler determinism HIGH remains CLOSED** at exact `e021b27cffe4a87de185757a8d47b8685f462bb6` with a real bounded release-after-observation handshake and persisted exact-tree provenance.
- **R9-10B remains closed** at exact `70e87f086b0bbd2d13cd9bae073cb1cdb4c5abd4`; **R9-10C remains independently bounded no-finding closed** at reviewer exact `a1de16f5568951fdc5e99846436f918ed8d99ed2`.
- **H-R9-075/H-R9-076 remain closed**: SessionRuntime rejects forward-gap DeliveryAck atomically, and resumed-session fixtures apply earlier logical proof before later proof/confirm.
- R9-7 remains independently bounded no-finding closed at `cdac663e4d2649c8f87fc2263766616dc9b4bbe9`; R9-8 at `7c87ac675a381154ae7fceca987e7f2f106c26cf`; R9-9 at `b4a527a1fff994ee0836893f850c36eadb609eb1`.
- Candidate A (`Recovery::on_ack` future/never-sent largest) remains closed by the current pre-mutation guard/regressions. Candidate B (`record_datagrams` mixed queue/terminal projection) remains closed by mixed-delta/reason regressions.
- `READY_LIVE: none`. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track absent a new code/instrumentation/hypothesis/path condition producing a concrete unresolved real-network question.
- Release item 3 and item 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- No reviewer-local test execution is claimed for this pass. Repository/source truth came from the GitHub repository API/connector. Developer-local exact-tree gates remain first-class; hosted CI is supplemental.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency. **H-R9-079 is the front correctness/resource-lifecycle HIGH; close it before widening downstream review.** After closing it, continue immediately through the preserved queue below without waiting for another reviewer pass.

## READY_LOCAL 1 — FRONT HIGH: H-R9-079 SessionRuntime terminal stream/timer ownership

Independent finding: exact `26462bb31617486b793ee7d90e8dcf5176552bc9`, `docs/reviews/independent-r9-11-session-terminal-stream-timer-ownership-ee62338-20260920.md`.

Exact-current owner facts:

- `clear_runtime_state()` clears send/recv queues, receive dedup, confirmed watermarks, per-stream/session window accounting and queued bytes;
- it does **not** clear `streams: BTreeMap<StreamId, RuntimeStream>` or `close_deadline_ms: Option<u64>`;
- the function comment says every runtime-owned in-progress state is released and explicitly names only `total_bytes`, `last_activity_ms`, `events`, `next_event`, and `cancelled` as intentional lifetime/cumulative survivors;
- `idle_timeout_releases_all_runtime_owned_state`, `close_deadline_releases_all_runtime_owned_state`, and the cancel regression do not assert stream/timer release.

Smallest accepted repair:

1. release stream ownership and disarm the close deadline on terminalization (`streams.clear()` and `close_deadline_ms = None`, or the smallest equivalent preserving committed semantics);
2. focused deterministic regressions must cover cancel/Error, remote close, idle timeout, and graceful-close deadline, with a stream populated before terminalization and an armed deadline proven before expiry on the deadline path;
3. after terminalization prove streams empty, deadline absent, existing queue/dedup/watermark/window cleanup still true, intended lifetime facts still survive, and only the expected terminal evidence appears;
4. preserve H-R9-078 idempotence and ordinary post-terminal fail-closed behavior; do not invent a new error code or select a `SessionRuntime.events` capacity/TTL/LRU/history policy;
5. focused `neko-session` tests, then final pushed-SHA developer-local exact-tree gate: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, persisted exact GitHub-resolvable SHA + UTC start/end + exit codes + OS/arch + stable Rust. No decoder/parser/crypto-framing change => no mechanical fuzz requirement.

After repair/provenance, continue directly to R9-11A remainder.

## READY_LOCAL 2 — R9-11A SessionRuntime terminal cleanup remainder

On the repaired exact tree, independently re-challenge remote close, cancel, idle timeout and graceful-close deadline as separate terminal paths:

- every post-Closed/post-Error mutating API is fail-closed or explicitly idempotent by committed semantics;
- rejected post-terminal calls append no false success/delivery/window evidence;
- stream state/timer ownership, queues, dedup, watermarks, flow-control counters and queued-byte ownership are terminal-clean;
- lifetime facts remain only where comments/tests intentionally define them;
- `SessionRuntime.events` retained-state capacity remains `POLICY_BLOCKED_RESOURCE_BOUND`; do not invent a number.

Concrete defect -> smallest repair + positive/negative regression + exact-tree gate. No defect -> scope-precise independent bounded no-finding note and continue immediately.

## READY_LOCAL 3 — R9-11B Recovery / reliable-UDP terminal ownership re-challenge

Independently challenge combined terminal state after H-R9-077:

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

After coherent R9-11 repair/review stabilizes, run/persist developer-local provenance for the final pushed developer source/test SHA: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`; `git diff --check`; clean tree; exact pushed SHA; UTC start/end; exit codes; OS/arch; stable Rust. Run pinned decode fuzz only if the coherent group actually touches wire decoder/parser/crypto framing. Hosted CI is supplemental.

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
