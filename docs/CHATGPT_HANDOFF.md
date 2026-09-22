# ChatGPT reviewer handoff — cross-platform CLI/process review FRONT READY_LOCAL

## Current repository truth

- Synchronize to current `main` before work. Code/tests/reachable pushed commits outrank this prose, chat memory and stale checkbox state.
- Reviewer inspected all developer-owned commits through `d96aabeb066e6aaaaa279d2ce153d4ff554040ea`, then added the bounded pre-auth no-finding review at `cfce41edda3ad2d431ae5999c1c80f7eee2d8859`.
- **H-I4-090..095 remain closed/no-finding on their bounded resource-causality re-challenge.** H-I4-095 is closed at `0ed814ce4adbc6de9b803904baa665629dc2bb15`; `ReadyProof` / `BarrierProof` are target-neutral markers. Do not reopen these without a concrete current-tree counterexample.
- **I4-CLI-PROC-096 is CLOSED** at `b4af007df3b5c602e2735eb7bf349de6b4da7dff`: `start_server_for`, `ready_failover_server` and `ready_endpoint_rebind_server` share `wait_for_ready_marker`, which performs blocking stdout reads off-thread and uses bounded `recv_timeout`; timeout/EOF/read-error/channel-disconnect paths pass owned children through `bounded_reap_or_kill`. Focused negatives cover silent-live, silently-exiting and stdout-closing-but-alive child shapes. Developer-reported exact-tree provenance is `docs/notes/i4-cli-proc-096-provenance-b4af007-20260922.md` (`scripts/check.sh` 0, `git diff --check` 0, clean tree, 2026-09-22T08:00:27Z -> 08:06:30Z, Linux x86_64, rustc 1.98.0 stable). Reviewer observed no commit status or PR-triggered hosted workflow run for that SHA; absence of hosted evidence is not a failure.
- **Pre-auth malformed/rejection resource-accounting bounded challenge is NO-FINDING** at `docs/reviews/independent-preauth-rejection-accounting-d96aabe-20260922.md` (review commit `cfce41e`). Exact-current `ListenerAdmission`/`ProcessPreauthAdmission`, staged TCP ownership, input/response rollback/settlement, queue expiry/dequeue, pending failover UDP ownership and H-I4-091 post-release diagnostic ordering were re-read. No concrete engineering-control defect was found in that bounded scope. This does not close RSEC-001 promotion suitability or D019 source-retention policy.
- Candidate A (future/unsent Recovery ACK), Candidate B (mixed datagram drop reasons), prior H-I4/R9 runtime/source findings, Session/Carrier/ACK separation, CarrierState/CarrierManager, FairScheduler/flow accounting, carrier adapters, observability, package/operator and dependency/build findings remain closed unless materially changed or falsified by a concrete new counterexample.
- `SessionRuntime.events` retained-history capacity and D019 source-retention/no-reset remain maintainer/security policy gates. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely for freshness.

## FRONT READY_LOCAL — cross-platform CLI/process-test semantics

This is an item-4 independent bounded challenge. It is not permission to broaden the first-RC target beyond x86_64 Linux and is not a request to invent portability policy. No known HIGH/BLOCKER is open at handoff time; if the bounded review is no-finding, persist a precise no-finding note and immediately continue.

### Owners to inspect

Read exact-current:

- `crates/neko-cli/tests/probe.rs`, especially `ReadyServer`, `ReadyProof` / `BarrierProof`, `wait_for_ready_marker`, `bounded_reap_or_kill`, `malformed_classification_barrier`, process launch/finish helpers and target-gated tests;
- the CLI/runtime process-control and signal/readiness call sites in `crates/neko-cli/src/main.rs` that those tests exercise;
- `crates/neko-cli/Cargo.toml`, workspace target/dependency/build surface where relevant;
- current portability/release claims in `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `docs/release-security-review-packet.md`, and prior independent CLI portability notes.

### Invariants to challenge

1. Linux-only `/proc` measurements must be fully `cfg(target_os = "linux")` and must fail affirmative on Linux when required, while non-Linux Unix builds do not inherit unused/name-resolution failures.
2. Unix-specific commands, signals, shell syntax, permissions or process assumptions used only by tests must be target-gated; target-neutral helpers/types must remain name-resolvable on non-Unix targets.
3. Child ownership must converge on success and every failure shape: timeout, stdout EOF, read error and channel disconnect. No path may perform an unbounded wait on a child not proven exited or leave an indefinitely blocked reader that owns the only pipe endpoint.
4. Shared readiness helpers must preserve each command's exact readiness marker/startup-log semantics; refactoring must not turn a partial/incorrect marker into readiness.
5. Negative process tests must themselves be bounded and must not depend on accidental platform behavior while claiming a generic contract.
6. Do not infer Windows/macOS/BSD execution from Linux-local compilation/tests. First-RC target remains x86_64 Linux unless maintainers decide otherwise.

### Review -> repair contract

- First try to falsify the current helpers and call sites by source reasoning plus the narrow existing deterministic process regressions.
- Do **not** reopen I4-CLI-PROC-096 or H-I4-095 merely because their code was touched; require a concrete counterexample.
- If current committed semantics determine a real defect, apply the smallest repair plus a positive/negative regression, commit/push, then on the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, verify a clean tree and persist exact-tree provenance (SHA, UTC start/end, exits, OS/arch, stable Rust). No fuzz unless decoder/parser/crypto-framing owners change.
- If no defect is found, write one scope-precise independent bounded no-finding note naming owners inspected, commands/tests actually run or reused, exclusions and exact reachable anchor. Do not edit implementation merely to manufacture activity.
- Immediately continue to the next READY_LOCAL lane after commit/push/no-finding; do not wait for reviewer cadence.

## Rolling queue — keep continuous

Maintain the dependency-ordered queue; do not collapse it to one ticket or claim repository-wide exhaustion after a narrow sweep:

1. **Cross-platform CLI/process-test semantics** — current FRONT.
2. **CLI diagnostic/machine-output boundary.** Challenge `malformed_or_unadmitted` scope, JSON/human-output collision, diagnostic parsing stability, exit-code coupling, and evidence-only versus protocol semantics; diagnostics must not become authentication/Delivery/Path/ACK evidence.
3. **Algorithmic/resource boundedness reconciliation.** Re-challenge channel capacity/lifetime, timeout/EOF/disconnect cleanup, bounded line/log accumulation, queue/counter growth and test-local resource bounds. No capacity-pressure benchmark and no new policy numbers.
4. **Package/build + reproducibility spot re-challenge.** Reuse prior no-finding only for unchanged owners; verify recent CLI test/helper changes do not stale package/provenance claims. Do not invent signing/SBOM/key-custody policy.
5. **Item-4 + release-packet factual reconciliation.** Reconcile H-I4-090..095, I4-CLI-PROC-096, the current pre-auth no-finding, subsequent bounded reviews, `docs/release-security-review-packet.md`, `docs/status.md`, `IMPLEMENTATION_PLAN.md`, provenance and review notes. Local process/resource tests must not be promoted to WAN/performance/security approval.
6. **Reliable UDP / Carrier / scheduler / SessionRuntime targeted spot re-challenge.** Re-open owners materially changed since their dedicated independent bounded reviews; otherwise record exact-current reuse boundaries instead of rerunning equivalent sweeps.
7. **Observability + carrier adapters + dependency/build exact-current reuse challenge.** Verify recent changes have not crossed their prior review ownership boundary; if unchanged, record bounded reuse rather than checker/docs filler.
8. **Repository-wide 13-surface refill.** Re-apply reliable UDP recovery; CarrierState; CarrierManager/health/migration-back; FairScheduler/flow accounting; carrier adapters; SessionRuntime; observability; package/operator; dependency/build; cross-platform process tests; CLI output contract; algorithmic boundedness; release-packet factual consistency. Queue exhaustion is legal only after this broad inventory finds no concrete defect, no uncovered implemented core surface, no READY review-support and no READY live question.
9. **Conditional live.** Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved real-network question inside standing authorization. Otherwise keep `READY_LIVE: none`.

If several coherent slices complete in 10–30 minutes with stable quality, deepen the queue rather than waiting. Every 3–4 coherent slices or important repair cluster, perform one factual item-4/release reconciliation instead of rewriting large documents after every small commit.

## VPS / live boundary

Current classification remains `READY_LIVE: none`.

- IPv6 remains environment-blocked unless a real owned IPv6 endpoint/path appears.
- HY2 and repeated warm-failover current lines remain frozen against same-class retry without a materially new hypothesis.
- Periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD and Experimental Track are not repeated for freshness when their bounded question is already answered or current line is frozen/blocked.
- Standing authorization permits bounded self-owned client<->VPS ordinary TCP/UDP work only when a real new question becomes READY. It does not authorize third-party targets, production route/firewall/DNS/proxy/tunnel/qdisc changes, privileged/exotic-carrier work reserved by the authorization file, or pressure/adversarial-load conditions requiring maintainer choice.

## Evidence discipline retained

Developer-local, persisted local provenance, reviewer-local execution, hosted CI, live WAN and performance evidence are distinct evidence classes. All shared exact-tree provenance anchors must be reachable pushed commits. No unpublished/local-only SHA becomes repository evidence. No secret, protected identity, private topology or unnecessary absolute path belongs in provenance.

## Stop / escalation conditions

Normal progress does not require administrator notification. Escalate only for an automatically undecidable BLOCKER/HIGH, core Session/Carrier/ACK/crypto/wire architecture decision, D019 or another genuine policy/value choice, destructive/canonical migration, action outside standing authorization, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
