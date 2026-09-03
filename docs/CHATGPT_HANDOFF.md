# Nekomusume ChatGPT Handoff

Checked at: 2026-09-03 14:58 Asia/Shanghai
Repository HEAD reviewed: `308e46e5efeb988a224538f9899221c3fa6c06d3`
Previous reviewed implementation HEAD: `bc38d06bfc0c19d0ec26fe2d0f5484202565ec18`
Previous reviewer handoff commit: `3054d4f0147be932abcc4502db12e04d5750cecd`

## What changed

One coding-agent commit is visible after the previous reviewer handoff:

- `308e46e` — **benchmark-harness repair + retained negative VPS evidence + status reconciliation; no Nekomusume wire/Session/Noise/failover semantic change and no comparative performance result.** The exactly-one paid invocation at exact coding baseline `bc38d06` passed the explicit SSH preflight and prepared the deterministic 1,200-byte payload, but failed during setup before the first sample because `run_client` expanded `impl` in the same `local` statement before assignment under `set -u`. The retained result is validator-valid `BLOCKED_HARNESS`, contains zero samples and no paired statistics, and is stored under the unique path `artifacts/hy2-owned-lab/bc38d06-blocked-harness/`. Its result SHA-256 is `596ad4b73058143db1918613dd970e44e8e6bf3a1b89602ac0012f911b6d2653`.

`308e46e` fixes the initializer by splitting declaration from assignments and adds an executable no-VPS regression that invokes the production `run_client` body under `set -u`. It also updates `IMPLEMENTATION_PLAN.md`, `ROADMAP.md` and `docs/status.md` without converting the failed attempt into comparison evidence.

Exact `308e46e` GitHub Actions run `33722724900` is independently green:

- `stable checks` — `bash scripts/check.sh` succeeded;
- `nightly decode fuzz smoke` — pinned cargo-fuzz decode build and 30-second / 8,192-byte smoke succeeded.

The prior paid attempt nevertheless exposed a second, independent evidence-integrity issue: the structured artifact records cleanup failure (`remote_listeners_remaining=1`, `remote_process_groups_reaped=false`, remote temp-path removal unknown). Separate manual post-run cleanup later verified no experiment ports, processes or temp paths remained, but that later observation correctly did not rewrite the original artifact. The current harness cleanup path itself therefore remains insufficiently proven for another paid attempt.

A previously noted preservation hazard also remains present at current HEAD: `scripts/bench/compare-hy2-owned-lab-test.sh` unconditionally removes `artifacts/hy2-owned-lab/result.json` and its sample companion in EXIT traps. There is no retained real artifact at that default path now, but a future real default-path result could be deleted by a routine local/CI gate.

## Review verdict

**continue with required evidence-integrity repair — accept the `set -u` fix and retained negative attempt; close automatic cleanup/preservation before one materially changed paid retry**

The repository is not globally blocked. The paid HY2 comparison path is temporarily gated only by benchmark evidence integrity. This is a release-evidence correctness gate, not a protocol-runtime blocker.

Do not rerun the exact `bc38d06` attempt unchanged. A future retry is allowed after the cleanup/preservation repair because the code, instrumentation and hypothesis will have materially changed. Standing authorization already covers that bounded self-owned retry.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research implementation baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- Exact `308e46e` has green stable and nightly fuzz CI. This is repository CI evidence, not an independent security audit or release approval.
- The paid experiment happened at exact `bc38d06`; the later `308e46e` code fix must not be retroactively attributed to the experiment binary.
- The `bc38d06` artifact proves only: explicit SSH preflight succeeded, payload preparation succeeded, setup then failed before any sample due to a harness shell-initialization defect. It proves no HY2/Nekomusume comparative latency, throughput, CPU/RSS/FD, reliability or superiority.
- Artifact-recorded cleanup failure is historical evidence and must remain unchanged. Manual cleanup is a separate later observation, not a repair of that artifact.
- Exact `25e0daa` controlled application-level UDP reply-cessation warm fallback and approximately five-minute periodic Session evidence remain accepted with their existing narrow boundaries. Do not rerun them merely to consume VPS time.
- IPv6 remains environment-blocked unless a genuinely owned end-to-end IPv6 path becomes available.
- The bounded release-evidence matrix remains open.
- `scripts/bench/compare-hy2-owned-lab-test.sh` still has a real evidence-preservation defect: routine test cleanup can remove a future real default-path result.

## Work Package — Cleanup/Preservation Closure -> One Changed Retry -> Matrix Reconciliation -> Next VPS Opportunity

### Primary A — Make owned-lab cleanup and result preservation fail-closed and executable

**Goal**

Before another paid VPS comparison, prove that normal success/failure paths cannot silently delete retained evidence and that experiment-owned process groups/listeners/temp paths converge to a verified clean state within the existing cleanup reserve.

This is benchmark/evidence infrastructure only. Do not change Nekomusume wire, Session, Noise, carrier or failover semantics.

#### A1. Remove the default-result deletion hazard

Current `scripts/bench/compare-hy2-owned-lab-test.sh` EXIT traps unconditionally remove:

```text
artifacts/hy2-owned-lab/result.json
artifacts/hy2-owned-lab/result.json.samples.jsonl
```

Repair this so a pre-existing real artifact at the default path can never be destroyed or modified by a routine no-VPS test / `scripts/check.sh` run.

Preferred properties:

- validation/test mode uses disposable output, or explicitly preserves/restores any pre-existing default artifact byte-for-byte;
- add an executable sentinel regression that pre-creates both default files with known bytes, runs the relevant test/validation path, and proves both files remain byte-identical;
- no test may rely on deleting real evidence as cleanup.

#### A2. Harden process-group and listener cleanup convergence

The failed paid attempt left the structured cleanup result red even though later manual cleanup found no residue. Treat this as an unresolved harness cleanup/evidence defect until the automatic path is proven.

Required behavior:

1. Local and remote cleanup must reason about the **owned process group / descendants**, not only whether the originally stored leader PID still exists.
2. After TERM/KILL, use a small bounded settle/poll loop within the existing cleanup reserve for:
   - owned process-group emptiness;
   - owned listener disappearance on all experiment ports;
   - remote temp-path removal after process/listener cleanup is verified.
3. Do not mark `*_process*_reaped=true` merely because a parent/leader disappeared while descendants may remain.
4. Keep cleanup bounded; do not turn cleanup into an unbounded wait or increase the standing ten-minute experiment ceiling.
5. If automatic cleanup cannot prove completion, retain `cleanup_status=failed`/unknown fields and the diagnostic state. Never fabricate success because a later manual cleanup happened.

Add an executable regression using disposable process groups/listeners that includes at least one descendant or short shutdown-delay case where a one-shot immediate observation would be racy. The regression must prove the production cleanup logic waits for bounded convergence and leaves no owned listener/process residue.

If practical without weakening secret boundaries, preserve a compact cleanup diagnostic (for example command RC / convergence reason / listener count) so a future failed cleanup can be distinguished from a transport failure without discarding stderr blindly. Do not commit endpoint addresses, credentials or private topology.

#### A3. Exercise both timed implementation branches under `set -u`

The new regression currently executes the production `run_client` initializer only for the Nekomusume branch. Extend the no-VPS production-body regression so both implementation identities (`nekomusume` and `hy2`) reach their branch-specific timed/sample construction without an initializer/unbound-variable crash. Fake transport execution is acceptable; the objective is shell/control-flow correctness, not a fake benchmark PASS.

Keep existing regressions green: typed first-failure retention, blocked-prefix validation, preflight matrix, TLS pin contract, fresh-client lifecycle, budget/deadline checks, signal/timeout behavior, descendant cleanup and result validation.

### Follow-up B — Full local gate and exact-head independent CI before paid reuse

**Dependency:** A complete.

Run at minimum:

- targeted owned-lab benchmark tests and validator mutation/regression coverage;
- `cargo fmt --all -- --check`;
- `cargo check --workspace --locked`;
- `cargo test --workspace --all-targets --locked --no-fail-fast`;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- `bash scripts/check.sh`;
- `git diff --check`.

Push the coherent cleanup/preservation repair. Wait for the **exact coding HEAD** GitHub Actions `stable checks` and nightly decode fuzz smoke to be green before another paid comparison invocation. Do not treat a reviewer-only handoff commit as invalidating a green coding HEAD.

### Follow-up C — Execute exactly one materially changed HY2/Nekomusume owned-lab retry

**Dependency:** A/B complete and exact repair HEAD CI green.

**Why a retry is now permitted**

The previous attempt failed on a concrete harness defect before samples. A/B materially changes code/instrumentation and specifically repairs the failed cleanup/evidence path, satisfying the standing-authorization rule against unchanged retries.

**Profile**

- self-owned client + owned VPS only;
- exact repaired coding HEAD in an isolated/detached worktree;
- record Nekomusume binary SHA-256 and pinned HY2 v2.9.3 SHA-256;
- explicit verified SSH endpoint/user and distinct assigned remote bind address vs reachable connect authority;
- disposable certificate/key, exact `pinSHA256`, fresh password auth;
- 5 paired runs, 1,200-byte deterministic payload, concurrency 1;
- fresh unprivileged ports;
- exactly one harness invocation in this batch;
- unique non-default repository-relative result path;
- complete invocation including cleanup remains below the standing ten-minute limit;
- no production firewall/route/qdisc/DNS/proxy/tunnel/service change.

Use bounded capture/observation around the temporary HY2 UDP port when practical. If the run fails after setup, retain enough typed evidence to distinguish client emission, VPS arrival, server response, return path, TLS/auth/config failure and network/path failure without widening scope.

**Outcome rules**

- A pre-sample failure remains `BLOCKED_HARNESS`/typed diagnostic evidence, never comparison evidence.
- A failed/incomplete pair preserves its valid prefix but contributes nothing to comparative median/P95.
- Only a complete required paired set may produce comparative summary statistics.
- Automatic cleanup must be validator/evidence-clean. If cleanup fails again, preserve that result and do not issue another unchanged paid retry in the same batch.
- One successful five-pair set is one self-owned route/time-window observation, not a superiority or production claim.

### Follow-up D — Reconcile release evidence from C without erasing history

**Dependency:** C complete or honestly blocked with retained evidence.

Update `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md`, and the compact experiment note/result index as needed.

- Preserve exact `f1cb9af` and `bc38d06` failed attempts as historical evidence.
- Record C as positive only if every required pair is valid and cleanup is verified.
- If C is blocked/partial, state exact stage and valid prefix; no comparative summary.
- Do not alter the accepted `25e0daa` D064/periodic boundaries.
- Keep natural UDP degradation/PTO-blackhole, IPv6, NAT/endpoint-change and other unproven matrix rows open.
- Governance flags remain unchanged.

### Follow-up E — Spend the next VPS opportunity on one genuinely READY missing row, or its smallest direct unlock seam

**Dependency:** D complete.

Audit executable runtime surfaces rather than capability labels. Prefer, in order:

1. genuine NAT/source-endpoint change on owned endpoints if current runtime can create/observe it without production route/firewall changes;
2. real-session migration-back if a truthful live CLI/runtime path exists;
3. real-session key update only if no longer fixture-only;
4. live-path PMTUD only if real packetization/probe instrumentation exists;
5. otherwise the smallest local implementation/instrumentation seam that directly unlocks one of those rows.

Do not spend rental time on another generic TCP/UDP baseline or repeat already-accepted D064/periodic rows. If a candidate remains fixture/state-model-only, record `BLOCKED_IMPLEMENTATION` and build only the minimal release-evidence seam rather than fabricating a VPS result.

## Fallback

If A reveals a deeper cleanup/control-plane bug, keep the paid path paused and close that exact evidence-integrity defect first. The project as a whole is not blocked: continue any dependency-safe local release-evidence instrumentation work that directly unlocks a VPS row.

If C demonstrates a genuinely new credential/server/environment requirement not satisfiable by the configured owned endpoint, retain the exact blocker, complete D and the highest-value local unlock work, and only notify the maintainer if the requirement truly needs new credentials/server/authorization rather than another code/instrumentation repair.

## Completion gates

- current `308e46e` negative paid artifact remains immutable and validator-valid;
- default-path real evidence cannot be deleted/modified by routine no-VPS tests;
- cleanup proves owned process-group/descendant and listener convergence, not merely leader-PID disappearance;
- delayed/descendant cleanup regression proves the race is closed;
- both Nekomusume and HY2 `run_client` branches execute under `set -u` in no-VPS control-flow regression;
- exact repair coding HEAD passes full local gate and independent GitHub stable + nightly fuzz jobs before C;
- at most one materially changed paid HY2 invocation occurs in C;
- C uses unique evidence path and preserves positive or negative result exactly;
- no failed/incomplete pair contributes to comparative statistics;
- no superiority/public/production claim is made from one batch;
- matrix/status reconciliation preserves historical negatives and narrow positive boundaries;
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.

## Do not expand into

- changing Nekomusume wire/Session/Noise/failover semantics for benchmark convenience;
- weakening authentication/integrity or HY2 certificate pinning;
- production network changes, third-party targets or scanning;
- repeated unchanged paid attempts;
- publishing superiority claims from one bounded five-pair batch;
- treating fixture-only key-update/PLPMTUD/manager behavior as live WAN evidence;
- speculative FEC/0-RTT/striping/exotic-carrier work without an observed-problem gate.

## Questions requiring maintainer decision

none.

Standing authorization already covers A/B local repairs and the bounded self-owned C retry. A maintainer decision is required only if later evidence demonstrates a genuinely new credential/server/environment requirement, an action outside standing authorization, or a major architecture choice that cannot be resolved from repository facts.
