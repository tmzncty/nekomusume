# ChatGPT reviewer handoff — HY2 truth reconciled; pending-owner proof partially accepted; stabilize exact-tree gate and continue closure

## Reviewed state

- Previous reviewer baseline: exact `9d6ee5b79e5da73622a5ccc02fe60a5ba9d2468c` (`docs(handoff): accept HY2 negative and reconcile current truth`).
- Default `main` at this review: exact `c1524cf6258ea1bf6c2528105a63bf42633ff14c` (`docs: reconcile HY2 current-line truth`).
- New commits since the previous reviewer baseline:
  - exact `53fb7b59cd223ad3746e3385d2faa4d71607d085` — `test: scope pending preauth lifecycle proof`;
  - exact `4ab020df33df092a04001a6d0d157cb97e87fc6a` — merge preserving reviewer handoff while integrating `53fb7b5`;
  - exact `c1524cf6258ea1bf6c2528105a63bf42633ff14c` — `docs: reconcile HY2 current-line truth`.
- No active work branch is ahead of main. `work/e1a-staged-accounting-20260907` remains stale at exact `f4404257520e9a014ac4e785b0ab9a97f8aaf794`; `work/continue-20260904` is older still. Do not coordination-merge either branch. Fetch current main and continue from current truth.
- Exact-main Rust CI run `34256096848` had an initial `stable checks` failure in the pre-existing multistream unsupported-negotiation process test, then passed after one reviewer-triggered rerun of only that failed job. The nightly decode fuzz smoke was green. The initial failure was `TcpStream::connect(...).unwrap()` receiving `ConnectionRefused` after the test's fixed 50 ms child-start sleep; the second attempt completed successfully.

## Review verdict

### ACCEPT — HY2 current-line truth reconciliation is complete

`c1524cf` closes the previous documentation/evidence HIGH. Authoritative planning text now distinguishes:

- historical exact `61a6490`: local preflight/orchestration negative, no VPS runtime evidence;
- current exact `13da094`: `BLOCKED_HARNESS_CURRENT_LINE_HY2`, one materially changed self-owned attempt with valid `nekomusume-1` success followed by `hy2-1` exit, typed `unknown` / `client_started`, no complete pair and no performance comparison.

The same-class HY2 fair-pair retry remains `FROZEN_NO_RETRY` without a concrete new hypothesis plus material code/config/instrumentation/path change. Do not add generic HY2 diagnostics merely because the current category is `unknown`.

### PARTIAL ACCEPT — `53fb7b5` materially improves pending-owner static proof, but does not yet meet the full locality contract

The new checker is useful and should be kept:

- `failover_udp_pending` and `failover_udp_new` are explicitly marked persisted pending owners;
- reserve -> persisted-store ordering is checked inside a bounded producer region;
- pending cancellation is checked inside a bounded consumer region;
- expiry queue invalidation is checked inside a bounded expiry region;
- a negative fixture proves that an identical `pending = Some(PendingUdpNegotiation...)` string outside the producer region cannot rescue a missing producer store.

This closes the previous whole-file `reserve/store` false-pass shape.

Residual MEDIUM review-precision gap: `validate_pending_lifecycle(responder, text)` still searches from the beginning of the whole file for each producer/consumer/expiry begin/end anchor, and both UDP inventory entries currently use the same lifecycle region anchors. Therefore the two entries can prove the same first matching shared lifecycle rather than independently proving that each inventory surface is bound to the intended ownership region. In addition, the generic `admission_owner_anchor`, `success_cleanup_anchor`, `expiry_cleanup_anchor`, and `rejection_cleanup_anchor` membership check remains whole-file before pending-specific validation.

This is **not evidence of a runtime pre-auth leak**. It is a static-review precision defect: the checker claim is stronger than what it presently proves.

The coding agent should use the proposal protocol rather than wait for a reviewer-designed API. Reasonable minimal shapes include:

1. represent the shared UDP pending lifecycle once, with explicit parent/producer/consumer/expiry ownership regions and have the two responder entries reference that shared lifecycle; or
2. keep per-responder declarations but add a parent/controller boundary or occurrence-scoped region so identical sibling anchors cannot satisfy the wrong responder.

Choose the shape with the least duplicated state and easiest fail-closed negative tests. Do not create a generic static-analysis framework.

### CI-FLAKE-001 — MEDIUM exact-tree gate reliability defect

The exact current tree is green after the failed-job rerun, so there is no basis to call the product/runtime broken. However the initial failure is reproducible as a code-level race shape: `executable_rejects_unsupported_only_negotiation_before_noise_or_data` spawns the server, sleeps a fixed 50 ms, then unwraps one `TcpStream::connect` attempt. A temporarily slower CI runner can therefore fail before exercising the negotiation invariant.

Fix this narrowly. Preferred minimal shape is a bounded loopback connect-with-deadline helper that retries only pre-accept `ConnectionRefused`/equivalent startup errors and returns the first successful TCP stream as the **actual test connection**. Do not make a separate readiness connection that the one-shot server could accidentally accept. The test must still fail if the server never binds within the bounded deadline, and must still prove unsupported-only negotiation is rejected before Noise/session data.

Do not treat a green rerun as a reason to leave an obvious fixed-sleep process-start race indefinitely in a release/security gate.

### RSEC / D019 boundary remains unchanged

Current honest RSEC state remains:

- `ENGINEERING_CONTROLS_PRESENT`;
- `INDEPENDENT_REVIEW_OPEN`;
- `SOURCE_RETENTION_POLICY_BLOCKED`.

The D019 source-lifetime conflict remains a maintainer/security-policy checkpoint: current cleanup can remove the final source entry and later admit the same source with fresh source-lifetime accounting, while D019's no-reset wording requires a stronger lifetime notion. Agent/reviewer must not invent TTL/LRU/history size/capacity, silently weaken D019, or claim terminal source-retention compliance.

That policy checkpoint blocks full RSEC/D019 closure only. It does not block the local engineering review, package/operator evidence, or other independent output lanes below.

## Design / proposal protocol

The external coding agent remains an active designer. For architecture-internal local implementation/test shapes, it should compare 1–3 minimal designs where useful, state the invariant/risk/minimal negatives, choose the least-state/least-API/least-policy shape, implement, test, commit, push, and continue without waiting for reviewer preapproval.

Stop only for actual architecture/security-policy/authorization boundaries: Session/Carrier/ACK/crypto/wire semantic changes, new numeric security policy, destructive migration, production or third-party action, new credentials/permissions, or a genuine maintainer value judgment.

## Rolling queue

Treat A–C as one compact **local gate closure package** rather than three disconnected audit mini-projects. If the agent completes one in 10–30 minutes with good tests, continue immediately through the package and then D/E.

### A. MEDIUM / READY_LOCAL — determinize the multistream child-start gate

**Goal:** remove the fixed-50-ms process-start race exposed by exact-main CI attempt 1.

**Why now:** exact-head eventually passed, but the release/security gate should not depend on scheduler luck.

**Files:** `crates/neko-cli/tests/multistream.rs`; a tiny test-only helper in the same test module is preferred unless an existing helper already fits.

**Protected invariant:** no production runtime, negotiation, Noise, wire, Session, Carrier, or security-policy semantics change.

**Behavior:** bounded retry of the actual test connection until server bind/readiness or deadline; no sacrificial readiness connection; unsupported-only negotiation still must fail before Noise/data.

**Tests/gates:** focused `cargo test -p neko-cli --test multistream`, repeat the focused test enough times locally to expose obvious races without turning this into load testing, `scripts/check.sh`, `git diff --check`, exact-head CI.

**Commit/push:** yes. Continue immediately to B.

### B. MEDIUM / READY_LOCAL — finish pre-auth pending-owner checker locality

**Goal:** ensure an anchor in a sibling/shared block cannot satisfy the wrong inventoried lifecycle.

**Why now:** `53fb7b5` is a good partial repair, but both UDP entries currently resolve identical lifecycle begin/end strings from whole-file origin and generic cleanup/owner membership is still whole-file.

**Files:** `scripts/check-preauth-responder-inventory.py`, `docs/preauth-responder-inventory.v1.json`, focused checker fixture/test code only as needed.

**Protected invariant:** review precision only; no runtime pre-auth behavior, no policy values, no source-retention semantics change.

**Behavior:** choose one minimal design under the proposal protocol. Prove the intended shared/per-responder ownership scope explicitly. Negative tests must include at least: an identical producer/store anchor in a sibling region cannot rescue the intended lifecycle; an identical consumer cancellation elsewhere cannot rescue a missing intended cancellation; an expiry cleanup outside the intended expiry region cannot satisfy it. Region-scope generic cleanup/owner anchors where the inventory claims responder-local proof.

**Tests/gates:** focused checker, mutation/fixture negatives, `scripts/check.sh`, `git diff --check`, exact-head CI.

**Commit/push:** yes. Continue immediately to C.

### C. MEDIUM / READY_LOCAL — lock the one legacy repeated-failover digest exception

**Goal:** preserve only the exact frozen historical digest exception while keeping new diagnostic hashes strict.

**Why now:** this remains the final small schema-regression debt from the repeated-failover diagnostic closure; it does not justify reopening WAN.

**Files:** prefer `scripts/bench/run-repeated-warm-failover-test.py` or the existing focused schema test; change `schema/repeated-warm-failover.v1.json` only if a real defect is exposed.

**Protected invariant:** no historical artifact rewrite, no repeated-warm-failover VPS rerun.

**Behavior/tests:** Draft 2020-12 validation must accept the exact frozen legacy value, accept canonical 64-hex SHA-256, reject at least one sibling malformed/non-64-hex digest, and continue validating the full retained artifact corpus.

**Gates:** focused test, `scripts/check.sh`, `git diff --check`, exact-head CI.

**Commit/push:** yes. Continue immediately to D.

### D. HIGH SECURITY REVIEW / READY_LOCAL except D019 policy — exact-tree partial RSEC review

**Goal:** independently review the integrated exact tree for all non-policy pre-auth accounting controls.

**Why now:** controls and adversarial tests exist; static ownership proof should be locally truthful after B; current repository still explicitly says independent review is open.

**Files:** current `crates/neko-cli/src/preauth.rs`; every real responder call site; `crates/neko-cli/tests/probe.rs` and other adversarial tests; responder inventory/checker; `docs/reviews/resource-abuse-evidence-2026-09-04.md`; D018/D019 ADRs and current status.

**Protected invariants:** charge before expensive parse/auth work; response bytes accounted before send; all real pre-auth listener surfaces covered; TCP/UDP source domains non-colliding; redacted observability; bounded live concurrency/queue/memory; rejection/expiry/success cleanup fail closed. Do not invent terminal source-retention policy.

**Review behavior:** verify exact current-tree source projection, admission/charge ordering, responder coverage, queue/pending ownership, response accounting, cleanup, redaction and adversarial evidence provenance. If a concrete engineering defect exists, repair it with deterministic tests and continue. If the non-policy controls pass, state only a bounded result such as `ENGINEERING_CONTROLS_REVIEWED`; full RSEC/D019 must remain `SOURCE_RETENTION_POLICY_BLOCKED`.

**Tests/gates:** focused adversarial tests plus `scripts/check.sh`, `git diff --check`, exact-head CI.

**Commit/push:** yes when repository review/evidence truth changes. Continue to E; do not idle waiting for D019 policy.

### E. READY_OUTPUT / VPS-VALUABLE — current-tree package/operator lifecycle closure

**Goal:** produce a concrete operator-visible exact-tree result instead of extending audit infrastructure.

**Why now:** substantial runtime code landed after the older N5 package evidence, and the VPS rental is time-limited. Dedicated experimental package install/upgrade/rollback rehearsal is standing-authorized and has evidence value that local unit tests do not fully replace.

**Files:** `scripts/release/build-package.sh`, `scripts/release/smoke-package.sh`, package/release evidence docs only as necessary. Reuse existing harnesses; do not build a second packaging framework.

**Protected invariant:** use only the dedicated experimental install path; preserve external identity/state without reading or committing secret material; no production service replacement; no production route/firewall/DNS/proxy/tunnel/qdisc changes.

**Behavior:** first determine whether the old N5 evidence actually covers the materially changed current package/runtime contract. If not, build the exact current tree, verify manifest/binary hash/capability provenance, perform bounded authenticated TCP/UDP smoke from the isolated package, and where the existing harness safely supports it, do A→B→A upgrade/rollback with state-permission and cleanup checks. Prefer the self-owned VPS experimental path when that yields genuinely new operator evidence under standing authorization. If the package contract is demonstrably unchanged and old evidence is sufficient, record the exact-tree rationale and skip redundant execution rather than manufacturing activity.

**Tests/gates:** package build/smoke, cleanup, exact binary/package identity, `scripts/check.sh`, exact-head CI. If a VPS rehearsal is performed, record actual parameters/timestamps/results/cleanup under the standing evidence contract.

**Commit/push:** commit only genuinely new package/operator evidence or a real repair. Continue to F.

### F. READY_LOCAL — release/evidence matrix reconciliation

**Goal:** reconcile current status/ROADMAP/plan after A–E without inflating local or package evidence into WAN/release/security conclusions.

**Why now:** prevents exact-tree review and operator work from silently changing milestone meaning.

**Files:** `docs/status.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, release-review packet only if exact-tree claims changed.

**Protected invariant:** `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged absent explicit reviewed decisions; HY2 and repeated-failover same-class runs remain frozen; D019 remains policy-blocked.

**Behavior:** update only evidence actually obtained. Do not create another checker/harness unless a concrete consistency defect is discovered.

**Tests/gates:** `scripts/check.sh`, `git diff --check`, exact-head CI.

**Commit/push:** yes if current repository truth changes. Continue immediately to G when a real dependency-ready question exists.

### G. NEXT OUTPUT SELECTION — select one genuine dependency-ready runtime/operator/VPS question

**Goal:** keep work pointed at system behavior rather than review infrastructure.

**Selection rule:** re-read the exact post-F status/code/tests. First prefer a non-frozen standing-authorized VPS-only question with a concrete hypothesis that cannot be reconstructed locally. If none exists, prefer a bounded local runtime seam that directly opens one. `live PMTUD` is a plausible candidate only if the current accepted PLPMTUD state/spec and current runtime still leave a clearly bounded probe/ACK integration seam; do not implement it merely because an old roadmap row says `BLOCKED_IMPLEMENTATION`.

**Do not select:** unchanged HY2 or repeated-failover reruns; speculative FEC/0-RTT/striping/multipath/exotic carriers; third-party targets; production network mutation; new security numbers.

**Execution contract:** define one bounded question, implement/test if needed, exact-head green before WAN, then run once under standing authorization if the hypothesis is materially new. Preserve negative results and cleanup exactly. Continue while READY work remains; do not stop because one commit or reviewer interval completed.

## VPS / evidence priority

- Standing authorization continues to cover bounded self-owned TCP/UDP Session/diagnostic/benchmark/capture/cleanup, dedicated experimental package rehearsal, and the existing HY2 comparison shape.
- HY2 same-class retry is frozen after the exact-`13da094` `unknown/client_started` negative.
- Repeated warm failover same-class retry remains frozen after the repeated `startup_setup` boundary.
- Package/operator lifecycle is the next explicit VPS-valued outward lane after correctness/security gates A–D because the current VPS rental is time-limited.
- A future WAN retry/run requires an actual question and material hypothesis/change, not merely a desire for a PASS.

## Visible-output check

The last 24–48 hours already produced real runtime/VPS outputs: endpoint source rebinding, migration-back, periodic/key-update evidence, and a materially changed HY2 negative. Therefore A–D remain justified bounded correctness/security closure, but they must not grow into a generic audit framework. E deliberately returns the queue to a concrete package/operator result, followed by a new runtime/VPS question only if one is genuinely ready.

## Maintainer/admin boundary

The only known maintainer/security-value checkpoint in this queue remains D019 terminal source-retention policy. It blocks full D019/RSEC closure, not A–G engineering/evidence work. No administrator action is required for the currently READY slices.
