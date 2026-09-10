# ChatGPT reviewer handoff — close matrix lane and reconcile Era-4 navigation

## Reviewed state

- Previous reviewer-owned handoff: exact `58b7161acd7eaaff2f8f99ebbb6bc4c164cd1d77` (`docs(handoff): unstick strict matrix argv implementation`).
- Current default `main` before this reviewer update: exact `8749efa45bb16048352c2ce23993ffa12fb68506` (`docs: record strict matrix probe contract`).
- New developer sequence after the previous handoff:
  1. exact `acb81bb7395d0683c669eac96aaa787dfc0abe66` — implementation/tests (`fix: parse matrix probe arguments strictly`);
  2. exact `8749efa45bb16048352c2ce23993ffa12fb68506` — docs/provenance (`docs: record strict matrix probe contract`).
- No new VPS/WAN experiment landed in this sequence.
- Open PR #3 remains an old unrelated pre-auth branch; do not merge/cherry-pick it as a substitute for current-main work.
- GitHub exposes zero hosted status records for exact `acb81bb`. Hosted CI remains optional cross-evidence and is not a wait condition.
- This reviewer performed GitHub repository/source/evidence review only. No reviewer-executed local CI is claimed.

## Review verdict — matrix operator-contract lane CLOSED

The previous matrix-argv MEDIUM and LOW fixture race are closed at exact implementation/test commit `acb81bb` with developer-local exact-tree evidence recorded by `8749efa`.

### What the implementation now proves

`crates/neko-cli/src/main.rs` now has a compact matrix-specific typed parser. It consumes `probe` matrix argv sequentially and rejects ambiguity before socket work:

- `--matrix` is required exactly once;
- `--target`, `--transport`, and `--ip-version` are required exactly once;
- `--timeout-ms` and `--bytes` are optional at most once;
- `--json` is optional at most once;
- unknown tokens, stray positional tokens, duplicate options, and missing values fail through the existing exit-2 path;
- malformed numeric values fail instead of falling back to defaults;
- typed parsing is followed by existing `reachability::validate(...)`, then and only then by `reachability::run(...)`.

The semantic boundary remains local-only: non-loopback targets, family mismatch, port zero, timeout outside `1..=5000`, and payload outside `1..=1200` reject before the normal matrix call reaches socket work. No public-WAN scope was added.

`crates/neko-cli/tests/probe.rs` now exercises the requested grammar negatives, including the valid-first/invalid-second duplicate `--bytes 17 --bytes 1201`, and asserts exit `2` with empty stdout. The prior bind/drop TCP completed-failure fixture was replaced by a held-open silent loopback UDP endpoint, removing the identified port-reuse race. Reachable local TCP/UDP still produce exit `0`; a completed local failure produces exit `1`; the existing timestamp field is bracketed in the process test.

### Developer-local exact-tree evidence

For exact `acb81bb7395d0683c669eac96aaa787dfc0abe66`, the persisted developer-local provenance records:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh  -> exit 0
git diff --check                                -> exit 0
gate UTC: 2026-09-10T16:14:07Z -> 2026-09-10T16:16:03Z
host: Linux 6.8.0-137-generic x86_64
rustc: 1.98.0 (88d9e12ae 2026-08-18)
initial/final source tree: clean
```

This is developer-local local-loopback/operator evidence. It is not reviewer CI, hosted CI, WAN evidence, independent review, security approval, public-listener approval, RC, release, or production authorization.

No fuzz was required: this slice changed CLI argv parsing, not network wire decode/parser/crypto framing.

### Bounded adjacent review

One bounded review of

```text
dispatch -> strict matrix argv parse -> semantic validate -> local socket -> artifact/human output -> exit code
```

found no new BLOCKER/HIGH and no concrete contradiction requiring another matrix-parser slice. Do not continue this lane into generic CLI parser normalization, a repository-wide argument framework, schema-checker expansion, or the deferred cross-transport meaning of `payload_bytes` merely because more hardening is possible.

## New finding — MEDIUM release/evidence navigation drift

The next concrete dependency-ready issue is not another matrix checker. It is a stale machine-readable Era-4 navigation contract.

`docs/release-security-review-packet.md` currently calls `docs/era4-ledger-2026-08-30.json` the reviewed machine-readable navigation source. However the ledger still classifies track K (`migration-back gate`) as `BLOCKED_IMPLEMENTATION` and track N as `BLOCKED_DEPENDENCY` specifically because K is blocked. That no longer matches later repository truth: the current `IMPLEMENTATION_PLAN.md` records migration-back as `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION` at exact `5d6582c`, while keeping the overall release-evidence item open and `READY_LIVE: none`.

This is a navigation/evidence correctness defect: an automated reader can derive an obsolete dependency explanation from a file that the release packet presents as current navigation. It is **not** permission to open a live row, and it does not mean every downstream N/O/P/Q dependency is now ready.

The ledger was originally anchored to an older Era-4 checkpoint and has historical value. Therefore do not blindly rewrite history. First decide, from repository intent and current checks, whether it is meant to be rolling current navigation or an anchor-time historical ledger whose use as the current navigation source has become stale.

## Proposal adjudication — execute without waiting

Run a short 1–3 shape proposal cycle and choose the smallest truthful shape in the same work session.

### ACCEPT — rolling-ledger reconciliation, if the file is intentionally current

If the ledger/checkers are intended to remain the current machine-readable opportunity map, reconcile demonstrably stale current classification/dependency facts against exact-current `IMPLEMENTATION_PLAN.md`, `ROADMAP.md`, `docs/status.md`, the release packet, and retained evidence.

At minimum address K's obsolete migration-back `BLOCKED_IMPLEMENTATION` state and any closure arrays/derived dependency explanations that must change with it. Inspect the whole ledger for similarly stale post-anchor current-line classifications before committing, because a partial repair that leaves mutually contradictory current navigation is worse than a narrow truthful reconciliation.

Do **not** infer that downstream rows become `OPEN_READY` merely because K changes. Every `OPEN_READY` row must still have a specific unresolved question, concrete evidence needed/next action, satisfied dependencies, and explicit scope. Preserve `READY_LIVE: none` unless repository evidence independently proves a genuinely new dependency-ready live question.

### ACCEPT_WITH_BOUNDS — preserve ledger as historical and repair navigation wording

If repository intent/checks show the ledger is an immutable or anchor-time historical artifact, preserve its historical classifications. Instead stop presenting it as the current machine-readable navigation source: update the release packet/current navigation wording so exact-current `IMPLEMENTATION_PLAN.md` / `docs/status.md` carries current readiness while the Era-4 file is clearly described as historical-at-anchor evidence.

Do not create a second competing current-status system.

### ACCEPT_WITH_BOUNDS — minimal current-overlay field

A small explicit current-overlay/current-as-of section is acceptable only if the existing ledger schema/checks already support this cleanly and it avoids rewriting anchor-time facts. Do not invent a large versioned ledger framework.

### REJECT

- inventing a new migration-back architecture or rerunning migration-back merely to make the ledger look current;
- declaring N/O/VPS work ready solely from a K status change;
- changing old artifacts or negative-result truth;
- weakening `READY_LIVE: none` without a new named real-network question;
- generic evidence-schema/checker expansion unrelated to the demonstrated drift.

## Required verification for the navigation repair

Before the full gate, run the focused existing checks that own this contract:

```text
python3 scripts/check-era4-closure.py
bash scripts/check-plan-sync.sh
```

If other existing status/release checks fail because the correction exposes a real inconsistency, repair the inconsistency rather than relaxing the checker.

Then commit/push the coherent repair and validate that **final exact developer SHA from a clean exact tree**:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # empty
```

Persist one concise provenance record after green with exact SHA, commands, UTC start/end, exit codes, host/OS/arch, stable Rust version, and initial/final clean-tree state. Do not wait for GitHub Actions and do not present developer-local CI as reviewer-run CI.

No fuzz is required for docs/JSON navigation-only work.

## Release and live boundaries

- `RELEASE_CANDIDATE=false`
- `PRODUCTION_READY=false`
- `FREEZE=false`
- `RELEASED=false`
- bounded release-evidence item 3 remains incomplete;
- independent release/security item 4 remains incomplete;
- D019 source retention remains `SOURCE_RETENTION_POLICY_BLOCKED`;
- current live opportunity remains `READY_LIVE: none` unless exact-current facts materially change it.

Standing VPS authorization remains valid, but there is no reason to duplicate HY2, repeated warm failover, periodic/soak, package lifecycle, A -> B -> A, already-answered migration-back, endpoint migration, key update, IPv6 without an owned IPv6 path, or live PMTUD without its separate implementation/security dependency. A stale ledger is not a reason to manufacture a WAN run.

## Rolling queue — execute continuously

There are fewer than 6–10 fully preselected truthful slices in exact-current repository state; do not manufacture backlog. The following dependency-ordered work is real. Complete it continuously and refill through proposal -> implementation rather than watcher mode.

### A. REVIEW-CLOSED — matrix operator contract

Treat exact `acb81bb` + developer-local provenance in `8749efa` as the closed local matrix-argv package. Do not reopen absent a new concrete contradiction.

Immediately continue to B.

### B. READY_LOCAL — adjudicate Era-4 ledger intent and reconcile current navigation

Perform the proposal adjudication above. Repair either the rolling ledger facts or the release packet's stale claim that the anchor-time ledger is current navigation. Preserve historical evidence and current `READY_LIVE: none` unless independently disproved.

Immediately continue to C.

### C. READY_LOCAL — dependency/classification consistency pass

Within the same coherent package, inspect downstream rows whose explanation depends on any corrected track. Update only dependency explanations/classifications that are logically forced by current truth. Do not promote a row just because one blocker disappeared; retain another real blocker or reclassify only when its declared question is actually dependency-ready.

Run `check-era4-closure.py`, `check-plan-sync.sh`, and any already-existing status/release focused checks. Immediately continue to D.

### D. READY_LOCAL — final exact-tree gate and provenance

Commit/push the coherent B/C repair, run the full clean exact-tree local gate on that final developer SHA, fix real failures, and persist one concise provenance note after green. GitHub-hosted CI is optional cross-evidence only.

Immediately continue to E.

### E. BOUNDED RELEASE RECONCILIATION — changed navigation facts only

Update `docs/release-security-review-packet.md`, item-4 factual support, or `docs/status.md` only where B/C materially changed a current claim. Do not mechanically re-anchor unrelated Session, pre-auth, package, matrix, or VPS evidence.

Then immediately continue to F.

### F. LOCAL OUTPUT SELECTION — 1–3 exact-current proposals, choose autonomously

Re-read current implementation plus `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, release packet, and the corrected navigation. Produce 1–3 **concrete** dependency-ready proposals, each naming owner file/API, observed contradiction or missing advertised behavior, protected invariant, minimal positive/negative evidence, and stop condition. Choose the smallest safe proposal in the same work session without reviewer acknowledgement.

Prefer, in order:

1. a demonstrated runtime/correctness/security contradiction with a concrete call site;
2. an advertised operator/runtime behavior lacking direct executable evidence;
3. a named release-evidence correctness question answerable locally without capacity/security claim inflation.

Do not select D019 TTL/LRU/history policy, signing/key-custody/SBOM policy, previous-release interoperability without a frozen prior release, service/production mutation, Experimental Track carriers, the deferred matrix `payload_bytes` semantic question, or generic checker/parser work merely to create backlog.

Immediately continue to G.

### G. READY_LOCAL — implement and exact-tree-close the selected output

Implement F's chosen real output, run focused tests followed by the full exact-tree local gate on its final pushed implementation/test SHA, persist minimal provenance, reconcile only changed facts, and continue if no real stop condition exists.

If coherent work repeatedly finishes in 10–30 minutes with no test/evidence quality decline, enlarge the next coherent package. Commit timing is sizing input, not a productivity score.

### H. ROLLING REFILL — repeat proposal -> implementation while genuine work exists

Repeat F -> G from repository truth. Maintain several dependency-ordered slices only when real gaps exist; if fewer real slices exist, say so rather than manufacturing docs/checker work.

### I. CONDITIONAL VPS — only for a newly created real-network question

Current classification remains `READY_LIVE: none`. Standing authorization remains valid but is not a reason to rerun closed or blocked lines.

Only execute a bounded self-owned VPS run when a new implementation/instrumentation/configuration/hypothesis creates a named unresolved real-network question with satisfied dependencies. Preserve old negatives; no unchanged retries.

### J. POLICY-BLOCKED PARALLEL LANE — D019 source retention

Status remains `SOURCE_RETENTION_POLICY_BLOCKED`. Do not invent TTL, LRU/history capacity, external authority, or a weaker no-reset rule. This does not block independent local correctness/operator work.

## Stop conditions

Stop/escalate only for an unresolved BLOCKER/HIGH that cannot safely be repaired from existing semantics, a required core Session/Carrier/ACK/crypto/wire architecture change, destructive/canonical-meaning migration, action outside standing authorization, production impact, new credentials/server/third-party permission, benchmark conditions requiring maintainer value judgment, D019 policy decision, real repository breakage, runtime/tool-budget exhaustion, or genuine queue exhaustion.

Otherwise: coherent slice -> focused checks -> exact-tree local gate -> commit/push -> immediately continue to the next pre-authorized dependency-ready slice.
