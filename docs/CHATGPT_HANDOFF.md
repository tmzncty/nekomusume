# Nekomusume ChatGPT Handoff

Checked at: 2026-09-03 14:00 Asia/Shanghai
Repository HEAD reviewed: `bc38d06bfc0c19d0ec26fe2d0f5484202565ec18`
Previous reviewed implementation HEAD: `aabdf84585b59326f085faf8beda5cedd1d03ba9`
Previous reviewer handoff commit: `87d4fec2f523563080b802406bb97b25d56dfef4`

## What changed

One coding-agent commit is visible after the previous reviewer handoff:

- `bc38d06` — **benchmark preflight regression/evidence-cleanup repair only; no Nekomusume wire/Session/Noise/failover semantic change and no new VPS experiment.** The no-VPS owned-lab test now actually executes the advertised preflight matrix instead of leaving nominal branches as no-ops. It covers a matching configured SSH identity that progresses to the next bind-address gate, mismatched configured identity, SSH configuration inspection failure, SSH/auth-control-plane failure, timeout, and a successfully reached remote command returning nonzero. The adapter now retains `ssh-command` separately from RC124 timeout and RC255 SSH failure, classifies an assigned-bind-address failure as a typed blocked stage, and installs an early EXIT/INT/TERM cleanup path that removes the disposable local runtime directory. The same commit also fixes the duplicated wording in `docs/status.md`.

Exact `bc38d06` GitHub Actions run `33720478587` is green. Both required jobs completed successfully:

- `stable checks` — `bash scripts/check.sh` green;
- `nightly decode fuzz smoke` — pinned `cargo-fuzz` decode build plus 30-second/8192-byte smoke green.

The previous handoff's required preflight regression/cleanup gate is therefore closed. The changed-hypothesis paid HY2/Nekomusume attempt is now READY under standing authorization.

One non-blocking evidence-hygiene finding remains: `scripts/bench/compare-hy2-owned-lab-test.sh` currently names the repository default `artifacts/hy2-owned-lab/result.json` in its EXIT cleanup because successful `--validate` uses that fixed default. There is no tracked artifact at that path now, so this did not corrupt current repository evidence, but a future real result left at that default path could be deleted by a later local gate. The paid attempt below must therefore use a unique non-default result path, and the test should be made preservation-safe before later routine gates are allowed to touch a retained default-path artifact.

## Review verdict

**pass preflight gate — execute exactly one changed-hypothesis HY2/Nekomusume owned-lab attempt now; preserve result at a unique path**

Do not spend another reviewer cycle polishing the preflight harness before using the rental window. `bc38d06` closes the blocker that prevented the changed retry, and its exact coding HEAD has independent green stable + nightly CI.

This reviewer-only handoff commit does **not** invalidate the green gate on `bc38d06`. For the paid run, prefer an exact-`bc38d06` detached/worktree build and record its binary SHA-256. Do not wait for or manufacture a new CI run merely because this coordination file advanced `main` after the reviewed coding HEAD.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research implementation baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- `bc38d06` is test/harness/evidence-control work only. It adds no WAN behavior and no performance result.
- Exact `bc38d06` CI is green for both repository stable checks and nightly decode fuzz smoke.
- Exact `25e0daa` controlled application-level UDP reply-cessation warm fallback remains accepted bounded evidence: 3/3 records, 48 application bytes, two uncertain/replayed, duplicate 0, lost 0, approximately 434 ms failure-decision-to-first-resumed-data. It is not natural Internet degradation/PTO-blackhole proof.
- Exact `25e0daa` approximately five-minute periodic direct-path sample remains accepted bounded evidence: 60 x 32-byte records, 60/60 confirmed, no missing/duplicate/conflict. It is one bounded sample, not a production soak or reliability rate.
- Do not rerun those accepted rows just to consume VPS time.
- No complete HY2/Nekomusume paired sample set, comparative median/P95, or superiority evidence exists yet.
- Historical failed HY2 attempts remain negative/procedural evidence. The last retained preflight failure does not prove a root-only requirement; the configured SSH identity must be whatever the owned endpoint actually resolves and the harness now checks it explicitly.
- Standing authorization already permits this bounded self-owned HY2 comparison and bounded port-scoped capture. No new per-run WAN permission is needed.
- IPv6 remains environment-blocked unless a genuinely owned end-to-end IPv6 path becomes available.
- The bounded release-evidence matrix remains open.

## Work Package — One Paid HY2 Pair -> Preserve/Validate Evidence -> Reconcile Matrix -> Next VPS Opportunity

### Primary A — Execute exactly one changed-hypothesis fair HY2/Nekomusume owned-lab invocation

**Goal**

Use the now-green repaired harness to obtain either the first complete semantically fair paired sample set or one high-information typed blocker. Do not perform an unchanged second invocation in this batch.

**Exact coding baseline**

- Prefer exact `bc38d06bfc0c19d0ec26fe2d0f5484202565ec18` in a detached/isolated worktree.
- Record exact Nekomusume binary SHA-256 and the already pinned HY2 v2.9.3 SHA-256.
- The reviewer handoff commit may exist on `main`; it is coordination-only and need not trigger a new code gate before execution.

**Required profile**

- self-owned client + owned VPS only;
- existing verified SSH endpoint and explicit expected SSH user from the owned configuration;
- dedicated assigned remote bind address; client connect address remains separately verified;
- HY2 v2.9.3 pinned artifact already recorded by the repository;
- fresh disposable experiment certificate/key plus exact `pinSHA256` and fresh password authentication;
- 5 paired runs;
- 1,200-byte deterministic payload per sample;
- concurrency 1;
- fresh unprivileged experiment ports in the repository's allowed range;
- one harness invocation only;
- complete invocation, including cleanup reserve, remains below the standing ten-minute single-run limit;
- no production firewall/route/qdisc/DNS/proxy/tunnel/service change.

**Evidence-path rule**

Do **not** use the repository default `artifacts/hy2-owned-lab/result.json` for this paid result. Use a unique repository-relative experiment path, for example:

```text
artifacts/hy2-owned-lab/<experiment-id>/result.json
```

with its sample/evidence companions. Preserve the exact path in the research note/commit. This avoids the current no-VPS test cleanup hazard until Follow-up C repairs it.

**Bounded diagnostics**

Wrap the single invocation with a bounded capture/observation scoped only to the temporary HY2 UDP port when practical under the existing standing authorization. Retain only the minimum needed metadata/hash/packet-direction counts; raw pcap need not be committed if it exposes unnecessary address detail.

If the run fails after preflight, distinguish at least:

- client packets emitted toward the reachable connect authority;
- arrival at the VPS temporary UDP port when observable;
- server response emission when observable;
- response return to client when observable;
- HY2 TLS/auth/config vs network/path failure using retained logs and structured stages.

Do not alter provider firewall/routing or production network state to force success.

**Outcome rules**

- If preflight blocks, retain exactly one typed `BLOCKED_HARNESS` artifact, validate it, verify cleanup, and stop the HY2 execution path for this batch.
- If admission succeeds but any paired sample fails, retain the valid sample prefix plus typed failure. No comparative summary is legal.
- Only if every required pair succeeds may the validator produce the complete comparison summary.
- Even a complete 5-pair result is one self-owned route/time-window observation, not a superiority or production claim.

### Follow-up B — Validate and freeze this attempt's evidence before any routine gate can touch it

**Dependency:** A complete, whether positive or blocked.

1. Validate the result with `scripts/bench/validate-hy2-owned-lab.py`.
2. Verify exact payload bytes/hash, Nekomusume and HY2 binary identities, run count, per-sample lifecycle/resource evidence, and cleanup fields.
3. Verify no experiment-owned process/listener/temp path remains on client or VPS.
4. Retain a compact research/evidence note with exact experiment id, coding commit, binary hashes, actual parameters, start/end times, result path, capture metadata when used, and claim boundary.
5. Preserve negative evidence exactly; do not rerun merely to seek a PASS.

If the artifact is incomplete or invalid because of a harness/evidence bug discovered during A, do not reinterpret it as performance evidence. Retain it as diagnostic evidence and make the defect the next local repair.

### Follow-up C — Make the no-VPS test preservation-safe

**Dependency:** B evidence safely retained at its unique path. This is a small local evidence-hygiene repair and must not delay A.

Repair `scripts/bench/compare-hy2-owned-lab-test.sh` so routine tests cannot delete or overwrite a pre-existing real result at the harness's default output path.

Acceptable approaches include preserving/restoring a pre-existing default artifact or changing the validation/test plumbing so the successful `--validate` path uses disposable repository-relative test output without touching a real default result. Add an executable regression with a sentinel pre-existing default result/sample file and prove the test leaves them byte-identical afterward.

Do not weaken blocked-artifact validation and do not change transport semantics.

After this small repair, run the targeted benchmark tests and full local gate; push it as a separate coherent commit. CI may run normally, but this repair is not a reason to repeat A.

### Follow-up D — Reconcile the bounded release-evidence matrix from the retained A result

**Dependency:** B complete. C may land before or after the documentation reconciliation if conflict-free.

Update `docs/status.md`, `IMPLEMENTATION_PLAN.md`, and `ROADMAP.md` using only the retained structured A artifact and its exact boundary.

- If A produced a complete valid paired set, record the exact 5-pair batch, raw sample provenance, median/P95/failures, application bytes and symmetric transport-client CPU/RSS/FD evidence. Keep wording bounded to this self-owned endpoint/route/time window and do not claim superiority.
- If A blocked or had any failed pair, record the exact failure stage and valid prefix, and state explicitly that no comparative summary exists.
- Preserve historical HY2 negatives; do not rewrite them as if the new run occurred at their commits.
- Preserve the accepted exact-`25e0daa` D064 and periodic boundaries unchanged.
- Keep natural UDP degradation/PTO-blackhole, IPv6, NAT/endpoint-change and any other genuinely unproven matrix rows open.
- Governance flags remain unchanged.

### Follow-up E — Use remaining VPS opportunity only for a genuinely READY missing row, or build the smallest direct unlock seam

**Dependency:** D complete or A honestly blocked and B retained.

Audit current executable runtime surfaces, not capability labels. Do not treat fixtures as live evidence.

Priority candidates remain:

1. genuine NAT/source-endpoint change on owned endpoints, only if the current live runtime can create/observe it without production route/firewall modification;
2. real-session migration-back, only if a current CLI/runtime path already exposes it truthfully;
3. real-session key update, only if it is no longer fixture-only;
4. live-path PMTUD evidence, only if packetization/probe instrumentation is exposed in the real runtime;
5. otherwise the **smallest local implementation/instrumentation seam that directly unlocks one of the above**, with tests and no architecture expansion.

At the current reviewed status, `key-update` is still advertised as a fixture and PLPMTUD as bounded state rather than live path evidence. Do not schedule a fake VPS row from those labels. If all four remain implementation-blocked, record the precise `BLOCKED_IMPLEMENTATION` reason and select the smallest direct unlock seam rather than burning VPS time on another generic TCP/UDP baseline.

## Fallback

If A reveals that the current harness still cannot preserve a truthful typed result, retain all diagnostics, stop paid execution after that one invocation, and make the exact evidence-integrity defect Primary. Do not retry the same endpoint/configuration until code, instrumentation or hypothesis changes materially.

If A demonstrates a genuinely new credential/server/environment requirement not satisfiable by the already configured owned endpoint, record that exact blocker and continue with Follow-up C/D plus the highest-value dependency-ready local/VPS work. Only then does the maintainer-notification gate apply.

## Completion gates

- exact `bc38d06` preflight gate remains independently green;
- at most one changed-hypothesis paid HY2 invocation is executed in this batch;
- the paid result uses a unique non-default evidence path;
- the result is validator-clean or explicitly retained as invalid/diagnostic with no comparison claim;
- cleanup of experiment-owned process/listener/temp state is checked;
- no failed/incomplete pair contributes to comparative median/P95;
- no superiority/production/public-reachability claim is made from one batch;
- the no-VPS test is made preservation-safe before future routine gates can delete a default-path artifact;
- status/plan/roadmap reflect the exact new outcome without erasing old negatives;
- accepted D064/periodic rows are not needlessly rerun;
- release/production/global-freeze flags remain unchanged.

## Do not expand into

- changing Nekomusume protocol/runtime semantics for benchmark convenience;
- weakening authentication/integrity or removing HY2 certificate pinning;
- production network changes;
- third-party targets or scanning;
- a second unchanged HY2 invocation in the same batch;
- publishing superiority claims from a 5-pair one-window result;
- pretending fixture-only key-update/PLPMTUD/manager behavior is real-session WAN evidence;
- speculative FEC/0-RTT/striping/exotic-carrier work without an observed-problem gate.

## Questions requiring maintainer decision

none at this review point.

A maintainer decision is required only if the single changed-hypothesis run or the subsequent READY-row audit demonstrates a genuinely new credential/server/environment requirement, an action outside standing authorization, or a major architecture choice that cannot be resolved from repository facts.
