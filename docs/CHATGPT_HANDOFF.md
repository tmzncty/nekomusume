# ChatGPT reviewer handoff — H-R9-080 REOPENED: mutation-sensitive SessionRuntime terminal oracles

## Current repository truth

- Current developer handoff anchor reviewed this pass: reachable `main` exact `15dd77d16a0975b6e150f24794de9ce7aa250314`.
- New developer-owned commits reviewed since the prior reviewer pass:
  - `8c3f96de1a0de3c102c09cb2a0ff87b6971dce0a` — strengthens cancel/remote-close terminal cleanup tests with a shared populated fixture;
  - `15dd77d16a0975b6e150f24794de9ce7aa250314` — developer handoff claiming H-R9-080 closed at `8c3f96d`.
- Latest independent reviewer support: exact `ea7040b1ecd5d6b17625eec5985356b4c3b7eea1`, `docs/reviews/independent-r9-11-session-terminal-oracle-reopen-8c3f96d-20260920.md`.
- **H-R9-079 source repair remains accepted** at exact `271e512c01cefcfc40745aa41725bbd3e2585bb6`: `SessionRuntime::clear_runtime_state()` clears queues, receive dedup, confirmation watermarks, per-stream/session flow-control accounting, queued bytes, stream ownership, and `close_deadline_ms` while retaining documented lifetime facts.
- **H-R9-080 is CLOSED** at exact `fea90f4808c5de757b9a371a7e0465472035acb5` (on top of `eeec59f5b2f877ebb1dc2b9b1b569d5fc513b324`): `populated_runtime()` queues two sends, ACKs only the first — `confirmed` is non-empty while the second remains positively in-flight. `session_send_inflight > 0`, `send_inflight[stream] > 0`, `session_recv_window_used > 0`, `recv_window_used[stream] > 0` are asserted separately (not OR/map-presence); `queued_bytes > 0`; `close_deadline_ms` armed. Repeated terminal calls are event-count idempotent; a rejected post-terminal mutator adds no fresh evidence. Developer-local clean exact-tree provenance for exact `fea90f4`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-20T00:51:06Z → 2026-09-20T00:56:05Z, Linux x86_64, rustc 1.98.0.
- Developer-local clean exact-tree provenance recorded for exact `8c3f96d`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0; `git diff --check` exit 0; clean worktree; 2026-09-19T23:50:06Z -> 2026-09-19T23:55:13Z; Linux x86_64; rustc 1.98.0. Treat this as developer-reported local provenance, not reviewer-local execution. GitHub commit-workflow lookup returned no PR-triggered workflow run for exact `8c3f96d`; do not promote that absence into a failure claim.
- **H-R9-077 remains CLOSED** at exact `e3dca29babfaf330931f26aa3697aa3cd3c98545`: reliable-UDP teardown releases receiver ACK ownership as well as sender recovery state.
- **H-R9-078 remains CLOSED** at exact `442a8058e4ae27f867bd27db632494a945ec30f2`: repeated cancel on terminal Error is idempotent and the focused regression is active.
- Sampler determinism HIGH remains closed at exact `e021b27cffe4a87de185757a8d47b8685f462bb6` with bounded release-after-observation handshake and persisted exact-tree provenance.
- H-R9-075/H-R9-076 remain closed; R9-7/R9-8/R9-9/R9-10B/R9-10C remain at their prior accepted reachable anchors.
- Candidate A (`Recovery::on_ack` future/never-sent largest) remains closed by the current pre-mutation guard/regressions. Candidate B (`record_datagrams` mixed queue/terminal projection) remains closed by mixed-delta/reason regressions.
- `READY_LIVE: none`. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD, or Experimental Track absent a new code/instrumentation/hypothesis/path condition producing a concrete unresolved real-network question.
- Release item 3 and item 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- No reviewer-local test execution is claimed for this pass. Repository/source truth came from the GitHub repository connector. Developer-local exact-tree gates remain first-class; hosted CI is supplemental.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency. Because H-R9-080 is a current HIGH, close it before widening R9-11.

## READY_LOCAL 1 — FRONT HIGH: H-R9-080 positive send-accounting + survivor + cancel negative oracle

Independent finding: exact `ea7040b1ecd5d6b17625eec5985356b4c3b7eea1`, `docs/reviews/independent-r9-11-session-terminal-oracle-reopen-8c3f96d-20260920.md`.

Keep the current shared source cleanup. Repair only the oracle strength.

Use one bounded fixture for both cancel and remote close that proves every owned surface is genuinely live before terminalization:

1. open stream 1;
2. `queue_send("aa")`, then `queue_send("bb")`;
3. `pop_send()` exactly one record;
4. `delivery_ack(stream=1, offset=0, len=2)` so `confirmed` is non-empty while the second two-byte send remains positively in-flight;
5. `receive(offset=0, "xy")` so `recv`, `received`, `recv_window_used` and session receive-window ownership are non-empty;
6. `close_graceful()` so `close_deadline_ms` is actually armed;
7. before terminalization assert the relevant preconditions **separately**, not through OR/map-presence shortcuts: non-empty `streams`, `send`, `recv`, `received`, `confirmed`; `send_inflight[stream] > 0`; `session_send_inflight > 0`; `recv_window_used[stream] > 0`; `session_recv_window_used > 0`; `queued_bytes > 0`; armed deadline; snapshot `total_bytes` or another already-documented lifetime survivor.

Then for **first cancel/Error** and **remote close** separately prove:

- `streams`, `send`, `recv`, `received`, `confirmed`, `send_inflight`, `recv_window_used` are empty;
- `close_deadline_ms.is_none()`;
- `session_send_inflight == 0`, `session_recv_window_used == 0`, `queued_bytes == 0`;
- the documented lifetime survivor remains unchanged;
- exactly one correct terminal event is added;
- repeated cancel/remote close leaves observable-event count unchanged;
- a representative rejected post-terminal mutator on **both** paths cannot append fresh success/delivery/window evidence.

Keep idle-timeout and close-deadline controls green. Do not invent an error code, lifecycle semantic, event-retention cap, capacity/TTL/LRU/history value, or D019 policy. No decoder/parser/crypto-framing change => no mechanical fuzz requirement.

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
