# ChatGPT reviewer handoff — unstick strict matrix argv implementation

## Reviewed state

- Previous reviewer-owned handoff: exact `8a36a6acb292e70803055337881a9f7f98df5cb9` (`docs(handoff): finish strict matrix argv closure`).
- Current default `main` before this reviewer update is still exact `8a36a6acb292e70803055337881a9f7f98df5cb9`; no developer-owned commit has landed after that handoff.
- Current reviewed developer implementation/test head therefore remains exact `71c97b5c53a69c4b24aad14021f7ea8aa204a01d` (`fix: classify invalid matrix probe arguments`).
- No developer documentation/provenance commit exists after `71c97b5`; do not treat that implementation as exact-tree-gated closure.
- GitHub still exposes zero hosted status records for exact `71c97b5`. Hosted CI is optional cross-evidence and must not become a wait condition.
- Open PR #3 remains the old external pre-auth branch and is unrelated to this matrix argv slice; do not merge/cherry-pick it as a substitute for current-main work.
- No new VPS/WAN experiment or live-evidence row landed. Current repository classification remains `READY_LIVE: none`.
- This reviewer performed repository/source review only; no reviewer-executed local CI is claimed.

## Coordination verdict

### IMPLEMENTATION STAGNATION / READY_LOCAL — do not wait for another reviewer cycle

The same dependency-ready matrix argv defect remains at the queue head while repository integrity, local validation policy, authorization, and implementation ownership are all clear. Under `AGENTS.md` §3.2 this is implementation stagnation, not a reason to keep polling.

The prior handoff already specified the invariant. This update narrows the implementation shape so the coding agent can execute immediately without another design round.

## Existing finding remains valid

### MEDIUM — matrix argv grammar accepts ignored/ambiguous arguments

Exact-current `matrix_probe` still uses a first-match helper based on `args.windows(2).find(...)`, and `json_mode` only tests whether `--json` appears anywhere. No pass proves that every token belongs to the matrix grammar. Therefore unknown flags, stray positional tokens, duplicate options, and a valid-first/invalid-second duplicate can be ignored while a real probe/artifact is produced.

This is operator/evidence correctness, not a demonstrated network-scope escape: `reachability::validate` still rejects non-loopback/public targets, family mismatch, port 0, timeout outside `1..=5000`, and payload outside `1..=1200` before the normal matrix call reaches socket work.

The release-facing tested-tree anchor must remain at the previous genuinely gated tree until the replacement matrix implementation has a real green exact-tree local gate.

## Proposal adjudication — implementation shape is now preselected

The coding agent does **not** need to wait for approval among these choices.

### ACCEPT — Shape A: one compact matrix-specific sequential parser

Add a private typed holder such as `MatrixProbeArgs` plus a small `parse_matrix_probe_args(&[String]) -> MatrixProbeArgs` in `crates/neko-cli/src/main.rs`.

Parse `args[1..]` exactly once with an index/iterator and explicit seen-state:

- `--matrix`: required exactly once, no value;
- `--target VALUE`: required exactly once;
- `--transport VALUE`: required exactly once;
- `--ip-version VALUE`: required exactly once;
- `--timeout-ms VALUE`: optional at most once, default `500`;
- `--bytes VALUE`: optional at most once, default `32`;
- `--json`: optional at most once, no value.

For every value-taking option, reject a missing next token and reject the case where the next token is another `--...` option instead of a value. Reject unknown flags, stray positional tokens, duplicate required/value options, duplicate `--matrix`, and duplicate `--json` immediately with existing `fail(...)` / exit 2. Never overwrite an already-populated field.

After structural parsing succeeds, parse the typed target/transport/version/numeric values and then call the existing `reachability::validate(...)` exactly before `reachability::run(...)`. Invalid syntax or semantics must therefore exit 2 with no matrix artifact/result on stdout and no socket work.

Keep the current `main` dispatch rule: ordinary authenticated `probe` remains untouched when `--matrix` is absent. Do not move generic probe arguments into this parser.

### ACCEPT_WITH_BOUNDS — Shape B: consumed-token bitmap around current parser

A consumed-token/duplicate checker around the existing first-match retrieval is acceptable only if it proves every argv token is consumed exactly once and duplicate options are rejected **before** first-match values can influence execution. If this becomes more code or more fragile than Shape A, abandon it and use Shape A.

### REJECT — Shape C: repository-wide CLI/parser framework

Do not refactor all commands, add a parser dependency, or normalize every historical command-line surface in this slice. That creates unrelated migration risk and is not required to close the observed defect.

## Required executable evidence

Keep the existing process test style in `crates/neko-cli/tests/probe.rs`; a separate parser framework/test harness is unnecessary.

Add a compact table-driven set that asserts exit `2` and empty stdout for at least:

- unknown `--bogus` option;
- stray positional token;
- duplicate `--target`;
- duplicate `--transport`;
- duplicate `--ip-version`;
- duplicate `--timeout-ms`;
- duplicate `--bytes`, specifically including `--bytes 17 --bytes 1201`;
- missing value after every value-taking option family (representative cases are acceptable if helper coverage is shared);
- duplicate `--matrix`;
- duplicate `--json`.

Preserve the existing semantic-invalid rows and the reachable TCP/UDP exit-0 cases.

### LOW fixture repair — fold into the same test package if convenient

The current completed-failure case binds an ephemeral TCP listener, records the address, drops the listener, then launches the child; another process could theoretically claim the released port.

Preferred minimal deterministic replacement: keep a loopback UDP socket bound and alive but deliberately never reply, run matrix UDP against that bound address with a short bounded timeout, and assert exit `1` plus `reachable=false`. Because the port stays bound, this avoids the bind-release race without adding port-allocation infrastructure.

If this replacement causes platform-specific instability, retain the existing TCP negative and document the LOW race rather than expanding scope.

## Exact-tree local-CI closure

Do not create a provenance-only commit for `71c97b5`. First land the strict parser + focused process tests as one coherent replacement implementation/test commit and push it.

Then validate **that exact developer SHA from a clean exact tree**:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # empty
```

Persist one concise provenance note only after green, recording exact SHA, commands, distinct UTC start/end, exit codes, host/OS/arch, stable Rust version, and initial/final clean-tree state. Preserve a real red result and fix its concrete cause; do not unchanged-rerun until lucky.

No fuzz is required unless this unexpectedly changes wire decoder/parser/crypto framing. This CLI argv parser is not the network wire parser.

## Evidence/claim boundary

Until that replacement exact tree is green:

- code exists for the earlier semantic-invalid repair: yes (`71c97b5`);
- strict argv closure exists: no;
- developer persisted full exact-tree local CI for the matrix package: no;
- reviewer-executed CI: none;
- hosted CI/status: no status records for `71c97b5`;
- WAN/live evidence: none;
- release/security/production conclusion: none.

Do not update release packet or item-4 tested-tree anchors to an ungated matrix commit.

## Rolling queue — execute continuously

The repository does not expose a truthful preselected 6–12 hours of concrete work without inventing tasks. Complete the real slices below, then refill through exact-current proposal -> implementation rather than watcher mode.

### A. READY_LOCAL — strict matrix argv parser

Implement accepted Shape A unless a smaller fully-auditable Shape B is clearly superior. Protect: every matrix token consumed exactly once; ambiguity rejected before socket work/evidence emission; ordinary authenticated probe unchanged.

Commit/push the coherent implementation and immediately continue to B.

### B. READY_LOCAL — grammar negatives and stable 0/1/2 process contract

Add the required invalid grammar rows and retain semantic-invalid/reachable coverage. Fold the held-silent-UDP completed-failure fixture into this same package if convenient.

Immediately continue to C.

### C. READY_LOCAL — final replacement exact-tree gate

Run the required local gate on the final pushed implementation/test SHA from a clean exact tree. Fix real failures, rerun, and save one provenance note after green. Never wait for GitHub-hosted CI.

Immediately continue to D.

### D. BOUNDED DOC/RELEASE RECONCILIATION — matrix facts only

Update `docs/reachability-matrix.md` to state the strict executable grammar/0-1-2 contract if the final code materially changes what the document says. Update `docs/status.md`, release packet, or item-4 factual support only if a fact they currently claim actually changed.

Do not mechanically re-anchor unrelated Session, RSEC, package, or VPS evidence. The deferred `payload_bytes` cross-transport/versioned-meaning question remains **DEFERRED**.

Immediately continue to E.

### E. REVIEW-CLOSURE — one bounded matrix call-path check, then close the lane

Review only:

```text
dispatch -> strict argv parse -> semantic validate -> local socket -> artifact/human output -> exit code
```

Fix only a concrete contradiction. If clean, explicitly close this matrix operator-contract lane. Do not continue into generic parser/checker work.

Immediately continue to F.

### F. LOCAL OUTPUT SELECTION — propose 1–3 exact-current real outputs and choose autonomously

Re-read exact-current `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, release packet, and the implementation owned by each candidate. Produce 1–3 concrete dependency-ready proposals and choose the smallest safe one in the same work session without waiting for reviewer acknowledgement.

Prefer:

1. a demonstrated runtime/correctness/security contradiction with a concrete call site;
2. an advertised operator/runtime behavior lacking direct executable evidence;
3. a named release-evidence question answerable locally without capacity/security claim inflation.

Each proposal must identify owner file/API, observed contradiction/missing behavior, protected invariant, minimal positive/negative evidence, and stop condition.

Do not select D019 TTL/LRU/history policy, signing/key-custody policy, SBOM publication policy, previous-release interoperability without a frozen prior release, service/production mutation, Experimental Track carriers, the deferred matrix `payload_bytes` semantic question, or generic checker work merely to create backlog.

Immediately continue to G.

### G. READY_LOCAL — implement and exact-tree-close the selected output

Implement the chosen F output, run the full exact-tree local gate on its final pushed implementation/test SHA, reconcile only changed facts, and continue to H when no real stop condition exists.

If coherent work repeatedly finishes in 10–30 minutes without rushed tests/evidence, enlarge the next coherent package. Commit timing is sizing input, not a productivity score.

### H. READY_LOCAL / ROLLING REFILL — repeat proposal -> implementation while genuine work exists

Repeat F -> G from repository truth. Keep several dependency-ordered slices ready when genuine gaps exist; if only fewer real slices exist, state that honestly rather than manufacturing checker/docs work.

### I. CONDITIONAL VPS — only when exact-current implementation creates a new real-network question

Current opportunity classification is `READY_LIVE: none`. Standing authorization remains valid but is not a reason to duplicate evidence.

Only execute a VPS run when a new implementation/instrumentation change creates a named unresolved real-network question with satisfied dependencies. Do not unchanged-rerun HY2, repeated warm failover, periodic/soak, package lifecycle, A -> B -> A, already-answered endpoint migration/key update, IPv6 without a real owned IPv6 path, or live PMTUD before its separate authenticated wire/security gate.

### J. POLICY-BLOCKED PARALLEL LANE — D019 source retention

Status remains `SOURCE_RETENTION_POLICY_BLOCKED`. Do not invent TTL, LRU/history capacity, external authority, or a weaker no-reset rule. This does not block dependency-independent local correctness/operator work.

## Stop conditions

Stop/escalate only for an unresolved BLOCKER/HIGH that cannot be safely repaired from existing semantics, a required change to core Session/Carrier/ACK/crypto/wire architecture, destructive/canonical-meaning migration, action outside standing authorization, production impact, new credentials/server/third-party permission, benchmark conditions requiring maintainer value judgment, D019 policy decision, real repository breakage, runtime/tool-budget exhaustion, or genuine queue exhaustion.

Otherwise: coherent slice -> exact-tree local gate -> commit/push -> immediately continue to the next pre-authorized dependency-ready slice.
