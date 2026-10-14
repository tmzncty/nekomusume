# D076 causal recheck: distinguish stream-count effect from time-window and path-hash effects

Status: proposed experiment protocol (updated 2026-10-14: time-window effect is now **observed**, not merely hypothesized — see below)

## Why this exists

D076 observed the following US→CN public-TCP aggregate throughput sequence:

| parallel streams | aggregate throughput |
|---:|---:|
| 1 | 0.49 Mbps |
| 2 | 3.65 Mbps |
| 4 | 199 Mbps |
| 8 | 193 Mbps |

The observation is real, but the causal statement "per-connection clamp confirmed" is stronger than the current evidence supports.

Two facts make causal re-check mandatory:

1. `N=2 -> N=4` is a discontinuous ~55x jump, not the shape expected from a simple fixed per-flow rate cap.
2. In the same overnight window, the XJ downlink control recovered from 0.04–1.93 Mbps to ~180 Mbps on a single flow. Time-varying path/operator state therefore existed during the measurement window.

**Update 2026-10-14 (D077 evidence window):** the time-window effect itself is now directly observed on the same command, same path, same node pair (us-209→bjlh:40200, iperf3 -P 4): morning 10-13 10:55 = 199 Mbps, midday 12:45 = 1.47 Mbps, next morning 10-14 07:53 = 314 Mbps (limits doc §3.3, D077). So:

- **H1 (time-window) is no longer a hypothesis about whether windows exist — they demonstrably do.** What remains open is whether H1 explains the original N=1/2/4/8 sequence, i.e. whether the morning measurements of N=4/N=8 and the earlier N=1/N=2 happened to straddle window edges. The A/B/A/B/A interleave (Experiment 1) is now the decisive test.
- H2 (5-tuple/path entropy) remains fully open and untested.
- Note on the W5/D077 multi-stream evidence (neko N=4 = 247.3/311.4 Mbps, NIC-counter verified): it was taken inside one morning window with the iperf3 gate precheck at 314 Mbps — it validates the multi-stream *escape mechanism* at that window, not a causal claim about stream count.

(Units errata applied 2026-10-14: historical neko TCP anchor "15.22/16.59/17.87 Mbps" field values were bytes/s mislabeled as bps; true bit-rates are 121.8/132.7/143.0 Mbps — ratios unchanged. Any future cross-reference to R1 throughput should use the ×8 values. iperf3 numbers in this document are true bit-rates and unaffected.)

The safe statement today is:

> During one measurement window, N=1/N=2 were slow and a later N=4/N=8 measurement reached ~200 Mbps. Multi-flow is an effective operational escape hatch in that window; the mechanism is not yet identified.

This document defines experiments that separate four candidate mechanisms:

- H1: time-window/path-state recovery (stream count is incidental);
- H2: 5-tuple/ECMP/LAG path selection (some flow hashes land on a good path);
- H3: genuine per-flow policer / loss interaction;
- H4: shared middlebox/classifier state with a connection-count threshold.

## Experiment 1: A/B/A/B/A interleave

Run the same endpoint pair and direction with minimal gaps:

```text
N=1  -> 30 s
N=4  -> 30 s
N=1  -> 30 s
N=4  -> 30 s
N=1  -> 30 s
```

Record per-stream, not only SUM:

- sender/receiver throughput;
- retransmissions;
- RTT / RTO / cwnd / rwnd where available (`ss -ti` or TCP_INFO);
- local source port and remote tuple;
- wall-clock start/end timestamp.

Interpretation:

- `0.5, 200, 0.5, 200, 0.5`: strong stream-count effect;
- `0.5, 200, 180, 200, 190`: time-window recovery likely explains original jump;
- N=4 aggregate ~200 with one stream ~190 and the rest ~0.5–3: path-hash/ECMP candidate;
- N=4 aggregate ~200 with all four near ~50 and N=1 repeatedly slow: shared classifier/policer threshold becomes more plausible.

## Experiment 2: downshift from 4 -> 3 -> 2 -> 1

Start N=4, wait for steady throughput, then terminate one stream at a time while preserving the surviving sockets.

Questions:

- Does one surviving stream remain near line rate? If yes, the winning socket/5-tuple is more important than concurrency itself.
- Does aggregate throughput collapse immediately below a particular live-flow count? If yes, investigate shared state / threshold classification.
- Do all surviving streams remain proportionally capped? If yes, per-flow policy is more plausible.

## Experiment 3: source-port sweep / flow reselection

Run repeated N=1 probes while varying only the source port. Keep destination, protocol, payload, direction, duration, and time spacing fixed.

Example result shape to look for:

```text
src=30000  0.5 Mbps
src=30001  0.7 Mbps
src=30002  183 Mbps
src=30003  0.4 Mbps
src=30004  191 Mbps
```

Stable fast/slow buckets correlated with flow identity would strongly support ECMP/LAG/hashed-path selection.

If the harness cannot pin source ports directly, create a tiny raw TCP probe helper rather than overloading the production Session path.

## Experiment 4: repeated randomized order

Repeat permutations of `{1,2,4,8}` rather than always increasing stream count. Minimum recommended design:

```text
4,1,8,2
2,8,1,4
1,4,2,8
8,2,4,1
```

This prevents monotonic time drift from masquerading as a stream-count curve.

Use at least three repetitions per cell before calling a mechanism "confirmed".

## Decision rule for D076 wording

Until Experiment 1 plus either Experiment 2 or 3 is complete, use:

> multi-flow recovery observed; causal mechanism unresolved.

Only restore "per-connection clamp confirmed" if repeated interleaving excludes time-window recovery and the per-stream evidence matches a per-flow limiting mechanism.

If port sweep shows stable fast/slow flow identities, prefer language such as:

> flow-identity/path-selection sensitivity confirmed; multi-flow increases path entropy and escape probability.

## Design consequence for W5

Do **not** teach an adaptive algorithm that `N=4` is a magic threshold yet.

Keep fixed-stream W5 phase 1 because it is a useful operational mechanism regardless of cause. For phase 2, retain two distinct strategies until mechanism is known:

1. **stream-count adaptation** — increase simultaneous streams when aggregate capacity scales with concurrency;
2. **flow reselection / path entropy** — rapidly probe candidate 5-tuples, retain a good carrier, and retire poor candidates when flow identity dominates.

These strategies have different resource and failure semantics and should not be conflated.
