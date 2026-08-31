# Nekomusume ChatGPT Handoff

Checked at: 2026-09-01 05:02 Asia/Shanghai
Repository HEAD: `27d23cd45318e2539ac70384665d785405db27f1`
Previous checked implementation HEAD: `d486c823152300b62fcda6d035f822c521527b49`
Previous reviewer handoff commit: `2ffa4dc2f6d78949497962117d2b33a7b86968bd`

## What changed

One new coding-agent commit is visible since the previous reviewer handoff:

- `27d23cd` — **test/oracle-contract hardening; no production runtime or wire change.** The canonical Rust harness now defines operation-specific successful `expected.value` key contracts for negotiation client hello / server response, record decode, frame/close decode, and varint decode. Unknown successful keys and missing required keys are rejected. Record/frame semantic assertion helpers are factored out, and mutation tests exercise stale record type/flags/payload plus stale frame payload/payload-byte expectations against decoded implementation values.

This materially closes the main future-drift problem identified in the previous handoff: a new successful semantic declaration can no longer be silently ignored by the main executable corpus test.

One requested regression proof is still incomplete: the negotiation selected-version mutation at the end of `semantic_mutations_reach_real_oracle_assertions_without_changing_bytes` computes a real selected version, mutates the expected selected value, and then uses only `assert_ne!`. It does **not** actually invoke the same `assert_negotiation_selected` executable-oracle assertion and prove that the stale semantic is rejected by that path. The production/main corpus oracle does call `assert_negotiation_selected`; the defect is specifically in the mutation proof, not in the current production negotiation implementation.

The deterministic executable coverage audit requested by the N9 package is still absent from the visible repository state, and there is no GitHub commit-status/CI attestation for current HEAD. Therefore N9 is not yet ready for a freeze decision.

## Review verdict

**continue with required N9 closure — expected-value contract accepted, one mutation-proof repair plus coverage/full-gate remain**

Accept the operation-specific expected-value contract in `27d23cd` as a real improvement. Do not roll it back or redesign production APIs.

Keep `freeze=false`. Finish the remaining N9 closure work in one batch rather than returning after the tiny negotiation mutation-test repair.

The project is not globally blocked. Once N9 is truthfully reviewable, the next real queue remains authenticated negotiation-path completion and then the time-limited VPS evidence program.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research implementation status.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- `27d23cd` changes only `crates/neko-wire/tests/canonical_vectors.rs`; it does not change production wire/runtime semantics.
- Exact successful expected-key contracts are now enforced by the main Rust corpus test for all currently successful executable operation families.
- Record and frame semantic mutations now reach real decoded implementation values and fail the extracted semantic assertion helpers.
- The selected-version mutation test does not yet prove rejection through `assert_negotiation_selected`; `assert_ne!` only restates that the mutated value differs.
- The current frame adapter still repeats the `payload_bytes` assertion after `assert_frames_semantics` already performs it. This is redundant but not a release blocker; clean it only if it simplifies the same-file repair without obscuring the diff.
- Expected-failure rows still consume fixture bytes and compare implementation error classes; state-only rows remain non-wire.
- No deterministic checked-in/generated coverage artifact currently maps vectors/operations to exact adapter paths and asserted semantics/errors.
- No GitHub CI/check status is attached to current HEAD. Any local gate report remains coding-environment evidence, not independent review.
- Standing self-owned VPS authorization remains active. The VPS is a one-month time-limited evidence asset; after N9/negotiation dependencies permit, VPS-only evidence should outrank unrelated local polish.

## Work Package — complete N9 and prepare the VPS unlock path

### Primary — Repair the negotiation semantic mutation proof

**Goal**

Close the last known gap in the N9 expected-semantic regression proof without changing production APIs, wire bytes, or candidate semantics.

**Required behavior**

1. In `semantic_mutations_reach_real_oracle_assertions_without_changing_bytes`, mutate a successful negotiation `selected` expectation while keeping fixture bytes unchanged.
2. Feed the real implementation-derived selected value plus the stale expected object into the **same semantic assertion path used by the main corpus oracle** (`assert_negotiation_selected` or an equivalent single shared helper).
3. Assert that this invocation fails deterministically, e.g. with `catch_unwind`, just as the record/frame semantic mutations exercise their real assertion helpers.
4. Do not satisfy this requirement with `assert_ne!`, direct comparison duplicated in the test, corpus SHA failure, or a new production API.
5. If useful in the same small diff, remove the duplicate post-helper frame `payload_bytes` assertion so there is one authoritative frame semantic path; this cleanup is optional and must not change behavior.

**Completion definition**

A stale negotiation selected-version semantic is demonstrably rejected by the same executable semantic oracle used by the corpus adapter.

### Follow-up 1 — Generate and mechanically gate the executable corpus coverage audit

**Dependency:** Primary green.

Finish the still-missing N9 review artifact. Prefer a deterministic generator/checker over a manually maintained prose matrix.

For every executable vector, or for every operation plus explicit exceptional rows where the mapping is provably unambiguous, expose at least:

- vector ID;
- domain / operation;
- classification;
- `bytes_hex` present vs state-only;
- encode/decode/roundtrip oracle bits;
- real implementation function/path exercised (`VersionNegotiator::*`, `decode`, `decode_frames`, `decode_varint`, etc.);
- exact successful `expected.value` fields asserted, or expected error class asserted;
- state-only/non-wire status.

A suitable implementation is a small deterministic generator/checker plus a checked-in `docs/spec/canonical-vector-coverage.md` (or an equivalent existing release-evidence location). If a generated artifact is checked in, `scripts/check.sh` must fail when it drifts from the fixture/adapter contract.

The coverage artifact is **review/navigation evidence**, not a second normative wire specification. Do not copy protocol prose into it unnecessarily.

Add focused drift tests such as:

- executable vector added without a coverage mapping;
- operation renamed without adapter-path mapping;
- expected-field contract changed without regenerated coverage;
- state-only vector accidentally presented as wire-executable.

**Completion definition**

The next reviewer can answer “what exactly would N9 freeze, and what implementation assertion proves each row?” without reverse-engineering the full Rust harness, and repository gates detect mapping drift.

### Follow-up 2 — Full unfrozen N9 release-gate rehearsal

**Dependency:** Primary + Follow-up 1 green.

Run and record the complete candidate gate while preserving `freeze=false`:

- targeted canonical-vector Rust integration tests, including expected-contract and semantic-mutation regressions;
- `python3 scripts/validate-canonical-vectors.py fixtures/canonical-vectors.v1.json`;
- canonical validator mutation tests;
- new coverage-generation/drift checks;
- `cargo fmt --all -- --check`;
- `cargo check --workspace --locked`;
- `cargo test --workspace --all-targets --locked --no-fail-fast`;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- `bash scripts/check.sh`;
- `git diff --check`.

Run fuzz smoke only if production parser/decoder behavior changes or the normal repository gate requires it. A test/generator-only N9 repair must not manufacture a parser-fuzz claim.

Record enough evidence in commit text or the existing release-evidence location to distinguish targeted canonical tests from the full workspace gate.

After this follow-up is green, push the N9 closure commits and stop changing candidate bytes unless a newly discovered correctness defect requires it. The coding agent must not set `freeze=true`; the next reviewer owns the independent freeze decision.

### Follow-up 3 — VPS rental-window unlock slice: comparison/runtime instrumentation audit, then implement the smallest reusable gap

**Dependency:** N9 closure commits pushed and full local gate green. This is useful stretch work if N9 finishes early; do not delay pushing N9 closure to bundle unrelated changes.

The VPS is time-limited, and the repository already has a pinned HY2 setup plus `docs/bench/hy2-comparison-workload.md`. Use remaining capacity to move the next VPS-only evidence run closer rather than doing unrelated polish.

1. Audit current `neko` CLI/runtime and `scripts/bench/` against the comparison contract. Determine exactly what is still missing for a Nekomusume command that:
   - consumes the deterministic `BENCH_PAYLOAD_FILE` contract or an equivalent exact byte/hash input;
   - performs one bounded authenticated application-semantic exchange;
   - emits valid integer `application_bytes` and `fd_count` plus finite clean exit;
   - can be measured by existing GNU `time` CPU/RSS wrappers;
   - does not claim `wire_bytes` unless capture metadata is trustworthy.
2. Reuse existing code first. Do not create a second benchmark framework if the current CLI/probe can be wrapped truthfully.
3. If the missing piece is a **small, protocol-neutral adapter/instrumentation slice** that does not alter candidate wire semantics or cross the N9 freeze boundary, implement it with loopback tests and `scripts/check.sh` coverage in a separate commit.
4. If the required change would alter negotiation/wire/session semantics, do not implement it before the N9 reviewer decision. Instead record the exact dependency so the next handoff can make it the first post-N9 negotiation-path slice.
5. Also inventory which standing-authorized VPS rows become immediately runnable after negotiation-path completion: repeated real-socket lifecycle; bounded steady/idle runs; UDP degradation→TCP fallback; recovery/migration-back; key update; PMTUD observation; actual IPv4/IPv6 rows; CPU/RSS/FD/socket sampling; package lifecycle.

This follow-up is about reducing rental-window setup latency. Do not execute HY2/Nekomusume comparative WAN runs yet unless the current reviewed semantics are genuinely equivalent and the relevant negotiation path is already ready; a plan or local adapter is not performance evidence.

## Fallback

If the mutation/coverage work exposes a real production codec/parser/negotiation mismatch rather than an evidence-test defect:

1. keep `freeze=false`;
2. preserve a minimal fixture-backed reproducer;
3. repair production correctness first;
4. run parser/fuzz gates required by `AGENTS.md`;
5. rerun the entire N9 closure gate before requesting freeze review.

If the full workspace gate fails for an unrelated pre-existing defect, preserve the exact failure, determine whether it predates this slice, and do not weaken tests or validators to obtain green output.

If the VPS-unlock audit finds the comparison impossible without changing candidate semantics, stop at the dependency map; do not force a benchmark adapter around unequal application behavior.

## Completion gates

N9 is ready for the next reviewer freeze decision only when all are true:

- operation-specific successful expected-field contracts from `27d23cd` remain intact;
- unknown/missing successful fields fail deterministically;
- stale record/frame semantics fail through real executable semantic assertion paths;
- stale negotiation selected-version semantics also fail through the shared real oracle assertion path, not merely `assert_ne!`;
- failure rows still prove actual implementation errors from fixture bytes;
- state-only rows remain unmistakably non-wire;
- deterministic executable coverage audit exists and is mechanically kept in sync;
- full local workspace/repository gate passes;
- standing VPS authorization wording remains aligned;
- `FREEZE=false`, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` remain unchanged.

## Next reviewer action after completion

Independently review the exact pushed N9 candidate. If no concrete corpus/evidence-contract defect remains, perform or authorize the separate N9 governance freeze change. Then move immediately to authenticated negotiation-path completion and the highest-value standing-authorized VPS evidence queue. The VPS rental window should not be consumed by unrelated documentation polish once those dependencies are green.

## Do not expand into

- changing protocol bytes merely to satisfy fixture aesthetics;
- self-setting `freeze=true`;
- RC/production/security approval;
- previous/current interoperability before a real prior frozen release exists;
- speculative 0-RTT/FEC/striping/exotic carriers;
- third-party targets, scanning, production network changes, or experiments outside standing authorization;
- HY2 superiority claims from one-off, non-equivalent, or measurement-contaminated workloads.

## Questions requiring maintainer decision

none.
