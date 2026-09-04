# Nekomusume ChatGPT Handoff

Checked at: 2026-09-04 19:58 Asia/Shanghai
Repository HEAD reviewed: `bb80ab73c721125bfac55c21475a86672493cd94`
Previous reviewed coding/evidence HEAD: `58c9bc71e73afc4b600614bbd5d61af033a28f54`
Previous reviewer handoff commit: `76acb04d2c54d6d14add11678485ea662c882ba2`

## What changed

Four coding/review commits landed after the previous reviewed implementation head.

- `131792e` — **release-evidence taxonomy repair; no runtime/network semantic change.** It adds `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`, restricts `OPEN_READY` to rows carrying concrete `evidence_needed` / `next_action` / `requires` / `execution_scope`, removes A/G/H/I/J from the generic open-ready set, reconciles current HY2/standing-authorization navigation, and adds `scripts/check-era4-closure.py` to the repository gate.
- `0121c25` — **dependency-integrity repair; no runtime/network semantic change.** It discovers that N depends on implementation-blocked K and O depends on N, introduces `BLOCKED_DEPENDENCY`, removes N/O from `OPEN_READY`, and makes the closure validator reject an open-ready row whose direct dependency is blocked or governance-gated. Current live-ready release evidence is therefore truthfully **none**.
- `bb9e268` — **evidence-integrity repair; no transport semantic change.** It removes the impossible self-checksum entry from the N8 evidence manifest, adds `scripts/check-evidence-manifests.py`, verifies canonical tracked Git blobs rather than mutable worktree bytes, and wires that check into `scripts/check.sh`.
- `bb80ab7` — **security review finding only; no implementation fix yet.** It adds `docs/reviews/resource-abuse-evidence-2026-09-04.md` and identifies `RSEC-001`: the current `PreauthBudget` provides a bounded per-budget-object anti-amplification/input-response cap, but the process-owned/global/per-source pre-auth admission accounting required by `SECURITY.md` and D019 is not implemented.

Exact `bb80ab7` GitHub Actions completed successfully on both `main` and `work/continue-20260904`; main run `33870316055` concluded `success`. This is exact-head CI evidence, not security approval.

The newest review finding materially changes queue priority. The VPS remains time-limited, but a HIGH security/release blocker outranks further live evidence. Current closure already reports no dependency-ready VPS row, so there is no lost truthful VPS experiment to run before closing this security seam.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — R-011/R-012 taxonomy/navigation work is accepted; RSEC-001 is a HIGH release/security blocker and becomes the entire first queue lane. Bounded local/loopback implementation is already administrator-authorized by D020 (`docs/adr/m1-g0-research-authorization.md`), so no new maintainer permission is required to implement the existing D019 candidate contract.**

Do not promote RC/security/public-listener claims while RSEC-001 is open. Do not jump to unrelated release polish merely because the security implementation is multi-commit. Consume A-F continuously as one security-closure program, pushing coherent commits between slices. After A-F are complete and exact-head CI is green, resume the broader release/security audit queue without waiting for another reviewer round-trip.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- `131792e`/`0121c25` improve release-opportunity classification; they add no behavior evidence.
- `bb9e268` makes one historical checksum manifest mechanically verifiable; it does not turn historical evidence into stronger WAN/security evidence.
- `bb80ab7` records a real HIGH security gap. The gap is **process-global/per-source pre-auth admission accounting**, not a cryptographic primitive failure and not a demonstrated Session-delivery correctness bug.
- Existing `neko_crypto::PreauthBudget` is a stricter inner/per-handshake bounded object for some fields. Do not delete or weaken it. D019 process/source/global ceilings should compose outside it; effective limits may be the stricter of existing inner bounds and D019 outer ceilings. Do not invent replacement numeric limits.
- D019 originally stated no implementation authorization, but D020 / `docs/adr/m1-g0-research-authorization.md` explicitly superseded the blanket prohibition and authorizes bounded local/loopback implementation of the existing G0 candidate contracts. Public/production exposure remains prohibited.
- D019 accounting controls process-owned application state. Do not claim it controls kernel SYN backlog, provider NAT state, or other resources outside the process.
- No VPS/live row is currently dependency-ready according to the corrected closure map. N/O are dependency-blocked; NAT/source change, live migration-back/key-update/PMTUD are implementation-blocked; owned IPv6 is environment-blocked; repeated/periodic/HY2 current lines are closed orchestration lines. Do not manufacture a live run.
- Protected identity material, credentials, raw private diagnostics and private topology remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a rolling multi-hour queue. Every dependency-satisfied slice is pre-authorized to start immediately after the previous coherent slice is validated, committed and pushed. **Do not stop after one commit, one nominal hour, or one reviewer interval.** Pause only for a newly discovered BLOCKER/HIGH that invalidates the planned design, a core Session/Carrier/ACK/crypto/wire architecture decision not resolved by existing ADRs, action outside standing authorization, production impact, missing credentials/third-party authority, actual repository breakage, runtime/tool-budget termination, or true queue exhaustion.

### A — RSEC-001 foundation: process-owned pre-auth admission state machine

**Status:** `READY_LOCAL`; highest priority.

**Goal**

Implement the reusable bounded accounting primitive required by D019 without changing wire bytes, Noise construction, Session delivery semantics, Carrier semantics, or candidate numeric ceilings.

**Implementation direction**

Prefer a small typed process-admission module/state machine rather than scattering counters through socket code. It may live in `neko-crypto` if kept transport-agnostic, or in `neko-cli` if source/process ownership makes that materially clearer; do not create a new crate solely for this seam. A typed source key should distinguish carrier + received source tuple while keeping raw addresses out of normal diagnostics.

The controller must cover D019 counting domains:

- concurrent pre-auth states per source and globally;
- per-source state-lifetime input bytes/packets;
- one-second monotonic global input byte/packet windows;
- parser/work units per packet, per source lifetime and global window;
- pre-auth memory reservation per state and globally;
- pending queue entries per source/globally;
- per-source response bytes/packets plus existing 3x anti-amplification;
- one-second monotonic global response byte/packet windows;
- idle timeout, maximum state lifetime and response-send deadline;
- checked overflow => exhaustion/fail closed.

Preserve existing `PreauthBudget` as an inner bound where it is already used/tested; do not loosen it merely to match D019's larger source ceilings.

**Required invariants**

1. Every charge is atomic: rejected charge leaves all counters unchanged.
2. Source/global/state limits are checked before the process-owned allocation/work/queue/send they protect.
3. Expiry/close releases ownership exactly once; rejected/expired state cannot silently reopen.
4. Unknown/unusable source identity maps to one bounded shared bucket rather than bypassing source limits.
5. Counter overflow is budget exhaustion, never wraparound.
6. Diagnostics expose bounded aggregate/reason data only; no raw unauthenticated payload, key material or unnecessary source identity.

**Tests**

Use injected/small test limits where needed for fast boundary tests, plus explicit assertions that production defaults correspond to the existing D019 candidate ceilings. Cover source/global saturation, exact boundary, +1 rejection, overflow, atomic failure, expiry and double-release.

Run targeted tests + normal full local gate + `git diff --check`; push a coherent commit.

**Continue immediately to B:** yes.

---

### B — Integrate admission before ordinary TCP/UDP responder handshake work

**Status:** `PREAUTHORIZED_AFTER_A`.

**Goal**

Make ordinary bounded responder paths actually consume the process-owned controller before pre-auth application work.

**Scope**

Map current `server` TCP and UDP responder flows (and their shared helpers) to the admission controller:

- charge/open application pre-auth state immediately after the process has a received peer/source tuple and before constructing responder handshake/application state;
- charge received bytes/packet before decode/parse of unauthenticated handshake/negotiation material;
- charge bounded parser/work accounting at explicit pre-auth parse/work points without pretending to count CPU cycles;
- reserve/check process-owned state/buffer/queue accounting before corresponding allocation/enqueue;
- charge response bytes/packet and anti-amplification before serialization/send decision;
- terminal success/rejection/timeout closes the pre-auth state exactly once.

Do not claim control over kernel accept/SYN resources. Do not turn the bounded research server into a public daemon.

**Negative process tests**

Prove admission exhaustion occurs before Noise/authorization/session evidence and yields no `Delivery`, `PathValidated`, `DeliveryAck`, warm-carrier or equivalent success evidence. Existing valid loopback handshake must continue to pass.

Push after targeted + full gate.

**Continue immediately to C:** yes.

---

### C — Share one process/global controller across multi-carrier failover pre-auth paths

**Status:** `PREAUTHORIZED_AFTER_B`.

**Goal**

Close the most important meaning of “global across carriers”: the bounded failover responder must not own independent budgets that can each reach a global ceiling separately.

**Scope**

Where `failover-server` accepts both UDP-primary and warm TCP pre-auth/resume work in one process, use one shared process-owned admission controller. Source keys may remain transport-distinct, but global counters/memory/queue/window limits must be common.

Preserve D064 single-active/multi-ready semantics, readiness proof, Session delivery accounting and warm promotion unchanged. Admission rejection must happen before a rejected path becomes authenticated/admitted/warm or changes Session/carrier state.

**Tests**

- two carriers/sources contend for one small injected global limit;
- one source cannot evade its source cap by retry/reconnect/carrier transition when D019 says the counting domain must persist for the pre-auth lifetime;
- rejection on one carrier does not corrupt admitted state on another;
- cleanup frees only the rejected/closed ownership;
- no readiness/delivery evidence on pre-auth rejection.

Push after full gate.

**Continue immediately to D:** yes.

---

### D — Audit and integrate every remaining externally reachable responder pre-auth seam

**Status:** `PREAUTHORIZED_AFTER_C`.

**Goal**

Avoid closing RSEC-001 only for the simplest probe while another command bypasses process admission.

Audit current responder commands/modules, including at least periodic and multistream/process-runner paths, and classify each as:

- uses the shared process-admission controller before pre-auth work;
- post-auth only and therefore outside this pre-auth seam;
- fixture/in-process only and non-listener;
- intentionally blocked/not executable.

Integrate any real responder pre-auth bypass that is within the current bounded research runtime. Prefer shared helpers over copy/pasted counters.

Add a machine-checkable inventory/test or compact review artifact so adding a new externally reachable responder command cannot silently omit the admission contract.

Push after full gate.

**Continue immediately to E:** yes.

---

### E — D019 adversarial boundary/evidence-barrier closure

**Status:** `PREAUTHORIZED_AFTER_D`.

**Goal**

Turn the process admission implementation into security evidence rather than merely code presence.

Required deterministic/adversarial coverage:

- source concurrency exact max / max+1;
- global concurrency exact max / max+1;
- unknown-source shared bucket saturation;
- input bytes/packets source and global windows;
- parser/work-unit packet/source/global limits;
- memory reservation state/global limits;
- pending queue source/global limits;
- response source/global limits and 3x anti-amplification composition;
- monotonic window rollover behavior without retroactively making a rejected operation succeed;
- idle timeout 1s, lifetime 5s, response-send deadline 100ms using an injectable clock/no slow wall-clock test;
- checked integer overflow;
- cancellation/timeout/duplicate cleanup;
- fail-closed rejection leaves no Delivery/PathValidated/ACK/readiness/authz-equivalent evidence;
- secret-safe diagnostics.

Do not add high-load or VPS stress tests to prove these counters. Deterministic local/loopback tests are the first security gate.

If fuzz targets are unaffected, do not manufacture a new protocol-fuzz claim; normal CI/fuzz workflow still applies as configured.

Push after full local gate.

**Continue immediately to F:** yes.

---

### F — Re-review RSEC-001 and reconcile release/security navigation

**Status:** `PREAUTHORIZED_AFTER_E`; requires exact-head CI green before declaring the finding closed.

**Goal**

Perform a fresh evidence review of the exact implementation, not a developer self-assertion.

Update `docs/reviews/resource-abuse-evidence-2026-09-04.md`, `docs/release-security-review-packet.md`, `docs/status.md`, and closure/navigation only as supported by evidence.

RSEC-001 may be marked resolved only if:

- all real responder pre-auth entry points are accounted for or explicitly outside scope;
- source/global counters are process-owned and shared where required;
- charge ordering/evidence barrier is tested;
- timeout/cleanup/overflow behavior is tested;
- exact-head local gate passes;
- exact-head GitHub CI is green.

Even then, the result is **bounded research implementation + internal review evidence**. Independent external/two-person security review remains a separate release gate. Do not promote `RELEASE_CANDIDATE`, `PRODUCTION_READY`, `FREEZE` or `RELEASED`.

**Continue immediately to G if no new HIGH/BLOCKER:** yes.

---

### G — Finish the compatibility/freeze-boundary review lane

**Status:** `READY_LOCAL_AFTER_F`.

Audit exact current policy and tests for:

- corpus-v1 content-addressed freeze vs global protocol non-freeze;
- current/current negotiation and unsupported/future rejection;
- downgrade/transcript binding into Noise;
- resume/version binding and replay boundary;
- no fictional previous frozen release;
- docs that might still imply corpus freeze == protocol/release freeze.

Add only a small deterministic regression if an actual gap is found; otherwise produce/reconcile a concise review record. Do not reopen frozen corpus bytes without a concrete correctness defect.

**Continue immediately to H:** yes.

---

### H — Package/operator + evidence provenance integrity review

**Status:** `READY_LOCAL_AFTER_G`.

Combine the still-valid prior review lanes rather than rerunning package/VPS work for freshness.

Verify reviewer traceability for:

- x86_64 package/build identity;
- install -> readiness/smoke -> upgrade -> rollback;
- retained external-state boundary without reading protected identity material;
- shutdown/listener/process/temp cleanup;
- evidence manifests and canonical Git-blob checksum validation;
- exact-head CI references (no nonexistent/local-only SHA claims);
- stale links/artifact hashes in the release-security packet.

`bb9e268` closes one manifest defect, but audit whether other tracked checksum/result manifests make unverifiable/self-referential/stale claims. Fix only concrete defects.

**Continue immediately to I:** yes.

---

### I — Reclassify the remaining `OPEN_READY` rows and re-evaluate VPS opportunity

**Status:** `READY_LOCAL_AFTER_H`.

The structural R-011 repair is accepted, but current B/C/D/E/F/L/M/T descriptions still include broad “run/review existing gate” language. Re-evaluate each against actual missing evidence after the security work:

- if the bounded question is already answered, move to `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`;
- if a specific assertion/test is genuinely missing and executable, keep `OPEN_READY` with that exact missing assertion;
- if a dependency is blocked/governance-gated, classify accordingly;
- do not keep a row open-ready merely because another test run is possible.

Then re-evaluate the live matrix. If and only if a genuine dependency-ready VPS row now exists under standing authorization and answers a declared missing release question, execute the smallest meaningful bounded row. Otherwise record `READY_LIVE: none` and do not manufacture VPS traffic.

No unchanged retry of repeated/periodic/HY2 closed current lines.

## Completion gates

This queue's security lane A-F is complete only when:

- a process-owned pre-auth controller implements D019 source/global/state accounting without weakening current inner limits;
- ordinary TCP/UDP and multi-carrier failover responder pre-auth paths use it;
- every other externally reachable responder seam is inventoried/integrated or explicitly out of scope;
- atomic charge, overflow, timeout, cleanup, queue/memory/work, anti-amplification and evidence-barrier tests pass;
- exact-head full local gate and GitHub CI are green;
- RSEC-001 is independently re-reviewed and either closed with bounded evidence or retained with a precise remaining gap;
- release/security flags remain unchanged.

The broader rolling queue is complete only after G-I are also consumed or a real stop condition is reached.

## Do not expand into

- public/production listener exposure;
- new production-capacity claims or arbitrary replacement D019 numeric limits;
- kernel/network-stack resource claims that process accounting cannot prove;
- changing Noise/wire/Session delivery/Carrier architecture to make admission implementation convenient;
- 0-RTT, FEC enablement, striping/aggregation or exotic carriers;
- speculative live key-update/PMTUD/migration implementation merely to fill release rows;
- unchanged repeated/periodic/HY2 retries;
- third-party targets, scanning or production network changes;
- reading/adding protected identity or secret/private diagnostic material.

## Questions requiring maintainer decision

none at this review. D020 already authorizes bounded local/loopback implementation of the existing G0 candidate security contracts. If implementation discovers that D019's counting domains/values are internally inconsistent or require a core architecture change, stop that affected lane and record the exact conflict for maintainer review rather than silently amending D019.
