# Independent bounded I4-CLI-M machine-readable exit / JSON review — exact `1b1b8c8`

Bounded independent static challenge of the exact-current machine-readable CLI/process evidence surface at reachable `1b1b8c8ff5e158e1b63bba99a1969da1069c700d`. This review reads the current owners and deterministic regressions; it does **not** claim reviewer-local test execution, hosted CI, WAN evidence, performance suitability, human-output review, signing/release policy, or production approval.

## Owners inspected

- `crates/neko-cli/src/main.rs`
- `crates/neko-cli/src/reachability.rs`
- `crates/neko-cli/tests/probe.rs`
- `scripts/bench/process-resource-sampler.py`
- `scripts/bench/process-resource-sampler-test.py`
- `scripts/bench/process-resource.schema.json`
- `scripts/bench/validate-process-resource.py`
- `scripts/bench/run-periodic-command.py`
- `scripts/bench/run-periodic-command-test.py`
- `scripts/bench/run-repeated-warm-failover.py`
- `scripts/bench/run-repeated-warm-failover-command.py`
- `scripts/bench/run-repeated-warm-failover-command-test.py`

Applicable current claims were cross-checked against `README.md`, `AGENTS.md`, `IMPLEMENTATION_PLAN.md`, `docs/status.md`, `docs/release-security-review-packet.md`, and the current handoff. Release items 3/4 remain incomplete; this note does not change release flags.

## Challenged invariants and result

### Matrix probe machine result — no finding

`probe --matrix ... --json` emits the single `reachability::run` artifact to stdout and exits 0 only when its one bounded case is `reachable=true`; a reachable-false/validation/network outcome remains a complete JSON artifact and exits 1. The current `reachability-matrix.v1` producer has exactly one case and one semantic `reachable` field, so the current string gate cannot be confused by another same-shaped field. Argument-shape errors still fail closed through the CLI usage/error path rather than pretending to be a completed matrix result.

The gate is schema-coupled (`artifact.contains("\"reachable\":true")`), so a future multi-case/schema change must re-challenge it; that is not a defect in the exact-current one-case schema.

### Authenticated probe / lab machine result — no finding

The ordinary authenticated probe's `--json` success path emits its result only after the bounded authenticated exchange/echo has completed. Runtime/configuration failures use nonzero fail-closed exits and stderr rather than manufacturing a success JSON object. The success-only probe JSON contract is therefore not being used as a typed failure artifact.

The bounded reliable-UDP lab builds a terminal `ok` predicate from the committed counters plus cleanup truth, emits that exact `ok` value in JSON mode, and exits 1 when `ok` is false. A typed negative therefore cannot coexist with wrapper success on this owner.

Failover diagnostics are a separate diagnostic stream (`--diagnostic`/experiment-id) rather than the single-document probe/matrix `--json` contract. Current failover completion/failure tests assert process exit and typed diagnostic transitions together; this review does not promote the undocumented `--json` token into a new failover public interface.

### Periodic wrapper — no finding

`run-periodic-command.py` derives one terminal `result.status` only after readiness, process exits, bounded log parsing/accounting, and local/remote cleanup postchecks. It returns 0 only for `status == "passed"`; timeout, malformed result, log overflow, startup failure, endpoint failure, or cleanup failure remain nonzero. Plan/argument failures are separately exit 2 and stderr. Thus a completed machine result cannot say failed/cleanup_failed while the wrapper exits success.

### Repeated warm-failover wrapper — no finding

`run-repeated-warm-failover.py` writes the batch atomically with `status = passed` only after exactly six valid passed cycle rows and no first failure, and returns 0 only for that status. Collector nonzero/invalid JSON/schema evidence becomes a typed `invalid_cycle_evidence` failure and stops the batch. `run-repeated-warm-failover-command.py` propagates the real runner return code for non-preflight runs; its preflight returns 0 only when the synthetic runner itself succeeded and all six dispatches completed.

### Process-resource sampler / validator — no finding after H-R9-082/083/084

The sampler preserves the child exit fact inside `result.exit`, but its own wrapper returns nonzero whenever terminal cleanup is not affirmatively complete. `owned_sockets_after_exit=None` therefore cannot coexist with sampler success. The focused regression covers partial `/proc/net` observation and escaped TCP/UDP owners as cleanup-unknown/nonzero outcomes. The validator independently accepts only `process_reaped=true`, `process_group_empty=true`, `owned_sockets_after_exit=0`, `complete=true` with the exact sampler-created process-group scope.

This keeps child outcome, cleanup truth, sampler exit, and validator acceptance as distinct but mutually consistent facts; developer-local provenance and hosted/live evidence remain separately classified.

## Exclusions / residual gates

- Human wording, cat markers and stderr presentation are deliberately left to I4-CLI-H.
- No capacity/adversarial-load suitability decision is made here; I4-BND remains separate, and `SessionRuntime.events` capacity remains a maintainer/security policy gate.
- No new live-network question was created: `READY_LIVE: none`.
- No decoder/parser/crypto-framing owner changed in this review-only slice; no fuzz claim is made.

## Conclusion

No concrete BLOCKER/HIGH or source-decided repair was found in this bounded I4-CLI-M scope on exact reachable `1b1b8c8`. Persist this as item-4 review support and continue immediately to the independent I4-CLI-H human-output/stderr challenge; do not treat this no-finding as item-4 completion or release approval.
