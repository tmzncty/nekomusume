# ChatGPT reviewer handoff — close cross-command identity evidence, then repair canonical failover CLI

## Reviewed state

- Previous reviewer-owned handoff: exact `c03b69d1579d80e8719e4eec2f7c4f30e2c9f651` (`docs(handoff): finish identity validation across command paths`).
- Current developer-owned docs head reviewed this cycle: exact `29f2132cf75cf1234021467505e0c75fea51ef66` (`docs: close cross-command identity provenance`).
- Developer sequence since the previous reviewer handoff:
  - `ad3059c9e31434adfb1b94e5dfa899cc4851c3ef` — persists local exact-`489c345` identity/config evidence and renames the stale exact-`0e664d6` identity-security note.
  - `3164030d428fb4b9ff8db90bd9ddd3fc49e473a0` — reorders deterministic failover/endpoint-rebind bind/address and peer-key validation before persistent identity creation, and routes those peer keys through the shared exact-32-byte parser.
  - `29f2132cf75cf1234021467505e0c75fea51ef66` — re-anchors release-facing identity prose to exact `3164030`.
- Exact `3164030` has GitHub-hosted Rust CI run `34345047449` with successful `stable checks` (`bash scripts/check.sh`) and successful pinned nightly decode fuzz smoke. This is **hosted cross-evidence only**. It is not developer-local exact-tree provenance and is not reviewer-executed CI.
- Work branches remain stale coordination artifacts:
  - `work/e1a-staged-accounting-20260907` = `f4404257520e9a014ac4e785b0ab9a97f8aaf794`; current main contains it and is 82 commits ahead.
  - `work/continue-20260904` = `d271a99a2ab26abbcb146c411ba0fde697395abe`; current main is 166 commits ahead while the branch retains one old unique commit. Do not merge either merely to manufacture work.

## Review verdict

### ACCEPT — cross-command implementation ordering at exact `3164030`

The implementation repair is the intended minimal shape and does not change Session, Carrier, ACK, Noise, crypto framing, or wire semantics:

- `failover_server` now parses both bind addresses and validates `--client-key` with the shared exact-32-byte peer-key helper before `load_or_generate`;
- `failover_client` parses the target address and validates `--server-key` before identity creation;
- `endpoint_rebind_server` parses its bind address and validates `--client-key` before identity creation;
- `endpoint_rebind_client` validates the already-address-parsed `--server-key` before identity creation.

The adjacent source review now finds ordinary server/client, failover server/client, and endpoint-rebind server/client performing deterministic peer-key/address/bind validation before persistent identity creation. `keygen` has no remote peer/address configuration to prevalidate. If focused process tests below remain green, the identity/config audit lane should be closed rather than expanded into new filesystem policy.

### HIGH / READY_LOCAL — exact-tree local provenance and focused cross-command regression claims are not yet closed

Do **not** treat `29f2132` as a completed release-facing closure yet.

Current release-facing docs say their reviewed/tested tree is exact `3164030`, but the repository does not retain a developer-local exact-`3164030` CI/provenance note. The existing `docs/local-identity-config-489c345-20260909.md` predates the cross-command implementation repair and therefore cannot attest the changed failover/endpoint-rebind code.

There is also an internal exact-tree inconsistency in `docs/reviews/release-item4-subgates-20260909.md`: its title/scope say exact `3164030`, while its `Exact-tree gates` paragraph still says the developer-local gate was run on exact `489c345` and cites the `489c345` evidence note. Hosted run `34345047449` does not repair that local-provenance mismatch.

Finally, exact `3164030` changed only `crates/neko-cli/src/main.rs`; it did not add the focused failover/endpoint-rebind process-negative matrix requested by the previous handoff. Existing generic invalid-configuration coverage is not a substitute for proving every newly reordered command family.

This is a **HIGH release/evidence correctness finding**, not a claim that the implementation logic itself is currently wrong. The fix is tests + exact-tree local evidence + factual reconciliation, not another security framework.

Minimum focused regression matrix, preferably table-driven to avoid duplication. Every case starts with a nonexistent temp identity path, exits nonzero, and asserts that the identity remains absent:

1. `failover-server`: malformed hex and valid-hex 31/33-byte `--client-key`; malformed `--udp-bind`; malformed `--tcp-bind`.
2. `failover-client`: valid-hex 31/33-byte `--server-key`; malformed address/target syntax.
3. `endpoint-rebind-server`: valid-hex 31/33-byte `--client-key`; malformed bind syntax.
4. `endpoint-rebind-client`: valid-hex 31/33-byte `--server-key`.
5. Keep ordinary server/client invalid-config, restrictive-umask exact-0600, descriptor-bound reload, symlink/special/insecure-mode rejection, and valid command paths green.

If these tests expose a missed ordering defect, repair it in the same coherent developer slice. Otherwise do not churn the already-correct implementation.

### MEDIUM / READY_LOCAL / VISIBLE OUTPUT — advertised canonical `failover` command is not actually executable

After the HIGH above closes, switch away from identity auditing to a visible operator/runtime output.

Current CLI help and both human/JSON capability reports advertise `failover` as the canonical command and describe `failover-server|failover-client` as legacy aliases. Dispatch also sends `failover` into `failover_gate`. But `failover_gate` only accepts `failover-server` or `failover-client`; the canonical `neko failover ...` path always fails with `use failover-server or failover-client`.

This is a concrete CLI contract drift, not a wire/protocol design issue. The coding agent should run a short proposal cycle and autonomously choose the smallest fail-closed repair. Acceptable shapes include:

1. `neko failover --role server|client` delegating to the existing implementations;
2. `neko failover server|client` as a subcommand form;
3. if strictly smaller and clearer, stop pretending `failover` is canonical and make the two role-specific commands canonical everywhere.

Choose the option with the least new parsing/state/API and clearest compatibility behavior. No maintainer approval is needed unless the proposed shape unexpectedly changes core protocol semantics or introduces a value/policy decision.

Minimum tests: the canonical surface must route to the intended existing role behavior; missing/unknown role must fail before persistent identity side effects; help/human capabilities/JSON capabilities/dispatch must describe the same canonical-vs-alias contract. Do not add a new command-registry framework unless the minimal repair genuinely needs it.

## Local-CI-first rule

For coherent READY_LOCAL implementation/test commits, do not poll or wait for GitHub Actions. Validate the **exact pushed developer SHA** in a clean temporary worktree/clone.

Minimum default gate:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # must be empty
```

When the result anchors release-facing facts, retain exact SHA, commands, distinct UTC start/end timestamps, exits, host/OS/arch, stable Rust version, and initial/final clean-tree state. Label developer-local CI, persisted local provenance, reviewer-executed checks, and GitHub-hosted CI separately.

The identity ordering/test slice and canonical-failover CLI dispatch slice do not touch wire decoder/parser/crypto framing, so this handoff introduces no extra fuzz requirement. Existing hosted fuzz success is supplemental only.

## Rolling queue — execute continuously in dependency order

There is one unresolved HIGH. Execute A -> B -> C -> D continuously. Then execute E -> F -> G -> H -> I without waiting for another reviewer cycle when dependencies remain satisfied. J is conditional live work; K remains policy-blocked. Do not enter watcher/polling mode merely because the reviewer interval has not arrived.

### A. HIGH / READY_LOCAL — add the missing cross-command identity/config regression matrix

**Goal:** prove the exact `3164030` ordering behavior across every changed command family, and repair only if a focused test exposes a real defect.

**Files/concepts:** `crates/neko-cli/tests/probe.rs`; `crates/neko-cli/src/main.rs` only if a test fails for a real ordering reason.

**Protected invariants:** deterministic local parse failures create no persistent identity; valid paths still auto-generate identities; actual bind/connect/reachability remains a later runtime boundary; no Session/Carrier/ACK/crypto/wire change.

**Tests:** use the matrix in the HIGH finding. Prefer one table-driven process test over many near-duplicates.

**Commit/push:** one coherent developer-owned test/repair commit. Continue immediately to B.

### B. READY_LOCAL — exact-tree local gate and minimal packaged operator smoke

On the exact pushed A SHA, in a clean temporary checkout, run:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain
```

Persist minimum provenance with distinct UTC start/end, host/OS/arch, stable Rust, exact exits and initial/final clean state. Do not wait for Actions.

Because this is release-facing CLI behavior, build one native x86_64 package outside the source tree and smoke only materially relevant facts:

- restrictive-umask keygen still creates exact `0600` and stable reload;
- one ordinary wrong-length peer key leaves no identity;
- one failover or endpoint-rebind wrong-length/malformed deterministic invocation leaves no identity.

Do not create another generic package checker. Continue immediately to C.

### C. READY_LOCAL / FACTUAL RECONCILIATION — repair exact-tree claims

Refresh only affected facts in:

- `docs/release-engineering.md`;
- `docs/release-security-review-packet.md`;
- `docs/reviews/release-item4-subgates-20260909.md`;
- the local identity/config evidence note.

The header/scope and `Exact-tree gates` paragraph must point to the same real tested developer SHA from A/B. Cite the persisted local evidence. Keep hosted run `34345047449` separate if mentioned. Do not describe exact `3164030` as developer-local CI unless an actual exact-316 local record is retained; normally the A test commit will supersede it as the tested tree.

Preserve `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, item-3 incompleteness, independent-review absence, D019 blockage, and all WAN/performance boundaries.

Continue immediately to D.

### D. BOUNDED REVIEW CLOSURE — close the identity/config lane

Perform exactly one final bounded inspection of all `load_or_generate` call sites and their immediately preceding deterministic argument parsing. Check only:

- peer/address/bind validation after secret creation;
- raw remote-key parsing bypassing the exact-32-byte helper;
- secret material in output/logging;
- identity mutation before a deterministic error;
- readiness announced before configuration/identity prerequisites.

If A/B pass and no concrete defect remains, explicitly close the identity/config lane. Do **not** expand into parent-directory policy, hard-link frameworks, generic in-memory secret hardening, or another checker suite.

Continue immediately to E.

### E. READY_LOCAL / PROPOSAL — choose the canonical `failover` CLI repair

Record a short 1–3-option proposal in a developer-owned implementation note or commit message. State canonical shape, legacy alias behavior, side-effect ordering, and minimum tests. Autonomously select the smallest fail-closed option; do not wait for reviewer approval.

Continue immediately to F.

### F. READY_LOCAL / VISIBLE OUTPUT — make canonical failover CLI truthful and usable

Implement the selected E shape by reusing existing `failover_server` / `failover_client` behavior rather than forking the runtime. Preserve all current failover, negotiation, authentication, resume, accounting and diagnostic semantics.

Tests must cover canonical routing, legacy compatibility (if retained), missing/invalid role failure before identity creation, and help/human/JSON capability consistency.

Commit/push one coherent slice. Continue immediately to G.

### G. READY_LOCAL — exact-tree local gate for F

Run the normal exact-tree local gate on the pushed F SHA. No extra fuzz unless the implementation unexpectedly touches wire/parser/crypto framing. Persist provenance only when used to support release-facing facts.

Continue immediately to H.

### H. READY_LOCAL / MILESTONE RECONCILIATION — record the visible output without claim inflation

Update CLI/release/status documentation only if F materially changes an operator contract or release-facing fact. Do not change release/governance flags. Do not turn a local CLI repair into WAN, failover-performance, interoperability or security approval.

Continue immediately to I.

### I. LOCAL OUTPUT SELECTION — keep a real queue, not a watcher

Re-read exact-current status/plan/release packet and implementation. If another named dependency-ready local output exists, take it. Otherwise propose 1–3 concrete runtime/operator/release-correctness gaps grounded in current code/evidence, state owner/invariant/minimum tests, and autonomously choose the smallest non-policy option.

Prefer visible runtime/operator/release behavior over checker/schema/document-only refinement. Do not reopen package archive variants, generic DeliveryLedger auditing, previous-release compatibility, FEC/0-RTT/striping/multipath/exotic carriers, or generic evidence-schema work without a newly observed problem.

### J. CONDITIONAL VPS OUTPUT — only when a genuinely new live question opens

Current repository truth remains `READY_LIVE: none`. Standing authorization remains valid, but authorization alone is not a reason to rerun answered/frozen rows.

Only execute live work if exact-current code/evidence opens a specific unresolved real-network question with materially changed code/config/instrumentation/path/hypothesis and satisfied dependencies. Otherwise do not unchanged-rerun HY2, repeated warm failover, periodic, installed-package lifecycle, distinct A->B->A, generic soak, IPv6 without a real owned IPv6 path, or live PMTUD before its separate authenticated wire/security design gate.

### K. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Bounded non-policy engineering controls remain available for review, but terminal source-retention/no-reset semantics still require maintainer/security-policy judgment. Do not invent TTL, LRU/history capacity, external authority or weaker reset semantics. This does not block A-J.

## Governance boundaries

- `IMPLEMENTATION_COMPLETE=true` remains bounded research-implementation status only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain authoritative.
- release item 3 remains incomplete; current opportunity classification remains `READY_LIVE: none` for remaining unimplemented capabilities.
- item 4 remains independent-review incomplete; developer-prepared factual support is not an audit/security approval.
- canonical corpus freeze remains corpus-specific and does not freeze the global protocol.
- standing VPS authorization remains valid, but no dependency-ready live row currently exists.
- D019 remains a maintainer/security-policy checkpoint and must not be silently invented by the coding agent.
