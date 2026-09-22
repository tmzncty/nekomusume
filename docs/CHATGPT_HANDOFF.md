# ChatGPT reviewer handoff — H-I4-105 auxiliary peer-thread boundedness HIGH FRONT

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- **H-I4-105 is CLOSED** at `af21440`: auxiliary peer threads that the test later `join()`s now have independent test-local completion bounds — `bounded_accept` (nonblocking + poll) for TCP listeners, `set_read_timeout` before the first `recv_from` for UDP peers, and `set_read_timeout` on accepted streams before frame reads. Owners: `matrix_probe` TCP/UDP peers, `tcp_and_udp_transcript_mismatch` peers, `tcp_and_udp_reject_unsupported_selected_version` TCP peer, `multistream.rs` `executable_rejects_one_byte_different_negotiation_binding` TCP peer. Exact-tree provenance: `docs/notes/h-i4-105-provenance-af21440-20260922.md` — `check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-22T20:02:53Z → 20:08:40Z, Linux x86_64, rustc 1.98.0.
- **H-I4-104 is CLOSED** at developer-owned `0467152d4e595c30ba6373333cd19bf8fa382cc0`: `start_periodic_server` delegates readiness to `wait_for_ready_marker(..., 5s)`. Persisted developer-local exact-tree provenance is `docs/notes/h-i4-104-provenance-0467152-20260922.md`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, 2026-09-22T17:50:09Z → 17:56:00Z, Linux x86_64, rustc 1.98.0 stable. No exact-`0467152` hosted workflow/status was visible in the reviewer pass that opened H-I4-105. This is developer-reported local provenance, not reviewer-local execution.
- H-I4-097..104 remain closed on their original child-process/readiness/barrier ownership claims except where an exact-current counterexample identifies a distinct owner. H-I4-090..095 remain closed on malformed-resource causality/cfg proof surfaces absent owner change/falsification.
- Candidate A (`Recovery::on_ack` future/unsent ACK) and Candidate B (`record_datagrams` mixed drop reasons) remain closed unless materially changed or falsified by exact-current source/tests.
- `SessionRuntime.events` retained-history capacity and D019 source-retention/no-reset remain maintainer/security policy gates. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely for freshness.

## FRONT HIGH — H-I4-105 auxiliary peer-thread socket ownership

### Concrete defect

The previous HIGH sequence hardened child-process cleanup and stdout/readiness ownership in `crates/neko-cli/tests/probe.rs`, but exact-current CLI tests still contain independent auxiliary socket peer threads whose blocking operations have no test-local bound before the main test calls `join()`.

Concrete owners at reviewed anchor `67dad3c`:

1. `probe.rs::matrix_probe_distinguishes_invalid_failed_and_reachable_outcomes`
   - TCP peer blocks in `TcpListener::accept()` with no deadline; after the CLI returns, main calls `tcp_peer.join()`.
   - UDP peer blocks in first `recv_from()` with no timeout; after the CLI returns, main calls `udp_peer.join()`.
   - If the probe regresses by returning without connecting/sending, the intended positive assertion is never reached; the gate hangs in the peer/join.
2. `probe.rs::tcp_and_udp_transcript_mismatch_rejects_before_application_echo`
   - TCP peer uses unbounded `accept()` and `frame_read_test()`/`read_exact()` operations.
   - UDP peer uses unbounded first/second `recv_from()`.
   - Main joins after `run_client(...)`; an early client failure can strand the peer indefinitely instead of producing handshake-rejection evidence.
3. `probe.rs::tcp_and_udp_reject_unsupported_selected_version_before_noise`
   - UDP branch already has a read timeout.
   - TCP branch still has unbounded `accept()` and first frame read before any read timeout; main then joins after client completion.
4. `multistream.rs::executable_rejects_one_byte_different_negotiation_binding_before_session_data`
   - auxiliary TCP peer uses unbounded `listener.accept()` and two blocking `frame_read()`/`read_exact()` calls; main joins after the client returns.

Invariant: every auxiliary peer owner that will be `join()`ed must have an independent harness-local completion bound before blocking accept/recv/frame-read work. A CLI regression must become deterministic negative evidence, not an indefinite test hang.

This is a release/item-4 **test-harness correctness HIGH**, not a production runtime transport leak and not a Session/Carrier/ACK/crypto/wire semantic finding. Ordinary synchronous CLI `.output()` calls are not automatically defects; H-I4-105 is specifically about separately-owned peer workers that can outlive the CLI path and strand `join()`.

Full finding: `docs/reviews/reviewer-h-i4-105-bounded-peer-thread-ownership-20260922.md`.

### Closure contract

1. Add narrow test-local completion bounds to each affected auxiliary peer before its first potentially blocking socket operation:
   - UDP: set a read timeout before the relevant first `recv_from()`;
   - accepted TCP streams: install read deadlines before blocking frame reads;
   - listener acceptance: use a bounded/nonblocking accept loop or a strictly equivalent narrow helper with a local deadline.
2. Preserve current positive/rejection semantics and exact negotiation/crypto assertions. A timeout is harness failure only; it is never protocol evidence.
3. Reuse established small test-local bounds where practical. Do **not** create a repository-wide timeout/capacity/security policy value or a generic process framework.
4. Add focused deterministic negative coverage for the common owner shape: omit the expected connect/datagram/frame and prove the peer completes/fails within its local bound instead of hanging `join()`. If a narrow shared helper covers multiple callers, keep caller-specific tests minimal.
5. Preserve H-I4-090..104 proof/cleanup behavior, including `ReadyProof`, `BarrierProof`, malformed classification ordering, FD/RSS margins, `ATTEMPTS`, and bounded child cleanup.
6. No decoder/parser/crypto-framing implementation change is implicated; do not run fuzz mechanically.
7. On the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm a clean tree, and persist developer-local exact-tree provenance with reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust.
8. Continue immediately into the next ownership/review slice after closure; do not wait for reviewer cadence and do not declare repository-wide queue exhaustion from this repair.

## Rolling queue — keep continuous after H-I4-105

Do not collapse this to one ticket. Dependency-ready order:

1. **H-I4-105 repair + focused auxiliary-peer bounded negative regression + exact-tree provenance** — current FRONT HIGH.
2. **Remaining process/socket/thread ownership causal sweep.** Re-read exact-current direct/indirect `try_wait`, `wait`, `wait_with_output`, `.output()`, socket `accept`/`recv`/`read_exact`, stdout/stderr drain, peer/reader `join`, channel timeout and child/thread ownership sites in `crates/neko-cli/tests/probe.rs`, `crates/neko-cli/tests/multistream.rs`, and other process-test owners. Classify each as exit/peer-completion-proven, success-path self-bounded, or failure-path externally bounded. Repair only concrete unproven hang counterexamples; ordinary synchronous CLI calls are not automatically defects.
3. **Cross-platform CLI/process factual reconciliation.** Reconcile I4-CLI-PROC-096 and H-I4-097..105 with exact-current helper/cfg/socket semantics. Keep Linux-local execution, macOS/BSD source/cfg reasoning and Windows claims separate.
4. **Algorithmic/resource boundedness reconciliation.** Reconcile accepted boundedness reviews with current channel bounds, stdout accumulation, cleanup deadlines, socket peer lifetime, reader lifetime and child/thread ownership. No capacity-pressure benchmark and no invented policy/security values.
5. **Release packet / item-4 factual reconciliation.** Qualify stale statements that all process failure paths are bounded or that repository-wide review is exhausted. Retain valid H-I4-090..104 boundaries. Do not promote local process/socket tests to WAN/performance/security approval.
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
