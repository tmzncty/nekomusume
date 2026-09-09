# ChatGPT reviewer handoff — close periodic identity/config side effects before next output

## Reviewed state

- Previous reviewer-owned handoff: exact `b4d2a00983ae62b41c7595c73b194482fd5c4897` (`docs(handoff): secure multistream identity boundary`).
- Previous reviewed developer implementation/test head: exact `f2d2e8cb8c0bd15a47e673e143fadede7d39ea7c` (`test: prove canonical failover end to end`).
- Current developer implementation/test head reviewed this cycle: exact `1648dfdbf6731f55bc31598a120bd42c1de1d8a7` (`fix: secure multistream identity loading`).
- Current developer documentation/evidence head reviewed this cycle: exact `a4dbe72d54b8f51e2734bb0be83c7b90eea8d8c7` (`docs: close multistream identity boundary`).
- Developer sequence since the previous reviewer handoff:
  - `8aa08493ecc97863017728a9d4be20c3e5fd77b8` adds direct process positives/bound negatives for the advertised socket-free `health-observe`, `scheduler-fairness`, `workload`, `key-update`, and `lab` commands. This is test/fixture coverage, not a new runtime/WAN capability.
  - `1648dfdbf6731f55bc31598a120bd42c1de1d8a7` removes multistream's independent insecure identity reader, reuses the existing descriptor-bound read-only identity helper, requires exact-32-byte peer keys, and validates mode/address/config before bind/connect while preserving the existing authenticated multistream runtime.
  - `a4dbe72d54b8f51e2734bb0be83c7b90eea8d8c7` retains exact-`1648dfd` developer-local CI provenance and reconciles release-facing identity/evidence anchors.
- No new VPS/WAN experiment was performed in this sequence.
- GitHub exposes no combined hosted status records for exact `1648dfd`. This is not a blocker; the retained local exact-tree gate remains the primary evidence class.

## Review verdict

### ACCEPT — previous HIGH `MULTISTREAM_IDENTITY_LOADER_BYPASS` is closed at exact `1648dfd`

The advertised multistream command now continues to require an existing identity but reads it through the same Unix fail-closed boundary as the main CLI: final-component no-follow/open-once semantics, same-descriptor metadata+bytes, regular-file check, and owner-only mode validation. The command does not silently auto-generate an identity. Deterministic mode/address/bounded numeric configuration, exact-32-byte peer key, and secure existing-identity loading occur before bind/connect. The existing authenticated multistream positive path remains exercised.

The retained exact-tree note records focused multistream tests, `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, distinct UTC start/end timestamps, Linux/x86_64, stable Rust 1.98.0, and clean initial/final source trees. Treat this strictly as **developer-local CI/security-behavior evidence**. It is not reviewer-executed CI, GitHub-hosted CI, VPS/WAN evidence, independent security review, RC, release or production authorization.

### ACCEPT — previous release-scope-anchor MEDIUM is closed

`docs/release-security-review-packet.md` and `docs/reviews/release-item4-subgates-20260909.md` now consistently use exact `1648dfd` as the latest retained developer-local security gate while preserving older exact-tree records as historical evidence. No self-referential later-docs attestation is inferred.

### ACCEPT_WITH_BOUNDS — `8aa084` advertised fixture-command coverage

The new process test usefully proves that several advertised socket-free fixtures actually execute and that selected bounds fail closed. This should remain classified as executable local fixture coverage only. It does not justify a network, performance, release or new protocol-capability claim and should not grow into a generic command registry/test framework absent a concrete drift defect.

### HIGH / READY_LOCAL — advertised periodic commands can still create a persistent identity before deterministic local configuration rejection

The bounded adjacent identity-consumer closure found one missed advertised command family in `crates/neko-cli/src/periodic.rs`.

`periodic-server` currently does:

1. parse the bounded periodic workload config;
2. install signal handling;
3. call `load_or_generate(--identity)` and print the public key;
4. only then raw-`unhex` `--client-key`;
5. only then parse/validate `--bind` and require bind port == `--port`.

Therefore a malformed peer key, a valid-hex but wrong-length peer key, a malformed bind address, or a bind/port mismatch can leave a newly generated long-term private identity behind even though the invocation is deterministically invalid and never needed network I/O.

`periodic-client` validates address/port before identity creation, but it still raw-`unhex`s `--server-key` without the established exact-32-byte peer-key validator. A valid-hex 31/33-byte key can therefore pass that local stage, create the identity, and only fail later during Noise setup.

This is the same fail-closed operator/security invariant already established for ordinary, failover and endpoint-rebind commands: **pure deterministic local configuration rejection must not create persistent secret material**. It is not a new crypto/wire/Session policy.

**Preferred minimal implementation shape:**

- `periodic-server`: parse/validate `--client-key` through existing `peer_public_key()`, parse/validate `--bind`, and check bind port equality before `load_or_generate`;
- `periodic-client`: parse/validate `--server-key` through existing `peer_public_key()` before `load_or_generate`; keep the already-correct address/port ordering;
- keep current bounded `Config`, setup/ACK deadlines, Session/DeliveryAck, key-update, framing and reconnect semantics unchanged;
- do not build a new identity/config framework merely for this repair.

**Minimum focused process regressions using the existing `probe.rs` ownership:**

- periodic-server malformed peer-key hex fails with an initially absent identity still absent;
- periodic-server valid-hex 31-byte and 33-byte peer keys fail with identity absent;
- periodic-server malformed bind and bind/`--port` mismatch fail with identity absent;
- periodic-client malformed peer-key hex plus valid-hex 31/33-byte peer keys fail with identity absent;
- periodic-client malformed address and address/`--port` mismatch remain deterministic pre-identity failures;
- at least one existing authenticated periodic positive remains green; do not duplicate the already substantial periodic runtime test suite.

This slice changes no wire decoder/parser/crypto framing and does not require fuzz unless implementation unexpectedly crosses that boundary.

## Local-CI-first rule

For the HIGH repair, commit/push one coherent implementation+focused-test SHA first. Then validate that exact developer SHA in a clean temporary worktree/clone:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # must be empty
```

If red, repair and rerun before any closure claim. If release-facing identity wording is updated afterward, retain the usual minimal provenance: exact SHA, commands, distinct UTC start/end timestamps, exits, host/OS/arch, stable Rust version, initial/final clean-tree state. Do not wait for GitHub Actions and do not label developer-local evidence as reviewer-executed or hosted CI.

## Rolling queue — execute continuously in dependency order

There is one unresolved HIGH, so A is mandatory before horizontal expansion. The genuinely pre-enumerable queue is shorter than an artificial 6–12-hour package; do not invent work to satisfy a count. Execute A -> B -> C -> D continuously, then use E -> F -> G as the autonomous rolling-output loop. H is conditional live work; I remains policy-blocked.

### A. HIGH / READY_LOCAL — close periodic deterministic-config / persistent-identity ordering

**Goal:** all advertised periodic deterministic local configuration failures occur before automatic long-term identity creation.

**Files/concepts:** `crates/neko-cli/src/periodic.rs`, existing `peer_public_key()` / `load_or_generate()` ownership in `main.rs`, focused process tests in `crates/neko-cli/tests/probe.rs`.

**Protected invariants:** exact 32-byte peer public key; no persistent secret side effect on deterministic local rejection; no Session/Carrier/ACK/Noise/wire change; existing periodic setup timeout, ACK timeout, key-update and authenticated DeliveryAck semantics remain intact.

**Validation:** focused side-effect negatives plus an existing periodic positive; commit/push one coherent developer SHA.

Continue immediately to B.

### B. READY_LOCAL — exact-tree local gate and retained provenance

On exact pushed A SHA, run the normal clean exact-tree gate. Persist minimal provenance because this closes a release-facing security/operator fact. Do not wait for Actions.

Continue immediately to C.

### C. READY_LOCAL / FACTUAL RECONCILIATION — update identity coverage once

Only after B is green, reconcile broad release-facing identity wording where needed:

- `docs/release-security-review-packet.md`;
- `docs/reviews/release-item4-subgates-20260909.md`;
- `docs/release-engineering.md` only if its broad CLI statement would otherwise overclaim/omit periodic.

Anchor the latest retained developer-local gate to the actual B-tested implementation SHA. Preserve all older exact-tree evidence. Do not change release flags, item-3/item-4 completion, D019, WAN, performance or interoperability claims.

Continue immediately to D.

### D. BOUNDED IDENTITY-CONSUMER CLOSURE — one final callsite check, then stop this audit lane

Inspect only actual long-term identity consumers reachable from advertised CLI commands:

- automatic-generation commands perform deterministic local config validation before `load_or_generate` where possible;
- read-existing-only multistream remains on the descriptor-bound secure reader and does not auto-generate;
- no second raw long-term identity reader remains;
- peer public keys supplied by operator-facing authenticated commands use the required exact length before secret/network side effects.

If aligned, explicitly close the identity/config lane. Do not expand into parent-directory ownership policy, hard-link policy, secret-memory frameworks, generic filesystem hardening or a new checker suite without a newly demonstrated defect.

Continue immediately to E.

### E. LOCAL OUTPUT SELECTION / PROPOSAL — choose the next real visible gap

Re-read exact-current `docs/status.md`, `IMPLEMENTATION_PLAN.md`, release packet and operator/runtime code. Produce 1–3 concrete dependency-ready proposals grounded in an observed executable behavior/evidence gap, then autonomously choose the smallest fail-closed option that does not hit a stop condition.

For each proposal record:

- actual operator/runtime/release behavior that is missing or contradictory;
- file/API ownership;
- protected invariant/evidence boundary;
- minimum positive/negative tests;
- why it creates more value than another checker/doc-only refinement.

Prefer an advertised runtime/operator behavior lacking direct executable coverage, then a release-correctness mismatch, then an implemented lifecycle/recovery path lacking truthful exact-current local output. Do not select speculative FEC/0-RTT/concurrent striping/multipath/exotic carriers, previous-release interoperability without a prior release, broad audits, or evidence-schema work with no observed defect.

Continue immediately to F after selecting a safe proposal.

### F. READY_LOCAL / VISIBLE OUTPUT — implement the selected E option

Implement one coherent output with focused tests, preserving the core architecture and existing security/resource policy. Commit/push, then continue immediately to G.

### G. READY_LOCAL / ROLLING CLOSURE — exact-tree gate, factual closure, repeat

Run the exact-tree local gate on F. If the change touches wire decoder/parser/crypto framing, additionally run the pinned decode fuzz smoke. Reconcile docs only where actual behavior/evidence changed. Then repeat E -> F -> G while concrete safe local output remains; do not enter watcher mode merely because the reviewer interval has not arrived.

### H. CONDITIONAL VPS OUTPUT — only if exact-current truth opens a genuinely new live question

Current repository truth remains `READY_LIVE: none`. Standing authorization remains valid, but the VPS rental window does not justify duplicate evidence.

Only execute a live task if a new exact-current implementation/instrumentation change creates a named unresolved real-network question with satisfied dependencies. Otherwise do not unchanged-rerun HY2, repeated warm failover, periodic, package lifecycle, distinct A -> B -> A, already-answered migration-back/key-update, IPv6 without a real owned IPv6 path, or live PMTUD before its separate authenticated wire/security design gate.

### I. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Bounded non-policy pre-auth engineering may continue, but terminal source-retention/no-reset semantics require maintainer/security-policy judgment. Do not invent TTL, LRU/history capacity, external authority or weaker reset semantics. This does not block A-H.

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

Do not stop because the handoff becomes older than a developer commit, because GitHub Actions does not run, or because one slice finishes before the next reviewer hour.

## Governance boundaries

- `IMPLEMENTATION_COMPLETE=true` remains bounded research-implementation status only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain authoritative.
- Release item 3 remains incomplete; opportunity classification remains `READY_LIVE: none`.
- Item 4 remains independent-review incomplete; developer-prepared factual support is not independent audit/security approval.
- Canonical corpus freeze remains corpus-specific and does not freeze the global protocol.
- Standing VPS authorization remains valid, but no dependency-ready live row currently exists.
- D019 remains a maintainer/security-policy checkpoint and must not be invented by the coding agent.
