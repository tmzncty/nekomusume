# ChatGPT reviewer handoff — H-R9-077 closed at e3dca29; R9-11 lifecycle continues

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `e3dca29babfaf330931f26aa3697aa3cd3c98545` (`fix(carrier): H-R9-077 teardown releases receiver ACK ownership too`), on top of `e021b27cffe4a87de185757a8d47b8685f462bb6`.
- Latest developer bounded review support reviewed: exact `15c0d9f1f15fcbc2f40039d7622bfdce9a33fb26`, `docs/reviews/dev-r9-10c-replay-cleanup-20260919.md`.
- Latest independent reviewer support: exact `877f574e5afa48fc2a3e2396957f37aca47fdf80`, `docs/reviews/independent-r9-11-ack-teardown-ff8f3f7-20260920.md`.
- **H-R9-077 closed at e3dca29:** `ReliableUdpRuntime::teardown()` now replaces `PacketAckTracker` with a fresh empty tracker for the same generation — observed `ranges`, `largest_observed` and `pending_ack` do not survive terminalization, matching the sender-side quiesce. `poll_outgoing_ack` was already `torn_down`-gated; ownership now matches.
- Developer-local clean exact-tree provenance for exact `e3dca29`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-19T19:50:17Z → 2026-09-19T19:55:14Z, Linux x86_64, rustc 1.98.0. This remains developer-reported local evidence; reviewer-local execution is not claimed.
- **Sampler determinism HIGH remains fully CLOSED** at exact `e021b27`: the child owns five `/dev/null` FDs plus a loopback listener before readiness and waits for `sampler.release`; the sampler writes release only after real `/proc` observation reaches the requested FD minimum and sees an owned socket. Developer-local exact-tree gate is persistently recorded for `e021b27`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-19T17:49:54Z → 17:54:54Z, Linux x86_64, rustc 1.98.0. GitHub-hosted Rust CI run `35459239845` was success and remains supplemental only. The old docs-only `df02bd0` closure is superseded.
- **R9-10B remains closed** at source/test exact `70e87f086b0bbd2d13cd9bae073cb1cdb4c5abd4`: automatic-health TCP replay constructs each replay record from the authoritative retained `(DataId, bytes)` returned by `FailoverController::tcp_resend()` rather than an equal-count positional slice.
- **R9-10C remains independently bounded no-finding closed** at reviewer exact `a1de16f5568951fdc5e99846436f918ed8d99ed2` over source/test exact `70e87f086b0bbd2d13cd9bae073cb1cdb4c5abd4`. Full lifecycle/resource terminalization remains R9-11.
- Repository truth for duplicate semantics remains: repeated `FailoverController::confirm` is typed `NotFound`; exact duplicate **receive** is the idempotent `Ok(false)` case.
- **H-R9-075 remains closed:** `SessionRuntime::delivery_ack` rejects a forward gap before watermark/in-flight/event mutation, with atomic negative coverage.
- **H-R9-076 remains closed:** the resumed-session fixture applies already-delivered offset-0 Session proof before later resumed proofs and confirms retained identity only after successful proof application.
- R9-7 remains independently bounded no-finding closed at `cdac663e4d2649c8f87fc2263766616dc9b4bbe9` over source/test exact `d97a536`; R9-8 at `7c87ac675a381154ae7fceca987e7f2f106c26cf`; R9-9 at `b4a527a1fff994ee0836893f850c36eadb609eb1`.
- Candidate A (`Recovery::on_ack` future/never-sent largest) remains closed by the current guard before RTT/loss/PTO mutation and dedicated regressions. Candidate B (`record_datagrams` mixed queue/terminal drop projection) remains closed by current mixed-delta regressions and separate reason projection.
- `READY_LIVE: none`. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track absent a new code/instrumentation/hypothesis/path condition producing a concrete unresolved real-network question.
- Release item 3 and item 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- No reviewer-local test execution is claimed for this pass. Developer-local clean exact-tree gates remain first-class; hosted CI remains supplemental.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency. **While H-R9-077 is open, it is the front correctness/resource HIGH and downstream implementation/review must not expand the error surface.** After closing it, continue immediately through the preserved queue below without waiting for the next reviewer pass.

## READY_LOCAL 1 — FRONT HIGH: H-R9-077 receiver ACK ownership teardown repair

Independent finding: `877f574e5afa48fc2a3e2396957f37aca47fdf80`, `docs/reviews/independent-r9-11-ack-teardown-ff8f3f7-20260920.md`.

Exact owner facts on the reviewed tree:

- `PacketAckTracker` owns bounded `ranges`, `largest_observed` and `pending_ack` for one path generation.
- `ReliableUdpRuntime::on_packet_received(..., true)` populates that state.
- `ReliableUdpRuntime::teardown()` currently clears `packet_frames`, retransmit plaintext and `PathRecovery` live ownership, then sets `torn_down`, but does not clear/reset `acks`.
- `poll_outgoing_ack()` after teardown returns `None` due to `torn_down`; this prevents wire emission but does not release the generation-scoped receiver state.

Smallest accepted repair shape:

1. add a deterministic ACK-tracker quiesce/reset that releases ranges/observation/pending state, or replace the tracker with a fresh empty tracker for the same generation;
2. invoke it from `ReliableUdpRuntime::teardown()` with the existing sender-side quiesce;
3. regression must create multiple observed ranges plus a pending ACK before teardown and prove after teardown that ACK observations/ranges are empty, largest observation is absent, pending is false, and `poll_outgoing_ack()` is `None`;
4. keep the existing sender-side assertions: Recovery in-flight zero, Reno bytes zero, retained plaintext zero;
5. preserve approved lifetime Recovery diagnostics exactly as H-R9-066 requires;
6. avoid adding a production inspection API solely for tests when a crate-private/test-only oracle or direct tracker invariant suffices;
7. no change to ACK encoding/architecture, Session delivery semantics, D019, limits or release policy.

Run focused carrier tests, then on the final pushed source/test SHA run and persist the clean exact-tree developer gate:

`PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`

`git diff --check`

Record exact GitHub-resolvable pushed SHA, UTC start/end, exit codes, clean-tree state, OS/arch and stable Rust version. No decoder/parser/crypto framing change is required by this repair, so do not mechanically run fuzz.

After the repair and provenance are pushed, continue directly to R9-11A.

## READY_LOCAL 2 — R9-11A SessionRuntime terminal cleanup and post-terminal mutation

Read exact-current `SessionRuntime` owners/tests and challenge remote close, cancel, idle timeout and graceful-close deadline as separate terminal paths.

Required checks:

- send/recv queues, dedup history, confirmed watermarks, per-stream/session send/receive window accounting and queued-byte ownership are released on every terminal path that owns them;
- post-Closed and post-Error mutating APIs fail closed or are explicitly idempotent by committed semantics;
- rejected post-terminal calls do not append false success/delivery/window evidence;
- approved lifetime facts (`total_bytes`, historical event log, sequence/history fields) may remain only where current comments/tests intentionally define them as lifetime diagnostics;
- do not invent a new event-history capacity or D019-like retention policy. The known `SessionRuntime.events` retained-state capacity question remains a policy-blocked resource-bound item, not a number for this lane to choose.

Concrete defect -> smallest repair + positive/negative regression + gate. No defect -> scope-precise independent bounded no-finding note and continue immediately.

## READY_LOCAL 3 — R9-11B Recovery / reliable-UDP terminal ownership re-challenge

After H-R9-077, independently re-challenge the combined terminal state of `PathRecovery`, `ReliableUdpRuntime`, `RetransmitBuffer`, packet->frame ownership and receiver ACK tracking:

- live sent map/outstanding copies/Reno charge/retransmit plaintext/packet-frame map/ACK obligations are zero or otherwise explicitly terminal-inert;
- quiesce preserves only documented lifetime history, never stale retransmission eligibility or fresh health outcome;
- post-teardown send/retransmit/receive/apply-ACK/PTO/health/control-plane mutators cannot manufacture fresh positive evidence;
- abort/teardown ordering cannot orphan retained plaintext or make an aborted packet ACK-valid;
- no regression of future/never-sent ACK fail-closed behavior.

No finding -> independent bounded note; finding -> smallest repair.

## READY_LOCAL 4 — R9-11C retained Session replay and Carrier-generation terminality

Challenge `FailoverController`, `ConcurrentCarrierManager`, retained `DataId`/bytes ownership and migration-back state across path failure/drain/terminalization.

Invariants:

- unresolved retained Session replay is **not** silently erased merely because UDP/path recovery terminates;
- confirmed replay identities are removed only after the required Session logical proof;
- failed/retired path generations cannot become active or receive new ownership without a fresh legal generation/transition;
- migration-back does not resurrect stale failed-generation ownership;
- Carrier packet feedback never substitutes for Session confirmation.

Do not change D064 single-active/warm semantics or replay retention policy values.

## READY_LOCAL 5 — R9-11D executable process/socket lifecycle and result truth

Challenge automatic-health failover/migration-back executable paths plus TCP/UDP process fixtures:

- success, explicit failure, timeout, shutdown and post-return terminal paths release sockets/process-owned resources they no longer own;
- listener/process cleanup is proved by deterministic owner/socket/process evidence, not inferred from a zero exit code;
- service lifecycle transitions cannot create a false READY/success result after a terminal failure/stop;
- shutdown/result JSON/human output and exit status remain mutually consistent;
- cleanup failure must remain visible as failure/unknown, not be promoted to success.

Do not open a live lane unless this local review produces a new real-network-only question.

## READY_LOCAL 6 — R9-12 final exact-tree provenance

After the coherent R9-11 repair/review group is stable, run and persist developer-local provenance for the final pushed developer source/test SHA:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- `git diff --check`
- clean tree
- exact GitHub-resolvable pushed SHA, UTC start/end, exit codes, OS/arch and stable Rust version

Run pinned decode fuzz only if the coherent group actually touched wire decoder/parser/crypto framing. Hosted CI is supplemental cross-evidence.

## READY_LOCAL 7 — dedicated final independent R9 bounded review

On the final R9 exact source/test tree, independently re-challenge the combined invariants across Recovery, receiver ACK ownership, Session logical proof, Carrier readiness/promotion, uncertain retention, TCP replay/dedup, migration-back, process result truth and terminal cleanup. A no-finding bounded review is valid item-4 support. Any concrete defect converts immediately to the smallest repair lane; do not claim R9 closure first.

## READY_LOCAL 8 — Q10 factual reconciliation

Reconcile release packet/status claims against exact reachable R9 review/provenance anchors. Update only factual indexes/boundaries that actually changed. Do not mark release item 4 complete and do not promote local evidence into WAN/security/release claims.

## READY_LOCAL 9 — Q11 release-evidence boundary reconciliation

Recheck item 3/WAN rows, historical negatives and `READY_LIVE`. Current authoritative classification remains `READY_LIVE: none`; only new code/instrumentation/hypothesis/path conditions creating a concrete unresolved self-owned real-network question may open a live lane. VPS rental priority does not justify repeating a closed/blocked same-class experiment.

## READY_LOCAL 10 — Q12 governance/release-state reconciliation

Confirm D019 and other policy/value gates remain separate and RC/production/freeze/release authority has not been inferred from engineering completion. Do not select TTL/LRU/history/capacity/security numbers, signing/key-custody/SBOM/publication policy, destructive migration, frozen-release policy or release authority.

## READY_LOCAL 11 — repository-wide item-4 refill

Item 4 remains incomplete. Repeat the broad core-surface inventory rather than declaring queue exhaustion from one R9 sweep. Prefer implemented core surfaces lacking a dedicated reachable independent bounded challenge on the current/relevant tree, or whose semantics changed since the previous challenge:

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
13. release-packet factual consistency and evidence boundaries.

No-finding independent review is valid item-4 support. Do not count one narrow no-finding as repository-wide exhaustion.

## READY_LOCAL 12 — conditional live question only

Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved self-owned real-network question, classify it against `docs/standing-vps-lab-authorization.md` and `docs/vps-rental-window-priority.md`. Ordinary bounded self-owned TCP/UDP work inside standing authorization needs no per-run approval. Do not manufacture live work merely because the VPS is rented.

## Stop / escalation conditions

Continue implementation/review -> tests -> commit -> push -> next dependency-ready slice without waiting for reviewer cadence. Stop and escalate only for an unresolved BLOCKER/HIGH that cannot be auto-decided, core Session/Carrier/ACK/crypto/wire architecture change, D019 or other policy/value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
