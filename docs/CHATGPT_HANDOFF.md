# ChatGPT reviewer handoff — finish identity/config boundary, then select the next visible local output

## Reviewed state

- Previous reviewer-owned handoff: exact `dd99c0c8c10e31d3d521687bdbf5c974bca44784` (`docs(handoff): close identity reload TOCTOU before next output`).
- Developer-owned head reviewed this cycle: exact `e46c05658cd2202941ef2fd26ec547dcb20938b3` (`docs: close identity configuration ordering`).
- `main` was exactly `e46c05658cd2202941ef2fd26ec547dcb20938b3` before this reviewer-only handoff update.
- Developer sequence since the previous handoff:
  - `03016dd84e7862b20ab422681c5da28ead15926f` — changes existing-identity reload to one `O_NOFOLLOW|O_CLOEXEC` open, validates metadata on that `File`, and reads bytes through the same descriptor; it also temporarily added an unrequested parent-directory policy.
  - `67149e2d070595e96447931ac53e7978e9e42170` — initial factual/evidence closure, but it referred to a non-reachable `e5aeef2...` tested-tree identity.
  - `e8f4fc70554df1bb0c11784d7a54e43a73726c3b` — removes the unrequested parent-directory ownership/writability policy and keeps only the reviewer-requested descriptor-bound final-component fix.
  - `0e664d6ede8a93bcb377d31dd917ea3b94612c0f` — consolidates the duplicate identity filesystem process coverage.
  - `d1d2d298dbfeca1cca7bfae4dcda96f26491484c` / `4b7291a973d9dc6236e5213d6aa3a707cd5a057f` — repair the descriptor-bound local-CI/tested-tree truth to real exact `0e664d6`; the evidence file basename still contains the obsolete `e5aeef2` token even though its contents now truthfully anchor `0e664d6`.
  - `899b190379c7a1dd213bfb520eff221ef15cfe39` — moves server bind/trust parsing and client peer/address/payload parsing before automatic identity creation; adds deterministic invalid-config/no-identity process negatives.
  - `e46c05658cd2202941ef2fd26ec547dcb20938b3` — records exact-`899b190` developer-local CI/native packaged invalid-config evidence and refreshes release/item-4 factual wording.
- Developer-local exact-tree evidence retained for exact `0e664d6`: `2026-09-09T08:18:32Z` -> `08:20:22Z`, Linux x86_64, Rust `1.98.0`, `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, initial/final source tree clean.
- Developer-local exact-tree evidence retained for exact `899b190`: `2026-09-09T08:28:09Z` -> `08:30:00Z`, Linux x86_64, Rust `1.98.0`, stable gate and `git diff --check` exit 0, source tree clean; one native x86_64 package rejected a deterministic malformed client address without creating the requested external identity.
- GitHub combined status exposes no hosted status records for exact `899b190` or exact developer head `e46c056`; do not poll or wait for Actions. Local exact-tree CI remains the ordinary closure mechanism.
- `work/e1a-staged-accounting-20260907` is still exactly `f4404257520e9a014ac4e785b0ab9a97f8aaf794`; `main` is now more than 70 commits ahead with no branch-only work. `work/continue-20260904` remains the stale diverged `d271a99a2ab26abbcb146c411ba0fde697395abe`. Do not coordination-merge either merely to manufacture work.

## Review verdict

### ACCEPT — descriptor-bound existing-identity reload closes the prior HIGH

The current implementation opens an existing Unix identity with `O_NOFOLLOW|O_CLOEXEC`, obtains file metadata from that same open `File`, requires a regular file with no group/world permission bits, and reads identity bytes through that same descriptor. This removes the previous final-component `symlink_metadata(path) -> reopen(path)` race without changing Session/Carrier/ACK/crypto/wire semantics.

The later `e8f4fc7` correction was also appropriate: the prior reviewer contract did **not** establish a parent-directory ownership/writability policy, so the temporary extra policy added in `03016dd` should not have been promoted as a security requirement. Keep the descriptor-bound final-component contract; do not grow this into an `openat2`/directory-FD filesystem framework without a concrete new threat requirement.

The prior **HIGH / IDENTITY_RELOAD_TOCTOU** is therefore closed.

### ACCEPT_WITH_BOUNDS — deterministic server/client configuration is now mostly before identity creation

Exact `899b190` correctly moves bind syntax/port consistency, server peer-key hex parsing, client address parsing, and benchmark/payload parsing before `load_or_generate`. The process negatives prove malformed non-hex server trust input and malformed server/client addresses leave a nonexistent identity path absent. Network bind/connect failure remains a runtime boundary and is not required to precede identity creation.

However, the release-facing sentence that deterministic trust configuration is fully validated before identity creation is still too strong.

### MEDIUM / READY_LOCAL — valid-hex wrong-length peer keys still create identities before deterministic rejection

`unhex()` validates hex syntax and even length, but it does **not** require a Noise/X25519 public key to be exactly 32 bytes. `TrustPolicy::new` simply stores the supplied `Vec<u8>` and performs no key-length validation. On the client, `InitiatorHandshake::new_with_prologue_binding` is the later layer that will reject an invalid remote-key length.

Therefore values such as a valid-hex 31-byte or 33-byte `--client-key` / `--server-key` are already deterministically invalid configuration, yet current server/client code can still call `load_or_generate` first. This is the remaining concrete part of the earlier **IDENTITY_SIDE_EFFECT_BEFORE_CONFIG_VALIDATION** finding.

Repair shape is local and does not require a new policy:

- introduce one small peer-public-key parser/helper that decodes hex and requires exactly 32 bytes before identity creation;
- use it for both server `--client-key` and client `--server-key` so the contract cannot drift between directions;
- do not move Noise/authentication semantics into CLI parsing; this is only the already-required static X25519 key-size check;
- add focused server and client process negatives for syntactically valid but short/long peer keys, each with a previously nonexistent identity path and an assertion that the path remains absent;
- keep the existing malformed-hex and malformed-address cases.

Do not claim the identity/config side-effect lane closed until this is fixed and exact-tree gated.

### MEDIUM / OPERATOR CORRECTNESS — requested `0600` creation is still umask-dependent

The current Unix creation path uses `OpenOptionsExt::mode(0o600)` before writing private bytes. That is secure against accidentally broad group/world permission because the process umask can only remove bits, but POSIX creation mode is still masked by the caller's umask. Under an unusually restrictive umask that removes owner read/write bits, first creation can succeed through the already-open descriptor while the resulting file is not reusable on the next process invocation.

This matters because release/operator wording currently describes a reusable exact `0600` identity and package smoke asserts `0600` under the ordinary environment.

Do **not** loosen security or add an umask policy. The minimal implementation shape is to create the new inode fail-closed as today, then set permissions on the already-open `File` descriptor to exact `0600` **before writing private bytes**, then write/sync. This preserves the security invariant while making the operator contract independent of an unusually restrictive caller umask.

A focused Unix process regression should exercise a restrictive child umask without changing the parent test process globally, then prove first create succeeds at exact `0600` and the next invocation reloads the same public key. Avoid a generic permissions framework.

### LOW / FACTUAL HYGIENE — one evidence filename still carries the nonexistent `e5aeef2` token

`docs/local-identity-security-e5aeef2-20260909.md` now truthfully says its tested tree is exact `0e664d6`, and current release/item-4 text also names `0e664d6`. The basename alone is stale and can mislead future automated provenance readers.

During the next factual reconciliation, rename it to an exact-`0e664d6` basename and update links. This is evidence hygiene, not a reason to rerun CI or VPS evidence.

## Local-CI-first rule

For coherent READY_LOCAL implementation/test commits, do not poll or wait for GitHub Actions. Validate the **exact pushed implementation SHA** in a clean temporary worktree/clone.

Minimum default gate:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # must be empty
```

Retain exact SHA, exact commands, distinct UTC start/end timestamps, exits, host/OS/arch, Rust stable version, and initial/final clean-tree state when the result anchors release evidence. Label developer-local CI, reviewer-executed checks, and hosted CI separately.

The identity/config work does not change wire decode/parser/crypto framing, so fuzz is not required unless an implementation unexpectedly crosses that boundary.

## Rolling queue — execute continuously in dependency order

There is no unresolved HIGH. Execute A -> B -> C -> D -> E continuously. Then do F -> G -> H -> I without waiting for another reviewer cycle if the selected local output remains dependency-ready. J is conditional live work; K remains policy-blocked. Do not enter watcher/polling mode merely because `READY_LIVE: none`.

### A. READY_LOCAL / OPERATOR CORRECTNESS — finish static peer-key validation before identity creation

**Goal:** every deterministic server/client peer-key format/size error fails before persistent identity creation.

**Files/concepts:** `crates/neko-cli/src/main.rs`, focused process coverage in `crates/neko-cli/tests/probe.rs`.

**Protected invariants:** no authentication/wire/session behavior change; no private-key logging; server/client still auto-generate on a valid execution path; network reachability remains a later runtime boundary.

**Behavior:** decode and require exactly 32 bytes for `--client-key` / `--server-key` before `load_or_generate`. Reuse one helper in both directions.

**Minimum tests:** existing invalid hex/address cases plus server short/long valid-hex key and client short/long valid-hex key; all rejected cases use absent temp identity paths and prove those paths remain absent.

**Commit/push:** one coherent implementation/test commit. Continue immediately to B.

### B. READY_LOCAL / OPERATOR CORRECTNESS — make newly generated identity exactly `0600` before private write under restrictive umask

**Goal:** preserve the owner-only creation invariant while making the reusable identity mode independent of caller umask.

**Files/concepts:** new-file branch of `load_or_generate`, one focused Unix process test.

**Protected invariants:** `create_new(true)` remains; existing path is never truncated or silently chmod-repaired; no private bytes are written before the file descriptor is known to have exact `0600`; existing identity reload semantics remain unchanged.

**Behavior:** after fail-closed new-file open, set permissions through the already-open `File` to exact `0600`, then write/sync. If permission-setting fails, fail before private bytes are written.

**Minimum test:** isolated child with restrictive umask creates identity; resulting mode is `0600`; second invocation reloads same public key; ordinary create/reload/permissive/symlink/non-regular negatives stay green.

**Commit/push:** coherent implementation/test commit. Continue immediately to C.

### C. READY_LOCAL — exact-tree local gate and minimal provenance for A+B

Validate the final pushed implementation SHA containing A+B in a clean temporary checkout with the normal local gate. Record distinct UTC start/end, host/OS/arch, stable Rust version, exact commands/exits and initial/final clean tree. No hosted-CI wait.

Because A+B change the packaged binary/operator identity path, produce one native x86_64 package outside the source tree and smoke only the newly material facts: one valid create/reload and one valid-hex wrong-length no-identity-created case. If practical, include the restrictive-umask create/reload path; do not create a second package checker.

Continue immediately to D.

### D. READY_LOCAL / FACTUAL RECONCILIATION — make release/item-4 truth match the exact gated implementation

Refresh only affected facts in:

- `docs/release-engineering.md`;
- `docs/release-security-review-packet.md`;
- `docs/reviews/release-item4-subgates-20260909.md`;
- the local identity/config evidence note.

Rename the misleading `docs/local-identity-security-e5aeef2-20260909.md` to an exact-`0e664d6` basename and update all links; do not rewrite the validated facts or invent a new run.

Preserve `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, independent-review absence, D019 blockage and all WAN claim boundaries.

Continue immediately to E.

### E. BOUNDED REVIEW CLOSURE — close the identity/config lane unless one concrete defect remains

Perform exactly one bounded adjacent review of current server/client identity/config ordering for:

- deterministic parse/validation that still occurs after persistent secret creation;
- secret material in stdout/stderr/logging;
- readiness prerequisite inflation before validation;
- mutation-before-error around the identity file.

If no additional concrete defect is demonstrable without inventing new local-filesystem policy, explicitly close this audit lane. Do not expand into parent-directory policy, hard-link threat frameworks, generic secret-memory hardening, or a new checker suite unless a current requirement supplies that threat model.

Continue immediately to F.

### F. LOCAL OUTPUT SELECTION — propose 1–3 concrete next outputs and choose one autonomously

Re-read exact-current `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md`, release packet and implementation. Current release-evidence item 3 has `READY_LIVE: none`; item 4 still lacks independent review and D019 policy closure. Do not invent WAN work or a fake release gate.

If no already-named dependency-ready local output exists, propose 1–3 **specific** runtime/operator/release-correctness outputs grounded in an observed code/evidence gap. Each proposal must name:

- the exact output/problem;
- owner file/API;
- protected invariant;
- minimum positive/negative test;
- whether it changes core architecture, wire/crypto semantics or numeric security policy.

Autonomously choose the smallest option needing no maintainer value decision. Record the short proposal/choice in the developer-owned implementation note or commit message; do not wait for reviewer pre-approval.

### G. READY_LOCAL — implement the selected F output

Implement the chosen bounded output as one coherent slice. Prefer a visible runtime/operator/release behavior over another schema/checker/document-only refinement. Do not reopen package archive variants, generic DeliveryLedger auditing, previous-release compatibility, FEC/0-RTT/striping/multipath/exotic carriers, or generic evidence-schema work without a newly observed problem.

Continue immediately to H.

### H. READY_LOCAL — exact-tree gate for G

Run the normal exact-tree local gate on the pushed G implementation commit. Add fuzz only if G actually touches wire decoder/parser/crypto framing. Persist minimal provenance only if it supports a release-facing factual claim.

Continue immediately to I.

### I. READY_LOCAL / MILESTONE RECONCILIATION — record the visible output, then continue or select again

Update `docs/status.md` / implementation plan / release packet only if G materially changes a status, acceptance boundary or release-facing fact. If G does not change those truths, avoid documentation churn. Then either proceed to another already-ready concrete output or repeat the bounded F proposal cycle; do not enter watcher mode solely because the reviewer interval has not arrived.

### J. CONDITIONAL VPS OUTPUT — only if a genuinely new live question opens

Current repository truth remains `READY_LIVE: none`. Standing authorization remains valid, but authorization is not a reason to rerun already-answered/frozen rows.

Only execute live work when exact-current implementation/evidence opens a specific unresolved real-network question with materially changed code/config/instrumentation/path/hypothesis and satisfied dependencies. Otherwise do not unchanged-rerun HY2, repeated warm failover, periodic, installed-package lifecycle, distinct A->B->A, generic soak, IPv6 without a real owned IPv6 path, or live PMTUD before its separate authenticated wire/security design gate.

### K. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Non-policy engineering controls have bounded factual support. Terminal source-retention/no-reset semantics still require maintainer/security-policy judgment. Do not invent TTL, LRU/history capacity, external authority or weaker reset semantics. This does not block independent A-J work.

## Governance boundaries

- `IMPLEMENTATION_COMPLETE=true` remains bounded research-implementation status only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain authoritative.
- release item 3 remains incomplete; machine-readable opportunity classification still says `READY_LIVE: none` for remaining unimplemented capabilities.
- release item 4 remains incomplete because independent review, D019 policy closure and other release boundaries remain absent.
- current x86_64-only first-RC scope does not turn missing native aarch64 evidence into a blocker unless N6 changes.
- standing VPS authorization remains valid for genuinely new dependency-ready self-owned bounded questions.
- HY2 current line and repeated warm-failover current line remain frozen against unchanged retries.
- IPv6 remains environment-blocked when no owned path exists.
- live PMTUD remains behind a separate authenticated wire/security design gate.
- signing/key custody/SBOM/publication trust remain separate release concerns; do not fabricate credentials or policy.

## Stop / escalation

Do not notify or stop for ordinary progress. Escalate only for a core Session/Carrier/ACK/crypto/wire architecture change; new security numeric policy; destructive/canonical-meaning migration; action outside standing authorization; production impact; new credentials/server/third-party permission; benchmark value judgment; unresolved major security issue that cannot be repaired within existing intent; D019 policy decision; or entry into a genuinely new project stage.
