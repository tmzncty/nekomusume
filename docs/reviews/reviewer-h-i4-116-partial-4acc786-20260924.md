# H-I4-116 exact-current partial closure review — `4acc786`

**Exact source/test anchor:** `4acc786919a979d38ddd34af75a65e4026e54982`

**Classification:** reviewer source/control-flow review of a developer-owned correctness repair. This note is not developer-local CI, hosted CI, cross-platform execution, fuzz, WAN evidence, or a performance conclusion.

## Result

H-I4-116 is **PARTIALLY REPAIRED, NOT YET CLOSED**.

The developer-owned exact-current repair is narrow and directionally correct:

- `neko_reliable::Reno::lost(0)` now returns before changing `cwnd`, `ssthresh`, or `bytes_in_flight`.
- The direct regression `reno_lost_zero_is_noop` proves those three fields remain unchanged for zero loss.
- The same direct test retains a non-zero `lost(400)` positive control, so real Reno loss reduction has not been disabled at the owner level.

This preserves the existing congestion numeric policy and requires no ACK, wire, crypto, D019, or persistent-congestion redesign.

## Remaining closure gap

The exact-current tree still lacks the focused cross-owner regression required by the H-I4-116 contract.

`PathRecovery::on_ack` continues to aggregate charged bytes for ACKed and lost packets and then calls:

```text
reno.acked(released_acked)
reno.lost(released_lost)
```

The owner-level `lost(0)` guard therefore repairs the production path, but the integration contract still needs an executable regression proving that a **loss-free ACK outcome** cannot cause multiplicative decrease through `PathRecovery` / `ReliableUdpRuntime`.

Two existing tests are insufficient to distinguish the repaired behavior from the pre-fix bug:

- `congestion_window_gates_send_and_releases_on_ack` only proves a 1200-byte send is possible after ACK; even the buggy halved window would admit 1200 bytes.
- `runtime_send_admission_refuses_when_cwnd_full_and_recovers_on_ack` only proves a 400-byte send is possible after ACKing ten 400-byte packets; the buggy post-ACK halved window would still admit 400 bytes.

A minimal deterministic owner-spanning regression can use existing `path_recovery_tests` seams without adding production debug APIs:

1. create `PathRecovery` with MSS 1200;
2. send one 1200-byte ack-eliciting packet;
3. ACK that packet with no induced loss;
4. assert `bytes_in_flight() == 0`;
5. assert `can_send(12_000)` (or an equivalent threshold that succeeds with the unchanged/grown loss-free window and fails under the old zero-loss multiplicative decrease).

A narrow aggregate positive-loss control should also preserve the intended integration fact that `released_lost > 0` still causes one Reno reduction for that aggregate loss outcome. Reuse existing public admission/accounting seams; do not add a production cwnd getter merely for the test.

## Evidence boundary

For exact `4acc786919a979d38ddd34af75a65e4026e54982`:

- GitHub combined status exposed no status entries at reviewer time.
- GitHub workflow-run lookup exposed no runs at reviewer time.
- No reachable repository-persisted developer-local exact-tree provenance for this final source tree was present at reviewer time.
- The reviewer did **not** run the Rust/full repository gate, cross-platform execution, fuzz, WAN tests, or performance measurements.
- No decoder/parser/crypto-framing owner moved in this repair, so fuzz is not mechanically required for H-I4-116 closure.

## Required next action

Keep H-I4-116 at the queue front until the coding agent:

1. adds the focused loss-free `PathRecovery` / `ReliableUdpRuntime` integration regression;
2. preserves/proves the non-zero aggregate-loss reduction path;
3. pushes the final source/test SHA;
4. runs the developer-local clean exact-tree gate on that pushed SHA:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - clean tree;
5. persists reachable provenance with exact SHA, UTC start/end, exit codes, OS/arch, stable Rust version, and clean-tree state;
6. immediately continues the remaining R-REC-4 Reno/fault-simulation review rather than waiting for the next reviewer cadence.

Release items 3 and 4 remain incomplete; release/governance flags and `READY_LIVE: none` are unchanged.
