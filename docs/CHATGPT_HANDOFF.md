# Nekomusume ChatGPT Handoff

Checked at: 2026-09-01 06:00 Asia/Shanghai
Repository HEAD: `29a5fbc28e5ad35acb5600b2d3810c4bcf130cba`
Previous checked implementation HEAD: `27d23cd45318e2539ac70384665d785405db27f1`
Previous reviewer handoff commit: `848fda642f8b807fde41f305234904897ca31acd`

## What changed

Two coding-agent commits are visible after the previous reviewer handoff:

- `824db456` — **test/oracle regression repair; no production runtime or wire change.** It replaces the weak stale-negotiation `assert_ne!` proof with a mutation that feeds a real implementation-derived selected version plus a stale expected value through the same shared `assert_negotiation_selected` semantic assertion path used by the executable corpus oracle, and proves rejection with `catch_unwind` while fixture bytes stay unchanged. This closes the negotiation semantic-mutation defect from the previous review.
- `29a5fbc2` — **local rehearsal evidence only.** It records the reported targeted canonical tests, corpus validator/mutation tests, coverage generator checks, fmt/check/test/clippy, `scripts/check.sh`, and `git diff --check` for `824db456`. It does not change runtime behavior, candidate bytes, or release state.

The N9 candidate is materially closer to reviewable, but two independent review blockers remain.

First, `scripts/generate-canonical-review.py` still maps each negotiation operation to a single coarse adapter string even when the executable Rust harness uses different real functions for encode and decode. In particular, successful `negotiation/client_hello` uses `VersionNegotiator::client_hello` to encode and `VersionNegotiator::server_accept_hello` to decode/establish selection; successful `negotiation/server_response` can use `server_accept_hello` to produce bytes and `client_accept_response` to consume them. The generated review artifact therefore still does not fully satisfy its own N9 purpose of identifying the exact implementation path exercised by each oracle bit.

Second, GitHub Actions is now independently observable and **red** at current HEAD. The stable `bash scripts/check.sh` job passes, but `nightly decode fuzz smoke` fails before any fuzz target executes. The failure is not a newly discovered parser crash: `.github/workflows/ci.yml` pins `cargo-fuzz 0.12.0`; on current nightly `rustc 1.100.0-nightly (908501772 2026-08-30)`, installing that old release fails while compiling its locked `rustix 0.36.5` dependency because current nightly rejects the reserved `rustc_layout_scalar_valid_range_*` attributes. The same install-stage CI failure is visible on older repository history as well, so this is a long-standing CI toolchain/pin defect rather than a regression introduced by N9.

Upstream `rust-fuzz/cargo-fuzz` has a current tagged release `0.13.2` (published 2026-06-09). Treat that as the first upgrade candidate, not as an assumed fix: prove install/build/run compatibility before changing the repository pin.

## Review verdict

**needs repair — N9 semantic oracle is accepted, but freeze is blocked by inaccurate negotiation coverage mapping and a red independent fuzz CI gate**

Keep `freeze=false`.

Do not interpret the GitHub Actions fuzz failure as a parser finding; it occurs at cargo-fuzz installation. Do not hide it with `continue-on-error`, `RUSTC_BOOTSTRAP`, or by removing the fuzz job. Restore a reproducible green fuzz gate, make the generated negotiation coverage oracle-specific and exact, then rerun the complete N9 gate.

The project is not globally blocked. This is enough work for a full batch, and once N9 closes the next priority is the negotiation path that unlocks the time-limited VPS evidence program.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research implementation status.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- `824db456` is test-only semantic-oracle evidence; it changes neither protocol bytes nor runtime behavior.
- `29a5fbc2` is a local evidence note; it is not independent CI attestation.
- GitHub Actions stable checks at `29a5fbc2` are green.
- GitHub Actions fuzz job at `29a5fbc2` is red **before fuzz execution**, during `cargo install cargo-fuzz --version 0.12.0 --locked`.
- The exact CI root cause observed in job logs is old `cargo-fuzz 0.12.0` locked dependency `rustix 0.36.5` failing to compile on current nightly because reserved/internal `rustc_layout_scalar_valid_range_start/end` attributes are rejected.
- The same install-stage failure existed in earlier CI history, so do not label it an N9 code regression.
- Local `scripts/fuzz-smoke.sh` currently does not enforce a cargo-fuzz version, but its error text tells operators to install `0.12.0`; update that guidance/contract consistently with any tested new pin.
- Successful negotiation semantic mutation is now exercised through the shared real assertion path and is accepted.
- Generated canonical coverage remains useful but still understates actual negotiation encode/decode paths by presenting one adapter string per operation.
- Candidate corpus remains content-addressed and `freeze=false`.
- Standing self-owned VPS authorization remains active; the VPS rental window remains time-sensitive, but N9 evidence integrity and green independent gates come first.

## Work Package — restore N9 independent evidence, then close the review surface

### Primary A — Repair the pinned fuzz CI toolchain contract and make independent fuzz CI green

**Goal**

Restore a reproducible bounded GitHub Actions decode-fuzz gate without weakening the fuzz target, silently tolerating failure, or turning an infrastructure error into a fake PASS.

**Why now**

N9 is a release-governance freeze decision. The repository now has independent CI evidence, and it is red. The stable gate is green, but the fuzz job never reaches `cargo fuzz build/run`; freezing while knowingly carrying a broken mandatory CI gate would create an avoidable evidence defect.

**Likely files**

- `.github/workflows/ci.yml`;
- `scripts/fuzz-smoke.sh`;
- `scripts/fuzz-smoke-test.sh` if version-contract behavior is tested there;
- a small tooling note only if needed to record the chosen pin/rationale.

**Required behavior**

1. Reproduce/understand the current failure as an **installation/toolchain compatibility** failure, not a fuzz finding.
2. Test the current upstream pinned release candidate `cargo-fuzz 0.13.2` first. Upstream release tag `0.13.2` was published 2026-06-09. Do not assume compatibility merely because it is newer.
3. If `0.13.2` installs and the repository decode target successfully builds/runs under the current nightly, update the CI pin to exactly `0.13.2` and align local operator guidance/version checks so CI and local fuzz contracts do not drift.
4. Preserve the bounded fuzz semantics: decode target, seed corpus, finite timeout, `max_len=8192`, failure on actual crash/finding/build error.
5. Prefer an explicit version check (`cargo fuzz --version` or equivalent) so a locally installed stale binary cannot silently masquerade as the selected tool contract.
6. Do **not** use `continue-on-error`, `|| true`, blanket retries, `RUSTC_BOOTSTRAP`, or removal/demotion of the fuzz job to get green status.
7. If `0.13.2` itself does not work, preserve the exact failure and choose the narrowest reproducible fallback: a tested date-pinned nightly compatible with the selected cargo-fuzz release may be used only with documented rationale. Do not blindly move both tool and compiler until something happens to pass.
8. Add/update focused shell/contract tests so the selected cargo-fuzz version cannot drift independently between CI and `scripts/fuzz-smoke.sh` guidance.

**Validation**

Before push, run at minimum:

- shell syntax/contract tests covering fuzz scripts;
- `cargo fuzz --version` showing the selected pin;
- bounded `FUZZ_TIME` local `scripts/fuzz-smoke.sh` or the equivalent exact build/run path;
- `bash scripts/check.sh`;
- `git diff --check`.

After push, **inspect the actual GitHub Actions run**. Completion requires both:

```text
stable checks = success
nightly decode fuzz smoke = success
```

A local pass with GitHub fuzz CI still red does not close Primary A.

**Do not expand into**

- production parser changes unless a real fuzz finding appears;
- changing fuzz target semantics merely to accommodate tooling;
- unrelated dependency upgrades;
- VPS experiments while measuring/debugging the CI toolchain.

### Follow-up B — Make canonical review coverage oracle-specific and exact for negotiation

**Dependency:** A implemented/pushed; stable local gate green. This work is independent of VPS and should be completed before freeze.

**Goal**

Make `docs/spec/canonical-vector-review.v1.md` truthfully identify the implementation functions exercised by each enabled encode/decode/roundtrip oracle, especially negotiation rows.

**Current defect**

The generator currently has one coarse adapter mapping per `(domain, operation)`. That is insufficient where one operation exercises different implementation paths for encode and decode.

**Required behavior**

1. Replace the single adapter string model with oracle-specific path metadata or an equivalent deterministic representation, e.g.:
   - encode path;
   - decode path;
   - roundtrip/replay path when meaningfully distinct.
2. For `negotiation/client_hello`, the generated review must expose that encoding exercises `VersionNegotiator::client_hello` while the decode/selection oracle exercises `VersionNegotiator::server_accept_hello`.
3. For successful `negotiation/server_response`, expose the producer path (`server_accept_hello` where used) separately from the consumer path (`client_accept_response`).
4. Preserve exact mappings for record/frame/close/varint/error adapters; do not make the new representation negotiation-only if that would create future ambiguity elsewhere.
5. Keep generation deterministic and checked-in artifact drift fail-closed through `scripts/check.sh`.
6. Extend `scripts/generate-canonical-review-test.py` so a missing/mislabeled enabled oracle path fails. A test should specifically catch the old coarse negotiation mapping shape.
7. Do not change candidate bytes or production negotiation semantics to satisfy the report.

**Completion definition**

For every executable vector, the generated review can answer which real implementation path each enabled oracle bit actually exercises; an enabled negotiation decode oracle cannot be shown only as `client_hello`, and an encode path cannot be hidden behind a decode-only label.

### Follow-up C — Full unfrozen N9 rehearsal with independent CI confirmation

**Dependency:** A+B green.

Run the complete candidate gate, still with `freeze=false`:

- targeted canonical-vector Rust integration tests;
- semantic mutation regressions, including stale negotiation selection;
- corpus validator + mutation tests;
- generated canonical review `--check` + drift tests;
- `cargo fmt --all -- --check`;
- `cargo check --workspace --locked`;
- `cargo test --workspace --all-targets --locked --no-fail-fast`;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- `bash scripts/check.sh`;
- selected bounded decode fuzz smoke using the repaired pin/toolchain;
- `git diff --check`.

Push a coherent N9 repair/rehearsal state. Then verify the resulting GitHub Actions run rather than relying only on commit-message claims.

**Completion definition**

All local N9 gates pass; generated coverage is exact; GitHub stable and fuzz jobs both pass; no candidate wire/semantic bytes changed as an accidental side effect; governance flags remain unchanged.

After C, stop changing canonical bytes unless a concrete correctness defect appears. Do **not** self-set `freeze=true`; the next reviewer will inspect the exact green pushed candidate and perform/authorize the separate governance freeze transition.

### Follow-up D — If C finishes early, prepare the first post-N9 negotiation/VPS unlock slice without crossing the freeze boundary

**Dependency:** C pushed and green. This is stretch work to avoid wasting the VPS rental window, but it must not delay the clean N9 handoff.

Audit the current probe/UDP/failover-resume paths against `IMPLEMENTATION_PLAN.md` item 2 and the standing-authorized VPS evidence queue. Produce or implement only the smallest **non-wire-changing** prerequisite that makes the first post-N9 real-socket/VPS run immediately executable after freeze review.

Prioritize gaps in this order:

1. authenticated version negotiation actually wired into probe/UDP pre-session admission;
2. failover/resume binding of the negotiated version/transcript;
3. deterministic downgrade/unsupported/replay/amplification negative tests;
4. a reusable command/harness that can drive a bounded authenticated exchange on the self-owned VPS and emit process/application metrics needed by the release evidence matrix/HY2 comparison.

If implementing any of these would change frozen candidate wire bytes or semantics, **do not implement before N9 freeze review**. Instead produce an exact dependency map in ordinary developer-owned docs/tests and stop at the boundary.

If a protocol-neutral benchmark adapter is already possible with current CLI semantics, it may be implemented/tested separately so the VPS paired comparison can start immediately after the negotiation gate closes. Do not run unequal HY2/Nekomusume comparisons or make performance claims.

## Fallback

If `cargo-fuzz 0.13.2` installs but exposes a real target compile/fuzz finding:

1. classify it as a real correctness gate, not CI infrastructure;
2. keep `freeze=false`;
3. preserve the exact reproducer/artifact;
4. make the parser/wire correctness repair the new Primary;
5. rerun fuzz and the full N9 gate before returning to freeze.

If `0.13.2` cannot be made reproducible on current nightly, preserve logs and use a tested date-pinned nightly only if that is the smallest maintainable compatibility fix. Do not weaken the job.

If exact generator mapping work reveals the Rust executable harness itself calls an unexpected path, update the evidence map to the code fact first; change runtime behavior only if an independent correctness/spec reason exists.

If post-N9 preparation requires a real protocol semantic change, record the dependency and leave it for the post-freeze negotiation-path work package.

## Completion gates

N9 is ready for the next reviewer freeze decision only when all are true:

- stale negotiation selected semantics fail through the shared real oracle assertion path (`824db456` behavior retained);
- operation-specific successful expected-field contracts remain fail-closed;
- record/frame/negotiation semantic mutation regressions remain green;
- generated canonical review maps enabled encode/decode/roundtrip oracles to the exact real implementation paths, including both sides of negotiation;
- generated review drift tests catch missing/mislabeled oracle-path coverage;
- candidate corpus remains content-addressed and `freeze=false` during rehearsal;
- full local Rust/repository gates pass;
- bounded decode fuzz actually runs rather than failing at tool installation;
- GitHub Actions `stable checks` and `nightly decode fuzz smoke` both pass on the pushed N9 candidate;
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged until the reviewed governance transition.

## After this batch

The next reviewer should inspect the exact green candidate. If no concrete corpus/evidence-contract defect remains, make or explicitly authorize a separate N9 governance freeze commit. That transition must update the corpus hash/schema/validator/tests/docs coherently and must **not** imply RC, security approval, production readiness, or release.

Immediately after N9, move to authenticated negotiation-path completion and then the highest-value standing-authorized VPS-only evidence. The one-month VPS rental window should not be consumed by unrelated local polish once those dependencies are green.

## Do not expand into

- hiding a red CI job;
- `RUSTC_BOOTSTRAP`/`continue-on-error` as a release-gate workaround;
- broad dependency churn;
- candidate wire-byte changes merely to simplify tests or reports;
- self-setting `freeze=true` before the next exact-candidate review;
- RC/production/security approval;
- previous/current interoperability before a real prior frozen release exists;
- speculative 0-RTT/FEC/striping/exotic carriers;
- third-party targets, scanning, production network changes, or experiments outside standing authorization;
- HY2 superiority claims from non-equivalent or one-off workloads.

## Questions requiring maintainer decision

none.
