# ChatGPT reviewer handoff — H-I4-109 probe bounded-output cleanup HIGH FRONT

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- Developer-owned `10d97e7e2c03ca19d9c575d0f0897c1703af1f8d` converts the remaining real multistream client owners to spawned bounded waits; `75d65e653b3c245e6d6146a293a183f058952804` hardens `multistream.rs::bounded_wait_with_output` so `kill` / post-kill `try_wait` failures are explicit rather than treated as exit proof. Reviewer source inspection continues to accept this H-I4-107 direction.
- **H-I4-107 still needs final reachable developer-local exact-tree provenance.** No persisted source-equivalent provenance note was found on the reviewed head recording `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, UTC start/end, OS/arch and stable Rust. A later exact-tree gate may cover unchanged H-I4-107 source together with H-I4-108/109 if boundaries are stated truthfully.
- Developer-owned `4eadea95f2d5cede3fb6d7518506c0dfef92885d` makes substantial H-I4-108 progress: real `client` / `failover-client` owners in `crates/neko-cli/tests/probe.rs` are moved from synchronous `.output()` to spawned `bounded_client_output`; stdout/stderr are drained off-thread; a live-client timeout regression was added. This is **not yet a full H-I4-108 closure** because the new helper itself has a cleanup-ownership defect and final exact-tree provenance is absent.
- Reviewer commit `c4e3b4ca530087904442a43a8ff89464e34bebc8` opens **H-I4-109 HIGH**: exact `4eadea95` reintroduces into `probe.rs::bounded_wait_with_output` the pre-H-I4-107 error handling — ignored `kill()` errors and post-kill `try_wait Err` accepted as reap/exit proof. Full finding: `docs/reviews/reviewer-h-i4-109-probe-bounded-output-cleanup-ownership-20260923.md`.
- Exact-current `multistream.rs::bounded_wait_with_output` already demonstrates the intended narrow ownership classification. Prefer reuse/faithful mirroring; do not create a generic process framework.
- H-I4-097..106 remain closed on their original source claims absent exact-current falsification. H-I4-107 source direction is accepted but provenance remains open; H-I4-108 source conversion is partial pending H-I4-109 + owner sweep + provenance.
- H-I4-090..095 remain closed on malformed-resource causality/cfg proof surfaces absent owner change/falsification.
- Candidate A (`Recovery::on_ack` future/unsent ACK) and Candidate B (`record_datagrams` mixed drop reasons) remain closed unless materially changed or falsified by exact-current source/tests.
- `SessionRuntime.events` retained-history capacity and D019 source-retention/no-reset remain maintainer/security policy gates. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely for freshness.

## FRONT HIGH — H-I4-109 probe bounded-output cleanup ownership

### Concrete defect

Exact `4eadea95` adds this useful H-I4-108 shape:

```text
spawn product client with piped stdout/stderr
  -> drain pipes off-thread
  -> poll child.try_wait() to local deadline
  -> cleanup on timeout/error
```

But its cleanup branches are not ownership-truthful:

- deadline branch: `let _ = child.kill()` ignores kill failure;
- deadline reap loop: `Ok(Some(_)) | Err(_) => break` treats observation failure as successful reap;
- initial `try_wait Err` branch repeats both mistakes.

A panic is bounded caller failure, but it does not prove child exit or bounded cleanup. A live child and pipe-reader threads may remain unresolved on these error branches. The current `sleep 30` regression only proves the ordinary deadline -> successful kill/reap path.

This is release/item-4 **test-harness correctness HIGH**, not production Session/Carrier/ACK/crypto/wire semantics.

### Closure contract

1. Repair `probe.rs::bounded_wait_with_output` so both deadline and initial-`try_wait Err` paths check `kill()` and never accept post-kill `try_wait Err` as exit/reap proof.
2. Prefer a narrow reuse/refactor of the already-hardened `multistream.rs` ownership pattern or a strictly equivalent test-local shape. Do not build a new generic process framework.
3. Reader joins are legal only after child exit / pipe closure is proven. If cleanup itself fails, fail explicitly with exit still classified unproven; never silently claim convergence.
4. Preserve all H-I4-108 networked-client conversions and the live-client timeout regression. If forcing kernel `try_wait`/`kill` errors safely would require unsafe or brittle platform tricks, source-level ownership proof plus existing deterministic live-child regression is sufficient.
5. After helper repair, finish owner-by-owner classification of remaining `probe.rs` `.output()` / child / socket / thread sites. Do not mechanically convert local fail-fast keygen/help/capability/argument-validation paths.
6. Preserve protocol, crypto, diagnostic, JSON/human-output, settlement, failover and resource assertions. No Session/Carrier/ACK/crypto/wire architecture change.
7. No decoder/parser/crypto-framing implementation change is implicated; do not run fuzz mechanically.
8. On final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist developer-local exact-tree provenance with reachable SHA, UTC start/end, exit codes, OS/arch, stable Rust and clean-tree state. One final exact-tree note may truthfully cover unchanged H-I4-107 plus H-I4-108/109.
9. Continue immediately into the next ownership/review slice after closure; do not wait for reviewer cadence and do not infer repository-wide queue exhaustion from this repair.

## Rolling queue — keep continuous after H-I4-109

Do not collapse this to one ticket. Dependency-ready order:

1. **H-I4-109 repair + H-I4-107/108/109 exact-tree provenance closure** — current FRONT HIGH cluster.
2. **Remaining process/socket/thread ownership causal sweep.** Re-read exact-current direct/indirect `try_wait`, `wait`, `wait_with_output`, `.output()`, socket `accept`/`recv`/`read_exact`, stdout/stderr drain, peer/reader `join`, channel timeout and child/thread ownership sites in `crates/neko-cli/tests/probe.rs`, `crates/neko-cli/tests/multistream.rs`, and other process-test owners. Classify each as exit/peer-completion-proven, source-self-bounded, or externally bounded. Repair only concrete unproven hang/ownership counterexamples.
3. **Cross-platform CLI/process factual reconciliation.** Reconcile I4-CLI-PROC-096 and H-I4-097..109 with exact-current helper/cfg/socket/process semantics. Keep Linux-local execution, macOS/BSD source/cfg reasoning and Windows claims separate.
4. **Algorithmic/resource boundedness reconciliation.** Reconcile accepted boundedness reviews with current channel/output bounds, stdout accumulation, cleanup deadlines, socket peer lifetime, reader lifetime and child/thread ownership. No capacity-pressure benchmark and no invented policy/security values.
5. **Release packet / item-4 factual reconciliation.** The packet predates H-I4-097..109. Qualify stale statements that process failure paths are globally bounded or review is exhausted; index reachable repair/review notes only after exact-tree provenance exists. Do not promote local process/socket tests to WAN/performance/security approval.
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
- At reviewer anchor `4eadea95`, combined status and PR-triggered workflow lookup exposed no hosted checks. Do not convert absence of hosted checks into a local-gate claim.
- Hosted Actions are cross-evidence, not a wait condition and not a replacement for the developer-local clean exact-tree gate.
- Fuzz only when wire decoder/parser/crypto framing changes materially.
- Standing VPS authorization permits bounded self-owned TCP/UDP lab work, but current authoritative classification remains `READY_LIVE: none`; no repeat live run without a new concrete question.
- Never decide D019, capacity/TTL/LRU/history/security values, signing/key-custody/SBOM/publication policy, core Session/Carrier/ACK/crypto/wire architecture, destructive/canonical migration, RC/freeze/release/production authority.
