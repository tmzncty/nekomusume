# Independent review — H-R9-076 resumed-session gap-ACK regression

**Classification:** HIGH correctness/integration + exact-tree gate blocker

**Reviewed implementation anchor:** `52017b5adb26f377857adad2c9a6ed945e539d10`

**Scope:** H-R9-075 repair in `SessionRuntime::delivery_ack`, the existing resumed-session integration fixture, and the executable R9 UDP ACK-ordering owner. This note does not review R9-10B replay source-of-truth yet and makes no WAN/release/performance claim.

## Finding

H-R9-075's source-side fail-closed guard is directionally correct: `SessionRuntime::delivery_ack` now rejects `offset > confirmed_watermark` before in-flight/window/event mutation. The focused unit regression proves the future range cannot advance the watermark and then proves ordered ACKs can advance it.

However, the final pushed tree is not a closed repair. The repository-wide stable gate fails in the existing integration test `crates/neko-carrier/tests/resumed_session.rs::bounded_udp_blackhole_tcp_resume_preserves_order_and_exactly_once_bytes`.

That fixture still encodes the old, now-forbidden cumulative-gap assumption:

1. it queues three send ranges into `receiver_runtime`, so the confirmed watermark starts at 0;
2. the first UDP record at offset 0 is received but no Session delivery proof is applied;
3. after TCP resume it iterates `ids.iter().skip(1)`, first receiving the later record at offset 5;
4. it immediately calls `receiver_runtime.delivery_ack(... offset=5 ...)` and then `failover.confirm(id)`;
5. exact `52017b5` now correctly rejects that call with `RuntimeError::Protocol` because `[0,5)` has no applied Session proof.

The hosted Rust CI run `35444056099` confirms this exact contradiction: nightly decode fuzz completed successfully, but the stable `bash scripts/check.sh` job failed at `resumed_session.rs:231` with `called Result::unwrap() on an Err value: Protocol`. Hosted CI remains supplemental evidence, but this deterministic failure is enough to reject closure; no clean exact-tree developer gate/provenance for `52017b5` is present in repository truth.

This is not evidence that the new fail-closed primitive should be weakened. The executable reliable-UDP owner already follows the intended invariant: a later exact Session ACK is buffered until its offset equals the confirmed watermark, and only then is `SessionRuntime::delivery_ack` invoked. The stale integration fixture must be brought into the same ordering/evidence model.

## Smallest repair contract

1. **Keep the H-R9-075 fail-closed guard.** Do not restore cumulative promotion across an unproved gap and do not change ACK/wire architecture.
2. Repair `bounded_udp_blackhole_tcp_resume_preserves_order_and_exactly_once_bytes` so later resumed ACKs are not directly applied across the missing first range. Model the existing executable behavior: retain/buffer later exact proofs without calling `delivery_ack`/`FailoverController::confirm` until the current watermark reaches their offsets.
3. Preserve the fixture's useful adversarial order. A good minimal shape is: receive resumed records for offsets 5 and 13 first; hold their exact ACK proofs; replay/observe the already-delivered offset-0 record; apply the offset-0 Session proof; then drain the held offset-5 and offset-13 proofs in watermark order. `FailoverController::confirm` must follow successful Session proof application, never precede it.
4. Keep the receiver dedup assertion and final application bytes `alpha-bounded-exactly-once`; keep `tcp_resend()` empty only after all three logical identities have valid Session confirmation.
5. Strengthen the H-R9-075 unit negative to assert the full atomic contract already requested by handoff: after the gapped ACK rejection, confirmed watermark, per-stream send-inflight, session send-inflight, and observable event count are unchanged. Because this test is inside the `neko-session` module it can inspect the private accounting directly without adding production API.
6. Keep the positive ordered ACK path green and keep the executable reverse/out-of-order ACK buffering process positive control green.
7. Run the final pushed repair SHA in a safe clean checkout/worktree:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - verify clean tree and persist exact reachable SHA, UTC start/end, OS/arch, stable Rust version, exit codes, clean-tree state.
8. No decoder/parser/crypto framing change is required by this repair; do not mechanically add a local fuzz requirement. Hosted fuzz success at `52017b5` does not substitute for the failed stable gate.

## Exclusions

- no D019/source-retention or capacity-value decision;
- no new Session ACK encoding or cumulative-ACK policy;
- no Carrier/crypto/wire architecture change;
- no live VPS work;
- no release/RC/freeze/production transition.

## Queue consequence

H-R9-075 remains semantically open until this integration regression and the full exact-tree gate are closed. Treat this compatibility/gate failure as **H-R9-076 HIGH** at queue front. R9-10B remains the immediate next dependency-ready review lane after repair: challenge whether executable TCP replay is driven/validated by the authoritative retained `(DataId, bytes)` replay set rather than only its length plus a positional slice of `records`.
