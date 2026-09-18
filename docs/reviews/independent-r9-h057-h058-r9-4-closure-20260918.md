# Independent R9 bounded review — H-R9-057 / H-R9-058

Date: 2026-09-18

## Scope and exact anchor

Reviewed exact reachable source/test tree `c585727d5a8b4bc4958b69cdacbfdcf75226b99f` and the substantive H-R9-056/R9-4 implementation at `458713c478d3d30086e9799aaac568f454b660aa` (plus formatting/style descendants `14dcc09`, `cded9a9`, `c585727`).

Inspected owners:

- `crates/neko-cli/src/main.rs::recv_udp_delivery_ack`
- the post-return server Carrier-ACK emission branch, including `--delay-r9-ack-reorder`
- the post-return client dual-settlement / PTO loop
- `neko_session::SessionRuntime::{delivery_ack,confirmed_watermark}`
- `crates/neko-cli/tests/probe.rs::reliable_udp_ack_loss_delayed_original_reorder_settles`
- the existing post-return Carrier-ACK-withheld and Session-ACK-withheld negatives
- GitHub-hosted Rust CI run `35359163020` for exact `c585727`.

This is a source + hosted-cross-evidence review. No reviewer-local execution is claimed. No WAN/live evidence is involved.

## HIGH H-R9-057 — the R9-4 delay/reorder seam is not one-shot and exact HEAD is red

The intended seam contract says: withhold **only the first** legitimate post-return Carrier ACK, wait for the first fresh retransmission, then release the delayed original before the retransmission ACK; every later post-return packet must receive ordinary Carrier feedback.

Exact-current code uses only `delayed_post_ack: Option<(Vec<u8>, u64)>` as state:

1. first post packet: `None -> Some(delayed original)`;
2. second post packet: `Some -> take() -> None`, send delayed + current ACK;
3. third post packet: because state is `None` again, it is treated as a new "first" packet and withheld;
4. the pattern can repeat.

That is not a one-shot delay seam; it becomes alternating ACK suppression.

Hosted run `35359163020` demonstrates the consequence on exact `c585727`: `stable checks` failed. In `reliable_udp_ack_loss_delayed_original_reorder_settles`, the client sent original PN 4, PTO copies PN 5 and PN 6, then received positive retirements for PN 4 and PN 5 only. PN 6 remained in flight. Subsequent PTO deadlines fired until the bounded `post-return reliable-UDP dual settlement timeout`, so the new R9-4 positive fixture did not settle.

The same exact-head run also failed the two pre-existing post-return negative process tests:

- `reliable_udp_post_return_carrier_ack_withheld_fails`
- `reliable_udp_post_return_session_ack_withheld_fails`

Those failures expose stale cardinality assumptions after the new post-return receive-timeout behavior was changed to continue into PTO/recovery rather than terminalize on the first receive timeout. They must be reconciled truthfully; do not restore the old premature failure merely to satisfy old `residual.len() == 1` assertions.

### Required repair / discriminator

Use bounded explicit one-shot seam state (boolean or tiny enum is sufficient; no protocol state change):

- first post-return Carrier ACK only: delay it;
- next post-return packet: release delayed original first, then current ACK;
- all later post-return packets: ACK normally; never re-arm the delay automatically.

The positive R9-4 process test must tolerate scheduler timing that produces more than one retransmission before the delayed ACK is processed, while proving every actually sent later packet has a path to Carrier settlement. It must finish with `in_flight == 0`, exactly one positive Session confirmation transition for the tested range, no false Carrier rejection, and no post-resolution retransmission.

Reconcile the two existing negative tests to the new recovery loop rather than weakening them:

- Carrier ACK suppressed: Session may confirm, PTO/retransmit may repeat within the bounded deadline, but there is no positive Carrier retirement or settlement; the **terminal/final residual** must show Session done and Carrier still outstanding.
- Session ACK suppressed: Carrier may settle, but there is no positive Session confirmation or settlement; the **terminal/final residual** must show Carrier done and Session still outstanding.

Repeated intermediate residual diagnostics are not themselves a correctness failure. The oracle should bind the final residual/terminal result and relevant positive-transition cardinalities, not assume one receive timeout.

## HIGH H-R9-058 — H-R9-056 accepts non-exact / invalid logical ACKs below the watermark

The H-R9-056 direction is correct: a **freshly authenticated exact duplicate** Session `DeliveryAck` for a range already confirmed by this bounded operation must be idempotent/classification-only rather than charged as malformed.

The exact-current implementation is broader than that contract:

```rust
let end = offset.checked_add(len as u64).unwrap_or(u64::MAX);
if session_rt.confirmed_watermark(stream) >= end {
    diagnostic("accepted_empty_logical_ack");
    continue;
}
```

`confirmed_watermark(stream)` is only a prefix high-water. It is not evidence that the received `(stream, offset, len)` is an exact range previously admitted/confirmed by the current bounded operation.

Concrete counterexamples after a stream watermark has advanced to 16:

- an authenticated `DeliveryAck { offset: 8, len: 8 }` is accepted-empty even if that exact range was never an outbound record;
- an authenticated `DeliveryAck { offset: 1, len: 1 }` is accepted-empty for the same reason;
- more seriously, `len == 0` with `offset == 0` has `end == 0`, so on a stream whose watermark is 0 it can be accepted-empty even though `SessionRuntime::delivery_ack` explicitly rejects zero-length acknowledgements and no zero-length outbound record exists.

This violates the previous H-R9-056 repair contract that wrong stream/offset/length or otherwise unadmitted logical ACKs remain fail-closed. It also consumes no malformed budget, so authenticated semantically-invalid feedback can be hidden as benign stale feedback.

### Required repair / discriminator

Keep the positive H-R9-056 behavior but make its witness exact and bounded. Use existing current-operation ownership/context (for example, a bounded exact record/range context retained by the caller) rather than treating a cumulative watermark as exact-history proof. Do **not** add unbounded Session-ACK history, TTL/LRU/capacity policy, or a second Session delivery architecture.

Required deterministic tests:

1. freshly sealed exact duplicate of a genuinely confirmed current-operation `(session, stream, offset, len)` -> accepted-empty/classification-only, malformed budget unchanged, no second `delivery_ack`/window-release transition;
2. freshly sealed same-session ACK with a subrange/interior range whose `end <= confirmed_watermark` but which is not an exact admitted record -> `unexpected_logical_ack` / malformed path, no state mutation;
3. zero-length ACK -> fail closed, never accepted-empty;
4. unrelated/wrong stream/offset/length negatives remain fail closed;
5. identical authenticated-envelope replay remains a crypto replay rejection; do not conflate it with the fresh-envelope exact-duplicate case.

The R9-4 process fixture must then prove the exact positive duplicate generated by the real retransmission owner traverses this repaired bounded-exact path.

## Gate / queue effect

R9-4 is not independently closed at exact `c585727`:

- exact-head hosted stable gate is red;
- the intended one-shot ACK-loss/reorder seam is not one-shot;
- the H-R9-056 accepted-empty predicate is broader than the admitted exact-range contract.

Nightly decode fuzz smoke in the same hosted run succeeded, but there was no wire/parser/crypto-framing change here and that success does not override the failed stable gate or either finding.

Keep R9-5 and later lanes queued, but do not expand the changed correctness surface until H-R9-057/H-R9-058 are repaired, focused regressions pass, and a pushed exact-tree developer-local clean gate is recorded. After that, finish the full R9-4 process discriminator and continue immediately into R9-5 without waiting for reviewer cadence.

`READY_LIVE: none` remains correct: both findings are deterministic local correctness/evidence issues and introduce no new real-network question.
