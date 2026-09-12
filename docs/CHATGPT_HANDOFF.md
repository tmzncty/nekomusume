# ChatGPT reviewer handoff — confirmed recovery/observability repairs, then continue core item-4 review

## Reviewed repository truth

- Default branch before this refresh: exact `5cde95ce2fd54e87c1c79f83156e74bf0d46f10e` (`docs(review): confirm recovery and observability correctness findings`).
- Previous reviewer queue-opening handoff: exact `e6a03776c20c5b32b4676d992cf7fcc613cff2e2`.
- No developer-owned source/test commit has landed since that handoff; current source/test anchor remains reachable exact `4034f86ceb356a3e04b8ff713b97338af3e71265`.
- Exact `5cde95c` adds reviewer-only independent static/source review support. It confirms two bounded MEDIUM findings against exact `e6a0377`; it does not itself repair implementation or claim a developer-local test gate.
- Governance is unchanged: item 3 incomplete, item 4 incomplete, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, D019 policy-blocked, `READY_LIVE: none`.
- No new VPS/WAN execution is authorized or needed by these findings.

## Reviewer verdict

The high-throughput queue remains valid, but the first two candidates are no longer merely speculative. Exact reviewer note [`docs/reviews/independent-recovery-observability-review-e6a0377-20260913.md`](reviews/independent-recovery-observability-review-e6a0377-20260913.md) confirms both as current-semantics defects.

These are MEDIUM, not BLOCKER/HIGH, so they do not suspend the whole repository. They should nevertheless be repaired first because each can corrupt local recovery/evidence conclusions used by later review surfaces.

The external agent is pre-authorized to consume this queue continuously. Finish a coherent slice -> focused tests -> commit/push -> exact pushed-tree local gate/provenance -> immediately continue to the next dependency-safe slice. Do not wait for reviewer cadence.

# MUST_EXECUTE_LOCAL 1 — reject strictly-future ACK atomically

Owners:

- `crates/neko-reliable/src/lib.rs`, especially `Recovery::on_ack`;
- `docs/spec/m2-udp-recovery.md` only if factual clarification is required;
- direct `neko-reliable` tests.

Confirmed defect:

`Recovery::on_ack` uses peer-provided `ack.largest()` as packet-threshold history before proving that the largest ACKed packet number could have been sent by this `Recovery` instance. A strictly-future ACK can therefore make real in-flight packets satisfy the packet-threshold loss rule, remove them from `sent`, and expose their FrameIds for retransmission.

Required repair contract:

1. Add the regression first.
2. A nonempty ACK when no packet has ever been sent must fail closed.
3. `largest > largest_sent` must fail closed before RTT/loss/PTO/sent-state mutation.
4. Rejection must be atomic: no RTT update, no packet removal, no lost/retransmit output, no PTO reset, no outstanding-frame mutation.
5. Do **not** require every ACKed number to remain in the current `sent` map; already-retired/duplicate ACK history is a separate valid case.
6. Do not redesign ACK wire format, packet numbering, loss detection or error taxonomy. Reuse an existing error if adequate.

Validation:

- focused `neko-reliable` tests including future ACK and duplicate/retired ACK behavior;
- full `cargo test -p neko-reliable` (or equivalent package target);
- commit/push coherent repair;
- on the exact pushed developer SHA in a clean checkout/worktree: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean-tree confirmation;
- persist concise sanitized provenance with exact reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust version.

No fuzz is required unless the repair unexpectedly changes an external decoder/framing path.

After green closure, immediately continue to MUST_EXECUTE_LOCAL 2.

# MUST_EXECUTE_LOCAL 2 — preserve mixed datagram-drop reasons in observability

Owners:

- `crates/neko-session/src/lib.rs`, `DatagramRuntime` / `DatagramCounters` only to understand current counter semantics;
- `crates/neko-observe/src/lib.rs`, `Producer::record_datagrams`;
- direct `neko-observe` / `neko-session` tests as needed.

Confirmed defect:

Current counter semantics distinguish:

- terminal/post-close generic drop -> increments `dropped` only;
- queue-full drop -> increments both `queue_dropped` and `dropped`;
- oversize -> increments `rejected_oversize` and is separately projected.

Therefore `queue_dropped` is a subset of generic `dropped`. `record_datagrams` currently treats `queue_delta > 0` as a boolean reason for **every** generic dropped event in that observation window, so a mixed window can report terminal drops as `queue_full`.

Required repair contract:

1. Add a mixed-delta regression first: at least one queue-full generic drop plus one terminal generic drop between observations.
2. Preserve the exact total number of generic dropped events.
3. Emit `queue_full` only for the queue-drop subset; classify the remaining generic dropped delta as terminal under current `DatagramRuntime` semantics.
4. Keep oversize projection separate and unchanged unless a concrete independent defect is found.
5. If externally supplied counters are internally inconsistent (for example queue delta exceeds generic dropped delta), fail closed or project conservatively without inventing a new transport state/policy.
6. Do not change `DatagramRuntime` transport semantics or counter definitions merely to simplify the observer.

Validation:

- focused mixed-reason and normal single-reason observer tests;
- package tests for `neko-observe` and any directly affected Session tests;
- commit/push coherent repair;
- exact pushed-tree clean local full gate + `git diff --check` + provenance.

No decoder fuzz is required for an observability-only repair.

After green closure, immediately continue to REVIEW_LOCAL 3.

# REVIEW_LOCAL 3 — independent bounded UDP recovery review

Owners:

- `crates/neko-reliable/src/lib.rs`;
- `docs/spec/m2-udp-recovery.md`;
- direct tests.

Challenge the rest of the already-committed recovery contract after finding 1 is repaired:

- ACK range canonicalization and atomic bound rejection;
- packet-number exhaustion and monotonicity;
- duplicate/already-retired ACK interaction;
- `outstanding_frames` copy counts across original + retransmission overlap;
- packet/time-threshold loss and reorder boundary;
- RTT/ACK-delay arithmetic and mutation ordering;
- PTO backoff/saturation and the rule that PTO itself does not declare loss;
- persistent-congestion threshold/accounting;
- Reno bytes-in-flight/cwnd/ssthresh accounting under ack/loss ordering and saturation;
- deterministic fault simulation termination/result accounting.

If a concrete defect exists and current semantics decide the answer: smallest repair + regression + exact-tree closure, then continue. If no defect: write a bounded independent no-finding note with exact reachable anchor, commands/tests and exclusions. No performance/WAN/security-approval claim.

# REVIEW_LOCAL 4 — CarrierState generation/validation/hysteresis

Owners:

- `crates/neko-carrier/src/lib.rs` `CarrierState`, `CarrierEvent`, `PathRecord`;
- `docs/spec/m0-carrier-state.md`;
- current Carrier/Session architecture references.

Challenge:

- packet feedback cannot validate a path or create Session delivery;
- old/future generation errors are mutation-safe;
- activation requires validation + existing hysteresis and cannot create two active owners;
- active/degraded/draining/failed transitions preserve single-active ownership;
- unrelated successful events do not accidentally satisfy a path-specific validation/readiness invariant beyond the semantics explicitly committed for `dwell_events`;
- path/resource error ordering is deterministic and non-mutating;
- `active_epoch` advances only on accepted activation.

Do not change D064 constants/policy or architecture. Defect -> repair; no finding -> independent note.

# REVIEW_LOCAL 5 — Concurrent Carrier Manager / health / migration-back

Identify exact-current owners for `ConcurrentCarrierManager`, `ConcurrentPathKey`, health scoring/samples, switch events/reasons, hold/cooldown and migration-back tests/specs.

Challenge current committed semantics:

- single-active, multi-ready; no application striping;
- warm path cannot carry new application data before promotion;
- health/path feedback cannot masquerade as Session delivery;
- failed or generation-stale paths cannot be promoted;
- voluntary switch hysteresis/cooldown/hold is generation-scoped and monotonic;
- migration-back cannot bypass validation or active-owner handoff;
- switch/failure event timestamps and reasons match actual state transitions.

No new scoring constants/architecture.

# REVIEW_LOCAL 6 — FairScheduler / multistream / flow-control accounting

Identify exact owners for `FairScheduler`, `StreamPriority`, stream/session queue/window accounting and multistream tests.

Challenge:

- interactive/bulk fairness and starvation guard;
- per-stream/session byte/window caps;
- enqueue/dequeue accounting on rejection, close, reset and partial progress;
- stream removal/close cannot leak queued-byte/open-stream accounting;
- large/bulk streams cannot bypass current limits or contradict current anti-starvation claims;
- overflow/saturation cannot promote capacity or delivery evidence.

Do not redesign the scheduler. Property-style tests are welcome only when tied to a concrete invariant.

# REVIEW_LOCAL 7 — carrier adapter close/error/resource semantics

Owners: current Memory/UDP/TCP carrier adapters and direct tests.

Challenge:

- existing message/buffer limits before allocation/enqueue where applicable;
- claimed idempotent close and queued-data drain semantics;
- local-close/peer-close/I/O/poison/truncation mappings cannot become Session/path evidence;
- native TCP/UDP behavior is not normalized into a stronger common guarantee than implemented;
- failure paths are mutation/accounting safe.

No async/runtime framework refactor.

# REVIEW_LOCAL 8 — SessionRuntime lifecycle/resource/DeliveryAck accounting

Owners:

- `crates/neko-session/src/lib.rs` `SessionRuntime`, limits, stream/window/inflight/queue accounting;
- direct runtime/process tests;
- current Session/M3 specs.

Challenge:

- hard limits before mutation;
- stream/session window accounting released exactly once by intended transitions;
- duplicate/overlap data does not consume unbounded retained state or create delivery evidence;
- close/cancel/error/idle/deadline paths leave counters/state consistent;
- rejected transitions are atomic;
- `DeliveryAck` cannot acknowledge invalid/unsent logical ranges or become application `delivered/effect` evidence;
- queue/total-byte counters remain consistent after reset/close/failure.

Do not redesign Session ACK architecture.

# REVIEW_LOCAL 9 — observability projection full bounded review

After finding 2 is repaired, inspect the rest of `neko-observe` and its source projections:

- ring capacity/drop accounting;
- sequence overflow/fail-closed behavior;
- timestamp/ordering claims;
- recovery loss/retransmit/PTO counters versus source results;
- scheduler high-water/starvation counters;
- secret-free fixed-vocabulary correlation/data;
- switch completed/failed shape versus manager outcome.

Observer defects may repair projection/tests only; do not change transport semantics to make metrics easier.

# REVIEW_LOCAL 10 — package/reproducibility/operator implementation

Independent static/deterministic review of current release/package scripts and tests; do **not** repeat VPS rehearsals.

Challenge clean-source enforcement, archive path/type/root/mode/checksum validation timing, reproducibility inputs/source identity, dedicated install-path containment, external state/identity retention boundaries, rollback binary selection and cleanup observation ordering.

Do not invent signing/key-custody/SBOM/publication policy.

# REVIEW_LOCAL 11 — dependency/build surface

The current dependency safety note is developer-run factual evidence, not independent approval.

Independently inspect manifests, lockfile, enabled/default features, build scripts/native hooks, security-sensitive direct dependencies and project `unsafe_code = "forbid"` inheritance.

Catch executable graph/feature drift or unexpected native/build-time paths. Distinguish project-source unsafe policy from dependency internals. Do not auto-upgrade, add a scanner framework, or make vulnerability claims without authoritative evidence.

# REVIEW_LOCAL 12 — cross-platform CLI/process semantics

Review supported-platform assumptions in current CLI/process tests and platform helpers: socket terminal-close semantics, `cfg(unix)`, signals/shutdown/exit codes, temporary-file cleanup, Unix identity-file permission boundaries, invalid-config-before-network/identity ordering, and incidental OS errors versus protocol semantics.

Do not promise unsupported Windows production behavior.

# REVIEW_LOCAL 13 — CLI machine/human/exit contract

Across shipped bounded CLI surfaces, verify invalid arguments/config, completed negative experiments, successful results, machine JSON, human output and exit codes cannot contradict each other or emit success-shaped artifacts before evidence exists. Preserve secret/private-topology boundaries. No new CLI framework.

# REVIEW_LOCAL 14 — algorithmic resource boundedness, not pressure testing

Cross-check existing Session, Carrier, reliable, pre-auth and observability hard ceilings/lifecycle releases. This is not RSEC-001 capacity benchmarking and must not select new capacity/security values.

If an existing committed cap is bypassed, repair it. If a missing cap requires choosing a new value/policy, stop only that finding and classify it for maintainer judgment.

# REVIEW_SUPPORT 15 — factual reconciliation

After the two repairs plus each coherent group of roughly 3-4 review slices, reconcile only established facts into `docs/release-security-review-packet.md`, `docs/reviews/release-item4-subgates-20260909.md`, and `docs/status.md` only when an actual status/boundary changes.

Keep developer-local gates, reviewer independent notes, hosted CI, live WAN evidence and performance conclusions distinct. Do not mark item 4 complete merely from bounded engineering review; final maintainer/security judgment remains separate.

## Continuous execution / stop conditions

The agent should not idle merely because one slice is a no-finding review. Continue through the numbered queue while dependencies remain satisfied.

Only stop for unresolved BLOCKER/HIGH, a core Session/Carrier/ACK/crypto/wire architecture choice, destructive/canonical migration, new security-policy values, D019 policy, action outside standing authorization, production/third-party/new-credential action, maintainer-valued pressure/benchmark conditions, repository breakage, actual runtime/tool exhaustion, or genuine broad queue exhaustion.

Standing VPS authorization remains valid, but authoritative live classification remains `READY_LIVE: none`. Do not mechanically rerun HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint migration, key update, IPv6, PLPMTUD or Experimental Track work without a materially new dependency-ready real-network question.
