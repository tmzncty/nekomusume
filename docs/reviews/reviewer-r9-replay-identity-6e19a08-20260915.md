# R9 reviewer checkpoint — exact `6e19a08`

**Reviewed tree:** `6e19a08f2e7e2fc9b747810b1c70cbf01162ed31` (reachable `main` at review time).

**Scope:** the H-R9-019 TCP replay-identity repair and the dedicated migration-back P2 process fixture. This is a bounded reviewer checkpoint, not release/security approval and not WAN evidence.

## Verdict

### Controlled replay identity: ACCEPT

The controlled/non-automatic client path now replays from `uncertain_start` rather than from positional index 1. For the intended `count=4`, `--reliable-udp`, `--migration-back` ownership partition, the replay set is therefore the genuine uncertain record at offset 32; reliable-owned offsets 0/16 and the reserved post-return offset 48 are excluded.

This closes the concrete controlled-path H-R9-019 mis-selection found at the prior tree. Do not revert the `uncertain_start` selection.

### Automatic-health exact-identity contract: still review-support work

The automatic-health branch still reduces `failover.tcp_resend()` to `.len()` and reconstructs a positional slice from `uncertain_start`. On the current CLI path the tracked uncertain set is contiguous and no pre-replay confirmation mutates it, so this review did not reproduce a current automatic-mode mis-selection at exact `6e19a08`. However, the implementation still does not consume the exact `(DataId, payload)` ownership returned by `tcp_resend()` and the focused exact-identity regression requested by the prior handoff is still absent. Keep that as R9 integration review-support before R9-12; do not treat equal cardinality as a general ownership proof.

### Positive migration-back P2: OPEN / READY_LOCAL

Changing the dedicated fixture to `count=4` makes the desired ownership shape satisfiable, but the test is still intentionally vacuous as positive evidence:

- it discards `server_status` and `server_log`;
- it does not require client success;
- it accepts a run that never reaches `r9_udp_post_return_sent`;
- all post-return Session/Carrier settlement assertions remain inside `if client_log.contains("r9_udp_post_return_sent")`;
- current runtime emits no dedicated `r9_udp_post_return_settled` terminal event proving both logical confirmation and Recovery `in_flight==0`.

Therefore P2 is not closed by a green test at this tree. The next dependency-ready action is to make the positive P2 non-vacuous and classify the first missing structured milestone if it fails.

## Hosted cross-evidence

GitHub-hosted `stable checks` and `nightly decode fuzz smoke` both completed successfully on exact `6e19a08`. These are cross-evidence only; the R9-2 closure still requires developer-local clean exact-tree provenance on the final pushed source/test SHA.

## Required continuation

1. Make P2 require successful client/server termination and exact structured milestones for offset 32 TCP replay and offset 48 post-return reliable ownership.
2. Add `r9_udp_post_return_settled` only after exact Session confirmation and Carrier recovery settlement both complete; require it in P2.
3. Exercise both post-return acknowledgement-domain arrival orders without sleeps/new deadlines.
4. Add the persistent malformed-budget P3 and incomplete-settlement P4 negatives.
5. Persist the clean exact-tree local gate, then continue R9-3 through R9-12 and Q10/Q11/Q12 without waiting for reviewer cadence.

`READY_LIVE` remains none: the current blocker is local implementation/evidence closure plus independent review, not standing authorization.