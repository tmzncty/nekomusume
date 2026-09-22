# ChatGPT reviewer handoff — H-I4-107 multistream bounded-owner repair HIGH FRONT

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- Developer-owned `a5e80bb85948419e20a01430ee8381bf475fa3bd` converts the three H-I4-106 **server-side** `wait_with_output()` owners in `crates/neko-cli/tests/multistream.rs` to a local `bounded_wait_with_output`; `f3093d5a6c5ec063d29f08b0eada0a4afee6a538` adds a deterministic no-client timeout regression. Developer-local exact-tree provenance is `docs/notes/h-i4-106-provenance-a5e80bb-20260922.md` (`PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, Linux x86_64, rustc 1.98.0 stable). No exact-`a5e80bb` hosted workflow/status was visible in the reviewer pass that opened H-I4-107; do not relabel this as hosted or reviewer-local execution.
- **H-I4-106 must not yet be treated as fully repository-closed.** Its original closure contract required bounded ownership for both sides of the two real server/client tests, but exact-current `bounded_tcp_multistream_loopback_is_ordered_and_json_evidenced` and `unauthorized_client_is_rejected_by_allowlist` still run the networked client with synchronous unbounded `.output()` before the new server helper is reached.
- **H-I4-107 is OPEN HIGH** at reviewer source anchor `0dfed87fc96f1427e333f8d42dde34f96a31ae2a`: in addition to those remaining client owners, the new `bounded_wait_with_output` helper still abandons unresolved child/pipe ownership on `try_wait`/cleanup error branches. Full finding: `docs/reviews/reviewer-h-i4-107-bounded-wait-helper-error-ownership-20260923.md`.
- H-I4-097..105 remain closed on their original child/readiness/barrier/socket-thread ownership claims except where an exact-current counterexample identifies a distinct owner. H-I4-090..095 remain closed on malformed-resource causality/cfg proof surfaces absent owner change/falsification.
- Candidate A (`Recovery::on_ack` future/unsent ACK) and Candidate B (`record_datagrams` mixed drop reasons) remain closed unless materially changed or falsified by exact-current source/tests.
- `SessionRuntime.events` retained-history capacity and D019 source-retention/no-reset remain maintainer/security policy gates. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely for freshness.

## FRONT HIGH — H-I4-107 multistream bounded-owner repair remains incomplete

### Concrete defects

Exact-current `crates/neko-cli/tests/multistream.rs` still has three related ownership gaps:

1. **Positive real client remains unbounded.** `bounded_tcp_multistream_loopback_is_ordered_and_json_evidenced` starts the real multistream client with synchronous `.output()`. If the client blocks on connect/handshake/framed I/O, the test never reaches `bounded_wait_with_output(&mut server, ...)`.
2. **Unauthorized real client remains unbounded.** `unauthorized_client_is_rejected_by_allowlist` retains the same synchronous networked client `.output()` shape. The expected authorization failure is product behavior under test, not a harness-local termination proof.
3. **The new server helper is not ownership-truthful on error branches.** It starts stdout/stderr drain threads, then:
   - outer `try_wait Err` panics immediately with no bounded terminate/reap;
   - deadline cleanup ignores `kill()` result;
   - post-kill `try_wait Err` is grouped with `Ok(Some(_))`, even though error does not prove exit/reap;
   - panic then abandons potentially live pipe-reader threads before exit/pipe-closure proof.

`f3093d5` proves only the ordinary live-server timeout branch with normal kill/reap. It does not bound the two client owners or prove the helper's OS-error branches.

Invariant: every networked child whose termination is part of the item-4 oracle needs an independent harness-local completion bound, and every helper failure branch must distinguish exit proof, bounded cleanup success, and bounded cleanup failure/unproven exit. Product behavior under test cannot be the only termination proof.

This is a release/item-4 **test-harness correctness HIGH**, not a production Session/Carrier/ACK/crypto/wire semantic finding.

### Closure contract

1. Finish the original H-I4-106 shape: give the networked client processes in `bounded_tcp_multistream_loopback_is_ordered_and_json_evidenced` and `unauthorized_client_is_rejected_by_allowlist` their own narrow test-local bounded owner. Do not move `.output()` into an unkillable worker thread.
2. Keep the three converted server call sites and ordinary timeout semantics, but make every `bounded_wait_with_output` failure branch ownership-truthful.
3. Initial `try_wait Err` must enter bounded best-effort terminate/reap before failing; do not immediately panic with unresolved child ownership.
4. Deadline cleanup must not ignore `kill()` result and must not treat post-kill `try_wait Err` as `Ok(Some)`. Preserve truthful classes: exit proven; bounded cleanup succeeded; bounded cleanup itself failed / exit unproven.
5. Do not block on stdout/stderr drain joins while exit/pipe closure is unproven. If cleanup itself fails and a live child may still own the write ends, fail closed without falsely claiming drain convergence. Prefer a small test-local child owner/helper; do not build a repository-wide process framework.
6. Preserve current negotiation/crypto/JSON/human-output assertions and the deterministic no-client regression. Add focused bounded-client coverage. If forcing real `try_wait`/`kill` OS errors would require unsafe/kernel fault injection, source-level ownership proof is acceptable; do not manufacture brittle platform tricks.
7. No decoder/parser/crypto-framing implementation change is implicated; do not run fuzz mechanically.
8. On the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist developer-local exact-tree provenance with reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust.
9. Continue immediately into the next ownership/review slice after closure; do not wait for reviewer cadence and do not infer repository-wide queue exhaustion from this repair.

## Rolling queue — keep continuous after H-I4-107

Do not collapse this to one ticket. Dependency-ready order:

1. **H-I4-107 repair + bounded real-client ownership + helper error-path ownership proof/regression + exact-tree provenance** — current FRONT HIGH.
2. **Remaining process/socket/thread ownership causal sweep.** Re-read exact-current direct/indirect `try_wait`, `wait`, `wait_with_output`, `.output()`, socket `accept`/`recv`/`read_exact`, stdout/stderr drain, peer/reader `join`, channel timeout and child/thread ownership sites in `crates/neko-cli/tests/probe.rs`, `crates/neko-cli/tests/multistream.rs`, and other process-test owners. Classify each as exit/peer-completion-proven, success-path source-self-bounded, or failure-path externally bounded. Repair only concrete unproven hang/ownership counterexamples; ordinary synchronous CLI calls are not automatically defects.
3. **Cross-platform CLI/process factual reconciliation.** Reconcile I4-CLI-PROC-096 and H-I4-097..107 with exact-current helper/cfg/socket/process semantics. Keep Linux-local execution, macOS/BSD source/cfg reasoning and Windows claims separate.
4. **Algorithmic/resource boundedness reconciliation.** Reconcile accepted boundedness reviews with current channel/output bounds, stdout accumulation, cleanup deadlines, socket peer lifetime, reader lifetime and child/thread ownership. No capacity-pressure benchmark and no invented policy/security values.
5. **Release packet / item-4 factual reconciliation.** Qualify stale statements that all process failure paths are bounded or that repository-wide review is exhausted. Retain valid H-I4-090..105 boundaries and only re-close H-I4-106 after both client/server owner contract and H-I4-107 helper-error ownership are satisfied. Do not promote local process/socket tests to WAN/performance/security approval.
6. **Pre-auth malformed/rejection accounting exact-current reuse challenge.** Reuse prior independent review only where owner source/tests remain unchanged; otherwise narrowly re-challenge changed ownership. D019 remains a policy gate.
7. **CLI diagnostic / exit-code / JSON / human-output boundary exact-current reuse challenge.** Diagnostics remain evidence-only and never authentication/Delivery/Path/ACK evidence.
8. **Package/build/reproducibility spot re-challenge.** Verify recent process-test repairs do not stale manifests/scripts/provenance assumptions. Do not invent signing/SBOM/key-custody policy.
9. **Reliable UDP / CarrierState / CarrierManager / FairScheduler / SessionRuntime targeted spot re-challenge.** Re-open materially changed owners; otherwise record exact-current reuse boundaries rather than rerunning equivalent sweeps.
10. **Observability + carrier adapters + dependency/build exact-current reuse challenge.** Same owner-diff rule; no checker/schema/docs filler.
11. **Repository-wide 13-surface refill.** Re-apply every required core surface after the HIGH and dependent reconciliation close. Queue exhaustion is legal only if the broad inventory finds no concrete defect, no uncovered implemented core surface, no READY review-support and no READY live question.
12. **Conditional live.** Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved real-network question within standing authorization. Otherwise keep `READY_LIVE: none`.

If several coherent slices complete in 10–30 minutes with stable quality, deepen the queue and continue implementation/review -> tests -> commit -> push -> next slice without waiting. Every 3–4 coherent slices or important repair cluster, do one factual item-4/release reconciliation rather than rewriting large docs after every small commit.

## Required 13-surface refill inventory

At each meaningful repository-wide refill, explicitly ask whether each surface has a reachable, dedicated, independent bounded review that remains valid on current owners:

1. `neko-reliable` UDP recovery — ACK range, future/unsent ACK, loss/retransmit, RTT/PTO, persistent congestion, Reno, fault simulation;
2. `neko-carrier::CarrierState` — generation, validation, hysteresis, single-active, drain/fail/activate;
3. Concurrent Carrier Manager / health / migration-back;
4. FairScheduler / multi-stream / session+stream flow-control accounting;
5. Memory/UDP/TCP carrier adapter close/error/resource semantics;
6. `SessionRuntime` lifecycle/resource/window/DeliveryAck accounting;
7. `neko-observe` projection/event/counter/high-water correctness;
8. package/reproducibility/operator scripts;
9. Cargo manifests/lock/features/build/native hooks/unsafe inheritance;
10. cross-platform CLI/process-test semantics;
11. CLI exit-code / JSON / human-output contract;
12. algorithmic resource boundedness without capacity-pressure benchmarks or new policy values;
13. release-packet factual consistency and evidence boundary.

A bounded no-finding review is valid item-4 support when it identifies inspected owners, challenged invariant, deterministic checks/commands, exclusions and an exact reachable anchor. Do not create checker/schema/framework/docs churn merely to manufacture a slice.

## Evidence discipline

- Developer-reported local CI, repository-persisted provenance, reviewer source review, hosted CI, live WAN evidence and performance conclusions are distinct evidence classes.
- Accepted exact-tree provenance must anchor a GitHub-resolvable pushed SHA. Never publish a local-only/unreachable SHA as shared exact-tree evidence.
- Hosted Actions are cross-evidence, not a wait condition and not a replacement for the developer-local clean exact-tree gate.
- Fuzz only when wire decoder/parser/crypto framing changes materially.
- Standing VPS authorization permits bounded self-owned TCP/UDP lab work, but current authoritative classification remains `READY_LIVE: none`; no repeat live run without a new concrete question.
- Never decide D019, capacity/TTL/LRU/history/security values, signing/key-custody/SBOM/publication policy, core Session/Carrier/ACK/crypto/wire architecture, destructive/canonical migration, RC/freeze/release/production authority.
