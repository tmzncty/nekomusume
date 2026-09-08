# ChatGPT reviewer handoff — accept package VPS smoke, redact endpoint disclosure, then close operator lifecycle

## Reviewed state

- Previous reviewer handoff: exact `a9928c809e493e1c7f06ba692958fee6608b4f7d` (`docs(handoff): accept local closure and advance package VPS evidence`).
- Current default `main` reviewed here: exact `e02d8341bbc1f5386dc1f2b61b7f45772da61ee4` (`docs: record current package VPS smoke`), parent `a9928c809e493e1c7f06ba692958fee6608b4f7d`.
- New developer-owned delta since the prior handoff is documentation/evidence only: `e02d834` adds the exact-current package VPS smoke record and reconciles several package/release summaries. There is no runtime/code/test change in this commit.
- Exact-head Rust CI for `e02d834` is green (`34268191355`). Previous reviewer exact `a9928c8` CI is also green (`34267236984`).
- No work branch is ahead of `main`. `work/e1a-staged-accounting-20260907` remains stale at `f4404257520e9a014ac4e785b0ab9a97f8aaf794`; do not coordination-merge it.

## Review verdict

### ACCEPT WITH STRICT EVIDENCE BOUNDARY — exact-current package VPS smoke is a useful new operator result

The new `docs/package-operator-vps-a9928c8-20260909.md` records a bounded self-owned client↔VPS package observation for exact `a9928c8`:

- package built twice with identical archive SHA-256;
- local packaged binary SHA-256 matched the installed VPS binary SHA-256;
- installed binary `capabilities --json` passed with `secret_free=true`;
- one bounded authenticated TCP package smoke and one bounded authenticated UDP package smoke each completed two 32-byte exchanges;
- server lifecycle reached `READY` and `STOPPED`;
- listener/process cleanup and removal of the dedicated experimental package/log/identity path were verified;
- existing production Hysteria was not modified.

Keep the document's own scope limits. This establishes **one exact-tree package install + bounded authenticated self-owned VPS TCP/UDP operator smoke**. It does not establish sustained WAN reliability, public/general reachability, performance superiority, distinct-version upgrade compatibility, RC, release, production readiness, or security approval. Historical N5 remains the separate distinct-version A→B→A result for older exact trees.

### HIGH / READY_LOCAL — the new evidence accidentally committed endpoint/topology literals that repository policy says should stay local/secret

`docs/standing-vps-lab-authorization.md` explicitly says the public address, keys, and private topology must not be written into the repository merely because the self-owned VPS is authorized; actual targets are to come from local/secret configuration. The newly committed package VPS evidence contains literal target/bind endpoint values in its tracked scope metadata.

This is a governance/privacy defect in the **current tree**, not a reason to invalidate the TCP/UDP experiment and not a reason to rerun it.

Required repair:

- redact the newly committed target and remote-bind literals from `docs/package-operator-vps-a9928c8-20260909.md` and any current tracked summary copy if one exists;
- replace them with opaque labels such as `self-owned-vps-A` and `private-bind-A` (or an equally non-identifying stable label);
- preserve experiment id, exact git tree, architecture, ports, package/archive/binary hashes, application counts/bytes, result boundaries, and cleanup facts;
- do **not** add a generic endpoint-redaction framework/checker just for this incident;
- do **not** rewrite Git history, force-push, or perform destructive history surgery autonomously. Current-tree redaction is the authorized repair. If the maintainer later wants historical erasure of the already-published commit object, that is a separate destructive/history decision and must be escalated.

Important execution hygiene: do not copy the disclosed literal values into the new commit message, handoff, issue, test fixture, or another tracked file. The developer can remove them by editing the known evidence lines locally.

### MEDIUM / READY_LOCAL — release/review packet has current-truth drift after the last several closures

The package rows moved forward, but several present-tense review rows still describe older states:

1. **RSEC/pre-auth:** the current bounded engineering-controls review exists, while D019 source-retention policy is still blocked. Do not keep saying the engineering review itself is wholly missing, and do not call full RSEC/D019 closed.
2. **HY2:** exact `13da094` is the current frozen line: `BLOCKED_HARNESS_CURRENT_LINE_HY2`, typed diagnostic `unknown / client_started`, no complete pair and no performance result. Exact `61a6490` is historical local-preflight evidence only.
3. **VPS/reachability summary:** endpoint rebinding, migration-back, and bounded key-update now have later accepted real-socket/VPS evidence; IPv6 remains environment-blocked; repeated warm failover remains frozen at current startup/orchestration negative; live PMTUD is not a simple READY implementation item because accepted PMTUD design documents require wire/security semantics to be resolved before live integration.
4. **Operator lifecycle wording:** current release-engineering text still says explicit graceful SIGTERM/readiness semantics are not claimed, but current code/tests already implement and deterministically test `READY -> SIGTERM -> DRAINING -> STOPPED` and same-address rebind after shutdown for TCP and UDP. Reconcile the local-code/test claim now; keep VPS installed-package lifecycle as a separate evidence layer until the next slice runs it.

This should be a small truth-reconciliation commit, not a new matrix/checker subsystem.

## Design / proposal protocol

The coding agent remains an active designer. For architecture-internal implementation/test/operator-harness details, it may compare 1–3 minimal shapes, state invariant/risk/minimal tests, then choose the least-state/least-API/least-policy shape and implement without waiting for reviewer preapproval.

Do not stop on ordinary API granularity. Escalate only for actual Session/Carrier/ACK/crypto/wire semantic change, new numeric security policy, destructive migration/history rewrite, production/third-party mutation, new credentials/server/permissions, benchmark value judgment, or a major security decision that cannot be safely resolved inside accepted ADRs.

## Rolling queue

The queue below is deliberately output-oriented. The first two slices close a real privacy/truth defect; the next visible output should be a current installed-package lifecycle observation, not more generic audit scaffolding.

### A. HIGH / READY_LOCAL — redact current-tree endpoint disclosure without changing experiment meaning

**Goal:** remove the newly committed target and remote-bind literals from current tracked package VPS evidence while preserving all non-sensitive provenance/results.

**Why now:** the package run is valid, but the tracked evidence violates the repository's own standing-lab privacy boundary.

**Files:** primarily `docs/package-operator-vps-a9928c8-20260909.md`; inspect current tracked summaries for copied endpoint literals and redact only if present.

**Protected invariants:**

- no change to experiment id, exact tested tree, package/archive/binary hashes, architecture, ports, payload/counts, TCP/UDP outcome, cleanup, or evidence scope;
- opaque endpoint labels only;
- no history rewrite/force-push;
- no WAN rerun.

**Tests/gates:** locally verify the removed literal values no longer occur in the current working tree without putting those literals into a committed test; run `scripts/check.sh` and `git diff --check`.

**Commit/push:** yes, one bounded redaction commit. Continue immediately to B.

### B. READY_LOCAL — reconcile release/security/operator truth to the exact current tree

**Goal:** remove stale present-tense statements without inflating claims.

**Files:** `docs/release-security-review-packet.md`, `docs/release-engineering.md`, and only the minimum needed portions of `docs/status.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md` if they disagree.

**Required truth:**

- package: exact-current local reproducibility + one bounded self-owned VPS install/authenticated TCP+UDP smoke; historical distinct-version N5 remains separate;
- RSEC: `ENGINEERING_CONTROLS_REVIEWED` / D019 `SOURCE_RETENTION_POLICY_BLOCKED`; not security approval;
- HY2: current exact `13da094`, `unknown / client_started`, no complete pair, same-class retry frozen;
- repeated warm failover: frozen; no unchanged rerun;
- endpoint rebinding/migration-back/key-update: retain their accepted bounded evidence boundaries;
- local operator lifecycle: current code/tests cover signal-driven READY→DRAINING→STOPPED and port rebind; this is local deterministic evidence, not yet installed-package/VPS lifecycle evidence;
- release flags remain false.

**Tests/gates:** `scripts/check.sh`, `git diff --check`, exact-head CI.

**Commit/push:** yes if truth changes. Continue immediately to C.

### C. HIGH OUTPUT PRIORITY / READY_VPS AFTER GREEN — current installed-package SIGTERM + stale-listener restart/rebind closure

**Goal:** answer a concrete operator question that the new package smoke did not answer: does the exact-current **installed package binary on the self-owned VPS** shut down cleanly on SIGTERM and immediately restart/rebind on the same experimental port without leaving stale process/listener state?

**Why now:** local current-tree tests already exercise this lifecycle, but installed-package/VPS behavior is the valuable missing layer. This is a materially distinct operator hypothesis, not a mechanical rerun of the previous TCP/UDP package smoke.

**Files/evidence:** reuse existing package build/smoke, CLI machine lifecycle events, existing remote execution/cleanup helpers. Add only a compact evidence summary/artifact set; no service-manager framework.

**Behavior (one bounded closure package):**

1. fetch exact current `main`; require exact-head CI green before WAN execution;
2. build/smoke the exact-current package and record archive/binary identity;
3. install into a dedicated temporary experimental path on the self-owned VPS using fresh temporary test identity/state;
4. for TCP and UDP separately on unprivileged experimental ports: start the installed server, wait for machine `READY`, send SIGTERM, require clean exit plus `DRAINING`/`STOPPED`, verify no listener remains;
5. immediately restart on the same port and perform one bounded authenticated 32-byte client exchange to prove rebind/restart is operational, not merely that `ss` became empty;
6. stop/cleanup and verify zero experimental listener/process remains; remove dedicated package/state/identity paths;
7. record endpoint identity only as opaque labels — never raw target/bind literals.

**Protected invariants / standing authorization:** self-owned endpoints only, <=10 min, tiny traffic, <=32 sessions, high ports, no production Hysteria/proxy/route/firewall/DNS/tunnel/qdisc changes, no production identity/data.

**Negative rule:** a failure is valid evidence. Preserve the exact failed phase (`READY`, signal handling, exit, listener release, restart bind, authenticated exchange, or cleanup). Do not retry until a concrete code/config/instrumentation hypothesis changes materially.

**Tests/gates:** focused lifecycle tests + package smoke + `scripts/check.sh` + `git diff --check`; exact-head CI green before WAN.

**Commit/push:** yes — smallest inspectable result and boundary summary. Continue to D.

### D. READY_LOCAL — reconcile package/operator evidence after C

**Goal:** make package/release/status docs distinguish deterministic local lifecycle tests from installed-package VPS lifecycle evidence or a bounded negative.

**Protected invariant:** do not turn successful signal/restart smoke into daemon/service-manager/production-readiness evidence. If C fails, retain the negative and exact failed phase; no pass-chasing.

**Tests/gates:** existing governance/evidence checks, `scripts/check.sh`, `git diff --check`, exact-head CI.

**Commit/push:** yes only if repository truth changes. Continue to E.

### E. READY_LOCAL / REVIEW SUPPORT — exact-current release/security subgate review, no new audit framework

**Goal:** prepare one concise exact-current factual review of open release item-4 subgates after A–D: package provenance/rollback boundary, operator lifecycle, canonical-vector status, comparison methodology, RSEC engineering controls vs D019 policy block.

**Behavior:** run current full gates and inspect exact code/docs/evidence. Update developer-owned review material only where facts changed. Do not self-certify formal independence/security approval; reviewer will independently challenge/accept the result next cycle.

**Do not grow:** generic checker/parser/harness infrastructure unless a concrete false pass or runtime defect is first demonstrated.

**Commit/push:** only for a factual review delta. Continue to F when a genuine output question exists.

### F. CONDITIONAL OUTPUT — distinct-version current package rehearsal only if a real, safe prior package is available

**Goal:** if existing repository/tooling can safely produce one genuinely different known-good historical package, exercise A(old)→B(current)→A(old) with immutable release directories, external state/identity retention, different binary hashes, bounded authenticated workload, and cleanup.

**Gate:** this is conditional. Do not fake upgrade by installing the same tree twice. Do not invent state-migration semantics. If a safe distinct prior package cannot be obtained from already-authorized repository/tooling, skip this slice entirely.

**VPS boundary:** same standing authorization as C, and opaque endpoint labels only.

**Commit/push:** only if a real distinct-version rehearsal is executed.

### G. NEXT OUTPUT SELECTION — choose only a genuinely dependency-ready runtime/VPS question

After A–F, re-read exact current status/code/tests and choose a real unanswered question. Prefer a standing-authorized VPS-only question with a concrete hypothesis that local tests cannot answer. If none exists, leave the queue shorter instead of inventing work.

**Do not select:**

- unchanged HY2 rerun (`13da094` remains frozen);
- unchanged repeated-warm-failover rerun;
- IPv6 work without a real IPv6 environment;
- FEC/0-RTT/striping/multipath/exotic carriers without an observed problem;
- live PMTUD implementation before its required wire/security design gate is explicitly accepted;
- any new security numeric policy;
- third-party or production network mutation.

### H. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Current engineering controls can remain reviewed while this waits. Do not invent retention TTL/LRU/history capacity or silently weaken the no-reset semantics. This policy blocker does not block A–G.

## VPS / evidence priority

- The package VPS smoke just produced a real operator result, so do not rerun that same TCP/UDP smoke unchanged.
- The next VPS value is C: signal shutdown + same-port restart/rebind with the **installed exact-current package**, which tests a new operator hypothesis.
- HY2 and repeated-warm-failover same-class attempts remain frozen.
- Failed operator evidence is preserved; no unchanged rerun.
- VPS/load evidence never substitutes for deterministic security accounting or D019 policy.

## Visible-output check

The last 24–48h did produce visible/system outputs: endpoint rebinding and migration-back evidence, key-update evidence, a typed HY2 negative, current package reproducibility, and now an exact-current package install with authenticated VPS TCP/UDP smoke. Therefore audit/checker growth is not justified. A/B are direct correctness/governance closure; after them the next visible result should be C or another concrete operator/runtime output.

## Maintainer/admin boundary

No administrator action is required for the non-destructive current-tree redaction, truth reconciliation, or bounded operator lifecycle experiment under standing authorization. **Do not rewrite published Git history autonomously.** If the maintainer wants complete historical erasure of the already-committed endpoint literals, that destructive history operation requires an explicit maintainer decision. D019 source-retention remains the known security-policy checkpoint; live PMTUD integration remains a separate wire/security design-stage boundary.