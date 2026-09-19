# Independent R9-7 review — H-R9-070 post-flood dual-feedback oracle

**Review anchor:** developer-owned source/test tree `d8b5adefc0309923be8540c928e4d435b85e1a52`; current reviewer-visible `main` before this note was `3c6e06871262712b046a597fe6d5e5ba4f62c9b1`.

**Evidence class:** bounded independent source/test/evidence review. Reviewer-local execution is not claimed. Developer-local exact-tree provenance recorded in `docs/CHATGPT_HANDOFF.md` remains distinct from GitHub-hosted CI. Hosted Rust CI run `35419496058` for exact `d8b5ade` completed successfully; this is cross-evidence only.

## Scope inspected

- `crates/neko-cli/src/main.rs` post-return reliable-UDP server owner around the `--flood-r9-ack` seam, Session `DeliveryAck`, and Carrier packet ACK emission.
- `crates/neko-cli/tests/probe.rs::reliable_udp_post_return_malformed_bound_is_terminal`.
- Exact developer diff `d8b5ade` closing H-R9-069's exact malformed-count / terminal-cause oracle.
- Applicable boundaries in `SECURITY.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, `IMPLEMENTATION_PLAN.md`, and current `docs/CHATGPT_HANDOFF.md`.

Excluded: no WAN execution, no policy-value changes, no D019 decision, no crypto/wire redesign, no capacity benchmark.

## Accepted narrow closure

H-R9-069's two narrow discriminators are repaired at exact `d8b5ade`:

1. the server seam emits exactly the existing malformed budget (`3`) rather than `0..=3`;
2. the client process regression requires exactly three typed `unexpected_logical_ack` classifications and the explicit `UDP delivery acknowledgement malformed bound exceeded` terminal cause.

The executable owner from H-R9-068 still treats only the ordinary delivery-ack receive timeout as non-terminal continuation; malformed-bound exhaustion remains terminal fail-closed.

## HIGH H-R9-070 — the claimed post-flood resurrection challenge is still under-discriminating

The regression says it proves that the server progressed beyond the malformed flood and emitted otherwise-valid feedback which could have settled the operation, yet the assertion is:

```rust
server_log.contains("\"event\":\"udp_return_delivery_ack_sent\"")
    || server_log.contains("\"event\":\"udp_return_packet_ack_sent\"")
```

That is insufficient for the current two-domain settlement contract.

For this exact seam, `seam_active == false`. The exact source owner sends the three bad Session DeliveryAcks first, then sends the legitimate Session `DeliveryAck`, then sends the legitimate Carrier packet ACK on the ordinary branch. Settlement requires both logical Session confirmation and Carrier packet retirement. The current test accepts **either one**.

Concrete mutation counterexamples that remain green:

- remove/suppress the post-flood Session `DeliveryAck` but keep the Carrier packet ACK;
- remove/suppress the post-flood Carrier packet ACK but keep the Session `DeliveryAck`;
- let the server fail after producing only one of those events; `srv_status` is currently discarded.

In all three cases the client still reaches the malformed bound, exits nonzero, emits no final success, and the current `OR` assertion can still pass. But then later valid feedback was never complete enough to settle the operation, so the claimed "malformed terminality cannot be resurrected by subsequent otherwise-valid feedback" challenge becomes vacuous.

This is an R9-7 process/evidence-oracle HIGH. It does **not** demonstrate a new transport-state defect in the current H-R9-068 source repair.

## Required smallest repair

Treat this as a test/seam/evidence repair unless a source defect appears.

1. Keep `MAX_POST_HANDSHAKE_MALFORMED` and all policy values unchanged.
2. Keep the exact three independently sealed unadmitted Session ACKs and explicit malformed-bound terminal cause.
3. Require **both** existing server diagnostics from this bounded operation:
   - `udp_return_delivery_ack_sent` for the exact post-return Session range;
   - `udp_return_packet_ack_sent` for the exact post-return packet identity.
4. Make the oracle mutation-sensitive to post-flood ordering. Prefer the smallest approach:
   - either add one seam-only typed `malformed_flood_complete`/equivalent diagnostic immediately after the third bad ACK and assert both valid-feedback events occur after it;
   - or prove the same ordering with an equally explicit existing typed discriminator. Do not infer ordering from a generic server summary.
5. If the current server result is deterministic on this seam, require its truthful terminal status instead of discarding `srv_status`; current source reaches `migration_back_complete` and should not be allowed to crash after only one feedback event while the client assertion stays green.
6. Preserve: client nonzero exit, exact malformed terminal cause, zero `r9_udp_post_return_settled`, zero `failover_client_ok`, zero final client `summary`, zero `ordered_records_complete`.
7. Preserve the ordinary timeout/PTO continuation and the R9-4 delayed/reordered ACK positive path.
8. No Session/Carrier/ACK/crypto/wire architecture change; no new retention/capacity policy.

After the repair, run on the final pushed developer SHA in a safe clean checkout/worktree:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust version, and clean-tree state. Decoder fuzz is not mechanically required unless decoder/parser/crypto framing changes.

## Queue effect

- H-R9-069: narrow exact-count / terminal-cause repair accepted closed.
- H-R9-070: **OPEN HIGH**, front of R9-7.
- R9-7 remains blocked pending this repair and then a remaining bounded process/result-truth sweep.
- R9-8 through R9-12, dedicated independent R9 review, Q10/Q11/Q12 reconciliation, and repository-wide item-4 refill remain queued and must not be collapsed.
- `READY_LIVE: none`; this finding is deterministic local correctness/evidence work and introduces no new real-network hypothesis.
- Release item 3 and item 4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`.
