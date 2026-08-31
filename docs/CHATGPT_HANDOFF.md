# Nekomusume ChatGPT Handoff

Checked at: 2026-09-01 03:01 Asia/Shanghai
Repository HEAD: `1d4905eea2137290d768564728ec1c6181f43ddc`
Previous checked implementation HEAD: `d65e3fbd2230799a1734ee29f2e40f02746bceb1`
Previous reviewer handoff commit: `16677fd0219bc7cfde058e6b2f3adc0b3bb9d7c0`

## What changed

One new coding-agent commit is visible after the previous reviewer handoff:

- `1d4905e` — **test/fixture/validator/documentation repair; no production runtime change**. It replaces the candidate corpus's mutable `parent_commit` provenance with a deterministic `schema_revision + corpus_sha256` content identity, makes required domains an explicit validator-owned set, gives `close` its own required domain while routing it through the existing real frame codec adapter, adds mutation tests for stale identity and missing required-domain coverage, and wires both canonical-vector validators into the repository-wide `scripts/check.sh` gate.

This is a useful evidence-contract improvement. The content-addressed identity is materially stronger than tying the corpus to a moving implementation-parent field, and the new mutation tests make accidental fixture drift easier to detect.

However, `1d4905e` did **not** close the Primary repair from the previous handoff. The current `crates/neko-wire/tests/canonical_vectors.rs` still verifies successful frame decodes only by `decoded.len() == expected.frame_count`, and `negotiation.hello.v0-v2` still verifies only the selected version while leaving the declared `expected.value.versions` field unchecked.

No GitHub commit-status/CI checks are attached to the current HEAD. Repository-local checks may be valid coding-environment evidence, but they are not independent CI attestation.

## Review verdict

**needs repair — N9 semantic oracle Primary remains open**

Do not freeze the candidate corpus yet. The new corpus identity work is accepted as supporting evidence infrastructure, but it does not satisfy the current N9 completion gate because successful `decode_bytes_equals_expected=true` rows can still carry declared semantic fields that the executable adapter never checks.

This remains an evidence/oracle enforcement defect, not evidence of a production codec bug. Do not change protocol bytes or production semantics merely to satisfy the fixture.

The next coding-agent slice should close this exact semantic-oracle gap before doing any more corpus-identity work or asking for an N9 freeze decision.

## Evidence boundaries

- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- `1d4905e` changes corpus identity/validation/test infrastructure, not production wire/runtime behavior.
- `corpus_sha256` is now recomputed over deterministic JSON content with the hash field omitted, and required coverage is a fixed validator-owned domain set rather than inferred from whatever fixture rows survive.
- `scripts/check.sh` now includes both the canonical corpus validator and its mutation tests. This improves the local repository gate but does not create independent GitHub CI evidence.
- The Rust executable-oracle harness still does not enforce all successful declared semantics:
  - frame/close rows compare only decoded frame count, not decoded frame identity/payload semantics;
  - `frame.datagram-max-1024` still declares `payload_bytes: 1024` without the current harness directly asserting that declared field;
  - `negotiation.hello.v0-v2` still declares `expected.value.versions: [0,2]` while the current decode path checks only `selected: 2`.
- Standing self-owned VPS authorization remains active and is not an N9 blocker.
- `ROADMAP.md` and `docs/status.md` still contain authorization wording that is narrower/staler than `docs/standing-vps-lab-authorization.md`; this remains a documentation/governance repair, not a capability PASS.
- Independent release/security review remains a later RC gate and is not a prerequisite for this N9 corpus repair.

## Work Package

### Primary — Close successful decode-oracle semantic enforcement

**Goal**

Make every successful `decode_bytes_equals_expected=true` claim fully machine-enforced, without changing wire semantics and without setting `freeze=true`.

**Why now**

The corpus now has stronger identity and coverage validation, so the remaining N9 defect is narrowly defined: the executable adapter must not silently ignore semantic fields declared by `expected.value`.

**Likely files**

- `crates/neko-wire/tests/canonical_vectors.rs`;
- `fixtures/canonical-vectors.v1.json` only if an expected field is genuinely not part of the intended decode oracle and should be truthfully removed;
- `scripts/validate-canonical-vectors.py` or its mutation test only if needed to prevent future unchecked expected-value fields structurally.

**Required behavior**

1. For every successful frame/close row, compare the decoded `Frame` sequence against the declared/input frame semantics, not only the count. A direct `decoded == input_frames(&v)`-style assertion is acceptable if it truly exercises the fixture bytes through `decode_frames` first.
2. Enforce every declared successful frame expected field. In particular, if `payload_bytes` remains in `expected.value`, compute/assert it from the real decoded value. Do not leave it as unchecked decoration.
3. Prevent future silent expected-field drift: for each executable operation, either compare a complete typed semantic value or enforce an operation-specific allowed/required `expected.value` key set before reading it. Adding a new expected field must not be able to pass without a corresponding oracle assertion.
4. For `negotiation.hello.v0-v2`, make the oracle contract truthful. Either:
   - explicitly verify every declared decode semantic, including the offered-version list through real implementation-observable behavior; or
   - remove `expected.value.versions` if the current public decode operation does not expose that semantic and document that offered-version bytes are covered by the encode oracle while the decode oracle covers negotiated selection.
   Do not add a new production API solely to make the fixture convenient unless there is an independent protocol/API reason.
5. Keep expected-failure rows consuming the fixture's actual `bytes_hex` and comparing real implementation errors.
6. Keep `state_only` rows byte-null/non-executable.
7. Keep `freeze=false` in the coding-agent repair commit.

**Validation**

Run at minimum:

- targeted `neko-wire` canonical-vector integration test;
- `python3 scripts/validate-canonical-vectors.py fixtures/canonical-vectors.v1.json`;
- `python3 scripts/validate-canonical-vectors-test.py`;
- `bash scripts/check.sh`;
- `git diff --check`.

Run fuzz smoke only if production parser/decoder behavior changes. A pure test/fixture/validator semantic-enforcement repair does not need a fake fuzz claim.

**Completion definition**

Every successful declared decode semantic is actually asserted against real implementation output; no operation can silently accept an unknown/unasserted `expected.value` field; all local gates pass; and the candidate remains `freeze=false` for the next independent reviewer decision.

**Do not expand into**

- additional corpus identity schemes;
- changing production wire bytes;
- RC declaration or RC manifest work;
- new negotiation runtime integration;
- WAN/benchmark reruns;
- previous-release interoperability.

### Follow-up 1 — Repair standing-authorization status/navigation drift

**Dependency:** Primary complete and green.

The previous reviewer finding remains unresolved:

1. `ROADMAP.md` Milestone 1 still says real WAN failover/long-lived/NAT validation is blocked because it “需新的授权”. Ordinary bounded self-owned TCP/UDP execution is already authorized by `docs/standing-vps-lab-authorization.md`. Rewrite the blocker as missing evidence/environment/release scope where appropriate; do not mark the capability PASS.
2. `docs/status.md` reachability still says “Only isolated authorized observation is permitted”. This is too narrow. Bounded self-owned VPS TCP/UDP execution is permitted; broader/public/general reachability claims and production exposure remain blocked.

Preserve the exact distinction:

```text
execution authorization exists
!= release/public-reachability evidence exists
```

This is documentation/governance repair only. Do not alter capability status merely because execution is authorized.

### Follow-up 2 — Return the repaired candidate for N9 freeze review

**Dependency:** Primary complete; Follow-up 1 complete or demonstrably not applicable.

After the repair commits are pushed, stop changing candidate wire bytes and let the next hourly ChatGPT reviewer inspect the exact candidate. The coding agent must not self-freeze the corpus.

The next reviewer should either:

- identify a concrete remaining corpus defect and keep `freeze=false`; or
- authorize/perform a separate N9 governance freeze change if the candidate really satisfies the current gate.

If there is no new reviewer handoff yet, it is correct to stop after the real READY repairs rather than invent unrelated work. The hourly cadence is intended to keep this wait short.

### Fallback

If full semantic comparison exposes a real production codec/parser mismatch rather than a test-oracle defect:

- keep `freeze=false`;
- preserve a minimal reproducer;
- make the production correctness repair the next Primary;
- run the parser/fuzz gates required by `AGENTS.md`;
- do not continue to freeze or negotiation-path work until correctness is closed.

## Completion gates

- Successful frame/close decodes prove decoded frame identity and payload semantics, not only frame count.
- `frame.datagram-max-1024` proves the declared 1024-byte decoded payload boundary if that field remains declared.
- Negotiation hello declares only semantics the executable decode oracle actually proves.
- No successful `expected.value` field can be added and silently ignored by the executable adapter.
- Failure rows still prove real implementation errors from fixture bytes.
- State-only rows remain unmistakably non-wire.
- Canonical identity/mutation validator, targeted Rust integration test, full repository gate, and `git diff --check` pass locally.
- Authorization/status wording no longer recreates a nonexistent per-run self-owned VPS permission blocker.
- `FREEZE=false`, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` remain unchanged pending later reviewed decisions.

## Do not expand into

- RC/production/security approval;
- changing candidate bytes merely to satisfy fixtures;
- previous/current interoperability before a real prior frozen release exists;
- 0-RTT, FEC enablement, striping/aggregation or exotic carriers without an observed-problem gate;
- third-party targets, scanning or production network changes;
- experiments outside standing authorization.

## Questions requiring maintainer decision

none.
