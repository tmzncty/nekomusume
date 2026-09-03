# Nekomusume ChatGPT Handoff

Checked at: 2026-09-03 10:00 Asia/Shanghai
Repository HEAD: `2a13992189999e83f145fbd6c4c3af691d122195`
Previous reviewed implementation HEAD: `5665b91f1ac01da1af478f87f24605c57f057336`
Previous reviewer handoff: `393c25d8f025b492c5084a0fc65aac16e8b7c778`

## What changed

Three new coding-agent commits are visible after the previous reviewer handoff. They are all benchmark/evidence-infrastructure work; none changes Nekomusume wire, Session, failover, crypto semantics or release governance, and none is a new VPS comparison run.

- `363bb58` — **listener parser/fixture repair.** `parse-listener.py` now accepts the two `ss -H` shapes relevant to this repository (state-first filtered output and a netid-retaining variant), matches an exact non-wildcard/non-loopback address+port+protocol, rejects ambiguous duplicates, and adds IPv4/IPv6-shaped fixtures plus a direct parser test.
- `ce126a5` — **complete-result fail-closed repair.** `benchmark-result.v1` now rejects a required sample with `failures != 0`, rejects unsuccessful/missing per-sample client transport resource evidence, and adds a direct failed-sample/resource regression. A failed required pair can no longer be converted into a valid complete comparison merely by recomputing the summary from the remaining successes.
- `2a13992` — **identity/deadline/bound mutation tests.** The validator test now mutates pinned Nekomusume/HY2 hashes, client/server identities, work/cleanup/whole-lab deadline relations, the 600-second ceiling and application-byte bound and proves the current validator rejects those altered artifacts.

The exact `2a13992` GitHub Actions run is independently green: `stable checks` completed successfully and the nightly 30-second decode fuzz smoke completed successfully.

These commits materially close R-HY2-14 and most of R-HY2-15. They also make the parser itself compatible with the production `ss -H -ltn/-lun` state-first shape. However, the previous Primary is not yet complete because the production control-plane path is still not exercised end-to-end, and review of the exact current shell uncovers one concrete live-run staging defect that would make spending the paid VPS window premature.

## Review verdict

**needs repair — validator/parser logic is much closer, but the executable paid-VPS admission path is still not proven and currently contains a concrete parser-staging defect; do not run the HY2 pair yet**

The next slice remains benchmark/evidence infrastructure only. Do not modify Nekomusume wire, Session, Noise, failover or crypto semantics to make the benchmark convenient.

Once the control-plane behavior is exercised through the real production helper path, the parser fixture is part of the full repository gate, the exact repair HEAD CI is green, and the staging defect below is closed, the coding agent is pre-authorized to proceed directly to the standing-authorized self-owned VPS paired run without waiting for another reviewer handoff.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research implementation baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- `363bb58`, `ce126a5`, and `2a13992` are benchmark/evidence-infrastructure commits, not transport capability evidence.
- Exact-head GitHub CI at `2a13992` is green. However, `scripts/check.sh` currently does **not** invoke the newly added `scripts/bench/parse-listener-test.py`, so the green stable job does not independently attest the new parser fixture test even though the repository-wide shell/validator checks are green.
- No new HY2 paired samples, comparative medians/P95 or superiority evidence exist at this HEAD.
- The accepted D064 controlled application-fault warm VPS row and accepted approximately five-minute periodic direct-path row remain valid only at their recorded exact implementation/binary identities. Do not rerun them merely to consume rental time.
- IPv6 remains environment-blocked unless a genuinely owned end-to-end IPv6 path becomes available.
- Standing authorization already permits the intended bounded self-owned HY2/Nekomusume paired experiment, bounded capture/diagnostics and cleanup once the harness admission gate is green. No new per-run maintainer permission is required.
- `docs/status.md`, `IMPLEMENTATION_PLAN.md` and `ROADMAP.md` remain stale relative to the accepted D064/periodic positive evidence and the evolving HY2 harness state. Reconcile them after the HY2 step closes honestly; do not erase historical negative rows.

### R-HY2-13 status — parser semantics substantially repaired, integration still incomplete

`363bb58` fixes the parser's earlier field-shape mismatch: state-first `ss -H -ltn/-lun` rows and netid-retaining rows are both handled, protocol state is checked, exact address+port is required, and ambiguous duplicates fail.

Two integration gaps remain:

1. `scripts/check.sh` does not run `scripts/bench/parse-listener-test.py`. Therefore the direct parser fixtures are not part of the current CI/stable admission gate.
2. `compare-hy2-owned-lab.sh` still contains an unused/duplicate `remote_listener_ready()` helper whose command construction uses `ss -H -l${protocol}` while the real call sites use explicit `ss -H -lun` / `ss -H -ltn`. This dead duplicate is a drift trap and violates the previous requirement to have one tested readiness primitive.

The production HY2/Nekomusume loops should use one helper whose exact command/parser path is the path exercised by tests.

### R-HY2-14 status — closed at validator level

`ce126a5` closes the previously identified complete-result hole:

- a complete required sample set with any `failures != 0` is rejected;
- complete per-sample client resource evidence must be exactly present once for every Nekomusume/HY2 run;
- accepted client resource rows must have `exit={code:0,timed_out:false}`;
- negative/blocked artifacts remain the place for retained failure rows without a comparative summary.

Do not reopen this unless the new control-plane tests expose a contradiction.

### R-HY2-15 status — accepted as useful mutation coverage, with no release claim

`2a13992` adds the requested identity/deadline/application-bound mutation matrix. The validator code itself separately enforces exact 64-hex binary hashes, work+cleanup=whole, whole=maximum bound, whole <= 600000 ms, application-byte arithmetic, client lifecycle/resource scope and pinned client/server transport identities.

This is evidence-schema hardening only. It does not prove the shell actually respects those deadlines under a hanging/failed remote control-plane operation; that remains R-HY2-16.

### R-HY2-16 — still open: no executable fake-remote proof of the production control path

The current `compare-hy2-owned-lab-test.sh` still primarily exercises:

- preflight/address guards through `--validate`;
- static source assertions for bind/connect/TLS/fresh lifecycle;
- standalone result-validator behavior;
- local process-group cleanup/race behavior.

It does not yet drive the same live shell path used by the paid run through a deterministic fake SSH/`ss`/remote-process seam.

The following behaviors therefore remain unproven by the current CI gate:

- exact remote HY2 UDP listener readiness succeeds through the production helper;
- exact remote Nekomusume TCP listener readiness succeeds through the production helper;
- wildcard/wrong address/wrong protocol/malformed/ambiguous remote rows fail through that same helper;
- a local same-numbered listener cannot satisfy remote readiness;
- missing HY2 UDP readiness produces `hy2-server-readiness` before any timed client sample;
- missing Nekomusume TCP readiness produces `nekomusume-readiness` before the corresponding timed sample;
- early remote process exit fails promptly;
- hanging SSH/control-plane work is bounded by the remaining work deadline;
- work deadline exhaustion still leaves the declared cleanup reserve usable;
- an over-budget profile is rejected before remote experiment work begins.

Static grep assertions are useful guardrails but are not a substitute for these behavior tests.

### R-HY2-17 — new concrete blocker: `parse-listener.py` is referenced by the remote tar set but never staged into `$run`

At the exact current HEAD, `compare-hy2-owned-lab.sh` stages:

```text
neko-cli
hysteria
process-resource-sampler.py
echo-payload.py
```

into the local temporary run directory, then later creates the echo server/config/cert material. The remote tar command includes:

```text
parse-listener.py
```

and the remote readiness commands execute:

```text
python3 '$remote/parse-listener.py'
```

but the script never copies `scripts/bench/parse-listener.py` into `$run` before that tar command.

A real run can therefore fail during staging before reaching the intended readiness proof. This must be fixed and covered by the same executable control-plane regression before the paid VPS pair.

This is a benchmark-harness defect, not a transport defect.

## Work Package — Control-Plane Admission Closure -> Exact-Head CI -> Paid HY2 Pair -> Matrix Reconciliation

### Primary A — Make one production readiness primitive and prove its remote artifact is actually staged

**Goal**

Close R-HY2-13 integration drift and R-HY2-17 before any VPS execution.

**Likely files**

- `scripts/bench/compare-hy2-owned-lab.sh`;
- `scripts/bench/parse-listener.py` only if the exact production contract needs a small correction;
- `scripts/bench/parse-listener-test.py`;
- `scripts/bench/compare-hy2-owned-lab-test.sh`;
- `scripts/check.sh`.

**Required behavior**

1. Use exactly one readiness helper for both HY2 UDP and Nekomusume TCP remote listener admission.
2. The helper must invoke the exact intended `ss` shape (`-lun` for UDP, `-ltn` for TCP, or one explicitly documented equivalent) and feed that shape to the exact parser covered by fixtures.
3. Remove or replace the dead `remote_listener_ready()` path that constructs `ss -H -l${protocol}` if it is not the production contract.
4. Ensure `parse-listener.py` is actually available wherever production executes it. Either:
   - stage/copy it into `$run` before the remote tar and verify the tar set contains it; or
   - parse remote `ss` output locally through the repository parser so no remote parser copy is required.
   Choose one clear contract; do not keep both partially alive.
5. Keep exact non-wildcard dedicated bind address + exact port + protocol semantics and deterministic IPv4/bracketed-IPv6 parsing.
6. Wire `python3 scripts/bench/parse-listener-test.py` into `scripts/check.sh` (or into an already-called test that truly executes it) so exact parser fixtures are independently exercised by stable CI.
7. Add a regression that would fail if a required remote helper/artifact named by the production command is absent from the staged run/tar set.

**Completion definition**

There is one readiness primitive, its parser/artifact path is real and staged, the exact production input shape is fixture-tested, and the parser test is part of the full repository/CI gate.

### Follow-up B — Exercise the production control-plane path with deterministic fake SSH/`ss`/remote processes

**Dependency:** A green.

Close R-HY2-16. Prefer extending the existing owned-lab test harness rather than inventing a second orchestration implementation.

The test seam must exercise the same production functions/branches used by a real run. It may fake SSH, `ss`, remote process status, time/deadline progression and experiment binaries, but it must not merely grep the shell source.

Required scenarios:

1. exact remote UDP listener success;
2. exact remote TCP listener success;
3. wildcard remote listener rejection;
4. wrong-address remote listener rejection;
5. wrong-protocol remote listener rejection;
6. malformed/ambiguous remote output rejection;
7. local same-port listener irrelevant to remote readiness;
8. HY2 listener absent -> typed `hy2-server-readiness` blocker before any timed sample;
9. Nekomusume listener absent -> typed `nekomusume-readiness` blocker before that timed sample;
10. early remote process exit fails promptly;
11. hanging remote/control operation cannot exceed remaining work/stage budget;
12. work deadline expiry still permits the declared cleanup reserve path;
13. whole-lab/work/cleanup over-budget configuration is rejected before remote experiment execution;
14. missing staged parser/helper artifact fails in deterministic local test rather than on the paid VPS.

Preserve all prior bind/connect, certificate-pin, fresh-client lifecycle, typed partial failure, process-group cleanup and result-validator regressions.

**Completion definition**

A deterministic test can demonstrate both positive readiness and all material fail-closed branches through the same helper/control path that the paid run will use.

### Follow-up C — Full local gate and exact-repair-HEAD GitHub CI

**Dependency:** A-B complete.

Run at minimum:

- direct parser fixtures;
- owned-lab fake-remote/control-plane behavior regressions;
- result-validator complete/blocked regressions;
- identity/deadline/bound mutation matrix;
- process-resource sampler regressions;
- `cargo fmt --all -- --check`;
- `cargo check --workspace --locked`;
- `cargo test --workspace --all-targets --locked --no-fail-fast`;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- `bash scripts/check.sh`;
- `git diff --check`.

Push coherent repair commits and wait for the **exact repair HEAD** GitHub `stable checks` and nightly decode fuzz smoke to be green.

The coding agent is explicitly pre-authorized to continue directly to D as soon as exact-head CI is green. Do not wait for another reviewer turn if all A-C gates are satisfied.

### Follow-up D — Immediately spend the standing-authorized VPS window on one fair HY2/Nekomusume paired attempt

**Dependency:** C exact-head CI green.

This is the highest-value READY rental-window task after admission closure. Do not insert unrelated local polish between C and this run.

Use the already established bounded profile unless the harness itself computes a smaller safe profile:

```text
self-owned client <-> owned VPS only
pinned HY2 v2.9.3 exact SHA-256
5 paired runs if admitted
1200-byte deterministic payload per sample
concurrency 1
fresh unprivileged experiment ports
fresh transport client/session per timed sample
whole-lab bound < 10 minutes including cleanup reserve
```

Keep the existing production HY2 service/config untouched. Do not alter production firewall, route, qdisc, DNS, proxy or tunnel state.

Because historical HY2 attempts failed during QUIC establishment, keep changed-hypothesis bounded diagnostics around only the disposable experiment UDP port:

- prove the temporary HY2 server is bound on the exact dedicated local bind address/port;
- prove the client targets the verified connect authority rather than the local bind address;
- observe whether experiment UDP leaves the client, reaches the VPS, elicits a reply and returns;
- retain redacted HY2/Nekomusume logs sufficient to distinguish network/path vs TLS/auth/config failure;
- retain capture metadata/hash/counts/timestamps where captured; raw pcap does not need to enter Git;
- always verify experiment-owned process/listener/temp-path cleanup.

If all required pairs succeed:

- retain every raw sample;
- validate one complete `benchmark-result.v1`;
- calculate median/P95/failures only from the complete successful set;
- retain exact payload hash/application bytes and process-group client CPU/RSS/FD evidence;
- make only a route/time-window/batch-specific comparison; no superiority or production claim.

If any required pair fails:

- retain one typed `BLOCKED_HARNESS`/diagnostic artifact with the valid prefix + failed row and no comparative summary;
- preserve changed-hypothesis diagnostics;
- do not repeat unchanged;
- move to the next truthful VPS opportunity after status reconciliation.

### Follow-up E — Reconcile accepted VPS evidence and the exact HY2 outcome into the release matrix

**Dependency:** D closes either positively or honestly blocked.

Repair current status/evidence drift without deleting historical negatives:

1. `docs/status.md`
   - replace stale `current-exact-head VPS warm evidence remains absent` wording with the accepted exact `25e0daa` positive D064 controlled application-fault result and its strict boundary;
   - add the accepted approximately five-minute periodic direct-path result and retain the fact that it is one bounded sample, not a production reliability rate;
   - replace the older local-only HY2 harness note with the exact final admission state and D outcome.
2. `IMPLEMENTATION_PLAN.md`
   - remove stale statements that current-lineage D064/periodic rows are absent;
   - keep bounded release matrix item 3 unchecked while declared IPv6/environment, natural degradation, NAT/endpoint-change, HY2 or other required rows remain unresolved;
   - record a HY2 comparison as positive only if D produces a complete valid paired set.
3. `ROADMAP.md`
   - preserve `controlled application-level UDP reply cessation != natural network UDP degradation/PTO blackhole`;
   - preserve `approximately five minutes != general long-lived stability`;
   - update HY2 only to the exact D evidence.

Governance flags stay unchanged.

### Follow-up F — Use the remaining rental window for one next genuinely READY VPS-only row

**Dependency:** E complete.

Do not rerun the accepted D064/periodic rows or an unchanged failed HY2 hypothesis.

Audit live runtime/instrumentation and choose at most one row that is already executable and truthful:

1. genuine owned source-endpoint/path change;
2. real-session migration-back;
3. real-session key update;
4. live PMTUD observation.

A fixture/state model is not a live runtime seam. If none is READY, record the exact implementation/instrumentation dependency and make that local unlock slice the next work package. Do not spend VPS time manufacturing evidence for a capability that is not actually wired to the live runtime.

## Fallback

If A-B exposes a benchmark-harness defect deeper than the listed control/evidence layer:

- keep all transport/release governance flags unchanged;
- preserve a minimal deterministic reproducer;
- fix only the smallest harness/validator path required for truthful evidence;
- rerun the full local + exact-head CI admission gate;
- do not spend the paid VPS window on a known-invalid harness.

If D is blocked by a changed path/provider condition after the repaired diagnostics classify it, preserve the negative evidence and move to another genuinely READY VPS-only row after E; do not mechanically retry the same failed hypothesis.

## Completion gates

This package is complete only when all are true:

- one production readiness helper is used for both remote protocols;
- required parser/helper artifacts are actually staged or parsing is intentionally local;
- the exact parser fixture test is part of `scripts/check.sh`/stable CI;
- exact remote listener parsing rejects wildcard/wrong-address/wrong-protocol/malformed/ambiguous/local-host false positives;
- complete benchmark results cannot contain failed required samples;
- blocked artifacts retain failure evidence and no summary;
- identity/deadline/application-bound mutation gates remain green;
- production readiness/process/deadline behavior is exercised by deterministic fake-remote tests, not source grep alone;
- full local repository gate passes;
- exact repair HEAD stable CI and nightly fuzz smoke are green;
- the self-owned paid pair yields either one complete valid success batch or one retained typed blocker with no comparison summary;
- cleanup is verified;
- status/plan/roadmap are reconciled to accepted D064/periodic and exact HY2 evidence;
- no release/production/global-freeze flag is promoted automatically.

## Do not expand into

- Nekomusume wire/Session/Noise/crypto changes to satisfy benchmark infrastructure;
- rerunning accepted D064/periodic rows without a new research question;
- publishing HY2 superiority from one batch;
- long-lived daemon/service deployment;
- third-party targets or scans;
- production firewall/route/qdisc/DNS/proxy/tunnel modification;
- experiments outside `docs/standing-vps-lab-authorization.md`;
- speculative 0-RTT/FEC/multipath/exotic-carrier work without an observed-problem gate.

## Questions requiring maintainer decision

none.
