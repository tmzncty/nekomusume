# Nekomusume ChatGPT Handoff

Checked at: 2026-09-02 08:59 Asia/Shanghai
Repository HEAD: `e07066b4c3f8e3ad8b33af08f27682173f415a9c`
Previous reviewed implementation HEAD: `62d4d3576e24d4b0f951a615b0d116a74f5d7a73`
Previous reviewer handoff commit: `9df7d251ab0f8f2be84be466955208f9b42edd5e`

## What changed

One meaningful coding-agent commit is visible since the previous reviewer handoff:

- `e07066b` — **benchmark harness / cleanup / test / documentation repair; no production protocol semantic change and no new VPS experiment.** It makes the process-resource sampler respond to SIGTERM/SIGINT by terminating its owned child process group, moves several local/remote benchmark wrappers into explicit process groups with `setsid`, adds signal/listener regressions, and retains typed benchmark failure-row generation.

The current GitHub Actions result materially changes the previous review state: the `CI` workflow for `e07066b` is green in both the stable job and the fuzz job. The stable job completed format/check/workspace tests/Clippy/`scripts/check.sh`/`git diff --check`; the fuzz job also completed successfully. Therefore the previous R-310 “current-head CI failure must be reproduced before VPS work” blocker is closed at this exact HEAD.

The repair is useful but does not close every evidence-truthfulness concern in the HY2 paired harness. In particular, the blocked-result path still synthesizes a SHA-256 for literal `empty` when the intended payload has not been generated, and missing remote-cleanup observations can still be coerced into a numeric listener count instead of represented as unknown/not-observed. The process sampler also reports cleanup fields such as zero owned sockets after exit without an explicit post-exit observation in the generic sampler contract, and its current deterministic grandchild coverage does not yet prove normal-exit descendant cleanup in every same-process-group case.

These remaining HY2-harness issues are important for fair-comparison evidence, but they do **not** block the separately implemented D064 failover/resume path or the periodic authenticated-session path. Because the VPS rental window is time-limited and current-head stable/fuzz CI is green, the release-evidence matrix should resume VPS-first work while the HY2 evidence contract is repaired before any HY2 rerun.

## Review verdict

**continue with required fixes — bounded release-evidence matrix is READY; HY2 comparison branch remains evidence-blocked until its cleanup/provenance contract is truthful**

Do not serialize all work behind HY2 harness polish. Use the current exact HEAD for the two high-value self-owned VPS rows whose hypotheses have changed since their retained negative evidence:

1. D064 warm/cold failover/recovery after the reviewed three-second readiness-sequence repair;
2. five-minute periodic authenticated Session after setup timeout was separated from per-record ACK timeout.

In parallel sequence after those runs, close the HY2 harness truthfulness gaps, get the new repair through the full gate/CI, and only then perform a changed-hypothesis HY2 diagnostic or paired run.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline status.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains a corpus-specific fact only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- N9 corpus freeze and negotiation-path completion are already complete; the first unchecked implementation-plan item is the bounded release-evidence matrix.
- Current-head GitHub Actions stable and fuzz jobs are green. This is independent CI evidence for repository gates, not a security audit or release approval.
- `e07066b` changes benchmark-harness lifecycle/evidence handling and tests; it does not add a new Nekomusume wire, crypto, Session, failover or carrier semantic.
- Current exact-head positive D064 warm VPS evidence is still absent. The retained changed-path run from an older HEAD reached two authenticated admitted readiness responses and failed closed at challenge 3. The D064 runtime contract has since changed to a one-second per-probe / three-second whole-sequence policy, so a current-head run is a changed-hypothesis experiment, not an unchanged retry.
- Current exact-head five-minute periodic VPS evidence is still absent. The retained older attempt authenticated the server but exchanged zero application bytes because setup/handshake timing was coupled to the ACK deadline. Current code now has a distinct setup deadline, so a current-head run is also a changed-hypothesis experiment.
- HY2 paired comparison remains absent. The pinned HY2 v2.9.3 artifact and fair-pair adapter exist, but the prior temporary QUIC/UDP path timed out before the forwarding listener was ready; no paired samples or statistics exist.
- IPv6 remains environment-blocked unless a real owned IPv6 endpoint becomes available. Do not synthesize an IPv6 row.
- Standing self-owned VPS authorization remains active. No new per-run WAN permission is required for the bounded TCP/UDP work below.

## Work Package — VPS-First Release Evidence + HY2 Truthfulness Repair

### Primary A — Current-exact-head D064 warm/cold failover evidence

**Goal**

Use the exact current reviewed implementation to determine whether the accepted D064 readiness/failover contract now survives the self-owned client↔VPS path after the deadline repair.

**Why now**

The previous negative run is preserved and scientifically useful, but its failure hypothesis changed: current D064 code now gives the three authenticated readiness observations a one-second per-probe and three-second whole-sequence budget after negotiation, Noise authentication and resume validation. Current GitHub CI is green, and this row is VPS-only evidence that becomes harder to obtain after the rental window closes.

**Required behavior**

1. Build/use a binary whose identity is tied to the exact implementation HEAD being tested; record git/binary identity.
2. Use only administrator-controlled client/VPS endpoints and standing-authorized temporary high ports.
3. Exercise the existing D064 controlled failure seam. Do not introduce production route/firewall/qdisc changes to manufacture degradation.
4. For the warm path, record the complete authenticated readiness chronology: negotiation complete, Noise authenticated, resume validated, challenge/response 1..3, admission state, warm transition, controlled UDP failure, promotion/fallback and post-promotion application delivery.
5. Prove that warm TCP carries no new application data before atomic promotion.
6. Record Session/failover accounting that the implementation actually exposes: confirmed, uncertain, replayed, duplicate and lost bytes/records; failure and new-active timestamps; generations/epoch/reason where available.
7. If the existing harness supports the cold control without widening scope, run it as a bounded comparison row in the same lab session. Do not invent a cold path just for symmetry.
8. Preserve a negative result exactly if readiness or promotion still fails. A failed current-head run is valid evidence.
9. Verify cleanup: no experiment listener/process/temp runtime remains.

**Completion definition**

A current-head positive or negative D064 artifact is committed with exact parameters, binary identity, client/server evidence, event chronology and verified cleanup. A positive row may support bounded controlled-fault failover evidence only; it is not natural-WAN degradation or production proof.

### Follow-up B — Current-exact-head five-minute periodic authenticated Session

**Dependency:** Primary A completed or failed with a retained, cleanup-verified artifact. A negative A does not block this independent row.

**Goal**

Test the repaired setup/ACK deadline separation on the owned VPS with one bounded five-minute authenticated periodic workload.

**Required behavior**

1. Use the current exact implementation/binary identity and the existing periodic client/server path.
2. Run one approximately five-minute scenario, remaining below the standing ten-minute single-run limit.
3. Record setup timing separately from per-record ACK/confirmation timing.
4. Record expected/sent/confirmed application records and bytes, missing/duplicate records, failures/timeouts, and confirmation-latency samples that are actually observable.
5. Collect process-scoped CPU/RSS/FD/socket evidence using the existing sampler only to the extent its fields are directly observed/truthful; do not upgrade unavailable metrics to zero.
6. Record start/end timestamps and cleanup state.
7. If setup or application delivery fails again, retain the exact negative row and stop unchanged retries until a new hypothesis/instrumentation/code change exists.

**Completion definition**

A current-head positive or negative five-minute periodic artifact exists with non-ambiguous application-delivery accounting and cleanup. A positive result is one bounded self-owned-path sample, not sustained production proof.

### Follow-up C — Close remaining HY2 harness evidence-truthfulness gaps locally

**Dependency:** A/B evidence has been pushed, or one of those paths is independently blocked by a retained technical failure. This repair must complete before another HY2 comparison attempt.

#### C1. Payload provenance must never be synthetic

The blocked-result path currently falls back to SHA-256 of literal `empty` if failure happens before payload creation. Replace this with a truthful representation.

Required contract:

- configured/intended `payload_bytes` may remain part of the experiment contract;
- observed payload artifact/hash must be nullable or explicitly `not_generated` / `not_observed` when creation did not occur;
- a blocked artifact must never present a hash for bytes that were not the experiment payload;
- validator/schema tests must reject contradictory combinations such as nonzero configured payload bytes + synthetic/claimed observed hash when no payload artifact existed.

#### C2. Cleanup unknown must remain unknown

Do not coerce a missing remote cleanup marker into `remote_listeners_remaining=1` or another invented measurement.

Required contract:

- cleanup measurements may be `null` / not-observed when the observation itself failed;
- `cleanup_status=verified` is legal only when every required cleanup observation was explicitly made and showed the expected true/zero state;
- missing/parse-failed cleanup evidence yields an unverified/failed status without inventing listener counts;
- schema/validator and blocked-result tests cover this distinction.

#### C3. Process-group cleanup must be proved, not inferred from direct-child reap

The sampler now sends TERM/KILL to the child process group on timeout/signal, which is a good repair. Harden the generic cleanup contract so `cleanup.complete=true` cannot be produced merely because the direct child was reaped.

At minimum:

- after normal child exit, check whether the sampler-owned process group still contains descendants before declaring cleanup complete; terminate/reap/wait for owned descendants when safely possible;
- after SIGTERM/SIGINT/timeout, verify the group is empty after termination;
- do not hardcode `owned_sockets_after_exit=0` unless a post-exit owned-socket observation proves zero; otherwise make it nullable/not-observed;
- add deterministic harmless tests where the direct child spawns a same-process-group grandchild/listener and then exits normally;
- add signal/timeout variants that assert the descendant PID is gone and listener is absent, not only that the sampler/direct child exited;
- preserve strict scope: only sampler-created process groups/owned test ports, never arbitrary host processes.

#### C4. Failure-path regression matrix

Add/retain deterministic tests for:

- failure before payload generation;
- malformed/missing cleanup marker;
- normal child exit leaving a descendant;
- SIGTERM/SIGINT descendant cleanup;
- timeout descendant cleanup;
- output/runtime deletion only after owned processes are verified gone;
- blocked artifacts remain machine-valid but cannot contain comparative summary/statistics.

Run the complete local repository gate. Push the repair and require the next GitHub Actions stable/fuzz run to be green before using the repaired harness on VPS.

### Follow-up D — Changed-hypothesis HY2 owned-lab diagnostic / paired run

**Dependency:** C complete and current repair CI green.

**Goal**

Use the time-limited VPS to learn why the prior temporary HY2 QUIC/UDP path never reached forwarding readiness, then obtain paired samples only if both sides satisfy the same application contract.

**Required behavior**

1. Reuse the pinned HY2 v2.9.3 artifact/hash and the existing dedicated non-wildcard `LAB_REMOTE_BIND_ADDRESS` contract.
2. This must be a changed-hypothesis run: add/use instrumentation or capture targeted at the prior HY2 server/client/QUIC readiness timeout. Do not blindly repeat the old attempt.
3. Keep Nekomusume and HY2 on the same owned client/VPS, comparable route/time window, MTU, security class, payload, run count and load.
4. Preserve typed failure rows if either implementation fails. Partial runs are diagnostic evidence only.
5. Produce median/P95/failure comparison only if the complete required paired sample set succeeds for both implementations.
6. CPU/RSS/FD/application-byte fields must be observed under the same contract; `wire_bytes` remains null unless bounded capture metadata makes it trustworthy.
7. Verify cleanup of all temporary Nekomusume/HY2 processes/listeners/config/cert/temp paths.
8. No superiority claim from a single successful batch.

### Follow-up E — Use remaining VPS session time for the next genuinely missing row

**Dependency:** A/B complete and D either complete or honestly environment-blocked; do not rerun failed unchanged experiments.

Choose the highest-value dependency-satisfied row still absent from the release matrix, not a speculative feature. Prefer in this order:

1. a distinct bounded real-socket failover/recovery lifecycle sample if A was a single transition and the existing harness already supports repetition without changing semantics;
2. a real-session resource/leak observation with repeated open/exchange/close if current evidence does not already answer it;
3. real-session key update or PMTUD observation only if there is already a truthful executable path/instrumentation at current HEAD;
4. package/readiness/cleanup revalidation only if meaningful release-relevant code changed since N5.

Do not repeat already-sufficient UDP/TCP baseline rows merely to keep the VPS busy.

### Fallback

If A or B fails on the changed current-head hypothesis:

- retain the negative evidence and verified cleanup;
- do not mechanically retry;
- make the exact failure stage/event boundary the next local diagnostic slice;
- continue independent READY VPS rows that do not depend on that failure;
- never convert the failure into a generic `need WAN authorization` blocker.

If HY2 still cannot establish the temporary QUIC/UDP path after C and a changed-instrumentation D run, preserve the diagnostic artifact and classify it as an environment/path/implementation-evidence blocker. Continue other Nekomusume release-matrix evidence rather than looping on HY2.

## Completion gates

This package is complete when the applicable items below are true:

- current GitHub Actions stable/fuzz success for `e07066b` is preserved as the starting gate;
- a current-head D064 warm/cold result, positive or negative, exists with exact chronology and cleanup;
- a current-head five-minute periodic result, positive or negative, exists with application accounting and cleanup;
- HY2 blocked-result payload provenance no longer invents a hash for a payload that was not generated;
- unavailable cleanup observations remain unknown rather than fabricated numeric values;
- process-resource cleanup cannot claim complete solely from direct-child reap and is regression-tested with descendants;
- the HY2 repair passes the full local gate and its GitHub stable/fuzz CI before another VPS comparison run;
- any HY2 diagnostic/comparison result is preserved with fair-pair boundaries and verified cleanup;
- no negative result is deleted or promoted to a PASS;
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.

## Do not expand into

- third-party targets or scanning;
- production firewall/route/DNS/proxy/tunnel/qdisc changes;
- a single run longer than the standing ten-minute limit or mechanically split long soak/pressure tests;
- public/general reachability or production claims from self-owned paths;
- RC/security approval before the bounded release matrix and independent review are complete;
- 0-RTT, enabled FEC, striping/aggregation, exotic carriers or other experimental features without an observed-problem gate;
- changing the frozen N9 canonical corpus unless a genuine corpus correctness defect is discovered.

## Questions requiring maintainer decision

none.
