# ChatGPT reviewer handoff — challenge package-gate provenance and archived-mode validation

## Reviewed state

- Previous reviewer commit/handoff: exact `f302545d905d55a42aafb2ab177c3d7be2bc1ad5` (`docs(handoff): accept provenance repairs and harden package gate`).
- Current default `main` reviewed here: exact `7ee10f8d87dcd833ce673e71dc639021757755ec` (`docs: close package archive validation`).
- Four developer-owned commits landed after the previous handoff:
  - `c7f333b0232da3e987c04c5d3aba59424ead956a` — retain a five-record sanitized phase anchor for the distinct-version A(old) -> B(current-at-run) -> A(old) rehearsal;
  - `e8ee3f8273523feb0bdb54b60dc308c6ef805e65` — add package archive shape/type/path/layout/checksum/mode regressions and wire them into `scripts/check.sh`;
  - `61264ddc4e5bc864f6612084d3153356daaff8ff` — document the package archive gate;
  - `7ee10f8d87dcd833ce673e71dc639021757755ec` — reconcile implementation-plan/release-review package claims.
- Exact-head Rust CI is green for the implementation/test commit `e8ee3f8` (`34290143174`) and for current `7ee10f8` (`34290761181`).
- `work/e1a-staged-accounting-20260907` is 37 commits behind current main and has no unique commit. `work/continue-20260904` still has one ancient unique commit (`d271a99`) but is 121 commits behind current main; its pending-preauth ownership intent has been superseded by later main-line work. Do not coordination-merge either branch.

## Review verdict

### ACCEPT WITH MEDIUM FOLLOW-UP — distinct-version raw anchor improved, but timing provenance is still incomplete

`c7f333b` materially improves the exact `91a735c -> dc90c5f -> 91a735c` rehearsal provenance. The repository now retains the exact five sanitized phase records, their SHA-256 `4e54f3205cf295ec82e9186eb6b6271558e6babddae819c5ef34e1595a0ddb09`, record count `5`, and reported command exit status `0`. The phase content is intentionally small and contains no endpoint IP/private-topology/secret material.

Keep the bounded operator conclusion: genuinely different A/B package+binary hashes, one authenticated TCP 32-byte exchange at each A1/B/A2 stage, final selected release back on A, external temporary state marker retained, and cleanup recorded. Do not promote this to arbitrary state-schema compatibility, production upgrade policy, release or security approval.

However, the prior handoff and standing WAN evidence policy require the experiment to remain associateable with **start/end time** as well as result/cleanup. The retained five-line anchor has no timestamps, and the narrative does not supply actual experiment start/end UTC. A Git commit timestamp is not a substitute for experiment timing.

Classify the remaining gap as `MEDIUM / TIMING_PROVENANCE_NOT_RETAINED` until one of two truthful outcomes is recorded:

1. if already-existing local sanitized material contains the real start/end timestamps, retain only those minimum non-secret facts; or
2. if no such material remains, explicitly qualify the release-facing claim as a developer-recorded bounded observation whose execution timing anchor was not retained.

**Do not rerun the VPS rehearsal merely to manufacture timing provenance.**

### ACCEPT PARTIALLY — archive shape/type/path regression is real and exact-head green

`e8ee3f8` adds a dedicated `scripts/release/smoke-package-test.sh`, wires it into `scripts/check.sh`, and tests path traversal, symlink, hardlink, FIFO, unexpected layout, checksum tamper and one restrictive bad-mode case. The smoke now validates an exact expected archive member set and rejects non-directory/non-regular member kinds before extraction. Exact `e8ee3f8` CI and current exact `7ee10f8` CI are green.

This closes the earlier broad `RELEASE_TOOLING_GAP` for **shape/type/path/layout regression presence**. It does not yet justify saying the archived permission metadata is fully validated fail-closed.

### HIGH / EXACT_TREE_PROVENANCE_DRIFT — item-4 review now verifies a post-38ea package gate while still claiming exact `38ea310`

Current `docs/reviews/release-item4-subgates-20260909.md` is explicitly scoped as developer-prepared factual review support for exact `38ea31057f91eba60baa5a4c4e1b76515d6e65fa`, and its `Exact-tree gates` section says `scripts/check.sh` / `git diff --check` passed on that exact tree. But current text now includes the new package archive-validation implementation/regressions introduced only later at exact `e8ee3f8`.

The release packet has the same provenance shape: its global header says evidence/gates were rerun through reachable exact `38ea310`, while its package row now indexes and describes `scripts/release/smoke-package-test.sh` and the post-38ea fail-closed archive regression.

Therefore the **older 38ea-reviewed facts remain usable**, but the new package-validation statement is not an exact-38ea verified subgate fact. Current `7ee10f8` CI being green proves the current repository gate; it does not retroactively make the new code exist at `38ea310`.

Repair this first as a small truth/provenance slice. Do not delete the useful package tests and do not invalidate unrelated item-4 facts. Either temporarily mark package-validation as post-38ea evidence pending a refreshed exact-tree review, or use a per-subgate tested-tree anchor. After the mode-validation repair below is committed and green, refresh the package subgate against that real reachable implementation/test commit in a separate review-text commit.

### MEDIUM / RELEASE_TOOLING_MODE_VALIDATION — post-extraction `stat` can normalize an insecure archive mode into an accepted mode

The new smoke claims/review text say insecure package modes fail closed, but the implementation currently checks executable/document modes **after** extraction with:

```text
tar -xzf ... --no-same-permissions
stat -c %a ...
```

That does not validate the archived mode exactly. `--no-same-permissions` applies the extracting process's umask. Under the common `umask 022`, an archive member recorded as `0666` extracts as `0644`, and an archived executable recorded as `0777` extracts as `0755`; the later `stat` therefore sees the policy value and can accept metadata that the release builder itself never emits. The existing negative test uses `0600`, which survives the umask and therefore does not catch this direction of normalization.

This is a release-tool correctness/evidence defect, not evidence of a runtime protocol exploit: the extraction step is normalizing permissions downward. But the current exact-mode/fail-closed claim is too strong and the validator is caller-umask-dependent.

The coding agent should repair this without new policy invention. Validate archive header modes **before extraction** against the already-existing builder contract:

- package root/directories: `0755`;
- `bin/neko-cli`: `0755`;
- documentation files: `0644`;
- `SHA256SUMS`: `0644`.

The agent may choose a robust GNU-tar metadata parse or a tiny local helper (for example Python `tarfile`) under the proposal protocol. Prefer the shape least dependent on human-formatted `tar -tv` output. Keep the existing post-extraction safety checks if useful, but they must not be the only proof of archived mode.

Add negative regressions that run under a controlled ordinary umask and prove at minimum:

- archived binary `0777` rejects even though extraction would normalize it to `0755`;
- archived doc `0666` rejects even though extraction would normalize it to `0644`;
- archived `SHA256SUMS` `0666` rejects;
- a directory with unexpected permissive mode rejects if exact builder-layout modes are enforced;
- the existing valid archive and restrictive-mode/path/link/type/layout/checksum negatives remain green.

No network activity, signing/key policy, Session/Carrier/ACK/crypto/wire change, or VPS rerun belongs in this slice.

### ACCEPTED CONTINUING BOUNDARIES

- RSEC bounded non-policy engineering controls remain reviewed; full D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`. Do not invent TTL/LRU/history capacity/external authority or weaken no-reset semantics.
- HY2 exact `13da094` remains frozen `BLOCKED_HARNESS_CURRENT_LINE_HY2` at typed `unknown / client_started`; no complete pair or performance result and no unchanged retry.
- repeated warm failover exact `f17b648` remains frozen at primary `startup_setup`; periodic remains an orchestration negative. No unchanged retries.
- exact package lifecycle and the distinct A/B/A scenario are already answered bounded operator questions; no same-class VPS rerun for documentation polish.
- IPv6 remains environment-blocked if no owned IPv6 path exists.
- live PMTUD still requires its separately accepted authenticated wire/security design gate before live integration; stale `BLOCKED_IMPLEMENTATION` wording is not permission to change wire semantics.
- release flags remain `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`.

## Design / proposal protocol

No administrator approval is needed for the provenance/mode work below. It does not change core protocol semantics, introduce security numeric policy, perform destructive migration, or exceed standing authorization.

For archived-mode validation, the coding agent should briefly compare 1–3 minimal shapes if necessary, then implement the smallest robust one. Reviewer cares about the invariant and negative tests, not a pre-approved helper/API signature.

Do not expand this into signing, key custody, SBOM, artifact transparency service, generic package framework, or a new audit parser.

## Rolling queue

The queue remains intentionally shorter than 6–10 hours because current live/WAN rows are either answered, frozen or blocked and the release packet itself says there is no current live `OPEN_READY` row. Complete A–D continuously; do not wait for an hourly boundary between commits.

### A. HIGH / READY_LOCAL — remove the post-38ea exact-tree claim drift

**Goal:** make the current release packet and item-4 review truthful before adding more release claims.

**Why now:** they currently present post-38ea package-validation behavior inside a review whose tested-tree anchor is exact `38ea310`.

**Files:** `docs/release-security-review-packet.md`, `docs/reviews/release-item4-subgates-20260909.md`; only other status/release docs if needed to remove the same exact-tree ambiguity.

**Behavior:** preserve all older exact-38ea facts; mark package archive validation as later evidence pending refreshed exact-tree factual review, or use an explicit per-subgate tested-tree anchor. Do not call the entire item-4 packet invalid and do not claim independent review.

**Tests/gates:** evidence/governance checks, `scripts/check.sh`, `git diff --check`, commit/push, exact-head CI green.

**Commit/push:** yes for a real truth delta. Continue immediately to B.

### B. MEDIUM / READY_LOCAL — validate archived modes before extraction and regression-test umask normalization

**Goal:** make package mode validation deterministic and actually about archive metadata rather than the caller's extraction umask.

**Files:** `scripts/release/smoke-package.sh`, `scripts/release/smoke-package-test.sh`; tiny helper only if it is the least fragile implementation. Update release docs only after behavior is green.

**Protected invariants:** existing exact member shape/type/path checks, exact builder-mode contract, checksum verification, native `capabilities --json` / `secret_free=true`, cross-target execution skip, no secret/identity/network behavior.

**Required negatives:** permissive archived modes (`0777` executable, `0666` doc, `0666` checksum, and directory-mode mismatch if exact directory modes are checked) must reject under a normal umask; existing valid/restrictive/path/link/type/layout/checksum cases remain green.

**Tests/gates:** focused package smoke first, then `scripts/check.sh`, `git diff --check`, commit/push, exact-head CI green.

**Commit/push:** yes. Continue immediately to C.

### C. MEDIUM / READY_LOCAL — close or qualify the distinct-version timing provenance gap

**Goal:** satisfy standing evidence truth without rerunning an answered VPS scenario.

**Action:** inspect only already-existing local sanitized result material. If real experiment start/end UTC timestamps still exist, retain the minimum non-secret timing anchor and update the evidence hash/count as needed. If they do not exist, state explicitly that timing provenance was not retained and downgrade only the strength of the release-facing provenance wording.

**Forbidden:** new VPS run, guessed timestamps, using Git commit time as experiment time, endpoint IP/private topology/secret material.

**Commit/push:** only for a real retained-timing or qualification delta. Continue immediately to D.

### D. READY_LOCAL / FACTUAL CLOSURE — refresh package item-4 review on a real post-fix exact tree

**Goal:** after B is committed and exact-head green, create a developer-prepared factual package/release subgate review whose tested-tree anchor actually contains the archived-mode fix and package regressions.

**Shape:** use the real reachable implementation/test commit from B as the tested tree; run `scripts/check.sh` and `git diff --check` on that exact tree before the separate review-text commit. The packet may say later text updates index that tested tree, but must not attribute future code to it.

**Rules:** package validation remains release-tool evidence, not signing/publication trust/security approval; D019/HY2/repeated/periodic/IPv6/PMTUD boundaries remain unchanged; items 3/4 and RC remain open.

**Commit/push:** yes for the review/provenance closure. Continue immediately to E.

### E. CONDITIONAL REVIEW SUPPORT — one concrete release claim only

After A–D are green, re-read exact-current item-4 packet plus current code/tests. If one concrete correctness/security/release claim is materially weaker than its wording and can be repaired inside existing architecture/policy, implement the smallest repair + negative test. If no concrete defect exists, skip this slice. Do not create generic checker/parser/harness scaffolding.

### F. CONDITIONAL VPS / OUTPUT SELECTION — currently no honest READY live row

The current release packet classifies the live actionable row as none. Preserve that instead of consuming the rental window with duplicate work.

After A–E, re-read `docs/status.md`, `IMPLEMENTATION_PLAN.md`, the Era-4 ledger and standing authorization. Run a VPS task only if a **new concrete unresolved real-network question** has a satisfied dependency plus a material code/config/instrumentation/path hypothesis. Do not select unchanged HY2, repeated failover, periodic, package lifecycle, distinct A/B/A, generic extra soak, IPv6 without environment, or live PMTUD before its wire/security design gate.

If no such question exists, leave the live queue empty. The VPS being time-limited is a reason to prioritize valuable evidence, not to manufacture redundant evidence.

### G. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Current engineering controls remain useful, but terminal source-retention semantics require a maintainer/security-policy decision. Do not invent TTL/LRU/history capacity, external retention authority or weakened no-reset semantics. This does not block A–F.

## Visible-output check

The last 24–48 hours still contains real output: current-package build/install evidence, ordered externally visible shutdown lifecycle, real installed-package restart/rebind evidence, a genuine distinct-binary A/B/A rehearsal, and now a real package archive adversarial gate. This is not an audit-only period.

The package-smoke work remains directly justified because it protects an operator/release artifact already promoted into the item-4 evidence packet. After the archived-mode defect and exact-tree review drift are closed, do not keep growing package checkers unless another concrete release defect is found.

## Stagnation check

This is **not** `STALLED_IMPLEMENTATION`: four new developer-owned commits landed after the prior handoff, including a substantive package-validation test/tooling change, and exact-head CI is green. The current issues are review findings on the new work, not lack of implementation progress.

## Maintainer/admin boundary

No administrator action is required for A–F within the constraints above. Do not rewrite published Git history autonomously. D019 remains the known maintainer/security-policy checkpoint. Signing key custody/publication trust, any new security numeric policy, core wire/session/crypto changes, production/service-manager mutation outside the standing experimental boundary, or destructive migration still require explicit maintainer action.
