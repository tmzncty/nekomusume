# Independent R9-7 process/result-truth review — exact `788a4dc`

**Finding:** `H-R9-067` — HIGH, acceptance/evidence-oracle correctness. This finding does **not** establish a new runtime defect in the current source; it establishes that the deterministic process regression used to prove H-R9-062's terminal result is weaker than the claim and would remain green under a material success/failure regression.

**Exact developer source/test anchor reviewed:** `788a4dcbb4ab802f1444991d7898ec48ed7a9030`, reachable from `main` at review time.

## Challenged claim

For an injected post-admission socket error on the real reliable-UDP post-return first-send owner:

- the packet reservation must be rolled back;
- no positive sent/PTO/retransmit/settlement evidence may be fabricated for that failed copy;
- the bounded operation must terminate nonzero rather than proceed to final success/accounting/summary output.

The production source currently follows that intent: `r9_udp_post_return_send_failed` follows `abandon_sent`, the dual-settlement loop cannot satisfy Session completion, and the bounded deadline reaches `fail("post-return reliable-UDP dual settlement timeout")` before the final success continuation.

## Concrete oracle gap

`crates/neko-cli/tests/probe.rs::reliable_udp_first_send_socket_failure_rolls_back` correctly checks the rollback-domain evidence:

- `r9_udp_post_return_send_failed` exists;
- `r9_udp_post_return_sent` does not;
- `r9_udp_post_return_settled` does not;
- no PTO/retransmit is emitted for the aborted copy;
- a residual row, when present, reports zero Recovery in-flight.

But the test does **not** directly require the client process to exit nonzero and does **not** forbid the later success continuation (`failover_client_ok`, final `summary`, or `ordered_records_complete`). Its terminal assertion is effectively:

```text
residual exists OR stdout contains "acknowledgement" OR process is nonzero
```

Therefore a regression that emits the existing residual/classification evidence and then erroneously returns success/final summary could still satisfy this test. The test comment says the run is "not success", but the executable oracle does not prove that claim.

This matters for R9-7 because process/result truth is an independent evidence domain: correct Recovery rollback is insufficient if the CLI can subsequently classify the bounded operation as successful. A release-item-4 process oracle must turn red if the production owner regresses from terminal failure to final success.

## Required repair / closure contract

Use the smallest deterministic repair; a tests-only repair is sufficient if current source behavior remains as reviewed.

1. Keep the existing **production** `--fail-r9-first-send` injection path. Do not replace it with a helper-only/unit-only simulation.
2. In `reliable_udp_first_send_socket_failure_rolls_back`, explicitly require a nonzero client exit. If current CLI exit-code contract is already intentionally pinned to `fail()`'s code 2 in this surface, asserting code 2 is acceptable; otherwise `!status.success()` is the minimum required discriminator and avoids inventing a new policy.
3. Explicitly forbid final success/result evidence after the injected socket failure: at minimum no `failover_client_ok`, no final client `summary`, no `ordered_records_complete`, and no `r9_udp_post_return_settled`.
4. Keep the existing domain checks: exactly the typed send-failed classification for the failed first-send owner, no positive sent evidence for that packet, no PTO/retransmit for the aborted copy, and zero Recovery in-flight in residual evidence when emitted.
5. Preserve a paired positive control on the ordinary production path showing successful first-send -> dual Session/Carrier settlement -> final success. Existing positive migration-back/post-return process tests may serve this role if they exercise the same owner and are named in the closure note.
6. Do not weaken the controlled `--drop-r9-data` loss seam: it remains a Recovery-owned loss that legitimately drives PTO/retransmission and may complete successfully.
7. No protocol/wire/crypto/Session/Carrier architecture change is authorized by this finding. Do not add a result-schema framework or checker merely to close the test.
8. On the final pushed repair SHA run and persist the normal exact-tree local gate:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust version and clean-tree state. No fuzz is mechanically required for a tests-only/process-oracle repair.

## Evidence boundary

The exact `788a4dc` developer-local gate and hosted Rust CI run `35410930506` are valid cross/current-tree evidence, but they cannot fill this oracle gap because the test itself does not discriminate erroneous final success. Reviewer-local execution is not claimed.

**R9-7 disposition:** BLOCKED on H-R9-067. Repair this front HIGH, then continue immediately through the remaining R9-7 diagnostic/result-domain challenge and the existing deep queue without waiting for the next reviewer cadence.
