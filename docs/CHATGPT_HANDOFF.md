# ChatGPT reviewer handoff — close reachable-tree repair, then rebuild item-4 provenance

## Reviewed repository truth

- Previous reviewer handoff: exact `7d3ce5aa87f4c657c3495c4fb43803f54c0946fb`.
- Developer sequence after that handoff first reached exact `89f26b567512fe531b42c9a3f97c262fa5cbc0fc`, then exact `dc52a5e38ea3cc1d113b00a1118a146bebbfd91a` repaired the compile defect while the reviewer was inspecting the red tree.
- The current reviewer-only handoff commit is a descendant of `dc52a5e`; it does not change runtime/test behavior and must not be treated as the implementation tested-tree anchor.
- No VPS/WAN experiment occurred in this sequence.
- Release truth remains unchanged: item 3 incomplete, item 4 incomplete, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, D019 policy-blocked, `READY_LIVE: none`.

## Reviewer verdict

### Source repair `dc52a5e` — ACCEPT_WITH_BOUNDS

Exact `89f26b5` was genuinely broken: GitHub-hosted `stable checks` failed because the added `#[test] fn ...(&mut TcpStream)` could not compile. Exact `dc52a5e` removes that invalid test attribute/shape, removes `BrokenPipe` from the accepted close outcomes, and changes the original unsupported-only test to accept only orderly EOF or `ConnectionReset` while hard-failing emitted bytes and all other errors.

That closes the **source shape** of the compile blocker.

One minor cleanup remains before final provenance: `assert_rejected_negotiation_closed` is currently a helper used only by this single seam. The previous reviewer bound explicitly said not to create a helper abstraction solely for this repair. Prefer inlining the small `match` back into `executable_rejects_unsupported_only_negotiation_before_noise_or_data` unless the bounded process-test sweep finds a second/third concrete current test that legitimately needs the identical contract. Do not broaden usage merely to justify the helper.

GitHub-hosted checks for `dc52a5e` are extra cross-evidence only and were still running when this review was written. Do not wait for them. The required closure remains local exact-tree validation on a reachable pushed developer commit.

### Evidence-integrity HIGH remains OPEN

The newly committed local review/provenance notes claim clean exact-tree gates on commits described as reachable, including:

- `1db7a9795c37389cf4bc28af2638228f0f58f5fb`;
- `7275062b99ba3152dc05d54d9405fa446332a77e`;
- `c5d0b153a7de92de6d995f0e111947548706abbf`;
- `58b5d13cdb783b018e5e1ec51c9e24ea51f316e8`;
- `77111629ec172b6292a0ab9e360707b1f43ec0e3`;
- `1be290d0449286ecab9412572c05e282c12f2d05`;
- reconciliation tree `6f50d700b673b18679ee1ca3e2f6423a5ded3d3`.

GitHub repository truth cannot resolve those exact SHAs as repository commits. In addition, the pushed sequence that indexed them originally ended at compile-red exact `89f26b5` before `dc52a5e` repaired the source.

Do not infer that no local command ever ran. The concrete problem is provenance: the shared GitHub evidence surface cannot reproduce the claimed exact trees, while the pushed tree that indexed them differed materially. Under the repository's exact-tree policy, these notes are **QUARANTINED / NOT ACCEPTED AS EXACT-TREE EVIDENCE** until reproduced against reachable pushed commits.

The release packet/item-4 wording that says each of those trees has its own clean exact-tree local gate must be corrected or superseded before further item-4 promotion/navigation work.

## MUST_EXECUTE_LOCAL 1 — finish the reachable test repair and exact-tree gate

Owner: `crates/neko-cli/tests/multistream.rs`.

1. If no second concrete current use exists, inline the small terminal-read `match` into `executable_rejects_unsupported_only_negotiation_before_noise_or_data` and remove the one-use helper.
2. Preserve exactly:
   - `Ok(0)` accepted;
   - `ConnectionReset` accepted;
   - `Ok(n > 0)` hard failure;
   - every other I/O error hard failure;
   - exact uniform handshake rejection output and no success/records output.
3. Do not change runtime code or FIN/RST behavior.
4. Do not add `BrokenPipe` or another platform error without an existing committed contract proving equivalence.

Run:

```bash
for i in $(seq 1 20); do
  cargo test -p neko-cli --test multistream \
    executable_rejects_unsupported_only_negotiation_before_noise_or_data -- --exact
done
cargo test -p neko-cli --test multistream
```

Commit/push the final coherent test tree if it changes from `dc52a5e`. Then validate that **exact pushed developer SHA** from a clean checkout/worktree:

```bash
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --short
```

Persist one concise sanitized provenance note with exact reachable SHA, commands, UTC start/end, exit codes, OS/arch, stable Rust version and clean-tree state. No fuzz required for the test-only repair.

If `dc52a5e` itself is retained as the final source shape, it may be the tested-tree anchor, but the local gate must actually be rerun/recorded for that exact reachable SHA.

## MUST_EXECUTE_LOCAL 2 — rebuild the bounded review evidence on a reachable tree

Do not try to preserve the unpublished SHA chain as though it were repository history.

Use one reachable exact post-repair developer tree as the review anchor and re-run the bounded item-4 support surfaces:

1. process-test determinism / OS-artifact assumptions;
2. crypto API misuse/invariants: nonce exhaustion/uniqueness, transcript/context binding, replay, authorization, key-update synchronization;
3. wire/parser fail-closed/allocation bounds;
4. CLI secret/output/pre-auth admission ordering;
5. Session/Carrier evidence-domain separation.

For each surface, record either a concrete defect and smallest repair/regression, or a bounded no-finding result. It is acceptable to consolidate these into one reachable-tree review note rather than recreating six intermediate SHAs.

Hard requirements:

- review anchor must be a pushed commit GitHub can resolve;
- commands/results must correspond to that exact tree;
- if source changes during review, commit/push first and move the anchor to the new reachable exact tree before claiming a full gate;
- do not label a local-only/unpublished commit as reachable;
- do not rewrite historical timestamps/commands as though they were executed on a different tree;
- no cryptanalysis, D019 invention, new crypto/wire architecture, security-capacity numbers or policy changes.

Wire/parser source changes require the pinned decode fuzz smoke. No decoder change means no new fuzz requirement beyond the ordinary full gate.

## MUST_EXECUTE_LOCAL 3 — evidence/index repair

After the reachable review anchor is green:

1. Correct `docs/release-security-review-packet.md` and `docs/reviews/release-item4-subgates-20260909.md` so the unpublished `1db7a97` / `7275062` / `c5d0b15` / `58b5d13` / `7711162` / `1be290d` chain is not represented as accepted reachable exact-tree evidence.
2. The existing local-note files may remain as historical developer reports, but mark/index them as superseded/unverified if retained.
3. Point current item-4 factual support to the new reachable exact-tree review/provenance instead.
4. Run release/status/link checks and final `scripts/check.sh + git diff --check` on the coherent docs closure tree as appropriate.
5. Item 4 remains incomplete; no RC/production/freeze/release flag changes follow automatically.

This is release/evidence correctness, not cosmetic docs work.

## Queue after HIGH closure

Only after the provenance/index HIGH is closed may ordinary rolling review resume. If the rebuilt bounded challenge finds a concrete existing-semantics defect, smallest repair -> regression -> push -> exact-tree local gate -> provenance -> continue. If it finds no concrete defect, do not manufacture implementation.

Then classify remaining work honestly:

- independent maintainer/security judgment remains outside developer self-review;
- D019 remains a maintainer policy/value decision;
- representative adversarial-load/capacity suitability remains unestablished and may require maintainer-selected pressure conditions / authorization;
- item 3 environment/live evidence remains incomplete;
- signing/key custody/SBOM and previous frozen release remain release-policy/dependency facts;
- RC/release/production authority remains explicit.

## Live boundary

Current authoritative opportunity remains `READY_LIVE: none`. Do not repeat HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint migration, key update, IPv6, PLPMTUD or Experimental Track work without a materially new dependency-ready question.

Standing VPS authorization remains valid for future eligible self-owned TCP/UDP work; this handoff creates no new live question.

## Continuous execution order

1. **READY_LOCAL:** finish exact terminal-close test shape if needed.
2. **EXACT-TREE CLOSURE:** focused repeated test + multistream target + clean reachable exact-tree `scripts/check.sh` + `git diff --check` + provenance.
3. **EVIDENCE HIGH REPAIR:** rerun bounded item-4 support reviews on one or more reachable pushed exact trees.
4. **INDEX REPAIR:** supersede/quarantine unverifiable old local-note anchors and correct release/item-4 indexing.
5. **CONDITIONAL REPAIR:** any concrete existing-semantics defect found during the rebuilt review.
6. **CHECKPOINT:** if no further concrete local defect exists, record genuine queue exhaustion and leave external/policy/environment gates open.

Do not pause for GitHub-hosted CI availability. Stop expansion only for an unresolved correctness/security/evidence BLOCKER/HIGH, core architecture decision, destructive/canonical migration, action outside standing authorization, production/third-party action, new credential/server permission, maintainer-valued benchmark/load conditions, repository breakage or actual runtime/tool-budget exhaustion.
