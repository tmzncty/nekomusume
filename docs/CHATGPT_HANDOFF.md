# ChatGPT reviewer handoff — close multistream identity bypass before next visible output

## Reviewed state

- Previous reviewer-owned handoff: exact `b2aa019c1e35e06cd56d850efd5f19e817909ce1` (`docs(handoff): validate canonical failover positive path`).
- Previous reviewed developer implementation head: exact `71e85f59d453450e5065311b60eebd570d182a48` (`feat: route canonical failover roles`).
- Current developer implementation/test head reviewed this cycle: exact `f2d2e8cb8c0bd15a47e673e143fadede7d39ea7c` (`test: prove canonical failover end to end`).
- Current developer documentation/evidence head reviewed this cycle: exact `9603de373d7521012e47d28fb8da50103d6aebde` (`docs: record canonical failover validation`).
- Developer sequence since the previous review:
  - `f2d2e8cb8c0bd15a47e673e143fadede7d39ea7c` changes the existing controlled UDP-stop -> TCP-resume executable regression to launch the advertised canonical `failover --role server` / `failover --role client` forms rather than the legacy aliases.
  - `9603de373d7521012e47d28fb8da50103d6aebde` retains exact-`f2d2e8c` developer-local CI and native-package provenance and links the canonical path into release-facing factual material.
- No new VPS/WAN experiment was performed in this sequence.
- GitHub exposes no combined hosted status records for exact `f2d2e8c`. This is not a blocker; the retained local exact-tree gate is the primary evidence class.

## Review verdict

### ACCEPT — canonical failover positive-path gap is closed at exact `f2d2e8c`

The previous MEDIUM is closed. The existing bounded executable loopback test now drives the canonical server and client command forms through the real authenticated UDP -> TCP resume implementation instead of merely proving parser/error routing. It keeps the existing ordered-record and authenticated DeliveryAck assertions; no duplicate runtime implementation was introduced.

The retained note records developer-run local exact-tree validation of `f2d2e8c`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, distinct UTC start/end timestamps, Linux/x86_64 and stable Rust provenance, clean initial/final source trees, and a native x86_64 package smoke. Treat this strictly as **developer-local executable/package evidence**. It is not reviewer-executed CI, hosted CI, VPS/WAN evidence, failover-performance evidence, independent security review, RC, release or production authorization.

### ACCEPT — bounded command-surface closure

One bounded command-surface inspection now aligns:

- help advertises the current canonical command set;
- human capabilities reports the same canonical set and identifies `failover-server` / `failover-client` as aliases;
- JSON capabilities contains canonical commands and does not promote the two aliases into canonical entries;
- top-level dispatch routes the advertised commands;
- canonical `failover --role server|client` delegates to the existing role implementations;
- missing/unknown role and unknown command remain fail closed;
- exact `f2d2e8c` supplies a direct canonical positive path.

Close the generic CLI-inventory lane. Do not introduce a registry/refactor merely because the command table could be represented more elegantly.

### HIGH / READY_LOCAL — advertised `multistream` bypasses the secure long-term identity loader

A new concrete security/evidence defect was found while selecting the next real output.

`crates/neko-cli/src/multistream.rs` has its own `identity()` implementation that directly calls `std::fs::read_to_string(path)` and parses `private:public`. It therefore bypasses the Unix long-term identity invariants already established for the main CLI loader:

- no `O_NOFOLLOW` final-component protection;
- no same-descriptor metadata/read binding;
- no regular-file check;
- no owner-only permission check;
- a symlink or group/world-readable private-key file can be accepted by an advertised experimental CLI command.

This is not speculative hardening. `multistream` is listed in help/human/JSON capabilities, performs Noise-authenticated TCP Session work, and consumes long-term private key material. Release-facing material currently uses broad wording that “the CLI” opens existing identities no-follow and rejects insecure paths, so the implementation and evidence boundary disagree.

This HIGH is the queue head. Do not expand into another runtime feature until it is closed.

**Preferred minimal implementation shape:** reuse the existing descriptor-bound **read-existing-identity** logic from the crate root for multistream. Do not call a generating helper when `multistream --identity` is absent; multistream currently requires an existing identity and should continue to fail rather than silently create one. A tiny shared/helper extraction is acceptable if needed for ownership/visibility. Do not create a generic secret-storage framework.

While touching the same path, move deterministic multistream local configuration validation before network side effects where practical: parse/validate mode, bounded counts/windows, address, exact-32-byte peer key, and securely load the existing identity before `bind/accept` or `connect`. This is the same fail-closed operator boundary, not a new protocol design.

**Minimum tests:**

- secure owner-only regular identity is accepted far enough to reach the expected next bounded network/runtime boundary or completes an existing loopback positive;
- Unix `0644` multistream identity is rejected without modifying its bytes or mode;
- Unix symlink identity is rejected and the target remains unchanged;
- malformed/short/long peer key and malformed address fail before a listener/connect attempt where a deterministic process test can prove it without adding a harness framework;
- existing multistream positive behavior remains green.

No Session/Carrier/ACK/Noise/wire semantic is to be changed. This slice does not require decode fuzz unless implementation unexpectedly touches wire/parser/crypto framing.

### MEDIUM / READY_LOCAL — release-facing exact-tree scope anchors are internally stale

`9603de3` correctly adds the exact-`f2d2e8c` local canonical-failover evidence and updates the later exact-tree gate text, but two release-facing scope anchors still describe the older tree:

- `docs/release-security-review-packet.md` still says at the top that developer local gates were rerun through exact `aefd49f`;
- `docs/reviews/release-item4-subgates-20260909.md` still has an `aefd49f` title/scope while its exact-tree gate section now describes `f2d2e8c`.

Do not rewrite historical evidence or infer that later docs commits were tested trees. After the multistream HIGH is repaired and its final implementation/test SHA receives a clean developer-local gate, reconcile these scope anchors once to that actual tested tree (or explicitly split “review scope” from “latest retained gate” if that is the truthful intent). Avoid another chain of one-commit evidence churn.

## Local-CI-first rule

For the HIGH repair, commit/push the coherent implementation + focused tests first, then validate that exact developer SHA in a clean temporary worktree/clone:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # must be empty
```

If release-facing facts are anchored to it, retain exact SHA, commands, distinct UTC start/end timestamps, exits, host/OS/arch, stable Rust version, and initial/final clean-tree state. Keep evidence classes separate:

1. developer-reported local CI;
2. repository-persisted developer local-CI provenance;
3. reviewer-executed checks;
4. GitHub-hosted CI.

Do not wait for Actions. A docs-only reviewer/developer reconciliation after a validated exact implementation tree does not need another hosted-CI wait loop.

## Rolling queue — execute continuously in dependency order

There is one unresolved HIGH, so A is mandatory before horizontal expansion. The real pre-enumerable queue is smaller than an artificial 6–12-hour package; do not invent fake work. Execute A -> B -> C -> D -> E continuously, then use F -> G -> H as the autonomous rolling-output loop. I is conditional live work; J remains policy-blocked.

### A. HIGH / READY_LOCAL — secure multistream existing-identity loading and local config ordering

**Goal:** make the advertised multistream command obey the same long-term private-key file boundary already claimed for the CLI.

**Files/concepts:** `crates/neko-cli/src/multistream.rs`; the existing secure identity read helper in `crates/neko-cli/src/main.rs` or a minimal shared extraction; focused tests in existing CLI test ownership.

**Protected invariants:** existing identity must be regular/owner-only on Unix, final symlink must fail closed, metadata and bytes come from the same opened file, no silent identity generation for multistream, peer key is exact 32 bytes, no Session/Carrier/ACK/Noise/wire change.

**Implementation behavior:** reuse one secure existing-identity reader; prevalidate deterministic local config before bind/connect where practical; do not create a second hardening framework.

**Validation:** focused secure/insecure/symlink/peer-key/address negatives plus existing multistream positive; then commit/push one coherent developer implementation/test SHA.

Continue immediately to B.

### B. READY_LOCAL — exact-tree local gate for A

On exact pushed A SHA, clean temporary checkout:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain
```

If red, repair and rerun before any closure claim. If green, retain minimal provenance because the result will repair a release-facing security fact. Do not wait for GitHub Actions.

Continue immediately to C.

### C. READY_LOCAL / FACTUAL RECONCILIATION — repair release scope anchors once

Update only the facts made truthful by A/B:

- `docs/release-security-review-packet.md`;
- `docs/reviews/release-item4-subgates-20260909.md`;
- `docs/release-engineering.md` only if its broad CLI identity wording needs the multistream inclusion spelled out.

Anchor the latest developer-local gate to the actual B-tested implementation SHA. Keep historical `aefd49f`, `f2d2e8c`, identity, package and VPS evidence as historical exact-tree records; do not rewrite them.

Preserve governance:

- `RELEASE_CANDIDATE=false`;
- `PRODUCTION_READY=false`;
- `FREEZE=false`;
- `RELEASED=false`;
- item 3 remains incomplete;
- independent item 4 remains incomplete;
- D019 remains policy-blocked;
- a local multistream security repair creates no WAN/performance/interoperability/production claim.

Continue immediately to D.

### D. BOUNDED REVIEW CLOSURE — close the identity/CLI-adjacent lane

Inspect only the adjacent concrete surfaces:

- all current long-term identity-file consumers in `neko-cli` use the intended secure existing/create boundary;
- multistream deterministic peer key/address/config validation occurs before avoidable network side effects;
- canonical command inventory remains aligned after the repair;
- no broad release wording exceeds actual command coverage.

If aligned, close this lane. Do not expand into parent-directory policy, hard-link policy, generic secret-memory hardening, or a new identity checker suite without a newly demonstrated defect.

Continue immediately to E.

### E. LOCAL OUTPUT SELECTION / PROPOSAL — select the next real visible gap

Re-read exact-current `docs/status.md`, `IMPLEMENTATION_PLAN.md`, release packet and current CLI/runtime code. Propose 1–3 specific dependency-ready non-policy outputs grounded in an observed behavior/evidence gap. For each proposal name:

- operator/runtime/release behavior;
- files/API ownership;
- protected invariant/evidence boundary;
- minimum positive/negative tests;
- why it is more valuable than another checker/doc-only refinement.

Prefer, in order:

1. an advertised runtime/operator behavior lacking a direct executable positive path;
2. a release-correctness mismatch between actual package/operator behavior and documented contract;
3. an implemented lifecycle/recovery behavior lacking truthful exact-current local output.

Do not select speculative FEC/0-RTT/concurrent striping/multipath/exotic carriers, previous-release interoperability without a prior release, generic evidence schemas, or broad audits with no newly observed problem.

Autonomously choose the smallest fail-closed option that does not hit a stop condition and continue immediately to F.

### F. READY_LOCAL / VISIBLE OUTPUT — implement selected E option

Implement one coherent output with focused tests. Preserve core architecture and do not invent new policy numbers. Commit/push, then continue immediately to G.

### G. READY_LOCAL — exact-tree gate and factual closure for F

Run the normal exact-tree local gate on the pushed F SHA. If F touches wire decoder/parser/crypto framing, also run the pinned decode fuzz smoke. Persist provenance only where needed for release-facing facts. Reconcile status/docs only if actual behavior/evidence changed.

Continue immediately to H.

### H. ROLLING OUTPUT — repeat proposal -> implementation -> exact-tree closure

Repeat E -> F -> G while concrete dependency-ready local output remains. Do not enter watcher mode merely because the reviewer interval has not arrived. If a proposal cycle genuinely finds no concrete safe local work, record the exhausted options rather than fabricating tasks.

### I. CONDITIONAL VPS OUTPUT — only if exact-current truth opens a genuinely new live question

Current repository truth remains `READY_LIVE: none`. Standing authorization is valid, but authorization and VPS rental pressure do not justify duplicate evidence.

Only execute a live task when exact-current implementation/evidence creates a named unresolved real-network question with satisfied dependencies and materially changed code/config/instrumentation/path/hypothesis. Otherwise do not unchanged-rerun current HY2, repeated warm failover, periodic, installed-package lifecycle, distinct A -> B -> A rehearsal, already-answered migration-back/key-update, IPv6 without an owned IPv6 path, or live PMTUD before its separate authenticated wire/security design gate.

### J. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Bounded non-policy pre-auth engineering work may continue, but terminal source-retention/no-reset semantics require maintainer/security-policy judgment. Do not invent TTL, LRU/history capacity, external authority or weaker reset semantics. This does not block A-I.

## Stop conditions

Stop continuous coding only for a real condition:

- unresolved BLOCKER/HIGH correctness/security/evidence finding;
- core Session/Carrier/ACK/crypto/wire architecture change;
- destructive/canonical-meaning migration;
- action outside standing authorization;
- production impact;
- new credentials/server/third-party permission;
- benchmark conditions requiring maintainer value judgment;
- repository/tool/runtime breakage preventing safe progress;
- actual runtime/tool-budget exhaustion;
- or a genuinely exhausted rolling queue after the required proposal cycle finds no concrete safe work.

Do not stop because the handoff becomes older than a developer commit, because Actions does not run, or because one slice finishes before the next reviewer hour.

## Governance boundaries

- `IMPLEMENTATION_COMPLETE=true` remains bounded research-implementation status only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain authoritative.
- Release item 3 remains incomplete; opportunity classification remains `READY_LIVE: none`.
- Item 4 remains independent-review incomplete; developer-prepared factual support is not independent audit/security approval.
- Canonical corpus freeze remains corpus-specific and does not freeze the global protocol.
- Standing VPS authorization remains valid, but no dependency-ready live row currently exists.
- D019 remains a maintainer/security-policy checkpoint and must not be invented by the coding agent.
