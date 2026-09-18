# Independent bounded review — H-R9-055 pre-deadline PTO decision owner

## Result

**NO FINDING. H-R9-055 is independently closed at source/test anchor `b41e37a03594e040faeedf4d78ed2115fc10e160`.**

This review is intentionally narrow. It closes the executable just-before-PTO-deadline acceptance oracle reopened by H-R9-055; it is not a release/security approval and does not close release item 4.

## Exact reviewed owners

- `crates/neko-cli/src/main.rs`
  - `due_pto_probe`
  - the post-return reliable-UDP settle loop that calls `due_pto_probe`
  - `cli_regression_tests::due_pto_probe_is_quiet_before_deadline_and_fires_at_it`
- `crates/neko-carrier/src/lib.rs`
  - `ReliableUdpRuntime::pto_probe`
  - the read-only `recovery_engine` / `in_flight` observations used by the regression
- exact implementation/test anchor: `b41e37a03594e040faeedf4d78ed2115fc10e160`
- current docs-only descendant observed at review start: `efd2b4e45f3693211d85d01efa504efa00c00381`

Applicable repository boundaries were re-read from `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, `docs/standing-vps-lab-authorization.md`, `docs/vps-rental-window-priority.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, `docs/release-security-review-packet.md`, and `docs/CHATGPT_HANDOFF.md`.

## Challenged invariant

For the executable post-return PTO owner:

1. at `now_us < next_pto_deadline_us`, no `pto_probe` / `Recovery::on_pto` transition may occur;
2. at `now_us == deadline`, one legitimate PTO probe transition may occur;
3. removing or weakening the production deadline guard must make the focused regression fail;
4. the review must not infer Session delivery, Carrier ACK retirement, WAN behavior, or release readiness from the PTO transition.

## Source challenge

`due_pto_probe` owns both the deadline decision and the mutating call. It first obtains `next_pto_deadline_us(1_000, 0)`, returns `None` while `now_us < deadline`, and only then calls `rt.pto_probe()`. The production post-return settle loop calls this same helper rather than reproducing the predicate separately.

`ReliableUdpRuntime::pto_probe` is the mutating owner below that gate: it calls `PathRecovery::on_pto(4)` and maps scheduled stable frame identities back to retained plaintext. Therefore the early `None` branch is structurally before the PTO mutation; there is no second mutation in `due_pto_probe` that could occur on the pre-deadline path.

The focused regression prepares one outstanding packet, derives the real current deadline from the same runtime, snapshots `pto_count` and in-flight ownership, calls the production helper at `deadline - 1`, and requires `None`, unchanged `pto_count`, and unchanged in-flight ownership. It then calls the same helper at exactly `deadline` and requires `Some` plus a `pto_count` increase. If the `now_us < deadline` guard is removed or weakened, the pre-deadline call reaches `pto_probe`, increments PTO state, and the regression becomes deterministically red. The old weak test that directly invoked `Recovery::on_pto` at `deadline - 1` was removed.

The helper extraction changes the diagnostic observation point from the old inline shape: the caller now emits `r9_udp_pto_fired` after `pto_probe` has returned, so the reported `pto_count` is post-transition. No current spec/ADR/test requires the old pre-transition count, and the post-transition value is consistent with the event name; no correctness finding is opened from this ordering change.

## Evidence truth

Developer-local provenance recorded in the reachable handoff for exact `b41e37a` reports:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` -> exit 0
- `git diff --check` -> exit 0
- clean worktree at the pushed SHA
- Linux x86_64, rustc 1.98.0
- 2026-09-18T11:54:02Z through 2026-09-18T11:58:23Z

GitHub-hosted Rust CI run `35341890289` for exact `b41e37a` completed successfully. Its `stable checks` job ran `bash scripts/check.sh` successfully; its separate nightly decode fuzz-smoke job also succeeded. Hosted CI is cross-evidence only and is not substituted for developer-local exact-tree provenance.

Reviewer-local execution is **not claimed** in this note: the current automation sandbox could not resolve `github.com` for a local clone. The closure is based on exact pushed source/test inspection plus the separately classified developer-local and hosted evidence above.

## Exclusions

Not reviewed or claimed here:

- R9-4 ACK-loss/delayed-original/reorder behavior;
- adversarial Carrier continuations, ownership/resource boundedness, health/promotion, warm readiness, uncertain replay, terminal cleanup, or the final R9 process slice;
- live WAN/HY2/soak/migration-back/IPv6/PLPMTUD evidence;
- D019 retention policy, capacity/security values, signing/SBOM/publication policy, protocol/wire architecture, RC/freeze/release/production authority.

## Queue consequence

H-R9-055 and R9-3 may leave the queue. The next dependency-ready slice is R9-4. The existing R9-4 through R9-12, dedicated independent R9 review, Q10/Q11/Q12 reconciliation, and repository-wide item-4 refill remain real work; repository-wide queue exhaustion is false. `READY_LIVE` remains `none` absent a new concrete real-network question.
