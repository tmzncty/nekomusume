# ChatGPT reviewer handoff — accept provenance repairs and close the package-validation gap

## Reviewed state

- Previous reviewer handoff: exact `899faafc50e5b53c2d049ba4f8eb908d7a482c29` (`docs(handoff): reconcile lifecycle evidence provenance`).
- Current default `main` reviewed here: exact `968503bf8e0349c86242d5222d40e2e6ddf742fb` (`docs: reconcile package closure facts`).
- `main` is exactly four commits ahead of the previous reviewer handoff:
  - `7fce5b86759ed7c3d77f73dc533811e6226ca48b` — record one genuine distinct-package A(old) -> B(current-at-run) -> A(old) VPS rehearsal;
  - `e743c64a4f62498dd98f03d64797acb44cb18fdc` — reconcile the exact-`7cebe6b` lifecycle evidence provenance and add a retained sanitized result anchor;
  - `38ea31057f91eba60baa5a4c4e1b76515d6e65fa` — repair the release item-4 exact-tree provenance to a real reachable tree;
  - `968503bf8e0349c86242d5222d40e2e6ddf742fb` — reconcile package/release/status facts after those repairs.
- Exact-head Rust CI is green for `7fce5b8` (`34284166309`), `e743c64` (`34284519680`), `38ea310` (`34284862193`) and current `968503b` (`34286660635`).
- No work branch is ahead of `main`. `work/e1a-staged-accounting-20260907` remains stale at `f4404257520e9a014ac4e785b0ab9a97f8aaf794`; `work/continue-20260904` remains stale at `d271a99a2ab26abbcb146c411ba0fde697395abe`. Do not coordination-merge either.

## Review verdict

### ACCEPT — exact-`7cebe6b` lifecycle provenance conflict is reconciled

`e743c64` gives the missing chronology instead of hiding the contradictory history. It states that there was exactly one materially changed VPS invocation, that the command exit status was `0`, that exact `e84770e` incorrectly reported an incomplete/nonzero prefix, and that exact `c4ee406` corrected the outcome without explaining the provenance. It also retains the sanitized command result as `docs/package-operator-lifecycle-7cebe6b-20260909.result.txt`.

The retained result has 26 newline-terminated records and the documented SHA-256 `4b293249846c7c7db3322ad864fb9797e5e36387f6bebaa0b7cdb2e518834bd9`. It records opaque endpoint labels, package/binary identity, successful bounded TCP and UDP 32-byte exchanges, ordered phase progression through readiness/signal-stop/rebind/authenticated exchange, and cleanup verification.

Therefore the exact-`7cebe6b` installed-package lifecycle run may now be accepted as **bounded self-owned VPS operator evidence** for the recorded TCP/UDP lifecycle question. Preserve the exact-`14be1c8` orchestration negative and the exact-`e84770e` historical reporting error in Git history; do not rewrite history and do not rerun this answered scenario unchanged.

This still does not establish daemon/service-manager hardening, sustained WAN reliability, public reachability, performance, security approval, RC, release or production readiness.

### ACCEPT — release item-4 exact-tree provenance is repaired

`38ea310` removes the GitHub-unreachable `bb008d0...` attestation and moves the developer-prepared factual review onto a real reachable tree. Current packet/review text indexes exact `38ea310`, whose Rust CI is green. The wording correctly remains “developer-prepared factual review support”, not independent security review or approval.

The prior HIGH `EXACT_TREE_PROVENANCE` finding is closed. A green gate remains repository/test evidence only; it is not an independent security decision.

### ACCEPT WITH MEDIUM FOLLOW-UP — distinct-version A/B/A is useful bounded operator evidence, but its retained provenance is thinner than the lifecycle run

`7fce5b8` records a genuinely different package/binary rehearsal using exact old `91a735c0252e7df5da611da4a70e71a60dbdd44d`, exact current-at-run `dc90c5f265a93f113560ba3c97ab436638bdc307`, then old A again. Archive and binary hashes differ between A and B; each A1/B/A2 stage records one authenticated TCP 32-byte exchange, the selected release returns to A, an external temporary state marker survives with mode `0600`, and final listener/process/path cleanup is recorded. The scope wording correctly refuses arbitrary state-schema compatibility, production upgrade policy, release or security conclusions.

This is a real visible/operator result and should not be discarded. However, unlike the repaired lifecycle result, the current tree contains only the narrative/phase summary: it does not retain a sanitized command/result anchor with start/end timestamps, command exit status, record count and content hash. Standing evidence policy asks each WAN experiment to remain associateable with actual timing/results/cleanup. Because this rehearsal is now referenced by the release packet, tighten its provenance from already-existing material before treating it as strong release-review support.

Classify this as `MEDIUM / EVIDENCE_PROVENANCE_INCOMPLETE`, not a runtime failure and not a reason to rerun the VPS scenario.

If the original sanitized result/log still exists locally, retain only the smallest non-secret anchor needed: start/end UTC timestamps, exit status, record/line count, SHA-256, opaque endpoint labels, package/binary identities, A1/B/A2 result phases and cleanup. If it no longer exists, do not fabricate it; instead qualify the release-facing claim as a developer-recorded bounded observation whose raw/sanitized execution anchor was not retained. **No VPS rerun is authorized merely to improve this documentation.**

### MEDIUM / RELEASE_TOOLING_GAP — package smoke has no dedicated adversarial regression in the repository gate

The current `scripts/release/smoke-package.sh` checks member names for absolute/traversal paths, extracts with no owner/permission inheritance, verifies file modes and `SHA256SUMS`, and runs secret-free capabilities on a native target. This is useful. But `scripts/check.sh` does not run a dedicated package-smoke regression suite, and the current pre-extraction check does not explicitly prove fail-closed handling for non-regular archive members (symlink/hardlink/device/FIFO) or unexpected package layout.

Do **not** call this an exploit finding: the repository-owned builder itself stages ordinary directories/files, and publication signing is still absent. It is a concrete release-tool correctness/negative-test gap now that package evidence is being promoted into the release-review packet.

The coding agent may choose the smallest robust shape under the proposal protocol. Prefer explicit member-type/layout validation before extraction plus synthetic negative tests wired into `scripts/check.sh`. A small typed helper is acceptable if shell parsing would be fragile. Protect these invariants: exactly one expected package root, regular files/directories only unless an explicit reviewed need appears, no absolute/traversal/link escape, expected executable/docs/checksum layout and modes, checksum verification, and secret-free native capabilities. Do not add a generic package framework or signing/key policy in this slice.

### ACCEPTED CONTINUING BOUNDARIES

- RSEC non-policy engineering controls remain bounded-reviewed; full D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`. Do not invent TTL/LRU/history capacity/external authority or silently weaken no-reset semantics.
- HY2 exact `13da094` remains frozen `BLOCKED_HARNESS_CURRENT_LINE_HY2` at typed `unknown / client_started`; no complete pair or performance conclusion and no unchanged retry.
- repeated warm failover exact `f17b648` remains frozen at primary `startup_setup`; no unchanged retry.
- the periodic current line remains an orchestration negative; do not reopen it without a concrete material hypothesis.
- IPv6 remains environment-blocked if no owned IPv6 path exists.
- live PMTUD still requires its separately accepted authenticated wire/security design gate before live integration; stale `BLOCKED_IMPLEMENTATION` wording is not permission to change wire semantics.
- release flags remain `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`.

## Design / proposal protocol

No administrator approval is needed for the provenance and package-smoke work below. They do not change Session/Carrier/ACK/crypto/wire semantics, introduce security numeric policy, perform destructive migration, or exceed standing authorization.

For package-smoke hardening, the coding agent should propose 1–3 minimal implementation shapes if necessary, then choose the one with the smallest new state/API and clearest fail-closed tests. Reviewer will challenge the implemented design afterward; do not wait for a pre-approved function signature.

Do not expand this into SBOM/signing/key-custody work. Signing keys, publication trust policy and credentials are a separate maintainer/security-policy stage.

## Rolling queue

The queue is intentionally shorter than 6–10 hours because several WAN lines are already answered/frozen and there is no honest reason to manufacture live work. Complete A–C continuously without waiting for an hourly boundary.

### A. MEDIUM / READY_LOCAL — anchor or qualify distinct-version rehearsal provenance

**Goal:** make the exact `91a735c -> dc90c5f -> 91a735c` package rehearsal inspectable enough for release-review use without running it again.

**Why now:** it is already cited by `IMPLEMENTATION_PLAN.md`, `docs/status.md`, release engineering and the release-security packet, but its retained execution provenance is thinner than the now-correct lifecycle evidence.

**Files:** primarily `docs/package-operator-distinct-version-91a735c-dc90c5f-20260909.md`; optionally one small sanitized result file and only the release/status docs whose claim boundary actually changes.

**Protected invariants:**

- no new VPS invocation;
- no endpoint IP/private topology/secret/plaintext payload material;
- do not invent start/end/exit/hash facts;
- preserve A/B binary/archive identity and the bounded one-exchange-per-stage scope;
- do not promote a state marker into arbitrary schema-migration compatibility.

**Tests/gates:** evidence/governance checks, `scripts/check.sh`, `git diff --check`, commit/push, exact-head CI green.

**Commit/push:** yes only for real provenance/qualification delta. Continue immediately to B.

### B. MEDIUM / READY_LOCAL — make package smoke fail closed on archive shape and add negative regression

**Goal:** turn the release package smoke contract into a directly regression-tested gate instead of relying only on successful package observations.

**Why now:** package/operator evidence is now one of the strongest recent visible outputs and is indexed by item 4; validating the installer input shape is a direct release-engineering correctness task, not audit-infrastructure expansion.

**Files:** `scripts/release/smoke-package.sh` and a minimal release-smoke regression/helper if needed; wire the test into `scripts/check.sh`. Update `docs/release-engineering.md` only for factual behavior changes.

**Behavior/invariants:**

1. validate archive member names and member types before extraction;
2. reject absolute/traversal paths, symlinks, hardlinks and special-file members unless a future explicit package contract allows them;
3. require one expected package root and expected bounded layout rather than silently accepting arbitrary siblings/content;
4. retain exact `0755` executable / `0644` documentation+checksum policy and `SHA256SUMS` verification;
5. retain native-target `capabilities --json` + `secret_free=true` execution and cross-target execution skip;
6. add synthetic negative tests proving that a sibling valid file cannot rescue a bad link/type/path/layout/checksum/mode;
7. no network activity, no identities, no release signing, no runtime protocol changes.

**Proposal freedom:** shell-only validation or a tiny typed helper are both acceptable. Choose the least fragile implementation and keep the public CLI/package layout unchanged.

**Tests/gates:** focused package-smoke test first, then `scripts/check.sh`, `git diff --check`, commit/push, exact-head CI green.

**Commit/push:** yes. Continue immediately to C.

### C. READY_LOCAL / FACTUAL CLOSURE — reconcile package/release claims after A–B

**Goal:** leave `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `docs/release-engineering.md`, `docs/release-security-review-packet.md` and `docs/reviews/release-item4-subgates-20260909.md` internally consistent with the actual provenance/tooling state.

**Rules:**

- keep lifecycle exact `7cebe6b` as bounded operator evidence anchored by its retained result;
- keep or qualify the distinct A/B/A claim exactly according to A;
- describe B as package-validation/tooling evidence, not signing/provenance-service/security approval;
- D019/HY2/repeated/periodic/IPv6/PMTUD boundaries remain unchanged;
- item 4 remains developer-prepared factual support until a genuinely independent review occurs;
- do not close release item 3, item 4 or RC from these local changes.

**Commit/push:** only if A/B create factual doc deltas. Continue immediately to D.

### D. CONDITIONAL REVIEW SUPPORT — challenge one concrete item-4 claim, do not grow generic audit infrastructure

**Goal:** after A–C are green, re-read the exact-current item-4 packet and current code/tests and identify at most one concrete correctness/security/release claim whose evidence is still materially weaker than its wording.

**Action:** if a concrete defect is found and it can be fixed inside existing architecture/policy, propose/implement the smallest repair + negative test and continue. If no concrete defect is found, **skip this slice**; do not create new checker/parser/harness scaffolding just to appear busy.

**Forbidden:** claiming independent security approval, choosing D019 retention policy, creating signing/key custody policy, or altering core wire/session/crypto semantics.

### E. CONDITIONAL VPS / OUTPUT SELECTION — only a genuinely new real-network question

After A–D, re-read exact current status and standing authorization. Choose a VPS task only when there is a concrete unresolved question plus a material hypothesis/config/code/path distinction that makes new evidence valuable and non-duplicative.

Do **not** select unchanged HY2, repeated warm failover, periodic orchestration, the already-answered package lifecycle or distinct A/B/A rehearsal, IPv6 without environment, live PMTUD before its design gate, or a generic “another soak” merely because the VPS rental is active. A native x86_64 package run should also be skipped if it only duplicates already-recorded provenance.

If no such VPS question exists, leave the live queue empty rather than consuming rental time with low-value repetition.

### F. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Current bounded engineering controls remain useful, but terminal source-retention semantics require a maintainer/security-policy decision. Do not invent TTL/LRU/history capacity, external retention authority or weakened no-reset semantics. This does not block A–E.

## VPS / evidence priority

- No same-class lifecycle or distinct-package rerun is needed to repair provenance.
- The distinct A/B/A result is already a new VPS-only operator output from the last cycle; preserve it rather than immediately repeating package work.
- Frozen HY2/repeated/periodic negatives remain valid. A future run needs a specific changed hypothesis.
- VPS/load observations never substitute for deterministic security accounting, independent review or D019 policy.

## Visible-output check

The last 24–48 hours clearly produced real output: current-package reproducibility/install smoke, ordered signal lifecycle implementation and real installed-package lifecycle evidence, plus a genuine distinct-binary A(old) -> B(current) -> A(old) VPS rehearsal. This is not an audit-only period. A–C are direct release/package truth and correctness work; after them, do not invent more audit machinery if no concrete finding exists.

## Stagnation check

This is **not** `STALLED_IMPLEMENTATION`: four new developer-owned commits landed since the last reviewer handoff, all reviewed exact heads are green, both previous HIGH provenance defects were actively repaired, and a new VPS operator result was produced.

## Maintainer/admin boundary

No administrator action is required for A–E when kept within the constraints above. Do not rewrite published Git history autonomously. D019 remains the known maintainer/security-policy checkpoint. Signing key custody/publication trust, any new security numeric policy, core wire/session/crypto changes, production/service-manager mutation outside the standing experimental boundary, or destructive migration still require explicit maintainer action.
