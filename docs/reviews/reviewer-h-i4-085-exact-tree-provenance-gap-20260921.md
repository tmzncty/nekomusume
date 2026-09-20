# H-I4-085 continuation — truthful dependency/build note lacks final exact-tree provenance

**Repository anchor before this note:** reachable `main` exact `9e483618554acbd3fa3544e9a17e3ec6436b1a6a`.

## Classification

**HIGH — release-item-4 evidence closure / provenance only.**

The substantive dependency/build classification defect is repaired. This continuation does **not** dispute the corrected facts about `snow`/`ring`, does not claim those dependencies are unsafe, and does not request a dependency/feature/crypto-policy change.

## What is now correct

Current `docs/reviews/dev-i4-bld-dependency-build-surface-20260921.md`, repaired by reachable `250c4d2e87f45a2cfcb9dd5abedf6ac7620cf4d8` and `97e282de9f97f061e1b8f5ae36a8cc0cab73d301`, now truthfully separates workspace-member facts from the active transitive production closure:

- `snow 0.10.0` is a normal/default-feature production dependency of `neko-crypto`;
- `snow` ships a build script and uses `rustc_version` as a build dependency;
- active resolved `ring 0.17.14` has its build script, `links = "ring_core_0_17_14_"`, `cc` build dependency, and native C/assembly surface;
- no workspace member itself ships a local `build.rs`/`links` declaration.

The locked Cargo-tree/feature evidence is now persisted. The earlier false “no build/native hooks” statement is superseded.

## Remaining closure gap

The repository contract requires ordinary READY_LOCAL implementation/test/**docs-evidence** slices to carry a developer-local clean exact-tree gate for the final reachable closure tree: at minimum

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
clean worktree
```

with exact reachable SHA, UTC start/end, exit codes, OS/arch, Rust stable version, and clean-tree state.

Current handoff closure text only points back to the developer-reported exact `8cbd9af39a600f4ddd5fbc50208791e8584c6d74` gate. That gate predates the H-I4-085 evidence corrections and therefore does not validate the corrected closure tree. The reachable commits `250c4d2`, `58cb04d`, `97e282d`, `1f13c0d`, `44a0073`, and the later independent FS1 note are docs/evidence/navigation changes; no reachable provenance record found in the current repository asserts the required clean exact-tree gate on the corrected H-I4-085 closure tree.

A historical green executable-source gate cannot be promoted into exact-tree provenance for later evidence changes.

## Minimal closure contract

1. Synchronize to the exact current reachable closure/navigation tree after the reviewer handoff that fronts this gap.
2. In a safe clean checkout/worktree run:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - verify the worktree is clean.
3. Persist provenance for that exact tested SHA: UTC start/end, both exit codes, OS/arch, Rust stable version, clean-tree state, and exact reachable SHA. Do not include secrets, private topology, credentials, or unnecessary absolute paths.
4. A later provenance-only commit may record the test of its reachable parent/tested SHA; do not claim the provenance-only commit itself was tested unless it actually was.
5. No fuzz is required: no decoder/parser/crypto-framing code changed. No live run is relevant.
6. After the provenance anchor is reachable, H-I4-085 may close and the agent should continue immediately to the already-reviewed/next dependency-ready item-4 lanes.

## Evidence boundary

This finding is about exact-tree evidence provenance, not code correctness and not dependency safety. It does not reopen the corrected `snow`/`ring` classification itself. It does not alter release/governance state.

`READY_LIVE: none` remains unchanged.