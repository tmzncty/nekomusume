# Independent bounded R9-11D1 review — executable result truth / terminal classification

**Source/test anchor reviewed:** `8cbd9af39a600f4ddd5fbc50208791e8584c6d74` (reachable from `main`; current navigation before this note was `e28999a2abb89d45db227becaf07081700c7faf7`).

**Finding:** no concrete defect found in this bounded D1 slice.

## Owners inspected

- `crates/neko-cli/src/main.rs`: failover-client / failover-server automatic-health failover and migration-back terminal owners; `fail()`; timeout/handshake diagnostics; post-return reliable-UDP settlement/failure path; final `failover_client_ok` emission.
- `crates/neko-cli/tests/probe.rs`: automatic-health/migration-back success path and negative process fixtures, including migration-back tamper, reliable-UDP post-return incomplete/malformed/socket-failure terminal cases, future-ACK negative, and failover handshake timeout stage classification.
- `scripts/bench/process-resource-sampler.py`, `scripts/bench/process-resource-sampler-test.py`, `scripts/bench/validate-process-resource.py`, and `scripts/bench/process-resource.schema.json`: exact H-R9-084 terminal cleanup/result-truth repair boundary.
- Applicable current release/spec/status claims and `docs/CHATGPT_HANDOFF.md` D1 contract.

## Invariants challenged

1. A terminal failure, timeout, stop, or incomplete post-return settlement must not later become command success because a later cleanup/read/child action succeeds.
2. Process exit status and emitted terminal evidence must agree on the authoritative outcome; a successful final marker must be unreachable after a terminal failure.
3. Unknown/incomplete cleanup must remain fail-closed rather than being promoted to success.
4. Negative fixtures must prove the intended failure phase was reached rather than pass because setup failed earlier.
5. Late post-return work must not manufacture delivery/switch/readiness success after terminal failure.

## Bounded challenge result

- Automatic-health/migration-back success emits final success only after the failover/replay/migration-back/post-return path completes. The negative post-return fixtures assert non-zero exit and absence of final success/settlement evidence when first-send socket failure, malformed-bound exhaustion, missing logical confirmation, or migration-back validation failure terminates the path.
- The migration-back tamper and reliable-UDP terminal negatives are phase-sensitive: their diagnostics/error assertions bind failure to the intended recovery/post-return phase rather than a generic setup failure.
- Failover handshake timeout is non-zero and carries deterministic last-success-stage diagnostics; it does not emit a successful terminal marker.
- H-R9-084 now makes the process-resource sampler wrapper non-zero whenever terminal cleanup is not affirmatively complete while preserving the measured child exit fact in structured output. The positive cleanup-complete control remains success; child failure/timeout/interruption cannot be laundered by cleanup.
- No examined path showed a later zero-return cleanup/read action overwriting a prior terminal failure, and no terminal negative reaches `failover_client_ok` / ordered completion / post-return settlement.

## Evidence boundary

This is an independent source/test review on the exact reachable tree, not a claim of reviewer-local execution. No reviewer-local tests, hosted CI, WAN run, performance conclusion, or production/release approval is claimed here. The developer-reported clean exact-tree gate for `8cbd9af39a600f4ddd5fbc50208791e8584c6d74` remains developer-local provenance only.

Excluded from this slice and intentionally left to the following lanes: independent TCP/UDP socket ownership/rebind proof (D2), descendant/process-group/temp-runtime cleanup (D3), shutdown/late-completion/stale-owner false-success negatives (D4), live WAN behavior, capacity/adversarial-load policy, D019, core Session/Carrier/ACK/crypto/wire redesign, release authority, and policy/value choices.

**R9-11D1 disposition:** CLOSED / bounded no-finding support for release item 4. Continue immediately to R9-11D2; this note is not R9 or release closure.
