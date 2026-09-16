# Reviewer checkpoint — R9-2 at exact `d133dbb`

## Scope

Independent bounded review of developer-owned exact source/test commit `d133dbb311fe61288cb8cf0881747fba0e48877c` (`fix(cli): Carrier ACK outcome class — accepted-empty vs rejected distinct (H-R9-026)`) against the current handoff, `neko-reliable` ACK semantics, and the cross-process R9 receive/demux owner.

Classification: implementation review only. No WAN/live run, performance conclusion, release approval, protocol freeze, or production claim.

## Repository / CI truth

- Exact reviewed developer source/test SHA: `d133dbb311fe61288cb8cf0881747fba0e48877c`.
- Developer diff changes only `crates/neko-cli/src/main.rs`; no focused regression was added in this commit.
- Hosted GitHub Actions run `35087016773` completed successfully on exact `d133dbb`:
  - `stable checks`: `bash scripts/check.sh` — success;
  - `nightly decode fuzz smoke`: pinned cargo-fuzz install, `cargo fuzz build decode`, and `cargo fuzz run decode -- -max_total_time=30 -max_len=8192` — success.
- Hosted CI is cross-evidence only. Final developer-local clean exact-tree provenance for R9-2 is still absent.
- Open PRs at review time: none.

## Accepted part of H-R9-026 repair

The shared authenticated ACK classifier no longer erases `PathRecoveryError` with `Result::ok()`. `UdpAcknowledgement::Carrier` now carries:

- `applied`: at least one actual recovery packet retirement;
- `acked_packets`: the actual retired packet identities, possibly empty for accepted stale/duplicate feedback;
- `rejected`: `apply_ack` returned an error, e.g. future/never-sent feedback.

The settlement-drain caller also correctly distinguishes `rejected` from accepted-empty: only `rejected == true` increments/emits `r9_udp_packet_ack_rejected`; accepted-empty is consumed as a typed non-event.

This is useful progress and must be retained.

# HIGH H-R9-026 remains open — the primary logical-confirmation receive loop still mislabels accepted-empty ACKs as rejected

The earlier receive loop, while logical Session confirmations are still being collected, currently matches:

```text
UdpAcknowledgement::Carrier { applied, .. } => {
    if applied {
        packet_ack_applied += 1;
        emit r9_udp_packet_ack_applied
    } else {
        packet_ack_rejected += 1;
        emit r9_udp_packet_ack_rejected
    }
}
```

This discards the new `rejected` bit at the caller.

Under current `Recovery::on_ack` semantics, a stale/duplicate canonical Carrier ACK can be validly accepted with `acked_packets=[]`; the shared classifier therefore returns `applied=false, rejected=false`. If that feedback arrives while the operation is still waiting for logical Session DeliveryAck(s), this primary loop increments the rejection counter and emits `r9_udp_packet_ack_rejected` anyway.

That is a concrete evidence/correctness contradiction to the H-R9-026 contract: accepted-empty feedback is still mislabeled as rejection on one live path through the same owner. A future/never-sent ACK remains a real typed rejection and should continue to produce the rejection event. No architecture change is required.

R9-3 remains blocked until this is repaired and regressions exist.

## Smallest repair contract

Do not change ACK framing, Session semantics, Carrier architecture, crypto/wire architecture, deadlines, malformed budgets, or any policy number.

1. Change the primary Carrier branch to preserve the same three-way outcome already used by the settlement drain, e.g. consume `{ applied, rejected, .. }`.
2. `applied == true`: keep the existing positive packet-transition counter/event.
3. `applied == false && rejected == true`: keep the rejection counter/event.
4. `applied == false && rejected == false`: accepted-empty stale/duplicate; consume as a typed non-event. It must not increment positive or rejection counters and must not emit either event.
5. Audit the remaining `UdpAcknowledgement::Carrier` consumers only for accidental re-collapse of these classes; do not create a second receive owner or redesign the ACK API.

## Required focused deterministic regressions

Use the existing authenticated ACK receive/demux seam and exercise the branch while logical confirmation is still outstanding.

### A. accepted-empty duplicate/stale before logical completion

- first cause a legitimate Carrier ACK retirement so a repeated canonical ACK becomes accepted-empty under current recovery semantics;
- deliver the duplicate/stale ACK through the primary logical-confirmation receive loop;
- require zero new positive packet transition and zero `r9_udp_packet_ack_rejected` increment/event from the duplicate;
- require Session state/logical confirmation unchanged by the duplicate itself;
- preserve the existing absolute deadline and malformed budget.

### B. future/never-sent before logical completion

- deliver a canonical ACK whose largest packet number is greater than `largest_sent` through the same primary owner;
- require exactly one typed rejection event/counter and zero positive packet transition;
- prove recovery RTT/PTO/loss/cwnd/packet state and Session state are unchanged by the rejected feedback, consistent with the current atomic `Recovery::on_ack` guard.

The developer commit `d133dbb` adds no test changes, so these regressions are still required before H-R9-026 can close.

## Queue / release effect

Keep the existing deep queue. After the narrow H-R9-026 repair and focused tests, continue immediately with the current P2 exact C1-C4 closure, post-return ACK-order challenge, the two P4 single-domain-suppression negatives, R9-2 developer-local exact-tree provenance, then R9-3 through R9-12 and the dedicated independent cross-process R9 review / factual release reconciliation.

`READY_LIVE` remains none: this finding is local correctness/evidence and creates no new real-network question.
