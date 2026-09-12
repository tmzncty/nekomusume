# ChatGPT reviewer handoff — resume local repair and item-4 review queue

## Reviewed repository truth

- Default branch: `main`.
- Exact reviewer HEAD before this refresh: `f6c79b6deeed57782febb8208b0bf67a050b4d4c` (`docs(handoff): escalate persistent multistream execution stall`).
- Previous reviewed developer-owned head remains exact `e96b9c1be84dbafb78010dca30cb7e3d5f5ada1a` (`docs: index independent item-4 checkpoint`).
- No developer-owned commit has landed after that checkpoint; the current source still contains the queued test-only defect in `crates/neko-cli/tests/multistream.rs`.
- The maintainer has explicitly resumed execution after a real model/tool-budget interruption. Do **not** treat that historical runtime-budget stop as an ongoing blocker. This is an execution-availability fact only; it does not change release/security/network authorization.
- Repository release truth remains unchanged: item 3 incomplete, item 4 incomplete, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, D019 policy-blocked, and `READY_LIVE: none`.
- `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, `docs/standing-vps-lab-authorization.md`, `docs/vps-rental-window-priority.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, and `docs/release-security-review-packet.md` were re-read for this refresh. No architecture or authorization expansion is introduced here.

## Reviewer verdict

The prior claim that all local work was exhausted is stale for the current branch. There is an exact-current `READY_LOCAL` test defect, and after that repair the still-open independent release/security gate supports additional **bounded local review/repair work** that does not invent policy or require WAN execution.

This queue is deliberately not a new feature backlog. It is a release/security review-support queue. A slice may produce either:

1. a concrete defect with existing semantics -> smallest repair + regression + exact-tree closure; or
2. a bounded no-finding review note with precise scope/boundaries.

Do not manufacture implementation just to keep the queue busy.

## LOCAL 1 — MUST_EXECUTE: terminal EOF/RST test repair

Owner: `crates/neko-cli/tests/multistream.rs`

Test: `executable_rejects_unsupported_only_negotiation_before_noise_or_data`

Current exact-current code still does:

```rust
assert_eq!(
    socket.read(&mut byte).unwrap(),
    0,
    "server entered Noise/data admission"
);
```

Replace only this terminal-close assertion with equivalent semantics:

```rust
match socket.read(&mut byte) {
    Ok(0) => {}
    Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => {}
    Ok(n) => panic!("server emitted {n} byte(s) after unsupported negotiation"),
    Err(error) => panic!("unexpected terminal-close error: {error}"),
}
```

Protected invariant: an unsupported-only syntactically valid negotiation must terminate before any Noise/Session data is emitted. TCP may surface fail-closed termination as orderly EOF or reset. Bytes after rejection remain a hard failure.

Hard bounds:

- accept only `Ok(0)` or `ErrorKind::ConnectionReset`;
- `Ok(n > 0)` remains hard failure;
- all other I/O errors remain hard failure absent an already-existing independent contract;
- keep `assert_uniform_handshake_rejection`, exact `neko: handshake rejected\n`, no success JSON and no record output;
- do not modify runtime code to force FIN;
- do not weaken unsupported-version rejection;
- do not create a socket-close framework/helper abstraction solely for this seam;
- no fuzz required because this slice is test-only and changes no wire/parser/crypto/framing behavior.

### Focused validation

Run a bounded timing-variation repetition and the whole target:

```bash
for i in $(seq 1 20); do
  cargo test -p neko-cli --test multistream \
    executable_rejects_unsupported_only_negotiation_before_noise_or_data -- --exact
done
cargo test -p neko-cli --test multistream
```

If Cargo exact-filter syntax needs a mechanical adjustment, adjust the command only.

Commit and push the coherent test repair, then validate the **exact committed developer SHA** in a clean checkout/worktree:

```bash
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --short
```

Persist minimal developer-local provenance: exact SHA, commands, UTC start/end, exit codes, OS/arch, stable Rust version, clean-tree state. Do not persist credentials, private addresses/topology, or unnecessary absolute paths.

If any gate fails, repair the actual failure and rerun. GitHub-hosted CI is optional cross-evidence and never a wait condition.

**When green, immediately continue to LOCAL 2; do not wait for reviewer cadence.**

## LOCAL 2 — cross-platform negative/process-test determinism sweep

Goal: use the concrete EOF/RST failure as a trigger for one bounded cross-platform test-semantics review, not a general refactor.

Primary surface:

- `crates/neko-cli/tests/*.rs`
- directly related CLI process-test helpers only when needed to understand a finding.

Look specifically for tests that accidentally encode an OS-specific transport/process artifact instead of the actual repository contract, including:

- TCP orderly EOF vs reset when both are equivalent fail-closed outcomes;
- timing/startup assumptions that turn a valid terminal state into a flaky failure;
- assertions that require an incidental OS error code where the contract is broader but still fail-closed;
- platform-specific filesystem/permission assumptions lacking an appropriate `cfg` boundary.

Acceptance criteria for a repair:

- name the exact test/helper and the real semantic invariant;
- prove the current assertion is narrower/wrong on a supported environment;
- preserve or strengthen the negative/security invariant;
- add/adjust a bounded regression only; no generalized test framework.

Do **not** weaken errors merely to make CI green. If no additional concrete defect exists after this bounded sweep, record no finding and move on.

Any code/test change -> focused target tests -> commit/push -> clean exact-tree `scripts/check.sh` + `git diff --check` + provenance before continuing.

## REVIEW 3 — cryptographic API-misuse/security-invariant challenge

This is independent review support for item 4, not cryptanalysis and not a new cryptographic design.

Primary owners:

- `crates/neko-crypto/src/lib.rs`
- directly relevant crypto tests/fixtures;
- `docs/research/security-threat-model.md` and current ADR/spec references only as needed to resolve existing intended semantics.

Challenge current implementation against already-stated repository invariants:

- nonce uniqueness / checked overflow refusal;
- direction / epoch / key-phase separation;
- transcript / prologue / record-context binding;
- replay / duplicate / old-epoch rejection;
- authorization/trust status enforcement before protected data admission;
- key-update synchronization boundaries already implemented;
- secret-safe error/log behavior.

Hard bounds:

- do not invent a new handshake, cipher, KDF, key-store, persistent replay store, TTL/LRU/history-size, or retention policy;
- D019 remains policy-blocked;
- no claim of cryptanalysis/security approval;
- if a concrete API misuse or invariant violation exists and current semantics already determine the answer, repair the smallest owner path and add positive/negative regression;
- if no concrete defect is found, write a bounded review note that states what was inspected and what remains outside scope.

If any change touches parser/framing/wire decoder behavior, use the repository pinned fuzz tooling per current policy; otherwise ordinary crypto unit/integration + full local gate is sufficient.

## REVIEW 4 — wire/parser fail-closed and allocation-bound challenge

Primary owners:

- `crates/neko-wire/src/lib.rs`
- direct decode call paths that transform untrusted network bytes into typed state.

Review only current advertised semantics:

- length/count bounds before allocation;
- integer overflow/truncation handling;
- malformed/truncated/trailing input;
- unknown version/frame/type behavior;
- no panic on attacker-controlled decode input;
- no unbounded allocation driven by a decoded length/count;
- stable deterministic encoding where the current contract requires it.

Do not redesign the wire format or change canonical meaning. If a defect is found, smallest repair + regression + required decode fuzz smoke using the pinned toolchain, then exact-tree full local gate. If none is found, record a bounded no-finding note and continue.

## REVIEW 5 — CLI secret/output and pre-auth failure-surface challenge

Primary owners:

- `crates/neko-cli/src/main.rs`
- `crates/neko-cli/src/preauth.rs`
- directly related process/integration tests.

Security invariants are already stated in `SECURITY.md`: logs must not expose PSK/private key/plaintext payload; unauthenticated control input must not mutate admitted connection state; experimental service must not become an arbitrary open proxy.

Perform a bounded source/test review for concrete contradictions such as:

- private key/secret/plaintext payload included in stderr, structured events, panic/debug output, or persisted evidence;
- distinguishable pre-auth error output that contradicts an existing uniform-rejection contract;
- output emitted as success before authentication/admission actually succeeds;
- unauthenticated input causing durable/session-admission state transition through an existing call path.

Do not invent new log redaction infrastructure or a new policy framework absent a concrete leak/contradiction. Concrete existing-semantics defect -> smallest fix + regression + exact-tree closure. No finding -> bounded note only.

## REVIEW 6 — Session/Carrier evidence-domain separation challenge

Primary owners:

- `crates/neko-session/src/lib.rs`
- `crates/neko-carrier/src/lib.rs`
- `docs/specs/nekomusume-session-v0.md`
- directly relevant failover/session tests.

This is not a redesign. Challenge only already-fixed invariants:

- `confirm_received` is Session delivery evidence, not application `delivered/effect` evidence;
- path/packet feedback does not silently promote Session delivery;
- `Unsent -> InFlight` preserves assignment-time context;
- advanced-state novel-byte overlap remains fail-closed;
- TCP reliability does not substitute for Session delivery state;
- carrier migration/failover does not create delivery proof by itself.

If current code violates an already-stated invariant, smallest repair + regression. If not, record bounded no-finding scope. Do not change ACK architecture, Carrier ownership model, or wire semantics without a new reviewer/maintainer decision.

## CHECKPOINT 7 — item-4 factual reconciliation

After LOCAL 1-2 and REVIEW 3-6:

1. Update `docs/release-security-review-packet.md` only for actually established new facts/findings.
2. Update item-4 factual support/status only if status wording truly changes.
3. Distinguish:
   - developer-local exact-tree CI;
   - any GitHub-hosted cross-evidence;
   - independent/bounded review notes;
   - live WAN evidence;
   - performance conclusions.
4. Do not mark item 4 complete merely because these bounded reviews found no defect.
5. Do not change RC/production/freeze/release flags automatically.

Run relevant release/status/link checks and final exact-tree `scripts/check.sh + git diff --check` for any coherent docs closure commit. Reviewer-only docs commits later do not retroactively become the tested implementation tree.

## Queue-exhaustion / escalation boundary

After the above queue, a genuinely exhausted local queue is valid. Remaining known classes are allowed to stay blocked:

- **D019:** source-retention/no-reset policy/value decision;
- **RSEC-001:** representative adversarial-load/capacity-suitability evidence when test pressure/capacity choices require maintainer judgment or exceed standing limits;
- **item 3:** natural-loss/remaining environment evidence, IPv6 environment block, and frozen same-class negative live lines absent a materially new hypothesis;
- **signing / key custody / SBOM:** release-policy decisions, not automatic local coding filler;
- **previous frozen release:** unavailable by repository fact until one exists;
- **release/production authority:** explicit maintainer/independent decision.

If a review discovers a new concrete BLOCKER/HIGH correctness/security/evidence defect, stop expansion and put its repair at queue head. If a newly discovered issue requires core Session/Carrier/ACK/crypto/wire architecture change, destructive/canonical migration, new security-policy numbers, action outside standing authorization, production/third-party access, or new credentials/server permission, escalate instead of inventing an answer.

## Live/release boundary

- `IMPLEMENTATION_COMPLETE=true` only in the repository's bounded research/governance sense;
- item 3 remains incomplete;
- item 4 remains incomplete;
- `RELEASE_CANDIDATE=false`;
- `PRODUCTION_READY=false`;
- `FREEZE=false`;
- `RELEASED=false`;
- D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`;
- current `READY_LIVE: none` remains authoritative.

Do not rerun unchanged HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint/source migration, key update, IPv6, PLPMTUD, or Experimental Track work merely because the VPS exists. Standing authorization remains valid if a materially new dependency-ready self-owned TCP/UDP question emerges later.

## Continuous execution order

1. **MUST_EXECUTE_LOCAL:** EOF/RST multistream test repair.
2. **FOCUSED + EXACT-TREE CLOSURE:** repeated target test -> multistream target -> commit/push -> clean exact-tree full local gate -> provenance.
3. **LOCAL REVIEW/REPAIR:** bounded cross-platform negative/process-test determinism sweep.
4. **ITEM-4 SECURITY REVIEW SUPPORT:** crypto API-misuse/invariant challenge.
5. **ITEM-4 SECURITY REVIEW SUPPORT:** wire/parser fail-closed/allocation-bound challenge.
6. **ITEM-4 SECURITY REVIEW SUPPORT:** CLI secret/output + pre-auth failure-surface challenge.
7. **ITEM-4 SECURITY REVIEW SUPPORT:** Session/Carrier evidence-domain separation challenge.
8. **RECONCILE:** release packet/item-4 factual update only for established facts.
9. **STOP OR ESCALATE:** if no further concrete local defect exists, record genuine local queue exhaustion and leave policy/environment/release-authority gates open.

Complete coherent slice -> required local gate -> commit + push -> continue immediately to the next dependency-ready slice. Do not wait for the hourly reviewer merely because one slice finished.
