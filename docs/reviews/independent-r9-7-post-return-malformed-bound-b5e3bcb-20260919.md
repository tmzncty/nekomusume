# Independent R9-7 post-return malformed-bound review — exact `b5e3bcb`

**Finding:** `H-R9-068` — HIGH, correctness/security + process/result-truth. The post-return reliable-UDP receive owner does not preserve the helper's fail-closed malformed-feedback bound: terminal receive errors are collapsed into the same continuation used for an ordinary receive timeout, so an operation can cross `MAX_POST_HANDSHAKE_MALFORMED` and still later consume valid feedback and reach success.

**Exact developer source/test anchor reviewed:** `b5e3bcbc63edab6ecf049573ce08a54fe31d6686`, reachable from `main` at review time. Current `main` contains only later review/handoff documentation on top of this source/test tree.

## H-R9-067 closure carried forward

The exact `b5e3bcb` repair closes H-R9-067's narrow terminal-result oracle gap. `reliable_udp_first_send_socket_failure_rolls_back` now directly requires a nonzero client exit and forbids `failover_client_ok`, final client `summary`, `ordered_records_complete`, and `r9_udp_post_return_settled`, while retaining the first-send rollback/PTO/retransmit checks. Existing ordinary post-return success coverage remains the paired production-path control.

Developer-local provenance recorded in the handoff for exact `b5e3bcb` is a clean `scripts/check.sh` + `git diff --check` pass. GitHub-hosted Rust CI run `35413664812` for exact `b5e3bcb` also completed successfully. Hosted CI is cross-evidence only; reviewer-local execution is not claimed in this review.

## Challenged invariant

`recv_udp_delivery_ack` owns a finite operation-wide malformed/unadmitted feedback budget. Once that budget reaches `MAX_POST_HANDSHAKE_MALFORMED`, the helper returns `Err("UDP delivery acknowledgement malformed bound exceeded")`. A receive/socket failure is likewise returned as a distinct terminal error. Only a normal bounded receive timeout is eligible to be non-terminal when the executable owner intentionally wakes for PTO / delayed-ACK progress.

Therefore the executable post-return owner must not collapse:

- ordinary receive timeout;
- malformed-budget exhaustion;
- socket/receive failure

into one continuation class. Crossing the malformed bound must terminate the bounded operation before any later Session/Carrier feedback can restore success.

## Concrete source counterexample

In the post-return dual-settlement loop, the current caller matches the helper result as:

```text
Err(_) if drop_r9_data => emit r9_udp_post_return_recv_timeout; continue
Err(_)                  => emit r9_udp_post_return_residual; continue
```

Both arms discard the actual helper error. The generic branch is documented as a non-terminal R9-4 receive-timeout continuation, but it also receives `UDP delivery acknowledgement malformed bound exceeded` and `UDP delivery acknowledgement receive failed`.

A deterministic adversarial sequence therefore exists on the current source:

1. post-return reliable Data is admitted/sent normally;
2. the authenticated peer sends `MAX_POST_HANDSHAKE_MALFORMED` fresh-envelope, semantically unadmitted Session DeliveryAcks (or other malformed/unadmitted datagrams);
3. `recv_udp_delivery_ack` increments the persistent malformed counter and returns the malformed-bound error on the final one;
4. the caller downgrades that terminal error to `r9_udp_post_return_residual` (or `r9_udp_post_return_recv_timeout` under the controlled-loss seam) and re-enters the loop;
5. a later valid Session DeliveryAck and Carrier ACK can then be consumed, allowing dual settlement and final success despite the operation already crossing its explicit malformed bound.

The helper itself is fail-closed; the executable owner is not. This is a process/result-truth and security-boundary defect, not a request to change the numeric malformed threshold.

## Current test gap

The existing operation-wide malformed-budget process regression covers the initial reliable-UDP receive path and proves that malformed #1/#2, a valid Carrier ACK, then malformed #3 reaches the bound. Current post-return process coverage exercises accepted-empty duplicate Session ACK evidence and ACK-loss/reorder, but does not inject a post-return malformed-bound exhaustion followed by otherwise valid settlement feedback. Therefore the post-return catch-all `Err(_)` can remain green.

## Required repair / closure contract

Use the smallest current-semantics repair.

1. Preserve the existing `recv_udp_delivery_ack` finite malformed budget and threshold exactly; do not change numeric policy.
2. In the post-return executable owner, distinguish the helper's ordinary timeout from terminal errors. Only the explicit `UDP delivery acknowledgement timeout` may follow the current non-terminal PTO/delayed-ACK continuation. `malformed bound exceeded` and receive/socket failure must terminate fail-closed.
3. Under `--drop-r9-data`, do not relabel arbitrary helper errors as `r9_udp_post_return_recv_timeout`; only an actual receive timeout may use that classification.
4. Add a focused deterministic **process-level production-owner** regression. Prefer a bounded server test seam that, after migration-back/post-return Data, sends exactly the existing malformed budget worth of **fresh authenticated but semantically unadmitted** Session DeliveryAcks before otherwise valid Session/Carrier feedback. Re-seal each feedback separately so crypto replay rejection is not the reason for failure.
5. The negative must prove: client exits nonzero; the malformed/unexpected classifications reach the existing bound; no `r9_udp_post_return_settled`; no `failover_client_ok`; no final client `summary`; no `ordered_records_complete`; later valid feedback cannot resurrect the operation.
6. Preserve positive controls: ordinary post-return success and the one-shot ACK-delay/reorder / controlled-data-loss seams may treat an actual receive timeout as intermediate and still converge when valid feedback arrives within their bounded settlement window.
7. Do not redesign Session/Carrier/ACK/crypto/wire architecture, add a new capacity number, or introduce a generic result-schema/checker framework merely for this repair.
8. On the final pushed repair SHA run and persist the normal developer-local exact-tree gate:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust version and clean-tree state. Decoder/parser/crypto framing is not changed by the expected repair, so fuzz is not mechanically required.

## Disposition

**R9-7 remains BLOCKED on H-R9-068.** H-R9-067 is independently closed at exact `b5e3bcb`. Repair H-R9-068 first, then continue immediately through the remaining R9-7 process/result-truth review and the existing deep queue without waiting for reviewer cadence.
