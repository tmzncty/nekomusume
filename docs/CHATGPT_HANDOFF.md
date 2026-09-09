# ChatGPT reviewer handoff — finish identity filesystem atomicity, then remove pre-validation secret side effects

## Reviewed state

- Previous reviewer-owned handoff: exact `cdb7b5462cdd69739a191fcc13c324ec0f9447b4` (`docs(handoff): correct preauth collision provenance`).
- Default branch reviewed at exact `70e2a3f0043c1bd4592e8c7a6fc3712f81867e43` (`docs: close local identity boundary`).
- Developer-owned sequence since the prior reviewer handoff:
  - `749976f89056fd98e2295941471b185cce8b524f` — replaces write-then-chmod identity creation with `OpenOptions + create_new + mode(0600)`, rejects stable non-regular/group-world-accessible identity paths, and adds focused process coverage.
  - `d4cab69bcf09cf7eea5a67eed27afab7a8f15a40` — adds a second overlapping identity-filesystem process test.
  - `70e2a3f0043c1bd4592e8c7a6fc3712f81867e43` — records exact-`749976f` developer-local CI/native packaged-operator evidence and refreshes release/item-4 factual wording.
- Retained developer-local gate for exact `749976f`: `2026-09-09T07:11:06Z` -> `07:12:57Z`, Linux x86_64, Rust `1.98.0`, `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, initial/final source tree clean. A native x86_64 package from the same tree also completed secure-create/reload/permissive-file rejection smoke outside the source checkout.
- GitHub combined status exposes no hosted status records for exact `749976f`; do not poll or wait for Actions. Developer-local exact-tree CI remains the normal closure path.
- `work/e1a-staged-accounting-20260907` remains fully behind main (`f4404257`, main +67 at this review). `work/continue-20260904` remains a stale diverged branch (`d271a99a`, main +151 / branch +1). Do not coordination-merge either merely to manufacture work.

## Review verdict

### ACCEPT_WITH_BOUNDS — original identity creation/steady-state path gap is materially improved

Exact `749976f` fixes the original write-before-chmod defect: on Unix the new file is created with restrictive mode before private bytes are written, `create_new(true)` avoids truncating/replacing an already-existing final path, and stable symlink/special-file/group-world-readable inputs fail before ordinary key use. The package-level smoke and exact-tree local gate are useful developer evidence.

Keep all existing maturity boundaries: this is not independent security review, signing/key custody, RC, release, public-listener approval, or production readiness.

### HIGH / READY_LOCAL — existing-identity reload still has a final-component TOCTOU gap

Current `read_identity` does:

```text
symlink_metadata(path)
  -> validate file type / mode
  -> fs::read_to_string(path)
```

The metadata check and content open are separate path-based operations. A final path that is a regular owner-only file at the metadata check can be replaced with a symlink before `read_to_string`; the second operation follows that new path. Therefore current release-facing wording that reload “rejects symlinks” is only true for a stable path, not for concurrent final-component substitution.

Classify this as **HIGH / IDENTITY_RELOAD_TOCTOU** because it sits exactly on the long-term private-key filesystem boundary and invalidates the stronger fail-closed claim. Do not open unrelated new runtime/release work until this is repaired and re-anchored.

This is automatically repairable without changing Session/Carrier/ACK/crypto/wire semantics and without inventing numeric security policy.

**Preferred minimal implementation shape on Unix:** open the existing identity once with a no-follow open, then perform file-type/mode validation and all reads through that same open file descriptor. Rust stable `std::os::unix::fs::OpenOptionsExt::custom_flags` supports `O_NOFOLLOW`; adding the small `libc` dependency solely for the platform constant is acceptable if needed. A shape such as:

```text
OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(path)
  -> file.metadata()
  -> validate regular + owner-only
  -> read identity bytes from that File handle
```

removes the check-then-reopen race for the final path component. Preserve the current `create_new(true) + mode(0600)` creation path. Do not replace this with a bigger directory-fd/openat2 framework unless a concrete parent-directory threat requirement is separately established.

**Rejected shape:** another `symlink_metadata(path)` / `metadata(path)` check followed by a fresh path open is not a fix.

### MEDIUM / TEST_PROVENANCE_AND_DUPLICATION — current main contains post-gate overlapping tests

Exact `749976f` is the retained locally gated implementation tree, but `d4cab69` added a second substantially overlapping identity-filesystem test after that exact gate. The current tree therefore contains additional test code not covered by the retained exact-`749976f` local-CI record. This does not invalidate the tested implementation, but it should not be treated as exact-current gate evidence.

While repairing the HIGH, consolidate the two overlapping identity tests rather than growing a third variant. Prefer the existing `127.0.0.1:0` invalid-identity server form over a fixed `40080` in a path that is expected to fail before bind. Add one cheap non-regular negative (for example a directory) if needed to lock the file-type contract; do not build a generic filesystem-security test framework.

### MEDIUM / READY_LOCAL AFTER HIGH — deterministic invalid configuration can create a persistent identity before rejection

There is also one concrete adjacent mutation-before-error defect worth fixing after the HIGH. `server()` currently calls `load_or_generate` before parsing/validating `--client-key` and before validating the final bind address. A command that is deterministically invalid can therefore create a long-term private identity and then exit. `client()` likewise creates/loads its identity before all deterministic client argument validation is complete.

Classify this as **MEDIUM / IDENTITY_SIDE_EFFECT_BEFORE_CONFIG_VALIDATION**. The goal is not to redesign identity storage or remove auto-generation. It is to make deterministic CLI configuration errors fail before persistent secret creation.

Minimal contract:

- parse and validate all deterministic, side-effect-free command configuration that can be checked without network I/O before `load_or_generate`;
- malformed/missing peer key, malformed address/bind syntax, incompatible port arguments, invalid payload-mode options, etc. must not create a new identity;
- do not require network reachability before identity generation; a connect/bind runtime failure is a separate boundary unless the implementation can cheaply avoid the persistent side effect without architectural churn;
- preserve lifecycle truth: `ConfigurationAccepted` must not be satisfied before the configuration actually covered by that prerequisite has been accepted.

Focused negative tests should use a previously nonexistent temp identity path and prove deterministic invalid commands exit nonzero with the identity path still absent.

## Local-CI-first rule

For coherent READY_LOCAL implementation/test commits, do not poll or wait for GitHub Actions. Validate the **exact pushed implementation SHA** in a clean temporary worktree/clone.

Minimum default gate:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # must be empty
```

Retain exact SHA, exact commands, distinct UTC start/end timestamps, exits, host/OS/arch, Rust stable version, and initial/final clean-tree state when the result anchors release evidence. Label developer-local CI, reviewer-executed checks, and hosted CI separately.

The identity work does not change wire decode/parser/crypto framing, so no fuzz is required unless the implementation unexpectedly crosses that boundary.

## Rolling queue — execute continuously in dependency order

There is one unresolved HIGH. Finish A -> B -> C before unrelated work. Then continue D -> E -> F -> G without waiting for another reviewer cycle while dependencies remain satisfied. H is conditional live work; I remains policy-blocked. Do not enter watcher/polling mode merely because `READY_LIVE: none`.

### A. HIGH / READY_LOCAL — make existing-identity reload no-follow and handle-based

**Goal:** remove the final-component metadata/open TOCTOU while preserving current secure creation semantics.

**Files/concepts:** `crates/neko-cli/src/main.rs`, `crates/neko-cli/Cargo.toml` only if a Unix constant dependency is required, focused process/filesystem tests in `crates/neko-cli/tests/probe.rs`.

**Protected invariants:** no private-key logging; no silent chmod repair; create races do not truncate/replace existing identities; stable symlink/special/insecure modes fail closed; no Session/Carrier/ACK/crypto/wire change.

**Implementation:** use one no-follow open for an existing Unix identity, inspect metadata from the opened `File`, and read bytes from that same handle. Preserve the non-Unix path with the smallest behavior-compatible implementation. Consolidate the two current overlapping identity tests while touching this lane.

**Minimum tests:** secure create/reload; `0644` unchanged rejection; stable symlink unchanged-target rejection; one non-regular input rejection; invalid identity never reaches server READY. Avoid a flaky race-loop test; the no-follow single-open implementation structure is the deterministic race closure.

**Commit/push:** one coherent implementation/test commit. Continue immediately to B.

### B. READY_LOCAL — exact-tree local gate and provenance for A

Validate the exact pushed A SHA in a clean temporary checkout with the normal local gate. Persist only a small non-secret CI note if release-facing evidence needs re-anchoring. Because A changes the packaged binary/dependency set, record host/Rust/UTC/clean-tree facts accurately; do not fabricate hosted CI.

Continue immediately to C.

### C. READY_LOCAL / FACTUAL RECONCILIATION — correct the symlink/fail-closed claims

Refresh only facts made stale by A in:

- `docs/release-engineering.md`;
- `docs/release-security-review-packet.md`;
- `docs/reviews/release-item4-subgates-20260909.md`;
- the identity local-CI note if a superseding note is created.

Anchor implementation claims to the real A exact SHA. Preserve `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, independent-review absence and D019 policy blockage.

If an actual native package is needed to retain the packaged-operator claim after the binary-changing fix, run one local native package smoke outside the source tree: secure create/reload + permissive existing file + stable symlink rejection, cleanup, no key material retained. This is local operator evidence only; no VPS is needed.

Continue immediately to D.

### D. READY_LOCAL / OPERATOR CORRECTNESS — validate configuration before persistent identity side effects

**Goal:** deterministic invalid CLI configuration must not create a new long-term identity.

**Files/concepts:** CLI argument parsing/config assembly and lifecycle prerequisite ordering in `crates/neko-cli/src/main.rs`; focused process tests.

**Protected invariants:** no CLI surface invention unless required by the existing contract; no change to authentication/wire/session semantics; keygen remains the explicit command for intentionally creating an identity; server/client still auto-generate when their validated execution path legitimately needs an identity.

**Tests:** at least malformed/missing peer key and malformed bind/address or payload-mode configuration with a nonexistent temp identity path; assert nonzero exit and path still absent. Ensure valid existing process tests remain green.

**Commit/push:** coherent implementation/test commit, then continue to E.

### E. READY_LOCAL — exact-tree gate for D and one produced-package operator smoke

Run the normal exact-tree local gate on D. If D changes the packaged binary, produce one native x86_64 package outside the source tree and smoke one valid identity path plus one deterministic invalid-config/no-identity-created case. Retain only non-secret hashes/result/timing/cleanup facts if documentation needs them.

Continue immediately to F.

### F. READY_LOCAL / FACTUAL CLOSURE — close the identity/config lane and stop checker growth

Reconcile release-engineering/item-4/release packet only where D/E changed facts. Then perform exactly one bounded adjacent identity/config review for secret leakage, mutation-before-error or readiness ordering. If no additional concrete defect is demonstrable without inventing policy, explicitly close this audit lane. Do not add more identity checker variants merely because more filesystem cases exist.

Continue immediately to G.

### G. LOCAL OUTPUT SELECTION — choose the next visible runtime/operator/release output

Re-read exact-current `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md`, release packet and code. If a named dependency-ready local output exists, take it. Otherwise propose 1-3 concrete options grounded in observed code/evidence; each proposal must name the gap/output, owner file/API, protected invariant, minimum positive/negative test, and whether it changes architecture/security policy. Autonomously choose the smallest option needing no maintainer value decision and implement it.

Do not reopen package archive variants, generic DeliveryLedger auditing, previous-release compatibility, FEC/0-RTT/striping/multipath/exotic carriers, or generic checker/schema work without a new observed problem.

### H. CONDITIONAL VPS OUTPUT — only if a genuinely new live question opens

Current repository truth remains `READY_LIVE: none`. Standing authorization is still valid, but authorization is not a reason to rerun already-answered/frozen rows.

If later work creates a new concrete unresolved real-network question with a materially changed implementation/configuration/instrumentation/path/hypothesis, execute it boundedly under standing authorization and retain cleanup/negative evidence. Otherwise do not unchanged-rerun HY2, repeated warm failover, periodic, installed-package lifecycle, distinct A->B->A, generic soak, IPv6 without a real owned IPv6 path, or live PMTUD before its separate authenticated wire/security design gate.

### I. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Non-policy engineering controls have bounded factual support. Terminal source-retention/no-reset semantics still require maintainer/security-policy judgment. Do not invent TTL, LRU/history capacity, external authority or weaker reset semantics. This does not block independent A-H work.

## Governance boundaries

- `IMPLEMENTATION_COMPLETE=true` remains bounded research-implementation status only.
- release item 3 remains incomplete; item 4 remains incomplete because independent review and D019 policy closure are absent.
- release packet currently classifies the next live action as none; local correctness/release work continues.
- standing VPS authorization remains valid for genuinely new dependency-ready self-owned bounded questions.
- HY2 current line and repeated warm-failover current line remain frozen against unchanged retries.
- IPv6 remains environment-blocked when no owned path exists.
- live PMTUD remains behind a separate authenticated wire/security design gate.
- signing/key custody/SBOM/publication trust remain separate release concerns; do not fabricate credentials or policy.

## Stop / escalation

Do not notify or stop for ordinary progress. Escalate only for a core Session/Carrier/ACK/crypto/wire architecture change; new security numeric policy; destructive/canonical-meaning migration; action outside standing authorization; production impact; new credentials/server/third-party permission; benchmark value judgment; unresolved major security issue that cannot be repaired within existing intent; D019 policy decision; or entry into a genuinely new project stage.
