# ChatGPT reviewer handoff — H-I4-093 FRONT HIGH: BarrierProof binding must remain non-Linux-Unix warning-clean

## Current repository truth

- Synchronize to current `main` before work. Code/tests/reachable pushed commits outrank this prose and older review notes.
- Reviewer re-read the required governance/spec/status/release surface and reviewed all developer-owned commits after the previous handoff anchor `a89118d1d7fe11e4c5bc3b83bbdbcb47d978b4c9`.
- Developer source/test commit reviewed:
  - `28e1caa6a68b7012ed7126e5e448608843ba7a6f` — `malformed_classification_barrier(...)` now returns `BarrierProof` on `Ok`; the Linux post-resource snapshot consumes that returned value inside `gated_resource_snapshot(...)`.
  - Developer-reported clean exact-tree provenance is persisted in `docs/notes/h-i4-092-provenance-28e1caa-20260922.md`: `scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, 2026-09-22T02:50:44Z → 02:56:35Z, Linux x86_64, rustc 1.98.0 stable.
  - `0b15ef3e3769267dcf53cbcb4c50756f6aaa5d3c` — handoff/provenance reconciliation only.
  - No visible hosted status/workflow result for `28e1caa` is classified only as absence of hosted evidence, not CI failure.
- **H-I4-092's Linux causal proof seam is accepted/closed.** The exact-current Linux path is `READY -> affirmative pre-churn snapshot -> churn starts before first malformed send -> bounded server-side classification barrier after malformed cleanup -> barrier-derived proof -> affirmative post-churn snapshot`. Pre/post phase proof and measurement are coupled inside the snapshot helper.
- **H-I4-093 is CLOSED** at `b103c9784805b48f8b488d0af705c940c1f82993`: `#[cfg(not(target_os = "linux"))] let _ = barrier_proof;` explicitly sinks the proof binding on non-Linux Unix so `cargo clippy --workspace --all-targets --locked -- -D warnings` stays warning-clean on unix && !linux, while the Linux `/proc` gate still consumes `barrier_proof` inside the coupled snapshot call. Exact-tree provenance: `docs/notes/h-i4-093-provenance-b103c97-20260922.md` — `check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-22T03:50:28Z → 03:56:25Z, Linux x86_64, rustc 1.98.0.
- This finding is a cross-platform test/evidence defect, not a demonstrated transport/resource leak. Keep the accepted Linux `/proc` observation and H-I4-090/H-I4-091 source ordering intact.
- Candidate A (future/unsent Recovery ACK), Candidate B (mixed datagram drop reasons), H-I4-085..092 runtime/source findings, prior R9 process/result seams, Session/Carrier/ACK separation, CarrierState/CarrierManager, FairScheduler/flow accounting, carrier adapters, observability, package/operator, dependency/build, and prior CLI machine/human contracts remain closed unless materially changed or falsified by a concrete new counterexample.
- `SessionRuntime.events` retained-history capacity remains a maintainer/security policy gate; do not invent TTL/LRU/history-size/capacity values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat prior WAN/VPS evidence merely for freshness.

## FRONT HIGH — H-I4-093 non-Linux Unix BarrierProof warning cleanliness

Do not redo the already-correct Linux baseline ordering, bounded stdout wait, EOF cleanup, post-release diagnostic ordering, or H-I4-092 barrier-derived proof semantics. Close only the target-specific binding seam:

1. Preserve the exact-current Linux post-barrier proof: the post `/proc` resource snapshot must still require the value returned by successful `malformed_classification_barrier(...)`. Do not replace it with an arbitrary `FnOnce`, constant-true sentinel, no-op gate, or manually forged substitute.
2. On `unix && !target_os="linux"`, ensure the returned proof is explicitly consumed/sunk or otherwise destructured in a warning-clean shape while the portable socket/lifecycle portion of `udp_listener_rejects_bounded_malformed_churn_then_authenticates_and_cleans_up` continues to run.
3. Smallest acceptable repair is test-only/cfg-only. A target-specific `let _ = barrier_proof;` sink or equivalent is preferable to new framework/schema machinery. Do not alter runtime semantics.
4. Preserve H-I4-090/H-I4-091 invariants: affirmative Linux baseline after READY and before first malformed send; `churn_started` before the first send; bounded off-thread classification wait; timeout/disconnect/EOF kill/reap + reader join; `malformed_or_unadmitted` only after `preauth.release(admission)`; existing FD/RSS margins and `ATTEMPTS`; missing-event and early-EOF regressions.
5. Do not change Session/Carrier/ACK/crypto/wire architecture, D019, capacity/security/TTL/LRU/history numbers, release flags or publication/signing policy.
6. No fuzz unless decoder/parser/crypto framing owners change.
7. Run focused CLI process tests, then on the **final pushed source/test SHA** run:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - verify clean tree
   Persist exact reachable SHA, UTC start/end, exit codes, OS/arch, `rustc --version`, and clean-tree state.
8. If a real macOS/BSD/non-Linux-Unix runner/toolchain is available, an all-target clippy/check there is useful cross-evidence. If it is not available, do not claim it ran; retain static cfg reasoning separately from Linux developer-local provenance.
9. After closure continue immediately through the rolling queue; do not wait for reviewer cadence.

## Rolling queue — keep continuous after the HIGH

Maintain a real dependency-ordered queue; do not collapse to a one-ticket idle state:

1. **H-I4-093 cfg-safe proof binding + exact-tree provenance.** Smallest test/cfg repair only.
2. **I4-PORT-RES exact-current independent causal re-challenge.** Re-attempt both causal edges after the cfg repair: `READY -> affirmative pre-churn snapshot -> first malformed send` and `all bounded malformed inputs classified + corresponding cleanup complete -> barrier-derived proof -> affirmative post-churn snapshot`. Try whole-block and split/oracle mutations; a scoped no-finding is valid only if both fail closed. Reuse accepted H-I4-090..092 facts only where owners are unchanged.
3. **Pre-auth malformed/rejection resource-accounting bounded challenge.** Inspect exact-current `admit_carrier`, `charge_input`, invalid-negotiation release, response-admission rejection, cached/pending owner paths and process-level pre-auth counters. Verify rejection cannot become authentication/delivery evidence and diagnostics do not retain unbounded state. Do not choose D019 policy values.
4. **Cross-platform CLI/process-test semantics.** Re-challenge Linux-only `/proc` cfgs, non-Linux Unix warning cleanliness, process/signal assumptions, reader-thread lifecycle, child kill/reap/join ownership, timeout/EOF/disconnect behavior and test portability. Windows/native execution is not to be invented; distinguish unsupported/unexecuted targets from source-shape defects.
5. **CLI diagnostic/machine-output boundary.** Challenge `malformed_or_unadmitted` scope, JSON/human-output collision, diagnostic parsing stability, exit-code coupling and evidence-only versus protocol semantics.
6. **Algorithmic/resource boundedness reconciliation.** Challenge channel capacity/lifetime, timeout/EOF/disconnect cleanup, bounded line accumulation, queue/counter growth and test-local resource bounds. No capacity-pressure benchmark and no new policy numbers.
7. **Package/build + reproducibility spot re-challenge.** Reuse prior no-finding only for unchanged owners; verify recent CLI test/helper edits did not make package/provenance claims stale. Do not invent signing/SBOM/key-custody policy.
8. **Item-4 + release-packet factual reconciliation.** Reconcile the final resource-oracle closure and current bounded reviews with `docs/release-security-review-packet.md`, `docs/status.md`, `IMPLEMENTATION_PLAN.md`, provenance notes and review notes. Do not promote local resource testing into WAN/performance/security approval.
9. **Repository-wide 13-surface refill.** Re-apply the required core inventory: reliable UDP recovery; CarrierState; CarrierManager/health/migration-back; FairScheduler/flow accounting; carrier adapters; SessionRuntime; observability; package/operator; dependency/build; cross-platform process tests; CLI output contract; algorithmic boundedness; release-packet factual consistency. Reuse prior reviews only for genuinely unchanged owners. Queue exhaustion is legal only after the broad inventory finds no concrete defect, no uncovered implemented core surface, no READY review-support and no READY live question.
10. **Conditional live.** Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved real-network question within standing authorization. Otherwise keep `READY_LIVE: none`.

If several coherent slices complete in 10–30 minutes with stable quality, deepen the queue rather than waiting. Every 3–4 coherent slices or important repair cluster, perform one factual release/item-4 reconciliation instead of rewriting large docs after each small commit.

## VPS / live boundary

Current classification remains `READY_LIVE: none`.

- IPv6 remains environment-blocked unless a real owned IPv6 endpoint/path appears.
- HY2 and repeated warm-failover current lines remain frozen against same-class retry without a materially new hypothesis.
- Periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD and Experimental Track are not repeated for freshness when their bounded question is already answered or their current line is frozen/blocked.
- Standing authorization permits bounded self-owned client↔VPS ordinary TCP/UDP work only when a real new question becomes READY; it does not authorize third-party targets, production route/firewall/DNS/proxy/tunnel/qdisc changes, privileged/exotic-carrier work reserved by the authorization file, or pressure/adversarial-load conditions requiring maintainer choice.

## Review / repair and evidence contract retained

For every bounded lane: read exact-current owners/tests plus applicable spec/ADR/status claim; state the invariant; try to falsify it with source reasoning and focused deterministic tests. If a concrete defect exists and committed semantics determine the answer, apply the smallest repair, add positive/negative regression where meaningful, commit/push, run exact-tree gates, persist provenance and continue.

No-finding slices must name inspected owners, commands/tests actually run, exclusions and exact reachable anchor. Do not modify code merely to manufacture activity. Hosted CI is additional cross-evidence, never a waiting condition. Developer-local, reviewer-local, hosted, live WAN and performance evidence remain distinct classes.

## Stop / escalation conditions

Normal progress does not require administrator notification. Escalate only for an automatically undecidable BLOCKER/HIGH, core Session/Carrier/ACK/crypto/wire architecture decision, D019 or another real policy/value choice, destructive/canonical migration, action beyond standing authorization, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
