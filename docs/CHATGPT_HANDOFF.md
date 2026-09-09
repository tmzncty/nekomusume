# ChatGPT reviewer handoff — close canonical failover with a real positive path, then continue visible local output

## Reviewed state

- Previous reviewer-owned handoff: exact `eacd3a1e87cf6cc48e0efafc7add9956798b2e31` (`docs(handoff): close identity evidence before canonical failover CLI`).
- Previous reviewed developer head: exact `29f2132cf75cf1234021467505e0c75fea51ef66` (`docs: close cross-command identity provenance`).
- Current developer head reviewed this cycle: exact `71e85f59d453450e5065311b60eebd570d182a48` (`feat: route canonical failover roles`).
- Developer sequence since the previous review:
  - `aefd49fed7414f1cb927ba55dac235e99c1db24a` — adds the missing table-driven ordinary/failover/endpoint-rebind deterministic rejection matrix and proves invalid peer-key/address/bind input leaves an absent identity path absent.
  - `c5803b96fcfe6034818b167ce3a7978284ed918d` — persists developer-local exact-`aefd49f` CI/package provenance and reconciles release-engineering, release packet and item-4 factual anchors to that real tested tree.
  - `71e85f59d453450e5065311b60eebd570d182a48` — makes `failover --role server|client` route to the existing failover implementations, updates help, and adds missing/invalid-role and role-routing negative checks.
- No new VPS/WAN experiment was performed in this sequence.
- GitHub currently exposes no combined hosted status records for exact `71e85f5`. This is **not a blocker** and must not create polling/waiting; local exact-tree CI remains the primary gate.

## Review verdict

### ACCEPT — previous HIGH identity/config evidence gap is closed at exact `aefd49f`

The prior release/evidence correctness finding is closed.

`aefd49f` adds the focused cross-command process matrix requested by the previous handoff. The retained developer-local note records distinct UTC start/end times, Linux/x86_64 host information, stable Rust, `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, native x86_64 package smoke, archive SHA-256, and clean initial/final source trees. The release packet and item-4 factual review now consistently anchor that developer-local evidence to exact `aefd49f` rather than mixing `3164030` and `489c345`.

This evidence is **developer-run local CI/package evidence**, not reviewer-executed CI, independent security review, VPS/WAN evidence, RC, release, protocol freeze or production authorization.

The bounded identity/config audit lane is therefore closed unless a new concrete defect appears. Do not reopen parent-directory policy, hard-link frameworks, generic secret-memory hardening, or another identity checker suite merely because more hardening is imaginable.

### ACCEPT_WITH_BOUNDS — canonical `failover --role server|client` implementation shape

Exact `71e85f5` chooses the smallest previously accepted CLI shape:

```text
neko failover --role server
neko failover --role client
```

`failover-server` and `failover-client` remain legacy aliases. The canonical dispatcher delegates to the existing `failover_server` / `failover_client` implementations rather than forking runtime behavior, so no Session, Carrier, ACK, negotiation, Noise, resume, accounting or wire semantic was intentionally changed.

Current source ordering remains fail closed for persistent identity side effects:

- canonical missing/unknown `--role` fails inside `failover_gate` before either role implementation and before identity creation;
- the server role validates count/bytes/duration, ports, UDP/TCP bind syntax and exact-32-byte peer key before `load_or_generate`;
- the client role validates bounded mode prerequisites, target syntax and exact-32-byte peer key before `load_or_generate`.

Help, human capabilities and JSON capabilities continue to advertise `failover` as canonical and keep the two role-specific names as aliases.

### MEDIUM / READY_LOCAL — canonical failover has no direct positive executable regression yet

The current `71e85f5` test proves that canonical dispatch reaches role-specific **error** behavior: missing/unknown role is rejected without identity creation, `--role server` reaches the missing `--client-key` failure, and `--role client` reaches the missing `--server-key` failure.

That is useful routing evidence, but it does **not** directly prove that the advertised canonical forms complete the intended bounded authenticated failover runtime. The substantial positive loopback tests still invoke `failover-server` / `failover-client` legacy aliases.

This is not a claim that the implementation is known broken. It is a focused visible-CLI regression gap. Close it without duplicating the entire failover suite:

- adapt one existing bounded positive loopback failover test, or add one small focused test, so the server is launched as `failover --role server` and the client as `failover --role client`;
- prove one existing success contract end-to-end (prefer the controlled UDP stop -> TCP resume path because it is short and already stable);
- retain one legacy-alias positive test so compatibility remains exercised;
- keep missing/unknown-role no-identity negatives green.

Do not introduce a command registry/framework merely for this test.

### MEDIUM / READY_LOCAL — latest canonical implementation lacks retained developer-local exact-tree closure

The retained exact-tree local CI/package evidence currently ends at `aefd49f`, before the canonical CLI implementation commit. There is no persisted developer-local exact-`71e85f5` gate, and no hosted status is currently exposed for that SHA.

After the positive canonical regression is committed, validate the **final pushed implementation/test SHA**, not `71e85f5` by historical implication. Use a clean temporary checkout/worktree and the local-CI-first gate below. Persist provenance if it is used for release/operator factual claims.

This is a closure requirement, not an instruction to wait for GitHub Actions.

## Local-CI-first rule

For coherent READY_LOCAL implementation/test commits, do not poll or wait for GitHub Actions. Validate the exact pushed developer SHA in a clean temporary worktree/clone.

Minimum default gate:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # must be empty
```

When the result anchors release-facing facts, retain exact SHA, commands, distinct UTC start/end timestamps, exits, host/OS/arch, stable Rust version, and initial/final clean-tree state. Keep these evidence classes distinct:

1. developer-reported local CI;
2. repository-persisted local-CI provenance;
3. reviewer-executed checks;
4. GitHub-hosted CI.

The canonical CLI dispatch/test slice does not touch wire decoder/parser/crypto framing, so no additional fuzz requirement is introduced. If a future selected slice does touch those boundaries, use the pinned fuzz toolchain and required decode smoke.

## Rolling queue — execute continuously in dependency order

There is no unresolved BLOCKER/HIGH. The real dependency-ready queue is smaller than 6–12 hours of pre-enumerable work, so do not invent fake slices. Execute A -> B -> C -> D continuously, then E -> F -> G -> H. At H, if no concrete local output exists, run the required 1–3-option proposal cycle and immediately implement the selected smallest safe option instead of entering watcher mode. I is conditional live work; J remains policy-blocked.

### A. READY_LOCAL / VISIBLE OUTPUT — prove canonical failover positive routing

**Goal:** demonstrate that the canonical command is not merely parseable but actually drives the existing bounded failover runtime.

**Files/concepts:** `crates/neko-cli/tests/probe.rs`; `crates/neko-cli/src/main.rs` only if the positive regression exposes a real routing defect.

**Protected invariants:** canonical dispatch reuses existing failover runtime; Session/Carrier/ACK/Noise/resume semantics unchanged; aliases remain compatible; deterministic invalid role/config still has no persistent identity side effect.

**Minimum validation:** one end-to-end loopback success using `failover --role server` and `failover --role client`, plus existing alias coverage and existing role/config negatives.

**Commit/push:** one coherent developer-owned test/repair commit. Continue immediately to B.

### B. READY_LOCAL — exact-tree local gate for A

On the exact pushed A SHA in a clean temporary checkout:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain
```

If all pass, record the exact tested SHA and clean-tree state. Do not wait for Actions. If the gate fails, repair and rerun before any closure claim.

Continue immediately to C.

### C. READY_LOCAL / OPERATOR EVIDENCE — minimal packaged canonical smoke only if needed for a release-facing claim

If the CLI/release docs are going to state that the canonical command is exact-current package behavior, build one native x86_64 package outside the source tree and run a **minimal** smoke that proves the packaged binary exposes the same `failover --role server|client` contract. Do not rerun a full VPS failover or create a generic checker.

If release-facing docs do not need a new packaged assertion, this slice may be skipped honestly; the local exact-tree executable regression from A/B is sufficient for code correctness.

Continue immediately to D.

### D. READY_LOCAL / MILESTONE RECONCILIATION — close canonical CLI contract without claim inflation

Update only facts materially changed by A-C. Suitable locations are `docs/release-engineering.md`, `docs/release-security-review-packet.md`, `docs/reviews/release-item4-subgates-20260909.md`, or a small local CLI provenance note **only if** those documents actually need the new canonical command as release-facing evidence.

Preserve all governance boundaries:

- `RELEASE_CANDIDATE=false`;
- `PRODUCTION_READY=false`;
- `FREEZE=false`;
- `RELEASED=false`;
- release item 3 remains incomplete;
- independent item 4 remains incomplete;
- D019 remains policy-blocked;
- no WAN, failover-performance, interoperability or production claim follows from a local CLI command repair.

After this, explicitly close the canonical CLI contract lane and continue immediately to E.

### E. BOUNDED REVIEW CLOSURE — one final command-surface check

Perform one bounded inspection only of:

- help vs human capabilities vs JSON capability inventory;
- canonical `failover` vs legacy aliases;
- missing/unknown role side-effect ordering;
- canonical positive route to the existing role runtime;
- unknown command failure.

If these align, close this lane. Do not expand into a generic parser/registry refactor unless a concrete defect requires it.

Continue immediately to F.

### F. LOCAL OUTPUT SELECTION / PROPOSAL — identify the next real visible gap

Re-read exact-current `docs/status.md`, `IMPLEMENTATION_PLAN.md`, release packet and code. Propose 1–3 **specific** dependency-ready non-policy outputs grounded in an observed code/evidence gap. Each proposal must name:

- the operator/runtime/release behavior;
- files/API ownership;
- protected invariant/evidence boundary;
- minimum positive/negative tests;
- why it is more valuable than another checker/doc-only refinement.

Prefer, in order:

1. a real runtime/operator behavior that is advertised but lacks a direct executable path/test;
2. release correctness where package/operator behavior and documented contract disagree;
3. a bounded lifecycle/recovery behavior already implemented but lacking a truthful exact-current local output.

Do not select speculative FEC/0-RTT/striping/multipath/exotic carriers, previous-release compatibility without a prior release, generic evidence schemas, or another broad DeliveryLedger audit without a newly observed problem.

Autonomously select the smallest fail-closed option and continue immediately to G. Do not wait for reviewer approval unless the selected shape hits a true stop condition.

### G. READY_LOCAL / VISIBLE OUTPUT — implement the selected F option

Implement one coherent slice with focused tests. Preserve Session/Carrier/ACK/crypto/wire architecture unless the task is explicitly escalated for design review. Do not add new policy numbers.

Run relevant focused tests during implementation, commit/push, then continue immediately to H.

### H. READY_LOCAL — exact-tree gate and factual closure for G

Run the normal exact-tree local gate on the pushed G SHA. If G touches wire decoder/parser/crypto framing, also run the pinned decode fuzz smoke required by repository policy. Persist provenance only when needed for release-facing facts.

Reconcile status/docs only where the actual behavior/evidence changed. Then repeat F -> G -> H with the next concrete dependency-ready local output while safe work remains; do not enter watcher mode merely because the reviewer interval has not arrived.

### I. CONDITIONAL VPS OUTPUT — only when exact-current truth opens a genuinely new live question

Current repository truth remains `READY_LIVE: none`. Standing authorization is valid, but authorization alone is not a reason to consume the VPS rental window with duplicate evidence.

Only execute a live task when exact-current code/evidence creates a named unresolved real-network question with satisfied dependencies and materially changed code/config/instrumentation/path/hypothesis. Otherwise do not unchanged-rerun:

- HY2 current line;
- repeated warm failover current line;
- periodic current line;
- installed-package lifecycle;
- distinct A -> B -> A package rehearsal;
- migration-back or live key-update bounded questions already answered;
- IPv6 without a real owned IPv6 path;
- live PMTUD before its separate authenticated wire/security design gate.

If a new READY live row appears, obey `docs/standing-vps-lab-authorization.md`, retain exact parameters/evidence/cleanup, and keep negative results.

### J. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Bounded non-policy pre-auth engineering controls may continue to be reviewed, but terminal source-retention/no-reset semantics require maintainer/security-policy judgment. Do not invent TTL, LRU/history capacity, external authority or weaker reset semantics. This does not block A-I.

## Stop conditions

Stop continuous coding only for a real condition:

- unresolved BLOCKER/HIGH correctness/security/evidence finding;
- a change to core Session/Carrier/ACK/crypto/wire architecture;
- destructive/canonical-meaning migration;
- action outside standing authorization;
- production impact;
- new credentials/server/third-party permission required;
- benchmark conditions requiring maintainer value judgment;
- repository/tool/runtime breakage that prevents safe progress;
- actual runtime/tool-budget exhaustion;
- or a genuinely exhausted rolling queue after the required local proposal cycle finds no concrete safe work.

Do not stop because `docs/CHATGPT_HANDOFF.md` is older than the newest implementation commit, because GitHub Actions did not run, or because one coherent slice finished before the next reviewer hour.

## Governance boundaries

- `IMPLEMENTATION_COMPLETE=true` remains bounded research-implementation status only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain authoritative.
- Release item 3 remains incomplete; current opportunity classification remains `READY_LIVE: none`.
- Item 4 remains independent-review incomplete; developer-prepared factual support is not an audit/security approval.
- Canonical corpus freeze remains corpus-specific and does not freeze the global protocol.
- Standing VPS authorization remains valid, but no dependency-ready live row currently exists.
- D019 remains a maintainer/security-policy checkpoint and must not be silently invented by the coding agent.
