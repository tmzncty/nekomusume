# ChatGPT reviewer handoff — H-I4-106 multistream process ownership HIGH FRONT

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- **H-I4-106 is OPEN HIGH** at reviewer finding `735212dac22bc010b6d2bcdf6a90da5ff7e06bd0`. Exact-current `crates/neko-cli/tests/multistream.rs` still has real multistream child owners whose `.output()` / `wait_with_output()` have no independent harness-local lifetime bound. Full finding: `docs/reviews/reviewer-h-i4-106-bounded-multistream-process-ownership-20260922.md`.
- **H-I4-105 is CLOSED** at developer-owned `af214408e8350e95723e3d855b1786da738588b3`: auxiliary peer workers now bound listener accept / first UDP receive / accepted-stream frame reads before the main test joins them. Persisted developer-local exact-tree provenance: `docs/notes/h-i4-105-provenance-af21440-20260922.md` (`PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, Linux x86_64, stable Rust). No exact-`af214408` hosted workflow/status was visible in the reviewer pass that opened H-I4-106; do not relabel developer-local provenance as hosted or reviewer-local execution.
- H-I4-097..105 remain closed on their original child/readiness/barrier/socket-thread ownership claims except where an exact-current counterexample identifies a distinct owner. H-I4-090..095 remain closed on malformed-resource causality/cfg proof surfaces absent owner change/falsification.
- Candidate A (`Recovery::on_ack` future/unsent ACK) and Candidate B (`record_datagrams` mixed drop reasons) remain closed unless materially changed or falsified by exact-current source/tests.
- `SessionRuntime.events` retained-history capacity and D019 source-retention/no-reset remain maintainer/security policy gates. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely for freshness.

## FRONT HIGH — H-I4-106 bounded multistream product-process ownership

### Concrete defect

The H-I4-105 auxiliary-peer repair is valid, but the remaining ownership sweep found a distinct real-process owner in `crates/neko-cli/tests/multistream.rs`.

At reviewed source anchor `945d03dbf1b671811be498b769873349120c2b4c`:

1. `bounded_tcp_multistream_loopback_is_ordered_and_json_evidenced`
   - spawns the real multistream server;
   - runs the real client through synchronous `Command::output()`;
   - then calls `server.wait_with_output()`;
   - neither child lifetime is independently bounded by the test harness. The 50 ms startup sleep is not a readiness/lifetime proof.
2. `unauthorized_client_is_rejected_by_allowlist`
   - has the same real-server + real-client `.output()` / `wait_with_output()` ownership shape without an independent deadline.
3. `executable_rejects_unsupported_only_negotiation_before_noise_or_data`
   - bounds its direct socket startup/read observation, but then calls `server.wait_with_output()` with no independent child-exit deadline. A server that closes/resets the tested socket but remains alive would strand the gate.

Invariant: a built-binary/process integration test must have a harness-local completion bound independent of the product behavior whose termination is under test. A lifecycle/protocol regression must become bounded negative evidence, not an indefinitely hung item-4 gate.

This does **not** mean every synchronous `.output()` is automatically defective. The finding is limited to exact-current networked multistream child owners for which no separate finite exit proof exists. Exact-current `crates/neko-cli/src/multistream.rs` uses blocking TCP/framed I/O as part of the semantics under test and does not provide an outer wall-clock proof that can substitute for a harness deadline.

The earlier `49925acf7d2d77595562d9ad3327f4b145a31fd1` cross-platform process no-finding does not close this surface: its declared owners were `probe.rs` readiness/cleanup helpers plus `main.rs`/manifest portability, not these `tests/multistream.rs` real child lifecycle waits.

This is a release/item-4 **test-harness correctness HIGH**, not a production Session/Carrier/ACK/crypto/wire semantic finding.

### Closure contract

1. Give the affected multistream child owners a narrow test-local deadline while retaining kill/reap ownership on timeout/error. Do not implement a timeout by moving `Command::output()` into an unkillable worker thread.
2. Preserve stdout/stderr evidence without creating a pipe deadlock. If stdout/stderr are piped while the child is live, drain concurrently or use a strictly equivalent bounded pattern; blocking final drain is safe only after exit/pipe-closure proof.
3. Timeout is harness failure only. After deadline, perform bounded best-effort termination/reap and fail closed.
4. Add one deterministic negative regression for the common process-owner shape: intentionally keep a child/process alive past the local deadline and prove the helper returns/fails within its bound instead of hanging.
5. Reuse a narrow helper where useful, but do not build a generic process framework or introduce repository-wide timeout/capacity/security policy numbers. Fast pre-network/config/keygen `.output()` calls are not in this finding unless a later source review produces a separate concrete counterexample.
6. Preserve H-I4-090..105 closure semantics and exact negotiation/crypto/output assertions. No decoder/parser/crypto-framing implementation change is implicated; do not run fuzz mechanically.
7. On the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm a clean tree, and persist developer-local exact-tree provenance with reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust.
8. Continue immediately into the next ownership/review slice after closure; do not wait for reviewer cadence and do not declare repository-wide queue exhaustion from this repair.

### Stagnation assist — concrete narrow implementation shape

`main` remained at the reviewer handoff without a developer-owned H-I4-106 commit across multiple reviewer opportunities. Per `AGENTS.md` §3.2, treat that as implementation stagnation, not as a reason for both sides to wait. The invariant is already decided; no maintainer/policy choice is required.

A minimal acceptable shape is test-local to `crates/neko-cli/tests/multistream.rs`:

1. Introduce one small captured-child helper/struct that **retains the `Child` handle**. Spawn with `stdout`/`stderr` piped, immediately take both pipes, and drain them concurrently into bounded channels/buffers so a live child cannot deadlock on a full pipe.
2. Its bounded wait polls `try_wait()` until a caller-supplied test-local deadline. `Ok(Some(status))` establishes exit proof. `Ok(None)` at deadline and `Err(_)` both go through bounded best-effort `kill` + bounded `try_wait` reap before returning/panicking as harness failure. Do not call blocking `wait()`/`wait_with_output()` while exit is unproven.
3. After direct-child exit is proven, collect the stdout/stderr drain results through a bounded receive/join step and construct/preserve the same `std::process::Output`-equivalent evidence (`status`, `stdout`, `stderr`). Keep current assertions and JSON/human-output checks unchanged.
4. Use that owner for **both** sides of the two real server/client tests: start the captured server, run the captured client with its own deadline, then bounded-wait the server. For `executable_rejects_unsupported_only_negotiation_before_noise_or_data`, only the server owner needs replacement; keep its already-bounded socket startup/read oracle intact.
5. The deterministic negative regression can use the actual `neko-cli multistream --mode server` with valid temporary identity/key material on an unused loopback port and intentionally provide **no client**. The helper must reach its local deadline, terminate/reap the still-blocked server, and return/fail within the bound. This avoids shell/platform-specific sleeper dependencies and directly exercises the owner shape under review.
6. The existing 50 ms startup sleep is not a lifetime proof. It may remain only as scheduling/race mitigation if current positive tests still need it; do not cite it as closure evidence and do not redesign product readiness semantics in this slice.
7. Keep the helper local and small. Do not move it into production code, do not create a repository-wide process framework, and do not rewrite fast pre-network/config/keygen `.output()` calls merely for symmetry.

This is an implementation-shape refinement of H-I4-106, not a new finding and not a scope expansion. After the repair/provenance commit, continue directly to the remaining process/socket/thread ownership sweep.

## Rolling queue — keep continuous after H-I4-106

Do not collapse this to one ticket. Dependency-ready order:

1. **H-I4-106 repair + focused bounded process-owner negative regression + exact-tree provenance** — current FRONT HIGH.
2. **Remaining process/socket/thread ownership causal sweep.** Re-read exact-current direct/indirect `try_wait`, `wait`, `wait_with_output`, `.output()`, socket `accept`/`recv`/`read_exact`, stdout/stderr drain, peer/reader `join`, channel timeout and child/thread ownership sites in `crates/neko-cli/tests/probe.rs`, `crates/neko-cli/tests/multistream.rs`, and other process-test owners. Classify each as exit/peer-completion-proven, success-path source-self-bounded, or failure-path externally bounded. Repair only concrete unproven hang counterexamples; ordinary synchronous CLI calls are not automatically defects.
3. **Cross-platform CLI/process factual reconciliation.** Reconcile I4-CLI-PROC-096 and H-I4-097..106 with exact-current helper/cfg/socket/process semantics. Keep Linux-local execution, macOS/BSD source/cfg reasoning and Windows claims separate.
4. **Algorithmic/resource boundedness reconciliation.** Reconcile accepted boundedness reviews with current channel/output bounds, stdout accumulation, cleanup deadlines, socket peer lifetime, reader lifetime and child/thread ownership. No capacity-pressure benchmark and no invented policy/security values.
5. **Release packet / item-4 factual reconciliation.** Qualify stale statements that all process failure paths are bounded or that repository-wide review is exhausted. Retain valid H-I4-090..105 boundaries. Do not promote local process/socket tests to WAN/performance/security approval.
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
