# Nekomusume ChatGPT Handoff

Checked at: 2026-09-03 11:02 Asia/Shanghai
Repository HEAD: `026586859214fa712a8259bd374ed0d9631645be`
Previous reviewed implementation HEAD: `2a13992189999e83f145fbd6c4c3af691d122195`
Previous reviewer handoff commit: `b3a1e58f87f6bc758077f351c6f4b5ddf91b211f`

## What changed

One new coding-agent commit is visible after the previous reviewer handoff:

- `0265868` — **benchmark/control-plane test infrastructure; no Nekomusume wire, Session, Noise, failover or release-governance semantic change.** It extracts the owned-lab deadline/SSH/listener-readiness logic into `scripts/bench/owned-lab-control-plane.sh`, adds deterministic fake-remote behavior coverage in `owned-lab-control-plane-test.sh`, stages `parse-listener.py` into the paid-run artifact set, routes HY2 UDP and Nekomusume TCP readiness through the same helper, uses bounded stage/client deadlines through the shared control-plane helper, and wires the new parser/control-plane tests into `scripts/check.sh`.

This materially addresses the previous R-HY2-13/R-HY2-16/R-HY2-17 admission work. The new helper uses exact remote `ss -H -ltn` / `ss -H -lun` shapes and parses those through the strict repository parser; the fake control-plane test covers exact TCP/UDP readiness, wildcard/wrong-address/wrong-protocol/malformed/ambiguous rejection, local-same-port irrelevance, typed readiness failures, early remote process exit, bounded hanging control operations, cleanup reserve after work expiry and pre-remote over-budget rejection.

However, the **exact current HEAD is not green**. GitHub Actions run `33708014112` at `0265868` completed with:

- `nightly decode fuzz smoke`: **PASS**;
- `stable checks`: **FAIL** inside `bash scripts/check.sh`.

The stable log reaches the generic local benchmark fixture after Rust tests/clippy and the earlier policy/resource checks, then fails with:

```text
jq: invalid JSON text passed to --argjson
```

The failure occurs before `scripts/check.sh` reaches the newly added owned-lab parser/control-plane tests at the end of the script. Therefore the current CI result does **not** independently attest the new control-plane behavior, even though the new code/test structure is materially closer to the required paid-run admission contract.

The generic `scripts/bench/compare-hy2.sh` and `scripts/bench/compare-hy2-test.sh` were not changed in the `2a13992..0265868` implementation delta, and the prior exact `2a13992` Rust CI was green. Reviewer-side reconstruction of the currently fetched generic scripts under a clean shell with `jq 1.7` also passes the good/bad fixture sequence. That does not clear the repository gate; it indicates the failure should be treated as a determinism/input-shape problem to isolate, not papered over by blindly rerunning CI.

## Review verdict

**needs repair — control-plane admission is substantially implemented, but exact-head stable CI is red in the generic benchmark JSON path; do not spend the paid VPS comparison window until the JSON contract is deterministic and exact repair HEAD CI is green**

The paid HY2/Nekomusume pair remains the highest-value VPS opportunity immediately after this repair. Do not insert unrelated local polish between a green admission gate and the VPS run.

This is a benchmark/evidence-harness blocker, not evidence of a transport/runtime regression. Do not change Nekomusume wire, Session, Noise, carrier or failover semantics to make this test green.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research implementation baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- `0265868` is benchmark/control-plane test infrastructure, not new transport capability evidence.
- Exact-head nightly decode fuzz smoke is independently green, but exact-head stable checks are red. The current tree therefore fails the required release/benchmark admission gate.
- The accepted exact `25e0daa` D064 controlled application-level UDP reply-cessation warm-fallback row remains bounded positive evidence: 3/3 records, 48 application bytes, two uncertain/replayed records, duplicate 0, lost 0, and approximately 434 ms failure-decision-to-first-resumed-data acceptance. It is not natural Internet degradation/PTO-blackhole proof.
- The accepted exact `25e0daa` approximately five-minute periodic direct-path row remains bounded positive evidence: 60 × 32-byte records, 60/60 confirmed, no missing/duplicate/conflict, with recorded process/resource and cleanup observations. It is one bounded sample, not a production reliability rate.
- Do not rerun either accepted row merely to consume VPS time.
- No complete HY2/Nekomusume paired sample set, comparative median/P95 or superiority evidence exists yet.
- Standing authorization already covers the intended self-owned bounded HY2/Nekomusume run, bounded capture/diagnostics and cleanup once the admission gate is green. No new per-run maintainer authorization is needed.
- IPv6 remains environment-blocked unless a genuinely owned end-to-end IPv6 path becomes available.
- `docs/status.md`, `IMPLEMENTATION_PLAN.md` and `ROADMAP.md` remain stale relative to the accepted D064/periodic positive evidence and evolving HY2 harness state. Reconcile them after the HY2 step closes honestly; preserve historical negative rows.

### R-HY2-13 / R-HY2-17 — implementation shape now acceptable, CI attestation pending

At current HEAD:

- one shared readiness primitive exists for both remote HY2 UDP and Nekomusume TCP listener admission;
- exact `ss -H -lun` / `ss -H -ltn` shapes are used;
- `parse-listener.py` is staged into the remote run set;
- direct parser fixtures are now listed in `scripts/check.sh`;
- the old duplicate inline readiness loops are removed from the production adapter.

Keep this structure unless the red gate exposes a concrete contradiction. Do not reintroduce multiple readiness implementations.

### R-HY2-16 — fake control-plane coverage is materially present, but exact-head stable CI did not reach it

`owned-lab-control-plane-test.sh` now exercises the important helper behavior with fake `date`, `ssh`, `timeout` and `sleep`, including positive remote TCP/UDP readiness and the required fail-closed cases. This is the right testing direction.

But because `scripts/check.sh` failed earlier in `compare-hy2-test.sh`, the current GitHub stable job never independently executed the new end-of-gate control-plane test. The next repair must restore the entire gate and then preserve this test green.

### R-HY2-18 — new blocker: generic benchmark stdout/JSON extraction can produce an invalid `--argjson` value under CI

The exact stable log fails with `jq: invalid JSON text passed to --argjson` during the generic `compare-hy2-test.sh` stage. `compare-hy2.sh` currently derives several values independently from adapter stdout and then feeds `wire` through `--argjson`:

```text
reported_hash=$(jq ... "$raw" || true)
app=$(jq ... "$raw" || true)
wire=$(jq -c '.wire_bytes // null' "$raw" || echo null)
fd=$(jq ... "$raw" || true)
...
jq -nc ... --argjson wire "$wire" ...
```

A particularly dangerous shape is a command that emits a valid JSON object plus unexpected stdout contamination or multiple JSON values: `jq` may produce output before returning non-zero, and the shell fallback can append another `null`, yielding a multi-value string that is invalid as one `--argjson` argument. Even if that is not the exact runner mechanism, the current gate proves that the adapter-output contract is not deterministic enough across environments.

The correct repair is to validate **one exact adapter result object** once, derive all typed fields from that validated object, and fail the sample truthfully if stdout is empty, malformed, contaminated or contains multiple JSON values. Do not turn malformed adapter output into a successful sample and do not suppress diagnostics by converting arbitrary text to `null`.

## Work Package — Deterministic JSON Gate Repair -> Full Admission CI -> Paid HY2 Pair -> Matrix Reconciliation

### Primary A — Make generic benchmark adapter output single-object, typed and fail-closed

**Goal**

Close R-HY2-18 and make `scripts/bench/compare-hy2-test.sh` deterministic on the exact GitHub runner path without weakening the benchmark contract.

**Likely files**

- `scripts/bench/compare-hy2.sh`;
- `scripts/bench/compare-hy2-test.sh`;
- optionally one small helper if a single JSON-object parser materially reduces shell ambiguity; do not create a parallel benchmark implementation.

**Required behavior**

1. Reproduce the exact failure locally where possible with `bash -x scripts/bench/compare-hy2-test.sh` and inspect the value passed to every `--argjson`. Record which value becomes invalid; do not guess in the final commit message.
2. Treat the benchmark command contract as: **stdout must contain exactly one JSON object and no unrelated stdout text**. Parse/validate that object once per sample before extracting application bytes, payload hash, FD count and nullable wire bytes.
3. Reject or mark the sample failed when stdout is:
   - empty;
   - malformed JSON;
   - one valid object plus trailing non-JSON diagnostics;
   - multiple JSON values/objects;
   - a JSON array/scalar instead of the required object;
   - missing or wrongly typed required fields.
4. Keep diagnostics on stderr separate from the one-object stdout contract. Do not silently discard contamination and then treat the sample as successful.
5. Normalize nullable `wire_bytes` as a single validated JSON value before passing it to any `--argjson`, or avoid shell `--argjson` transport for that field by constructing the sample from the already validated JSON object inside one `jq` program.
6. Preserve the exact payload-byte/hash contract and existing failure counting. A bad HY2 hash still yields a retained failed sample, not a shell/JSON-construction crash.
7. Preserve loopback-only safety in the generic harness; this repair does not authorize the generic helper to become a WAN runner.
8. Do not modify Nekomusume protocol/runtime semantics.

**Regression matrix**

Extend `compare-hy2-test.sh` so it proves at minimum:

- valid object with `wire_bytes:null` succeeds;
- valid object with an allowed non-null wire value, if that shape is supported by the current schema, is preserved exactly;
- wrong payload hash becomes a typed sample failure without crashing JSON construction;
- empty stdout is a typed failure;
- malformed JSON is a typed failure;
- valid JSON followed by stdout garbage is a typed failure;
- two JSON objects are a typed failure;
- array/scalar stdout is a typed failure;
- missing FD/application/hash field is a typed failure;
- repeated execution of the fixture produces a valid result deterministically.

Do not use `|| echo null` in a way that can concatenate a fallback value to partial `jq` output.

### Follow-up B — Re-run and harden the complete local benchmark/control-plane gate

**Dependency:** A green.

Run the benchmark/evidence tests in a diagnostic order that makes the failing layer obvious:

1. `bash scripts/bench/compare-hy2-test.sh`;
2. `python3 scripts/bench/parse-listener-test.py`;
3. `bash scripts/bench/owned-lab-control-plane-test.sh`;
4. `bash scripts/bench/compare-hy2-owned-lab-test.sh`;
5. `python3 scripts/bench/validate-hy2-owned-lab-test.py`;
6. process-resource sampler and echo-payload regressions;
7. `bash scripts/check.sh`;
8. `git diff --check`.

Keep the new shared readiness helper and fake-remote scenarios green. If one fails after the generic JSON repair, fix the specific helper/test contract before proceeding; do not remove the fail-closed scenario merely to make CI pass.

If shell test ordering hides the true failing test, add concise test-name markers or a small runner improvement so future CI logs identify which benchmark subtest failed. Avoid verbose production output.

### Follow-up C — Push repair and require exact-repair-HEAD CI green

**Dependency:** A-B complete.

Push coherent repair commits, then require the exact repair HEAD to have:

- `stable checks`: green;
- `nightly decode fuzz smoke`: green.

The current `0265868` nightly fuzz success may be retained as evidence for that exact commit, but it does not substitute for the future exact repair HEAD gate after benchmark scripts change.

Do not spend the paid VPS comparison window while stable CI is red or while the exact repair HEAD CI is pending after substantive benchmark-harness changes.

The coding agent is pre-authorized to proceed directly from a fully green exact repair HEAD to Follow-up D without waiting for another reviewer handoff.

### Follow-up D — Immediately run one fair, bounded HY2/Nekomusume paid-VPS pair

**Dependency:** C exact repair HEAD CI fully green.

This remains the highest-value READY rental-window task. Do not insert unrelated local work between the green gate and this attempt.

Use the established owned-lab contract:

```text
self-owned client <-> owned VPS only
pinned HY2 v2.9.3 exact SHA-256
5 paired runs if admission succeeds
1200-byte deterministic payload per sample
concurrency 1
fresh unprivileged ports
fresh transport client/session per timed sample
separate remote bind authority and client connect authority
pinned disposable experiment certificate
whole lab including cleanup reserve < 10 minutes
```

Preserve the production HY2 service/config and all production firewall/route/qdisc/DNS/proxy/tunnel state unchanged.

Because historical HY2 attempts failed during QUIC establishment, retain bounded changed-hypothesis diagnostics on the disposable experiment UDP port only:

- exact temporary server listener identity/bind proof;
- exact connect authority used by the client;
- client->VPS and VPS->client packet-direction evidence where bounded capture is used;
- redacted HY2/Nekomusume logs sufficient to distinguish network/path, TLS/auth and local harness failure;
- capture metadata/hash/packet counts/timestamps if captured;
- experiment-owned process/listener/temp-path cleanup verification.

If all required paired samples succeed:

- retain all raw samples;
- validate one complete `benchmark-result.v1`;
- calculate median/P95/failures only from the complete successful set;
- retain exact payload hash/application bytes and symmetric process-group client CPU/RSS/FD evidence;
- state conclusions only for this route/time window/batch; no superiority or production claim.

If any required pair fails:

- retain the valid prefix plus failed row in one typed blocked/diagnostic artifact;
- produce no comparative summary;
- preserve the changed-hypothesis diagnostics;
- do not repeat unchanged.

### Follow-up E — Reconcile the release matrix, then spend at most one further VPS opportunity on a genuinely READY missing row

**Dependency:** D closes positively or honestly blocked.

First repair evidence/status drift in:

- `docs/status.md`;
- `IMPLEMENTATION_PLAN.md`;
- `ROADMAP.md`.

Required reconciliation:

- record the accepted exact `25e0daa` D064 controlled application-fault warm-fallback result without relabeling it as natural network degradation;
- record the accepted approximately five-minute periodic direct-path result as one bounded sample, not general long-connection proof;
- record the exact HY2 D outcome only at its real commit/binary/path identity;
- preserve historical failed rows and negative evidence;
- keep bounded release evidence matrix item 3 unchecked while declared natural degradation, IPv6/environment, NAT/endpoint-change, HY2 or other required rows remain unresolved;
- keep `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`.

Then audit the remaining VPS backlog and choose **at most one** additional row that is both dependency-ready and truthfully observable at the current runtime:

1. genuine owned source-endpoint/path change if it can be produced without production network modification;
2. real-session migration-back if a live runtime/CLI seam already exists;
3. real-session key update if a live runtime/CLI seam already exists;
4. live-path PMTUD observation if the current instrumentation exposes truthful packetization evidence.

If no such live seam exists, record `BLOCKED_IMPLEMENTATION` for the candidate row and spend the next development slice on the smallest instrumentation/runtime seam that directly unlocks a high-value VPS row. Do not use fixture-only capability as WAN evidence.

## Fallback

If the generic `compare-hy2-test.sh` failure cannot be reproduced outside GitHub:

- do not blindly rerun CI as the only action;
- harden the one-object stdout parser and add the contamination/multiple-object regressions anyway, because the exact runner has already falsified the current assumption;
- push the deterministic repair and use the new exact-head CI as the adjudication point.

If the generic JSON gate becomes green but the new fake control-plane test fails:

- keep VPS execution blocked;
- repair the exact failed control-plane branch;
- retain all positive helper tests;
- rerun full exact-head CI.

If the paid HY2 run remains blocked by the real network/path after a green harness and changed-hypothesis diagnostics:

- preserve the negative result;
- do not modify provider/production network policy to force success;
- move to the next truthful READY VPS-only evidence row.

## Completion gates

This work package is complete only when:

- generic benchmark adapter stdout is deterministically one validated JSON object per sample;
- malformed/contaminated/multi-object output cannot crash `--argjson` construction or become a successful sample;
- generic compare fixture is green and deterministic;
- direct parser fixtures are green;
- fake owned-lab control-plane behavior tests are green;
- owned-lab safety/result/cleanup validators remain green;
- full `scripts/check.sh` is green locally;
- exact repair HEAD `stable checks` and nightly fuzz are green;
- only then is one standing-authorized paid HY2/Nekomusume paired attempt executed;
- the HY2 result is retained as complete comparative evidence or honest typed blocked evidence, never a partial superiority claim;
- status/plan/roadmap reflect accepted D064/periodic/HY2 evidence at exact identities;
- no production/network authorization boundary is widened.

## Do not expand into

- changing wire, Session, Noise, failover or carrier semantics to satisfy benchmark tooling;
- rerunning accepted D064/periodic rows merely to use VPS time;
- publishing HY2 superiority from a partial/failed/semantically unequal sample set;
- previous/current interoperability before a real prior frozen release exists;
- 0-RTT, enabled FEC, striping/aggregation or exotic carriers without an observed-problem gate;
- third-party targets, scanning, production firewall/route/DNS/proxy/tunnel/qdisc changes;
- experiments beyond standing authorization.

## Questions requiring maintainer decision

none.
