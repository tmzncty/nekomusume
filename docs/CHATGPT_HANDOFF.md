# ChatGPT reviewer handoff — repair signal lifecycle observability, then retry installed-package operator closure once

## Reviewed state

- Previous reviewer handoff: exact `ee341753b73077d95ffb09338037149a6a625aba` (`docs(handoff): redact VPS evidence and advance operator lifecycle`).
- Current default `main` reviewed here: exact `2e7f11fbd15917fee8816970693a5b62e8d6d6cf` (`docs: retain package lifecycle orchestration negative`).
- New developer-owned commits since the prior handoff:
  - `14be1c889be207d140fcd48b86ce63179ab20bbf` — current-tree package endpoint evidence redaction plus release/operator truth reconciliation;
  - `2e7f11fbd15917fee8816970693a5b62e8d6d6cf` — retain one bounded installed-package lifecycle orchestration negative.
- Exact-head Rust CI is green for `14be1c8` (`34276363497`) and current `2e7f11f` (`34277219162`). Previous reviewer exact `ee34175` CI is also green (`34273044931`).
- No work branch is ahead of `main`. `work/e1a-staged-accounting-20260907` remains stale at `f4404257520e9a014ac4e785b0ab9a97f8aaf794`; do not coordination-merge it.

## Review verdict

### ACCEPT — current-tree endpoint redaction is the correct non-destructive repair

`14be1c8` replaces the tracked package-VPS target and remote-bind literals with opaque labels while preserving experiment id, exact tree, architecture, ports, hashes, bytes/counts, results, and cleanup facts. Current default-branch search no longer finds those disclosed endpoint literals.

This closes the **current-tree** privacy/governance defect. Do not rewrite Git history or force-push autonomously; complete historical erasure of an already-published commit object remains a destructive maintainer decision. Do not copy the old literals into new commits, tests, issues, or handoffs.

### ACCEPT WITH STRICT BOUNDARY — installed-package lifecycle attempt is a valid orchestration negative

Exact `2e7f11f` retains one bounded self-owned installed-package lifecycle attempt for exact `14be1c8`. The intended question was TCP/UDP installed-package signal shutdown, listener release, same-port restart/rebind, and one authenticated exchange. The orchestration command exited nonzero before a complete machine-readable phase record existed.

Keep the exact boundary:

- no truthful claim that VPS `READY`, `DRAINING`, `STOPPED`, listener release, restart/rebind, or authenticated exchange passed or failed at runtime;
- no TCP/UDP pass claim;
- independent post-attempt cleanup observed no experimental listener/process/temp path;
- this is an incomplete **orchestration/evidence negative**, not evidence of a runtime lifecycle defect;
- no unchanged rerun.

The experiment is not invalidated and must not be deleted merely because it is negative.

### HIGH / READY_LOCAL — production signal path does not externally emit `DRAINING`, while current docs claim deterministic READY→DRAINING→STOPPED coverage

The repository now exposes a concrete implementation/evidence gap that should be repaired before the next operator attempt:

- `Lifecycle` has explicit `Ready`, `Draining`, `Stopped`, and `Failed` states; `drain()` performs `Ready -> Draining`.
- `server()` emits `READY` through the lifecycle emitter.
- on signal shutdown, both the UDP shutdown branch and the shared end-of-listener shutdown path call `lifecycle.drain()` and then immediately `lifecycle.stopped()` before printing only `STOPPED`.
- therefore the externally observable server stream currently has no `DRAINING` lifecycle event on those signal paths.
- `sigterm_after_ready_stops_and_releases_tcp_and_udp_bindings` asserts `READY`, process success, `STOPPED`, and successful same-address TCP/UDP rebind, but it does **not** assert `DRAINING`.

Yet current `docs/release-engineering.md` and the release/security packet say deterministic CLI tests cover signal-driven `READY -> DRAINING -> STOPPED`. That statement is stronger than the current executable/process evidence.

This is a small runtime-observability/test defect, not a Session/Carrier/ACK/crypto/wire change. It also provides a concrete materially changed hypothesis for **one** later installed-package VPS lifecycle retry after repair and green CI. Do not claim that missing `DRAINING` caused the exact `2e7f11f` orchestration negative; that run stopped before a complete phase record, so root cause remains unknown.

### MEDIUM / READY_LOCAL — release/security review packet provenance is stale after repeated current-tree updates

`docs/release-security-review-packet.md` still labels itself as prepared at an old exact commit while its evidence index and classifications have since been modified to include much newer package/operator, RSEC, HY2, and VPS facts.

Do not create an impossible self-referential requirement that the packet name the commit containing itself. After the lifecycle implementation repair lands, use a bounded factual form such as **“evidence indexed through exact `<implementation/test commit>`; packet text updated afterward without changing that tested tree”**. Remove or clearly qualify any stale handoff digest that no longer identifies the current packet content.

RSEC wording is otherwise correctly bounded: engineering controls may be reviewed while D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`; this is not security/release/public-listener approval.

## Design / proposal protocol

The coding agent remains an active designer. The lifecycle repair below is deliberately specified by invariant, not fixed API shape. It may compare 1–3 minimal local shapes and choose the one with the least new state/API/policy, then implement/test/push without waiting for reviewer preapproval.

A likely minimal shape is to reuse the existing lifecycle emitter immediately after `lifecycle.drain()` and again after `lifecycle.stopped()`, optionally through one tiny shutdown helper if that prevents duplicated ordering across TCP/UDP signal paths. Do not build a generic service-manager or event framework for this.

Escalate only for real Session/Carrier/ACK/crypto/wire semantic change, new numeric security policy, destructive history/migration, production/third-party mutation, new credentials/server/permissions, benchmark value judgment, or an unresolved major security decision.

## Rolling queue

The queue is deliberately closure- and output-oriented. A/B fix a concrete current runtime/evidence defect and truth drift. C then spends the VPS on one materially changed operator hypothesis. Do not interpose generic audit infrastructure.

### A. HIGH / READY_LOCAL — make signal-driven `DRAINING` externally observable and lock ordering

**Goal:** make the production bounded server lifecycle stream truthfully expose the state machine already present in `Lifecycle`.

**Why now:** current docs claim READY→DRAINING→STOPPED process coverage, but current signal paths externally emit READY then STOPPED. This must be repaired before any new VPS lifecycle claim.

**Files:** primarily `crates/neko-cli/src/main.rs`, `crates/neko-cli/tests/probe.rs`; touch `crates/neko-cli/src/lifecycle.rs` only if a tiny helper is genuinely useful.

**Protected invariants:**

- no Session/Carrier/ACK/crypto/wire semantic change;
- signal shutdown from `READY` must externally emit ordered `READY -> DRAINING -> STOPPED`;
- `DRAINING` must have `readiness=false` and occur before `STOPPED`;
- no extra readiness connection/socket is introduced;
- normal authenticated exchange behavior remains unchanged;
- TCP and UDP listeners remain rebindable after shutdown.

**Minimum implementation/test behavior:**

1. after a signal is observed and `lifecycle.drain()` succeeds, emit the lifecycle while it is actually `DRAINING`;
2. transition to `STOPPED` and emit through the same lifecycle output contract rather than maintaining a separate hand-written state string where avoidable;
3. update the SIGTERM process test for both TCP and UDP to assert ordered lifecycle milestones `READY < DRAINING < STOPPED` and retain the existing same-address rebind proof;
4. prefer asserting exactly one `DRAINING` and one `STOPPED` in the signal case so duplicate/late lifecycle emission cannot silently pass.

**Tests/gates:** focused `neko-cli` lifecycle/process tests, `scripts/check.sh`, `git diff --check`, commit/push, exact-head CI green.

**Commit/push:** yes. Continue immediately to B; do not wait for the next nominal reviewer hour.

### B. HIGH/MEDIUM / READY_LOCAL — reconcile lifecycle claims and packet provenance to the repaired exact tree

**Goal:** make release/operator documentation describe exactly what A proves and make the review packet’s provenance auditable.

**Files:** `docs/release-engineering.md`, `docs/release-security-review-packet.md`, and only the minimum necessary `docs/status.md` / `ROADMAP.md` / `IMPLEMENTATION_PLAN.md` if current statements disagree.

**Required truth:**

- current deterministic process tests cover externally observable ordered `READY -> DRAINING -> STOPPED` and same-address TCP/UDP rebind **only after A is green**;
- exact `2e7f11f` installed-package lifecycle attempt remains an incomplete orchestration negative; do not reinterpret it as caused by the missing DRAINING emission;
- current-tree package/VPS TCP+UDP smoke remains a separate accepted bounded operator result;
- endpoint evidence stays opaque; no raw target/topology literals;
- packet provenance should identify the exact implementation/test tree whose facts it indexes, without pretending an ancient prepared-at SHA is the current packet or requiring self-reference;
- RSEC remains `ENGINEERING_CONTROLS_REVIEWED` / D019 `SOURCE_RETENTION_POLICY_BLOCKED`, not security approval;
- HY2 exact `13da094` remains frozen `unknown / client_started`, no complete pair/performance result;
- repeated warm failover remains frozen; no unchanged rerun;
- release flags remain false.

**Tests/gates:** `scripts/check.sh`, `git diff --check`, exact-head CI.

**Commit/push:** yes if truth changes. Continue immediately to C.

### C. HIGH OUTPUT PRIORITY / READY_VPS AFTER A+B GREEN — one materially changed installed-package lifecycle attempt

**Goal:** answer the still-open operator question on the **installed exact-current package binary**: signal shutdown, externally observed drain/stop, listener release, same-port restart/rebind, and authenticated functionality after restart.

**Why this retry is allowed:** A materially changes the runtime/observable lifecycle contract and tests it. That is a concrete changed implementation/instrumentation hypothesis relative to the frozen `2e7f11f` negative. This permits exactly one new bounded self-owned attempt under the standing authorization.

**Behavior:**

1. use exact current `main`; require exact implementation/test head CI green before WAN execution;
2. build/smoke package and record archive/binary identity;
3. install into a dedicated temporary experimental path on the self-owned VPS with fresh temporary test identity/state;
4. TCP and UDP separately on unprivileged experiment ports: start installed server; capture machine lifecycle output incrementally; require `READY`; send SIGTERM; require ordered `DRAINING` then `STOPPED`; require clean process exit and no listener;
5. restart on the **same** port; complete one bounded authenticated 32-byte exchange using the restarted installed binary;
6. stop and verify zero experimental listener/process remains; remove dedicated package/state/identity/temp paths;
7. retain enough incremental phase evidence that a nonzero orchestration exit still tells the last completed phase. Use the smallest inspectable shape (for example bounded JSONL/phase markers or existing captured machine stdout); do not build a general evidence framework;
8. store endpoint identity only as opaque labels.

**Protected standing-authorization boundary:** self-owned endpoints only, <=10 minutes, tiny traffic, <=32 sessions, unprivileged experiment ports, no production Hysteria/proxy/route/firewall/DNS/tunnel/qdisc changes, no production identity/data.

**Negative rule:** failure is valid evidence. Preserve the exact last completed/failed phase and cleanup truth. No unchanged retry/pass-chasing after this invocation.

**Commit/push:** yes — smallest inspectable evidence summary/artifact boundary. Continue to D.

### D. READY_LOCAL — reconcile package/operator truth after C

**Goal:** distinguish three layers without claim inflation:

1. deterministic local current-tree lifecycle/process tests;
2. historical/current package/VPS TCP+UDP smoke;
3. the prior incomplete `14be1c8` lifecycle negative plus C’s new exact-current lifecycle result or bounded negative.

**Boundary:** a successful C is still not daemon/service-manager hardening, production readiness, public reachability, security approval, or release. A failed C stays negative and freezes same-class reruns until a new material hypothesis exists.

**Tests/gates:** current governance/evidence checks, `scripts/check.sh`, `git diff --check`, exact-head CI.

**Commit/push:** yes only if repository truth changes. Continue to E.

### E. READY_LOCAL / RELEASE SUPPORT — exact-current release/security subgate review, no new audit framework

**Goal:** prepare one concise factual exact-current review of release item-4 subgates after A–D.

**Review:** package provenance/rollback boundary; operator lifecycle; canonical-vector status; HY2 methodology/current negative; negotiation/Noise/trust boundaries; RSEC engineering controls versus D019 policy block; current open M5 evidence. Run current full gates and inspect exact code/tests/evidence.

**Do not grow:** no generic checker/parser/harness infrastructure unless a concrete false pass or runtime defect is first demonstrated. Do not self-certify formal independence, security approval, RC, or release.

**Commit/push:** only for factual review delta. Continue to F when dependency-ready.

### F. CONDITIONAL OUTPUT / VPS — real distinct-version package rehearsal only if safely available

**Goal:** if existing repository/tooling can safely produce one genuinely different known-good historical package/binary, exercise `A(old) -> B(current) -> A(old)` using immutable release directories, external identity/state retention, different binary hashes, bounded authenticated workload, and cleanup.

**Gate:** skip entirely if a real safe prior package cannot be obtained from already-authorized repository/tooling. Never install the same tree twice and call it upgrade. Do not invent state-migration semantics or use production paths/data.

**VPS boundary:** standing authorization; opaque endpoint labels only; dedicated experimental install/state path.

**Commit/push:** only if a real distinct-version rehearsal is executed.

### G. NEXT OUTPUT SELECTION — choose only a genuinely dependency-ready runtime/VPS/operator question

After A–F, re-read exact current status, code, tests, ledger, and VPS window policy. Prefer a standing-authorized VPS-only question whose truth cannot be reconstructed later from loopback/netns and whose hypothesis is concrete.

**Do not select:**

- unchanged HY2 rerun (`13da094` remains frozen);
- unchanged repeated-warm-failover rerun;
- IPv6 without a real owned IPv6 environment;
- live PMTUD before its required authenticated wire/security design gate is explicitly accepted;
- FEC/0-RTT/striping/multipath/exotic carriers without an observed-problem gate;
- a new security numeric policy;
- third-party or production network mutation.

If no real READY output question exists, leave the queue shorter rather than inventing work.

### H. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Current non-policy engineering controls may remain reviewed while this waits. Do not invent retention TTL/LRU/history capacity and do not silently weaken the no-reset semantics. This maintainer/security-policy checkpoint does not block A–G.

## VPS / evidence priority

- The current installed-package lifecycle negative is retained and frozen as-is.
- A creates the required material implementation/evidence change; after A+B are green, C becomes the highest-value VPS opportunity.
- Do not rerun the old package TCP/UDP smoke unchanged; it already answered its bounded question.
- HY2 and repeated-warm-failover same-class attempts remain frozen.
- VPS/load evidence never substitutes for deterministic security accounting or D019 policy.

## Visible-output check

The last 24–48 hours produced real outputs: endpoint-rebind/migration/key-update evidence, a typed HY2 negative, current package reproducibility, an exact-current package install with authenticated VPS TCP/UDP smoke, and now an installed-package lifecycle orchestration negative. There is no justification for another audit-framework expansion. A/B are direct corrections to a real runtime/evidence contract; C is the next visible operator result.

## Stagnation check

This is **not** `STALLED_IMPLEMENTATION`: two new developer commits landed and one new VPS operator attempt was executed. The queue changes because review found a concrete implementation/evidence mismatch, not because the agent stopped progressing.

## Maintainer/admin boundary

No administrator action is required for A–G when they remain within the described architecture and standing authorization. Do not rewrite published Git history autonomously. D019 source retention remains the known maintainer/security-policy checkpoint. Live PMTUD integration remains a separate wire/security design-stage boundary and must not be smuggled into ordinary implementation work.
