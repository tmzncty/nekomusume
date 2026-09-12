# Independent bounded UDP recovery review (superseding) — exact `02b6eaa`

Bounded independent re-review of `crates/neko-reliable/src/lib.rs` against `docs/spec/m2-udp-recovery.md`, at reachable exact `02b6eaae573187aab2329398e95d69b5808296d7`. Supersedes the boundary conclusion of `docs/reviews/independent-reliable-udp-8f93b93-20260913.md` on one point only; all other findings there stand.

## Correction to the prior note

The earlier note (`8f93b93`) stated that "`largest_sent == None` (nothing sent yet) is not rejected by this rule, which is safe because an empty `sent` map yields no acked/lost output." That reasoning was about output volume only; it missed that the already-confirmed R1 contract classifies a **nonempty ACK against a no-sent-state `Recovery`** as mechanically invalid regardless of output. A peer ACKing packet numbers the sender could never have sent is malformed input, not merely harmless.

`02b6eaa` closes this residual boundary: `Recovery::on_ack` now rejects any nonempty ACK whenever `largest_sent` is `None` or `largest > largest_sent`, atomically and before any RTT/packet/loss/PTO mutation (`:289-296`). The valid boundaries are preserved:

- `largest == largest_sent` is accepted (`ack_largest_equal_to_largest_sent_is_accepted`);
- an already-retired packet number `<= largest_sent` that is absent from the live `sent` map is still acceptable — the ACK is processed and only live packets are acked/marked-lost; nothing rejects a retired-but-valid number;
- no-sent-state nonempty ACK is rejected (`ack_with_no_sent_state_is_rejected`);
- strictly-future largest remains rejected atomically (`ack_largest_beyond_largest_sent_is_rejected_atomically`).

## Confirmed unchanged findings (carried over)

ACK canonicalization and atomic range bounds, packet-number exhaustion/monotonicity, post-retirement ACK/loss, `outstanding_frames` copy counts, packet/time-threshold and reorder boundaries, RTT/PTO arithmetic, PTO-never-loss/delivery, persistent-congestion accounting, Reno bounds, and deterministic fault-simulation termination all hold on this exact tree — unchanged from the `8f93b93` review.

The empty-outstanding `on_pto` `pto_count` arming observation remains an open caller-scheduling/semantic boundary (`DEFER_POLICY_OR_ENVIRONMENT`), not a defect.

## Evidence

- `cargo test -p neko-reliable` on exact `02b6eaa`: 30 lib + 3 + 3 integration, all passed.
- Coherent exact-tree gate on the pushed source anchor containing both the completed R1 repair and the O1 observer repair:

```text
SHA:       02b6eaae573187aab2329398e95d69b5808296d7
start UTC: 2026-09-12T18:55:23Z
end UTC:   2026-09-12T18:57:19Z
host:      Linux 6.8.0-137-generic x86_64
rustc:     1.98.0 (88d9e12ae 2026-08-18)

PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh: exit 0
git diff --check:                            exit 0
initial/final tree:                          clean
```

R1 (future/no-sent ACK) and O1 (datagram drop reason attribution) are both closed on this reachable anchor. No new `READY_LIVE` question; release/governance state unchanged.
