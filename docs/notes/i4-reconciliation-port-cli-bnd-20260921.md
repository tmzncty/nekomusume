# Item-4 factual reconciliation — PORT/CLI/BND closure group

**Anchor:** exact source/test tree `42be539` + reviewer/dev commits through `dd166b3`.

## Evidence classes preserved

- **Developer-local executable source/test provenance:** `38379b1` (H-I4-085), `3acba04` (H-I4-086), `d819d38` (H-I4-087), `26aa4e8` (H-I4-088), `42be539` (I4-PORT-01) — each carries `check.sh` exit 0 + `git diff --check` exit 0 + clean worktree on its exact pushed SHA.
- **Developer-local bounded review notes:** FS1/FS2/AD1/AD2a/AD2b/BLD/PORT/CLI-M/CLI-H/BND — each is a bounded current-owner challenge, not independent reviewer execution.
- **Reviewer-source-review:** `reviewer-h-i4-085-*`/`reviewer-h-i4-086-*`/`reviewer-h-i4-087-*`/`reviewer-h-i4-088-*`/`independent-i4-*`/`independent-final-r9-*` — reviewer findings/challenges on reachable source.
- **Live WAN:** none — `READY_LIVE: none` throughout; no VPS authorization consumed.
- **Performance:** no capacity-pressure/adversarial-load benchmark run; BND is semantic boundedness only.

## Specific qualifications/supersessions

- **CLI-H "parsed result" wording:** the current `matrix_probe` / reachability gate does **not** deserialize a typed semantic result at the final human/exit branch. It calls `reachability::run(...)` to obtain a JSON artifact string and then tests `artifact.contains("\"reachable\":true")` for both the human success/failure wording and process exit. On the current one-case `reachability-matrix.v1` producer this exact substring test was not falsified by the bounded review, but the stronger phrase “driven by the parsed result” is superseded here. A schema change, multiple cases, or another `reachable` occurrence is a re-review trigger; no source repair is manufactured solely to make the old prose true.
- **H-I4-087 first-run flake:** `check.sh` at `d819d38` produced a transient `fixture.known-fd` `bind` race (`cp.returncode == 1`) on the first run; the second run passed all tests. The flake is timing/port-reuse, not a regression — truthfully recorded.
- **H-I4-085 snow/ring build surface:** `snow` is a normal production dependency with a `rustc_version` build script; `ring` is reached via `snow` default features and carries `links`/`cc`/native surface. Workspace members ship no `build.rs`/`links`.
- **Item-4 status:** remains **incomplete** — release items 3 and 4 are open; `RELEASE_CANDIDATE`/`PRODUCTION_READY`/`FREEZE`/`RELEASED` are unchanged; D019/policy gates remain maintainer-owned.

## Superseded claims

- `dev-i4-bld-dependency-build-surface-20260921.md` at `e11c1d2` — superseded by `c4925b6`/`250c4d2`/`97e282d`/`803bee3` corrections.
- `dev-i4-fs1-fair-scheduler-20260920.md` at `dab0be3` — wording corrected at `9adf1d2` (drained/inert, not close/terminal).
- `dev-i4-fs2-multistream-flow-control-20260921.md` at `051f588` — superseded by H-I4-086/087 repairs.
- `dev-i4-ad1-memorycarrier-close-resource-20260921.md` at `9288b83` — superseded by H-I4-088 record-count bound.
- `dev-i4-port-cross-platform-cli-process-20260921.md` at `11678b2` — superseded by `98978ee`/`42be539` unix-scope repair.

## No release decision

This reconciliation does not mark item 4 complete, does not change release/governance flags, and does not decide D019.
