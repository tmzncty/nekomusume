# Nekomusume ChatGPT Handoff

Checked at: 2026-09-01 04:01 Asia/Shanghai
Repository HEAD: `d486c823152300b62fcda6d035f822c521527b49`
Previous checked HEAD: `905c21e18e8632197b04858dc735030aff166961`
Previous reviewed implementation HEAD: `1d4905eea2137290d768564728ec1c6181f43ddc`

## What changed

Two coding-agent commits are new since the previous reviewer package:

- `40f6ed4` — **test/fixture semantic-oracle repair; no production runtime change.** Successful negotiation hello vectors now enforce the declared offered-version list and selected version against real `VersionNegotiator` behavior and bytes. Successful frame/close vectors now compare the complete decoded `Frame` sequence, retain an explicit frame-count assertion, and enforce declared payload-byte totals. The canonical fixture was updated so successful frame vectors declare the frame semantics that the executable oracle now proves. `freeze=false` remains unchanged.
- `d486c82` — **documentation/governance wording repair only.** `ROADMAP.md` and `docs/status.md` no longer incorrectly treat ordinary bounded self-owned VPS TCP/UDP execution as awaiting new per-run authorization. They preserve the distinction between execution authorization and missing public/general reachability, release, security, environment and production evidence.

The previous N9 closure package therefore made real progress: Primary A and Follow-up D are materially satisfied. The project is not blocked, but N9 is not yet ready for freeze review because the future-drift guard, reviewable coverage map and full local gate rehearsal requested by the prior package are still absent from the visible GitHub state.

No GitHub commit-status/CI checks are attached to current HEAD. Coding-environment local gates may be valid evidence when recorded, but they are not independent CI attestation.

## Review verdict

**continue with required N9 closure work — semantic repair accepted; freeze still blocked by evidence-contract completeness**

Accept `40f6ed4` as closing the previously identified successful-decode semantic gap for the currently declared fields. Accept `d486c82` as closing the standing-authorization wording drift without promoting capability status.

Do **not** change production wire semantics or candidate bytes merely for coverage aesthetics. Do **not** set `freeze=true` yet.

The remaining N9 work is now concentrated in three dependency-ordered tasks: (1) prevent future successful expected fields from being silently ignored, (2) make corpus coverage mechanically reviewable, and (3) run/record the complete unfrozen local release-gate rehearsal. Complete all three in one coding-agent batch before waiting for the next reviewer.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research implementation status.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain correct.
- `40f6ed4` changes executable tests/fixtures, not production codec/runtime behavior.
- The current Rust harness now checks complete frame identity/payload semantics and declared payload-byte totals rather than frame count alone.
- Negotiation hello's declared offered versions are now enforced by reconstructing the hello via the real `VersionNegotiator` and comparing bytes, while selected version is checked from server behavior. This is acceptable evidence for the current public API; no new production decode API is required merely for the fixture.
- The current harness still uses dynamic `serde_json::Value` expected objects and does not visibly enforce an exact allowed/required successful `expected.value` key set. A newly added successful expected key could therefore still be decorative unless the next repair prevents it.
- The Python structural validator checks root/vector shape, corpus identity, required domains and oracle prerequisites, but it does not currently define operation-specific successful expected-value contracts.
- The repository still lacks a checked-in/generated review artifact that maps corpus operations/vectors to exact implementation adapter paths and asserted successful semantic fields/errors.
- `d486c82` correctly says bounded self-owned VPS execution is authorized while broader reachability/release evidence remains missing. Authorization is not a capability PASS.
- `docs/vps-rental-window-priority.md` remains active scheduling policy. The rented VPS should be used aggressively for high-value evidence once N9/negotiation dependencies permit, without widening standing authorization.

## Work Package — finish N9 in one batch

### Primary — B. Enforce operation-specific successful expected-value contracts

**Goal**

Make it impossible for a future successful canonical vector to add, remove or rename a semantic field in `expected.value` while the executable test silently ignores it.

**Likely files**

- `crates/neko-wire/tests/canonical_vectors.rs`;
- optionally a small helper/module local to the test if that keeps the contract readable;
- `scripts/validate-canonical-vectors.py` / its mutation test only if structural enforcement belongs there;
- fixture/schema only when required by a truthful contract, not for cosmetic rewrites.

**Required behavior**

1. Define an exact successful expected-value contract for every executable operation currently handled by the Rust corpus adapter. Prefer typed deserialization or an explicit required/optional-key assertion over ad-hoc `Value` indexing.
2. At minimum cover:
   - `negotiation/client_hello`;
   - `negotiation/server_response` success;
   - `wire/record` success;
   - `frame|close/frames` success;
   - `wire/varint` success.
3. Unknown successful expected keys must fail. Missing required keys must fail. Optional keys must be explicit; currently `payload_bytes` may remain optional for frame semantics only if that is an intentional contract.
4. Expected-failure rows remain error-oracle rows and must not be forced into successful-value schemas.
5. State-only rows remain non-wire/non-executable.
6. Do not change production APIs or wire bytes solely to make this validation convenient.

**Regression/mutation proof**

Add tests demonstrating at least:

- unknown successful expected key rejected;
- required successful expected key missing rejected;
- stale selected version rejected by the real oracle;
- stale record type/flags/payload rejected by the real oracle;
- stale frame identity/payload or `payload_bytes` rejected by the real oracle.

The last three may use a refactored per-vector oracle helper with in-memory mutated vectors, or another deterministic method that actually exercises the same semantic assertion path. Do not claim a semantic mutation test if it only fails corpus SHA validation before reaching the executable oracle.

**Completion definition**

A future corpus editor cannot introduce a new successful semantic declaration that passes because the Rust adapter never reads it.

### Follow-up 1 — C. Generate and gate an executable corpus coverage audit

**Dependency:** Primary green.

Create a deterministic review artifact, preferably generated rather than manually maintained, that lets a reviewer see exactly what the candidate corpus freezes.

For every executable vector, or for every operation plus explicit exceptional vector rows where that is unambiguous, expose:

- vector ID;
- domain / operation;
- classification;
- wire bytes present vs state-only;
- encode/decode/roundtrip oracle bits;
- real implementation function/path exercised (for example `VersionNegotiator::*`, `decode`, `decode_frames`, `decode_varint`);
- exact successful expected fields asserted, or expected error class asserted;
- non-wire/state-only classification.

A practical shape is a small generator/checker plus checked-in `docs/spec/canonical-vector-coverage.md`. If a generated artifact is checked in, `scripts/check.sh` must fail when it drifts from the fixture/adapter contract.

Do not create a second normative protocol specification. This is release-review evidence/navigation only.

**Completion definition**

The next reviewer can answer “what is executable and what semantics/errors are asserted?” without reverse-engineering the full Rust harness, and drift is mechanically detectable.

### Follow-up 2 — E. Full N9 unfrozen release-gate rehearsal

**Dependency:** Primary + Follow-up 1 green.

Run and record at minimum:

- targeted canonical-vector Rust integration test(s);
- `python3 scripts/validate-canonical-vectors.py fixtures/canonical-vectors.v1.json`;
- canonical validator mutation tests;
- new expected-key/semantic-mutation/coverage drift tests;
- `cargo fmt --all -- --check`;
- `cargo check --workspace --locked`;
- `cargo test --workspace --all-targets --locked --no-fail-fast`;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- `bash scripts/check.sh`;
- `git diff --check`.

Run fuzz smoke only if production parser/decoder behavior changes or the repository gate requires it. Pure test/validator/coverage work does not need a fake fuzz claim.

Record enough local evidence in commit text or the repository's ordinary evidence location to distinguish targeted canonical tests from the full workspace gate. Preserve `freeze=false`.

### Follow-up 3 — VPS next-stage preflight, no WAN run yet

**Dependency:** N9 closure commits pushed and all local gates green. This is stretch work only if the above finishes early; do not delay the N9 closure commits for it.

Use the remaining time to prepare the highest-value next VPS work without crossing the N9 freeze decision or altering candidate bytes:

1. Audit the exact gap between current Nekomusume CLI/runtime commands and the existing pinned HY2 comparison contract (`docs/bench/hy2-comparison-workload.md`, pinned HY2 v2.9.3 setup).
2. Identify the smallest implementation/instrumentation slice required for an **equivalent application-semantic Nekomusume benchmark command** that can emit application bytes, FD count, CPU/RSS via the existing harness, finite timeout and clean exit.
3. Audit which release-evidence rows are already runnable under standing authorization immediately after negotiation-path completion: repeated real-socket lifecycle, bounded 5-10 minute resilience scenarios, TCP/UDP sessions, fallback/recovery, key update, PMTUD, IPv4/IPv6 as actually available, package lifecycle.
4. Produce a concise implementation/evidence plan in an existing non-normative release/bench note if useful; do not execute comparative WAN samples before the relevant negotiation/correctness path is reviewed.

This is preparation for the time-limited VPS window, not a new capability claim.

### Fallback

If Primary or coverage work exposes a real production codec/parser mismatch rather than only evidence-contract drift:

1. keep `freeze=false`;
2. preserve a minimal fixture-backed reproducer;
3. make the production correctness repair the immediate Primary;
4. run parser/fuzz gates required by `AGENTS.md`;
5. rerun the entire N9 closure gate before requesting freeze review.

If the full workspace gate fails for an unrelated existing defect, preserve exact failing evidence, determine whether it predates this slice, and do not hide it by weakening checks.

## Completion gates

N9 is ready for the next reviewer freeze decision only when all are true:

- current successful semantic assertions from `40f6ed4` remain intact;
- each executable successful operation has an exact expected-value field contract;
- unknown/missing successful fields fail deterministically;
- semantic mutation tests reach the real executable oracle rather than only corpus-hash rejection;
- failure rows still prove actual implementation errors from fixture bytes;
- state-only rows remain unmistakably non-wire;
- deterministic executable coverage audit exists and is mechanically kept in sync;
- full local workspace/repository gate passes;
- standing VPS authorization wording remains aligned;
- `FREEZE=false`, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `RELEASED=false` remain unchanged.

## Next reviewer action after completion

Review the exact pushed N9 candidate. If no concrete corpus/evidence-contract defect remains, perform or authorize the separate N9 governance freeze change. Then move immediately to authenticated negotiation-path completion and the VPS rental-window evidence queue; do not spend the rental window on unrelated polish.

## Do not expand into

- changing protocol bytes merely to satisfy fixture aesthetics;
- self-setting `freeze=true`;
- RC/production/security approval;
- previous/current interoperability before a real prior frozen release exists;
- speculative 0-RTT/FEC/striping/exotic carriers;
- third-party targets, scanning, production-network changes or experiments outside standing authorization;
- HY2 superiority claims from one-off or non-equivalent workloads.

## Questions requiring maintainer decision

none.
