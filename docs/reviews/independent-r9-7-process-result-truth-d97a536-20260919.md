# Independent R9-7 bounded review — process / result truth at d97a536

**Reviewed source/test anchor:** exact reachable pushed `d97a536414a78ff44ef80b1a6d428ddf96ae6e22` (with preceding H-R9-074 test repair `65d30dc8a70a9af0da23042f54147a87241123e7`).

**Repository/document head inspected:** `c16aac7e2e95a42fe024174fc699eba26bdd7bf5`.

**Classification:** bounded independent **NO NEW BLOCKER/HIGH FINDING** for the R9-7 process/result-truth slice inspected below. This closes the current H-R9-074 acceptance gap and the focused R9-7 re-challenge only; it is not a release/security approval and does not close later R9-8..R9-12 or repository-wide item 4.

## Scope inspected

- new developer-owned test commits `65d30dc8a70a9af0da23042f54147a87241123e7` and `d97a536414a78ff44ef80b1a6d428ddf96ae6e22`;
- current `crates/neko-cli/src/main.rs` reliable-UDP receive/demux, post-return first-send, PTO/retransmit, Session DeliveryAck, Carrier packet-ACK and terminal-result owners;
- current `crates/neko-cli/tests/probe.rs`, especially `reliable_udp_carrier_ack_send_failure_is_typed_not_sent`, `reliable_udp_post_return_malformed_bound_is_terminal`, first-send rollback coverage and reordered positive controls;
- current `crates/neko-reliable/src/lib.rs::Recovery::on_ack` / reservation rollback semantics;
- current Session delivery/ACK contract and the release/security/status/handoff boundaries relevant to this slice.

## Challenged invariants and result

1. **H-R9-074 packet/stage attribution is now non-vacuous.** The negative process fixture requires an initial `udp_packet_ack_sent`, requires a real `r9_udp_post_return_sent`, parses its packet number fail-closed, requires a post-return `udp_return_packet_ack_send_failed` for the target identity, requires the independent Session-side `udp_return_delivery_ack_sent`, forbids Carrier ACK application for the failed attempt, and forbids complete settlement/final failover success. The earlier optional `if let`/sentinel oracle is gone.
2. **Positive Carrier retirement remains tied to typed Recovery output.** The receive owner derives `applied` from a non-empty `acked_packets` set, keeps rejected and accepted-empty/stale outcomes distinct, and projects every actually retired packet rather than a representative-only packet.
3. **Dual-domain settlement remains conjunctive.** The post-return loop exits successfully only when Session logical ownership is complete (`post_outstanding.is_empty()`) and Carrier Recovery ownership is complete (`in_flight() == 0` when a runtime exists); Session DeliveryAck alone is not packet retirement and Carrier ACK alone is not Session delivery confirmation.
4. **Malformed-bound terminality remains fail-closed.** The reliable demux uses the existing finite malformed budget and returns `UDP delivery acknowledgement malformed bound exceeded`; the executable caller continues only the exact ordinary receive-timeout classification for PTO/delayed-feedback progress. Malformed-bound exhaustion and receive/socket failure do not re-enter the success path.
5. **First-send socket failure remains rollback, not positive send evidence.** The executable owner abandons the reserved Recovery packet on send failure and emits `r9_udp_post_return_send_failed`; it does not emit `r9_udp_post_return_sent` for that failed socket attempt.
6. **Future/never-sent Carrier ACKs remain fail-closed before mutation.** `Recovery::on_ack` rejects an ACK whose largest packet exceeds the committed `largest_sent` watermark before RTT/loss/PTO mutation; this preserves the earlier Candidate-A closure.
7. **Controlled loss remains explicitly distinguishable from socket failure.** `--drop-r9-data` deliberately commits Recovery ownership then suppresses the wire send and emits the controlled-drop evidence used to drive PTO/retransmit; it is not treated as an OS socket-success claim.

No current source/test contradiction was found that justifies another repair slice in this bounded R9-7 scope.

## Validation / evidence class

Reviewer-local command execution is **not claimed** in this review. The accepted first-class execution evidence remains the developer-recorded clean exact-tree gate for `d97a536`:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

with both commands recorded exit 0, clean tree, Linux x86_64, stable Rust, and exact pushed SHA. `scripts/check.sh` includes `cargo test --workspace --locked`, so the current non-ignored R9 process regressions are part of that exact-tree gate.

GitHub-hosted Rust CI run `35433084550` for exact `d97a536` completed `success` on 2026-09-19. Hosted CI is cross-evidence only; it is not substituted for developer-local provenance or reviewer execution.

Useful focused commands for any later mutation/reopen of this slice remain:

```text
cargo test -p neko-cli --test probe reliable_udp_carrier_ack_send_failure_is_typed_not_sent -- --exact --nocapture
cargo test -p neko-cli --test probe reliable_udp_post_return_malformed_bound_is_terminal -- --exact --nocapture
cargo test -p neko-cli --test probe reliable_udp_first_send_socket_failure_rolls_back -- --exact --nocapture
```

No decoder/parser/crypto-framing source changed in the H-R9-074 repair, so this review does not invent a new fuzz obligation.

## Exclusions / boundaries

- No R9-8 warm-readiness, R9-9 health/promotion, R9-10 uncertain replay/dedup, R9-11 lifecycle-wide or R9-12 final-provenance closure is claimed here.
- No new WAN question was created; `READY_LIVE` remains `none`.
- No change to D019, retention/capacity/security numbers, signing/SBOM/publication policy, Session/Carrier/ACK/crypto/wire architecture, release/freeze/production authority, or prior frozen-release policy.
- Release item 3 and item 4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`.

## Queue consequence

Accept H-R9-074 as closed at `d97a536` and mark the focused R9-7 process/result-truth re-challenge **bounded no-finding closed** at this review anchor. Proceed immediately to the existing mid-slice R9 factual reconciliation, then R9-8 through R9-12 and the dedicated independent R9 review without waiting for reviewer cadence.

The `c16aac7` handoff header already says H-R9-074 is closed, but its first READY_LOCAL section still re-lists H-R9-074 as an open FRONT HIGH ticket. That is a stale navigation inconsistency, not a source defect; the next handoff refresh must remove the duplicate closed ticket while preserving the deeper dependency-ordered queue.