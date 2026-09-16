# Reviewer checkpoint — exact `bbb3e7f`: H-R9-027 reaches the intended Recovery inputs, but acceptance remains false-positiveable

**Anchor reviewed:** `bbb3e7f5310dea6a2342a2eda212cbc34326fd89`

**Classification:** implementation/test-seam review; local correctness/evidence support for release item 4. No live/WAN claim and no protocol/release decision.

## Repository truth reviewed

- `main` was exact `bbb3e7f5310dea6a2342a2eda212cbc34326fd89` at review time.
- Developer-owned delta since the previous reviewer handoff: one source commit, `bbb3e7f`, touching only `crates/neko-cli/src/main.rs`.
- Hosted Rust CI run `35104209670` completed `success`; `stable checks` ran `bash scripts/check.sh`, and `nightly decode fuzz smoke` also completed successfully. This is hosted cross-evidence only, not developer-local exact-tree provenance.
- Open PRs: none.
- Authoritative repository state remains `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`; release items 3 and 4 remain incomplete; `READY_LIVE: none`.

## What exact `bbb3e7f` fixes correctly

The previous H-R9-027 source-shape defects are repaired:

1. `--send-stale-ack` no longer retransmits the identical authenticated ciphertext. It re-seals the same canonical Carrier-ACK plaintext through `SecureSession::seal_unreliable`, so the second semantic ACK has a fresh authenticated sequence/nonce and can pass crypto replay protection to reach the Recovery classifier.
2. `--send-future-ack` now constructs a non-empty canonical ACK range whose maximum is `u64::MAX - 1`, so `AckRanges::largest()` is populated and the value is deterministically greater than any packet number this bounded process could have sent. The seam therefore reaches the existing `largest > largest_sent` guard rather than failing first on an empty range.
3. Existing engine-level candidate-A protection remains present: `Recovery::on_ack` rejects future/never-sent largest values before RTT/loss/PTO mutation. The earlier exact `531c82d` regression snapshots RTT and in-flight state and proves a subsequent legitimate ACK still operates on untouched state; the authenticated `PathRecovery` seam also retains wrong-generation/future rejection coverage.

No architecture change is required.

## H-R9-028 — HIGH acceptance/evidence gap

The process regressions inherited from exact `91f481b` are still too weak to prove the repaired seams exercised the intended outcome classes, and both fault injectors are currently repeated rather than one-shot.

### A. Stale/duplicate accepted-empty is not actually observed

Current server code evaluates `--send-stale-ack` inside every reliable packet-ACK emission. There is no one-shot guard, so a count=4 run can inject the duplicate semantics repeatedly. The current process test then checks only:

- absence of `r9_udp_packet_ack_rejected`; and
- eventual `r9_udp_in_flight_settled(...remaining_in_flight=0)`.

It does **not** prove that any fresh-envelope duplicate reached `UdpAcknowledgement::Carrier { applied:false, rejected:false, acked_packets:[] }`. The same assertions can pass if the injected duplicate is never consumed, is dropped after the last real retirement, or otherwise contributes no classification evidence. It also does not assert client/server process success or exact positive-transition cardinality.

The injection ordering is also not the previous contract: the re-sealed semantic duplicate is sent before the ordinary `sealed_ack`, so the injected datagram can be the real retiring ACK and the later ordinary ACK becomes accepted-empty. That can still create an accepted-empty outcome, but it does not deterministically prove “one legitimate retirement, then one duplicate accepted-empty” and makes the intended seam harder to bind.

### B. Future/never-sent rejection is only a broad substring oracle

Current server code likewise evaluates `--send-future-ack` on every reliable packet-ACK emission, so multiple future rejections are possible. The process test asserts only that client stdout contains at least one `r9_udp_packet_ack_rejected` event. It does not require:

- exactly one injected future rejection;
- client/server successful completion;
- no false positive packet retirement from the future ACK;
- subsequent legitimate Carrier ACK retirement and terminal Recovery zero; or
- unchanged logical Session confirmation/accounting relative to the non-fault path.

Therefore a run in which the future ACK is rejected but later legitimate settlement breaks can still satisfy the present assertion.

## Required smallest repair

Keep H-R9-026 source outcome semantics closed. Repair only the bounded regression seam/oracles:

1. Add explicit one-shot ownership for each injection (`stale_ack_injected`, `future_ack_injected` or equivalent) outside the per-record ACK emission loop. Do not create a second receive owner.
2. For the stale case, arrange one normal canonical ACK retirement followed by one fresh-envelope copy of the same ACK semantics while another reliable-owned operation is still being drained, so the client deterministically consumes exactly one accepted-empty duplicate.
3. Make accepted-empty observable **only as diagnostic classification**, not as delivery/packet-transition evidence. A test-scoped/current diagnostic such as `r9_udp_packet_ack_accepted_empty` is acceptable when emitted only for `!applied && !rejected && acked_packets.is_empty()`. It must increment no applied/rejected/delivery counter.
4. Stale process regression: require client and server success; exactly one accepted-empty diagnostic; zero packet-ACK rejection; no extra positive Carrier retirement caused by the duplicate; unchanged Session logical confirmation cardinality; terminal Recovery zero.
5. Future process regression: exactly one rejection from the one-shot future injection; no future-ACK positive retirement; require subsequent legitimate Carrier ACK progress and final Recovery zero; require client/server success and unchanged Session logical completion.
6. Reuse/strengthen the existing exact engine-level future atomic regression rather than adding a parallel checker framework. If caller-visible Reno/PTO state is directly exposed in the existing `ReliableUdpRuntime` test seam, snapshot it there; do not invent policy values.
7. Run focused deterministic tests, then the full clean exact-tree local gate on the final pushed source/test SHA. No decoder/framing semantics are changed by the intended repair, so decoder fuzz is not required solely for this slice; hosted CI remains cross-evidence.

**R9-3 remains blocked until these process oracles prove the intended accepted-empty and future-rejected classifications rather than merely making the run green.**

## Preserved queue after H-R9-028

Do not shrink the existing deep handoff. After H-R9-028, continue immediately through positive P2 C1-C4 exact closure, both post-return ACK arrival orders, both P4 single-domain suppression negatives, R9-2 developer-local exact-tree provenance, then R9-3 through R9-12 and the dedicated cross-process R9 independent bounded review/reconciliation lanes.
