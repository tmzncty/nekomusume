# ChatGPT reviewer handoff — reopen high-throughput item-4 core review/repair queue

## Reviewed repository truth

- Default branch before this refresh: exact `e7566cdada8826e3f2722750d02474077bc501db` (`docs(handoff): accept proposal sweep and mark queue exhausted`).
- Most recent developer-owned commit before that handoff: exact `217c682cc5345bc82ee93135fc4efaf004845f60` (`docs: record bounded proposal sweep at 79e4443`).
- Most recent source/test anchor remains reachable exact `4034f86ceb356a3e04b8ff713b97338af3e71265` (`test: inline rejected negotiation close assertion`).
- Provenance/evidence HIGH is closed; pre-rebase/unreachable local anchors are quarantined and reachable item-4 review support is indexed.
- Current governance remains unchanged: item 3 incomplete, item 4 incomplete, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, D019 policy-blocked, `READY_LIVE: none`.
- No new VPS/WAN question is created by this handoff.

## Reviewer correction — prior queue-exhaustion claim was too narrow

Exact `217c682` performed one bounded **pre-auth proposal sweep** and correctly found no dependency-ready repair in that narrow surface. That result remains accepted.

It does **not** establish that repository-wide item-4 review/support work is exhausted.

The current independent-review directory contains bounded independent work for release evidence indexing, pre-auth/resource controls, HY2 methodology, wire/parser, and one broad checkpoint. It does not contain dedicated independent review coverage of several implemented core surfaces, including the UDP recovery engine, Carrier Manager/scheduler/flow-control path, carrier adapter close/resource semantics, observability projection, Session runtime resource/lifecycle behavior, package/reproducibility implementation, and cross-platform CLI/process behavior.

The maintainer explicitly wants the external agent kept substantially supplied. Reviewer cadence is inspection cadence, not a work quota. Therefore the coding/research agent is pre-authorized to execute the queue below continuously. A bounded review that finds no defect is a valid item-4 support result; a concrete existing-semantics defect immediately becomes a repair slice.

Do not manufacture generalized frameworks, speculative features, new release policy, or live experiments merely to create activity.

## Global execution contract for this queue

For every review slice:

1. Re-read exact-current owner source, tests, applicable spec/ADR/status claims.
2. State the exact invariant/claim being challenged.
3. Try to falsify it with source reasoning and focused deterministic tests.
4. If a concrete defect exists and current committed semantics already determine the answer: smallest repair -> bounded positive/negative regression -> commit/push -> validate the exact pushed SHA locally -> persist concise provenance -> continue.
5. If no defect exists: write one bounded review note with inspected owners, commands/tests, no-finding scope, exclusions, and exact reachable anchor; do not create code churn.
6. A code/test change receives focused tests plus clean exact-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and clean-tree confirmation. Record exact SHA, UTC start/end, exit codes, OS/arch, stable Rust version. GitHub-hosted CI is optional cross-evidence only.
7. Wire decoder/parser/framing source changes require pinned decode fuzz smoke. Review-only/docs-only work does not.
8. Do not wait for reviewer cadence between dependency-safe slices.

Only stop for unresolved BLOCKER/HIGH, core Session/Carrier/ACK/crypto/wire architecture choice, destructive/canonical migration, new security-policy values, D019 policy, action outside standing authorization, production/third-party/new-credential action, maintainer-valued pressure/benchmark conditions, repository breakage, actual runtime/tool exhaustion, or genuine exhaustion after the queue below is consumed/reclassified.

# High-throughput rolling queue

## READY_LOCAL 1 — future/unsent ACK poisoning in `neko-reliable`

Owners:

- `crates/neko-reliable/src/lib.rs`
- `docs/spec/m2-udp-recovery.md`

Concrete candidate from reviewer source inspection:

`Recovery::on_ack` obtains `largest = ack.largest()` and uses that value for packet-threshold loss detection. Current code does not first reject a `largest` packet number greater than `largest_sent`. A peer-supplied ACK for a strictly future/never-sent packet may therefore make real in-flight packets satisfy `largest - n >= PACKET_THRESHOLD`, producing false loss/retransmit evidence.

First write a focused regression proving the current behavior or disproving the concern. Protected invariant: ACK input must not create loss/retransmission evidence from a packet number the sender could not have sent.

Bounds:

- do not redesign ACK wire format;
- do not require every ACKed number to remain in the current `sent` map because valid already-retired ACK history may exist;
- the minimum obvious invalid boundary is `largest > largest_sent`;
- rejection must be atomic: no RTT mutation, no packet removal, no retransmit/lost output, no PTO reset;
- choose an existing error variant if its semantics fit; do not add protocol taxonomy solely for cosmetics.

If confirmed, classify at least correctness MEDIUM and close with smallest repair + regression. Continue immediately to slice 2.

## READY_LOCAL 2 — independent bounded UDP recovery review

Owners:

- `crates/neko-reliable/src/lib.rs`
- `docs/spec/m2-udp-recovery.md`
- direct tests only.

Challenge:

- ACK range canonicalization and atomic bound rejection;
- packet-number exhaustion and monotonicity;
- ACK/loss interaction after packet retirement;
- `outstanding_frames` copy counts across original + retransmission overlap;
- packet/time-threshold loss and reorder boundary;
- RTT/ACK-delay arithmetic, PTO backoff and saturation;
- PTO must not itself declare loss or Session delivery;
- persistent-congestion event threshold/accounting;
- Reno `bytes_in_flight`/cwnd/ssthresh saturation and loss/ack ordering;
- deterministic fault simulation termination and result accounting.

Output `docs/reviews/independent-reliable-udp-<anchor>-20260912.md` (or equivalent stable name). No performance or WAN claim. Any concrete defect -> repair immediately before continuing.

## READY_LOCAL 3 — CarrierState generation/validation/hysteresis review

Owners:

- `crates/neko-carrier/src/lib.rs` `CarrierState`, `CarrierEvent`, `PathRecord`;
- `docs/spec/m0-carrier-state.md`;
- current Carrier/Session architecture/ADR references.

Challenge:

- packet feedback never promotes path validation or Session delivery;
- old/future generation handling is fail-closed and mutation-safe;
- activation requires validated path + hysteresis and cannot create two active owners;
- active/degraded/draining/failed transitions preserve single-active ownership;
- drain/fail/activate ordering agrees with committed D064 semantics;
- unrelated events cannot accidentally satisfy a path-specific readiness/validation invariant;
- resource-limit/path-exists error ordering is deterministic and does not mutate state;
- active epoch advances only on accepted activation.

Do not change D064 policy constants or architecture. Concrete invariant defect -> smallest repair/tests. Otherwise bounded independent note.

## READY_LOCAL 4 — Concurrent Carrier Manager / health / migration-back review

Identify the exact-current owners for `ConcurrentCarrierManager`, `ConcurrentPathKey`, health scoring/samples, switch events/reasons and migration-back hold/cooldown logic in `neko-carrier` plus their direct tests/specs.

Challenge already-committed semantics:

- single-active, multi-ready; no application striping;
- warm path cannot carry new application data before promotion;
- health feedback cannot masquerade as Session delivery;
- failed/generation-stale paths cannot be promoted;
- voluntary switch hysteresis/cooldown/hold is monotonic and generation-scoped;
- migration-back cannot bypass validation or active-owner handoff;
- failure reason/event timestamps do not invert the state transition they describe.

No new carrier architecture or scoring constants. Defect -> repair; no finding -> independent note.

## READY_LOCAL 5 — FairScheduler / multi-stream / flow-control accounting review

Identify exact owners for `FairScheduler`, `StreamPriority`, stream/session queue/window accounting and multistream tests.

Challenge:

- interactive/bulk fairness and starvation guard;
- per-stream and session-wide byte/window caps;
- enqueue/dequeue accounting on rejection, close, reset and partial progress;
- one large/bulk stream cannot silently bypass limits or starve bounded interactive work contrary to current claims;
- stream removal/close cannot leak queued-byte/open-stream accounting;
- integer overflow/saturation cannot promote capacity or delivery evidence.

Do not invent a new scheduler. Add deterministic/property-style tests only when they answer a concrete uncovered invariant.

## READY_LOCAL 6 — carrier adapter close/error/resource review

Owners: current `MemoryEndpoint`, UDP adapter, TCP adapter/framing surfaces in `neko-carrier` and direct tests.

Challenge:

- max-message/max-buffer limits are checked before allocation/enqueue where applicable;
- close is idempotent where claimed;
- peer-close/local-close/error mapping cannot become Session/path evidence;
- queued data drain semantics match each adapter's contract;
- poison/I/O/truncation paths fail closed without hidden queue/accounting mutation;
- UDP/TCP native semantics are not silently normalized into a stronger common guarantee.

No generalized async runtime/refactor. Repair only concrete contradictions.

## READY_LOCAL 7 — SessionRuntime lifecycle/resource/flow-control review

Owners:

- `crates/neko-session/src/lib.rs` `SessionRuntime`, `RuntimeLimits`, stream/window/inflight/queue accounting;
- direct process/runtime tests;
- current Session v0 and M3 specs.

Challenge:

- hard limits validated before state allocation/mutation;
- stream/session window accounting is released exactly once by the intended delivery transition;
- duplicate data cannot consume unbounded retained state or advance delivery evidence;
- close/cancel/error/idle/deadline paths clear or retain state exactly as current contract says;
- rejected transitions are atomic;
- `DeliveryAck` cannot acknowledge unsent/invalid logical ranges or silently create `delivered/effect` evidence;
- queue/total byte counters remain consistent after reset/close/failure.

Do not redesign Session ACK architecture. Concrete defect -> smallest repair/regression.

## READY_LOCAL 8 — observability projection correctness review

Owners:

- `crates/neko-observe/src/lib.rs`;
- observability schemas/docs/tests;
- producer call sites only as needed.

Concrete candidate to investigate first:

`record_datagrams` computes total `dropped` delta and `queue_dropped` delta, but currently chooses `queue_full` for every emitted generic dropped event whenever `queue > 0`. Verify the `DatagramCounters` relationship. If `queue_dropped` is only a subset of total dropped reasons, mixed-reason windows may be misclassified. A review regression should exercise mixed counter deltas before any repair.

Also challenge:

- event ring capacity/drop accounting;
- sequence overflow/fail-closed behavior;
- event timestamps supplied/ordered as claimed;
- recovery retransmit/loss/PTO counters reflect source results without promotion to delivery evidence;
- scheduler high-water and starvation counters are monotonic and not double-counted;
- secret-free correlation/data vocabulary cannot accept arbitrary peer/private text;
- switch success/failure event shape matches actual manager outcome.

Observability fixes must not change transport semantics.

## READY_LOCAL 9 — package/reproducibility/operator independent review

Owners:

- `scripts/release/build-package.sh`;
- clean-source/archive validation scripts/tests;
- package smoke/install/upgrade/rollback helpers;
- package evidence docs only as navigation.

This is an independent static/deterministic review of implementation, not a repeat VPS rehearsal.

Challenge:

- clean-source enforcement timing;
- archive path/type/root/mode/checksum validation before extraction/use;
- reproducibility inputs and source/binary identity;
- dedicated install-path containment;
- external identity/state retention boundaries;
- rollback selects the intended binary/package and does not claim schema compatibility it did not test;
- cleanup/listener/process assertions cannot report success before observation.

Do not invent signing/key-custody/SBOM policy. Those remain external release-policy gates.

## READY_LOCAL 10 — dependency/build-surface independent review

The existing `docs/local-dependency-safety-8d7c147-20260912.md` is developer-run factual evidence, not independent approval.

Independently inspect current `Cargo.toml`, member manifests, `Cargo.lock`, direct security-sensitive dependencies, enabled/default features, build scripts/native hooks and `unsafe_code = "forbid"` inheritance.

Goals:

- catch executable dependency-graph or feature drift versus repository claims;
- identify unexpected native/build-time code or duplicated crypto implementation paths;
- verify locked versions correspond to manifests and current build;
- distinguish project-source `unsafe_code=forbid` from unsafe code inside dependencies.

Do not auto-upgrade dependencies, introduce cargo-deny/a new scanning framework, choose signing/SBOM policy, or make vulnerability claims without authoritative evidence. Concrete contradiction -> smallest manifest/docs/test repair; otherwise bounded independent note.

## READY_LOCAL 11 — cross-platform CLI/process portability review

Owners: `crates/neko-cli/tests`, `crates/neko-cli/src`, platform-specific identity/config/signal/socket helpers.

Review supported-platform assumptions only:

- `cfg(unix)` versus portable code;
- socket terminal-close/error semantics;
- signal/shutdown behavior and exit codes;
- temporary path/file cleanup;
- identity-file permission/security tests limited appropriately to Unix semantics;
- deterministic invalid-config-before-network/identity behavior;
- no platform-specific incidental error code is mistaken for protocol semantics.

Do not promise Windows production support if not claimed. Repair only concrete supported-environment contradictions.

## READY_LOCAL 12 — CLI exit-code / JSON / human-output contract review

Across current bounded CLI surfaces (`probe`, multistream, periodic/failover/reachability/benchmark commands that are actually shipped), verify existing contract consistency:

- invalid arguments/config -> invalid-argument exit class without success artifact;
- completed negative experiment/probe -> failure result, not parse error;
- successful bounded result -> success exit/artifact;
- machine JSON and human output do not contradict each other;
- failure before evidence creation cannot emit a success-shaped artifact;
- secret/private topology boundaries remain respected.

Do not create a new CLI framework. Concrete contradiction -> focused process regression + smallest repair.

## READY_LOCAL 13 — algorithmic resource-bound review without pressure testing

This is **not** RSEC-001 capacity benchmarking and must not select new capacity values.

Cross-check existing hard ceilings in Session, Carrier, reliable recovery, pre-auth and observability for algorithmic boundedness:

- allocations are bounded by already-selected constants/limits;
- loops over attacker-controlled counts terminate within validated bounds;
- retained maps/queues have an existing cap or lifecycle release;
- arithmetic overflow is rejected/saturated in a way consistent with current claims;
- one bounded input cannot create an unbounded number of retained objects/events.

If this finds a missing cap whose value requires policy selection, STOP that individual finding and classify it for maintainer judgment. If an existing committed cap should already apply but is bypassed, repair it.

## REVIEW_SUPPORT 14 — release packet factual reconciliation

After slices 1-13 (or after every coherent group of 3-4 if defects materially change facts), reconcile only established facts into:

- `docs/release-security-review-packet.md`;
- `docs/reviews/release-item4-subgates-20260909.md`;
- `docs/status.md` only if a status/boundary actually changes.

Keep developer-local gates, independent bounded reviews, hosted CI, live WAN evidence and performance conclusions separate.

Do not mark item 4 complete merely because all bounded engineering surfaces above were reviewed. Final independent maintainer/security judgment remains separate.

## CHECKPOINT 15 — next-stage classification, not idle by default

When this queue is consumed:

1. enumerate concrete defects fixed and no-finding independent surfaces;
2. identify any still-unreviewed implemented core surface before declaring local review-support exhausted;
3. inspect whether any repair created a materially new `READY_LIVE` question; if not, preserve `READY_LIVE: none`;
4. classify remaining gates: D019 policy, RSEC-001 representative adversarial-load/capacity suitability, item-3 environment/live evidence, signing/key custody/SBOM/publication policy, previous-frozen-release interoperability, final item-4 judgment, RC/freeze/release/production authority.

A future `queue exhausted` verdict is valid only after this broader core-surface inventory, not after one narrow proposal sweep.

## Live/release boundary

Standing VPS authorization remains valid, but current authoritative classification is still `READY_LIVE: none`. Do not mechanically rerun HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint migration, key update, IPv6, PLPMTUD or Experimental Track work without a materially new dependency-ready question.

This handoff deliberately supplies substantial local work while preserving the rule that network activity must answer a new real question.
