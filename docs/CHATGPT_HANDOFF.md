# Nekomusume ChatGPT Handoff

Checked at: 2026-09-03 12:00 Asia/Shanghai
Repository HEAD: `6641594795e9f570563bd138b14485d3fa95a845`
Previous reviewed implementation HEAD: `026586859214fa712a8259bd374ed0d9631645be`
Previous reviewer handoff commit: `23c422f02611165d1d1497368a43f349a11f19d3`

## What changed

Two coding-agent commits are visible after the previous reviewer handoff:

- `f1cb9af` — **benchmark/test admission repair; no Nekomusume wire/Session/Noise/failover semantic change.** The generic `compare-hy2.sh` path now validates adapter stdout as exactly one typed JSON object before extracting fields, and its regression matrix covers empty/malformed/contaminated/multiple/scalar/array/missing/wrong-typed output. The probe/failover integration tests also replace several fixed sleeps with bounded readiness observation and serialize fixed-port tests through one lock, reducing race-driven CI ambiguity.
- `6641594` — **documentation/status reconciliation only.** It correctly records the already accepted exact-`25e0daa` controlled warm-fallback and approximately five-minute periodic rows without promoting them to natural degradation or general long-lived reliability. It also records a later HY2 attempt as preflight-blocked and keeps all release-governance flags false.

The exact current HEAD is independently green in GitHub Actions run `33713069068`:

- `stable checks`: **PASS** (`bash scripts/check.sh`);
- `nightly decode fuzz smoke`: **PASS** (30-second decode fuzz smoke).

The previous R-HY2-18 generic JSON gate is therefore closed at the current tree.

However, review finds a new **evidence/provenance defect around the paid HY2 preflight attempt**. The `f1cb9af..6641594` tracked delta contains code/tests and status prose, but no tracked blocked-result/evidence artifact for the claimed paid attempt. `docs/status.md` and `IMPLEMENTATION_PLAN.md` currently attribute the SSH preflight failure to an alias wrapper retaining user `tmzn` instead of `root`, yet the repository's own `docs/bench/hy2-vps-setup-20260830.md` records successful VPS inspection as user `tmzn` with non-interactive sudo only where needed. The current owned-lab benchmark itself uses unprivileged high ports, `/tmp`, read-only `ip`/`ss`, and does not establish that root is required for benchmark admission.

Therefore the GitHub evidence supports only the narrower statement: **a paid HY2 attempt was reported blocked during SSH preflight, but the exact failed invocation, effective SSH identity, failure output/classification, and cleanup/result artifact are not currently auditable from tracked repository evidence.** The asserted `tmzn`-versus-`root` root cause must not be treated as established until the harness records it deterministically.

A second wording drift must also be repaired: standing authorization forbids **unchanged** repeat failures, not all future retries forever. The two same-configuration failed invocations are negative evidence and must not be repeated unchanged. A new attempt is allowed only after a substantive configuration/instrumentation/hypothesis change, with one bounded invocation and retained evidence.

## Review verdict

**needs evidence-contract repair — exact HEAD CI is green and the benchmark JSON/control-plane code is acceptable, but the paid-run SSH preflight currently fails outside the repository's structured evidence boundary; repair provenance and effective SSH identity before one changed-hypothesis retry**

This is not a transport/runtime correctness regression. Do not change Nekomusume wire, Session, Noise, Carrier Manager or failover semantics to solve it.

The HY2 paired comparison remains the highest-value VPS opportunity after the evidence/preflight repair. Do not insert unrelated local polish between a green exact repair HEAD and the changed-hypothesis paid run.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research implementation baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- Current exact-head stable CI and nightly decode fuzz smoke are independently green; this is repository gate evidence, not a security audit or release approval.
- Exact `25e0daa` controlled application-level UDP reply-cessation warm fallback remains accepted bounded evidence: 3/3 records, 48 application bytes, two uncertain/replayed records, duplicate 0, lost 0, approximately 434 ms failure-decision-to-first-resumed-data. It is not natural Internet degradation/PTO-blackhole proof.
- Exact `25e0daa` approximately five-minute periodic direct-path sample remains accepted bounded evidence: 60 × 32-byte records, 60/60 confirmed, no missing/duplicate/conflict. It is one bounded sample, not a reliability rate or production soak.
- Do not rerun those accepted rows merely to consume VPS time.
- No complete HY2/Nekomusume paired sample set, comparative median/P95 or superiority evidence exists.
- The reported exact-`f1cb9af` paid attempt has no tracked blocked-result/evidence artifact in the reviewed delta. Treat the detailed causal claim (`tmzn` instead of `root`) as **unverified attribution**, not repository evidence.
- `docs/bench/hy2-vps-setup-20260830.md` records prior successful inspection as unprivileged user `tmzn`; root is not a benchmark requirement established by current repository facts.
- Standing authorization permits a changed-hypothesis retry after substantive configuration/instrumentation change. It does not permit another unchanged retry.
- IPv6 remains environment-blocked unless a genuinely owned end-to-end IPv6 path becomes available.
- The bounded release-evidence matrix remains open.

## Work Package — Structured SSH Preflight Evidence -> Green Gate -> One Changed HY2 Attempt -> Matrix Reconciliation

### Primary A — Bring SSH preflight and effective identity inside the structured evidence contract

**Goal**

Make every remote paid-run attempt auditable from GitHub-visible evidence, including failures that occur before payload preparation or remote process launch. Eliminate implicit/ambiguous SSH user selection without requiring root or new credentials.

**Likely files**

- `scripts/bench/compare-hy2-owned-lab.sh`;
- `scripts/bench/owned-lab-control-plane.sh`;
- `scripts/bench/compare-hy2-owned-lab-test.sh` and/or `owned-lab-control-plane-test.sh`;
- `scripts/bench/validate-hy2-owned-lab.py` and its mutation tests if the result schema needs a preflight-failure contract;
- `docs/bench/hy2-comparison-workload.md` / `hy2-vps-setup-20260830.md` only for contract clarification;
- `docs/status.md` / `IMPLEMENTATION_PLAN.md` / `ROADMAP.md` only to correct the current unsupported root-cause/retry wording.

**Required behavior**

1. Split preflight into two classes:
   - pure local syntax/binary/safety validation before an experiment attempt begins;
   - remote/SSH-dependent admission once an experiment attempt has begun.
2. Before the first SSH-dependent operation that can fail due endpoint/authentication state, initialize an experiment/result context sufficient to retain a typed blocked artifact. A failed SSH authentication/read-only preflight must not disappear as an unstructured shell exit.
3. Make effective SSH identity explicit and deterministic. Add a bounded contract such as `LAB_SSH_EXPECTED_USER` (name may differ) and resolve both effective `hostname` and `user` through the **same configured SSH binary/path** used by the run (`ssh -G` or an equivalent deterministic config inspection). Fail closed before remote experiment work if the effective user does not match the declared expected user.
4. Do **not** assume or require `root`. The existing repository setup successfully used `tmzn`; the current benchmark uses high ports and `/tmp`. If a later diagnostic such as capture requires sudo/capability, keep that as a separate bounded diagnostic requirement rather than silently changing the benchmark transport identity to root.
5. A remote preflight authentication failure must retain at minimum:
   - exact git/binary identity;
   - experiment id;
   - stage such as `ssh-preflight`;
   - bounded return/timeout classification;
   - payload-prepared false if applicable;
   - no client/server samples when none ran;
   - cleanup fields truthfully zero/verified/unknown according to what was actually created;
   - no secret, key, private config or unnecessary address material.
6. Preserve stderr only as a redacted/hashed diagnostic or a bounded non-secret classification if needed; do not commit credential text, private keys or raw SSH config.
7. Correct the current status prose:
   - keep the two failed exact-configuration invocations as historical negative/procedural evidence;
   - replace the asserted `tmzn`-instead-of-`root` cause with the narrow fact actually supported by retained evidence unless the new deterministic contract proves the user mismatch;
   - state that **unchanged retry is prohibited**, while one changed-configuration/hypothesis retry remains allowed under standing authorization.
8. Do not erase or rewrite the accepted D064/periodic evidence.

**Regression matrix**

Add deterministic no-VPS tests proving at minimum:

- effective SSH hostname + expected user match is accepted;
- hostname match + wrong effective user fails before remote experiment work;
- `ssh -G`/config-inspection failure is typed/fail-closed;
- SSH authentication failure during the first remote read produces one valid blocked artifact with zero samples/payload as appropriate;
- timeout remains distinct from authentication/command nonzero failure;
- no remote process/temp path means cleanup fields are not falsely claimed from operations that never occurred;
- the blocked artifact validates under the repository result validator;
- no test requires real credentials or contacts a real VPS.

### Follow-up B — Full local gate and exact-repair-HEAD CI

**Dependency:** A complete.

Run the relevant benchmark/control-plane tests first, then the complete repository gate:

1. `bash scripts/bench/compare-hy2-test.sh`;
2. `python3 scripts/bench/parse-listener-test.py`;
3. `bash scripts/bench/owned-lab-control-plane-test.sh`;
4. `bash scripts/bench/compare-hy2-owned-lab-test.sh`;
5. `python3 scripts/bench/validate-hy2-owned-lab-test.py`;
6. process-resource sampler / exact-payload regressions;
7. `bash scripts/check.sh`;
8. `git diff --check`.

Push the coherent repair. Require the **exact repair HEAD** to have both:

- `stable checks`: green;
- `nightly decode fuzz smoke`: green.

If the repair changes only shell/evidence parsing, do not manufacture additional fuzz claims beyond the repository CI gate.

The coding agent is pre-authorized to proceed directly from a fully green exact repair HEAD to Follow-up C without waiting for another reviewer handoff.

### Follow-up C — One changed-hypothesis paid HY2/Nekomusume attempt

**Dependency:** B exact repair HEAD CI fully green.

This is a new allowed attempt because the experiment contract has substantively changed: effective SSH identity is explicit and preflight failures are now structured evidence. It is **not** an unchanged retry of the two historical failures.

Run **one harness invocation only** for this batch.

Use the established bounded profile:

```text
self-owned client <-> owned VPS only
pinned HY2 v2.9.3 exact SHA-256
5 paired runs if admission succeeds
1200-byte deterministic payload per sample
concurrency 1
fresh unprivileged ports
fresh transport client/session per timed sample
separate bind/connect authority
pinned disposable experiment certificate
whole lab including cleanup reserve < 10 minutes
```

Before invocation, verify the explicit expected SSH user matches the effective SSH configuration. Do not change production firewall/route/qdisc/DNS/proxy/tunnel/service state.

If SSH/preflight fails:

- retain exactly one structured blocked artifact from this invocation;
- stop this HY2 path for the batch;
- do not invoke the harness a second time with the same configuration;
- classify whether the remaining blocker is configuration, existing-credential/environment, or harness implementation.

If admission succeeds, continue through the normal fair paired contract. If all required pairs succeed, retain raw samples and only then compute the bounded comparison summary. If any required pair fails, retain the valid prefix + typed failure and produce no comparative summary.

Always verify experiment-owned process/listener/temp-path cleanup.

### Follow-up D — Reconcile evidence/status from the actual C outcome

**Dependency:** C complete or honestly blocked.

Update `docs/status.md`, `IMPLEMENTATION_PLAN.md`, and `ROADMAP.md` so they describe only tracked evidence:

- preserve exact `25e0daa` D064 and periodic boundaries;
- record the exact C artifact/result and commit identity;
- if C is blocked, link the tracked blocked artifact and state the actual observed stage/classification without speculative root cause;
- if C succeeds, record paired results only at their exact route/time/security/lifecycle scope, with no superiority claim;
- preserve prior failed attempts as historical negative evidence;
- keep release matrix item 3 unchecked while declared missing rows remain;
- keep all release/production/freeze flags unchanged.

### Follow-up E — If HY2 is environment/credential-blocked, immediately pivot to one other rental-window unlock slice

**Dependency:** C blocked for a reason that cannot be changed without new credentials/environment, or D complete after a positive C.

Do not sit idle and do not repeat HY2 unchanged. Audit the remaining VPS backlog against current runtime surfaces and select at most one high-value row/unlock:

1. genuine owned source-endpoint/path change;
2. real-session migration-back;
3. real-session synchronized key update;
4. live-path PMTUD observation.

If a live CLI/runtime seam already exists and standing authorization covers it, run one bounded row with structured evidence and cleanup. If the capability exists only as fixture/local state, record `BLOCKED_IMPLEMENTATION` and spend the remaining slice on the **smallest runtime/instrumentation seam that directly unlocks that one VPS row**, with deterministic tests. Do not treat fixture-only capability as WAN evidence.

If the only missing requirement is genuinely a new SSH credential/server/environment, that is a maintainer decision; record it precisely rather than weakening authentication or substituting root.

## Completion gates

This package closes when:

- SSH-dependent preflight failures produce tracked/validatable structured evidence rather than disappearing before result creation;
- effective SSH user selection is explicit and tested without assuming root;
- the unsupported `tmzn`-versus-`root` attribution is removed or replaced by deterministic proof;
- unchanged retry prohibition is distinguished from changed-hypothesis permission;
- all local benchmark/control-plane tests and full repository gate pass;
- exact repair HEAD stable + nightly CI are green;
- exactly one changed-hypothesis HY2 invocation is made after the green gate, or no invocation occurs because a pre-invocation deterministic contract fails locally;
- the resulting HY2 outcome is reconciled to tracked evidence;
- no D064/periodic claim is inflated;
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.

## Do not expand into

- changing Nekomusume wire/Session/Noise/failover semantics for benchmark convenience;
- requiring root merely to make SSH work;
- committing credentials, SSH private config, private keys or unnecessary topology;
- disabling TLS/authentication/integrity for HY2 or Nekomusume;
- production service/firewall/route/qdisc/DNS/proxy/tunnel changes;
- third-party targets or scanning;
- an unchanged third/fourth retry of the same SSH failure;
- publishing superiority claims from an incomplete/blocked/one-off comparison;
- speculative FEC/0-RTT/striping/exotic-carrier work without an observed-problem gate.

## Questions requiring maintainer decision

none at this review point.

A maintainer decision is required only if the changed-hypothesis structured preflight demonstrates that a **new credential/server/environment** is genuinely required and cannot be satisfied by the already authorized owned endpoint configuration.
