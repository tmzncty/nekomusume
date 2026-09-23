# ChatGPT reviewer handoff — H-I4-115 descendant-held output pipe HIGH FRONT

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- Last developer source/test anchor independently reviewed this pass: `46d624e2272fcd5cf9d86457d018c4f508b8ba10` (`test(cli): H-I4-114 duplicate-negotiation TCP read bound`). Subsequent `9a46fe91185bb8c00aa2140575bf787581b945c3` is docs/provenance/handoff only and does not change the reviewed test source.
- Developer-owned `f5f7bba259bd9ed9da60f5451dcecd292af9a322` closed the remaining networked matrix-probe client owner via existing `bounded_client_output(..., 10s)`. Developer-owned `46d624e...` installed the TCP duplicate-negotiation read timeout before the first `frame_read_test`.
- **H-I4-113 and H-I4-114 are CLOSED** at exact source/test `46d624e...`. Reachable developer-local exact-tree provenance is `docs/notes/h-i4-113-114-provenance-46d624e-20260923.md`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0 after one retained transient first-run bind race, `git diff --check` exit 0, clean tree, 2026-09-23T14:51:06Z → 14:56:50Z, Linux x86_64, stable rustc 1.98.0. GitHub Actions separately records one successful Rust CI workflow for exact `46d624e`; hosted CI remains cross-evidence, not a substitute for the developer-local gate.
- H-I4-111/112 remain closed at `fc23a7a88c917e116f1140333faa6d8f441ca557`; H-I4-107/108/109 remain accepted/provenanced through `4c445ce36aa06edc1b3d4952036073865e3f20d0`; H-I4-110 remains closed at `259495fad9949a526b12187ba59cf74aec0ee7f7`.
- Candidate A (`Recovery::on_ack` future/unsent ACK) and Candidate B (`record_datagrams` mixed drop reasons) remain closed unless materially changed or falsified by exact-current source/tests.
- `SessionRuntime.events` retained-history capacity and D019 source-retention/no-reset remain maintainer/security policy gates. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely for freshness.

## FRONT HIGH — H-I4-115 direct-child exit does not prove inherited output pipes reached EOF

Reviewer finding: `docs/reviews/h-i4-115-bounded-output-descendant-pipe-ownership-20260923.md` (reviewer commit `479781b33c2a7e8c24b65815cc21a4ad6a7a69b1`).

### Concrete defect

Exact-current `crates/neko-cli/tests/probe.rs::bounded_wait_with_output` and `crates/neko-cli/tests/multistream.rs::bounded_wait_with_output` both:

1. take piped stdout/stderr from an owned direct child;
2. spawn concurrent `read_to_end` reader threads;
3. bound **direct-child** exit via `try_wait` / kill / bounded reap;
4. once direct-child exit is observed, unconditionally `join()` both reader threads.

The helpers currently assume direct-child exit implies the pipe writers are all gone. That is false when a descendant inherited stdout/stderr. A direct child can exit and be reaped while a descendant keeps a pipe write descriptor open; the reader thread remains blocked in `read_to_end`, and the post-exit `join()` is no longer covered by the helper deadline. H-I4-106/107/109 hardened direct child ownership and pipe-full draining but did not establish descendant-writer EOF.

Exact-current `neko-cli` production source does not launch external subprocesses, so this is **test-harness boundedness/evidence reliability**, not a production process tree, Session, Carrier, ACK, crypto or wire finding. It matters because the helper is the outer harness termination proof for networked process tests: a wrapper or lifecycle regression that creates an inherited writer can turn a supposed bounded negative into a stranded gate.

Reviewer source inspection found the same seam in both helper copies. A reviewer-local generic Linux process-semantics check (not repository CI) ran the equivalent of `sh -c 'sleep 3 & exit 0'` with stdout/stderr piped: the direct shell exited in ~0.001s while pipe EOF arrived only ~3.001s later, after the descendant closed its inherited writer. This is only a proof of the OS ownership premise; it is not exact-tree Rust/full-gate evidence.

### Closure contract

1. Add a deterministic inherited-writer negative regression: the direct child exits promptly while a finite synthetic descendant keeps stdout and/or stderr open past the helper deadline. The harness must return/fail within the declared bound rather than block on a reader join. Keep the synthetic descendant finite and cleanup-safe.
2. Repair **both** `probe.rs` and `multistream.rs` bounded-output helpers, or use a genuinely smaller shared shape only if it avoids framework churn. Direct-child exit alone must not authorize an unbounded `JoinHandle::join()`.
3. Preserve concurrent draining while the direct child is live so the existing pipe-full deadlock protection remains. Preserve current `try_wait` / kill / bounded-reap failure classification; do not regress H-I4-107/109.
4. Reader completion after direct-child exit must itself be causally bounded. Reader-completion timeout is explicit harness failure, never successful evidence or silent output loss. Do not introduce production process-group/session policy merely to repair test ownership.
5. Do not mechanically convert fail-fast keygen/help/capabilities/invalid-config/socket-free fixtures. Do not change Session/Carrier/ACK/crypto/wire semantics or invent repository-wide timeout/capacity/security values.
6. No decoder/parser/crypto-framing implementation change is implicated; do not run fuzz mechanically.
7. On the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist reachable developer-local exact-tree provenance with exact SHA, UTC start/end, exit codes, OS/arch, stable Rust and clean-tree state.
8. Continue immediately to the next READY slice after closure; do not wait for reviewer cadence and do not infer repository-wide queue exhaustion from this helper repair.

## Rolling queue — keep continuous after H-I4-115

Do not collapse this to one ticket. Dependency-ready order:

1. **H-I4-115 repair + deterministic inherited-writer regression + exact-tree provenance.** Keep scope test-harness-local; no production process policy.
2. **Remaining process/socket/thread ownership causal sweep.** Continue exact-current `crates/neko-cli/tests/probe.rs` and `crates/neko-cli/tests/multistream.rs`, including direct `.output()` / `wait*`, `read_exact` / `read` / `recv` / `recv_from` / `accept`, output-drain completion, reader/thread joins and child/descendant ownership. Classify each as source-self-bounded, externally bounded, exit/peer-completion proven, or concrete unbounded owner. Repair only the last class. Keep keygen/help/capabilities/invalid-config/socket-free fail-fast exclusions unless source proves otherwise.
3. **Cross-platform CLI/process factual reconciliation.** Reconcile I4-CLI-PROC-096 and H-I4-097..115 against exact-current helper/cfg/socket/process semantics. Separate Linux execution evidence from macOS/BSD source reasoning and Windows claims. Do not claim Unix descendant/pipe behavior as Windows proof.
4. **Algorithmic/resource boundedness reconciliation.** Reconcile accepted boundedness reviews with channel/output bounds, stdout/stderr accumulation, cleanup deadlines, socket peer lifetime, reader lifetime, direct-child vs descendant ownership and thread completion. No capacity-pressure benchmark and no invented policy values.
5. **Release packet / item-4 factual reconciliation.** The packet is an evidence index and currently predates this process-ownership repair cluster. Qualify any broad statement that process failure paths are globally bounded; index H-I4-097..115 only after the relevant final exact-tree provenance is reachable. Local process/socket tests are not WAN/performance/security approval.
6. **Pre-auth malformed/rejection accounting exact-current reuse challenge.** Reuse prior independent review only where owner source/tests remain unchanged; narrowly re-challenge changed owners. D019 remains policy blocked.
7. **CLI diagnostic / exit-code / JSON / human-output contract exact-current reuse challenge.** Diagnostics remain evidence-only; never promote them to authentication, Session Delivery, Path or ACK evidence.
8. **Package/build/reproducibility spot re-challenge.** Verify recent process-test repairs do not stale manifests, lock/features, scripts, package provenance or native-hook/unsafe assumptions. Do not invent signing/SBOM/key-custody/publication policy.
9. **Reliable UDP / CarrierState / CarrierManager / FairScheduler / SessionRuntime targeted spot re-challenge.** Re-open materially changed owners; otherwise record exact-current reuse boundaries rather than rerunning equivalent deep sweeps. Candidate A remains closed unless current source falsifies it.
10. **Observability + carrier adapters + dependency/build exact-current reuse challenge.** Same owner-diff rule. Candidate B remains closed unless current source falsifies it. No checker/schema/framework/docs filler.
11. **Repository-wide 13-surface refill.** Re-apply every required core surface after the HIGH and dependent reconciliation close. Queue exhaustion is legal only if the broad inventory finds no concrete defect, no uncovered implemented core surface, no READY review-support and no READY live question, with all remaining work genuinely external/policy/environment/release-authority gated.
12. **Conditional live.** Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved real-network question within standing authorization. Otherwise keep `READY_LIVE: none`.

If several coherent slices complete in 10–30 minutes with stable quality, deepen the queue and continue implementation/review -> tests -> commit -> push -> next slice without waiting. Every 3–4 coherent slices or important repair cluster, perform one factual item-4/release reconciliation rather than rewriting large docs after every small commit.

## Required 13-surface refill inventory

At each meaningful repository-wide refill, explicitly ask whether each surface has a reachable, dedicated, independent bounded review that remains valid on exact-current owners:

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

- Developer-reported local CI, repository-persisted developer-local provenance, reviewer source/control-flow review, reviewer-local generic OS checks, hosted CI, live WAN evidence and performance conclusions are distinct evidence classes.
- Accepted exact-tree provenance must anchor a GitHub-resolvable pushed SHA. Never publish a local-only/unreachable SHA as shared exact-tree evidence.
- Exact `46d624e` has both reachable developer-local provenance and a separate successful hosted Rust CI workflow. Neither is reviewer-local Rust execution.
- H-I4-115 reviewer work is source/control-flow inspection plus a generic Linux pipe-inheritance check only. Reviewer did not run the repository Rust/full gate, cross-platform runtime, fuzz, WAN or performance work in this pass.
- Hosted Actions are cross-evidence, not a wait condition and not a replacement for developer-local clean exact-tree validation.
- Fuzz only when wire decoder/parser/crypto framing changes materially.
- Standing VPS authorization permits bounded self-owned TCP/UDP lab work, but authoritative classification remains `READY_LIVE: none`; no repeat live run without a new concrete question.
- Never decide D019, capacity/TTL/LRU/history/security values, signing/key-custody/SBOM/publication policy, core Session/Carrier/ACK/crypto/wire architecture, destructive/canonical migration, RC/freeze/release/production authority.
