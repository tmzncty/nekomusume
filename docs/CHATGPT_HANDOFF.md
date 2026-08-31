# Nekomusume ChatGPT Handoff

Checked at: 2026-09-01 06:57 Asia/Shanghai
Repository HEAD: `8726623cf375ab3ef478e6af4993e20bff2383e9`
Previous reviewed implementation HEAD: `29a5fbc28e5ad35acb5600b2d3810c4bcf130cba`
Previous reviewer handoff commit: `8187182b...`

## What changed

New coding-agent / CI-relevant work is visible since the previous review:

- `8726623` — **test/toolchain/CI hardening only; no production runtime or wire change.** It pins the fuzz path to a rustup nightly cargo-fuzz toolchain, adds a self-test for that toolchain wrapper, and runs `scripts/fuzz-smoke.sh` in GitHub Actions.
- The GitHub Actions run associated with `8726623` completed successfully. Both the stable repository checks and the fuzz-smoke job passed. This closes the prior independent-CI/fuzz-attestation gap for the current candidate.

The reviewer also re-read the current canonical coverage generator and its mutation tests. A finding in the previous handoff was stale/incorrect: the generated review mapping is **not** a coarse one-string operation mapping anymore. `scripts/generate-canonical-review.py` has oracle-specific encode/decode/roundtrip paths, and `scripts/generate-canonical-review-test.py` explicitly rejects missing enabled paths, mislabeled negotiation paths, and the legacy coarse mapping. The generated `docs/spec/canonical-vector-review.v1.md` exposes those oracle-specific paths. Therefore the previous “coarse mapping still blocks N9” claim is withdrawn.

The accepted N9 closure evidence now includes:

- exact successful `expected.value` contracts for every currently successful executable operation family;
- full frame/close decoded identity and payload semantics plus payload-byte assertions;
- record and negotiation semantic assertions;
- real semantic-mutation regressions, including selected-version rejection through the shared negotiation assertion path;
- content-addressed corpus identity and fixed required-domain validation;
- deterministic generated review coverage with oracle-specific implementation paths;
- local full-gate rehearsal at `29a5fbc`;
- independent GitHub Actions stable + fuzz success at current HEAD `8726623`.

No new production behavior was introduced by the latest fuzz-toolchain commit.

## Review verdict

**PASS for N9 candidate-corpus review at this exact candidate; authorize the separate corpus-specific freeze transition.**

The reviewer finds no remaining concrete defect in the current 42-vector canonical corpus evidence contract that requires keeping the corpus candidate mutable. The coding agent may now perform a narrow, mechanical N9 **canonical-corpus freeze** transition.

This authorization is intentionally narrow:

```text
canonical corpus frozen
!= whole protocol frozen
!= RELEASE_CANDIDATE
!= security approval
!= production readiness
!= release
```

The repository-wide governance facts must remain truthful. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` must remain unchanged. The existing global `FREEZE=false` must also remain unchanged **unless** repository governance already defines that flag specifically as the canonical-corpus freeze; current status text describes it as protocol/release freeze, so do not silently flip it. Record the N9 result as a corpus-specific freeze fact instead.

After the mechanical N9 transition is green and pushed, do not wait for another reviewer turn. Continue directly into authenticated negotiation-path completion and, if that becomes green in the same work window, into the first high-value VPS evidence run under standing authorization.

## Evidence boundaries

- Current canonical corpus: 42 vectors across 10 required domains.
- Current candidate corpus identity before the freeze transition: `84a49313974341b93d17415d2fffec2b2d0a68fb009f2a6b32381ad65ba93184`.
- `docs/spec/canonical-vector-corpus-scope.md` explicitly says this corpus covers public `neko-wire` record/frame/negotiation bytes and excludes cryptographic ciphertext, Noise messages, carrier packetization, failover/resume state, and previous-release interoperability. Preserve that scope.
- The generated coverage artifact is review/navigation evidence, not a second normative protocol specification.
- The N9 freeze decision does not validate generic CLI probe, UDP, or failover/resume version negotiation. `IMPLEMENTATION_PLAN.md` correctly lists those as the next separate implementation item.
- TCP multistream already performs explicit `VersionNegotiator` exchange, binds the authenticated negotiation transcript into Noise, and gates data admission. Generic TCP/UDP probe paths and failover/resume paths do not yet have equivalent integration and must not inherit the multistream claim.
- Standing self-owned VPS authorization is active. Ordinary bounded TCP/UDP listeners, diagnostic runs, failover/migration experiments, captures, cleanup, and HY2 comparison within its limits require no new per-run approval.
- The rented VPS remains a time-limited evidence asset. Once negotiation dependencies are green, real-socket/VPS evidence outranks unrelated local polish.

## Work Package — N9 freeze -> negotiation integration -> first VPS evidence

This package is deliberately thick. Execute A -> B -> C -> D in dependency order without returning after a ten-minute subtask. If a later stage exposes a real correctness/security defect, stop that path and use the fallback.

### Primary A — Perform the mechanical N9 canonical-corpus freeze transition

**Goal**

Freeze exactly the currently reviewed 42-vector canonical corpus and its existing public `neko-wire` bytes/semantics, without expanding the frozen scope and without promoting RC/security/production/global protocol-freeze state.

**Likely files**

- `fixtures/canonical-vectors.v1.json`;
- `schema/canonical-vector.v1.json`;
- `scripts/validate-canonical-vectors.py`;
- `scripts/validate-canonical-vectors-test.py`;
- `scripts/generate-canonical-review.py` and its tests only as required to represent/check the frozen state;
- `docs/spec/canonical-vector-corpus-scope.md`;
- `docs/spec/canonical-vectors-v1.md`;
- generated `docs/spec/canonical-vector-review.v1.md`;
- `IMPLEMENTATION_PLAN.md`;
- `docs/status.md` only for a precise corpus-specific status note if needed by repository status governance;
- existing check scripts if they currently hard-code the candidate/unfrozen state.

**Required behavior**

1. Change the canonical corpus from candidate `freeze=false` to the reviewed frozen state without modifying any vector byte or semantic field except metadata mechanically required by the freeze transition.
2. Recompute `corpus_sha256` using the existing deterministic identity algorithm after the freeze bit changes.
3. Update schema/validator/generator/tests so the repository now rejects an unexpected return to `freeze=false` for this v1 frozen corpus and continues rejecting stale content identity, missing required domains, unmapped executable oracles, and semantic contract drift.
4. Preserve the current 42-vector / 10-domain coverage and oracle-specific adapter mapping.
5. Mark N9 complete in `IMPLEMENTATION_PLAN.md` only after all freeze-transition gates pass.
6. Record a **corpus-specific** frozen fact. Do not turn it into a claim that Noise/ciphertext/carrier/failover/resume/global Session protocol is frozen.
7. Keep `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `RELEASED=false`, and the current global protocol/release `FREEZE=false` unless an existing authoritative governance checker proves that `FREEZE` has a narrower meaning. If a checker currently conflates them, repair the checker/status wording rather than silently escalating governance.

**Validation**

At minimum:

- targeted canonical-vector Rust tests;
- canonical validator and mutation tests;
- generated-review `--check` and generator mutation tests;
- `cargo fmt --all -- --check`;
- `cargo check --workspace --locked`;
- `cargo test --workspace --all-targets --locked --no-fail-fast`;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- `bash scripts/check.sh`;
- `git diff --check`.

No additional fuzz run is required solely because a reviewed fixture/governance freeze bit changed and production parser code did not. If parser/wire implementation code changes unexpectedly, run the normal fuzz gate and treat that as a correctness repair, not a mechanical freeze.

**Completion definition**

The exact current corpus is mechanically frozen and content-addressed; drift back to candidate state or semantic/byte mutation is caught; N9 is checked complete; all broader release/security/global-freeze flags remain truthful.

### Follow-up B — Integrate authenticated version negotiation into generic TCP and UDP probe paths

**Dependency:** A green and pushed.

**Goal**

Make the ordinary `neko server` / `neko client` TCP and UDP research probe paths perform the same fail-closed N1 negotiation discipline already proven in TCP multistream: negotiate before Noise data admission, bind the exact negotiation transcript into the authenticated handshake, and reject incompatible/malformed negotiation before application data.

**Why now**

This is the next explicit `IMPLEMENTATION_PLAN.md` item and directly unlocks high-value VPS evidence while the rental window is active.

**Required behavior**

1. Reuse/refactor the existing `VersionNegotiator` contract rather than inventing a second version protocol.
2. TCP probe client/server:
   - client sends canonical hello;
   - server selects a supported version or rejects;
   - client validates response;
   - both obtain the authenticated negotiation binding;
   - feed that binding into the Noise handshake/prologue binding;
   - call `admit_data` only after negotiation + authentication complete.
3. UDP probe client/server: same semantic sequence, with bounded datagrams and existing anti-amplification/resource constraints preserved.
4. No silent downgrade. Unsupported-only, malformed, duplicate/late negotiation and transcript mismatch must fail before Session/application data admission.
5. Do not change frozen N1 canonical bytes. If implementation cannot integrate them without changing bytes, stop and record the concrete incompatibility as a freeze/correctness blocker.
6. Preserve existing CLI bounds, identity/trust policy, lifecycle readiness, diagnostics, and secret-safe behavior.

**Tests**

Add bounded executable tests for both TCP and UDP covering at least:

- current/current success;
- unsupported-only peer rejection;
- malformed negotiation rejection;
- selected-version/transcript mismatch rejection before data;
- no successful application echo before negotiation admission;
- existing lifecycle/readiness and cleanup behavior remains intact.

Run fuzz only if public parser/decoder implementation changes.

### Follow-up C — Bind negotiation into failover/resume without weakening Session semantics

**Dependency:** B green.

**Goal**

Complete authenticated version negotiation for the failover/resume path so a logical Session cannot resume across an incompatible or unauthenticated version transition.

**Required behavior**

1. Initial UDP failover path and fallback TCP resume path must each use the same canonical version-negotiation primitive.
2. Bind negotiation identity/selected version to the authenticated Noise/resume context; a resume for an incompatible negotiated version must fail closed.
3. Preserve Session delivery semantics: confirmed/uncertain ranges and dedup remain Session-level evidence, not negotiation evidence.
4. Reject downgrade/transcript mismatch/replayed or duplicate negotiation attempts before resumed application data is accepted.
5. Keep anti-amplification/pre-auth bounds and existing ResumeGuard/resource limits intact.
6. Do not add UDP+TCP striping or concurrent application-data aggregation.

**Tests**

At minimum:

- negotiated current/current UDP->TCP resume success in controlled tests;
- unsupported future/current mismatch rejection;
- negotiation transcript mismatch rejection;
- resume-binding/version mismatch rejection;
- replay/duplicate negotiation behavior remains bounded/fail-closed;
- uncertain resend/dedup tests still prove exactly-once logical delivery where currently claimed.

### Follow-up D — Use the rented VPS immediately: negotiated real-socket sanity + bounded failover evidence

**Dependency:** B and C green, full local gates green, binary identity recorded.

Do not wait for another reviewer if all dependencies are met. Use `docs/standing-vps-lab-authorization.md` directly.

**Lab batch goal**

Produce the first post-negotiation real-socket evidence that is difficult to reconstruct after the VPS rental ends.

**Run, in one cleanup-safe bounded lab session where practical:**

1. negotiated authenticated TCP probe sanity on self-owned client <-> VPS;
2. negotiated authenticated UDP probe sanity;
3. negotiated UDP-primary -> TCP fallback/resume using the real failover path, with the implementation-supported bounded degradation/blackhole mechanism;
4. verify unsupported/mismatched version cannot reach application data on at least one real socket path if this can be done without expanding exposure;
5. collect experiment id, git/binary identity, exact params, start/end timestamps, client/server results, relevant structured diagnostics, process/socket/resource metadata when available, and cleanup state;
6. verify no unintended listener/process remains.

Stay inside standing limits: single run/batch semantics must not be used to evade the 10-minute, 256 MiB, or 32-session bounds. Do not modify production firewall/route/qdisc to induce loss. If the real failover degradation requires such a host-level network modification and no application-level/self-contained failure injection exists, mark only that row `BLOCKED_ENVIRONMENT/INSTRUMENTATION` and still execute the negotiated TCP/UDP sanity rows that are READY.

**Evidence boundary**

These are self-owned endpoint observations. Do not promote them to general Internet reachability, sustained production reliability, security approval, or performance superiority.

### Follow-up E — Rental-window unlock stretch: equivalent Nekomusume comparison command / resource sampler

**Dependency:** A-C green; do this only if D is blocked or finishes with meaningful time remaining.

Audit the existing pinned HY2 v2.9.3 comparison contract and current Nekomusume CLI. Implement the smallest reusable missing seam that directly unlocks a fair paired comparison or richer VPS evidence, preferring:

1. an exact Nekomusume application-exchange command satisfying the existing comparison workload contract (`BENCH_PAYLOAD_FILE`, exact application byte count/hash/target/timeout, JSON `application_bytes` and `fd_count`); or
2. a process-scoped CPU/RSS/FD/socket sampler usable by both Nekomusume and HY2 runs.

Do not run a comparison until the commands have equivalent application semantics and metadata. Do not alter HY2 production config or credentials; use the already documented temporary isolated setup.

## Completion gates

This work package is successful when as many dependency-satisfied stages as possible are completed in order, with no deliberate idle gap between tiny substeps:

- N9 corpus is mechanically frozen at the exact reviewed semantics/bytes, with a new valid content identity;
- N9 is marked complete without promoting RC/security/production/global protocol-freeze claims;
- generic TCP and UDP probes negotiate/authenticate before data;
- failover/resume is version-bound and fail-closed;
- if environment permits, at least negotiated TCP/UDP real-socket VPS evidence is collected and cleaned up, and negotiated real failover is attempted only with an authorized/self-contained failure mechanism;
- all changed code has the required local gates; GitHub CI should be allowed to attest the pushed state;
- negative results are retained rather than massaged into PASS.

## Fallback

If A reveals that freezing the corpus conflicts with current repository-global `FREEZE` checkers, do not flip global release/protocol state. Repair the governance model so corpus freeze is independently expressible, then continue A.

If B/C reveals that the frozen negotiation bytes cannot be integrated into the real probe/failover paths without a wire change, stop those paths, preserve a minimal reproducer, keep broader release state false, and record this as a concrete N9/negotiation compatibility defect requiring reviewer attention. Do not silently mutate frozen bytes.

If D is blocked only by missing IPv6, NAT-change capability, or an authorized failure-injection mechanism, do not block the whole project. Record the exact environment/instrumentation blocker and continue another READY VPS-value task such as TCP/UDP negotiated sanity, repeated real-socket lifecycle, package/operator evidence, comparison-command instrumentation, or resource sampling.

## Do not expand into

- RC declaration, security approval, production readiness, release tagging, or global protocol freeze;
- changing frozen canonical bytes for convenience;
- previous/current interoperability before a real previous frozen release exists;
- UDP+TCP striping/aggregation;
- speculative 0-RTT/FEC/exotic carriers without an observed-problem gate;
- third-party targets, scanning, production network mutation, or experiments outside standing authorization;
- HY2 superiority claims from one-off or semantically unequal runs.

## Questions requiring maintainer decision

none.
