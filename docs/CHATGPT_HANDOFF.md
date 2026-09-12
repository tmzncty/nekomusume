# ChatGPT reviewer handoff — finish residual ACK repair, accept observer fix, continue deep item-4 review

## Reviewed repository truth

- Default branch before this reviewer refresh: exact `e3054c98a96640ac0e95809c2033e7de036c7e7e` (`docs(review): independent bounded UDP recovery review at 8f93b93`).
- Previous high-throughput queue-opening handoff: exact `e6a03776c20c5b32b4676d992cf7fcc613cff2e2`.
- Developer source/test work since that queue opened:
  - `531c82d7a9bb55846a1ea96f9006ecb6e6ae8b3e` — future-ACK repair in `neko-reliable`;
  - `8f93b931b42e486882980f44813660ba6edb4267` — mixed datagram-drop observability repair.
- Reviewer/support work since then:
  - `5cde95ce2fd54e87c1c79f83156e74bf0d46f10e` confirmed the two MEDIUM findings;
  - `e3054c9` records a dedicated UDP-recovery review against source exact `8f93b93`.
- GitHub-hosted `stable checks` and `nightly decode fuzz smoke` both passed for exact `8f93b93`; these are useful cross-evidence only, not a substitute for the required developer-local exact-tree gate/provenance.
- Governance remains unchanged: item 3 incomplete, item 4 incomplete, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, D019 policy-blocked, and authoritative `READY_LIVE: none`.

## Reviewer verdict

### O1 datagram-drop attribution repair `8f93b93` — ACCEPT_WITH_BOUNDS

The repair matches the previously confirmed counter semantics: `queue_dropped` is a subset of generic `dropped`; mixed windows now emit `queue_full` only for that subset and `terminal` for the remainder. The regression constructs one queue-full plus one terminal drop in the same observation interval. The inconsistent-counter case is conservatively clamped rather than inventing transport state. No Session/transport semantics changed.

This closes the original O1 semantic defect, subject to final exact-tree local provenance on the coherent post-R1 source anchor below.

### R1 future-ACK repair `531c82d` — PARTIAL / MEDIUM remains OPEN

The repair correctly rejects `largest > largest_sent` atomically before RTT/loss/PTO/sent-state mutation and preserves `largest == largest_sent`.

However, the already-reviewed R1 contract in exact `5cde95c` explicitly includes **"no-sent-state with a nonempty ACK"** as mechanically invalid. Current code uses `self.largest_sent.is_some_and(|sent| largest > sent)`, so `largest_sent == None` still accepts a nonempty ACK and returns an empty success result.

Exact `e3054c9` incorrectly classifies this case as safe and therefore cannot close the dedicated UDP-recovery review yet. This is not a new protocol-design request: it is a residual boundary from the already-confirmed R1 finding.

The external agent is pre-authorized to repair this MEDIUM and then continue through the deep queue without waiting for reviewer cadence.

# MUST_EXECUTE_LOCAL 1 — close the no-sent ACK boundary

Owner: `crates/neko-reliable/src/lib.rs`, `Recovery::on_ack`, direct `neko-reliable` tests.

Required behavior:

1. Add a focused regression first: a fresh `Recovery` that has never accepted `on_sent` must reject any nonempty `AckRanges`.
2. Rejection must occur before any RTT/PTO/loss/frame/sent-state mutation.
3. Preserve the valid boundary already established by `531c82d`: `largest == largest_sent` is accepted.
4. Preserve already-retired/duplicate ACK history: once this Recovery has actually sent packet numbers, an ACK for an already-retired number `<= largest_sent` must not be rejected merely because that number is absent from the current live `sent` map.
5. Do not redesign ACK encoding, packet numbering, loss detection, or error taxonomy; use the existing fail-closed error surface if adequate.

Focused validation should include:

- fresh/no-sent + nonempty ACK rejection;
- future largest rejection remains atomic;
- largest-equals-largest-sent success;
- already-retired/duplicate ACK behavior remains supported;
- full `cargo test -p neko-reliable`.

Commit/push the coherent repair, then continue immediately to MUST_EXECUTE_LOCAL 2.

# MUST_EXECUTE_LOCAL 2 — coherent exact-tree local closure and provenance

Use the final pushed source SHA containing both the completed R1 repair and O1 observer repair as the tested-tree anchor.

Run in a clean checkout/worktree:

```bash
cargo test -p neko-reliable
cargo test -p neko-observe
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --short
```

Persist concise sanitized provenance with exact reachable SHA, commands, UTC start/end, exit codes, OS/arch, stable Rust version, and initial/final clean-tree state. Hosted CI remains additional cross-evidence only. No decoder fuzz is required solely for these state/observer changes unless a decoder/framing path is unexpectedly touched.

Then continue immediately to REVIEW_LOCAL 3.

# REVIEW_LOCAL 3 — re-close the dedicated UDP recovery review on the corrected anchor

Exact `e3054c9` is useful but contains one false closure statement: it says a nonempty ACK with `largest_sent == None` is safe. Do not rewrite historical evidence as if it never existed. Add a concise follow-up/superseding review on the corrected reachable source anchor that:

- confirms both R1 boundaries are now fail-closed;
- preserves the valid findings from `e3054c9` for ACK canonicalization, retirement, frame-copy counts, threshold/reorder, RTT/PTO, persistent congestion, Reno and deterministic simulation;
- retains the empty-outstanding PTO behavior only as a semantic/caller-scheduling boundary unless new committed semantics resolve it;
- states exact focused commands/tests and exclusions.

If this re-review finds another concrete current-semantics defect, repair it before moving on. Otherwise record a bounded no-finding closure and continue.

# REVIEW_LOCAL 4 — CarrierState generation / validation / hysteresis

Owners: `crates/neko-carrier/src/lib.rs` `CarrierState` / `CarrierEvent` / `PathRecord`, `docs/spec/m0-carrier-state.md`, direct tests.

Challenge:

- packet feedback cannot validate a path or create Session delivery;
- old/future generation errors are mutation-safe;
- activation requires validation + existing hysteresis and cannot create two active owners;
- active/degraded/draining/failed transitions preserve single-active ownership;
- path/resource error ordering is deterministic and non-mutating;
- `active_epoch` changes exactly as the committed spec claims on accepted activation and does not silently reuse/saturate in a way that contradicts "activation increments that epoch";
- unrelated accepted events do not accidentally satisfy a stronger path-specific readiness claim than the currently documented event-based dwell semantics.

Do not invent D064 constants or new failover policy. Concrete defect whose answer current semantics determine -> smallest repair/regression/exact-tree closure; otherwise dedicated independent no-finding note.

# REVIEW_LOCAL 5 — Concurrent Carrier Manager / health / migration-back

Identify exact current owners for `ConcurrentCarrierManager`, `ConcurrentPathKey`, health samples/states, switch events/reasons, hold/cooldown and migration-back.

Challenge current committed semantics:

- single-active, multi-ready; no application striping;
- warm path cannot carry new application data before promotion;
- path/packet feedback cannot become Session delivery evidence;
- failed or generation-stale paths cannot be promoted;
- hold/cooldown/hysteresis is generation-scoped and monotonic;
- migration-back cannot bypass validation or active-owner handoff;
- switch/failure event timestamp/reason/outcome matches the actual transition.

No new scoring constants or architecture.

# REVIEW_LOCAL 6 — FairScheduler / multistream / flow-control accounting

Identify exact owners for `FairScheduler`, `StreamPriority`, stream/session queue/window accounting and multistream tests.

Challenge:

- interactive/bulk fairness and starvation guard;
- per-stream/session byte/window caps;
- enqueue/dequeue accounting on rejection, close, reset and partial progress;
- stream removal/close cannot leak queued-byte/open-stream accounting;
- large/bulk streams cannot bypass limits or contradict anti-starvation claims;
- overflow/saturation cannot promote capacity or delivery evidence.

Property-style tests are welcome only when tied to a concrete invariant; do not redesign the scheduler.

# REVIEW_LOCAL 7 — carrier adapter close / error / resource semantics

Owners: current Memory, UDP and TCP carrier adapters and direct tests.

Challenge:

- message/buffer bounds before allocation/enqueue where applicable;
- claimed idempotent close and queued-data drain semantics;
- local-close / peer-close / I/O / poison / truncation mappings cannot become Session/path evidence;
- native TCP/UDP behavior is not normalized into a stronger guarantee than actually implemented;
- failure paths are mutation/accounting safe.

No async/runtime framework refactor.

# REVIEW_LOCAL 8 — SessionRuntime lifecycle / resource / DeliveryAck accounting

Owners: `crates/neko-session/src/lib.rs` `SessionRuntime`, limits, stream/window/inflight/queue accounting, direct runtime/process tests, current Session/M3 specs.

Challenge:

- hard limits before mutation;
- stream/session window accounting released exactly once;
- duplicate/overlap data does not create unbounded retained state or delivery evidence;
- close/cancel/error/idle/deadline paths leave counters/state consistent;
- rejected transitions are atomic;
- `DeliveryAck` cannot acknowledge invalid/unsent logical ranges or become application `delivered/effect` evidence;
- queue/total-byte counters remain consistent after reset/close/failure.

No Session ACK redesign.

# REVIEW_LOCAL 9 — observability projection full bounded review

The O1 reason-attribution bug is repaired, but the rest of `neko-observe` still needs dedicated review.

Challenge:

- ring capacity and dropped-event accounting;
- sequence overflow/fail-closed behavior;
- timestamp/order claims;
- recovery loss/retransmit/PTO counters versus source results;
- scheduler high-water/starvation counters;
- secret-free fixed-vocabulary correlation/data;
- switch completed/failed shape versus manager result;
- **algorithmic work bound:** `record_datagrams` currently projects counter deltas by per-event loops even though retained event capacity is bounded. Determine whether a large accumulated `u64` delta can cause CPU work disproportionate to the ring bound, and inspect the `u64 -> usize` conversion used by `repeat_n`. Do not select a new security/capacity policy value merely to fix this; if current bounded-observer claims already determine a mechanical cap/aggregation answer, repair minimally, otherwise classify the unresolved policy boundary precisely.

Observer repairs must not change transport semantics.

# REVIEW_LOCAL 10 — package / reproducibility / operator implementation

Independent static/deterministic review of current release/package scripts and tests; do not repeat VPS rehearsals.

Challenge clean-source enforcement, archive path/type/root/mode/checksum validation timing, reproducibility inputs/source identity, dedicated install-path containment, external state/identity retention boundaries, rollback binary selection and cleanup observation ordering.

Do not invent signing/key-custody/SBOM/publication policy.

# REVIEW_LOCAL 11 — dependency / build surface

Current dependency safety evidence is developer-run factual support, not independent approval.

Independently inspect manifests, lockfile, enabled/default features, build scripts/native hooks, security-sensitive direct dependencies, and workspace `unsafe_code = "forbid"` inheritance. Catch executable graph/feature drift or unexpected native/build-time paths. Distinguish project-source unsafe policy from dependency internals. Do not auto-upgrade or create a scanner framework.

# REVIEW_LOCAL 12 — cross-platform CLI / process semantics

Review supported-platform assumptions in CLI/process tests and helpers: terminal-close semantics, `cfg(unix)`, signals/shutdown/exit codes, temp-file cleanup, Unix identity-file permission boundaries, invalid-config-before-network/identity ordering, and incidental OS errors versus protocol semantics.

Do not promise unsupported Windows production behavior.

# REVIEW_LOCAL 13 — CLI machine / human / exit contract

Across shipped bounded CLI surfaces, verify invalid args/config, completed negative experiments, successful results, machine JSON, human output and exit codes cannot contradict one another or emit success-shaped artifacts before evidence exists. Preserve secret/private-topology boundaries. No new CLI framework.

# REVIEW_LOCAL 14 — algorithmic resource boundedness, not pressure testing

Cross-check Session, Carrier, reliable, pre-auth and observability hard ceilings and lifecycle releases. This is not RSEC-001 capacity benchmarking and must not select new capacity/security values.

If an existing committed cap is bypassed, repair it. If a missing cap requires choosing a new value/policy, classify only that finding for maintainer judgment and continue independent lanes.

# REVIEW_SUPPORT 15 — periodic factual reconciliation

After the residual R1 repair plus each coherent group of roughly 3-4 review slices, reconcile established facts into `docs/release-security-review-packet.md`, `docs/reviews/release-item4-subgates-20260909.md`, and `docs/status.md` only when an actual status/evidence boundary changes.

Keep developer-local gates, reviewer independent notes, hosted CI, live WAN evidence and performance conclusions distinct. Do not mark item 4 complete from bounded engineering review alone; final maintainer/security judgment remains separate.

## Continuous execution / stop conditions

Do not idle after a no-finding review. Continue through the numbered queue while dependencies remain satisfied.

Only stop for unresolved BLOCKER/HIGH, a core Session/Carrier/ACK/crypto/wire architecture choice, destructive/canonical migration, new security-policy values, D019 policy, action outside standing authorization, production/third-party/new-credential action, maintainer-valued pressure/benchmark conditions, repository breakage, actual runtime/tool exhaustion, or genuine repository-wide queue exhaustion.

Standing VPS authorization remains valid, but authoritative live classification remains `READY_LIVE: none`. Do not mechanically rerun HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint migration, key update, IPv6, PLPMTUD or Experimental Track work without a materially new dependency-ready real-network question.
