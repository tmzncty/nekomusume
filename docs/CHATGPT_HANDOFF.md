# ChatGPT reviewer handoff — accept bound pre-auth response ownership, then produce bounded RSEC evidence

## Reviewed state

- Previous reviewer-owned handoff: exact `a438afaad2ca4297541bebc6ba49f975c387d528` (`docs(handoff): close red gate and bind preauth response permits`).
- Previous reviewed developer implementation/test head: exact `acbd4bf44a9a7c13e615cd9266aa3caf897e8417` (`test: lease all failover process ports`).
- Previous reviewed developer documentation/evidence head: exact `f329e4966daf949aedc0433ac9b1ab0a529e252b` (`docs: correct port lease tested tree`).
- Current default-branch developer implementation/test head reviewed this cycle: exact `81edd7ba358562e5e6013f889b76e3243dff44fe` (`fix: account endpoint rebind preauth responses`).
- Current default-branch developer documentation/evidence head reviewed this cycle: exact `4c2e711d5910018041618cde01597f6567510cbd` (`docs: close preauth response ownership`).
- Developer sequence since the previous reviewer handoff:
  - `060f89e6625f4b50137ca77fc74cf8c006a9aff8` binds response permits to controller/state/attempt ownership, enforces one pending response per state, caps the permit deadline by state idle/lifetime boundaries, and preserves an earlier socket write timeout in the bounded send calculation.
  - `2379a9241c28374da4a352384eed27a8a1729268` removes one redundant test assignment only.
  - `e10a8460b6f31446d9a0406d82cebfd6acb51e57` explicitly settles the two intentional failover response-suppression seams instead of dropping charged permits.
  - `b1829f190084c762d2934bdb97c949f85bdcb034` is a lint-only test expression cleanup.
  - `81edd7ba358562e5e6013f889b76e3243dff44fe` moves endpoint-rebind UDP negotiation/Noise responses onto the same pre-auth admission, exact-charge and bounded-send path and extends the responder inventory.
  - `4c2e711d5910018041618cde01597f6567510cbd` persists the exact-`81edd7b` developer-local gate/evidence and reconciles release/resource-review factual anchors.
- No new VPS/WAN experiment was performed in this sequence.
- GitHub currently exposes no hosted combined-status records for exact `81edd7b`. This is not a blocker and must not trigger Actions polling.

## Review verdict

### ACCEPT — previous HIGH response-permit ownership/deadline finding is closed for the current bounded engineering scope

Current exact-main response permits are no longer identified only by a numeric state ID. `PreauthResponsePermit` now carries a checked process-local controller identity, monotonic response-attempt identity and state identity. `ProcessPreauthState` tracks the one pending response attempt, and a second charge for that state fails closed. Cross-controller completion fails even when both controllers have `PreauthStateId(0)`.

The effective permit deadline is the minimum of the configured response-send ceiling and the current state's idle/lifetime boundaries. The CLI send wrapper further refuses to widen an already-earlier observable socket write timeout. The configured 100 ms response-send ceiling remains the existing candidate ceiling; no new security-capacity number or D019 policy was introduced.

The two deliberate failover loss seams now explicitly settle a charged response attempt as suppressed rather than silently dropping the ownership token. Endpoint-rebind UDP now participates in the same admission/charge/bounded-response path. The machine-readable responder inventory is therefore eight surfaces / seven admission sites on current main.

This closes the prior engineering HIGH. It does **not** close RSEC-001 promotion suitability, D019 source retention, independent security review, RC, public-listener or production gates.

### ACCEPT — exact-tree developer-local closure is factual

The retained `docs/local-preauth-response-permits-81edd7b-20260910.md` records:

```text
exact implementation/test SHA              81edd7ba358562e5e6013f889b76e3243dff44fe
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh   exit 0
git diff --check                                  exit 0
initial/final source tree                         clean
gate UTC                                           2026-09-09T17:29:17Z -> 2026-09-09T17:31:09Z
host                                               Linux 6.8.0-137-generic x86_64
Rust                                               rustc 1.98.0
```

Focused ownership/deadline, ordinary TCP/UDP, failover-loss and endpoint-rebind positives are retained as developer-reported exact-tree evidence. Reviewer did not execute this CI and there is no hosted status for that SHA; neither fact weakens the accepted developer-local gate under the repository's local-CI-first policy.

Release packet, item-4 factual review and resource-abuse mapping now consistently anchor the changed pre-auth facts through exact `81edd7b`. No release/security claim inflation was found.

## Bounded follow-up findings before the next visible output

### MEDIUM / READY_LOCAL — TCP bounded-send helper does not restore the previous write timeout on the write-error path

`ListenerAdmission::send_tcp_response` snapshots the prior `TcpStream` write timeout and restores it after a successful `write_frame_until`, but an error from `write_frame_until` currently abandons the response permit and returns before restoring the prior timeout. The UDP wrapper restores its prior timeout before classifying the send result.

Current executable pre-auth call sites treat a bounded-response send failure as terminal for that command/process path, so this review does **not** claim an observed live Session failure or remotely exploitable production defect. It is nevertheless a real helper-contract/evidence gap: the retained evidence says reusable sockets restore their prior timeout, while the TCP helper only guarantees that on the success path.

Minimal repair boundary:

- restore the saved TCP write timeout after the bounded write attempt on both success and failure paths, before returning;
- preserve charged accounting and fail-closed permit terminalization;
- do not turn this into a generic timeout/async framework;
- add one focused deterministic regression for an errored TCP send if it can be done with the existing local socket test shape; otherwise keep the code repair minimal and narrow the evidence wording rather than building a new test harness.

Continue immediately to the next item.

### LOW / READY_LOCAL — `PreauthResponsePermit` documentation still claims Drop performs abandonment

The type comment says that dropping a permit abandons the response, but the permit has no `Drop` implementation and cannot mutate its issuing controller by itself. Current known intentional drop sites were correctly removed in `e10a846`; an accidental future drop would leave the state's pending marker until another fail-closed transition/release/expiry.

Fix the comment/contract wording to require explicit completion/suppression/abandonment. Do **not** add hidden RAII side effects or a new ownership framework merely to make the old sentence true.

### LOW / FACTUAL RECONCILIATION — one historical verification paragraph still names GitHub Actions as CI authority

`docs/reviews/resource-abuse-evidence-2026-09-04.md` retains an older verification sentence saying exact-head GitHub Actions is the CI authority. Current repository policy and README make clean exact-tree local `scripts/check.sh` the primary reproducible gate, with hosted Actions only optional cross-evidence. Correct this sentence the next time the resource-abuse document is factually updated; do not create a standalone docs-only churn commit for it.

## Local-CI-first rule

For every implementation/test slice below, push the coherent developer SHA first, then validate that exact SHA in a clean temporary worktree/clone:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # empty
```

Persist minimum provenance only after the final replacement SHA is green: exact SHA, exact commands, distinct UTC start/end timestamps, exits, host/OS/arch, stable Rust version and initial/final clean-tree state. If the gate is red, preserve the concrete failure and repair it before any closure claim; do not retry unchanged until lucky.

No fuzz is required for the timeout-restoration/documentation cleanup or a model/process-level resource observation unless a later implementation actually modifies wire decoder/parser/crypto framing. Hosted CI is optional cross-evidence and must not become a wait condition.

## Rolling queue — execute continuously in dependency order

The response-permit HIGH is closed. The next package intentionally moves out of broad checker/ownership auditing and into a real RSEC-001 evidence output. Do not finish one small commit and enter a watcher.

### A. MEDIUM / READY_LOCAL — close the TCP timeout restoration edge and truthful permit contract

Apply the minimal helper repair above and correct the false Drop wording in the same coherent slice. Re-run the focused pre-auth tests. Do not reopen the controller/state/attempt design, responder inventory, identity lane, port-lease lane or general timeout architecture without a newly demonstrated defect.

Continue immediately to B.

### B. READY_LOCAL / OUTPUT DESIGN — choose one bounded local adversarial/resource observation for RSEC-001

Run a short autonomous proposal cycle (1-3 shapes) against current `ProcessPreauthAdmission`, `ListenerAdmission`, existing process fixtures and `docs/adr/m1-g0-preauth-resource-budget.md`. Choose the smallest test/runner shape that produces an actual bounded observation rather than another static checker.

The observation must answer at least these named questions without inventing new policy values:

1. At existing candidate boundaries, do source/global pre-auth state/queue/response/work limits reject the first over-limit operation fail closed without creating Delivery/PathValidated/ACK evidence?
2. Under a small bounded loopback malformed/aborted pre-auth workload, does the executable clean up connections/FDs/listener state and remain capable of a subsequent valid authenticated exchange?
3. Do response failures/suppression/abandonment remain charged rather than reopening the same response attempt?

Use the **existing candidate limits** as the contract under test. Workload counts chosen only to exercise the boundary are test parameters, not new capacity/security policy. Source tuple semantics matter: model-level same-source saturation and process-level socket churn are different questions and must not be conflated.

Prefer extending existing tests/fixtures plus one small executable observation over creating a new adversarial-load framework.

Continue immediately to C.

### C. READY_LOCAL — deterministic model-boundary evidence at exact existing limits

Where existing tests do not already make the result executable/obvious, add compact deterministic coverage for the selected B observation. Prioritize actual existing ADR/default boundaries such as:

- per-source and global live-state ceiling plus first refusal;
- per-source/global queue ceiling plus first refusal;
- response byte/packet and anti-amplification refusal after exact charging;
- input/work/window exhaustion and checked overflow;
- expiry/release cleanup and no resurrection of a rejected state.

Do not duplicate tests already present merely to list every number again. If an existing test already proves a boundary, reuse it in the observation/evidence instead of cloning it.

Continue immediately to D.

### D. READY_LOCAL — bounded process-level malformed/churn observation

Use only loopback/local execution unless exact-current code creates a genuine reason for a self-owned VPS question. Keep concurrency and traffic low and within existing CLI/standing limits.

Minimum useful shape:

- establish a clean server baseline;
- perform a deliberately bounded set of malformed, truncated, early-close or otherwise pre-auth-rejected attempts using existing public command/process surfaces;
- collect only secret-safe process/resource facts already available or cheaply observable (for example FD/RSS/socket/listener counts and exits); do not retain payloads, keys, private endpoints or raw secret-bearing logs;
- verify no unintended listener/child residue from the observation;
- perform a valid authenticated exchange afterward to show the bounded rejection workload did not strand the executable;
- state the exact count/bytes/duration actually used.

This is a leak/pathological-growth/recovery observation, **not** a maximum-capacity benchmark and not public-listener suitability evidence. If the current command architecture cannot exercise true concurrent same-source admission, say so and keep that ceiling evidence at the model layer rather than faking it with distinct ephemeral source ports.

Continue immediately to E.

### E. READY_LOCAL — exact-tree green gate and one retained provenance note

On the final pushed A-D implementation/test SHA, run the exact-tree local gate. If green, persist one concise note containing the tested SHA, commands, UTC interval, host/OS/arch, Rust, clean-tree state, workload parameters and sanitized measured resource observations. If red, return to the concrete failing slice.

Do not use GitHub-hosted CI absence as a reason to stop or poll.

Continue immediately to F.

### F. BOUNDED RELEASE/SECURITY RECONCILIATION — narrow RSEC-001 only as far as the new evidence supports

Update changed facts in:

- `docs/reviews/resource-abuse-evidence-2026-09-04.md`;
- `docs/release-security-review-packet.md` if its tested-tree/evidence index genuinely advances;
- `docs/reviews/release-item4-subgates-20260909.md` only if exact-tree factual support changes.

Also correct the stale GitHub-Actions-authority sentence while touching the resource review.

Allowed claim shape: one bounded local adversarial/resource observation now exists for the exact workload and exact candidate values exercised.

Forbidden claim shape: RSEC-001 fully closed, candidate values production-suitable, D019 resolved, independent security review complete, public listener approved, RC/release/production ready. Suitability review and independent decision remain separate gates.

Continue immediately to G.

### G. LOCAL OUTPUT SELECTION — choose the next real release/runtime/operator output

After F, re-read exact-current implementation plan/status/release packet and produce 1-3 concrete dependency-ready proposals. Prefer, in order:

1. a remaining **demonstrated** correctness/security gap with no policy invention;
2. a real advertised runtime/operator behavior that lacks direct executable evidence;
3. another named release-evidence question that can be answered locally without pretending to be capacity/security approval.

For each proposal state the observed contradiction/missing behavior, file/API owner, protected invariant/evidence boundary and minimum positive/negative evidence. Autonomously choose the smallest safe proposal and implement it; do not wait for the next reviewer hour.

Do not choose signing/key-custody policy, SBOM publication policy, D019 TTL/LRU/history numbers, previous-release interoperability without a frozen prior release, experimental carriers, or another generic checker/audit merely to fill the queue.

Continue immediately to H.

### H. READY_LOCAL / ROLLING VISIBLE OUTPUT — implement, exact-tree close, repeat

Implement the chosen G output, run the exact-tree local gate, reconcile only changed facts, then repeat G -> H while concrete safe work remains. If a proposal cycle genuinely finds no dependency-ready safe local output, state that truth explicitly rather than entering a polling watcher or manufacturing documentation work.

### I. CONDITIONAL VPS OUTPUT — only if exact-current truth creates a new live question

Repository truth still says `READY_LIVE: none`. Standing VPS authorization remains valid but is not a reason to duplicate old evidence.

Only execute a VPS run if a new implementation/instrumentation change creates a named unresolved real-network question with satisfied dependencies. Otherwise do not unchanged-rerun HY2, repeated warm failover, periodic/soak, package lifecycle, distinct A -> B -> A, already-answered endpoint migration/key update, IPv6 without a real owned IPv6 path, or live PMTUD before its separate authenticated wire/security design gate.

### J. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Bounded non-policy engineering and local RSEC evidence may continue. Terminal source-retention/no-reset semantics still require maintainer/security-policy judgment. Do not invent TTL, LRU/history capacity, external authority or weaker reset semantics.

## Workload calibration

The developer sequence from `060f89e` through exact-green `81edd7b` closed a multi-file HIGH, migrated a missed responder, preserved local-CI provenance and reconciled evidence within one review interval. That supports keeping a multi-slice continuous queue rather than returning to one-commit tickets.

Commit timing is only a sizing signal. If the RSEC observation starts producing rushed/flaky process tests, unclear source-domain claims, or noisy measurements, shrink the process workload and preserve the deterministic model evidence rather than adding more harness. If it stays clean, continue directly through G/H.

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
- Release item 3 remains incomplete and exact-current opportunity classification remains `READY_LIVE: none`.
- Item 4 remains independent-review incomplete; developer-prepared factual support is not independent audit/security approval.
- RSEC-001 remains open for promotion suitability/independent review even after the bounded local observation requested above.
- D019 remains `SOURCE_RETENTION_POLICY_BLOCKED` and is not an autonomous coding decision.
- Canonical corpus freeze is corpus-specific and does not freeze the global protocol.
- Standing VPS authorization remains active within its file's limits but does not authorize third-party targets, production network changes, privileged/exotic carriers requiring separate approval, or unchanged repeated negative experiments.
