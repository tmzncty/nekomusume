# ChatGPT reviewer handoff — close the repaired local gate, then bind pre-auth response ownership

## Reviewed state

- Previous reviewer-owned handoff: exact `ff352b2f96ac57b5a9cc1a66973d2ceec62c344e` (`docs(handoff): repair red local gate before identity closure`).
- Previous reviewed developer implementation/test head: exact `363177c18773d52cfa9edc114ed380117e03b562` (`fix: validate periodic configuration before identity`).
- Previous reviewed developer documentation/evidence head: exact `34f24f7af5e595214b870e27c43204abfb1bb7d7` (`docs: record periodic identity gate result`).
- Current developer implementation/test head reviewed this cycle: exact `acbd4bf44a9a7c13e615cd9266aa3caf897e8417` (`test: lease all failover process ports`).
- Current developer documentation/evidence head reviewed this cycle: exact `f329e4966daf949aedc0433ac9b1ab0a529e252b` (`docs: correct port lease tested tree`).
- Developer sequence since the previous reviewer handoff:
  - `7f0b9a326a90254b4e37810012a37186880dd997` replaces the original fixed failover pair in the reproduced failure and completes the missing periodic deterministic no-identity-side-effect negatives.
  - `ca4a6beb6f45d2167e5e111f330e7a72b8ed9633` retains the exact-`7f0b9a3` diagnosis/provenance and periodic closure.
  - `7d263307cedc994d72311bfb82af8266030df17a` introduces a TCP+UDP `PortLease` test fixture and starts using lease-to-spawn handoff.
  - `21778691d5a54e5a5a00b4dad1f744392c2835ef` records the first lease attempt, but its tested-tree identifier was later shown to be wrong and is superseded by the correction below.
  - `acbd4bf44a9a7c13e615cd9266aa3caf897e8417` applies the lease helper to all dual-port failover process fixtures after the intermediate exact `7d26330` gate still exposed another fixed-port failure.
  - `f329e4966daf949aedc0433ac9b1ab0a529e252b` corrects the retained lease evidence to the actually tested reachable exact `acbd4bf` and re-anchors the release packet/item-4 factual support.
- No new VPS/WAN experiment was performed in this sequence.
- GitHub currently exposes no hosted combined-status records for exact `acbd4bf`. This is not a blocker and must not trigger Actions polling.

## Review verdict

### ACCEPT — the previous red exact-tree gate BLOCKER is closed

The earlier full local gate failure is no longer merely attributed to a guessed race. The retained exact-`7f0b9a3` note records the concrete failing class: the failover server failed its TCP bind before the diagnostic start event, and local socket inspection observed the old fixed TCP port in `TIME-WAIT`.

The first dynamic-port repair was not treated as magically final. Exact `7d26330` subsequently remained red because another failover process fixture still used a fixed pair. Exact `acbd4bf` then moved every dual-port failover process fixture onto the same bounded lease helper, and its developer-local exact-tree gate is retained green:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh -> exit 0
git diff --check                              -> exit 0
initial/final source tree                    -> clean
```

The retained provenance records distinct UTC start/end timestamps, Linux/x86_64 and Rust `1.98.0`. This is **developer-local test-fixture/CI evidence**, not reviewer-executed CI, hosted CI, WAN/failover reliability evidence, independent security review, RC, release or production approval.

The `PortLease` handoff is intentionally not claimed atomic: it owns both TCP and UDP bindings during selection and releases them immediately before child spawn. That removes the earlier unbounded probe/use gap while preserving the production `40080..=40100` policy. There is no current demonstrated residual failure that justifies another port-allocation framework. Do not keep expanding this test-harness lane unless a new exact-tree failure identifies a concrete remaining fixture defect.

### ACCEPT — the periodic deterministic-config matrix is complete

The previous periodic MEDIUM is closed. Exact `7f0b9a3` adds the previously missing cases:

- periodic-server bind / `--port` mismatch;
- periodic-client malformed peer-key hex;
- periodic-client malformed address;
- periodic-client address / `--port` mismatch.

The table continues to start with an absent identity path and verifies that deterministic rejection leaves it absent. The authenticated periodic positive remains green. No Session/Carrier/ACK/Noise/wire semantic change was introduced.

### ACCEPT_WITH_HISTORY — the transient wrong tested-tree reference is corrected

The intermediate lease documentation referred to an unreachable/non-current tested identifier `20396f9`. Current exact `f329e49` corrects the evidence filename, body, release packet and item-4 factual anchor to reachable exact `acbd4bf`, and explicitly records that exact `7d26330` was still red before the all-failover-fixture repair.

Treat the intermediate document state as superseded evidence drift, not as a reason to rewrite older history. Current release-facing anchors are factual again. Do not generate another docs-only correction unless a new tested tree actually supersedes `acbd4bf`.

### CLOSE — identity/config consumer audit lane

No production identity-consumer code changed in this developer sequence. The preceding reviewed sequence already brought ordinary server/client, failover, endpoint-rebind, periodic server/client and advertised multistream onto the established deterministic-config-before-secret and secure existing-identity boundaries, with exact-tree local evidence. The final periodic matrix is now present and the current green gate includes it.

Close this audit lane. Do **not** continue into parent-directory policy, hard-link policy, generic secret-memory frameworks or another identity checker suite without a newly demonstrated defect.

## New finding

### HIGH / READY_LOCAL — pre-auth response permits are not bound to the issuing controller/state attempt strongly enough

This is the next release/security correctness slice. It is an engineering HIGH for the pre-auth ownership/deadline subgate; it does **not** establish a remotely exploitable production vulnerability in the current bounded command call sites.

Current exact-main code shows three related ownership/deadline gaps:

1. `PreauthResponsePermit` carries only `state_id`, `admitted_at_ms` and `deadline_ms`. Every `ProcessPreauthAdmission` starts `next_id` from zero. `complete_response` / `abandon_response` identify ownership only through that state ID. A permit issued by one controller can therefore collide with a live same-numbered state in another controller; the type is one-shot by move, but it is not controller-bound.
2. `ProcessPreauthState` has no pending-response owner. The API can charge multiple response permits for one state before the first attempt is completed or abandoned. The current production wrappers normally charge/send sequentially, but the resource-ownership type does not enforce the invariant it claims to represent.
3. The permit send deadline is currently `charge_time + response_send_deadline_ms`. Completion later checks whether the state is still live, but the socket write itself is not capped by the earlier remaining idle/lifetime boundary. Near a state deadline, bytes may therefore be attempted after the state should have expired before `complete_response` rejects the late result. The TCP/UDP wrappers also replace an existing socket write timeout with the permit budget rather than preserving an earlier stricter caller deadline.

An external open PR (`#3`, based on old exact `01c876c`) independently describes the same controller-bound/one-shot/deadline concern and contains one possible implementation shape. **Do not merge or cherry-pick that stale branch wholesale.** Main has moved substantially and already contains later pre-auth ownership work. It may be used as review/research input only; implement against exact current main and retain current semantics/tests.

Protected boundary:

- no change to Session, Carrier, ACK, Noise, wire format or negotiation semantics;
- no new TTL/LRU/history/source-retention policy and no D019 policy decision;
- no weakening or increase of existing candidate pre-auth numeric limits;
- failed/abandoned/expired response attempts remain charged and fail closed;
- the 100 ms configured response-send ceiling remains an inclusive maximum, while any already-earlier state/caller deadline must win;
- no production/WAN listener or benchmark work is needed for this local correctness slice.

## Local-CI-first rule

For any implementation/test commit below, push the coherent developer SHA first, then validate that exact SHA in a clean temporary worktree/clone:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # empty
```

Persist only the minimum useful provenance after the final replacement SHA is green: exact SHA, exact commands, distinct UTC start/end timestamps, exits, host/OS/arch, stable Rust version and initial/final clean-tree state. Preserve any red intermediate result honestly; do not retry unchanged until lucky.

No fuzz is required if the response-permit repair remains in typed pre-auth ownership/socket deadline code and does not modify wire decoder/parser/crypto framing. If it unexpectedly crosses that boundary, use the repository pinned fuzz contract before closure.

Hosted CI remains optional cross-evidence. Do not wait for it and do not label developer-local CI as hosted or reviewer-executed CI.

## Rolling queue — execute continuously in dependency order

The new HIGH must close before unrelated horizontal expansion. The developer has been completing coherent slices quickly without lowering evidence quality, so the queue below intentionally includes the full ownership -> callsite -> release-factual closure rather than another one-commit watcher cycle.

### A. HIGH / READY_LOCAL — choose the minimal controller-bound response-permit shape and implement it

Run a short autonomous proposal cycle (1–3 implementation shapes) against current `crates/neko-crypto/src/lib.rs`; choose the smallest fail-closed design with the least new state/API.

Required semantics:

- a response permit is bound to exactly one `ProcessPreauthAdmission` controller and one state;
- a state has at most one pending response attempt;
- controller/permit identity cannot collide merely because two controllers both have `PreauthStateId(0)`;
- duplicate completion, completion after abandon/expire, cross-controller completion and stale permit reuse fail closed;
- release/expiry/rejection invalidate pending ownership without refunding charged response accounting;
- any new controller/permit identifiers are internal ownership tokens, not a new security-capacity policy.

A checked process-local monotonic identity is acceptable if it is the smallest shape; overflow must fail closed rather than wrap.

Continue immediately to B.

### B. HIGH / READY_LOCAL — make the effective send deadline the earliest applicable boundary

Keep the existing configured response-send ceiling but make the actual attempt deadline no later than:

- the response-send ceiling;
- the state's remaining idle deadline;
- the state's remaining lifetime deadline;
- any already-earlier caller/socket write deadline that the wrapper can observe without inventing policy.

Preserve the existing inclusive boundary semantics. A send that cannot begin with positive remaining time fails before writing. Failed/late attempts remain charged and terminal for that permit.

Do not turn this into a general async timeout framework.

Continue immediately to C.

### C. READY_LOCAL — integrate TCP/UDP send wrappers and focused ownership/deadline regressions

Update the existing `ListenerAdmission` wrapper and current responder call sites to the new ownership API. Preserve exact pre-auth response charging before send.

Minimum focused tests should cover:

- permit from controller A rejected by controller B even when both have the same numeric state ID;
- second pending permit for the same state rejected;
- completion/abandon/expiry are one-shot;
- state idle/lifetime boundaries cap the effective response attempt;
- exact configured boundary remains accepted while the first instant past it is rejected;
- an earlier existing socket timeout is not widened and is restored where the socket remains usable;
- failed/abandoned sends retain response charge and cannot reopen the same rejected attempt;
- ordinary TCP and UDP authenticated positives still pass.

Do not duplicate the full process suite or add a new checker framework.

Continue immediately to D.

### D. READY_LOCAL — exact-tree green gate and retained provenance

On the final pushed A/B/C implementation/test SHA, run the exact-tree local gate described above. A red result returns to A/B/C based on the concrete failure; do not paper it over with docs or hosted-CI absence.

Once green, persist one small local evidence note. Continue immediately to E.

### E. BOUNDED RELEASE/SECURITY RECONCILIATION — review all live pre-auth response call sites once

Inspect the actual current responder inventory (ordinary TCP/UDP, failover/resume, periodic and multistream surfaces that still send unauthenticated negotiation/handshake/readiness responses). Confirm every pre-auth response:

- charges the exact framed/datagram bytes before I/O;
- uses the controller-bound permit and bounded send wrapper;
- settles completion/abandon/expiry exactly once;
- does not silently fall back to raw unbounded write/send for pre-auth response material.

If one call site is missing, fix it in one coherent commit and rerun the exact-tree gate. If aligned, explicitly close this ownership lane; do not start another broad pre-auth audit by default.

Continue immediately to F.

### F. READY_LOCAL / FACTUAL RECONCILIATION — update release-facing facts only after E is green

Update only the facts actually changed:

- `docs/reviews/resource-abuse-evidence-2026-09-04.md` engineering-control mapping;
- `docs/release-security-review-packet.md` tested-tree anchor / pre-auth boundary;
- `docs/reviews/release-item4-subgates-20260909.md` exact-tree factual support if needed.

Do **not** mark RSEC-001 fully closed, D019 resolved, item 4 independently approved, RC/release/production ready, or adversarial-load suitability proven. D019 source retention remains a maintainer/security-policy checkpoint.

Continue immediately to G.

### G. LOCAL OUTPUT SELECTION — choose the next real release/runtime/operator output

Re-read exact-current status/plan/release packet after F and produce 1–3 concrete dependency-ready proposals. Prefer, in order:

1. a remaining **demonstrated** release/security correctness gap that can be closed without policy invention;
2. a real advertised runtime/operator behavior lacking direct executable evidence;
3. a bounded local adversarial/resource observation that answers a named RSEC-001 evidence question without selecting new capacity values.

For each proposal state the observed contradiction/missing behavior, file/API owner, protected boundary and minimum positive/negative evidence. Autonomously choose the smallest safe proposal and continue; do not wait for the next reviewer hour.

Do not choose signing/key-custody policy, SBOM publication policy, D019 retention numbers, speculative experimental carriers, previous-release interoperability without a prior frozen release, or another generic checker/audit merely to fill the queue.

Continue immediately to H.

### H. READY_LOCAL / ROLLING VISIBLE OUTPUT — implement, exact-tree close, repeat

Implement the chosen G output, run the exact-tree local gate, reconcile only changed facts, then repeat G -> H while concrete safe work remains. If a proposal cycle genuinely finds no dependency-ready safe local output, state that truth explicitly rather than entering a polling watcher or manufacturing documentation work.

### I. CONDITIONAL VPS OUTPUT — only if exact-current truth creates a new live question

Repository truth remains `READY_LIVE: none`. Standing authorization remains valid, but it is not a reason to duplicate old evidence.

Only execute a VPS run if a new implementation/instrumentation change creates a named unresolved real-network question with satisfied dependencies. Otherwise do not unchanged-rerun HY2, repeated warm failover, periodic/soak, package lifecycle, distinct A -> B -> A, already-answered endpoint migration/key update, IPv6 without a real owned IPv6 path, or live PMTUD before its separate authenticated wire/security design gate.

### J. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Bounded non-policy pre-auth engineering may continue, including A-H. Terminal source-retention/no-reset semantics still require maintainer/security-policy judgment. Do not invent TTL, LRU/history capacity, external authority or weaker reset semantics.

## Stop conditions

Stop continuous coding only for a real condition:

- unresolved BLOCKER/HIGH correctness/security/evidence finding outside the already-authorized repair path;
- core Session/Carrier/ACK/crypto/wire architecture change;
- destructive/canonical-meaning migration;
- action outside standing authorization;
- production impact;
- new credentials/server/third-party permission;
- benchmark conditions requiring maintainer value judgment;
- repository/tool/runtime breakage preventing safe progress;
- actual runtime/tool-budget exhaustion;
- or a genuinely exhausted rolling queue after the required proposal cycle finds no concrete safe work.

Do not stop because GitHub Actions does not run, because the handoff becomes older than a developer commit, or because one coherent slice finishes before the next reviewer hour.

## Governance boundaries

- `IMPLEMENTATION_COMPLETE=true` remains bounded research-implementation status only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain authoritative.
- Release item 3 remains incomplete; opportunity classification remains `READY_LIVE: none`.
- Item 4 remains independent-review incomplete; developer-prepared factual support is not independent audit/security approval.
- Canonical corpus freeze remains corpus-specific and does not freeze the global protocol.
- Standing VPS authorization remains valid, but no dependency-ready live row currently exists.
- D019 remains a maintainer/security-policy checkpoint and must not be invented by the coding agent.
