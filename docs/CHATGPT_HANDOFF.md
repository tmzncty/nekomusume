# ChatGPT reviewer handoff — finish strict matrix CLI closure before next output

## Reviewed state

- Previous reviewer-owned handoff: exact `7f40695705cce479096f80671808a0f45b5ea57a` (`docs(handoff): close ledger lane and repair matrix probe contract`).
- Previous reviewed developer implementation/test head: exact `0f8f19251bdb08c9179265ff26ed432fd07b894d` (`fix: reject globally stale advanced duplicates`).
- Previous reviewed developer documentation/evidence head: exact `c87519df3fb0a2f9067d23e1ca3aed60ca2af24f` (`docs: close global advanced duplicate rollback`).
- Current developer implementation/test head reviewed this cycle: exact `71c97b5c53a69c4b24aad14021f7ea8aa204a01d` (`fix: classify invalid matrix probe arguments`).
- No new developer documentation/provenance commit has landed after `71c97b5`; do not treat it as exact-tree-gated closure yet.
- Default `main` before this reviewer update was exact `71c97b5c53a69c4b24aad14021f7ea8aa204a01d`, exactly one developer commit ahead of the previous reviewer handoff.
- New sequence classification: one local CLI/operator implementation+test package. No research-only reconstruction and no VPS/WAN experiment landed in this sequence.
- GitHub exposes no hosted combined-status records for exact `71c97b5`. This is not a blocker and must not trigger Actions polling.
- This reviewer performed source/repository review only; no reviewer-executed local CI is claimed.

## Review verdict

### ACCEPT_WITH_BOUNDS — `71c97b5` fixes the originally reported semantic-invalid exit-class defect

The developer implemented the minimum pre-validation shape requested by the prior handoff:

- syntactically invalid `--timeout-ms` / `--bytes` no longer silently fall back to defaults;
- non-loopback target, IP-family mismatch, port `0`, timeout outside `1..=5000`, and payload outside `1..=1200` are validated before `reachability::run` and therefore exit through the CLI invalid-argument path;
- the local matrix operation still refuses non-loopback/public targets before socket work;
- process coverage now distinguishes invalid exit `2`, a completed failed probe exit `1`, and reachable TCP/UDP exit `0`;
- top-level help now exposes the canonical `probe --matrix` local-loopback form without inventing another command;
- the existing `reachability-matrix.v1` `observed_at_unix_ms` field is populated from wall clock when available, with `null` remaining schema-compatible on clock conversion failure.

The optional timestamp change does not alter the schema version or any network scope. No Session/Carrier/ACK/Noise/wire/crypto semantics changed, so no fuzz smoke is required for this package.

However, exact `71c97b5` has no persisted exact-tree `scripts/check.sh` provenance yet, and the bounded adjacent review below found one remaining operator-contract defect in the same argv surface. Do **not** spend a separate provenance commit on `71c97b5`; repair the remaining argv defect first, then gate the replacement exact tree once.

## New concrete local finding

### MEDIUM / READY_LOCAL — matrix argv grammar still accepts ignored/ambiguous arguments instead of exit `2`

The documented contract says invalid arguments exit `2`, but exact-current `matrix_probe` is still a permissive first-match parser:

- `get(key)` uses `args.windows(2).find(...)`, so only the first occurrence of a value option is consumed;
- no pass verifies that every token belongs to the matrix grammar;
- `json_mode` only checks whether `--json` appears anywhere.

Therefore unknown flags, stray positional tokens, and duplicate options can be silently ignored while a real probe/artifact is produced. Examples of the defect shape include:

```text
neko probe --matrix --target 127.0.0.1:9 --transport tcp --ip-version ipv4 --bogus x
neko probe --matrix --target 127.0.0.1:9 --transport tcp --ip-version ipv4 --bytes 17 --bytes 1201
```

In the second form the first `--bytes 17` wins and the later invalid value is ignored rather than making the invocation invalid. The same ambiguity exists for repeated target/transport/IP-version/timeout options.

This is **operator/evidence correctness**, not a demonstrated network-scope escape: the selected target still passes the explicit loopback validator. But an evidence-producing CLI must not claim a completed observation when supplied arguments were ignored or ambiguous.

#### Required invariant

For matrix mode only, before socket work:

- the grammar accepts only the command token `probe`, exactly one `--matrix`, exactly one `--target VALUE`, exactly one `--transport VALUE`, exactly one `--ip-version VALUE`, optional at-most-once `--timeout-ms VALUE`, optional at-most-once `--bytes VALUE`, and optional at-most-once `--json`;
- unknown flags, stray positional tokens, duplicate value options, duplicate boolean flags, and missing values exit `2`;
- invalid input emits no reachability artifact/result on stdout;
- non-loopback/public targets still fail before network probing;
- valid completed failure remains exit `1`; valid reachable local probe remains exit `0`;
- ordinary authenticated `probe` behavior when `--matrix` is absent is unchanged.

Preferred minimum implementation shape: add a compact matrix-specific argv validation/parser pass that yields the already-existing typed values, or add a small consumed-token/duplicate check around the existing parser. Do **not** build a repository-wide CLI framework in this slice.

#### Minimum executable evidence

Add compact process regressions for at least:

- unknown `--bogus` option -> exit `2`, no stdout result;
- stray positional token -> exit `2`;
- duplicate `--target`, `--transport`, and `--ip-version` -> exit `2`;
- duplicate `--timeout-ms` / `--bytes`, including a valid first value plus invalid second value -> exit `2`;
- missing value after a value-taking option -> exit `2`;
- duplicate `--matrix` and duplicate `--json` -> exit `2` if the strict grammar above is selected;
- all existing semantic invalid cases and 0/1/2 valid outcome cases remain green.

### LOW / SAME TEST PACKAGE — the current TCP-refused negative has an avoidable port-reuse race

The new process test obtains an ephemeral TCP port by binding `127.0.0.1:0`, drops the listener, and only then launches the child probe. That creates a small bind-release-to-connect race in which another local process could claim the port and make a nominal failure case reachable.

Do not build port-allocation infrastructure for this. If touching the test package, prefer a deterministic bounded completed-failure fixture, for example a UDP socket kept bound on loopback but deliberately not responding while the probe uses a short bounded read timeout. If the current TCP-refused case remains stable under the full gate, this is LOW and may be left alone; do not turn it into a separate lane.

## Evidence/claim boundary for `71c97b5`

The code and tests are present, but no persisted developer-local exact-tree gate exists for this SHA. The correct current claim is therefore:

- **code exists:** yes;
- **focused executable tests are present in-tree:** yes;
- **developer-reported/persisted full exact-tree local CI:** not yet for `71c97b5`;
- **reviewer-executed CI:** none this cycle;
- **GitHub-hosted CI/status:** none exposed for `71c97b5`;
- **WAN/live evidence:** none;
- **release/security/production conclusion:** none.

Do not update release-facing tested-tree anchors to `71c97b5` or to a later replacement until the final replacement implementation/test SHA has a real green exact-tree local gate.

## Local-CI-first rule

After the strict matrix argv repair and its focused tests are committed/pushed as one coherent replacement implementation/test SHA, validate that exact SHA from a clean temporary worktree/clone:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # empty
```

Persist minimum provenance only after the final implementation/test SHA is green: exact SHA, commands, distinct UTC start/end timestamps, exits, host/OS/arch, stable Rust version, and initial/final clean-tree state. If red, repair the concrete failure and rerun the relevant/full gate; do not unchanged-rerun until lucky.

No fuzz is required unless the repair unexpectedly changes wire decoder/parser/crypto framing. Developer-local persisted CI, reviewer source review, reviewer-executed CI, and GitHub-hosted CI remain separate evidence classes. Hosted CI is optional cross-evidence and must never become a wait condition.

## Rolling queue — execute continuously in dependency order

The repository does not currently expose a truthful 6-12 hours of preselected work without inventing tasks. Keep the concrete queue below, then refill autonomously through the proposal/implementation loop rather than entering watcher mode.

### A. READY_LOCAL — make matrix argv consumption strict and unambiguous

Goal: close the remaining invalid-argument classification hole without changing network scope.

Files/concepts: `crates/neko-cli/src/main.rs`, matrix-specific argument parsing/validation.

Protected invariant: no ignored/ambiguous matrix argument may survive to socket work or evidence emission.

Implement the minimum grammar described above. Preserve ordinary authenticated `probe` behavior.

Continue immediately to B.

### B. READY_LOCAL — extend the 0/1/2 process matrix with grammar negatives

Add the unknown/stray/duplicate/missing-value cases above and preserve all current semantic invalid, completed-failure, reachable TCP, reachable UDP, help, and timestamp coverage.

If convenient and deterministic, replace the dropped-listener TCP failure fixture with a held local silent UDP failure fixture; otherwise leave that LOW alone unless it actually causes gate instability.

Continue immediately to C.

### C. READY_LOCAL — final exact-tree local gate and concise provenance

Push the coherent implementation/test replacement first. On that exact clean tree run `scripts/check.sh`, `git diff --check`, and clean-tree verification. Preserve a real red result if one occurs and fix the concrete cause. Save one concise provenance note only after green.

Do not create a second CI framework and do not wait for GitHub Actions.

Continue immediately to D.

### D. BOUNDED DOC/RELEASE RECONCILIATION — update only matrix facts actually changed

Reconcile `docs/reachability-matrix.md` so it truthfully describes the executable 0/1/2 classification and, if useful, that completed v1 artifacts now carry `observed_at_unix_ms` when the host clock is representable.

Update `docs/status.md`, the release packet, or item-4 factual support only if the final matrix package materially changes a fact those documents claim. Do not mechanically re-anchor unrelated Session, RSEC, package, or VPS evidence merely because a newer commit exists.

The previously deferred `payload_bytes` cross-transport meaning remains **DEFERRED**. Do not silently reinterpret a versioned artifact field in this package.

Continue immediately to E.

### E. REVIEW-CLOSURE / READY_LOCAL — one bounded matrix call-path review, then stop this lane

Review exactly:

```text
dispatch -> strict argv parse -> semantic validate -> local socket operation -> artifact/human output -> exit code
```

Fix only a concrete correctness/evidence contradiction. If clean, explicitly close this matrix operator-contract lane. Do not extend into a generic CLI/parser/checker campaign.

Continue immediately to F.

### F. LOCAL OUTPUT SELECTION — propose 1-3 real outputs and choose autonomously

Re-read exact-current `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, release packet, and the implementation owned by each candidate. Produce 1-3 concrete dependency-ready proposals and choose the smallest safe one without waiting for the next reviewer hour.

Prefer, in order:

1. a demonstrated runtime/correctness/security defect with a concrete call site;
2. an advertised operator/runtime behavior lacking direct executable evidence;
3. a named release-evidence question answerable locally without capacity/security claim inflation.

Each proposal must name the observed contradiction/missing behavior, owner file/API, protected invariant, minimal positive/negative evidence, and stop condition.

Do not select D019 TTL/LRU/history policy, signing/key-custody policy, SBOM publication policy, prior-release interoperability without a frozen prior release, service/production mutation, Experimental Track carriers, the deferred `payload_bytes` versioned-meaning question, or generic checker work merely to create backlog.

Continue immediately to G.

### G. READY_LOCAL / ROLLING VISIBLE OUTPUT — implement the selected output and close it

Implement the chosen F output, run the exact-tree local gate on the final pushed implementation/test SHA, reconcile only changed facts, and continue to H rather than waiting for reviewer acknowledgement when no real stop condition exists.

If coherent work repeatedly finishes in 10-30 minutes without rushed tests/evidence, enlarge the next coherent package. Commit timing is sizing input, not a productivity score.

### H. READY_LOCAL / ROLLING REFILL — repeat proposal -> implementation while safe concrete work exists

Repeat F -> G from exact-current repository truth. Keep several dependency-ordered slices ready when genuine gaps exist; if the repository only exposes fewer real slices, state that honestly rather than manufacturing six hours of checker/docs work.

### I. CONDITIONAL VPS OUTPUT — only when exact-current truth creates a new live question

Current repository classification remains `READY_LIVE: none`. Standing VPS authorization is valid but is not a reason to duplicate old evidence.

Only execute a VPS run if a new implementation/instrumentation change creates a named unresolved real-network question with satisfied dependencies. Otherwise do not unchanged-rerun HY2, repeated warm failover, periodic/soak, package lifecycle, distinct A -> B -> A, already-answered endpoint migration/key update, IPv6 without a real owned IPv6 path, or live PMTUD before its separate authenticated wire/security design gate.

### J. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

The source-accounting retention/no-reset conflict remains a maintainer/security-policy question. Do not invent TTL, LRU/history capacity, external authority, or a weaker reset rule. This policy lane does not block dependency-independent local correctness/release work above.

## Stop conditions

Stop and escalate only for an unresolved BLOCKER/HIGH that cannot be safely repaired from existing semantics, a required change to core Session/Carrier/ACK/crypto/wire architecture, destructive/canonical-meaning migration, action outside standing authorization, production impact, new credentials/server/third-party permission, benchmark conditions requiring maintainer value judgment, D019 policy decision, real repository breakage, runtime/tool-budget exhaustion, or genuine queue exhaustion.

Otherwise: coherent slice -> exact-tree local gate -> commit/push -> immediately continue to the next pre-authorized dependency-ready slice.
