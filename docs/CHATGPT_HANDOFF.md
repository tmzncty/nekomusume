# ChatGPT reviewer handoff — H-I4-094 FRONT HIGH: pre-churn baseline must require server-start proof

## Current repository truth

- Synchronize to current `main` before work. Code/tests/reachable pushed commits outrank this prose and older review notes.
- Reviewer re-read the required governance/spec/status/release surface and reviewed all developer-owned commits after the prior H-I4-093 review anchor.
- Developer source/test commit reviewed:
  - `b103c9784805b48f8b488d0af705c940c1f82993` — H-I4-093 explicitly sinks `barrier_proof` on non-Linux Unix while Linux still consumes the barrier-derived proof inside the post-`/proc` resource measurement.
  - Developer-reported clean exact-tree provenance is persisted in `docs/notes/h-i4-093-provenance-b103c97-20260922.md`: `scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, 2026-09-22T03:50:28Z → 03:56:25Z, Linux x86_64, rustc 1.98.0 stable.
  - `76a04b6b00201032b000646ed497dc98adb4fa4b` — handoff/provenance reconciliation only.
  - No visible hosted status for `b103c97` is classified only as absence of hosted evidence, not CI failure.
- **H-I4-093 is CLOSED** at `b103c97`. The non-Linux Unix source shape is warning-clean by static cfg reasoning; do not claim macOS/BSD execution that did not occur.
- **H-I4-092's post-churn causal proof remains accepted.** The post Linux `/proc` snapshot requires the `BarrierProof` produced only after successful bounded malformed classification; timeout/disconnect/EOF cleanup and post-`preauth.release(...)` diagnostic ordering remain unchanged.
- **H-I4-094 is CLOSED** at `ca772093f4fc46bda3be9aade00d4fc884b48797`: `ReadyServer` carries a private-constructor `ReadyProof` obtainable only from `ready_failover_server` / `ready_endpoint_rebind_server` / `start_server_for`, and the pre-churn `gated_resource_snapshot` destructures it inside the gate — moving the baseline above the server-start barrier fails to compile, closing the readiness-proof gap flagged by `7bfb4c9`. The post edge remains barrier-derived via `BarrierProof` (H-I4-092). Exact-tree provenance: `docs/notes/h-i4-094-provenance-ca77209-20260922.md` — `check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-22T04:50:53Z → 04:56:50Z, Linux x86_64, rustc 1.98.0.
- Candidate A (future/unsent Recovery ACK), Candidate B (mixed datagram drop reasons), prior H-I4/R9 runtime/source findings, Session/Carrier/ACK separation, CarrierState/CarrierManager, FairScheduler/flow accounting, carrier adapters, observability, package/operator, dependency/build, and prior CLI machine/human contracts remain closed unless materially changed or falsified by a concrete new counterexample.
- `SessionRuntime.events` retained-history capacity remains a maintainer/security policy gate; do not invent TTL/LRU/history-size/capacity values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat prior WAN/VPS evidence merely for freshness.

## FRONT HIGH — H-I4-094 readiness-derived pre-baseline proof

The current code physically waits for the failover-server diagnostic `start` and then takes the Linux baseline before malformed churn. The problem is mechanical protection: `pid` exists before readiness and `gated_resource_snapshot(|| assert!(!churn_started), pid)` has no dependency on the value returned by `ready_failover_server(...)`. A whole-block refactor can therefore move the baseline above the start barrier without tripping the current oracle.

Close only this causal seam:

1. Add the smallest test-only proof/value obtainable only after `ready_failover_server(...)` has observed the server diagnostic `start`, or an equivalent typed/ownership coupling. A private `ReadyProof` returned alongside `ReadyServer` is one acceptable shape; do not create a general framework.
2. On Linux, the **actual pre-churn `/proc` snapshot acquisition must consume that readiness-derived proof inside the same helper call that invokes `process_resource_snapshot(pid)`**. Moving the measurement above the start barrier must fail to compile or deterministically fail.
3. Preserve the existing `!churn_started` gate in that measurement path so the baseline also remains before the first malformed send.
4. Preserve the accepted post edge unchanged: all `ATTEMPTS` malformed inputs must be classified after cleanup, successful barrier returns `BarrierProof`, and the Linux post snapshot consumes that proof; non-Linux Unix must remain warning-clean.
5. Preserve bounded off-thread stdout wait; timeout/disconnect/EOF kill+reap + reader join; post-release `malformed_or_unadmitted`; existing FD/RSS margins and `ATTEMPTS`; existing missing-event and early-EOF regressions.
6. Do not change runtime Session/Carrier/ACK/crypto/wire semantics, D019, capacity/security/TTL/LRU/history numbers, release flags, signing/SBOM/publication policy, or previous frozen decisions.
7. No fuzz unless decoder/parser/crypto framing owners change.
8. Run focused CLI process tests, then on the **final pushed source/test SHA** run:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - verify clean tree
   Persist exact reachable SHA, UTC start/end, exit codes, OS/arch, `rustc --version`, and clean-tree state.
9. After repair/provenance, immediately complete the I4-PORT-RES causal re-challenge. Do not wait for reviewer cadence.

## Rolling queue — keep continuous after the HIGH

Maintain a real dependency-ordered queue; do not collapse to a one-ticket idle state:

1. **H-I4-094 readiness-derived pre-baseline proof + exact-tree provenance.** Smallest test-only coupling repair.
2. **I4-PORT-RES exact-current causal re-challenge.** Re-attempt both causal edges on the repaired tree: `server start -> affirmative pre-churn snapshot -> first malformed send` and `all bounded malformed inputs classified + corresponding cleanup complete -> BarrierProof -> affirmative post-churn snapshot`. Try whole-block and split/oracle mutations. A scoped no-finding is valid only if both edges fail closed under those mutations.
3. **Pre-auth malformed/rejection resource-accounting bounded challenge.** Inspect exact-current `admit_carrier`, `charge_input`, invalid-negotiation release, response-admission rejection, cached/pending owners, expiry, queue reservations and process-level counters. Verify rejection cannot become authentication/delivery evidence and diagnostics do not retain unbounded state. Do not choose D019 policy values.
4. **Cross-platform CLI/process-test semantics.** Re-challenge Linux-only `/proc` cfgs, non-Linux Unix warning cleanliness, process/signal assumptions, reader-thread lifecycle, child kill/reap/join ownership, timeout/EOF/disconnect behavior and test portability. Do not invent Windows/macOS/BSD execution.
5. **CLI diagnostic/machine-output boundary.** Challenge `malformed_or_unadmitted` scope, JSON/human-output collision, diagnostic parsing stability, exit-code coupling and evidence-only versus protocol semantics.
6. **Algorithmic/resource boundedness reconciliation.** Challenge channel capacity/lifetime, timeout/EOF/disconnect cleanup, bounded line accumulation, queue/counter growth and test-local resource bounds. No capacity-pressure benchmark and no new policy numbers.
7. **Package/build + reproducibility spot re-challenge.** Reuse prior no-finding only for unchanged owners; verify recent CLI test/helper edits did not make package/provenance claims stale. Do not invent signing/SBOM/key-custody policy.
8. **Item-4 + release-packet factual reconciliation.** Reconcile H-I4-090..094 closure and current bounded reviews with `docs/release-security-review-packet.md`, `docs/status.md`, `IMPLEMENTATION_PLAN.md`, provenance notes and review notes. Do not promote local resource testing into WAN/performance/security approval.
9. **Repository-wide 13-surface refill.** Re-apply the required core inventory: reliable UDP recovery; CarrierState; CarrierManager/health/migration-back; FairScheduler/flow accounting; carrier adapters; SessionRuntime; observability; package/operator; dependency/build; cross-platform process tests; CLI output contract; algorithmic boundedness; release-packet factual consistency. Reuse prior reviews only for genuinely unchanged owners. Queue exhaustion is legal only after the broad inventory finds no concrete defect, no uncovered implemented core surface, no READY review-support and no READY live question.
10. **Conditional live.** Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved real-network question within standing authorization. Otherwise keep `READY_LIVE: none`.

If several coherent slices complete in 10–30 minutes with stable quality, deepen the queue rather than waiting. Every 3–4 coherent slices or important repair cluster, perform one factual release/item-4 reconciliation instead of rewriting large docs after each small commit.

## VPS / live boundary

Current classification remains `READY_LIVE: none`.

- IPv6 remains environment-blocked unless a real owned IPv6 endpoint/path appears.
- HY2 and repeated warm-failover current lines remain frozen against same-class retry without a materially new hypothesis.
- Periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track are not repeated for freshness when their bounded question is already answered or their current line is frozen/blocked.
- Standing authorization permits bounded self-owned client↔VPS ordinary TCP/UDP work only when a real new question becomes READY; it does not authorize third-party targets, production route/firewall/DNS/proxy/tunnel/qdisc changes, privileged/exotic-carrier work reserved by the authorization file, or pressure/adversarial-load conditions requiring maintainer choice.

## Review / repair and evidence contract retained

For every bounded lane: read exact-current owners/tests plus applicable spec/ADR/status claim; state the invariant; try to falsify it with source reasoning and focused deterministic tests. If a concrete defect exists and committed semantics determine the answer, apply the smallest repair, add positive/negative regression where meaningful, commit/push, run exact-tree gates, persist provenance and continue.

No-finding slices must name inspected owners, commands/tests actually run, exclusions and exact reachable anchor. Do not modify code merely to manufacture activity. Hosted CI is additional cross-evidence, never a waiting condition. Developer-local, reviewer-local, hosted, live WAN and performance evidence remain distinct classes.

## Stop / escalation conditions

Normal progress does not require administrator notification. Escalate only for an automatically undecidable BLOCKER/HIGH, core Session/Carrier/ACK/crypto/wire architecture decision, D019 or another real policy/value choice, destructive/canonical migration, action beyond standing authorization, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
