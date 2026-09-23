# ChatGPT reviewer handoff — H-I4-114 duplicate-negotiation TCP read bound HIGH FRONT

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- Developer-owned H-I4-113 source repair is accepted at `8fc460c8eeb2369efaa6e0651f2a3fd8f864ccae` plus formatting-only `d2827851751b0eddc782943ee3498aeba029f513`: the seven periodic real clients and two endpoint-rebind real clients now use existing `bounded_client_output` with the requested duration + 5s harness bounds while preserving their assertions. **Final reachable developer-local exact-tree provenance for that final source tree is still pending**; reviewer source inspection is not a substitute for the gate.
- H-I4-111 and H-I4-112 remain closed at `fc23a7a88c917e116f1140333faa6d8f441ca557`. H-I4-107/108/109 remain accepted/provenanced through `4c445ce36aa06edc1b3d4952036073865e3f20d0`; H-I4-110 remains closed at `259495fad9949a526b12187ba59cf74aec0ee7f7`. H-I4-108 remains an owner-inventory effort, not a repository-wide statement that every process/socket/thread owner is bounded.
- Candidate A (`Recovery::on_ack` future/unsent ACK) and Candidate B (`record_datagrams` mixed drop reasons) remain closed unless materially changed or falsified by exact-current source/tests.
- `SessionRuntime.events` retained-history capacity and D019 source-retention/no-reset remain maintainer/security policy gates. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely for freshness.

## FRONT HIGH — H-I4-114 duplicate-negotiation TCP response read can strand the gate

Reviewer finding: `docs/reviews/h-i4-114-duplicate-negotiation-tcp-read-bound-20260923.md`.

### Concrete defect

In exact-current `crates/neko-cli/tests/probe.rs::tcp_and_udp_reject_malformed_unsupported_and_duplicate_negotiation_before_echo`, the TCP **duplicate-negotiation** branch starts a real server, connects a real `TcpStream`, sends the first valid hello, then immediately calls `frame_read_test(&mut socket)` for the server response. `frame_read_test` performs blocking `read_exact` calls. The test only installs `set_read_timeout(Some(Duration::from_secs(1)))` **after** that first response has completed, immediately before the second/duplicate-hello close check.

Therefore a regression where the server accepts the connection but never emits, or only partially emits, the first negotiation response can block the test forever in `read_exact`; the later one-second timeout and bounded `finish_server(server)` are unreachable. This is release/item-4 test-harness correctness/evidence reliability HIGH, not a production wire/negotiation architecture finding.

The adjacent UDP duplicate branch already installs a read timeout before its first `recv`. The H-I4-105 transcript-mismatch and unsupported-selected-version TCP peer owners also install a read timeout before their first `frame_read_test`, so do not broaden this finding mechanically.

### Closure contract

1. Install the existing test-local TCP read bound **before the first** `frame_read_test(&mut socket)` in the duplicate-negotiation branch. Reuse an established local bound; do not introduce repository-wide timeout policy.
2. Preserve the first negotiation response bytes, second duplicate hello, terminal-close assertion, UDP branch, server status/log assertions, and protocol semantics exactly.
3. A separate synthetic regression is optional if it would merely duplicate socket timeout semantics; source-level ownership proof plus existing timed socket tests is acceptable. Do not create framework churn.
4. No decoder/parser/crypto-framing implementation change is implicated; do not run fuzz mechanically.
5. On the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist developer-local exact-tree provenance with reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust and clean-tree state. If accurate and explicitly scoped, that final gate may also close the pending H-I4-113 exact-tree provenance requirement for unchanged H-I4-113 source.
6. Continue immediately to the next ownership/review slice after closure; do not wait for reviewer cadence and do not infer repository-wide queue exhaustion from one socket-read repair.

## Rolling queue — keep continuous after H-I4-114

Do not collapse this to one ticket. Dependency-ready order:

1. **H-I4-114 repair + exact-tree provenance**, with H-I4-113 final source provenance folded in only if the final reachable gate explicitly covers that unchanged source.
2. **Remaining process/socket/thread ownership causal sweep.** Continue exact-current `crates/neko-cli/tests/probe.rs` and `crates/neko-cli/tests/multistream.rs`. Pay special attention to blocking `read_exact`/`read`/`recv`/`recv_from`/`accept`, direct `.output()` / `wait*`, pipe drains and thread/reader joins. Classify each as exit/peer-completion proven, source-self-bounded, externally bounded, or concrete unbounded owner; repair only the last class. Preserve fail-fast/self-bounded exclusions such as keygen/help/capabilities/invalid config/socket-free fixtures.
3. **Cross-platform CLI/process factual reconciliation.** Reconcile I4-CLI-PROC-096 and H-I4-097..114 with exact-current helper/cfg/socket/process semantics. Keep Linux-local execution, macOS/BSD source/cfg reasoning and Windows claims separate.
4. **Algorithmic/resource boundedness reconciliation.** Reconcile accepted boundedness reviews with current channel/output bounds, stdout accumulation, cleanup deadlines, socket peer lifetime, reader lifetime and child/thread ownership. Do not run capacity-pressure benchmarks or invent capacity/security policy values.
5. **Release packet / item-4 factual reconciliation.** Qualify stale statements that process failure paths are globally bounded or review is exhausted; index reachable repair/review notes only after the relevant exact-tree provenance exists. Local process/socket tests are not WAN/performance/security approval.
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
- H-I4-113 source conversion is accepted from exact-current source, but its final developer-local exact-tree provenance is still pending. Do not call reviewer source inspection a local gate.
- Reviewer H-I4-114 is source/control-flow inspection only; reviewer did not run Rust/full gate, cross-platform execution, fuzz, WAN or performance work in this pass.
- Hosted Actions are cross-evidence, not a wait condition and not a replacement for the developer-local clean exact-tree gate.
- Fuzz only when wire decoder/parser/crypto framing changes materially.
- Standing VPS authorization permits bounded self-owned TCP/UDP lab work, but current authoritative classification remains `READY_LIVE: none`; no repeat live run without a new concrete question.
- Never decide D019, capacity/TTL/LRU/history/security values, signing/key-custody/SBOM/publication policy, core Session/Carrier/ACK/crypto/wire architecture, destructive/canonical migration, RC/freeze/release/production authority.
