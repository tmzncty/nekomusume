# ChatGPT reviewer handoff — review wire closure and continue Session lane

## Reviewed state

- Previous reviewer-owned handoff: exact `b429bd338a3a364afbc9027149722f03044a51d3` (`docs(handoff): close matrix lane and reconcile Era-4 navigation`).
- Current default `main` before this reviewer update: exact `37b931935b5d3208ae76878304a4d92c5b92e748` (`docs: close bounded frozen wire corpus review`).
- New developer sequence after the previous handoff:
  1. exact `8fdd4bdba4b48a38d399d03d04a185184ff12319` — docs/navigation (`docs: reconcile Era-4 rolling navigation`);
  2. exact `92de66e4283cb17346ef438075be8493be51c868` — docs/navigation/release/status alignment (`docs: align Era-4 rolling navigation`);
  3. exact `525b61be78c628fd153164d514401305c960b460` — developer-local exact-tree provenance for exact `92de66e` (`docs: record Era-4 navigation provenance`);
  4. exact `37b931935b5d3208ae76878304a4d92c5b92e748` — bounded frozen-wire-corpus review plus B-row navigation closure (`docs: close bounded frozen wire corpus review`).
- No new runtime implementation, fixture reconstruction, package mutation, or VPS/WAN experiment landed in this sequence.
- GitHub exposes zero hosted status records for exact `92de66e` and exact `37b9319`. Hosted CI remains optional cross-evidence and is not a wait condition.
- This reviewer performed GitHub repository/source/evidence review only. No reviewer-executed local CI is claimed.

## Review verdict — Era-4 navigation reconciliation ACCEPTED

The previous navigation MEDIUM is closed by `8fdd4bd` + `92de66e`, with developer-local exact-tree evidence recorded by `525b61b`.

The repository now truthfully distinguishes the immutable 2026-08-30 historical anchor from the rolling current-navigation overlay. K is no longer incorrectly `BLOCKED_IMPLEMENTATION`; exact `5d6582c` is limited to one bounded scripted migration-back implementation/process/VPS question. N is limited to the existing nine-scenario privileged ICMP/netem harness and is not Nekomusume throughput/performance evidence. O aggregates already-answered bounded self-owned VPS questions and creates no new live row. The obsolete K -> N -> O dependency-block explanation is removed without promoting unrelated downstream governance/orchestration/implementation gates.

The persisted exact-tree developer-local gate for exact `92de66e` records:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh  -> exit 0
git diff --check                                -> exit 0
gate UTC: 2026-09-10T17:14:24Z -> 2026-09-10T17:16:20Z
host: Linux 6.8.0-137-generic x86_64
rustc: 1.98.0 (88d9e12ae 2026-08-18)
initial/final source tree: clean
```

Focused `check-era4-closure.py`, plan-sync, status-evidence, release-boundary and markdown-link checks were also recorded green. This is developer-local navigation/evidence consistency, not runtime, WAN, security review, RC, release or production evidence.

## Review verdict — B frozen-wire-corpus question ACCEPT_WITH_BOUNDS

Exact `37b9319` closes only the **bounded B-row question** against the already-frozen canonical corpus. The developer-run review was executed on reachable exact source tree `525b61be78c628fd153164d514401305c960b460` and records:

- frozen canonical corpus remains exactly 42 vectors across 10 required domains with `freeze=true`;
- corpus schema/identity validators and generated review checks passed;
- real `neko-wire` implementation adapters and candidate compatibility/property tests passed;
- Session readiness fuzz seeds remained exact process messages;
- isolated 30-second decode fuzz smoke completed without a crash;
- the mutable legacy fuzz-seed corpus remains distinct from frozen canonical vectors;
- no prior frozen release exists, so previous/current interoperability remains inapplicable rather than silently proven.

The classification update to `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION` is therefore acceptable only for this corpus/candidate compatibility question. It does **not** freeze Noise transcripts, ciphertext, Carrier packetization, failover/resume, the global protocol, RC, release, security or production behavior.

Do not reopen B merely to run more vectors or fuzz time unless a concrete contradiction, wire change, corpus change, or compatibility question appears.

## New finding — MEDIUM final-tree provenance gap for `37b9319`

The wire review commands were run against exact `525b61b`, then exact `37b9319` changed the machine-readable Era-4 closure, Markdown map, release packet, status text, and added the wire review note itself. No persisted full `scripts/check.sh` gate exists for the **final classification-changing tree `37b9319`**.

This does not invalidate the underlying wire/fuzz evidence and is not a security/correctness HIGH. It is a release-navigation/docs-evidence closure gap: the final tree that moves B from `OPEN_READY` to `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION` should itself pass the repository policy/status/link gates.

### READY_LOCAL — close this first without waiting

On an exact clean tree at `37b931935b5d3208ae76878304a4d92c5b92e748`, run at minimum:

```text
python3 scripts/check-era4-closure.py
bash scripts/check-plan-sync.sh
bash scripts/check-status-evidence.sh
bash scripts/check-release-boundaries.sh
bash scripts/check-markdown-links.sh
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # empty
```

If green, persist one concise developer-local provenance note anchored to exact `37b9319`. Do not rerun fuzz merely for the docs-only classification commit; the isolated decode fuzz evidence already belongs to the underlying bounded B review. If a policy/status/link check fails, repair the real inconsistency and gate the replacement final tree instead.

Do not wait for GitHub Actions.

## Next concrete lane — C Session delivery ledger bounded review

After the final B/docs provenance package is green, immediately enter C. Current machine-readable navigation has `C` as the earliest remaining `OPEN_READY` row; its dependency B is now bounded-question sufficient.

Do **not** restart the already-closed overlap/global-context checker spiral. Exact-current Session evidence already covers:

- fail-closed novel-byte overlap on `InFlight` / `Uncertain` / `Confirmed` ranges;
- ledger-global context rollback rejection for advanced exact duplicates;
- context-exact advanced duplicate idempotence;
- confirmation context atomically advancing the ledger-wide context;
- rejected insertion/confirmation preserving segment state, context, bytes and watermark.

Instead perform one bounded review of the remaining candidate state-transition semantics and either close C or repair a concrete contradiction.

### Specific question to inspect first — stale `Unsent -> InFlight` context

Exact-current `DeliveryLedger::transition` changes `Unsent -> InFlight` and `InFlight -> Uncertain` without consulting ledger-global `SessionContext`. A concrete scenario exists to examine:

1. insert segment A under context `(delivery_epoch=1, key_phase=0, path_generation=1)` and leave A `Unsent`;
2. advance ledger-global context through a disjoint valid operation to `(1,1,2)`;
3. call `mark_in_flight` on A, which currently carries the older segment context and takes no context argument.

Do not assume this is automatically a bug. Determine from `docs/spec/m0-session-state.md`, `docs/specs/nekomusume-session-v0.md`, callers/tests and current recovery semantics whether an `Unsent` segment is intentionally allowed to retain assignment-time context across a later global key/path advance, or whether making it `InFlight` would manufacture stale advanced evidence that violates the component-wise monotonic context model.

### Proposal adjudication for that question

- **ACCEPT** a minimal guard or typed transition only if repository semantics require the segment to be current/admissible at the moment it becomes `InFlight`. Protect atomic failure and do not redesign Session/Carrier/ACK/wire semantics.
- **ACCEPT_WITH_BOUNDS** an explicit tested/documented assignment-time-context rule if exact-current semantics genuinely allow queued `Unsent` bytes to become `InFlight` under their retained older context and later confirmation/migration rules already bound the evidence safely.
- **DEFER** if the question cannot be resolved without changing core Session semantics; record the exact ambiguity rather than inventing a policy.
- **REJECT** generic interval-framework work, a second ledger implementation, broad state-machine rewrites, or another checker-only expansion without a demonstrated contradiction.

Minimal evidence should include a focused positive/negative test around the disjoint-context scenario plus the existing transition/confirmation regressions. If a real defect is found, fix it and re-run the focused Session tests before the full gate. If no contradiction remains, close only the bounded C-row question with an explicit candidate limitation; do not claim protocol freeze or complete Session validation.

## Rolling queue — execute continuously

The repository currently exposes seven genuine local `OPEN_READY` rows: C, D, E, F, L, M and T. They are real review/output questions, but several may already be materially answered by current evidence and therefore should be **reviewed and reclassified**, not mechanically reimplemented or rerun. Keep moving without watcher mode.

### A. READY_LOCAL — exact `37b9319` final-tree gate/provenance

Run the final docs/navigation gate described above. Persist concise developer-local provenance only after green. Immediately continue to B.

### B. READY_LOCAL — C Session ledger bounded review

Resolve the stale-`Unsent -> InFlight` context question first, then do one bounded pass over `insert -> mark_in_flight -> mark_uncertain -> confirm_received -> watermark/limits`. Repair only concrete contradictions. Otherwise classify C bounded-question sufficient with explicit candidate boundaries. Immediately continue to C.

### C. READY_LOCAL — D authenticated-record/security matrix

Once C is truthfully sufficient, run the existing crypto/authenticated-record negative/security matrix against exact current code and inspect for one concrete unresolved candidate gap: authentication failure, replay/nonce/key-phase/AAD/context binding, malformed/truncated record, or fail-closed boundary. Do not invent a new crypto construction or security capacity policy. If source changes touch external record parsing/framing, use the repository-pinned fuzz path; otherwise do not rerun fuzz merely to fill time.

Close D only if the bounded question is actually answered. Immediately continue to D.

### D. READY_LOCAL — E/F Carrier deterministic boundary package

Review UDP lifecycle/encrypted loopback and TCP framing/failover deterministic gates as one coherent Carrier package where practical. Preserve Session-delivery vs packet/TCP feedback separation, bounded deadlines, cleanup and single-active failover invariants. Fix concrete reproducible contradictions; otherwise close only the bounded local E/F questions. Do not create WAN claims and do not reopen already-sufficient failover evidence solely for more sampling.

Immediately continue to E.

### E. READY_LOCAL — T unreliable-datagram boundary

Run/review the existing authenticated unreliable-datagram boundary tests. Protect the current provisional contract: <=1200-byte payload cap, no retransmission or ACK, no ordering guarantee, no promotion to reliable Session delivery/effect evidence, bounded mixed-queue behavior. Repair a concrete local contradiction if present; otherwise close the bounded T question. No new user-level datagram CLI is required by this row.

Immediately continue to F.

### F. READY_LOCAL — L observability/timeline evidence reconciliation

Inspect retained structured artifacts and current runtime events for one actually missing advertised metric. Prefer a direct runtime/operator evidence gap over new schema/checker machinery. If all bounded timeline questions already have sufficient retained evidence, reclassify L accordingly instead of manufacturing another WAN run. Current `READY_LIVE: none` remains authoritative absent a new named real-network question.

Immediately continue to G.

### G. READY_LOCAL — M deterministic-harness classification check

Do not rerun `scripts/check.sh` or fuzz only because M says `OPEN_READY`; those gates are already exercised repeatedly. Decide whether current deterministic regression/fault evidence already answers M's declared bounded question. If yes, reconcile M to bounded-question sufficient. If not, require one specific missing failure mode tied to a current runtime invariant before adding new harness code.

Immediately continue to H.

### H. ROLLING OUTPUT REFILL — 1–3 exact-current proposals, choose autonomously

After each closure package, re-read exact current repository truth and propose 1–3 concrete outputs naming owner file/API, demonstrated contradiction or missing advertised behavior, protected invariant/evidence boundary, focused positive/negative test and stop condition. Choose the smallest safe item in the same work session and implement/review it without reviewer acknowledgement.

Prefer:

1. correctness/security contradiction with a concrete call site;
2. advertised operator/runtime behavior lacking executable evidence;
3. a real release-evidence subgate answerable locally without claim inflation.

Do not select D019 retention policy, signing/key-custody/SBOM policy, previous-release interoperability without a previous frozen release, service/production mutation, Experimental Track work without observed-problem evidence, or generic checker/parser normalization merely to manufacture backlog.

For every coherent implementation/test/docs-evidence package: focused checks -> commit/push -> clean exact-tree `scripts/check.sh` + `git diff --check` -> concise provenance -> immediately continue if the next dependency-ready slice is pre-authorized and no real stop condition exists.

## Release and live boundaries

- `RELEASE_CANDIDATE=false`
- `PRODUCTION_READY=false`
- `FREEZE=false`
- `RELEASED=false`
- `CANONICAL_CORPUS_V1_FROZEN=true` applies only to the 42-vector/10-domain corpus identity.
- bounded release-evidence item 3 remains incomplete;
- independent release/security item 4 remains incomplete;
- D019 source retention remains `SOURCE_RETENTION_POLICY_BLOCKED`;
- current live opportunity remains `READY_LIVE: none`.

Standing VPS authorization remains valid, but no newly dependency-ready real-network question exists. Do not repeat HY2, repeated warm failover, periodic/soak, package lifecycle, already-answered migration-back, endpoint migration, key update, IPv6 without an owned IPv6 path, or live PMTUD without its separate implementation/security dependency. Negative evidence remains retained.

## Stop conditions

Stop/escalate only for an unresolved BLOCKER/HIGH that cannot safely be repaired from existing semantics, a required core Session/Carrier/ACK/crypto/wire architecture change, destructive/canonical-meaning migration, action outside standing authorization, production impact, new credentials/server/third-party permission, benchmark conditions requiring maintainer value judgment, D019 policy decision, real repository breakage, runtime/tool-budget exhaustion, or genuine queue exhaustion.

Otherwise: coherent slice -> focused checks -> exact-tree local gate -> commit/push -> immediately continue to the next pre-authorized dependency-ready slice.
