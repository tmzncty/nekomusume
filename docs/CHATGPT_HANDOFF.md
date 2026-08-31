# Nekomusume ChatGPT Handoff

Checked at: 2026-09-01 03:16 Asia/Shanghai
Reviewed implementation HEAD: `1d4905eea2137290d768564728ec1c6181f43ddc`
Reviewer base HEAD: `712cc61bd4f3f75d12610fb4121168ba2119e63f`
Previous reviewed implementation HEAD: `d65e3fbd2230799a1734ee29f2e40f02746bceb1`

## What changed

The repository is not globally blocked. The current implementation plan still has a real release-engineering queue after N9: negotiation-path completion, bounded release evidence, independent release/security review, then a separate RC decision. The present short-cycle bottleneck is narrower: N9's executable canonical corpus still has a semantic-oracle enforcement defect.

`1d4905e` materially improved the candidate corpus infrastructure by making corpus identity content-addressed/self-verifying, defining validator-owned required domains, adding mutation tests for stale identity/missing coverage, and wiring the corpus validators into `scripts/check.sh`. It did not change production wire/runtime semantics.

The remaining defect is visible in `crates/neko-wire/tests/canonical_vectors.rs`: successful frame/close decodes still prove frame count rather than complete decoded semantics, and the negotiation hello adapter does not enforce every semantic field declared by `expected.value`.

The previous handoff was too narrow operationally: it could be completed quickly and then forced the coding agent to wait for another reviewer turn. This handoff therefore keeps the same correctness boundary but expands N9 into a complete closure batch with sequential work that is all genuinely relevant to the freeze decision.

A new coordination policy, `docs/vps-rental-window-priority.md`, records that the currently rented VPS is a time-limited one-month research asset. After correctness gates permit, reviewer/coding-agent scheduling must prefer VPS-only evidence and local work that directly unlocks VPS evidence over ordinary polish/speculative work. This does not widen `docs/standing-vps-lab-authorization.md`.

## Review verdict

**needs repair — N9 is READY as a full closure batch; project is not otherwise stuck**

Do not freeze yet. Close the semantic oracle, harden the corpus contract against future silent semantic drift, audit executable coverage, repair known authorization/status wording drift, and run the complete local gate. These are one dependency-ordered N9 batch, not separate reviewer round-trips.

No production wire change is currently requested. If this batch discovers a real codec/parser mismatch, switch to the correctness fallback and keep `freeze=false`.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains repository status for the bounded research implementation baseline.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain correct.
- `1d4905e` is test/fixture/validator/documentation infrastructure, not runtime behavior.
- No GitHub CI attestation is attached to the current implementation HEAD; local gates remain coding-environment evidence, not independent review.
- Standing self-owned VPS authorization is active. It is not the current N9 blocker.
- `docs/vps-rental-window-priority.md` is scheduling/coordination policy only; it does not authorize experiments beyond the standing authorization.
- `docs/status.md` still contains stale/narrow reachability wording relative to `docs/standing-vps-lab-authorization.md`; repair the wording without promoting capability/release status.
- The release plan after N9 remains real work, and the rented VPS creates time pressure to collect real-network evidence once the relevant correctness path is ready, but do not cross the N9 freeze-review boundary until this closure batch is complete and pushed.

## Work Package — N9 Full Closure Batch

### Primary A — Make successful decode semantics fully executable

**Goal**

Every semantic field declared by a successful `decode_bytes_equals_expected=true` vector must be machine-enforced against real implementation output. No declared semantic field may be decorative.

**Files**

- `crates/neko-wire/tests/canonical_vectors.rs`;
- `fixtures/canonical-vectors.v1.json` only when the truthful contract requires removing a field that the public decode operation does not expose;
- validator/schema files only as required for structural enforcement.

**Required behavior**

1. Frame and close rows: decode fixture `bytes_hex` through real `decode_frames`, then compare complete decoded `Frame` semantics, not only `decoded.len()`.
2. If `frame_count` remains declared, assert it from the decoded sequence as an additional explicit boundary, not as the sole semantic oracle.
3. If `payload_bytes` remains declared (including the 1024-byte datagram boundary), derive/assert it from decoded output.
4. Negotiation hello: make `expected.value` truthful. Either prove every declared field via real implementation-observable behavior, or remove fields that the decode operation does not expose and document which semantic belongs to encode vs negotiated-selection evidence. Do not add a production API only to please the fixture.
5. Failure vectors must continue consuming fixture `bytes_hex` and comparing actual implementation error classes.
6. State-only rows remain byte-null and non-executable.
7. Do not alter candidate protocol bytes merely to make tests pass.

### Follow-up B — Enforce operation-specific expected-value contracts

**Dependency:** A green.

Prevent this class of bug from returning.

For each executable operation/domain adapter, define the allowed/required successful `expected.value` key set or deserialize into a complete typed expected semantic value. A new successful expected field must not be silently ignored.

At minimum cover the currently executable families:

- negotiation client hello / response;
- record decode;
- frame/close decode;
- varint decode;
- any other executable corpus adapters already present at current HEAD.

Add mutation/regression tests that demonstrate rejection/failure when:

- an unknown successful expected key is injected;
- a required expected key is removed;
- a declared semantic is changed while bytes remain unchanged;
- `payload_bytes`, frame identity/payload, selected version, record flags/type/payload, or equivalent asserted values become stale.

Keep the validator and Rust executable harness responsibilities clear: JSON/schema/coverage shape validation is not a substitute for implementation semantic assertions.

### Follow-up C — Produce an executable corpus coverage audit

**Dependency:** B green.

Create or update a small review-oriented artifact under `docs/spec/` or an equivalent existing release-evidence location that makes the N9 review mechanically inspectable.

For every vector (or every distinct operation plus explicit exceptional rows if a generated table is used), record/derive:

- vector/domain/operation;
- classification;
- whether bytes exist;
- encode oracle;
- decode oracle;
- roundtrip oracle;
- exact implementation function/path exercised;
- exact successful semantic fields asserted, or expected error asserted;
- state-only/non-wire status.

Prefer generation/validation from the corpus and adapter contract over a manually drifting prose list. The artifact must not become a second normative protocol spec.

Add a gate or consistency check if necessary so an executable vector cannot exist without a mapped adapter/coverage entry.

**Purpose:** the next reviewer must be able to answer “what exactly is frozen if we freeze this corpus?” without manually reverse-engineering hundreds of lines of test code.

### Follow-up D — Repair standing-authorization/status navigation drift

**Dependency:** A/B complete; may be done before C if it avoids unnecessary conflicts.

Repair the already identified governance wording drift:

1. `ROADMAP.md`: remove wording that treats ordinary bounded self-owned WAN execution as awaiting new per-run authorization. Replace it with the actual remaining evidence/environment/release boundary. Do not mark WAN/failover/long-lived capability PASS without evidence.
2. `docs/status.md`: replace “Only isolated authorized observation is permitted” or equivalent stale wording with the current fact: bounded self-owned VPS TCP/UDP execution is authorized, while broader/public/general reachability, sustained release evidence, third-party targets and production exposure remain blocked.
3. Keep the distinction explicit:

```text
execution authorization exists
!= release/public-reachability evidence exists
```

This is documentation/governance repair only.

### Follow-up E — N9 local release-gate rehearsal (still unfrozen)

**Dependency:** A-D green.

Run a full candidate rehearsal while preserving `freeze=false`:

- targeted `neko-wire` canonical-vector integration tests;
- canonical corpus validator;
- canonical validator mutation tests;
- any new expected-key/coverage mutation tests;
- `cargo fmt --all -- --check`;
- `cargo check --workspace --locked`;
- `cargo test --workspace --all-targets --locked --no-fail-fast`;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- `bash scripts/check.sh`;
- `git diff --check`.

Run fuzz smoke only if production parser/decoder behavior changes or the normal repository gate requires it. Do not manufacture a fuzz claim for pure test/validator changes.

Record enough evidence in the ordinary repository evidence location/commit message for the next reviewer to distinguish targeted tests from the full local gate.

After E passes, push all N9 closure commits. Then stop changing canonical bytes unless a newly discovered correctness defect requires it. Do **not** self-set `freeze=true`; the freeze decision remains reviewer/governance work.

## Completion gates

The entire N9 closure batch is complete only when all are true:

- successful frame/close vectors assert complete decoded frame identity/payload semantics;
- declared payload boundaries such as 1024-byte datagram payload are actually checked if they remain declared;
- negotiation expected semantics are truthful and fully enforced;
- operation-specific successful expected fields cannot be added/removed/changed silently;
- expected-failure rows still prove real errors from fixture bytes;
- state-only rows remain non-wire;
- corpus coverage is reviewable without relying on prose claims alone;
- standing VPS authorization wording no longer recreates a fake per-run permission blocker;
- full local repository gate passes;
- `FREEZE=false`, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `RELEASED=false` remain unchanged.

## After this batch — VPS rental-window priority

The next hourly reviewer should perform the actual N9 candidate review. If no concrete corpus defect remains, the reviewer may make/authorize the separate N9 governance freeze decision.

After N9 is closed, do **not** spend the rental window on unrelated local polish. Read `docs/vps-rental-window-priority.md` and prioritize the release queue in this order whenever dependencies permit:

```text
authenticated negotiation path completion needed for truthful WAN behavior
  ↓
VPS real-socket/WAN evidence batch
  - TCP/UDP authenticated sessions
  - repeated open/exchange/close
  - UDP degradation -> TCP fallback
  - uncertain resend/dedup/exactly-once evidence
  - migration-back / endpoint-path migration where the owned environment permits
  - real-session key update / PMTUD observations
  - IPv4/IPv6 as actually available
  ↓
bounded 5-10 minute resilience scenarios under standing authorization
  ↓
Nekomusume-equivalent command for existing HY2 comparison contract
  ↓
paired Nekomusume/HY2 bounded samples on the owned VPS
  ↓
native VPS resource/performance/package evidence
  ↓
independent release/security review
  ↓
separate RC decision
```

Where one VPS-only row is blocked by an implementation/instrumentation dependency, select the smallest local slice that directly unlocks that row rather than an unrelated enhancement.

Do not mechanically split a forbidden >10-minute soak or capacity test to evade standing authorization. Distinct 5-10 minute scenarios are valid only when they answer distinct questions (steady session, idle/periodic exchange, lifecycle repetition, failover/recovery, key-update interaction, etc.).

Do not run CPU-heavy fuzz/build work concurrently with performance-comparison samples; use spare VPS compute between measurement windows for bounded release builds, workspace gates, parser/property/fuzz campaigns after relevant changes, and process CPU/RSS/FD/socket instrumentation work.

The aim is maximum **evidence value per rental day**, not maximum utilization percentage.

The repository's formal next release queue remains:

```text
N9 review/freeze decision
  ↓
authenticated negotiation path completion for probe/UDP/failover-resume
  ↓
bounded release evidence matrix under standing authorization
  ↓
independent release/security review
  ↓
separate RC decision
```

Do not invent new experimental features to fill time; there is already legitimate release work after N9.

## Fallback

If A/B exposes a real production codec/parser mismatch rather than an oracle defect:

1. keep `freeze=false`;
2. preserve a minimal fixture-backed reproducer;
3. repair production correctness first;
4. run the parser/fuzz gates required by `AGENTS.md`;
5. rerun A-E against the corrected implementation;
6. do not continue toward freeze until correctness is closed.

If an item is inapplicable because the corpus does not declare that semantic, document why rather than inventing a new API/field.

## Do not expand into

- changing protocol bytes merely to satisfy fixtures;
- RC/production/security approval;
- previous/current interoperability before a real prior frozen release exists;
- WAN/benchmark reruns before N9 closure;
- 0-RTT, enabled FEC, striping/aggregation or exotic carriers without an observed-problem gate;
- third-party targets, scanning, production network changes, or experiments outside standing authorization.

## Questions requiring maintainer decision

none.
