# Nekomusume ChatGPT Handoff

Checked at: 2026-09-03 05:02 Asia/Shanghai
Repository HEAD: `a7a003646def6095c6ad755bb4ae1159c060a1c0`
Previous reviewed implementation HEAD: `fde80eba5aafa0394e994f40e345bc2f0b381e0f`
Previous reviewer handoff commit: `fc3c0b3c76fd36771ac9155fb77bc7553a4f85d9`

## What changed

One new coding-agent commit is visible after the previous reviewer handoff:

- `a7a0036` — **benchmark harness / result-validator repair only; no Nekomusume production wire, Session, failover or crypto semantic change; no new VPS comparison run.** It moves Nekomusume readiness observation onto the remote VPS, adds an explicit remote HY2 UDP-listener readiness stage, introduces work and cleanup deadlines with bounded SSH operations, passes expected client binary identities into sample construction, validates result bounds, and tightens client resource identity / exit agreement.

The exact `a7a0036` GitHub Actions run is green. `stable checks` completed successfully and the nightly 30-second `decode` fuzz smoke completed successfully.

Several previous findings are materially improved, but the paid-VPS pair is still not admissible. Review of the executable shell/validator contract finds four remaining evidence-integrity defects: the listener-readiness matcher is not a truthful exact-bind parser, a failed paired sample can still flow into a complete result with a comparative summary, the published duration bound under-reports the allowed cleanup window, and binary-identity validation remains incomplete for retained server resources / contract hashes. In addition, the new readiness/deadline behavior is not covered by executable regressions; current CI green therefore proves syntax and the existing tests, not these new runtime-control properties.

## Review verdict

**needs one final benchmark-admission repair before the VPS pair — CI green, but do not spend the paid VPS window on `a7a0036`**

This is a benchmark/evidence-harness correctness gate, not a Nekomusume transport defect. Keep wire, Session, failover and crypto semantics unchanged. Close the four concrete defects below, add executable regressions for the new control-plane behavior, obtain exact-repair-HEAD CI green, then immediately run the self-owned HY2/Nekomusume paired experiment under standing authorization.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline status.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- `a7a0036` adds no new WAN/HY2 comparative sample and no performance conclusion.
- Exact-head CI at `a7a0036` is independently green for `stable checks` and the nightly decode fuzz smoke. This does not prove remote listener parsing, whole-lab deadline behavior or fail-closed comparative-result classification because those new behaviors are not yet exercised by deterministic tests.
- The accepted D064 controlled application-fault warm VPS result and approximately five-minute periodic direct-path VPS result remain valid only at their recorded exact identities. Do not rerun them merely to consume rental time.
- IPv6 remains environment-blocked unless a genuinely owned end-to-end IPv6 path becomes available.
- Standing authorization permits the intended self-owned paired benchmark, bounded capture, cleanup and ordinary high-port TCP/UDP listeners. No new per-run permission is required once this harness gate is closed.
- `docs/status.md`, `IMPLEMENTATION_PLAN.md`, and `ROADMAP.md` still contain stale narrative relative to the accepted D064/periodic evidence. Repair that drift after the HY2 step succeeds or closes honestly with a retained blocker artifact; do not erase historical negative attempts.

### Reviewer finding R-HY2-08 — remote readiness matching is not an exact listener proof

The new readiness checks execute remotely, which fixes the host-locality error, but the actual matcher is not correct enough to authorize a paid run.

Current HY2/TCP readiness uses a pattern equivalent to:

```text
:(<bind-address>|0.0.0.0):<port><space>
```

against normal `ss -H -lun` / `ss -H -lnt` output. Typical local-address fields are shaped like:

```text
192.0.2.9:40098
```

not `:192.0.2.9:40098`, so the added leading colon can make a correctly bound listener fail readiness for the wrong reason. The alternative `0.0.0.0` is also contrary to the owned-lab contract: readiness must prove the exact dedicated bind address, not accept a wildcard listener.

Replace grep-shape inference with an exact, testable listener parser/command. It must:

- execute on the VPS;
- select the exact intended TCP/UDP port;
- normalize IPv4/IPv6 local-address representation;
- require the exact `LAB_REMOTE_BIND_ADDRESS`;
- reject wildcard, loopback, another local address and the correct port on the wrong protocol;
- fail closed when `ss` output is malformed or ambiguous.

A local listener with the same numeric port must remain irrelevant.

### Reviewer finding R-HY2-09 — a failed sample can still produce a comparative summary

`run_client()` records a typed failed row but does not stop the paired benchmark. The loop can continue, and final assembly builds `nekomusume.benchmark-result.v1`; `validate_result()` accepts a complete sample set containing failures and `expected_summary()` computes medians from the successful subset.

That violates the current owned-lab comparison contract and the previous handoff: **if any required Nekomusume/HY2 pair fails, the run is diagnostic/availability evidence, not comparative performance evidence.** A failed required sample must produce `BLOCKED_HARNESS` with retained sample evidence and no comparative `summary`.

Required behavior:

- after appending the first failed required sample, stop admitting new comparative samples (or finish only a specifically required diagnostic stage that cannot be mistaken for a benchmark sample);
- run bounded cleanup;
- emit one typed blocker artifact with the retained prefix/failed row and no summary;
- `benchmark-result.v1` is legal only when all `2 * RUNS` required samples exist and every one is successful;
- validator mutation tests must reject a complete benchmark-result containing even one failed sample.

Do not discard the failure row; preserving it is part of the evidence contract.

### Reviewer finding R-HY2-10 — advertised duration bound under-reports the full allowed lab window

`a7a0036` creates a work deadline and a later cleanup deadline. With the default profile, work may continue until the work deadline and cleanup may then continue within the reserved cleanup window. However the complete result currently emits:

```text
bounds.maximum_duration_ms = GLOBAL_DEADLINE_MS
```

where `GLOBAL_DEADLINE_MS` is the **work** deadline, not the later whole-lab deadline including cleanup. The artifact can therefore advertise a smaller maximum wall duration than the harness actually permits.

Standing authorization bounds the complete public experiment wall-clock, including cleanup. Make the result contract truthful by explicitly separating at least:

```text
work_deadline_ms
cleanup_reserve_ms
whole_lab_deadline_ms
```

or equivalent fields, with `bounds.maximum_duration_ms` representing the actual whole-lab maximum. Require `whole_lab_deadline_ms <= 600000` and mechanically bind the validator to the emitted/enforced values. A mutation that changes any deadline/bound relationship must fail.

The current remaining-budget SSH wrapper is directionally useful; do not replace it with an unbounded control-plane call.

### Reviewer finding R-HY2-11 — retained binary identity is still not fully exact/fail-closed

The client path now passes expected full hashes, but the persistent HY2 server resource row is still launched with a truncated identity (`sha256:66dbdb0608f25f30`), while the pinned HY2 artifact is a full 64-hex SHA-256. That makes a retained resource artifact's identity non-exact even though server metrics are separately labelled and excluded from client latency.

Also, complete-result validation builds pinned client identities from contract strings without first requiring both contract binary hashes to be valid 64-hex SHA-256 values.

Required repair:

- use the exact full pinned HY2 SHA-256 for the HY2 server sampler identity;
- use the exact copied Nekomusume binary SHA-256 for Nekomusume server resource rows as already intended;
- require `contract.nekomusume_binary_sha256` and `contract.hy2_binary_sha256` to match the SHA-256 grammar before using them;
- require all retained transport resource rows that claim a binary identity to be internally consistent with their implementation's exact pinned binary identity, or explicitly exclude a non-comparative row from the typed resource set rather than retaining a misleading identity;
- add mutation tests for truncated/invalid/stale binary identity.

### Reviewer finding R-HY2-12 — the new readiness/deadline code lacks executable regressions

The exact-head CI is green, but `scripts/bench/compare-hy2-owned-lab-test.sh` still mostly checks topology/TLS/lifecycle source strings and earlier cleanup/failure-row behavior. It does not execute the new remote readiness parser, prove a local listener cannot satisfy it, exercise HY2 readiness absence, or demonstrate deadline expiry against a hanging control-plane command.

Before a paid run, add a deterministic test seam around the new shell control-plane logic. The test does not need a real VPS; fake SSH/`ss`/timeout commands are acceptable if they exercise the same helper functions/command path used by production harness code.

At minimum prove:

- exact remote bind+port+protocol succeeds;
- same port on the local test host is irrelevant;
- remote wildcard and wrong-address listeners fail;
- absent HY2 UDP listener fails at `hy2-server-readiness` before client timing;
- absent Nekomusume TCP listener fails at `nekomusume-readiness` before client timing;
- an early remote process exit fails promptly;
- a hanging remote/control-plane operation cannot exceed its remaining stage/global budget;
- cleanup reserve remains available after work-deadline expiry;
- an over-budget parameter profile is rejected before remote execution.

Static `grep` assertions are useful guardrails but are not substitutes for these behavior tests.

## Work Package — Final Benchmark Admission Repair -> Exact-Head CI -> VPS Pair -> Release-Matrix Reconciliation

### Primary A — Build one exact remote-listener readiness primitive and use it everywhere

**Goal**

Close R-HY2-08 and make the remote readiness evidence reusable/testable instead of embedding fragile regexes at two call sites.

**Scope**

- `scripts/bench/compare-hy2-owned-lab.sh`;
- `scripts/bench/compare-hy2-owned-lab-test.sh`;
- a small helper under `scripts/bench/` only if it materially improves structured parsing/testing.

**Required behavior**

1. Implement one bounded remote-listener check that receives protocol, exact remote bind address and port.
2. Parse actual/fixture `ss` output structurally enough to compare the local endpoint exactly; do not accept `0.0.0.0`, `*`, `::`, another interface address or another protocol as equivalent.
3. Use the same primitive for persistent HY2 UDP readiness and per-sample Nekomusume TCP readiness.
4. Preserve early-process-exit detection for Nekomusume.
5. Keep every remote observation under the existing remaining-budget/cleanup-budget wrapper.
6. Add executable IPv4 and IPv6-shaped fixture tests even if real owned IPv6 execution remains environment-blocked.

**Completion:** both readiness paths are exact-bind, remote-only, bounded, and behavior-tested.

### Follow-up B — Make comparative result classification fail closed and make wall bounds truthful

**Dependency:** A green.

Close R-HY2-09 and R-HY2-10 together because both define whether the artifact is admissible evidence.

1. First failed required sample -> retain row -> stop comparative admission -> bounded cleanup -> `BLOCKED_HARNESS` -> no summary.
2. `benchmark-result.v1` -> all required pairs present **and all successful**.
3. Split work/cleanup/whole-lab deadline metadata explicitly.
4. `bounds.maximum_duration_ms` must equal the full enforced whole-lab wall bound, not only the work deadline.
5. Validator must reject:
   - a complete result containing any failed sample;
   - missing/altered work/cleanup/whole-lab deadline metadata;
   - `whole_lab > 600000`;
   - arithmetic relationships that do not match the enforced harness budget;
   - an application-byte bound inconsistent with payload × runs × implementations.
6. Preserve retained prefix evidence on deadline expiry; budget expiry is `BLOCKED_HARNESS`, not a benchmark failure sample used in medians.

### Follow-up C — Finish exact binary provenance and executable control-plane regressions

**Dependency:** B green; may be developed in parallel with B if files do not conflict.

Close R-HY2-11 and R-HY2-12.

- replace truncated HY2 server resource identity with the full pinned v2.9.3 SHA-256;
- validate both contract binary hashes as exact SHA-256;
- bind retained transport resource identities to the correct implementation hash;
- add stale/truncated identity mutations;
- add fake-remote readiness/deadline regressions listed in R-HY2-12;
- retain existing topology split, certificate pin, fresh-client lifecycle, cleanup, partial-record and resource-sampler regressions.

Do not alter Nekomusume transport semantics to make the harness convenient.

### Follow-up D — Full local gate and exact-repair-HEAD CI

**Dependency:** A-C complete.

Run at minimum:

- owned-lab harness behavior regressions;
- result-validator mutation regressions;
- resource-sampler regressions;
- `cargo fmt --all -- --check`;
- `cargo check --workspace --locked`;
- `cargo test --workspace --all-targets --locked --no-fail-fast`;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- `bash scripts/check.sh`;
- `git diff --check`.

Push the coherent repair. Require the **exact repair HEAD** GitHub `stable checks` and nightly decode fuzz smoke to be green. The green `a7a0036` jobs attest only to `a7a0036`.

### Follow-up E — Immediately run one changed-hypothesis self-owned HY2/Nekomusume pair

**Dependency:** D exact-head CI green.

Do not insert unrelated local polish between D and the VPS run. The rental window is time-limited and the harness repair is specifically intended to unlock this evidence.

Recommended bounded profile, subject to the repaired admission calculation:

```text
self-owned client <-> owned VPS only
pinned HY2 v2.9.3 full SHA-256
5 paired runs if admitted
1200-byte deterministic payload
concurrency 1
fresh unprivileged experiment ports
fresh client/session per timed sample
whole-lab maximum < 10 minutes including cleanup reserve
```

Use the existing standing authorization. Do not modify production firewall, route, qdisc, DNS, proxy/tunnel or the existing production HY2 service/config.

Because the previous changed-hypothesis HY2 attempt failed during QUIC establishment, collect bounded path diagnostics around only the disposable HY2 UDP port:

- prove exact remote listener readiness first;
- prove clients target the verified connect address rather than the bind address;
- if HY2 still fails, record whether packets leave the client, reach the VPS, elicit replies and whether replies return;
- retain packet counts/timestamps/capture hash or compact redacted metadata plus redacted HY2 logs sufficient to distinguish path/NAT/provider failure from TLS/auth/config failure;
- raw pcap need not be committed;
- do not change provider/firewall policy merely to force a success.

If **all** required pairs succeed, retain raw rows and calculate one bounded median/P95/failure-free comparative batch with exact application bytes/hash and process-group client CPU/RSS/FD evidence. Make no superiority/general-WAN claim.

If **any** required pair fails, retain the typed blocker artifact with no comparative summary and do not repeat unchanged.

Always verify cleanup of experiment-owned processes/listeners/temp paths.

### Follow-up F — Reconcile the release matrix, then select at most one next READY VPS-only row

**Dependency:** E succeeds or closes honestly with a retained blocker artifact.

First repair status/evidence drift without deleting historical negatives:

- `docs/status.md`: add the accepted D064 controlled-fault warm positive row, accepted approximately five-minute periodic direct-path row, the exact final HY2 harness state and the exact paired-run outcome;
- `IMPLEMENTATION_PLAN.md`: remove stale claims that current-lineage D064/periodic evidence is absent; keep bounded release matrix item 3 unchecked while declared rows remain unresolved;
- `ROADMAP.md`: preserve controlled application fault != natural UDP degradation and five-minute sample != general long-lived stability; update HY2 only to the exact outcome.

Then audit current runtime surfaces and choose **at most one** additional high-value VPS-only row that is already executable and truthfully instrumented: genuine owned endpoint/source change, real-session migration-back, real-session key update, or live PMTUD. A fixture/model is not a live runtime seam. If none is READY, record the exact local implementation dependency and make that unlock slice the next package.

## Completion gates

This package is complete only when all are true:

- remote readiness proves the exact dedicated bind address + intended port + protocol and rejects wildcard/other-address/local-host false positives;
- readiness/deadline behavior has executable deterministic regression coverage, not only static source checks;
- first failed required benchmark sample produces `BLOCKED_HARNESS` with no comparative summary;
- a complete benchmark result requires all required paired samples and zero failures;
- result duration metadata distinguishes work and cleanup and truthfully reports the full enforced whole-lab maximum <= 600000 ms;
- control-plane operations remain bounded by remaining work/cleanup budget;
- HY2 and Nekomusume retained transport resources use exact binary identities; contract binary hashes are validated as real SHA-256 values;
- exact repair-HEAD stable CI + nightly fuzz are green before VPS execution;
- the next VPS run either yields one complete fair paired batch or one changed-hypothesis typed blocker with bounded diagnostics and verified cleanup;
- historical D064/periodic/negative evidence remains immutable and is reconciled into status without claim inflation;
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.

## Fallback

If the repaired harness still cannot produce truthful exact readiness/budget/provenance evidence locally, keep the VPS pair blocked and preserve the smallest failing deterministic fixture. Do not bypass the harness gate to save rental time.

If exact-repair CI is red, fix that exact regression before touching the VPS. A green older HEAD is not sufficient.

If the VPS HY2 attempt remains path-blocked after E with the new exact readiness and bounded packet-direction evidence, retain the negative artifact and move to the next genuinely READY VPS-only row rather than rerunning unchanged.

## Do not expand into

- Nekomusume wire/Session/Noise/failover semantic changes for benchmark convenience;
- provider/firewall/route/qdisc/DNS/proxy/tunnel modifications;
- third-party targets or scanning;
- a >10-minute experiment or mechanical splitting to evade the standing bound;
- performance-superiority language from one batch;
- rerunning accepted D064/periodic rows without a new hypothesis;
- IPv6 claims without a real owned end-to-end IPv6 path;
- 0-RTT, enabled FEC, concurrent striping, heterogeneous aggregation or exotic carriers without an observed-problem gate.

## Questions requiring maintainer decision

none.
