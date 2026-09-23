# ChatGPT reviewer handoff — H-I4-113 periodic/endpoint client ownership HIGH FRONT

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- **H-I4-111 and H-I4-112 are CLOSED** at developer-owned `fc23a7a88c917e116f1140333faa6d8f441ca557`: `expired_preprogress_udp_session_is_retired_before_delivery_and_fresh_handshake_recovers` now routes its sequential expired (`--duration 2 -> 7s`) and recovery (`--duration 3 -> 8s`) clients through `bounded_client_output`. The false source-coverage sentence in the older `1d727f0` provenance was preserved and explicitly corrected rather than silently rewritten. Reachable exact-tree provenance is `docs/notes/h-i4-112-provenance-fc23a7a-20260923.md`: developer-reported `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, 2026-09-23T07:50:14Z -> 07:56:00Z, Linux x86_64, stable rustc 1.98.0. GitHub combined status for exact `fc23a7a` exposes no hosted status entries; absence is not a hosted pass/fail result.
- H-I4-107/108/109 remain accepted/provenanced through developer-owned `4c445ce36aa06edc1b3d4952036073865e3f20d0`. H-I4-110 remains closed at `259495fad9949a526b12187ba59cf74aec0ee7f7`. H-I4-097..107 and H-I4-109..112 remain closed on their exact source claims absent exact-current falsification. H-I4-108 remains a continuing owner-inventory effort, not a repository-wide claim that every process owner is bounded.
- Candidate A (`Recovery::on_ack` future/unsent ACK) and Candidate B (`record_datagrams` mixed drop reasons) remain closed unless materially changed or falsified by exact-current source/tests.
- `SessionRuntime.events` retained-history capacity and D019 source-retention/no-reset remain maintainer/security policy gates. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely for freshness.

## FRONT HIGH — H-I4-113 periodic / endpoint-rebind real client owners remain unbounded

Reviewer finding: `docs/reviews/h-i4-113-periodic-endpoint-client-ownership-20260923.md`.

### Concrete defect

Exact-current `crates/neko-cli/tests/probe.rs` still has two real-network process-test families where the server is bounded but the client is not. A product `--duration` is behavior under test, not an independent harness deadline; if connect/negotiation/Noise/auth/DeliveryAck/lifecycle/shutdown regresses, synchronous `Command::output()` can block forever before `finish_server(server)` becomes reachable.

**Periodic-session family — seven direct synchronous real clients:**

1. `periodic_session_delayed_confirmations_are_counted_on_one_session` — `periodic-client --duration 5`;
2. `periodic_session_synchronized_key_update_crosses_authenticated_socket` — `--duration 5`;
3. `periodic_session_mismatched_key_update_schedule_fails_closed` — `--duration 5`;
4. `periodic_session_accounts_missing_ack_and_fails_closed` — `--duration 2`;
5. `periodic_session_duplicate_ack_is_authenticated_and_idempotent` — `--duration 5`;
6. `periodic_setup_timeout_is_separate_from_ack_timeout` — `--duration 5`;
7. `periodic_setup_timeout_fails_before_application_records` — `--duration 2`.

`start_periodic_server` readiness and `finish_server` exit/drain are already bounded, but neither protects the test while the main thread is blocked in the client `.output()`.

**Endpoint-rebind family — two direct synchronous real clients:**

8. `endpoint_rebind_real_sockets_promote_new_source_and_reject_stale_old_source` — `endpoint-rebind-client --duration 5`;
9. `endpoint_rebind_wrong_challenge_fails_after_candidate_without_success` — `endpoint-rebind-client --duration 3`.

This is release/item-4 **test-harness correctness + evidence reliability HIGH**, not a production Session/Carrier/ACK/crypto/wire defect. Do not infer that every `.output()` is defective: keygen/help/capabilities, invalid-argument/configuration and socket-free fixture calls remain explicit fail-fast/self-bounded exclusions unless exact source proves otherwise.

### Closure contract

1. Reuse exact-current `bounded_client_output`; do not create another process framework.
2. Convert the seven periodic clients while preserving every existing argument/assertion:
   - `--duration 5` => `Duration::from_secs(10)` under the established duration + 5s harness convention;
   - `--duration 2` => `Duration::from_secs(7)`.
3. Convert both endpoint-rebind clients:
   - `--duration 5` => `Duration::from_secs(10)`;
   - `--duration 3` => `Duration::from_secs(8)`.
4. Preserve all delayed-confirmation, synchronized/mismatched key-update, missing/duplicate ACK, setup-timeout and authentication assertions; preserve endpoint candidate/challenge/promotion/sync/stale-source/failure assertions exactly.
5. Existing `bounded_client_output_fails_when_client_never_exits` is sufficient for the ordinary shared deadline path. Do not add duplicate generic sleep churn unless these owners expose a distinct failure mode.
6. Do not mechanically convert keygen, help/capabilities, invalid argument/configuration or socket-free fixture `.output()` calls.
7. No decoder/parser/crypto-framing implementation change is implicated; do not run fuzz mechanically.
8. On the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist developer-local exact-tree provenance with reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust and clean-tree state.
9. Continue immediately to the next ownership/review slice after closure; do not wait for reviewer cadence and do not infer repository-wide queue exhaustion from this family repair.

## Rolling queue — keep continuous after H-I4-113

Do not collapse this to one ticket. Dependency-ready order:

1. **H-I4-113 repair + exact-tree provenance** — current FRONT HIGH.
2. **Remaining process/socket/thread ownership causal sweep.** Continue exact-current `crates/neko-cli/tests/probe.rs`, `crates/neko-cli/tests/multistream.rs`, and other process tests. Pay special attention to later direct/indirect `.output()`, `wait`, `wait_with_output`, `try_wait`, blocking socket `accept`/`recv`/`read_exact`, pipe drains and peer/reader joins. Classify each as exit/peer-completion proven, source-self-bounded, externally bounded, or concrete unbounded owner; repair only the last class. Preserve explicit fail-fast/self-bounded exclusions.
3. **Cross-platform CLI/process factual reconciliation.** Reconcile I4-CLI-PROC-096 and H-I4-097..113 with exact-current helper/cfg/socket/process semantics. Keep Linux-local execution, macOS/BSD source/cfg reasoning and Windows claims separate.
4. **Algorithmic/resource boundedness reconciliation.** Reconcile accepted boundedness reviews with current channel/output bounds, stdout accumulation, cleanup deadlines, socket peer lifetime, reader lifetime and child/thread ownership. Do not run capacity-pressure benchmarks or invent capacity/security policy values.
5. **Release packet / item-4 factual reconciliation.** `docs/release-security-review-packet.md` top coverage boundary still predates H-I4-097..113. Qualify stale statements that process failure paths are globally bounded or review is exhausted; index reachable repair/review notes only after relevant exact-tree provenance exists. Local process/socket tests are not WAN/performance/security approval.
6. **Pre-auth malformed/rejection accounting exact-current reuse challenge.** Reuse prior independent review only where owner source/tests remain unchanged; otherwise narrowly re-challenge changed ownership. D019 remains a policy gate.
7. **CLI diagnostic / exit-code / JSON / human-output boundary exact-current reuse challenge.** Diagnostics remain evidence-only and never authentication/Delivery/Path/ACK evidence.
8. **Package/build/reproducibility spot re-challenge.** Verify recent process-test repairs do not stale manifests/scripts/provenance assumptions. Do not invent signing/SBOM/key-custody/publication policy.
9. **Reliable UDP / CarrierState / CarrierManager / FairScheduler / SessionRuntime targeted spot re-challenge.** Re-open materially changed owners; otherwise record exact-current reuse boundaries rather than rerunning equivalent sweeps.
10. **Observability + carrier adapters + dependency/build exact-current reuse challenge.** Same owner-diff rule; no checker/schema/framework/docs filler.
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
- Exact `fc23a7a` has reachable developer-reported local provenance for H-I4-112; GitHub combined status exposed no hosted entries. Do not call the absence a hosted pass/fail result.
- Reviewer H-I4-113 is source/control-flow inspection only; reviewer did not run Rust/full gate, cross-platform execution, fuzz, WAN or performance work in this pass.
- Hosted Actions are cross-evidence, not a wait condition and not a replacement for the developer-local clean exact-tree gate.
- Fuzz only when wire decoder/parser/crypto framing changes materially.
- Standing VPS authorization permits bounded self-owned TCP/UDP lab work, but current authoritative classification remains `READY_LIVE: none`; no repeat live run without a new concrete question.
- Never decide D019, capacity/TTL/LRU/history/security values, signing/key-custody/SBOM/publication policy, core Session/Carrier/ACK/crypto/wire architecture, destructive/canonical migration, RC/freeze/release/production authority.
