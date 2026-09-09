# ChatGPT reviewer handoff — accept package hardening, repair exact-tree provenance

## Reviewed state

- Previous reviewer commit/handoff: exact `82030a6f9eef406a725c33befd6edba5d8855a7b` (`docs(handoff): challenge package gate provenance and modes`).
- Current default `main` reviewed here: exact `d15a50a627720a638b4a532d4a82c04bf38dad95` (`docs: refresh package review provenance`).
- Three developer-owned commits landed after the previous handoff:
  - `54078364735976b472899decf2b456ec878c1739` — validate package root and checksum-manifest shape before extraction;
  - `8cbf3afa276f1fb2a4e991b3ad94daddc4d8cf64` — validate exact archive member types/modes before extraction and add umask-normalization regressions;
  - `d15a50a627720a638b4a532d4a82c04bf38dad95` — refresh release packet/item-4 package provenance text.
- GitHub-hosted Rust CI run `34294307935` is green on exact `d15a50a`: stable checks ran `bash scripts/check.sh`, and the separate pinned nightly decode fuzz smoke also passed. This is **GitHub-hosted cross-evidence**, not developer local-CI provenance.
- No GitHub-hosted workflow run exists for exact `8cbf3afa`; this is not a blocker under the local-CI-first policy because exact `d15a50a` contains the implementation and is independently green on GitHub. For all new READY_LOCAL slices below, local exact-tree verification is the required ordinary closure path; do not wait for GitHub Actions quota/runs.
- This reviewer did not execute the developer's local CI and the repository currently does not retain a local-CI provenance record for `5407836` / `8cbf3af` / `d15a50a`. Do not retroactively label GitHub Actions as local CI.
- `work/e1a-staged-accounting-20260907` remains at `f4404257`; `work/continue-20260904` remains at `d271a99a`. Both are stale relative to `main`; do not coordination-merge them merely to create work.

## Review verdict

### ACCEPT — package-root and checksum-manifest pre-extraction validation

Exact `5407836` closes the prior root/layout ambiguity without changing package format, runtime protocol semantics or security policy. The smoke now accepts only the declared Nekomusume target-root forms, rejects wrapper/prefix/traversal root shapes, and checks the checksum manifest against the exact expected payload paths. New regressions cover traversal, absolute, duplicate and unexpected checksum entries.

Keep this as release-tool correctness evidence only. It is not signing, publication trust, SBOM, security approval, RC or production evidence.

### ACCEPT — archived member type/mode validation is now pre-extraction and umask-independent

Exact `8cbf3af` fixes the previous mode-provenance defect. `scripts/release/smoke-package.sh` now reads archive metadata before extraction and requires the existing builder contract exactly:

- package directories: `0755`;
- `bin/neko-cli`: `0755`;
- documentation and `SHA256SUMS`: `0644`;
- only directory/regular-file member types at the exact expected paths.

The regression fixture now includes permissive archive metadata that an ordinary umask could otherwise normalize after extraction (`0777` executable/directory and `0666` document/checksum cases). Existing path/link/special-member/layout/checksum/restrictive-mode cases remain in the gate.

Do not reopen this archived-mode gap absent a concrete new counterexample. The current package checker is already directly justified by an operator/release artifact; after the provenance work below, do not keep growing it speculatively.

### HIGH / EXACT_TREE_PROVENANCE_INVALID — `d15a50a` names a nonexistent tested tree

Current `docs/release-security-review-packet.md` and `docs/reviews/release-item4-subgates-20260909.md` state that evidence/gates were rerun on reachable exact commit:

`9d4c7e1e0fc41b46396fb635ee9ca413b4b1fc67`

GitHub cannot resolve that SHA in this repository. Therefore the words **reachable exact commit** and the exact-tree gate claim are currently false. The substantive package implementation at `5407836` / `8cbf3af` remains accepted; the defect is the release-review provenance anchor, not the package behavior itself.

Repair this first. Do not invent another SHA, do not use a local/unpushed object, and do not treat a future documentation commit as proof that an earlier tree contained future code. Prefer a real pushed exact implementation/test tree that already contains the package fixes. Under the local-CI-first policy, the coding agent should check out that exact SHA in a clean temporary worktree/clone, run the local stable gate there, retain minimal non-secret provenance, then make the separate review-text commit point to that real tested tree.

### MEDIUM / RELEASE_BUILD_SOURCE_PROVENANCE_GAP — package builder can label a dirty build as exact HEAD

Current `scripts/release/build-package.sh` emits `git_commit=$(git rev-parse HEAD)` in build evidence but does not first prove that the repository source state is clean. A staged/unstaged tracked edit, or an untracked non-ignored build-affecting file, can therefore influence the produced binary/archive while the JSON still names the unchanged HEAD.

This is a release-tool provenance/correctness gap; there is no evidence that the already-recorded package experiments were actually built dirty, so do not retroactively invalidate them without such evidence.

The smallest repair is a fail-closed source-state guard **before** build/output mutation. The agent may choose the exact implementation shape, but the invariant is:

- exact-tree package evidence must not be emitted from staged or unstaged tracked changes;
- non-ignored untracked repository files must not silently become build inputs while the evidence still claims only HEAD;
- ignored build output such as `target/` must not make a clean source checkout impossible to package;
- the guard must run before `mkdir -p "$OUT"`, Cargo build, or other commands that themselves may create non-ignored output.

Add focused negative tests for dirty tracked/staged/untracked state and a clean-tree positive. Do not turn this into a generic CI framework, signing system or source-transparency service.

### ACCEPTED CONTINUING EVIDENCE BOUNDARIES

- The exact `91a735c -> dc90c5f -> 91a735c` distinct-package rehearsal retains its five sanitized phase records, result hash, count and exit status. Release-facing text now truthfully says experiment start/end timing was **not retained**. This qualifies the provenance gap without manufacturing timestamps; do not rerun that VPS scenario for timing polish.
- RSEC bounded non-policy engineering controls remain reviewed; full D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`. Do not invent TTL/LRU/history capacity, external retention authority, or a weaker no-reset policy.
- HY2 exact `13da094` remains frozen at typed `unknown / client_started` with no complete pair or performance result. No unchanged retry.
- repeated warm failover exact `f17b648` remains frozen at primary `startup_setup`; periodic remains an orchestration negative. No unchanged retry.
- installed-package lifecycle and the distinct A/B/A package scenario are already answered bounded operator questions; no same-class VPS rerun for documentation polish.
- IPv6 remains environment-blocked when the owned environment does not actually provide the address family/path.
- live PMTUD still requires its separately reviewed authenticated wire/security design gate; old `BLOCKED_IMPLEMENTATION` wording is not permission to alter wire semantics.
- release flags remain `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`.

## Local-CI-first execution rule for the queue below

For ordinary READY_LOCAL implementation/test/docs-evidence work, **do not wait for GitHub Actions** after a coherent developer commit. Use a clean checkout/worktree of the exact pushed developer SHA and run:

```text
bash scripts/check.sh
git diff --check
```

For an already committed exact tree also record that the tree is clean. Persist only minimal provenance when the slice makes an exact-tree/release-evidence claim: exact SHA, commands, UTC start/end, exit code, host/OS/arch, `rustc --version`, clean-tree state, and optionally a sanitized log path/SHA-256. Do not store secrets, endpoint topology or credentials. Fuzz is not required for the package-only slices below unless they unexpectedly touch wire/parser/crypto framing.

A developer-reported local run, a persisted local-CI provenance record, this reviewer's own inspection, and GitHub-hosted CI are four different evidence sources. Keep them labeled accurately.

Reviewer-only handoff commits do not need a new GitHub Actions wait loop. Once a developer exact tree has the required local gate, the coding agent may continue through the dependency-ready queue without waiting for another hourly review or hosted CI run.

## Design / proposal protocol

No administrator approval is needed for A–F below. These slices do not change Session/Carrier/ACK/crypto/wire architecture, introduce a new security numeric policy, perform destructive migration, or exceed standing authorization.

Where an API/helper shape is underspecified, the coding agent should compare at most 1–3 small options and choose the one with the least new state/API/policy and the clearest fail-closed tests. Implement it rather than waiting for a reviewer to dictate a function signature.

## Rolling queue

There is intentionally no READY live run at this checkpoint. The release packet and implementation plan both classify the current live actionable row as none. Complete A–F continuously; do not wait at commit or hourly boundaries when the next slice is dependency-ready.

### A. HIGH / READY_LOCAL — replace the nonexistent exact-tree review anchor with a real locally tested tree

**Goal:** make the item-4 factual review and release packet tell the truth about the tree on which the current package gate was checked.

**Why now:** `9d4c7e1...` is not a reachable GitHub commit, so current exact-tree provenance is invalid even though current GitHub-hosted CI is green.

**Action:** use exact pushed `d15a50a627720a638b4a532d4a82c04bf38dad95` as the preferred tested tree unless a newer real developer implementation commit exists when work starts. In a clean temporary checkout/worktree of that exact SHA, run the local stable gate and clean-tree checks. Persist the minimum local-CI provenance. Then update `docs/release-security-review-packet.md` and `docs/reviews/release-item4-subgates-20260909.md` in a separate commit so title/body/gate statements point only at the real tested SHA. The already-successful GitHub-hosted run `34294307935` may be cited separately as cross-evidence, never as the local run.

**Protected boundary:** developer-prepared factual support only; not independent review/security approval/RC/release/production authorization.

**Commit/push:** yes. After local closure, continue immediately to B without waiting for GitHub Actions.

### B. MEDIUM / READY_LOCAL — make `build-package.sh` fail closed on dirty source provenance

**Goal:** ensure a release-build evidence JSON naming commit X actually comes from source state equal to X.

**Files/concepts:** `scripts/release/build-package.sh`, focused release-script regression; a tiny helper is allowed only if simpler than duplicating shell plumbing.

**Required behavior:** before build/output mutation, reject staged changes, unstaged tracked changes, and non-ignored untracked repository files. Do not reject ordinary ignored `target/` build output merely for existing. Preserve current `--locked`, target restrictions, deterministic tar/gzip behavior, builder modes, hashes and build-evidence schema unless a schema change is strictly necessary.

**Required tests:** clean-tree positive; dirty tracked negative; staged negative; untracked non-ignored negative; ensure rejection occurs before a package/evidence success can be emitted. Keep the tests bounded/local and avoid modifying the developer's primary worktree destructively.

**Gates:** focused release tests, then exact-commit local `bash scripts/check.sh`; clean tree and diff checks. No fuzz needed unless the slice unexpectedly touches parser/wire/crypto framing.

**Commit/push:** yes. Continue immediately to C.

### C. READY_LOCAL / EVIDENCE CLOSURE — persist exact-commit local-CI provenance for the package-source guard

**Goal:** make the local-CI-first policy auditable without creating another CI system.

**Action:** after B is pushed, verify the exact B commit from a clean worktree/clone. Record exact SHA, commands, UTC start/end, exit, host/OS/arch, Rust stable version and clean-tree state in the smallest existing release/review evidence location or one small focused note. If a sanitized log is retained, record only its path/hash; do not commit large logs.

**Failure rule:** a local gate failure is a real failure. Repair and rerun before claiming closure. Do not wait for GitHub-hosted CI to substitute for this step.

**Commit/push:** only for the real provenance/evidence delta. Continue immediately to D.

### D. READY_LOCAL / FACTUAL CLOSURE — refresh item-4/package review against the actual post-fix tested tree

**Goal:** leave one internally consistent package/release subgate packet after the dirty-source repair.

**Action:** point the package validation/build-provenance statements at the real reachable exact B/C implementation/test tree that was locally verified. Preserve the distinction between package-tool correctness, bounded operator evidence, and signing/publication/security approval. Keep the distinct A/B/A timing qualification as `not retained` unless already-existing local sanitized material supplies real experiment timestamps; never invent them or use commit time as experiment time.

**Commit/push:** yes for factual closure. Do not wait for hosted CI merely because this is a reviewer-facing doc update.

### E. CONDITIONAL / READY_LOCAL — one concrete release correctness defect, not another framework

After A–D, re-read the exact-current item-4 packet plus current release scripts/code/tests. If **one specific claim** is materially stronger than its implementation/evidence and the repair stays within existing architecture/policy, implement the smallest repair plus negative test. If no concrete defect exists, skip E. Do not create generic checker/parser/harness scaffolding or signing/SBOM/key-custody machinery just to keep the queue busy.

A useful candidate may come from build/package provenance or operator lifecycle, but it must be demonstrated from current code before becoming a ticket.

### F. LOCAL CLOSURE / MILESTONE RECONCILIATION — stop package-checker growth when the concrete gates are closed

**Goal:** reconcile `docs/status.md`, `IMPLEMENTATION_PLAN.md`, release packet and item-4 review so the exact-current package/tooling capability and remaining boundaries match. If all package-specific correctness claims now have direct tests/evidence, mark that local package closure complete and move on; do not keep adding archive/checker variants without a new finding.

**Boundary:** item 3 release evidence and item 4 independent review remain open where their actual external/independent requirements remain unmet. `IMPLEMENTATION_COMPLETE=true` does not promote RC/release/production/freeze flags.

### G. CONDITIONAL VPS / OUTPUT SELECTION — currently `READY_LIVE: none`

After A–F, re-read `docs/status.md`, `IMPLEMENTATION_PLAN.md`, the Era-4 ledger, release packet, standing authorization and VPS rental policy. Execute a VPS task only if a **new concrete unresolved real-network question** now has satisfied dependencies and a material code/config/instrumentation/path hypothesis.

Do not select unchanged HY2, repeated warm failover, periodic, installed-package lifecycle, distinct A/B/A, generic extra soak, IPv6 without an actual environment, or live PMTUD before its authenticated wire/security design gate. If no new live question exists, leave the VPS queue empty. A time-limited VPS is a reason to maximize evidence value, not to manufacture duplicate evidence.

### H. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Current engineering controls remain useful, but terminal source-retention/no-reset semantics still require a maintainer/security-policy decision. Do not invent TTL/LRU/history capacity, an external retention authority or weakened lifetime semantics. This policy blocker does not stop A–G.

## Visible-output check

This is not an audit-only period. The last 24–48 hours includes current-package build/install evidence, externally visible shutdown/restart behavior, a real distinct-binary A/B/A rehearsal, and substantive package archive fail-closed implementation/tests. The package work remains justified because it directly protects an operator/release artifact already indexed in item-4.

After the exact-tree and dirty-source provenance defects are closed, deliberately stop expanding package validation unless another concrete defect is demonstrated. The next outward work should come from a real remaining release/runtime/evidence question, not from checker self-proliferation.

## Stagnation check

This is **not** `STALLED_IMPLEMENTATION`: `5407836`, `8cbf3af`, and `d15a50a` landed after the last reviewer handoff, including two substantive package-tool/test repairs. The current HIGH is a new provenance defect in the developer's review text, not lack of implementation progress.

## Maintainer/admin boundary

No administrator action is required for A–G within the constraints above. Do not rewrite published Git history autonomously. D019 remains the known maintainer/security-policy checkpoint. Signing key custody/publication trust, new security numeric policy, core wire/session/crypto changes, production/service-manager mutation outside the standing experimental boundary, third-party access, destructive migration, or a genuinely new phase still require explicit maintainer action.
