# ChatGPT reviewer handoff — H-I4-095 FRONT HIGH: `ReadyProof` must exist on non-Unix builds

## Current repository truth

- Synchronize to current `main` before work. Code/tests/reachable pushed commits outrank this prose and older review notes.
- Reviewer re-read the required governance/spec/status/release surface and reviewed all developer-owned commits after the prior H-I4-093 review anchor.
- H-I4-093 is closed at `b103c9784805b48f8b488d0af705c940c1f82993`: non-Linux Unix explicitly sinks `barrier_proof`, while Linux still consumes the barrier-derived proof inside the post-`/proc` measurement. Developer-reported clean exact-tree provenance remains local evidence only; do not claim macOS/BSD execution.
- H-I4-094 is closed for its **Linux causal intent** at `ca772093f4fc46bda3be9aade00d4fc884b48797`: `ReadyServer.ready_proof` is produced after each current ready helper's readiness/start observation, and the Linux pre-churn `gated_resource_snapshot(...)` consumes it together with `!churn_started`. Its developer-reported exact-tree gate is persisted in `docs/notes/h-i4-094-provenance-ca77209-20260922.md` (`scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, Linux x86_64, rustc 1.98.0 stable). No visible hosted status is only absence of hosted evidence.
- **New H-I4-095 HIGH is open** at review commit `6ef521e2f1e9453f47348b68bf2b2e29533a962b`: `ReadyServer` unconditionally contains `ready_proof: ReadyProof` and general helpers unconditionally construct it, but `ReadyProof` itself is declared only under `#[cfg(unix)]`. A non-Unix target compiling `crates/neko-cli/tests/probe.rs` therefore has a deterministic missing-type/name-resolution failure. This is item-4 build/portability correctness, not a runtime transport defect and not a claim that Windows is an RC target.
- H-I4-090..094 causal/resource invariants otherwise remain accepted unless falsified by a concrete new counterexample: READY/start before baseline; baseline measurement coupled to readiness proof and `!churn_started`; first malformed send only after churn phase starts; each malformed classification emitted after `preauth.release(...)`; bounded off-thread classification wait; timeout/disconnect/EOF cleanup; `BarrierProof` only on successful complete barrier; Linux post snapshot consumes that proof; non-Linux Unix remains warning-clean.
- Candidate A (future/unsent Recovery ACK), Candidate B (mixed datagram drop reasons), prior H-I4/R9 runtime/source findings, Session/Carrier/ACK separation, CarrierState/CarrierManager, FairScheduler/flow accounting, carrier adapters, observability, package/operator, dependency/build and prior CLI machine/human contracts remain closed unless materially changed or falsified by a concrete new counterexample.
- `SessionRuntime.events` retained-history capacity remains a maintainer/security policy gate; do not invent TTL/LRU/history-size/capacity values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat prior WAN/VPS evidence merely for freshness.

## FRONT HIGH — H-I4-095 non-Unix `ReadyProof` cfg closure

Exact-current `crates/neko-cli/tests/probe.rs` has:

```rust
struct ReadyServer {
    ...
    ready_proof: ReadyProof,
}

#[cfg(unix)]
struct ReadyProof { ... }
```

and unconditional constructions in general helpers such as `start_server_for(...)` / `start_periodic_server(...)`. The integration-test target itself is not target-gated in `crates/neko-cli/Cargo.toml`. Therefore a non-Unix Rust target sees the field/initializers but not the type.

Close this with the smallest test-only/cfg repair:

1. Prefer making the zero-sized `ReadyProof` marker itself target-neutral (remove `#[cfg(unix)]`) because it contains no Unix API and `ReadyServer` already carries it unconditionally; an equivalent consistently-cfg'd field/initializer repair is acceptable if it does not weaken Linux causal coupling.
2. Preserve H-I4-094: Linux pre-churn `gated_resource_snapshot(...)` must still consume readiness-derived proof **inside the same helper call that invokes `process_resource_snapshot(pid)`**, and must still assert `!churn_started`.
3. Preserve H-I4-092/H-I4-091: successful complete malformed barrier returns `BarrierProof`; Linux post snapshot consumes it; timeout/disconnect/EOF kill+reap and join ownership remain bounded; `malformed_or_unadmitted` remains post-`preauth.release(...)`; non-Linux Unix remains warning-clean.
4. Preserve existing FD/RSS margins, `ATTEMPTS`, runtime Session/Carrier/ACK/crypto/wire semantics, D019, release flags and signing/SBOM/publication policy. Do not invent capacity/security values.
5. No fuzz unless decoder/parser/crypto framing owners change.
6. Run focused `neko-cli` process/integration tests, then on the **final pushed source/test SHA** run:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - verify clean tree
   Persist exact reachable SHA, UTC start/end, exit codes, OS/arch, stable `rustc --version`, clean-tree state.
7. A Windows cross-target compile is useful only if the target/toolchain is already available; do not block on installation and do not claim Windows execution if it did not occur. Static cfg reasoning is sufficient to require this repair.
8. After repair/provenance, immediately perform the I4-PORT-RES causal re-challenge and continue. Do not wait for reviewer cadence.

## Rolling queue — keep continuous after the HIGH

Maintain a real dependency-ordered queue; do not collapse to a one-ticket idle state:

1. **H-I4-095 target-neutral/cfg-safe `ReadyProof` closure + exact-tree provenance.**
2. **I4-PORT-RES exact-current causal re-challenge.** Re-attempt both edges on the repaired tree: `server start/READY -> readiness-derived affirmative pre-churn snapshot -> first malformed send` and `all bounded malformed inputs classified after corresponding cleanup -> BarrierProof -> affirmative post-churn snapshot`. Try whole-block and split/oracle mutations; a scoped no-finding note is valid if both edges fail closed.
3. **Pre-auth malformed/rejection resource-accounting bounded challenge.** Inspect exact-current `admit_carrier`, `charge_input`, invalid-negotiation release, response-admission rejection, cached/pending owners, expiry, queue reservations and process-level counters. Verify rejection cannot become authentication/delivery evidence and diagnostics do not retain unbounded state. Do not choose D019 policy values.
4. **Cross-platform CLI/process-test semantics.** Re-challenge Linux-only `/proc` cfgs, non-Linux Unix warning cleanliness, non-Unix name-resolution/source shape, process/signal assumptions, reader-thread lifecycle, child kill/reap/join ownership, timeout/EOF/disconnect behavior and test portability. Do not invent Windows/macOS/BSD execution.
5. **CLI diagnostic/machine-output boundary.** Challenge `malformed_or_unadmitted` scope, JSON/human-output collision, diagnostic parsing stability, exit-code coupling and evidence-only versus protocol semantics.
6. **Algorithmic/resource boundedness reconciliation.** Challenge channel capacity/lifetime, timeout/EOF/disconnect cleanup, bounded line accumulation, queue/counter growth and test-local resource bounds. No capacity-pressure benchmark and no new policy numbers.
7. **Package/build + reproducibility spot re-challenge.** Reuse prior no-finding only for unchanged owners; verify recent CLI test/helper edits did not stale package/provenance claims. Do not invent signing/SBOM/key-custody policy.
8. **Item-4 + release-packet factual reconciliation.** Reconcile H-I4-090..095 closure/current bounded reviews with `docs/release-security-review-packet.md`, `docs/status.md`, `IMPLEMENTATION_PLAN.md`, provenance notes and review notes. Do not promote local resource testing into WAN/performance/security approval.
9. **Repository-wide 13-surface refill.** Re-apply: reliable UDP recovery; CarrierState; CarrierManager/health/migration-back; FairScheduler/flow accounting; carrier adapters; SessionRuntime; observability; package/operator; dependency/build; cross-platform process tests; CLI output contract; algorithmic boundedness; release-packet factual consistency. Reuse prior reviews only for genuinely unchanged owners. Queue exhaustion is legal only after the broad inventory finds no concrete defect, no uncovered implemented core surface, no READY review-support and no READY live question.
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
