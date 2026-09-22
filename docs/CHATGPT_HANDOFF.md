# ChatGPT reviewer handoff — I4-CLI-PROC-096 remains FRONT READY_LOCAL

## Current repository truth

- Synchronize to current `main` before work. Code/tests/reachable pushed commits outrank this prose, chat memory and stale checkbox state.
- Reviewer re-read the required governance/status/spec/release surfaces and reviewed all developer-owned commits after prior handoff `b5b8780e2d1750b0e6f005643cc115b55892b014`.
- **H-I4-095 remains CLOSED** at `0ed814ce4adbc6de9b803904baa665629dc2bb15`: `ReadyProof` / `BarrierProof` are target-neutral zero-sized markers. Developer-reported exact-tree provenance remains `docs/notes/h-i4-095-provenance-0ed814c-20260922.md`; no Windows/macOS/BSD execution or hosted run is inferred.
- The bounded H-I4-090..095 Linux resource-causality re-challenge remains no-finding at `docs/reviews/independent-i4-port-res-and-cli-readiness-20480d4-20260922.md`.
- `a50ce9d37c24646c5824f3dd738942a3ded16b36` **partially** repairs I4-CLI-PROC-096: `ready_endpoint_rebind_server(...)` now moves blocking stdout reads off-thread and bounds the silent-child path with `recv_timeout(5s)`. Its developer-reported exact-tree local gate is retained in `docs/notes/i4-cli-proc-096-provenance-a50ce9d-20260922.md` (`scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, Linux x86_64, rustc 1.98.0 stable). No hosted run/status was visible for that SHA during reviewer inspection.
- **I4-CLI-PROC-096 is CLOSED** at `b4af007df3b5c602e2735eb7bf349de6b4da7dff` (on top of `4264e05`'s `start_server_for` bounded-wait repair): all three readiness helpers now share `wait_for_ready_marker(child, marker, timeout)` — off-thread `read_line` + `recv_timeout`, and every failure shape (timeout, stdout EOF, read error, channel disconnect) kills+reaps the owned child via `bounded_reap_or_kill` — no unbounded `wait()` on a child not proven exited. Negative regressions cover silent-but-live, silently-exiting binary, and stdout-closing-but-alive children. Exact-tree provenance: `docs/notes/i4-cli-proc-096-provenance-b4af007-20260922.md` — `check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-22T08:00:27Z → 08:06:30Z, Linux x86_64, rustc 1.98.0.
- Candidate A (future/unsent Recovery ACK), Candidate B (mixed datagram drop reasons), prior H-I4/R9 runtime/source findings, Session/Carrier/ACK separation, CarrierState/CarrierManager, FairScheduler/flow accounting, carrier adapters, observability, package/operator, dependency/build and prior CLI machine/human contracts remain closed unless materially changed or falsified by a concrete new counterexample.
- `SessionRuntime.events` retained-history capacity remains a maintainer/security policy gate. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat prior WAN/VPS/HY2/warm-failover evidence merely for freshness.

## FRONT READY_LOCAL — finish I4-CLI-PROC-096 bounded process readiness

This is an item-4 process-test boundedness/reliability defect, **not HIGH/BLOCKER** and not a runtime transport defect. Current semantics determine the repair; do not wait for maintainer input.

### Remaining defect A — `start_server_for(...)`

Exact-current `crates/neko-cli/tests/probe.rs` still computes a two-second deadline, checks it before each loop iteration, then calls blocking `stdout.read_line(...)`. A live child that emits no newline/EOF can block forever after the clock check. The original I4-CLI-PROC-096 review explicitly named this helper; `a50ce9d` did not change it.

### Remaining defect B — endpoint-rebind EOF/read-error path

`ready_endpoint_rebind_server(...)` now bounds the silent/no-output timeout path, but its reader thread reports stdout EOF/read error through `Ok(Err(startup_log))`, and the main thread then calls only `child.wait()` before panic. A child may close stdout while remaining alive; `wait()` can then block indefinitely. The persisted provenance sentence claiming timeout **or EOF** kills/reaps the child is stronger than the exact source. The existing negative regression covers timeout only, not early stdout-close/read-error with a live child.

### Closure contract

1. Apply the smallest test-helper repair; do **not** build a general process framework.
2. Make `start_server_for(...)` genuinely bounded for a live-but-silent child, using the existing off-thread-reader + bounded receiver pattern or an equivalent interruptible mechanism.
3. Keep `ready_endpoint_rebind_server(...)` success semantics/readiness string/startup-log accumulation, but make **all failure exits** bounded: timeout, EOF, read error and channel disconnect must deterministically terminate/reap the owned child before panic/return. Do not call an unbounded `wait()` on a child that has not been proven exited.
4. Ensure reader ownership converges after termination; no indefinitely blocked detached reader is acceptable.
5. Add focused negative coverage for the uncovered shapes: a silent/no-ready path for `start_server_for` (or its narrow extracted wait primitive), and an endpoint-readiness child that closes stdout or otherwise terminates the reader path while remaining alive, proving bounded failure rather than blocking in `wait()`.
6. Preserve existing positive readiness strings/semantics, `ReadyProof` / `BarrierProof` causal gates, FD/RSS margins, ATTEMPTS, D019 and all security/capacity/release policy values.
7. No fuzz unless decoder/parser/crypto framing owners change.
8. Run focused `neko-cli` process tests, then on the **final pushed source/test SHA** run:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - verify clean tree
   Persist exact reachable SHA, UTC start/end, exit codes, OS/arch, stable `rustc --version`, clean-tree state.
9. Immediately continue to the next READY_LOCAL lane after commit/push/provenance. Do not wait for reviewer cadence.

## Rolling queue — keep continuous after I4-CLI-PROC-096

Maintain a real dependency-ordered queue; do not collapse to one ticket or claim queue exhaustion after a narrow sweep:

1. **I4-CLI-PROC-096 remaining bounded-wait repair + exact-tree provenance.**
2. **Pre-auth malformed/rejection resource-accounting bounded challenge.** Inspect exact-current `crates/neko-cli/src/preauth.rs`, `ProcessPreauthAdmission` owners and current failover-server call sites: `admit_carrier`, `charge_input`, staged TCP reservations, invalid-negotiation release, response permits, cached/pending owners, expiry, queue reservations and process counters. Challenge that rejection cannot become authentication/delivery evidence and diagnostics do not retain unbounded state. Do not choose D019 policy values.
3. **Cross-platform CLI/process-test semantics.** Re-challenge Linux-only `/proc` cfgs, non-Linux Unix warning cleanliness, non-Unix name resolution, signal assumptions, reader-thread lifecycle, child kill/reap/join ownership, timeout/EOF/disconnect behavior, target-neutral helpers and external command assumptions. Do not invent Windows/macOS/BSD execution.
4. **CLI diagnostic/machine-output boundary.** Challenge `malformed_or_unadmitted` scope, JSON/human-output collision, diagnostic parsing stability, exit-code coupling, and evidence-only versus protocol semantics.
5. **Algorithmic/resource boundedness reconciliation.** Challenge channel capacity/lifetime, timeout/EOF/disconnect cleanup, bounded line accumulation, queue/counter growth and test-local resource bounds. No capacity-pressure benchmark and no new policy numbers.
6. **Package/build + reproducibility spot re-challenge.** Reuse prior no-finding only for unchanged owners; verify recent CLI test/helper edits do not stale package/provenance claims. Do not invent signing/SBOM/key-custody policy.
7. **Item-4 + release-packet factual reconciliation.** Reconcile H-I4-090..095 closure, current I4-CLI-PROC-096 repair/reviews, `docs/release-security-review-packet.md`, `docs/status.md`, `IMPLEMENTATION_PLAN.md`, provenance and review notes. Local process/resource tests must not be promoted into WAN/performance/security approval.
8. **Reliable UDP / Carrier / scheduler / SessionRuntime targeted spot re-challenge.** Re-open only owners materially changed since their independent bounded reviews; otherwise record exact-current reuse boundaries rather than re-running equivalent sweeps.
9. **Repository-wide 13-surface refill.** Re-apply reliable UDP recovery; CarrierState; CarrierManager/health/migration-back; FairScheduler/flow accounting; carrier adapters; SessionRuntime; observability; package/operator; dependency/build; cross-platform process tests; CLI output contract; algorithmic boundedness; release-packet factual consistency. Queue exhaustion is legal only after the broad inventory finds no concrete defect, no uncovered implemented core surface, no READY review-support and no READY live question.
10. **Conditional live.** Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved real-network question within standing authorization. Otherwise keep `READY_LIVE: none`.

If several coherent slices complete in 10–30 minutes with stable quality, deepen the queue rather than waiting. Every 3–4 coherent slices or important repair cluster, perform one factual release/item-4 reconciliation instead of rewriting large docs after every tiny commit.

## VPS / live boundary

Current classification remains `READY_LIVE: none`.

- IPv6 remains environment-blocked unless a real owned IPv6 endpoint/path appears.
- HY2 and repeated warm-failover current lines remain frozen against same-class retry without a materially new hypothesis.
- Periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track are not repeated for freshness when their bounded question is already answered or their current line is frozen/blocked.
- Standing authorization permits bounded self-owned client↔VPS ordinary TCP/UDP work only when a real new question becomes READY. It does not authorize third-party targets, production route/firewall/DNS/proxy/tunnel/qdisc changes, privileged/exotic-carrier work reserved by the authorization file, or pressure/adversarial-load conditions requiring maintainer choice.

## Review / repair and evidence contract retained

For every bounded lane: read exact-current owners/tests plus applicable spec/ADR/status claim; state the invariant; try to falsify it with source reasoning and focused deterministic tests. If a concrete defect exists and committed semantics determine the answer, apply the smallest repair, add positive/negative regression where meaningful, commit/push, run exact-tree gates, persist provenance and continue.

No-finding slices must name inspected owners, commands/tests actually run, exclusions and exact reachable anchor. Do not modify code merely to manufacture activity. Hosted CI is additional cross-evidence, never a waiting condition. Developer-local, reviewer-local, hosted, live WAN and performance evidence remain distinct classes.

## Stop / escalation conditions

Normal progress does not require administrator notification. Escalate only for an automatically undecidable BLOCKER/HIGH, core Session/Carrier/ACK/crypto/wire architecture decision, D019 or another real policy/value choice, destructive/canonical migration, action beyond standing authorization, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
