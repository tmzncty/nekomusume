# Nekomusume ChatGPT Handoff

Checked at: 2026-09-04 05:00 Asia/Shanghai
Repository HEAD reviewed: `85346ce19f9941ac5c41437713e0c1aee81b2102`
Previous reviewed coding/evidence HEAD: `df4610b2915bc5be97b51133afbdf2937bfc9ae5`
Previous reviewer handoff commit: `2e6e46c24e40617963402d6e8cf64b719b545f53`

## What changed

Two coding/evidence commits landed after the previous review.

- `00ac2c1` — **periodic remote-launch diagnostic instrumentation; evidence tooling only, no Session/wire/Noise/runtime semantic change.** It adds bounded per-stream diagnostic hashes/byte counts/truncation metadata and a closed classification helper for `ssh_transport_exit`, `remote_binary_identity_reject`, `remote_exec_protocol_reject`, `server_runtime_exit_before_ready`, `start_timeout_no_terminal_evidence`, and `log_overflow`. Deterministic tests cover the class mapping.
- `85346ce` — **one final changed-hypothesis real self-owned VPS periodic attempt, retained as a typed negative; no positive Session/reliability evidence.** Exactly one live outer invocation followed one non-live structural preflight. The attempt ended before application traffic with server transport exit 255 / `ssh_transport_exit`, no readiness, no client launch, zero application traffic. Local and remote cleanup postchecks both report zero experiment-owned residue. This closes the current periodic orchestration line as `BLOCKED_ORCHESTRATION_CURRENT_LINE_PERIODIC`; an unchanged automatic retry is prohibited.

Exact current HEAD `85346ce` has GitHub Actions run `33805160164` completed successfully (`Rust CI` conclusion `success`). This is repository CI evidence, not security/release approval.

The periodic diagnostic change is directionally useful, but review found one evidence-semantic defect that must be repaired before the new result is propagated through status/ledger material:

### R-009 HIGH — `diagnostics.protocol_entered` overclaims remote-executor entry

Current `scripts/bench/run-periodic-command.py` sets:

```text
protocol_entered = (server_capture is not None)
```

`server_capture` becomes non-null immediately after the local wrapper spawns the server transport process and attaches stdout/stderr capture. It does **not** mechanically prove that the remote executor protocol was entered. The exact `85346ce` artifact therefore records:

```text
class = ssh_transport_exit
server_exit = 255
server stdout/stderr = empty
readiness_observed = false
protocol_entered = true
```

That combination is not a runtime contradiction, but the field name is an evidence claim stronger than the implementation proves. The previous handoff explicitly asked for “whether remote executor protocol was entered **if mechanically observable**”; capture allocation is not that proof.

This is an evidence-integrity defect in diagnostic metadata, not a Nekomusume transport failure and not a reason to retry the VPS experiment. Preserve the exact `85346ce` artifact immutably and add a correction/erratum rather than rewriting the negative result.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — final periodic current-line attempt is accepted as a pre-application `ssh_transport_exit` negative and the lane is closed; repair R-009 locally, reconcile the matrix, then move to the remaining HY2 diagnostic opportunity and release-review preparation.**

Do not perform another repeated-warm-failover or periodic live retry under the current instrumentation lines. Both current lines are closed as orchestration blockers. The project is not globally blocked: HY2 still has one materially changeable diagnostic path, and substantial local release/security preparation remains READY.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- Exact `25e0daa` remains the accepted bounded positive controlled warm fallback (3/3 records, 48 B, two uncertain/replayed, duplicate/lost 0, ~434 ms) and one approximately five-minute periodic direct-path sample (60 x 32 B, 60/60 confirmed). Neither is a production reliability rate or natural degradation/PTO-blackhole proof.
- Repeated warm failover current line is `BLOCKED_ORCHESTRATION_CURRENT_LINE`; no automatic retry.
- Periodic current line is now `BLOCKED_ORCHESTRATION_CURRENT_LINE_PERIODIC` from exact `85346ce`; no automatic unchanged retry.
- Exact `85346ce` proves only a pre-application SSH/server-transport failure classification plus zero-residue cleanup. It proves no Session establishment, application transfer, latency distribution, reconnect behavior, or long-lived reliability property.
- The existing artifact’s `protocol_entered=true` must not be used as proof that the remote executor protocol was entered; R-009 requires an additive correction.
- NAT/source-endpoint change, live migration-back, live key update and live PMTUD remain `BLOCKED_IMPLEMENTATION` under current executable reality. Do not implement them merely to fill a release matrix checkbox.
- Owned end-to-end IPv6 remains `BLOCKED_ENVIRONMENT` unless a real owned path appears.
- HY2 remains `BLOCKED_DIAGNOSTICS`; historical negative attempts remain valid and there is still no complete fair paired comparison.
- Endpoint/user/address/private topology, key paths, keys, raw private diagnostics and protected identity material remain local/untracked. Only bounded classifications/hashes/counts belong in tracked evidence.
- Standing authorization continues to cover bounded self-owned TCP/UDP Session, benchmark, capture, cleanup, HY2 comparison and package rehearsal within its limits. No per-run approval is required for the queue below.

## Rolling Work Queue

This queue is intentionally multi-hour. **Every dependency-satisfied slice is pre-authorized to start immediately after the previous coherent slice is validated, committed and pushed. Do not stop merely because one commit or one reviewer interval completed.** Pause only on a real stop condition from `AGENTS.md` / standing authorization / a new correctness-security-evidence blocker.

### A — Repair R-009 diagnostic semantics and preserve an additive erratum

**Status:** `READY_LOCAL` — highest priority.

**Goal:** make future periodic diagnostics distinguish local capture/process creation from actual remote-executor protocol entry, while preserving the already-retained exact `85346ce` negative unchanged.

**Likely files**

- `scripts/bench/run-periodic-command.py`;
- `scripts/bench/run-periodic-command-test.py`;
- a small additive erratum/correction artifact or evidence note referencing exact `85346ce` result/hash;
- schema/validator only if the field contract is machine-checked there.

**Required behavior**

1. Do not define remote-protocol entry from `server_capture is not None`.
2. If remote-executor entry is not mechanically observable for a failure class, report `unknown`/null or omit the claim. A truthful `capture_started=true` / transport-process-started fact may be recorded separately if useful.
3. Set remote-executor entry true only from a concrete structured marker or state transition that actually proves it. Do not parse host/user/path/key strings to infer it.
4. For `ssh_transport_exit` with empty remote output, remote-executor entry must not be true.
5. Preserve the closed diagnostic class enum, bounded private log hashes/counts and truncation fail-closed behavior.
6. Do **not** rewrite `artifacts/periodic-session/00ac2c1-final-transport-exit/result.json`. Add an erratum/note that identifies the exact artifact/hash and states that its `protocol_entered=true` field means only that capture was attached under the old implementation and is not evidence of remote protocol entry.
7. No Session/wire/Noise/carrier semantics change.

**Tests**

- SSH exit 255 + empty output => remote-protocol entry false/unknown, never true;
- local capture/process start may still be represented truthfully;
- positive remote-protocol marker, if one exists, is required for true;
- all prior periodic diagnostic/workload/success/cleanup/privacy regressions remain green.

**Validation / commit**

targeted tests -> full local gate -> `git diff --check` -> commit/push. Fuzz only if the normal gate requires it; do not manufacture a protocol-fuzz claim for evidence-only code.

**Continue immediately to B:** yes. Exact-head CI is not required before documentation-only B, but any later live HY2 slice must obey its own CI gate.

---

### B — Reconcile release/status matrices with the final periodic and repeated-failover boundaries

**Status:** `READY_LOCAL` after A.

Update exact facts only in:

- `docs/era4-e-resilience.md`;
- `docs/status.md`;
- `IMPLEMENTATION_PLAN.md`;
- `ROADMAP.md`;
- capability/evidence indexes already used by the repository.

Required reconciliation:

1. Repeated warm failover current line = `BLOCKED_ORCHESTRATION_CURRENT_LINE`; retain exact final negative and no automatic retry.
2. Periodic current line = `BLOCKED_ORCHESTRATION_CURRENT_LINE_PERIODIC`; retain exact `60cd40d` historical negative and exact `85346ce` final changed-hypothesis negative separately.
3. Preserve exact `25e0daa` positive bounded samples; do not promote them to natural degradation, reliability rate, or production long-lived proof.
4. Do not propagate the old `protocol_entered=true` field as a remote-protocol fact; link the A erratum.
5. Keep NAT/migration/key-update/PMTUD as `BLOCKED_IMPLEMENTATION`, IPv6 as `BLOCKED_ENVIRONMENT`, HY2 as `BLOCKED_DIAGNOSTICS` unless later evidence changes them.
6. Bounded release-evidence item 3 remains unchecked.
7. No RC/global freeze/production/release flag changes.

**Validation:** status/plan sync checks + `scripts/check.sh` + `git diff --check`.

**Continue immediately to C:** yes.

---

### C — Add one minimal HY2 diagnostic discriminator; do not live-retry yet

**Status:** `READY_LOCAL` after B, or while waiting on non-dependent CI.

**Goal:** turn the retained HY2 `client_exit` into a materially changed hypothesis without reopening the full comparison harness.

Reuse the pinned HY2 v2.9.3 artifact and existing fair-comparison lifecycle/security contract. Add only bounded diagnostics needed to classify the first failing HY2 pair stage, such as:

- temporary HY2 server/listener readiness;
- HY2 client transport process start/exit class;
- whether QUIC/UDP send was observed and whether a server response direction was observed when existing bounded capture metadata can prove it;
- TLS/auth/config stage only when structured logs/exit evidence support that classification;
- bounded private stderr/log SHA-256 + byte count + truncation flag.

Do not track raw addresses, credentials, certificate secrets, key paths or raw logs. Do not change firewall/route/qdisc/DNS/provider policy. Do not weaken TLS/authentication. Do not redesign the comparison workload.

**Tests:** deterministic stage-classification cases; malformed/private diagnostics never leak tracked strings; valid prefix is retained; cleanup remains fail-closed; a failed first pair cannot generate comparative median/P95.

**Validation / commit:** targeted tests -> full local gate -> push.

**Continue immediately to D after exact C-head CI green:** yes. While CI is pending, consume F/G/H independent local work rather than idle.

---

### D — One changed-hypothesis HY2/Nekomusume paired attempt

**Dependency:** C exact-head CI green.

Run exactly one bounded self-owned attempt under standing authorization:

- same owned client/VPS, route/time window, MTU/security/load class;
- pinned HY2 v2.9.3;
- deterministic 1200-byte payload;
- concurrency 1;
- 5 paired samples only if the first required Neko/HY2 pair succeeds and the existing contract permits continuation;
- fresh unprivileged experiment ports;
- total live invocation comfortably below 10 minutes;
- bounded capture only around experiment ports/window when needed;
- no production network/service changes;
- complete cleanup verification.

If the first HY2 pair fails, stop that live invocation, retain the typed new diagnostic and do **not** automatically retry again. Mark the current HY2 line blocked at the proven class.

If all required pairs succeed, retain raw rows and only the fair-contract bounded median/P95/failure/resource summary. No superiority claim from one batch.

**Commit/push:** minimal sanitized evidence + validator checks + full gate.

**Continue immediately to E:** yes.

---

### E — Build the release-evidence closure ledger / blocker map

**Status:** `READY_LOCAL` after D or an honest D block.

Create/update one machine-navigable review artifact for every declared release-evidence matrix row:

- required claim;
- best exact positive evidence;
- retained negatives;
- current classification from a closed set such as `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`, `BLOCKED_ORCHESTRATION_CURRENT_LINE`, `BLOCKED_DIAGNOSTICS`, `BLOCKED_IMPLEMENTATION_ARCHITECTURE`, `BLOCKED_ENVIRONMENT`, `OPEN_READY`;
- exact evidence needed to change classification;
- whether doing so needs a new architecture/value/environment decision.

The ledger must not silently narrow the release scope or convert inability to collect evidence into PASS.

If there is no `OPEN_READY` VPS row after D, **stop spending rental time on generic repeats** and continue F/G/H. Do not invent a new live capability solely to use the VPS.

**Continue immediately to F:** yes.

---

### F — Prepare the independent release/security review packet

**Status:** `READY_LOCAL`; may be consumed while C-head CI is pending.

This is preparation, **not** the independent review itself and does not complete item 4.

Make a bounded machine-navigable packet that links exact current evidence for:

- canonical corpus/freeze identity + executable oracles;
- negotiation compatibility policy;
- Noise/trust/authz transcript binding;
- wire/parser/fuzz evidence;
- pre-auth/resource limits;
- Session delivery ACK vs carrier/packet feedback separation;
- failover/resume positive and negative evidence with exact boundaries;
- package install/upgrade/rollback;
- operator readiness/shutdown/cleanup;
- VPS evidence matrix and blocker classifications;
- HY2 methodology/current result boundary;
- unresolved release/security findings.

Prefer links and generated indexes over duplicating normative prose. Do not label it an audit or security approval.

**Validation:** link/path consistency + existing doc/status gates.

**Continue immediately to G:** yes.

---

### G — Audit resource/abuse-limit evidence and close only small deterministic gaps

**Status:** `READY_LOCAL`.

Audit implementation/tests against `SECURITY.md` requirements:

- per-connection/global memory, CPU/rate bounds;
- UDP amplification / pre-auth admission;
- malformed lengths/counts/offsets;
- unknown version/type/frame handling;
- old/duplicate packet numbers and replay;
- fail-closed resource behavior;
- secret-safe logging and no open-proxy behavior.

First map existing tests/evidence. Do not duplicate already-covered checks. If a **small deterministic missing negative test** is found, add it and run the required gates. If a substantive design/resource-bound gap is found, record a release/security blocker; do not invent arbitrary production limits without a decision record.

No public listener or stress/capacity experiment beyond standing authorization.

**Continue immediately to H if no new BLOCKER/HIGH:** yes.

---

### H — Package/operator evidence link-integrity audit; rerun only if a real evidence gap exists

**Status:** `READY_LOCAL`.

N5/package lifecycle is already positive evidence. Do not repeat install/rollback simply because the VPS exists. Instead verify the future independent reviewer can trace:

- package/build identity and reproducibility claim;
- install -> smoke/readiness -> upgrade -> rollback sequence;
- external state retention boundary without reading protected identity material;
- listener/process cleanup evidence;
- exact commit/artifact hashes and the currently supported x86_64 first-RC scope.

If all evidence is already sufficient and linked, produce only an index/closure note. If one narrow operator assertion genuinely lacks current executable evidence and can be answered within standing authorization without touching protected identity material, a single bounded package rehearsal may be queued **only after documenting the missing question**. Do not rerun a sufficient lifecycle for freshness.

**Continue immediately to I:** yes.

---

### I — RC-scope decision dossier; terminal maintainer gate if matrix has no READY path

**Status:** `READY_LOCAL_PREPARATION`; any scope change itself is **not pre-authorized**.

If E shows item 3 cannot complete because the remaining requirements are only blocked by current-line orchestration, architecture, or environment, prepare a concise maintainer decision dossier without choosing:

1. keep the current release-evidence scope and remain pre-RC until the blocked evidence becomes genuinely available;
2. define a narrower first-RC claim and state exactly which claims are deferred/removed;
3. authorize the new implementation/environment work required to satisfy the existing matrix.

For each option state evidence consequences, compatibility/security risk, work required, and which existing artifacts remain valid.

Do not alter `IMPLEMENTATION_PLAN.md` release scope, `RELEASE_CANDIDATE`, production readiness, global freeze, or release flags until the maintainer chooses.

**Stop after I only if this is genuinely the first remaining gate requiring maintainer value judgment.**

## Completion gates for this rolling queue

- R-009 corrected additively; exact `85346ce` artifact remains immutable and is not over-interpreted.
- Repeated-failover and periodic current-line blockers are reflected consistently in status/plan/roadmap without erasing historical positives/negatives.
- HY2 gets at most one materially changed diagnostic attempt after deterministic repair + exact-head CI; no unchanged retry loop.
- Release-evidence ledger truthfully distinguishes sufficient bounded evidence, open work, orchestration blocks, implementation blocks and environment blocks.
- Independent-review packet is navigable but not mislabeled as an audit.
- Resource/abuse-limit audit closes only evidence-backed small gaps and records substantive gaps instead of inventing limits.
- Package/operator evidence is traced, not needlessly rerun.
- Governance remains: `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` unless a later explicit maintainer/reviewer decision changes the appropriate flag.

## Do not expand into

- another repeated-warm-failover or periodic retry on the closed current instrumentation lines;
- implementation of NAT rebinding, live migration-back, live key update or live PMTUD merely to fill item 3;
- IPv6 claims without a real owned IPv6 path;
- repeated HY2 attempts after the single changed-hypothesis D result;
- disabling authentication/integrity for comparison;
- production firewall/route/qdisc/DNS/proxy/tunnel changes;
- third-party targets/scanning;
- speculative 0-RTT/FEC/striping/exotic carriers without observed-problem evidence;
- reading/copying/committing protected identity or secret material;
- RC/global freeze/release/production claims from bounded research evidence.

## Questions requiring maintainer decision

none now. A maintainer decision may become required at slice I if the release-evidence matrix has no remaining `OPEN_READY` path and changing its scope/implementation/environment is the only way forward.
