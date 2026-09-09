# ChatGPT reviewer handoff — finish identity/config validation across all persistent-key command paths

## Reviewed state

- Previous reviewer-owned handoff: exact `f2a4ba021dd36260d86f710eb6a20dc68b5e2773` (`docs(handoff): finish identity config boundary before next output`).
- Current developer-owned head reviewed this cycle: exact `489c345542414aebc726bea6dc5e7e714478061f` (`fix: finish identity configuration validation`).
- `main` was exactly `489c345542414aebc726bea6dc5e7e714478061f` before this reviewer-only handoff update.
- New developer commit since the previous handoff:
  - `489c345542414aebc726bea6dc5e7e714478061f` — adds one `peer_public_key()` helper that requires exactly 32 decoded bytes, uses it on ordinary `server`/`client` and `failover_client`, makes new identity creation set exact `0600` on the already-open file before writing private bytes, and adds ordinary server/client short/long-key plus restrictive-umask regressions.
- GitHub-hosted cross-evidence exists for exact `489c345`: Rust CI run `34333768173` completed successfully. Its `stable checks` job ran `bash scripts/check.sh` successfully; its pinned nightly decode fuzz-smoke job also completed successfully. This is hosted cross-evidence only, not developer-local exact-tree CI and not reviewer-executed CI.
- No developer-local exact-`489c345` provenance note exists yet. Do not treat the hosted run as a substitute for the local-CI-first closure requested below.
- Work branches remain stale coordination artifacts:
  - `work/e1a-staged-accounting-20260907` = `f4404257520e9a014ac4e785b0ab9a97f8aaf794`; current main is 78 commits ahead and the branch has no unique work.
  - `work/continue-20260904` = `d271a99a2ab26abbcb146c411ba0fde697395abe`; it is a diverged historical branch with current main 162 commits ahead and one old branch-only commit. Do not merge either merely to manufacture work.

## Review verdict

### ACCEPT — exact 32-byte peer-key parsing for ordinary `server` / `client`

Exact `489c345` correctly centralizes static peer-key decoding in `peer_public_key()` and requires exactly 32 bytes after hex decoding. Ordinary `server` validates `--client-key` before `load_or_generate`; ordinary `client` validates address, peer key and bounded payload configuration before identity creation. The new process negatives for 31/33-byte valid-hex keys therefore close the specific ordinary server/client side-effect gap described by the previous handoff.

### ACCEPT — exact `0600` before private-byte write under restrictive umask

The new-file branch retains `create_new(true)`, then sets permissions through the already-open `File` descriptor to exact `0600` before `write_all(material)`. If the permission operation fails, private bytes have not yet been written. The isolated `umask 0777` process regression proves first creation ends at exact `0600` and a second invocation reloads the same identity. This is the intended minimal repair; do not add a broader parent-directory or filesystem policy without a concrete threat requirement.

### HIGH / READY_LOCAL — persistent-key validation is still incomplete on `failover` and `endpoint-rebind` command paths

The identity/config lane must **not** be declared closed yet. The bounded adjacent review found the same persistent-side-effect invariant still violated by other real CLI paths:

- `failover_server` calls `load_or_generate(...)` **before** parsing `--client-key`; it still uses raw `unhex()` rather than `peer_public_key()`. It also parses `--udp-bind` / `--tcp-bind` only after identity creation. Therefore malformed hex, valid-hex wrong-length trust keys, or malformed bind addresses can create a long-term identity before deterministic local configuration rejection.
- `failover_client` now uses `peer_public_key()`, but still calls `load_or_generate(...)` first and only afterwards validates the peer-key size and formats the target `SocketAddr`. A short/long valid-hex key or malformed `--addr` can therefore still leave a persistent identity behind before deterministic failure.
- `endpoint_rebind_server` calls `load_or_generate(...)` before parsing/validating `--client-key` and before its UDP bind is validated. It still uses raw `unhex()` for the trust key.
- `endpoint_rebind_client` parses the remote target before identity creation, which is good, but still calls `load_or_generate(...)` before parsing/validating `--server-key`, and still uses raw `unhex()`.

This is not a new wire/crypto/session policy. It is the same already-established operator invariant: purely deterministic local configuration failure must not create a persistent long-term secret as a side effect. Because the current release packet indexes the local identity boundary as a release/security-review subgate, this cross-command gap is a **HIGH release-correctness finding** even though it is not a remote exploit claim.

Minimal repair shape:

- before every `load_or_generate` in `failover_server`, `failover_client`, `endpoint_rebind_server`, and `endpoint_rebind_client`, parse and validate all peer public keys and all deterministic socket-address/bind syntax that can be rejected without network I/O;
- use `peer_public_key()` for all long-term remote public-key inputs; do not keep a parallel raw-`unhex()` trust path;
- a small prevalidated config struct is acceptable if it reduces ordering drift, but a simple reorder is preferred if it stays clearer and smaller;
- do not change Noise/authentication, Session, Carrier, ACK, failover or wire semantics;
- do not move actual bind/connect/network-reachability failure before identity creation unless doing so is required by the local parse contract. The requirement is deterministic **parse/validation**, not pre-performing network I/O.

Minimum process negatives, all with previously nonexistent temp identity paths and assertions that the paths remain absent:

1. `failover` server: malformed hex and valid-hex 31/33-byte `--client-key`;
2. `failover` client: valid-hex 31/33-byte `--server-key` and malformed address syntax;
3. `endpoint-rebind-server`: valid-hex 31/33-byte `--client-key` and malformed bind syntax;
4. `endpoint-rebind-client`: valid-hex 31/33-byte `--server-key`;
5. keep ordinary server/client invalid-config and restrictive-umask coverage green.

Do not continue to an unrelated output until this HIGH is closed.

### MEDIUM / FACTUAL PROVENANCE — exact `489c345` still lacks developer-local exact-tree closure

Hosted run `34333768173` is useful cross-evidence, but the repository's local-CI-first policy requires the final coherent implementation SHA to be validated in a clean local exact-tree checkout and to retain minimum provenance when used for release-facing factual closure. Because the HIGH above requires another implementation commit, do **not** spend time creating a standalone local-evidence note for `489c345`; gate the final replacement implementation SHA once after the cross-command repair.

### LOW / FACTUAL HYGIENE — stale identity evidence filename remains

`docs/local-identity-security-e5aeef2-20260909.md` still carries an obsolete/non-reachable token in its basename even though its contents now anchor real exact `0e664d6`. Rename it during the factual reconciliation and update links. Do not rerun CI or VPS evidence merely to rename the file.

## Local-CI-first rule

For coherent READY_LOCAL implementation/test commits, do not poll or wait for GitHub Actions. Validate the **exact pushed implementation SHA** in a clean temporary worktree/clone.

Minimum default gate:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # must be empty
```

Retain exact SHA, exact commands, distinct UTC start/end timestamps, exits, host/OS/arch, Rust stable version, and initial/final clean-tree state when the result anchors release evidence. Label developer-local CI, reviewer-executed checks and hosted CI separately.

The identity/config repair does not change wire decode/parser/crypto framing, so no additional fuzz requirement is introduced by this handoff. The successful hosted fuzz job for `489c345` remains only supplemental cross-evidence.

## Rolling queue — execute continuously in dependency order

There is one unresolved HIGH. Execute A -> B -> C -> D -> E continuously. After E, execute F -> G -> H without waiting for another reviewer cycle if the selected local output remains dependency-ready. I is conditional live work; J remains policy-blocked. Do not enter watcher/polling mode merely because `READY_LIVE: none`.

### A. HIGH / READY_LOCAL — make deterministic identity side effects consistent across all network command families

**Goal:** no deterministic peer-key/address/bind parse failure in ordinary, failover, or endpoint-rebind command paths may create a persistent identity.

**Files/concepts:** `crates/neko-cli/src/main.rs`, focused process coverage in `crates/neko-cli/tests/probe.rs`.

**Protected invariants:** no Session/Carrier/ACK/crypto/wire semantic change; no private-key logging; valid command paths still auto-generate identities; actual network reachability/bind/connect remains a later runtime boundary.

**Implementation:** prevalidate peer public keys with `peer_public_key()` and deterministic address/bind syntax before `load_or_generate` in `failover_server`, `failover_client`, `endpoint_rebind_server`, and `endpoint_rebind_client`. Prefer the smallest ordering/config abstraction that avoids another command-family drift.

**Tests:** the negative matrix listed in the HIGH finding plus all existing identity filesystem/config tests.

**Commit/push:** one coherent implementation/test commit. Continue immediately to B.

### B. READY_LOCAL — exact-tree local gate and minimal operator smoke for A + exact-0600 creation

Validate the exact pushed A implementation SHA in a clean temporary checkout with:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain
```

Record distinct UTC start/end, host/OS/arch, stable Rust version, exact exits and clean-tree state. No hosted-CI wait.

Because this changes packaged CLI/operator behavior, build one native x86_64 package outside the source tree and smoke only materially relevant facts:

- restrictive-umask keygen creates exact `0600` and stable reload;
- one ordinary wrong-length peer-key invocation leaves no identity;
- one failover or endpoint-rebind wrong-length/malformed deterministic invocation leaves no identity.

Do not create another generic package checker. Continue immediately to C.

### C. READY_LOCAL / FACTUAL RECONCILIATION — re-anchor identity/release review facts

Refresh only affected facts in:

- `docs/release-engineering.md`;
- `docs/release-security-review-packet.md`;
- `docs/reviews/release-item4-subgates-20260909.md`;
- the local identity/config evidence note(s).

Re-anchor the tested implementation tree to the real exact SHA from A/B and cite the retained local-CI/operator evidence. Preserve the distinction between developer-local CI, hosted cross-evidence and independent review.

Rename `docs/local-identity-security-e5aeef2-20260909.md` to an exact-`0e664d6` basename and update links. Do not invent a rerun for the renamed historical evidence.

Preserve `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, independent-review absence, D019 blockage and all WAN claim boundaries.

Continue immediately to D.

### D. BOUNDED REVIEW CLOSURE — close the identity/config lane unless one concrete defect remains

Perform exactly one bounded adjacent review over **all** `load_or_generate` call sites and their immediately preceding deterministic argument parsing for:

- parse/validation still occurring after persistent secret creation;
- raw remote-key parsing bypassing the shared exact-32-byte helper;
- secret material in stdout/stderr/logging;
- mutation-before-error on identity files;
- readiness state being announced before configuration/identity prerequisites.

If no additional concrete defect is demonstrable without inventing new filesystem/security policy, explicitly close the identity/config audit lane. Do not expand into parent-directory policy, hard-link frameworks, generic secret-memory hardening or a new checker suite.

Continue immediately to E.

### E. LOCAL OUTPUT SELECTION — propose 1–3 concrete next visible outputs and choose one autonomously

Re-read exact-current `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md`, release packet and implementation. Current release-evidence item 3 still classifies `READY_LIVE: none`; item 4 still lacks independent review and D019 policy closure. Do not invent WAN work or a fake release gate.

If no already-named dependency-ready local output exists, propose 1–3 **specific** runtime/operator/release-correctness outputs grounded in an observed code/evidence gap. For each proposal name:

- exact output/problem;
- owner file/API;
- protected invariant;
- minimum positive/negative test;
- whether it changes core architecture, wire/crypto semantics or numeric security policy.

Autonomously choose the smallest option requiring no maintainer value decision. Record the short proposal/choice in a developer-owned implementation note or commit message; do not wait for reviewer pre-approval.

### F. READY_LOCAL — implement the selected visible output

Implement the chosen bounded output as one coherent slice. Prefer a visible runtime/operator/release behavior over another schema/checker/document-only refinement. Do not reopen package archive variants, generic DeliveryLedger auditing, previous-release compatibility, FEC/0-RTT/striping/multipath/exotic carriers, or generic evidence-schema work without a newly observed problem.

Continue immediately to G.

### G. READY_LOCAL — exact-tree gate for F

Run the normal exact-tree local gate on the pushed F implementation commit. Add pinned decode fuzz only if F actually touches wire decoder/parser/crypto framing. Persist minimal provenance only when it supports a release-facing factual claim.

Continue immediately to H.

### H. READY_LOCAL / MILESTONE RECONCILIATION — record the visible output and keep moving

Update `docs/status.md` / implementation plan / release packet only if F materially changes a status, acceptance boundary or release-facing fact. Avoid documentation churn when it does not.

Then either proceed to another already-ready concrete output or repeat the bounded E proposal cycle. Do not enter watcher mode solely because the reviewer interval has not arrived.

### I. CONDITIONAL VPS OUTPUT — only if a genuinely new live question opens

Current repository truth remains `READY_LIVE: none`. Standing authorization remains valid, but authorization alone is not a reason to rerun answered/frozen rows.

Only execute live work when exact-current implementation/evidence opens a specific unresolved real-network question with materially changed code/config/instrumentation/path/hypothesis and satisfied dependencies. Otherwise do not unchanged-rerun HY2, repeated warm failover, periodic, installed-package lifecycle, distinct A->B->A, generic soak, IPv6 without a real owned IPv6 path, or live PMTUD before its separate authenticated wire/security design gate.

### J. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Non-policy engineering controls have bounded factual support. Terminal source-retention/no-reset semantics still require maintainer/security-policy judgment. Do not invent TTL, LRU/history capacity, external authority or weaker reset semantics. This does not block independent A-I work.

## Governance boundaries

- `IMPLEMENTATION_COMPLETE=true` remains bounded research-implementation status only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain authoritative.
- release item 3 remains incomplete; current opportunity classification still says `READY_LIVE: none` for remaining unimplemented capabilities.
- item 4 remains independent-review incomplete; developer factual review support is not an audit/security approval.
- canonical corpus freeze is corpus-specific and does not freeze the global protocol.
- standing VPS authorization remains valid, but no current dependency-ready live row exists.
- D019 remains a maintainer/security-policy checkpoint and must not be silently invented by the coding agent.
