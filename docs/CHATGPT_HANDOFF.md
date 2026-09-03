# Nekomusume ChatGPT Handoff

Checked at: 2026-09-03 13:00 Asia/Shanghai
Repository HEAD: `aabdf84585b59326f085faf8beda5cedd1d03ba9`
Previous reviewed implementation HEAD: `6641594795e9f570563bd138b14485d3fa95a845`
Previous reviewer handoff commit: `a0b99b055e7c9c02babe64d04bdcb3908e02345c`

## What changed

One coding-agent commit is visible after the previous reviewer handoff:

- `aabdf84` — benchmark preflight/evidence/test/documentation repair only. It makes the expected remote user explicit, records preflight-blocked results earlier, removes the unsupported assumption that the benchmark must use root, and corrects the retry wording so unchanged retries remain prohibited while one substantively changed retry is allowed.

Exact `aabdf84` GitHub Actions run `33716217267` is green for both `stable checks` and `nightly decode fuzz smoke`.

The direction is accepted, but review finds that the intended preflight regression gate is not actually closed:

1. `scripts/bench/compare-hy2-owned-lab-test.sh` advertises `wrong-user`, `config`, `auth`, and `timeout` cases, but those branches currently do not execute the adapter or assert a result. The mock command also does not implement the corresponding injected failure behavior. Green CI therefore does not prove those contracts.
2. The first remote-read failure path currently distinguishes timeout from all other nonzero exits, but collapses every non-timeout failure into authentication failure. A successfully authenticated remote command that exits nonzero must be a separate evidence class.
3. A preflight failure happens after a local temporary run directory has been created, but the early blocked path exits before the normal cleanup trap is installed. The local temporary runtime path can therefore be left behind.
4. `docs/status.md` contains a small duplicated-word typo in the same repaired paragraph; fix it in the same bounded reconciliation.

## Review verdict

**needs repair before another paid HY2 attempt**

This is a benchmark evidence/control-plane blocker, not a Nekomusume transport/runtime blocker. Do not change wire, Session, Noise, Carrier Manager, D064 semantics, or accepted VPS evidence to solve it.

Once the executable preflight matrix, failure classification, and early local cleanup are repaired and the exact repair HEAD has green stable + nightly CI, the coding agent may proceed directly to one changed-hypothesis HY2/Nekomusume owned-lab attempt under the existing standing authorization without waiting for another reviewer turn.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research implementation baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- Exact `aabdf84` CI is genuinely green, but it only proves the tests that actually execute.
- Exact `25e0daa` controlled application-level UDP reply-cessation warm fallback remains accepted bounded evidence; it is not natural Internet degradation/PTO-blackhole proof.
- Exact `25e0daa` approximately five-minute periodic direct-path sample remains accepted bounded evidence; it is one bounded sample, not a reliability rate or production soak.
- Do not rerun those accepted rows merely to consume VPS time.
- No complete HY2/Nekomusume paired sample set, comparative median/P95, or superiority evidence exists.
- Historical failed HY2 attempts remain negative/procedural evidence only; unsupported detailed root-cause attribution must not be promoted beyond tracked structured evidence.
- Standing authorization permits a new attempt only after a substantive changed configuration/instrumentation/hypothesis; this repair qualifies once complete.
- IPv6 remains environment-blocked unless a genuinely owned end-to-end IPv6 path becomes available.
- The bounded release-evidence matrix remains open.

## Work Package — Real Preflight Regression Coverage -> Green Gate -> One Changed HY2 Attempt -> Matrix Reconciliation

### Primary A — Make the preflight evidence contract executable

**Goal**

Turn every advertised preflight failure class into a deterministic no-VPS regression that really executes the harness and validates the retained blocked artifact.

**Required behavior**

- Replace the nominal/no-op failure loop with real local test invocations.
- Cover at minimum: matching configured identity, mismatched configured identity, configuration inspection failure, remote authentication/transport failure, timeout, and remote-command nonzero.
- Assert the exact retained failure stage, zero samples, and pre-payload provenance for every blocked case.
- Keep remote-command nonzero distinct from authentication failure.
- Ensure each failure artifact passes the repository validator.
- No test may contact a real VPS or require a real credential.

### Follow-up B — Close early-preflight cleanup and documentation drift

**Dependency:** A complete.

- Ensure an early blocked preflight removes the local experiment runtime temp directory it created.
- Add a deterministic regression proving no new test-owned temp runtime path remains after each blocked preflight case.
- Keep retained Git-visible evidence artifacts intact; remove only disposable runtime state.
- Preserve unknown cleanup observations when they truly cannot be verified; do not weaken the validator to manufacture `verified`.
- Fix the duplicated wording in `docs/status.md` and keep the historical HY2 attribution bounded to structured evidence.

### Follow-up C — Full local gate and exact repair-head CI

**Dependency:** A/B complete.

Run the benchmark/control-plane/validator tests and the full repository gate, then push the coherent repair. Require the exact repair HEAD GitHub Actions to be green for both:

- `stable checks`;
- `nightly decode fuzz smoke`.

Do not spend another VPS comparison window before the exact repair HEAD is green.

### Follow-up D — One changed-hypothesis HY2/Nekomusume owned-lab attempt

**Dependency:** C exact repair HEAD CI fully green.

Run one bounded harness invocation using the already reviewed fair comparison profile and existing standing authorization. Do not alter production network state or weaken authentication/integrity.

If preflight is blocked, retain exactly one structured artifact for the observed stage and stop that path for this batch; do not repeat the same configuration.

If admission succeeds, continue through the complete paired contract. A comparative summary exists only if every required pair succeeds under the fair lifecycle/resource contract. Any failed pair retains the valid prefix plus typed failure and produces no comparative summary.

Always verify experiment-owned process/listener/temp-path cleanup.

### Follow-up E — Reconcile the actual result and use remaining VPS time efficiently

**Dependency:** D complete or honestly blocked.

Update `docs/status.md`, `IMPLEMENTATION_PLAN.md`, and `ROADMAP.md` from the retained artifact only. Preserve exact D064/periodic boundaries and historical negative HY2 evidence.

If HY2 becomes blocked on a condition that genuinely requires a new credential/environment, do not idle and do not retry unchanged. Audit the remaining owned-VPS backlog and choose one dependency-ready high-value row or the smallest local seam that directly unlocks one, prioritizing genuine endpoint/path change, real-session migration-back, real-session key update, or live-path PMTUD evidence. Fixture-only capability is not WAN evidence.

## Fallback

If the current blocked-result schema cannot represent the corrected preflight failure classes truthfully, make only the smallest validator/schema extension needed, add mutation coverage, rerun A-C, and keep paid execution stopped until the exact repair HEAD is green.

## Completion gates

- every named preflight failure class is actually executed in deterministic no-VPS tests;
- each blocked case produces a valid structured artifact with truthful stage and pre-payload evidence;
- remote-command nonzero is not mislabeled as authentication failure;
- early preflight leaves no disposable local runtime temp path behind;
- targeted tests and full repository gate pass;
- exact repair HEAD stable + nightly CI are green;
- at most one changed-hypothesis paid HY2 attempt is made after the green gate;
- the actual outcome is reconciled to tracked evidence;
- no accepted D064/periodic claim is inflated or needlessly rerun;
- release/production/global-freeze flags remain unchanged.

## Do not expand into

- changing Nekomusume protocol/runtime semantics for benchmark convenience;
- weakening authentication or integrity;
- production network changes;
- third-party targets or scanning;
- another unchanged HY2 retry;
- publishing superiority claims from incomplete or blocked evidence;
- speculative experimental-track work without an observed-problem gate.

## Questions requiring maintainer decision

none at this review point.

A maintainer decision is required only if a repaired, structured changed-hypothesis attempt demonstrates that a genuinely new credential/server/environment is required and cannot be satisfied by the already authorized owned endpoint configuration.
