# Independent R9 review — H-R9-053 discriminator closure mismatch

**Reviewed anchor:** `482178973185c081b6b99b417490bf36287b612c` (reachable on `main` under handoff `f2bf103c7062197cdbfc5bb7e94a0fcfeabfdce5`).

**Classification:** HIGH evidence/oracle correctness (`H-R9-054`). This does **not** presently prove a new runtime defect in `Recovery`; it proves that the current H-R9-053 closure overstates what its deterministic regression actually discriminates.

## Scope inspected

- `crates/neko-reliable/src/lib.rs::Recovery::{on_sent,abandon_sent,on_ack}`
- `crates/neko-carrier/src/lib.rs::ReliableUdpRuntime::{on_retransmit_sent,abandon_retransmit}` and the H-R9-052/H-R9-053 path-recovery regressions
- `crates/neko-cli/src/main.rs::cli_regression_tests::abandoned_retransmit_preserves_retired_committed_watermark`
- current `docs/CHATGPT_HANDOFF.md` H-R9-053 acceptance contract
- `SECURITY.md` nonce-reuse prohibition and current packet-number monotonicity boundary

No wire decoder/parser/crypto framing was changed or reviewed here, so fuzz is not implicated by this finding. Reviewer did not execute a local gate in this pass; exact `4821789` developer-local provenance is retained separately, and hosted Rust CI run `35327703324` is cross-evidence only.

## Challenged invariant

After a successful committed packet `N` is ACKed/retired, a higher pre-send reservation `M` is aborted, and the committed watermark is restored:

1. late/duplicate ACK for the genuinely sent `N` remains legal under existing accepted-empty semantics;
2. ACK for aborted `M` fails closed atomically;
3. **packet-number reuse at or below committed `N` is rejected by the send owner**;
4. a fresh packet number above `N` remains admissible.

Item 3 matters independently of ACK classification because packet number is also the authenticated record sequence / nonce source on the reliable-UDP path; a regression that re-allows reuse must deterministic-red.

## Finding

The current source repair appears to implement the intended monotonicity: `Recovery::on_sent` rejects `p.number <= largest_sent`, and `abandon_sent` restores the recorded pre-reservation watermark.

However, the two regressions used to close H-R9-053 do **not** actually exercise the required reuse rejection:

- `path_recovery_tests::abandon_restores_committed_watermark_not_max_outstanding` comments that reuse of `3` or below “must be refused” and that a fresh `5` is admissible, but after aborting `4` it only tests ACK(4) rejection and duplicate ACK(3) acceptance. It does not call `on_retransmit_sent(3|<=3)` and does not prove the fresh-send control.
- `cli_regression_tests::abandoned_retransmit_preserves_retired_committed_watermark` likewise comments that `on_sent` still rejects packet numbers `<=` committed watermark, but its only post-abort send assertion is a successful fresh reservation at `101`; it never attempts `0` or `1` (the committed high-water in that fixture) and therefore would remain green if the send-side monotonicity check were accidentally weakened while ACK-side behavior stayed intact.

Accordingly the handoff sentence saying the current regression “proves ... `on_sent` still rejects `<=` committed watermark” is factually too strong. The exact source semantics may be correct today, but the acceptance-grade discriminator requested by the prior reviewer contract is missing.

## Smallest closure contract

Do not redesign Recovery, ACK architecture, Session/Carrier semantics, crypto framing, or policy values. Keep the accepted per-reservation committed-watermark repair.

Add a focused deterministic regression on the real `ReliableUdpRuntime` send owner:

1. commit `N`, ACK/retire `N` completely;
2. reserve higher `M`, abort `M`;
3. prove late/duplicate ACK(`N`) remains accepted under current semantics;
4. prove ACK(`M`) is rejected atomically;
5. **attempt a new send/retransmit using packet number `N` (and, where the API permits, one lower number) and require deterministic rejection with no new Recovery/Reno/frame ownership**;
6. paired control: a fresh packet number `> N` succeeds normally.

The regression must fail if `Recovery::on_sent` is changed to permit reuse at/below the restored committed watermark while leaving ACK behavior unchanged.

After the focused repair/test commit, run the normal clean exact-tree developer-local gate and continue immediately to H-R9-042; do not wait for reviewer cadence.
