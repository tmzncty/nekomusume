# ChatGPT reviewer handoff — secure the local identity-file boundary

## Reviewed state

- Previous reviewer-owned handoff: exact `d583e598b56888d07bba6ff29aa19c9c892e77ff` (`docs(handoff): finish CLI truth and supersede package provenance`).
- Developer state reviewed through default-branch exact `fc000fea5bf7c9da36471d31430fdb92325a805b` (`docs: close CLI package evidence lane`).
- New developer-owned sequence since that review:
  - `342e65c684a0997ced3fb3036b8889bd4243b8f9` — makes human `capabilities` cover the canonical command inventory and keeps failover legacy names as aliases; focused process regression covers help + human + JSON inventories and unknown-command rejection.
  - `b1b2552181a995d0d9b73def7bcde6d71418a344` — after the first local exact-tree gate exposed a collision in preauth test `response_send_restores_socket_write_timeouts`, replaces its hard-coded `127.0.0.1:40080` listener with an OS-assigned loopback port; production behavior is unchanged.
  - `fc000fea5bf7c9da36471d31430fdb92325a805b` — records exact-`b1b2552` developer-local CI/native package provenance and reconciles release/item-4 wording, closing the CLI/package inventory/provenance lane locally.
- GitHub combined-status lookup exposes no hosted status records for exact `b1b2552` or current `fc000fe`. Do not poll Actions or wait for quota; the repository-defined local exact-tree gate is the normal closure path.
- Work branches `work/e1a-staged-accounting-20260907` (`f4404257`) and `work/continue-20260904` (`d271a99a`) remain stale/non-authoritative. Do not coordination-merge them merely to manufacture work.

## Review verdict

### ACCEPT — CLI/package lane is locally closed

The human/JSON/help command inventory contradiction is closed. Exact `b1b2552` also removes the preauth test's fixed-port collision without changing runtime semantics. The retained local record for exact `b1b2552` contains distinct UTC start/end times, `scripts/check.sh`, `git diff --check`, initial/final clean-tree state, reproducibility, an actual produced native x86_64 package smoke, and archive/binary hashes. Treat this as **developer-local CI/release-tool evidence**, not reviewer-executed CI, hosted CI, VPS/WAN evidence, independent security review, RC, release, or production authorization.

Do not continue growing package archive/checker variants unless a new concrete defect appears.

### HIGH / READY_LOCAL — long-term identity files are not fail-closed at the filesystem boundary

Current `neko-cli` `load_or_generate` reads an existing identity with `fs::read_to_string` and accepts it without checking Unix file type or permission bits. For a missing identity it generates private/public key material, writes it with `fs::write`, and only afterwards changes the mode to `0600` on Unix.

This creates a concrete release/security correctness gap:

1. a newly generated private key is written before the restrictive chmod, so a permissive umask can expose broader permissions during creation;
2. an existing group/world-readable key file is silently accepted;
3. symlink/special-file identity paths are not explicitly rejected before read/write, so the operator boundary is not fail-closed and special files can have surprising/blocking behavior.

Classify this as **HIGH / LOCAL_IDENTITY_FILE_SECURITY_GAP**. It is automatically repairable and does not require maintainer policy: current code already expresses an intended `0600` boundary, so this work enforces an existing security intent rather than inventing a new numeric policy. It must be fixed before opening unrelated new runtime/release work.

## Local-CI-first rule

For coherent READY_LOCAL implementation/test commits, do not poll or wait for GitHub Actions. Verify the **exact pushed implementation SHA** in a clean temporary worktree/clone.

Minimum default gate:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # must be empty
```

When the result anchors release evidence, retain exact SHA, exact command(s), distinct UTC start/end timestamps, exit codes, host/OS/arch, stable Rust version, initial/final clean-tree state, and relevant package/binary hashes. Label developer-local CI, reviewer-executed checks, and hosted CI separately.

The identity-file work does not alter wire decoder/parser/crypto framing, so no fuzz is required unless the implementation unexpectedly crosses that boundary.

## Rolling queue — execute continuously in dependency order

There is one HIGH and it is first. Finish A -> B -> C -> D without waiting for another reviewer cycle when dependencies are satisfied. E selects the next real output; F is conditional live work; G remains policy-blocked. Do not enter watcher/polling mode merely because `READY_LIVE: none`.

### A. HIGH / READY_LOCAL — secure identity creation and reload

**Goal:** make every CLI path using `load_or_generate` fail closed around long-term private-key files.

**Files/concepts:** `crates/neko-cli/src/main.rs`, focused process/filesystem tests in `crates/neko-cli/tests/probe.rs` or an equally small CLI-owned test location.

**Protected invariants:**

- private key bytes are never logged;
- no Session/Carrier/ACK/crypto/wire semantic change;
- no D019/source-retention policy invention;
- no silent repair-and-continue of an insecure existing key;
- create races never truncate/replace an existing identity;
- server must not emit READY when its required identity boundary is invalid.

**Preferred minimal shape:** the Agent may compare 1-3 small implementations and choose the smallest fail-closed one. On Unix, prefer `OpenOptions` + `OpenOptionsExt::mode(0o600)` + `create_new(true)` so secret bytes are first written only after the file exists with restrictive permissions. For existing paths, inspect `symlink_metadata`; require a regular non-symlink file and owner-only permissions before reading. Reject insecure/special inputs rather than chmod-and-continue. A mode such as `0400` may remain acceptable if the implementation chooses an owner-only-bit invariant instead of exact `0600`; do not broaden beyond owner access. Preserve current behavior on non-Unix with the smallest separate path.

**Minimum tests:**

- newly generated identity is restrictive on Unix and a second secure reload returns the same public key;
- an existing `0644` identity is rejected and its bytes/mode remain unchanged;
- a symlink identity path is rejected without modifying the target;
- a secure existing identity is accepted;
- at least one server/process test proves an invalid identity path never reaches READY.

**Commit/validation:** make one coherent implementation/test commit, push it, then validate that exact SHA in a clean temporary checkout with the normal local gate. If the gate exposes a deterministic unrelated test collision, repair only that concrete blocker and re-anchor the exact implementation SHA; do not wait for Actions.

Continue immediately to B.

### B. READY_LOCAL — persist exact-tree identity-security CI provenance

Retain a small non-secret developer-local CI note for the exact successful A implementation SHA: UTC start/end, commands, exits, host/OS/arch, Rust stable, initial/final clean tree. Do not persist private identity contents. A focused test name/result summary is enough; do not create a generic security evidence framework.

**Commit/push:** yes for the small evidence note. Continue immediately to C.

### C. READY_LOCAL / OPERATOR OUTPUT — smoke the produced native package's identity boundary

Use an **actual package produced from exact A** on the local native x86_64 host; no VPS is needed.

Bounded operator questions:

- packaged `keygen --identity <external-temp-path>` creates an owner-only identity;
- a second invocation reloads the same identity/public key;
- a deliberately permissive existing identity is rejected rather than silently used or chmod-repaired;
- all temporary identity material is outside the source tree and is cleaned after the smoke.

Retain only non-secret facts: exact tree/package hash, command classes, permission/result state, UTC timing, exit status, cleanup. Never commit private/public key material from the smoke. This is local packaged-operator evidence only.

If A's focused process tests already make this package-level smoke redundant in a way that can be demonstrated directly, the Agent may record that reason and skip C rather than manufacture duplicate evidence.

Continue immediately to D.

### D. READY_LOCAL / FACTUAL RECONCILIATION — close the identity-file lane

Refresh only facts that actually became stale in:

- `docs/release-security-review-packet.md`;
- `docs/release-engineering.md`;
- `docs/reviews/release-item4-subgates-20260909.md`;
- `SECURITY.md` only if its persisted-key wording needs a factual clarification.

Anchor implementation behavior to the real exact A implementation SHA and local evidence to B/C. Preserve all maturity boundaries: independent security review absent, D019 policy unresolved, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`.

After D, stop identity-checker growth unless a new concrete defect exists.

### E. LOCAL OUTPUT SELECTION — one bounded adjacent follow-through, then move on

Inspect the exact-current CLI identity/config boundary for **one** adjacent concrete mutation-before-error, secret-leak, or operator-readiness defect. If no such defect is demonstrable without inventing new semantics, explicitly close this audit lane.

Then re-read current status/plan/roadmap/release packet. If another named READY_LOCAL output exists, implement it. Otherwise the Agent should propose 1-3 specific local runtime/operator/release options grounded in current code/evidence, each with observed gap/output, owning file/API, protected invariant, risk, minimum positive/negative test, and whether it changes architecture/security policy; autonomously select the smallest option that needs no maintainer value decision.

Do not reopen generic DeliveryLedger auditing, package archive variants, previous-release compatibility, FEC/0-RTT/striping/multipath/exotic carriers, or generic checker/parser infrastructure without an observed problem.

### F. CONDITIONAL VPS OUTPUT — only if a genuinely new live question opens

Current opportunity truth remains `READY_LIVE: none`. If E or later repository truth creates a **new concrete unresolved real-network question** whose code/config/instrumentation/path/hypothesis materially differs from retained evidence, execute it boundedly under standing authorization and preserve negative results.

Otherwise run nothing merely to use rental time. Do not unchanged-rerun HY2, repeated warm failover, periodic, installed-package lifecycle, distinct A->B->A, generic soak, IPv6 without a real owned IPv6 path, or live PMTUD before its separate authenticated wire/security design gate.

### G. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Non-policy engineering controls already have bounded factual review support. Terminal source-retention/no-reset semantics still require maintainer/security-policy judgment. Do not invent retention TTL, LRU/history capacity, external authority, or weaker reset semantics. This blocker does not prevent independent A-F work.

## Governance boundaries

- `IMPLEMENTATION_COMPLETE=true` remains bounded research-implementation status only.
- release item 3 remains incomplete; item 4 remains incomplete because independent review and D019 policy closure are absent.
- standing VPS authorization remains valid for genuinely new dependency-ready self-owned bounded questions; authorization is not the current blocker.
- HY2 current line remains its retained typed negative; no same-class retry.
- IPv6 remains environment-blocked when no owned path exists.
- signing/key custody/SBOM/publication trust remain separate release concerns; do not fabricate credentials or policy as ordinary READY_LOCAL work.

## Stop / escalation

Do not notify or stop for ordinary progress. Escalate only for a core Session/Carrier/ACK/crypto/wire architecture change; new security numeric policy; destructive/canonical-meaning migration; action outside standing authorization; production impact; new credentials/server/third-party permission; benchmark value judgment; unresolved major security issue; D019 policy decision; or entry into a genuinely new project stage.
