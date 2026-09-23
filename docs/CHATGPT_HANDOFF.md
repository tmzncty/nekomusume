# ChatGPT reviewer handoff — H-I4-115 source accepted; provenance + ownership sweep FRONT

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- Latest developer-owned source/test anchor independently reviewed this pass: `1f6d744c9954b9a7a5d693b43a20d137a5c5e75d` (`test(cli): H-I4-115 descendant-held pipe bound in bounded_wait_with_output`). It changes only `crates/neko-cli/tests/probe.rs` and `crates/neko-cli/tests/multistream.rs`.
- Independent reviewer note: `docs/reviews/independent-h-i4-115-bounded-output-1f6d744-20260924.md` (reviewer commit `eb498e16a6e2fcb4d58a61dd4423ef33bb6abb4b`).
- **H-I4-115 source repair is accepted at `1f6d744...` for the bounded-caller invariant.** Both helper copies now keep concurrent pipe draining but route reader completion through bounded channels; after direct-child exit/reap, descendant-held stdout/stderr produces an explicit finite harness failure rather than an unbounded reader `join()`. The Unix regression directly challenges the inherited-writer premise with a finite descendant.
- **H-I4-115 is not fully evidence-closed yet.** At reviewer time there was no reachable developer-local exact-tree provenance for `1f6d744...` (or a later unchanged-source anchor), and exact `1f6d744...` had no GitHub combined-status entries. Do not convert reviewer source inspection into developer-local or hosted CI evidence.
- The synthetic descendant in the negative regression is finite but can outlive the assertion for part of its finite sleep interval. This does not reintroduce the caller hang and is not a new production/process-tree finding. Only reopen this as a separate narrow harness issue if actual leaked-process interference is demonstrated; do not manufacture process-group policy.
- H-I4-113/114 remain closed at exact source/test `46d624e2272fcd5cf9d86457d018c4f508b8ba10` with reachable developer-local provenance and separately recorded hosted Rust CI. H-I4-111/112 remain closed at `fc23a7a88c917e116f1140333faa6d8f441ca557`; H-I4-107/108/109 remain accepted/provenanced through `4c445ce36aa06edc1b3d4952036073865e3f20d0`; H-I4-110 remains closed at `259495fad9949a526b12187ba59cf74aec0ee7f7`.
- Candidate A (`Recovery::on_ack` future/unsent ACK) and Candidate B (`record_datagrams` mixed drop reasons) remain closed unless materially changed or falsified by exact-current source/tests.
- `SessionRuntime.events` retained-history capacity and D019 source-retention/no-reset remain maintainer/security policy gates. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely for freshness.

## FRONT 1 — finish H-I4-115 exact-tree provenance, then continue immediately

The source defect is no longer open; this lane is evidence closure only.

1. On exact developer source/test `1f6d744...` or a later reachable commit whose relevant source is unchanged, run in a safe clean checkout/worktree:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - confirm clean tree.
2. Persist reachable developer-local provenance with exact SHA, UTC start/end, exit codes, OS/arch, stable Rust and clean-tree state. Do not record secrets, private addresses/topology, credentials or unnecessary absolute paths.
3. Keep evidence classes separate. GitHub Actions, if any appear later, are cross-evidence only and not a wait condition.
4. No decoder/parser/crypto-framing implementation changed; do not run fuzz mechanically.
5. After provenance is pushed, continue directly to FRONT 2 without waiting for reviewer cadence.

## FRONT 2 — remaining process/socket/thread ownership causal sweep

Continue the exact-current sweep instead of declaring queue exhaustion because the H-I4-115 seam is repaired.

### Scope

Inspect current `crates/neko-cli/tests/probe.rs`, `crates/neko-cli/tests/multistream.rs`, their invoked CLI owner paths, and any newly changed helpers. For each candidate wait, classify it as one of:

- source-self-bounded;
- externally/harness-bounded with a proven owner;
- peer/child completion causally proven;
- concrete unbounded owner.

Challenge at least:

- remaining direct `.output()` / `wait*` uses;
- `read_exact` / `read` / `recv` / `recv_from` / `accept` paths;
- output-drain completion and reader/thread joins;
- direct-child versus descendant ownership;
- socket peer lifetime and cleanup after negative paths.

Keep keygen/help/capabilities/invalid-config/socket-free fail-fast calls excluded unless exact source proves they can block on an external owner.

### Important policy boundary discovered during source reasoning

`crates/neko-cli/src/multistream.rs` contains authenticated application-level blocking socket reads after the pre-auth/Noise stage. That observation by itself is **not** authorization to invent an application idle timeout, duration, capacity or security number. Before treating it as a defect, read exact-current owner source plus applicable spec/ADR/status/security claims and decide whether the repository already fixes the required liveness semantics. If current semantics do not decide a timeout, classify it explicitly as an operational/policy boundary or maintainer gate rather than manufacturing a numeric repair. Outer process tests being bounded also must not be misreported as proof that the production/lab fixture recursively owns every authenticated peer lifetime.

If a concrete defect is found and current committed semantics already determine the answer, do the smallest repair + focused positive/negative regression + exact-tree gate + provenance. If no defect is found, write a bounded no-finding note identifying exact owners, challenged invariant, checks/reasoning, exclusions and reachable anchor; do not create framework/checker/schema filler.

## Rolling queue — keep continuous after FRONT 1/2

Do not collapse this to one ticket. Dependency-ready order:

1. **H-I4-115 exact-tree provenance closure** at `1f6d744...` or a later unchanged-source anchor.
2. **Remaining process/socket/thread ownership causal sweep** as above; repair only proven unbounded owners.
3. **Cross-platform CLI/process factual reconciliation.** Reconcile I4-CLI-PROC-096 and H-I4-097..115 against exact-current helper/cfg/socket/process semantics. Separate Linux execution evidence from macOS/BSD source reasoning and Windows claims. Unix inherited-pipe behavior is not Windows proof.
4. **Algorithmic/resource boundedness reconciliation.** Reconcile accepted boundedness reviews with channel/output bounds, stdout/stderr accumulation, cleanup deadlines, socket peer lifetime, authenticated application reads, reader lifetime, direct-child versus descendant ownership and thread completion. No capacity-pressure benchmark and no invented policy values.
5. **Release packet / item-4 factual reconciliation.** The packet is an evidence index, not approval. Qualify any broad statement that process failure paths are globally bounded; index H-I4-097..115 only after relevant exact-tree provenance is reachable. Local process/socket tests are not WAN/performance/security approval.
6. **Pre-auth malformed/rejection accounting exact-current reuse challenge.** Reuse prior independent review only where owner source/tests remain unchanged; narrowly re-challenge changed owners. D019 remains policy blocked.
7. **CLI diagnostic / exit-code / JSON / human-output contract exact-current reuse challenge.** Diagnostics remain evidence-only; never promote them to authentication, Session Delivery, Path or ACK evidence.
8. **Package/build/reproducibility spot re-challenge.** Verify recent process-test repairs do not stale manifests, lock/features, scripts, package provenance or native-hook/unsafe assumptions. Do not invent signing/SBOM/key-custody/publication policy.
9. **Reliable UDP / CarrierState / CarrierManager / FairScheduler / SessionRuntime targeted spot re-challenge.** Re-open materially changed owners; otherwise record exact-current reuse boundaries rather than rerunning equivalent deep sweeps. Candidate A remains closed unless current source falsifies it.
10. **Observability + carrier adapters + dependency/build exact-current reuse challenge.** Same owner-diff rule. Candidate B remains closed unless current source falsifies it. No checker/schema/framework/docs filler.
11. **Repository-wide 13-surface refill.** Queue exhaustion is legal only if the broad inventory finds no concrete defect, no uncovered implemented core surface, no READY review-support and no READY live question, with all remaining work genuinely external/policy/environment/release-authority gated.
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

A bounded no-finding review is valid item-4 support when it identifies inspected owners, challenged invariant, deterministic checks/commands or source reasoning, exclusions and an exact reachable anchor. Do not create checker/schema/framework/docs churn merely to manufacture a slice.

## Evidence discipline

- Developer-reported local CI, repository-persisted developer-local provenance, reviewer source/control-flow review, reviewer-local generic OS checks, hosted CI, live WAN evidence and performance conclusions are distinct evidence classes.
- Accepted exact-tree provenance must anchor a GitHub-resolvable pushed SHA. Never publish a local-only/unreachable SHA as shared exact-tree evidence.
- Exact `1f6d744...` H-I4-115 review is source/control-flow inspection only. At review time it had no combined-status entries and no reachable developer-local exact-tree provenance. The reviewer did not run repository Rust/full-gate, cross-platform runtime, fuzz, WAN or performance work in this pass.
- Exact `46d624e...` has both reachable developer-local provenance and a separate successful hosted Rust CI workflow; neither is reviewer-local Rust execution.
- Hosted Actions are cross-evidence, not a wait condition and not a replacement for developer-local clean exact-tree validation.
- Fuzz only when wire decoder/parser/crypto framing changes materially.
- Standing VPS authorization permits bounded self-owned TCP/UDP lab work, but authoritative classification remains `READY_LIVE: none`; no repeat live run without a new concrete question.
- Never decide D019, capacity/TTL/LRU/history/security values, signing/key-custody/SBOM/publication policy, core Session/Carrier/ACK/crypto/wire architecture, destructive/canonical migration, RC/freeze/release/production authority.
