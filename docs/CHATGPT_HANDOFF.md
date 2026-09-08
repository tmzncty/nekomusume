# ChatGPT reviewer handoff — local closure accepted; stop growing audit scaffolding and advance exact-tree package/VPS operator evidence

## Reviewed state

- Previous reviewer handoff: exact `8bf5a5c66cbb0d01ed01c64091cc18d2cb3cb127` (`docs(handoff): accept HY2 truth and refine closure queue`).
- Current default `main` observed during this review: exact `eecd06863f36488c471a4e0500e0692687789d9c` (`docs: record current package operator closure`), parent `f19ad280b7363b6f4a86c7c34dc50e2b12b92ea3`.
- New developer-owned work reviewed since the prior handoff:
  - `b36d153c9e297fbe0ed9922b83508e9ca3316609` — `test: lock legacy failover digest exception`;
  - `eec023cb4b2f0ac59dde0b289a59a45c92fb8c76` — `test: bound multistream server startup`;
  - `6952fa9928d895fcd3f9e51bfeebbe3719660488` — `test: isolate sibling pending owner proofs`;
  - `f19ad280b7363b6f4a86c7c34dc50e2b12b92ea3` — `docs: record bounded preauth engineering review`;
  - `eecd06863f36488c471a4e0500e0692687789d9c` — `docs: record current package operator closure`.
- Exact-head Rust CI is green for `b36d153` (`34262392377`), `eec023c` (`34263155476`), `6952fa9` (`34264256803`), and `f19ad28` (`34265058562`). The exact-`eecd068` run `34266873823` was still in progress when this handoff was written; it is a documentation-only child of already-green `f19ad28`, so it does not block the agent from preparing the next slice, but do not call exact `eecd068` CI green until GitHub actually says so.
- No work branch is ahead of `main`. `work/e1a-staged-accounting-20260907` remains stale at `f4404257520e9a014ac4e785b0ab9a97f8aaf794`; do not coordination-merge it.

## Review verdict

### ACCEPT — multistream startup race is closed narrowly and correctly

`eec023c` replaces the fixed 50 ms child-start sleep with a bounded connect-with-deadline loop that retries only listener-start `ConnectionRefused` and returns the first successful connection as the actual negotiation test connection. There is no sacrificial readiness connection for the one-shot server, and no production negotiation/Noise/Session/Carrier behavior changed. Keep this shape; no further generic process-readiness framework is requested.

### ACCEPT — repeated-failover legacy digest exception is locked to the one frozen historical value

`b36d153` keeps Draft 2020-12 validation over the retained corpus, accepts canonical 64-hex SHA-256 values, accepts only the exact frozen legacy malformed digest, and rejects sibling malformed/non-64-hex examples. This is the correct compatibility boundary. Do not rewrite the historical artifact and do not reopen repeated-warm-failover WAN because of schema work.

### ACCEPT — pending UDP ownership is now explicitly modeled as a shared lifecycle

`6952fa9` makes the two pending UDP inventory entries explicitly share one bounded producer/consumer/expiry lifecycle and adds mutation negatives so sibling producer/store, consumer cancellation, or expiry cleanup text cannot rescue a broken intended region. Manual exact-tree review of the current failover-UDP paths also finds the authenticated dequeue/release path and the new-admission rejection/enqueue-failure/non-hello releases present.

A residual review-precision limitation remains: some non-pending generic inventory anchors are still whole-file membership checks. That is **not current evidence of a runtime leak** and is no longer a reason to keep extending the checker. Reopen checker precision only if a concrete exact-tree review defect or false pass is demonstrated. The anti-audit-infrastructure rule applies now.

### ACCEPT WITH STRICT BOUNDARY — pre-auth engineering controls have a bounded exact-tree reviewer pass; D019 is still not closed

`f19ad28` records `ENGINEERING_CONTROLS_REVIEWED` for exact `6952fa9`. The reviewed implementation has the expected non-policy engineering shape: opaque TCP/UDP source-domain keys, aggregate-only `Debug`, validated process limits, per-source/global pre-admission count/memory checks, charge-before-parse input accounting, charge-before-send response accounting, bounded response permits/deadlines, source/global queue bounds, rollback/terminalization on accounting errors, and release/expiry cleanup.

The important contradiction remains visible in code: `ProcessPreauthAdmission::release()` removes the source entry when its final live state disappears, so the same source can later obtain fresh source-lifetime accounting. D019 says retry/reconnect/carrier/error cleanup must not reset the source/lifetime ceiling or reopen admission after it is reached. Therefore:

- `ENGINEERING_CONTROLS_REVIEWED` is a bounded engineering/reviewer result only;
- full RSEC/D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`;
- do not invent retention TTL, LRU/history capacity, or silently weaken the no-reset wording;
- adversarial-load/capacity suitability and formal security/release approval are not implied by the deterministic engineering tests;
- this policy blocker must not stall package/operator/runtime work that is independent of source-retention semantics.

### ACCEPT AS LOCAL OBSERVATION, NOT AUTHORITATIVE VPS EVIDENCE — current-tree package rehearsal

`eecd068` documents a local exact-`f19ad28` package rehearsal: two package builds with identical archive hash, package smoke, authenticated loopback TCP and UDP exchanges, same-tree immutable-directory symlink switch/rollback, external-state marker retention, and cleanup. The document is appropriately explicit that this is local-only, not a distinct-version upgrade result, not WAN evidence, not RC, and not production readiness.

Do not inflate this prose record into the old N5-style authoritative VPS artifact set: the new commit contains the bounded narrative and hashes but not an equivalent committed raw-log/manifest directory. That is acceptable as a local observation because the next priority is a real operator/VPS result, not backfilling another local evidence framework.

The historical N5 package lifecycle remains authoritative for exact old trees only. Its parent `91a735c...` is now 300 commits behind exact `f19ad28`; since then the runtime changed materially (large changes in carrier, CLI, crypto, wire/session and responder behavior). Therefore the old N5 A→B→A result cannot by itself establish current-binary package/operator behavior. This is enough reason to spend the next VPS opportunity on the current exact package.

## Design / proposal protocol

The external coding agent remains an active designer. For architecture-internal implementation/test/harness details, compare 1–3 minimal shapes when there is a genuine choice, state invariant/risk/minimal tests, choose the least-state/least-API/least-policy solution, implement, test, commit, push, and continue without waiting for reviewer preapproval.

Do not stop on an ordinary API-granularity problem. Stop/escalate only for the real boundaries: Session/Carrier/ACK/crypto/wire semantic change, new numeric security policy, destructive migration, production/third-party mutation, new credentials/server/permissions, or a maintainer value judgment.

## Rolling queue

The prior local A–D closure package is complete enough. **Do not spend another cycle polishing checker/docs before outward work.** The next several slices should form one package/operator evidence closure, then reconcile release truth and only then select another genuinely ready question.

### A. HIGH OUTPUT PRIORITY / READY_VPS — exact-current-tree package install + authenticated WAN operator smoke

**Goal:** turn the current local package observation into one new self-owned VPS operator result for the exact current tree fetched after this handoff.

**Why now:** the VPS rental is time-limited; old N5 evidence predates roughly 300 commits and does not cover the materially changed current binary; standing authorization explicitly permits dedicated experimental install/upgrade/rollback rehearsal plus bounded self-owned TCP/UDP Session traffic.

**Files/evidence:** reuse `scripts/release/build-package.sh`, `scripts/release/smoke-package.sh`, existing probe CLI and existing remote-control/resource/cleanup helpers. Add only a compact evidence directory + summary needed to make the run inspectable. Do not create a new package framework.

**Protected invariants / authorization:**

- self-owned client ↔ self-owned VPS only;
- dedicated experimental install/state/identity paths; fresh test identity/state, never repository or production identity material;
- temporary high ports inside standing bounds; no production Hysteria/proxy/listener replacement;
- no production route/firewall/DNS/proxy/tunnel/qdisc changes;
- bounded runtime/bytes/sessions; cleanup every explicitly started listener/process and remove experimental install/state material after capture;
- package evidence is not release/security approval.

**Behavior:**

1. fetch current `main`; verify the exact tree and its CI state before WAN;
2. build the exact-current package and run existing package smoke locally; preserve build JSON/archive SHA/binary SHA/toolchain/target provenance;
3. install that package into a dedicated experimental path on the self-owned VPS, run `capabilities --json`, and verify installed binary hash against package provenance;
4. using the installed VPS binary, perform one bounded authenticated TCP smoke and one bounded authenticated UDP smoke from the self-owned client over the real client↔VPS path; preserve exact application-byte/count results rather than only process exit codes;
5. prove listener/process cleanup and remove the dedicated experimental package/state/identity path after evidence capture.

A distinct-version A→B→A rehearsal is **optional, not a prerequisite** for this slice. Do it only if a genuinely distinct known-good package can be safely obtained/rebuilt from existing repository/tooling without inventing version semantics or requiring new credentials. Do not manufacture an "upgrade" by installing the same tree twice. If no suitable prior package is available, clean install + exact-current WAN TCP/UDP smoke + cleanup is the intended closure; retain historical N5 as the separate distinct-version rollback evidence.

**Tests/gates:** package build/smoke, `scripts/check.sh`, `git diff --check`; exact-head CI green before the VPS run. Evidence-only documentation commits after a successful/negative run still need exact-head CI, but do not rerun the network experiment merely because a later docs commit changes HEAD.

**Negative rule:** if TCP or UDP package/WAN smoke fails, preserve the bounded negative and cleanup. Retry only after a concrete package/runtime/config/instrumentation hypothesis changes materially. Do not chase a PASS.

**Commit/push:** yes — commit the smallest inspectable evidence set and exact boundary summary. Continue immediately to B.

### B. READY_LOCAL — reconcile package/operator and release-evidence truth

**Goal:** make `docs/status.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `docs/release-engineering.md`, and the release-review packet agree with what A actually proved.

**Why now:** current `eecd068` adds a local package claim; A should add either real VPS operator evidence or a bounded negative. Planning truth must distinguish those layers.

**Protected invariant:** keep these distinctions explicit:

- package code exists;
- local reproducible build/package smoke;
- installed-binary provenance;
- real self-owned VPS install/operator smoke;
- authenticated WAN TCP/UDP behavior;
- distinct-version upgrade/rollback evidence;
- RC/production/security approval.

Never collapse one layer into another. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged absent a separately reviewed decision. HY2 and repeated-warm-failover same-class retries remain frozen; D019 remains policy-blocked.

**Tests/gates:** `scripts/check.sh`, `git diff --check`, exact-head CI.

**Commit/push:** yes if repository truth changes. Continue immediately to C.

### C. READY_LOCAL / OPERATOR VALUE — audit current K/package contract for one concrete missing operator behavior, not a documentation wish list

**Goal:** after A/B, identify at most one current-tree package/operator defect or missing behavior that has direct operator value and can be closed without a new architecture/security-policy decision.

**Why now:** package work should produce a usable lifecycle, not only archives and prose, but the repository already contains older lifecycle/SIGTERM evidence and some release-engineering text may be stale after later runtime work.

**Files:** re-read current `docs/release-engineering.md`, M5 release gate, lifecycle code/tests, N4/N5/N8 evidence, current CLI server behavior and any package helpers.

**Selection examples (choose only if genuinely open on current tree):** stale-listener restart behavior, explicit bounded shutdown/readiness behavior, package permission/state retention, clean uninstall/rollback cleanup, or another concrete current operator defect exposed by A.

**Do not select:** signed publication/SBOM/key-custody policy, service-manager production profile, aarch64 execution without an available authorized executor, or anything that requires pretending old evidence is current.

**Proposal protocol:** if one architecture-internal shape is clearly bounded, propose/implement/test it and continue. If no real defect is found, record no new ticket and advance to D rather than manufacturing work.

**Commit/push:** only for a real implementation/test/evidence delta.

### D. READY_LOCAL — release/evidence matrix reconciliation after the package closure

**Goal:** close any remaining matrix drift introduced by current package/runtime work, without turning matrix maintenance into its own project.

**Files:** current release/evidence matrix sources, `docs/status.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, release/security review packet as actually needed.

**Behavior:** make each row point to exact current evidence and keep stale historical evidence labeled historical. Do not upgrade any WAN/performance/security/release claim because a package smoke succeeded.

**Tests/gates:** existing governance/evidence checks, `scripts/check.sh`, `git diff --check`, exact-head CI.

**Commit/push:** yes only when drift exists. Continue to E.

### E. NEXT OUTPUT SELECTION — choose one genuinely dependency-ready runtime/VPS question

**Goal:** keep the project moving toward system behavior once package/operator closure is complete.

**Selection rule:** re-read the exact post-D status/code/tests. Prefer a standing-authorized VPS-only question with a concrete new hypothesis that local tests cannot reconstruct. If none exists, prefer a small local runtime seam that directly opens such a question and does not change core semantics.

**Important PMTUD boundary:** current `docs/spec/m2-plpmtud.md` is still a socket-free research model. The accepted PMTUD ADR explicitly says the next implementation gate must first define authenticated probe/ACK wire fields, IPv4/IPv6 overhead accounting, PTB validation, timer/cooldown constants, fragmentation semantics and event schemas, and says no live implementation/public listener is authorized until that gate is accepted. Therefore **do not treat the old `BLOCKED_IMPLEMENTATION` roadmap row as permission to implement live PMTUD.** A live PMTUD integration now crosses wire/security-policy design and is a maintainer/new-stage boundary unless a later accepted ADR already resolves those requirements.

**Do not select:** unchanged HY2 or repeated-failover reruns; speculative FEC/0-RTT/striping/multipath/exotic carriers; third-party targets; production network mutation; invented security numbers.

If no non-frozen dependency-ready runtime/VPS question exists after D, stop adding infrastructure and leave the queue shorter rather than inventing work.

### F. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Current engineering controls may remain reviewed while this lane waits. Do not modify source-retention lifetime semantics without a maintainer decision choosing a bounded authority/retention model or revising D019 explicitly. Do not let this block A–E.

## VPS / evidence priority

- Current top priority is A: exact-tree package install + real authenticated self-owned client↔VPS TCP/UDP smoke, because it is both standing-authorized and not reproduced by the new local package narrative.
- HY2 remains frozen after exact `13da094` `unknown/client_started`; no unchanged retry.
- Repeated warm failover remains frozen after repeated `startup_setup`; no unchanged retry.
- A failed package/WAN attempt is preserved as evidence and retried only after a material hypothesis/change.
- No VPS/load run may substitute for deterministic security accounting or D019 policy.

## Visible-output check

The recent window has already produced real endpoint rebinding/migration/key-update/HY2 evidence, and this round also closes three local correctness/review debts plus records a current package observation. That is enough audit/checker work. The next visible output should be an operator/VPS result or a concrete package defect repair, not another generic checker/parser/harness.

## Maintainer/admin boundary

No administrator action is required for A–D under the existing standing authorization. D019 source-retention remains the known security-policy checkpoint. Live PMTUD integration is also not a normal READY implementation slice under the current ADR because its required wire/security gate is not yet accepted; escalate only if/when the project actually chooses to enter that new design stage.