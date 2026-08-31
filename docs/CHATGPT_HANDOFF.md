# Nekomusume ChatGPT Handoff

Checked at: 2026-09-01 01:58 Asia/Shanghai
Repository HEAD: `d65e3fbd2230799a1734ee29f2e40f02746bceb1`
Previous checked implementation HEAD: `ee555e2874420b7cf92a4c088a617daa94b8b23c`
Previous reviewer handoff commit: `16458f6dbe818e3ecbba83ef2982c1d5e0aafac5`

## What changed

One new coding-agent commit is visible after the previous reviewer handoff:

- `d65e3fb` — **test/fixture/validator/documentation only; no production runtime implementation**. It materially repairs the N9 candidate corpus: every non-`state_only` row is routed through an executable adapter; fixture bytes are consumed for decode/error oracles; state-only rows use `bytes_hex: null`; `close.empty` is promoted to a real frame vector; the old misleading datagram-max row is split into a truthful small vector plus a real 1024-byte maximum vector; negotiation response/error, frame boundary/error, outer-record, and canonical-varint coverage is expanded; and `docs/spec/canonical-vector-corpus-scope.md` explicitly documents executed coverage and deliberate exclusions.

The current corpus still has `freeze=false`, which is correct. No GitHub commit-status/CI checks are attached to `d65e3fb`; this review therefore treats repository code/fixture content as reviewable evidence, not independent CI attestation.

## Review verdict

**needs one bounded repair before N9 freeze review**

The seven concrete defects from the prior handoff are substantially addressed. The candidate corpus is much closer to being truthfully executable and the scope note now makes the wire/state boundary explicit.

However, one evidence-contract defect remains in `crates/neko-wire/tests/canonical_vectors.rs`: the test named `every_claimed_oracle_executes_real_implementation_code` does execute every claimed wire adapter, but some successful `decode_bytes_equals_expected=true` rows do **not compare the full declared `expected.value` semantics**.

Concrete examples:

1. Positive frame rows only assert `decoded.len() == expected.frame_count`. They do not assert the decoded `Frame` values equal the fixture/input semantics. In particular `frame.datagram-max-1024` declares `expected.value.payload_bytes = 1024`, but the current harness never checks that field.
2. `negotiation.hello.v0-v2` consumes the fixture hello bytes and verifies the selected version, but the declared `expected.value.versions` field is not itself checked. Encode equality currently makes a mismatch unlikely, but an oracle named `decode_bytes_equals_expected` must not rely on that implication.

This is an evidence/oracle enforcement issue, not a wire-semantics bug. Do **not** change protocol bytes to make the test convenient.

N9 therefore remains `freeze=false`. The next independent ChatGPT review should make the actual freeze decision after this narrow repair and green validation evidence. The coding agent must not set `freeze=true` in the repair commit.

## Evidence boundaries

- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- `d65e3fb` changes corpus/test evidence, not production wire/runtime behavior.
- The scope note correctly limits the candidate interoperability corpus to public `neko-wire` record/frame/negotiation bytes; ACK/reliable-UDP/datagram-policy/key-update/carrier-transition rows are explicitly state-only and non-wire.
- The current corpus covers all `Frame` variants presently visible in `neko-wire`, outer `RecordType::{Data,Ack,PathChallenge}`, negotiation hello/response/failure cases, frame count/payload boundaries, and canonical varint boundaries. Additional malformed permutations remain ordinary unit/fuzz coverage by explicit scope decision.
- No independent GitHub CI attestation exists for the current HEAD. Local gate claims must remain local evidence unless separately published/checked.
- Standing self-owned VPS authorization remains active and is not an N9 blocker.
- Independent release/security review is a later RC gate, not a prerequisite for this corpus repair/freeze review.

## Work Package

### Primary — Make successful decode oracles compare their full declared semantics

**Goal**

Close the final N9 executable-oracle gap without changing wire semantics or freezing the corpus.

**Why now**

N9 is the first READY release-engineering gate. The corpus is now structurally strong enough that the remaining issue is narrow: `decode_bytes_equals_expected=true` must mean exactly what it says for successful rows, not merely “the decoder returned something of the expected count.”

**Likely files**

- `crates/neko-wire/tests/canonical_vectors.rs`;
- `fixtures/canonical-vectors.v1.json` only if an `expected.value` field is redundant/incorrect and should be truthfully simplified;
- `scripts/validate-canonical-vectors.py` only if a general structural invariant is added.

**Required behavior**

1. For successful frame rows, compare the decoded semantic `Frame` sequence against the fixture/input semantics (for example, `decoded == input_frames(&v)`), not only `decoded.len()`.
2. If `expected.value` carries additional fields such as `payload_bytes`, either assert them from the real decoded value or remove them only if they are not actually part of the intended oracle contract. Do not leave declared expected fields unchecked.
3. For successful negotiation hello rows, explicitly enforce every declared expected semantic field, including the offered-version list if it remains in `expected.value`.
4. Keep expected-failure rows consuming the fixture's actual `bytes_hex` and comparing the real implementation error.
5. Keep `state_only` rows byte-null/non-executable.
6. Keep corpus `freeze=false` in the coding-agent repair commit.

**Validation**

Run at minimum:

- targeted `neko-wire` canonical-vector integration test;
- `python3 scripts/validate-canonical-vectors.py fixtures/canonical-vectors.v1.json` (with the repository's expected-parent convention if required);
- `bash scripts/check.sh`;
- `git diff --check`.

Run fuzz smoke only if production parser/decoder behavior or a fuzz-covered parser adapter is changed; a pure test/fixture assertion repair does not need a fake fuzz claim.

Because GitHub currently has no independent status checks, record local validation truthfully in the normal repository/commit evidence convention; do not call it independent CI.

**Completion definition**

Every successful `decode_bytes_equals_expected=true` row verifies every semantic field it declares, every failure row verifies the real error from fixture bytes, all local gates pass, and `freeze=false` remains unchanged for independent reviewer freeze decision.

**Do not expand into**

- changing wire semantics;
- RC declaration or RC manifest work;
- negotiation runtime integration;
- WAN/benchmark reruns;
- previous-release interoperability.

### Follow-up 1 — Repair standing-authorization status/navigation drift

**Dependency:** Primary complete and green.

Two repository-status texts still contradict the already accepted standing authorization:

1. `ROADMAP.md` Milestone 1 says real WAN failover/long-lived/NAT validation is blocked because it “needs new authorization”. Ordinary bounded self-owned TCP/UDP work is already authorized; the real missing pieces are evidence/environment/release scope, not repeated permission.
2. `docs/status.md` reachability says “Only isolated authorized observation is permitted”. This is too narrow after `docs/standing-vps-lab-authorization.md`; bounded self-owned VPS execution is permitted, while **public/general reachability claims** remain blocked.

Correct those words without changing any capability claim to PASS. Preserve the distinction:

```text
execution authorization exists
!= release/public-reachability evidence exists
```

This is documentation/governance repair only.

### Follow-up 2 — Wait for independent N9 freeze decision before changing candidate wire bytes

**Dependency:** Primary complete; status drift repair complete or not applicable.

Do not self-freeze the corpus and do not start a wire-changing negotiation slice while the repaired candidate is awaiting the next reviewer pass. The next ChatGPT review should inspect the exact repair commit and either:

- record another concrete defect and keep `freeze=false`; or
- make/authorize the separate N9 freeze governance change.

If no new handoff exists yet, it is acceptable to stop after the two real READY repairs rather than invent unrelated work. The hourly reviewer cadence is intended to keep this wait short.

### Fallback

If the Primary unexpectedly reveals a real production codec/parser defect rather than a test-oracle defect:

- keep `freeze=false`;
- stop N9 freeze work;
- preserve a minimal reproducer;
- make correctness repair the next Primary;
- run the full parser/fuzz gates required by `AGENTS.md`;
- do not continue into negotiation runtime integration until the codec defect is closed.

## Completion gates

- Every claimed successful decode oracle checks its complete declared semantics.
- Frame vectors prove decoded frame identity/payload semantics, not only frame count.
- `frame.datagram-max-1024` really proves the declared 1024-byte decoded boundary.
- Negotiation hello expected fields are all enforced or truthfully simplified.
- State-only rows remain unmistakably non-wire.
- Targeted canonical-vector test, structural validator, full repository gate and `git diff --check` pass locally.
- Authorization/status wording no longer creates a nonexistent per-run WAN permission blocker.
- `FREEZE=false`, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` remain unchanged pending later decisions.

## Do not expand into

- RC/production/security approval;
- changing candidate bytes merely to satisfy fixtures;
- previous/current interoperability before a real prior frozen release exists;
- 0-RTT, FEC enablement, striping/aggregation or exotic carriers without an observed-problem gate;
- third-party targets, scanning or production network changes;
- experiments outside standing authorization.

## Questions requiring maintainer decision

none.
