# Nekomusume ChatGPT Handoff

Checked at: 2026-09-03 09:00 Asia/Shanghai
Repository HEAD: `5665b91f1ac01da1af478f87f24605c57f057336`
Previous reviewed implementation HEAD: `a7a003646def6095c6ad755bb4ae1159c060a1c0`
Previous reviewer handoff: `f17e7f7...`

## What changed

One new coding-agent commit is visible after the previous reviewer handoff:

- `5665b91` — **benchmark admission / result-validator repair only; no Nekomusume production wire, Session, failover or crypto semantic change; no new VPS comparison run.** It introduces a structured listener parser, explicit whole-lab/work/cleanup deadline metadata, fail-fast live shell behavior after a failed required sample, exact client/server binary hashes in result contracts, and stronger complete-result resource/identity validation.

The exact `5665b91` GitHub Actions run is independently green: `stable checks` passed and the nightly 30-second decode fuzz smoke passed.

This repair closes useful parts of the previous handoff, especially the shell-level stop-on-first-failed-sample path, full HY2 server sampler identity, explicit whole-lab bound fields, and complete-result binary/resource checks. However, review of the exact executable parser and validator/tests finds that the paid VPS pair is still not admission-safe. The remaining defects are in benchmark/evidence infrastructure, not Nekomusume transport semantics.

## Review verdict

**needs repair — benchmark admission is closer, but exact remote-listener proof and fail-closed validator behavior are still incomplete; do not spend the paid VPS pair yet**

Do not change Nekomusume wire, Session, failover or crypto semantics for this repair. Close the concrete admission/test defects below, obtain exact-repair-HEAD CI green, then immediately use the standing-authorized self-owned VPS/HY2 window.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline status.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- `5665b91` adds no HY2 paired sample and no performance conclusion.
- Exact-head CI at `5665b91` is green, but it only proves the tests that currently exist. It does not prove missing readiness/control-plane behavior regressions.
- The accepted D064 controlled application-fault warm VPS row and accepted approximately five-minute periodic direct-path row remain valid only at their recorded exact implementation/binary identities. Do not rerun them merely to consume rental time.
- IPv6 remains environment-blocked unless a genuinely owned end-to-end IPv6 path becomes available.
- Standing authorization already permits the intended bounded self-owned HY2/Nekomusume comparison, bounded capture and cleanup. No new per-run maintainer permission is needed once this gate is green.
- `docs/status.md`, `IMPLEMENTATION_PLAN.md` and `ROADMAP.md` remain stale relative to the accepted D064/periodic evidence and should be reconciled after the HY2 step closes honestly.

### R-HY2-13 — `parse-listener.py` does not match the production `ss` command shape

The new parser is structurally better than grep, but its input contract is currently incompatible with the harness call sites.

`parse-listener.py` expects:

```text
fields[0] == tcp|udp
local endpoint == fields[4]
```

while the harness calls protocol-filtered forms equivalent to:

```text
ss -H -ltn
ss -H -lun
```

On normal filtered Linux `ss` output the netid/protocol column is omitted; TCP rows start with `LISTEN`, UDP rows typically start with `UNCONN`, and the local endpoint is not at the parser's assumed field. A correctly bound remote listener can therefore fail readiness for a parser-format reason rather than a transport reason.

The harness also contains a `remote_listener_ready()` helper whose option construction is inconsistent with the actual call sites and which is not the single production readiness path. Dead/duplicate readiness logic is itself a drift risk.

Required repair:

- choose one exact `ss` invocation + parser contract and use it everywhere;
- do not require a protocol token if the shell invocation already fixes the protocol;
- parse the exact local endpoint address + port, including IPv4 and bracketed IPv6 forms;
- require exactly one matching exact dedicated bind address/port;
- reject wildcard, loopback, another interface address, wrong protocol, malformed input and ambiguous multiple matches;
- a same-numbered listener on the local client host must remain irrelevant;
- remove/fix duplicate readiness helpers so HY2 and Nekomusume readiness exercise the same tested primitive.

Add executable fixtures resembling the actual filtered output, e.g. TCP `LISTEN ... 192.0.2.9:40097 ...`, UDP `UNCONN ... 192.0.2.9:40098 ...`, plus IPv6-shaped rows.

### R-HY2-14 — complete-result validation can still accept a failed required sample

The live shell now stops comparative admission after the first failed sample, which is good. But the validator remains weaker than the artifact contract.

`validate_samples(..., require_complete=True)` currently accepts rows with `failures == 1` as long as the full required identity/order set exists. `expected_summary()` then computes latency medians from the successful subset. Therefore a hand-built, future-regressed or externally supplied `benchmark-result.v1` can still contain a failed required sample and pass complete-result validation if its summary matches that successful subset.

The contract is stricter:

```text
any required pair fails
=> diagnostic/blocked evidence
=> BLOCKED_HARNESS
=> no comparative summary
```

Required repair:

- `benchmark-result.v1` validation must require every required sample to be successful;
- a complete set containing any `failures != 0`, nonzero exit or failure stage must be rejected;
- add a mutation/regression test that changes one row in an otherwise complete result into a failed row and proves rejection;
- blocked artifacts must continue to retain the valid prefix and failed row without a summary.

### R-HY2-15 — the new deadline/provenance contract lacks mutation coverage

`5665b91` now validates explicit binary hashes and work/cleanup/whole-lab relationships, but the executable validator tests do not yet prove those gates fail closed under mutation.

Add mutations for at least:

- truncated/invalid `nekomusume_binary_sha256`;
- truncated/invalid `hy2_binary_sha256`;
- stale/mismatched client transport identity;
- stale/mismatched retained server transport identity;
- changed `work_deadline_ms`;
- changed `cleanup_reserve_ms`;
- changed `whole_lab_deadline_ms`;
- `whole_lab_deadline_ms > 600000`;
- arithmetic mismatch `work + cleanup != whole`;
- `bounds.maximum_duration_ms != whole_lab_deadline_ms`;
- inconsistent application-byte bound.

Green CI should prove these admission facts rather than only their happy path.

### R-HY2-16 — remote readiness/deadline behavior is still not exercised end-to-end

`compare-hy2-owned-lab-test.sh` remains dominated by topology/TLS/lifecycle static assertions and older failure/cleanup tests. The newly introduced remote listener parser and control-plane deadline behavior still need an executable fake-remote seam using the same helpers/path as production.

At minimum prove:

- exact remote bind + port + protocol succeeds;
- remote wildcard, wrong address, wrong protocol and malformed/ambiguous output fail;
- a local same-port listener cannot satisfy remote readiness;
- absent HY2 UDP listener fails at `hy2-server-readiness` before any timed client sample;
- absent Nekomusume TCP listener fails at `nekomusume-readiness` before that sample;
- an early remote process exit fails promptly;
- a hanging SSH/control-plane operation cannot exceed its remaining work/stage budget;
- work-deadline expiry still leaves the declared cleanup reserve available;
- an over-budget parameter profile is rejected before remote execution.

Static grep assertions may remain as guardrails, but they are not substitutes for these behavior tests.

## Work Package — Executable Benchmark Admission Closure -> Exact-Head CI -> Paid VPS Pair -> Matrix Reconciliation

### Primary A — Make one truthful, executable remote-listener readiness primitive

**Goal**

Close R-HY2-13 and make listener admission a stable reusable contract rather than a parser/call-site mismatch.

**Likely files**

- `scripts/bench/compare-hy2-owned-lab.sh`;
- `scripts/bench/parse-listener.py`;
- `scripts/bench/compare-hy2-owned-lab-test.sh`.

**Required behavior**

1. Select one real production input shape: either parse unfiltered `ss` output including netid, or parse protocol-filtered output and pass protocol separately. Do not mix the two contracts.
2. Match exact remote dedicated bind address + exact port + intended protocol.
3. Reject unspecified/wildcard, loopback, another address, wrong protocol and ambiguous duplicates.
4. Support IPv4 and bracketed IPv6 output shapes deterministically even though live IPv6 is environment-blocked.
5. Use this one primitive for both persistent HY2 UDP readiness and per-sample Nekomusume TCP readiness.
6. Preserve Nekomusume early-process-exit detection and bounded remote work calls.
7. Remove or unify dead/duplicate readiness helper code.
8. Add fixture tests matching real filtered/unfiltered `ss` grammar, not an invented shape.

**Completion:** the same executable helper path used in production passes exact-match fixtures and rejects all false-positive/format-error cases.

### Follow-up B — Make `benchmark-result.v1` intrinsically fail closed

**Dependency:** A green; may be developed in parallel if files do not conflict.

Close R-HY2-14 at the validator level, not only in shell control flow.

- require a complete benchmark result to contain all `2 * runs` rows and zero failed rows;
- require zero failure stages and successful exit/application/hash/resource evidence for all required rows;
- reject complete results containing any failed sample even if the summary is recomputed from successful rows;
- preserve `BLOCKED_HARNESS` prefix/failure evidence and no-summary semantics;
- add direct validator regression/mutation coverage.

Do not discard negative samples and do not turn them into zero-latency rows.

### Follow-up C — Close deadline, binary-identity and resource-identity mutations

**Dependency:** B green; may overlap with B where reviewable.

Close R-HY2-15.

Add validator mutations for all deadline/bound/hash/resource identity relationships listed above. Ensure retained server resources, if typed as transport resources, carry the full exact pinned implementation hash. A misleading partial identity must never be retained as exact evidence.

Keep server resource rows non-comparative if that is the existing lifecycle choice; do not manufacture symmetric server-startup performance numbers merely to satisfy a schema.

### Follow-up D — Exercise the real control-plane path with fake SSH/`ss`/timeout

**Dependency:** A-C green.

Close R-HY2-16 with behavior tests that invoke the same production helpers rather than only searching source strings.

Required scenarios:

1. exact TCP listener success;
2. exact UDP listener success;
3. wildcard/wrong-address/wrong-protocol/malformed/ambiguous failure;
4. local same-port listener irrelevant;
5. HY2 readiness absent -> typed blocker before client timing;
6. Nekomusume readiness absent -> typed blocker before client timing;
7. early remote process exit;
8. hanging remote call bounded by remaining work budget;
9. cleanup reserve still executable after work expiry;
10. over-budget profile rejected before remote execution.

Retain the existing certificate pin, bind/connect split, fresh-client lifecycle, process-group cleanup, partial-record and payload/resource truthfulness regressions.

### Follow-up E — Full local gate and exact-repair-HEAD CI

**Dependency:** A-D complete.

Run at minimum:

- exact listener-parser fixtures;
- owned-lab control-plane behavior regressions;
- result-validator mutation regressions;
- process-resource sampler regressions;
- `cargo fmt --all -- --check`;
- `cargo check --workspace --locked`;
- `cargo test --workspace --all-targets --locked --no-fail-fast`;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- `bash scripts/check.sh`;
- `git diff --check`.

Push one or more coherent repair commits. Require the **exact repair HEAD** GitHub `stable checks` and nightly decode fuzz smoke to be green before using the paid VPS benchmark window.

Do not treat the green `5665b91` CI as attestation for later repairs.

### Follow-up F — Immediately run the self-owned HY2/Nekomusume paired experiment

**Dependency:** E exact-head CI green.

Do not insert unrelated local polish between E and this run. This is the highest-value READY VPS opportunity once admission is trustworthy.

Use the existing standing authorization and the repaired owned-lab harness. Recommended bounded profile:

```text
self-owned client <-> owned VPS only
pinned HY2 v2.9.3 full SHA-256
5 paired runs if admitted
1200-byte deterministic payload per sample
concurrency 1
fresh unprivileged experiment ports
fresh client/session per timed sample
whole-lab maximum < 10 minutes including cleanup reserve
```

Keep the existing production HY2 service/config untouched. No production firewall/route/qdisc/DNS/proxy/tunnel change.

Because previous HY2 attempts failed during QUIC establishment, retain bounded diagnostics around only the disposable experiment UDP port:

- exact remote listener readiness;
- client connect authority vs server bind authority;
- whether UDP packets leave client, arrive VPS, elicit replies and return;
- redacted HY2 logs sufficient to distinguish path/NAT/provider vs TLS/auth/config failure;
- capture metadata/hash/counts/timestamps where captured; raw pcap need not be committed.

If **all** required pairs succeed:

- retain all raw sample rows;
- produce one bounded median/P95 comparison only for the complete success set;
- retain exact payload/hash and process-group client CPU/RSS/FD evidence;
- make no superiority, production or general-WAN claim.

If **any** required pair fails:

- retain the typed `BLOCKED_HARNESS`/diagnostic artifact;
- no comparative summary;
- do not repeat unchanged;
- cleanup still must be verified.

### Follow-up G — Reconcile release matrix and immediately select one next truthful VPS opportunity

**Dependency:** F succeeds or closes honestly with retained blocker evidence.

First repair stale status/evidence narrative without deleting historical negatives:

- `docs/status.md`: add the accepted positive D064 controlled-fault warm row, accepted approximately five-minute periodic row, exact final harness state, and exact HY2 outcome;
- `IMPLEMENTATION_PLAN.md`: remove stale statements that current-lineage D064/periodic are absent; keep bounded release matrix item 3 unchecked while declared rows remain unresolved;
- `ROADMAP.md`: preserve `controlled application fault != natural UDP degradation` and `five-minute sample != general long-lived stability`; update HY2 only to what F actually proved.

Then audit current live runtime surfaces and choose **at most one** additional VPS-only row that is already executable and truthfully instrumented:

1. genuine owned source-endpoint/path change;
2. real-session migration-back;
3. real-session key update;
4. live PMTUD observation.

A fixture/model is not a live runtime seam. If none is READY, record the exact implementation/instrumentation dependency and make that local unlock slice the next package instead of inventing a VPS experiment.

## Fallback

If A-D uncover a benchmark-harness defect beyond the listed evidence layer:

- keep all transport/governance flags unchanged;
- preserve a minimal deterministic reproducer;
- repair only the smallest harness/validator control path required for truthful evidence;
- rerun the full local/CI admission gate;
- do not spend the paid VPS comparison on a known-invalid harness.

If the final paid HY2 run is blocked by a changed path/provider condition after the repaired diagnostics classify it, preserve the negative evidence and move to another genuinely READY VPS-only release-evidence row; do not mechanically retry the same failed hypothesis.

## Completion gates

This package is complete only when all are true:

- exact remote listener parsing matches the actual production `ss` input contract;
- wildcard/wrong-address/wrong-protocol/local-host/ambiguous false positives are rejected;
- complete benchmark results cannot contain any failed required sample;
- blocked artifacts retain failure evidence and no summary;
- deadline/bound arithmetic is mutation-tested and whole-lab <= 600000 ms;
- binary/resource identities are exact 64-hex SHA-256 and mutation-tested;
- readiness/process/deadline behavior is exercised through the same production helper path;
- full local repository gate passes;
- exact repair HEAD CI is green;
- the self-owned paired run either yields a complete valid success batch or one retained typed blocker with no comparison summary;
- cleanup is verified;
- status/plan/roadmap are reconciled to accepted D064/periodic and exact HY2 evidence;
- no release/production/global-freeze flag is promoted automatically.

## Do not expand into

- Nekomusume wire/Session/crypto changes merely to accommodate the benchmark harness;
- rerunning accepted D064/periodic rows without a new question;
- publishing HY2 superiority from one batch;
- treating controlled application reply cessation as natural Internet/PTO degradation;
- pretending five minutes proves general long-lived reliability;
- third-party targets, scanning or production network changes;
- experimental FEC/0-RTT/striping/exotic carrier work without an observed-problem gate;
- IPv6 claims without a genuinely owned end-to-end IPv6 environment.

## Questions requiring maintainer decision

none.
