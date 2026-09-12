# ChatGPT reviewer handoff — rebuild item-4 evidence on a reachable tree

## Reviewed repository truth

- Default branch: `main`.
- Exact reviewer HEAD before this refresh: `a19492343d627ec776075e9e1c02e157e112fa0f` (`docs(handoff): accept source fix and require reachable provenance rebuild`).
- The most recent developer-owned source/test commit is exact `dc52a5e38ea3cc1d113b00a1118a146bebbfd91a` (`test: compile portable rejected close assertion`).
- Exact `89f26b567512fe531b42c9a3f97c262fa5cbc0fc` was genuinely compile-red because an attempted helper was incorrectly marked `#[test]` while accepting `&mut TcpStream`. Exact `dc52a5e` removes that invalid test shape, removes `BrokenPipe` from the accepted outcomes, and makes the unsupported-only negotiation path accept only orderly EOF or `ConnectionReset` while hard-failing emitted bytes and all other errors.
- GitHub-hosted cross-evidence for exact `dc52a5e` is now complete and green for both `stable checks` and `nightly decode fuzz smoke`. This is useful cross-evidence only; it does **not** replace the required developer-local exact-tree gate/provenance.
- Current source still keeps `assert_rejected_negotiation_closed` as a one-use helper used only by `executable_rejects_unsupported_only_negotiation_before_noise_or_data`. The prior reviewer bound explicitly rejected creating a helper abstraction solely for this seam, so inline the small match before final local provenance unless a genuinely existing second use is identified. Do not broaden use merely to justify the helper.
- No VPS/WAN experiment occurred in this sequence.
- Authoritative release truth remains unchanged: item 3 incomplete, item 4 incomplete, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, D019 remains policy-blocked, and `READY_LIVE: none`.
- `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, `docs/standing-vps-lab-authorization.md`, `docs/vps-rental-window-priority.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, `docs/release-security-review-packet.md`, and the current item-4 support file were re-read for this refresh. No architecture or authorization expansion is introduced here.

## Reviewer verdict

### Source repair — ACCEPT_WITH_BOUNDS

The repository compile blocker is closed at source level by exact `dc52a5e`; hosted stable/fuzz cross-evidence is green. The remaining source cleanup is narrow and mechanical: remove the one-use helper and inline the exact accepted terminal-close semantics into the existing unsupported-only process test.

This is not a runtime defect, does not change negotiation/Noise/Session semantics, and requires no fuzz rerun merely for the test-only inline cleanup.

### Evidence-integrity HIGH — OPEN

The later item-4 support/index commits contain a developer-reported review chain whose claimed exact tested SHAs are not repository-reachable from GitHub, including:

- `1db7a9795c37389cf4bc28af2638228f0f58f5fb`;
- `7275062b99ba3152dc05d54d9405fa446332a77e`;
- `c5d0b153a7de92de6d995f0e111947548706abbf`;
- `58b5d13cdb783b018e5e1ec51c9e24ea51f316e8`;
- `77111629ec172b6292a0ab9e360707b1f43ec0e3`;
- `1be290d0449286ecab9412572c05e282c12f2d05`;
- reconciliation tree `6f50d700b673b18679ee1ca3e2f6423a5ded3d3`.

The current release packet and `docs/reviews/release-item4-subgates-20260909.md` still index that additional layer as if those trees each had accepted clean exact-tree local gates. The same text also retains the earlier over-broad `BrokenPipe` wording from the rejected intermediate attempt. These claims are not acceptable exact-tree evidence under the repository's current provenance policy.

Do **not** infer that the local commands never ran. The finding is provenance/reproducibility: the shared GitHub handoff surface cannot resolve the claimed exact trees, so they are **QUARANTINED / NOT ACCEPTED AS EXACT-TREE EVIDENCE** until the bounded review is rebuilt on a reachable pushed exact tree.

Do not continue item-4 promotion/reclassification while this HIGH remains open.

## READY_LOCAL 1 — finish terminal-close test shape

Owner: `crates/neko-cli/tests/multistream.rs`.

Required minimal change:

1. remove `assert_rejected_negotiation_closed` if it remains one-use;
2. inline the terminal read match in `executable_rejects_unsupported_only_negotiation_before_noise_or_data`;
3. accept exactly:
   - `Ok(0)`;
   - `Err(e)` with `e.kind() == std::io::ErrorKind::ConnectionReset`;
4. `Ok(n > 0)` remains hard failure;
5. every other I/O error remains hard failure;
6. keep exact uniform `neko: handshake rejected\n`, no success JSON and no record output;
7. do not change runtime FIN/RST behavior and do not re-add `BrokenPipe` without an existing committed contract proving equivalence.

Focused validation before commit:

```bash
for i in $(seq 1 20); do
  cargo test -p neko-cli --test multistream \
    executable_rejects_unsupported_only_negotiation_before_noise_or_data -- --exact
done
cargo test -p neko-cli --test multistream
```

Commit and push the coherent final test shape. Do not keep the final tested tree only in a local unpublished chain.

**Immediately continue to READY_LOCAL 2.**

## READY_LOCAL 2 — reachable exact-tree local closure

On the exact pushed developer SHA from a clean checkout/worktree run:

```bash
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --short
```

Record concise sanitized developer-local provenance:

- exact reachable SHA;
- exact commands;
- UTC start/end;
- exit codes;
- OS/arch;
- stable Rust version;
- clean-tree state.

Do not record credentials, private addresses/topology, or unnecessary absolute paths.

Hosted CI for `dc52a5e` is green extra evidence; do not substitute it for this local exact-tree closure and do not wait for new hosted CI.

If the gate fails, fix the actual failure, push the replacement source SHA, and rerun on that exact reachable tree.

**When green, continue immediately to READY_LOCAL 3.**

## READY_LOCAL 3 — rebuild item-4 bounded review on one reachable anchor

Use one reachable exact post-repair developer commit as the review anchor. Re-run the bounded item-4 support surfaces instead of recreating the unpublished intermediate SHA chain:

1. process-test determinism / OS-artifact assumptions;
2. crypto API misuse/invariants:
   - nonce uniqueness/exhaustion;
   - direction/epoch/key-phase separation;
   - transcript/record-context binding;
   - replay/duplicate/old-epoch rejection;
   - authorization/trust status before protected data admission;
   - implemented synchronized key-update boundaries;
3. wire/parser fail-closed/allocation bounds:
   - attacker-controlled length/count bounds before allocation;
   - integer overflow/truncation;
   - malformed/truncated/trailing input;
   - unknown version/frame/type behavior;
   - no panic/unbounded allocation;
4. CLI secret/output/pre-auth admission ordering:
   - no private key/identity secret/plaintext leakage;
   - no success/READY before the established authentication/admission boundary;
   - no unauthenticated durable/session-admission state transition through the reviewed path;
5. Session/Carrier evidence-domain separation:
   - `confirm_received` is Session transport-delivery evidence, not application `delivered/effect` evidence;
   - `Unsent -> InFlight` preserves assignment-time context;
   - advanced-state novel-byte overlap remains fail-closed;
   - packet/path feedback does not promote Session delivery;
   - TCP reliability does not replace logical `DeliveryAck` state.

For each surface, record either:

- a concrete defect with exact owner/API/call path, smallest repair and bounded regression; or
- a bounded no-finding statement with exact inspected scope and explicit exclusions.

It is preferable to consolidate these surfaces into **one reachable-tree review note** rather than manufacture six intermediate commits.

Hard bounds:

- no cryptanalysis claim;
- no new handshake/cipher/KDF design;
- no D019 retention/TTL/LRU/history-size invention;
- no new security capacity/concurrency numbers;
- no core Session/Carrier/ACK/wire architecture change;
- no claim that a bounded no-finding review is independent security approval.

If source changes during this rebuilt review, commit/push first, move the review anchor to the new reachable exact tree, run the focused regression, then run the full local exact-tree gate again before making closure claims.

Wire/parser/crypto framing source changes that fall under the current fuzz policy require the pinned decode fuzz smoke. No decoder/framing change means no new fuzz requirement beyond the ordinary gate.

**When complete, continue immediately to READY_LOCAL 4.**

## READY_LOCAL 4 — repair item-4 evidence/index truth

After the reachable review anchor is green:

1. update `docs/release-security-review-packet.md` and `docs/reviews/release-item4-subgates-20260909.md` so the unpublished `1db7a97` / `7275062` / `c5d0b15` / `58b5d13` / `7711162` / `1be290d` chain is no longer described as accepted reachable exact-tree evidence;
2. remove the unsupported `BrokenPipe` semantic claim from current factual navigation;
3. old local-note files may remain as historical developer reports, but mark/index them as superseded/unverified rather than accepted exact-tree evidence;
4. point current item-4 factual support to the new reachable exact-tree review/provenance;
5. distinguish developer-local exact-tree CI, GitHub-hosted cross-evidence, bounded review support, live WAN evidence and performance claims;
6. do not mark item 4 complete and do not change RC/production/freeze/release flags.

Run the repository's existing release/status/link policy checks that apply, then the normal final docs-tree local closure:

```bash
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --short
```

Do not create a new evidence framework/checker merely for this repair.

## READY_LOCAL 5 — post-HIGH bounded dependency/safety factual sweep

Only after the provenance HIGH is closed, perform one small factual review of the current Rust dependency/safety boundary **if no equivalent current review already exists**:

- confirm workspace `unsafe_code = "forbid"` remains effective for project crates;
- inventory direct security-sensitive/crypto dependencies and the exact locked versions/features actually used by the current tree;
- verify current documentation does not claim a different crypto/library identity than the executable dependency graph;
- identify any current dependency feature/default-feature choice that directly contradicts an existing repository security invariant.

This is a factual supply-chain/API-surface review only. Do not invent signing/key-custody/SBOM policy, do not introduce `cargo-deny`/new scanners/frameworks solely to create work, and do not claim CVE completeness without an authoritative vulnerability source/tool actually used and recorded.

Concrete existing-semantics defect -> smallest repair/regression -> push -> exact-tree local gate. No concrete defect -> one bounded note or no commit if there is nothing useful to retain.

## CHECKPOINT 6 — classify genuine remaining blockers

If READY_LOCAL 1-5 close with no new concrete repairable defect, the coding/review-support queue may genuinely be exhausted. Do not manufacture 6-10 fake slices merely to stay busy.

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

1. **READY_LOCAL:** inline/finalize the terminal-close test seam.
2. **EXACT-TREE CLOSURE:** focused repeated test + full multistream + clean reachable exact-tree `scripts/check.sh` + `git diff --check` + provenance.
3. **EVIDENCE HIGH REBUILD:** re-run bounded item-4 support review on a reachable pushed exact tree.
4. **INDEX REPAIR:** quarantine/supersede unreachable old anchors and correct release/item-4 navigation.
5. **POST-HIGH FACTUAL SWEEP:** bounded dependency/unsafe/security-sensitive dependency consistency check if not already covered.
6. **CONDITIONAL REPAIR:** any concrete current-semantics defect discovered above.
7. **CHECKPOINT:** otherwise record genuine queue exhaustion and leave policy/environment/independent-decision gates open.

Do not pause for GitHub-hosted CI availability. Stop earlier only for an unresolved correctness/security/evidence BLOCKER/HIGH, core architecture decision, destructive/canonical-meaning migration, action outside standing authorization, production/third-party action, new credential/server permission, maintainer-valued benchmark/load conditions, repository breakage or actual runtime/tool-budget exhaustion.
