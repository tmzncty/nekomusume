# ChatGPT reviewer handoff — accept reachable TCP gate and close provenance HIGH

## Reviewed repository truth

- Default branch: `main`.
- Exact reviewer HEAD before this refresh: `ca818c1c2890ab60e6ffd41f83f3a4ce6ecfcbe2`.
- Most recent developer source/test commit: exact `dc52a5e38ea3cc1d113b00a1118a146bebbfd91a` (`test: compile portable rejected close assertion`).
- Exact `b2ba69b7b6fce157e24fa1559b3042053d06df3f` persists developer-local clean exact-tree provenance for exact `dc52a5e`.
- Exact `dc52a5e` focused unsupported-negotiation test passed 20/20, full multistream passed, developer-local `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` and `git diff --check` passed on a clean detached checkout, with UTC/OS/arch/Rust/clean-tree provenance recorded in `docs/local-tcp-reset-close-regression-dc52a5e-20260912.md`.
- GitHub-hosted cross-evidence for exact `dc52a5e` is also green for both `stable checks` and `nightly decode fuzz smoke`. Hosted CI is additional evidence only; it did not substitute for the developer-local gate.
- The small `assert_rejected_negotiation_closed` helper is currently one-use. Reviewer no longer requires a style-only inline rewrite now that exact `dc52a5e` has a clean reachable tested-tree anchor. It is a local helper, not a framework or correctness/security defect; do not invalidate the tested anchor merely for style churn.
- No VPS/WAN experiment occurred in this sequence.
- Release truth remains unchanged: item 3 incomplete, item 4 incomplete, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, D019 policy-blocked, `READY_LIVE: none`.

## Reviewer verdict

### TCP close repair/provenance — ACCEPT

The compile blocker and requested terminal-close semantics are closed at exact reachable `dc52a5e` with developer-local exact-tree provenance and separate hosted cross-evidence.

Accepted semantics are narrow:

- orderly EOF accepted;
- `ConnectionReset` accepted;
- emitted bytes hard-fail;
- all other I/O errors hard-fail;
- uniform `neko: handshake rejected\n` remains required;
- no runtime FIN/RST behavior changed.

Do not reopen this seam absent a new concrete failure.

### Evidence-integrity HIGH — still OPEN and now the only queue-head blocker

The later item-4 support/index commits contain a developer-reported review chain whose claimed exact tested SHAs are not repository-reachable from GitHub:

- `1db7a9795c37389cf4bc28af2638228f0f58f5fb`;
- `7275062b99ba3152dc05d54d9405fa446332a77e`;
- `c5d0b153a7de92de6d995f0e111947548706abbf`;
- `58b5d13cdb783b018e5e1ec51c9e24ea51f316e8`;
- `77111629ec172b6292a0ab9e360707b1f43ec0e3`;
- `1be290d0449286ecab9412572c05e282c12f2d05`;
- reconciliation tree `6f50d700b673b18679ee1ca3e2f6423a5ded3d3`.

`docs/release-security-review-packet.md` and `docs/reviews/release-item4-subgates-20260909.md` currently still index that additional layer as if those exact trees had accepted clean local gates. The current item-4 support text also retains the rejected intermediate `BrokenPipe` wording.

Do not infer that the historical local commands never ran. The finding is reproducibility/provenance: the shared GitHub evidence surface cannot resolve those claimed tested trees. Under the repository's exact-tree policy, this layer is **QUARANTINED / NOT ACCEPTED AS EXACT-TREE EVIDENCE** until rebuilt on a reachable pushed anchor.

Do not promote/reclassify item 4 while this HIGH remains open.

## MUST_EXECUTE_LOCAL 1 — rebuild bounded item-4 review on a reachable pushed anchor

Use exact reachable `dc52a5e` as the starting review anchor unless a concrete defect discovered below requires a source change. Do not recreate six unpublished intermediate SHAs.

Re-run the bounded item-4 support surfaces:

1. **Process-test determinism / OS artifacts**
   - confirm current negative process tests encode repository semantics rather than incidental OS close/error behavior;
   - do not broaden EOF/RST equivalence beyond an existing committed contract.

2. **Crypto API misuse/invariants**
   - nonce uniqueness/exhaustion;
   - direction/epoch/key-phase separation;
   - transcript/record-context binding;
   - replay/duplicate/old-epoch rejection;
   - authorization/trust status before protected data admission;
   - implemented synchronized key-update boundaries.

3. **Wire/parser fail-closed/allocation bounds**
   - attacker-controlled length/count bounds before allocation;
   - integer overflow/truncation;
   - malformed/truncated/trailing input;
   - unknown version/frame/type behavior;
   - no panic/unbounded allocation from decoded lengths/counts.

4. **CLI secret/output/pre-auth ordering**
   - no private key/identity secret/plaintext leakage;
   - no success/READY before established authentication/admission;
   - no unauthenticated durable/session-admission state transition through reviewed paths.

5. **Session/Carrier evidence-domain separation**
   - `confirm_received` remains Session transport-delivery evidence, not application `delivered/effect` evidence;
   - `Unsent -> InFlight` preserves assignment-time context;
   - advanced-state novel-byte overlap remains fail-closed;
   - packet/path feedback does not promote Session delivery;
   - TCP reliability does not replace logical `DeliveryAck` state.

For each surface record either:

- a concrete defect with exact owner/API/call path, smallest repair and bounded regression; or
- a bounded no-finding statement with exact inspected scope and explicit exclusions.

Prefer **one consolidated reachable-tree review note** over six intermediate commits.

Hard bounds:

- no cryptanalysis claim;
- no new handshake/cipher/KDF design;
- no D019 TTL/LRU/history-size/source-retention invention;
- no new security capacity/concurrency policy numbers;
- no core Session/Carrier/ACK/wire architecture change;
- no claim that developer/self-review support is independent security approval.

If source changes during review:

1. smallest repair + bounded regression;
2. commit/push first;
3. move the review anchor to that new reachable exact tree;
4. run the full developer-local exact-tree gate before closure claims.

Wire/parser/crypto framing source changes that fall under current fuzz policy require the pinned decode fuzz smoke. No decoder/framing change means no extra fuzz requirement beyond ordinary gates.

**Immediately continue to MUST_EXECUTE_LOCAL 2; do not wait for reviewer cadence.**

## MUST_EXECUTE_LOCAL 2 — repair item-4 evidence/index truth

After the rebuilt reachable review anchor is green:

1. update `docs/release-security-review-packet.md` and `docs/reviews/release-item4-subgates-20260909.md` so the unpublished `1db7a97` / `7275062` / `c5d0b15` / `58b5d13` / `7711162` / `1be290d` chain is no longer represented as accepted reachable exact-tree evidence;
2. remove the unsupported `BrokenPipe` semantic claim from current factual navigation;
3. old local-note files may remain as historical developer reports, but mark/index them as superseded/unverified rather than accepted exact-tree evidence;
4. point current item-4 factual support to the new reachable exact-tree review/provenance;
5. distinguish developer-local exact-tree CI, GitHub-hosted cross-evidence, bounded review support, live WAN evidence and performance claims;
6. do not mark item 4 complete and do not change RC/production/freeze/release flags.

Run the existing release/status/link policy checks that apply, then on the final coherent docs closure tree run:

```bash
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --short
```

Persist concise sanitized provenance for the actual exact tested tree. Do not create a new provenance framework/checker.

**When green, immediately continue to REVIEW_SUPPORT 3.**

## REVIEW_SUPPORT 3 — bounded dependency/safety factual sweep

Only after the provenance HIGH is closed, perform one small factual review **if no equivalent current review already exists**:

- confirm workspace `unsafe_code = "forbid"` remains effective for project crates;
- inventory direct security-sensitive/crypto dependencies and the exact locked versions/features actually used by the current tree;
- verify current documentation does not claim a different crypto/library identity than the executable dependency graph;
- identify any current dependency feature/default-feature choice that directly contradicts an existing repository security invariant.

This is factual supply-chain/API-surface review only.

Do not invent signing/key-custody/SBOM policy, do not introduce `cargo-deny` or a scanner/framework solely to create work, and do not claim CVE completeness without an authoritative vulnerability source/tool actually used and recorded.

Concrete existing-semantics defect -> smallest repair/regression -> push -> exact-tree local gate. No concrete defect -> one bounded note or no commit if there is nothing useful to retain.

## CHECKPOINT 4 — classify genuine remaining blockers

If MUST_EXECUTE_LOCAL 1-2 and REVIEW_SUPPORT 3 close with no new concrete repairable defect, the local coding/review-support queue may genuinely be exhausted. Do not manufacture filler slices.

Classify remaining work truthfully:

- **item 4 independent judgment:** developer/self-review support does not equal independent maintainer/security approval;
- **D019:** source-retention/no-reset policy remains a maintainer policy/value decision;
- **RSEC-001 suitability:** representative adversarial-load/capacity-suitability evidence remains unestablished; selecting pressure/capacity conditions may require maintainer judgment and possibly authorization beyond ordinary bounded runs;
- **item 3/environment:** natural-loss evidence remains incomplete; IPv6 remains environment-blocked; retained HY2/periodic/repeated-failover negatives stay frozen absent a materially new hypothesis;
- **release engineering policy:** signing/key custody/SBOM/publication trust and previous-frozen-release interoperability remain separate policy/dependency facts;
- **release authority:** RC, freeze, release and production readiness remain explicit decisions.

A genuinely exhausted local queue is a valid stop condition. Resume only when repository truth yields a new dependency-ready defect/question or a maintainer decision unlocks a blocked gate.

## Live/release boundary

- `IMPLEMENTATION_COMPLETE=true` only in the repository's bounded research/governance sense;
- item 3 incomplete;
- item 4 incomplete;
- `RELEASE_CANDIDATE=false`;
- `PRODUCTION_READY=false`;
- `FREEZE=false`;
- `RELEASED=false`;
- D019 remains policy-blocked;
- `READY_LIVE: none` remains authoritative.

Do not rerun unchanged HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint migration, key update, IPv6, PLPMTUD or Experimental Track work merely because the VPS remains rented. Standing authorization remains valid for a future materially changed dependency-ready self-owned TCP/UDP question; this handoff creates no such live question.

## Continuous execution order

1. **EVIDENCE HIGH REBUILD:** consolidate bounded item-4 support review on a reachable pushed exact tree.
2. **INDEX REPAIR:** quarantine/supersede unreachable old anchors and correct release/item-4 navigation.
3. **EXACT-TREE CLOSURE:** final local `scripts/check.sh` + `git diff --check` + clean-tree provenance on the actual closure tree.
4. **POST-HIGH FACTUAL SWEEP:** dependency/unsafe/security-sensitive dependency consistency review if not already covered.
5. **CONDITIONAL REPAIR:** any concrete current-semantics defect discovered above.
6. **CHECKPOINT:** otherwise record genuine queue exhaustion and leave policy/environment/independent-decision gates open.

Do not pause for GitHub-hosted CI availability. Stop earlier only for an unresolved correctness/security/evidence BLOCKER/HIGH, core architecture decision, destructive/canonical-meaning migration, action outside standing authorization, production/third-party action, new credential/server permission, maintainer-valued benchmark/load conditions, repository breakage or actual runtime/tool-budget exhaustion.
