# H-I4-120 follow-up — rollback landed, closure still incomplete

**Severity:** BLOCKER remains open — core ACK/PTO governance + exact-tree evidence reliability

**Exact repository anchor reviewed:** `71f54a453be05f6c320fdb2dbcb08815f1a098dd`

## What changed since the original finding

The developer landed `9e79e6b9e2c69be9a99617f7ab0b8415864de679`, which correctly restores production `Recovery::on_ack` to the authorized pre-H-I4-119 baseline: `pto_count` resets when the newly acknowledged sent-packet set is non-empty. This production rollback is acceptable as a governance rollback and does not resolve H-I4-119.

The repository also landed an erratum at `f2f33b2384d4e39042ac64162410c26a8129fbc4` and a new developer-local provenance note at `71f54a453be05f6c320fdb2dbcb08815f1a098dd`.

## Why H-I4-120 is not closed yet

The original closure contract required the stale-ACK regression to remain semantics-neutral while H-I4-119 is unresolved. Exact-current `crates/neko-reliable/src/lib.rs` instead contains `non_ack_eliciting_ack_resets_pto_baseline`, which explicitly asserts `r.pto_count == 0` after acknowledging only the non-ack-eliciting packet. That pins one side of the disputed H-I4-119 rule in the test suite even though comments say the gate remains unresolved.

The test may preserve the corrected timing and assert that the older ack-eliciting packet remains outstanding, but it must not assert either disputed PTO-reset outcome until a maintainer/spec decision lands.

The new provenance note `docs/notes/check-gate-f2f33b2-20260924.md` is also insufficient under the repository evidence contract. It records an exact reachable SHA, command, clean tree, `git diff --check`, exit 0 and OS/arch, but:

- its UTC end is approximate (`~2026-09-24T10:5x:xxZ`) rather than an exact UTC end timestamp;
- it does not record the stable Rust version required by the handoff contract.

Therefore it cannot yet serve as the accepted H-I4-120 closure provenance. GitHub combined status and workflow-run lookup for `f2f33b2...` currently expose no hosted status/run entries, so no hosted pass/fail is inferred.

Finally, the old-note erratum says the unauthorized semantic change “was rolled back by `4f53eb817680c531f332e8da9d5e5aff828b93b3`.” That SHA is the earlier provenance commit, not the actual rollback. The rollback is `9e79e6b9e2c69be9a99617f7ab0b8415864de679`. The erratum must be corrected/superseded rather than treated as factually clean evidence.

## Required closure from current main

1. Keep the production rollback in `9e79e6b...` unchanged unless a maintainer/spec decision resolves H-I4-119.
2. Rewrite `non_ack_eliciting_ack_resets_pto_baseline` so the corrected timing remains covered without asserting either disputed PTO-reset result. Preserve the useful assertion that the older ack-eliciting packet remains outstanding and no stale/time-threshold loss occurs under that timing.
3. Correct or supersede the erratum statement so it names `9e79e6b...` as the rollback commit; do not silently rewrite the historical provenance narrative.
4. On the final reachable pushed source/test SHA, rerun `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` and `git diff --check`, verify clean tree, and persist exact SHA, exact UTC start and end, both exit codes, OS/arch, and stable Rust version. Do not reuse the incomplete `f2f33b2` note as full closure evidence.
5. Leave H-I4-119 explicitly unresolved as a maintainer/core ACK-PTO semantics gate. Do not change PTO thresholds, persistent-congestion policy, D019, Session/Carrier/wire/crypto semantics or release flags.
6. After the exact-tree closure is reachable, immediately continue the existing post-H120 Recovery delta re-challenge, thirteen-surface owner inventory refresh and release-packet reconciliation. No decoder/framing change is involved, so no mechanical decode fuzz is required.

## Evidence boundary

This follow-up is exact-current GitHub source/control-flow, repository-governance and evidence-record review. It is not reviewer-local Rust/full-gate execution, hosted CI, fuzz, WAN, cross-platform, adversarial-load or performance evidence.
