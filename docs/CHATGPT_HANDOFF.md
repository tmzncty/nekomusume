# ChatGPT reviewer handoff — H-I4-097 bounded process-cleanup HIGH FRONT

## Current repository truth

- Synchronize to current `main` before work. Code/tests/reachable pushed commits outrank this prose, chat memory and stale checkbox state.
- Reviewer independently re-read the current required governance/spec set and developer-owned work through exact `34be59192d6233acd4ee6f1c5387b3dbad8d8abd`, then added H-I4-097 at `35ba42e237ec92358de4083f5ebc44b722cddb2f`.
- The prior `34be591` `queue exhausted` classification is **superseded by H-I4-097**. Do not idle on that stale conclusion.
- **H-I4-090..095 remain closed** on their bounded malformed-resource causality/cfg proof surfaces unless a new concrete counterexample is found. H-I4-095 remains closed at `0ed814ce4adbc6de9b803904baa665629dc2bb15`.
- **I4-CLI-PROC-096 implementation shape remains useful but its broad bounded-cleanup conclusion is no longer accepted.** `b4af007df3b5c602e2735eb7bf349de6b4da7dff` moved blocking readiness reads off-thread and added timeout/EOF negatives, but exact-current `bounded_reap_or_kill` still performs an unconditional `wait()` after `try_wait` error or ignored `kill()` failure. The current independent no-finding at `49925ac` is therefore stale on this one invariant.
- **I4-BND at `5b7b22b` is also reopened only for the process-cleanup failure-path bound.** Its `sync_channel(64) -> sync_channel(1)` repair remains accepted. The no-finding claim that all timeout/EOF/disconnect cleanup is bounded is superseded by H-I4-097.
- Pre-auth malformed/rejection accounting, CLI diagnostic/machine-output, package/build reproducibility and unchanged core-surface reuse findings remain closed unless materially changed or falsified by a current-tree counterexample.
- Candidate A (future/unsent Recovery ACK), Candidate B (mixed datagram drop reasons), prior runtime/source findings, Session/Carrier/ACK separation, CarrierState/CarrierManager, FairScheduler/flow accounting, carrier adapters and observability remain closed unless their owners change or a concrete counterexample appears.
- `SessionRuntime.events` retained-history capacity and D019 source-retention/no-reset remain maintainer/security policy gates. Do not invent TTL/LRU/history-size/capacity/security values.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely for freshness.

## FRONT HIGH — H-I4-097 bounded cleanup failure paths

Authoritative reviewer finding: `docs/reviews/reviewer-h-i4-097-bounded-reap-kill-error-20260922.md` at `35ba42e`.

### Concrete defect

Exact-current `crates/neko-cli/tests/probe.rs::bounded_reap_or_kill` is documented as bounded, but currently has this control shape:

```rust
match child.try_wait() {
    Ok(Some(_)) | Err(_) => {}
    Ok(None) => { let _ = child.kill(); }
}
let _ = child.wait();
```

Two source-level failure paths can block past the test deadline:

1. `try_wait()` returns `Err(_)` and there is no proof the child exited, yet blocking `wait()` still follows.
2. `try_wait()` returns `Ok(None)`, `kill()` returns `Err(_)` while the child may remain live, the error is ignored, and blocking `wait()` still follows.

`malformed_classification_barrier` duplicates the same `kill(); wait(); join()` shape on timeout/EOF/channel error. If termination fails, both `wait()` and the reader-thread `join()` may cease to be bounded. Existing silent/live regressions exercise successful termination and do not cover these cleanup-error results.

This is **item-4/release-evidence process-oracle correctness**, not proof of a production/runtime transport leak and not a Session/Carrier/ACK/crypto/wire architecture finding.

### Closure contract

1. Apply the smallest test/helper repair; do not redesign process infrastructure.
2. No blocking `wait()` may occur without exit proof, or after termination unless reaping is itself bounded by an explicit local deadline/poll.
3. `try_wait` and `kill` errors must be explicit fail-closed cleanup errors; they must not fall through to an unbounded wait.
4. All `malformed_classification_barrier` failure exits must obey the same bounded cleanup rule. Do not `join()` a reader that may still be blocked on stdout of a child whose termination failed.
5. Add the narrowest deterministic regression(s) practical. Do not build a generic mocking/process framework solely to manufacture an OS failure.
6. Preserve H-I4-090..095 causal proof ordering, `ReadyProof`/`BarrierProof`, Linux `/proc` measurement scope, FD/RSS margins, malformed `ATTEMPTS`, post-`preauth.release` diagnostic ordering and the one-item handoff. No new policy/security/capacity value.
7. Final pushed source/test SHA: run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, verify clean tree, and persist exact-tree developer-local provenance with SHA, UTC start/end, exit codes, OS/arch and stable Rust. No fuzz unless decoder/parser/crypto-framing owners change.
8. Commit/push and immediately continue; do not wait for reviewer cadence.

## Rolling queue — keep continuous after H-I4-097

Do not collapse this to one ticket. Dependency-ready order:

1. **H-I4-097 repair + focused regression + exact-tree provenance** — current FRONT HIGH.
2. **Process-cleanup causal re-challenge.** Re-read every owner of `bounded_reap_or_kill`, direct kill/wait cleanup, reader ownership and deadline handling; confirm no unproven-exit blocking wait remains.
3. **Cross-platform CLI/process factual reconciliation.** Reconcile `49925ac`, I4-CLI-PROC-096 and any revised helper semantics. Keep Windows/macOS/BSD execution claims separate from Linux-local evidence.
4. **Algorithmic/resource boundedness reconciliation.** Reconcile `d814457` / `5b7b22b`, channel bounds, stdout accumulation, cleanup deadlines and reader lifetime. No capacity-pressure benchmark and no new policy numbers.
5. **Release packet / item-4 factual reconciliation.** Remove or qualify stale “all process failure paths bounded / queue exhausted” conclusions; retain valid H-I4-090..095 and one-item-channel evidence. Local process tests must not be promoted to WAN/performance/security approval.
6. **Pre-auth malformed/rejection accounting exact-current reuse challenge.** Reuse `cfce41e` only if relevant owners remain unchanged; otherwise re-challenge changed ownership narrowly.
7. **CLI diagnostic/machine-output boundary exact-current reuse challenge.** Reuse `4b8e70a` only across unchanged owners; diagnostics remain evidence-only and never authentication/Delivery/Path/ACK evidence.
8. **Package/build/reproducibility spot re-challenge.** Verify the process-test repair did not stale manifests/scripts/provenance assumptions. Do not invent signing/SBOM/key-custody policy.
9. **Reliable UDP / CarrierState / CarrierManager / scheduler / SessionRuntime targeted spot re-challenge.** Re-open only materially changed owners; otherwise record exact-current reuse boundaries rather than rerunning equivalent sweeps.
10. **Observability + carrier adapters + dependency/build exact-current reuse challenge.** Same owner-diff rule; no checker/docs filler.
11. **Repository-wide 13-surface refill.** Re-apply all 13 required surfaces after the HIGH and its dependent reconciliation are closed. Queue exhaustion is legal only if the broad inventory finds no concrete defect, no uncovered implemented core surface, no READY review-support and no READY live question.
12. **Conditional live.** Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved real-network question within standing authorization. Otherwise keep `READY_LIVE: none`.

If several coherent slices complete in 10–30 minutes with stable quality, deepen the queue and continue implementation/review -> tests -> commit -> push -> next slice without waiting. Every 3–4 coherent slices or important repair cluster, do one factual item-4/release reconciliation rather than rewriting large docs after every small commit.

## VPS / live boundary

Current classification remains `READY_LIVE: none`.

- IPv6 remains environment-blocked unless a real owned IPv6 endpoint/path appears.
- HY2 and repeated warm-failover current lines remain frozen against same-class retry without a materially new hypothesis.
- Periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD and Experimental Track are not repeated for freshness when their bounded question is already answered or current line is frozen/blocked.
- Standing authorization permits bounded self-owned client<->VPS ordinary TCP/UDP work only when a real new question becomes READY. It does not authorize third-party targets, production route/firewall/DNS/proxy/tunnel/qdisc changes, privileged/exotic-carrier work reserved by the authorization file, or pressure/adversarial-load conditions requiring maintainer choice.

## Evidence discipline retained

Developer-reported local CI, persisted local provenance, reviewer-local execution, GitHub-hosted CI, live WAN evidence and performance conclusions are distinct evidence classes. All shared exact-tree provenance anchors must be reachable pushed commits. No unpublished/local-only SHA becomes repository evidence. No secret, protected identity, private topology or unnecessary absolute path belongs in provenance.

## Stop / escalation conditions

Normal progress does not require administrator notification. Escalate only for an automatically undecidable BLOCKER/HIGH, core Session/Carrier/ACK/crypto/wire architecture decision, D019 or another genuine policy/value choice, destructive/canonical migration, action outside standing authorization, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
