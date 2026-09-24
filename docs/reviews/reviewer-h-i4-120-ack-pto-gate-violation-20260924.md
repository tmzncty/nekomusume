# H-I4-120 — H-I4-119 ACK/PTO maintainer-gate violation and unreachable provenance

**Severity:** BLOCKER — core ACK/PTO semantics / release-evidence reliability

**Exact repository anchor reviewed:** `4f53eb817680c531f332e8da9d5e5aff828b93b3`

## Finding

The repository had already reclassified H-I4-119 as a **MAINTAINER / CORE ACK-PTO SEMANTICS GATE**. At reachable handoff `c107aa71fdcf23004d25e078ec52134951ad62ab`, the explicit rule was: do not auto-patch `Recovery::on_ack` until a maintainer/spec decision chooses whether `pto_count` is reset by any newly acknowledged sent packet or only by newly acknowledged ack-eliciting packets.

Despite that gate, reachable developer commit `8d0a6e3a2824cd8338fff5a0cdb995107f0e114e` changes production `Recovery::on_ack` by introducing `any_ack_eliciting` and replacing the prior `if !out.acked_packets.is_empty()` reset condition with `if any_ack_eliciting`. The same commit adds `non_ack_eliciting_ack_does_not_reset_pto`, which asserts the gated project-specific semantic. This is not merely the stale-ACK timing-only change described by the commit title.

No current repository decision/spec amendment was found that selects this core semantic. `docs/decisions.md` and the provisional Session spec remain unchanged with respect to this PTO-reset choice, while `docs/CHATGPT_HANDOFF.md` still says H-I4-119 is unresolved and must not be auto-patched.

Therefore the exact-current tree has crossed a maintainer/core-semantics stop condition without the required decision.

## Evidence integrity subfinding

Reachable provenance commit `4f53eb817680c531f332e8da9d5e5aff828b93b3` records a developer-local exact-tree gate for SHA `5c2c558fab6521bfb1ebb3a89c0f49ae95604c47` and describes it as a **test-only** change with no production recovery/ACK/PTO semantic movement.

That SHA is not GitHub-resolvable from the connected repository, so it does not satisfy the repository rule that accepted exact-tree provenance must anchor a reachable pushed commit. The reachable source/test commit with the same stale-ACK timing narrative is `8d0a6e3a...`, and its exact diff contains the production H-I4-119 semantic change above. The existing provenance therefore cannot be accepted as shared exact-tree evidence for the current tree and its scope description is factually wrong for the reachable commit.

## Required closure

This finding cannot be closed by choosing the stronger ack-eliciting-only rule. That choice is still a maintainer/spec decision.

Absent an explicit maintainer/spec decision committed to repository truth, the dependency-ready closure is a **governance rollback to the last authorized pre-gate semantics**, not a new semantic choice:

1. Restore `Recovery::on_ack` to the pre-H-I4-119 behavior present at the policy-gate anchor: `pto_count` reset on any non-empty newly-acked sent-packet set.
2. Remove or rewrite the policy-selecting regression so it does not assert the disputed reset rule. The useful stale-ACK timing regression may remain only if it is semantics-neutral (for example, proving the older ack-eliciting packet remains outstanding after the newer non-eliciting ACK under the chosen timings).
3. Keep H-I4-119 explicitly unresolved as a maintainer/core ACK-PTO semantics gate. Do not characterize the rollback as selecting rule (1) permanently; it only restores the authorized baseline pending a decision.
4. Add an erratum/superseding provenance note for `docs/notes/check-gate-5c2c558-20260924.md`. Do not silently rewrite the historical record. State that the recorded SHA is unreachable and the reachable replacement tree must be validated separately.
5. On the final reachable pushed source/test SHA, run the developer-local clean exact-tree gate required by the handoff: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, and persist exact SHA / UTC / exit codes / OS-arch / stable Rust version.
6. After closure, rerun the thirteen-surface owner inventory because `neko-reliable` moved after the prior `68d9382` inventory. Resume packet maintenance only after this BLOCKER is no longer present.

No wire/parser/crypto-framing change is involved; do not run decode fuzz mechanically.

## Stop / scope boundaries

- Do not decide the final H-I4-119 ACK/PTO rule here.
- Do not change PTO thresholds, persistent-congestion policy, D019, Session/Carrier architecture, wire/crypto semantics, or release flags.
- Do not treat the unreachable `5c2c558...` provenance as accepted shared exact-tree evidence.
- `READY_LIVE` remains `none`; this finding creates no new WAN question.

This is exact-current GitHub source/control-flow and repository-governance review. It is not reviewer-local Rust/full-gate execution, hosted CI, fuzz, WAN, cross-platform, adversarial-load or performance evidence.
