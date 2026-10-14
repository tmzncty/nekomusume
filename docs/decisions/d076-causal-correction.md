# D076-C — causal correction: multi-flow recovery observed; mechanism unresolved

**Status: Accepted correction candidate**

This note supersedes only the **causal wording** of D076. It does not invalidate the measured samples or the W5 phase-1 implementation.

## Correction

D076 currently says that a per-connection inbound clamp was "confirmed" from this monotonic sequence:

| N | aggregate receive |
|---:|---:|
| 1 | 0.49 Mbps |
| 2 | 3.65 Mbps |
| 4 | 199 Mbps |
| 8 | 193 Mbps |

That wording is too strong for the evidence currently recorded.

The defensible conclusion is:

> **Multi-flow recovery was observed. The causal mechanism remains unresolved.**

Reasons:

1. `N=2 -> N=4` is a discontinuous ~55x jump, unlike a simple fixed per-flow ceiling.
2. The tests were monotonic in stream count, so stream count and elapsed time were not separated.
3. A separate XJ control recovered from 0.04–1.93 Mbps to ~180 Mbps single-flow in the same overnight window, proving that time-varying path/operator state existed.
4. Multiple 5-tuples can also sample different ECMP/LAG/middlebox paths or classifier buckets; aggregate SUM alone cannot distinguish that from a true concurrency threshold.

## What remains valid from D076

- The recorded N=1/2/4/8 measurements remain evidence.
- The ~200 Mbps path-capacity observation remains useful.
- Fixed multi-stream W5 phase 1 remains justified as an operational escape mechanism.
- The original observation is important enough to drive a causal recheck.

## What is no longer claimed

Until the causal recheck is complete, do not claim any of the following as established fact:

- a fixed per-connection policer is the mechanism;
- four simultaneous streams are a protocol-level magic threshold;
- N=4 causes the recovery rather than coinciding with recovery;
- adaptive phase 2 should always increase to four streams.

## Required recheck

Run `docs/experiments/d076-causal-recheck.md`:

1. interleaved A/B/A/B/A (`1,4,1,4,1`);
2. per-stream rather than SUM-only evidence;
3. 4->3->2->1 downshift with surviving sockets preserved;
4. repeated N=1 source-port sweep;
5. randomized order of `{1,2,4,8}` across repeated cells.

The deterministic toy model is `tools/d076_causal_sim.py`; the optional privileged Linux lab is `tools/d076_netns_lab.sh`.

## Phase-2 design fork

Keep both designs alive until evidence selects one:

- **stream-count adaptation** if capacity genuinely scales with simultaneous flow count;
- **flow reselection / path entropy** if flow identity determines whether a carrier lands on a good or bad path.

The second case may need fewer steady-state sockets: probe several candidate 5-tuples, retain the healthy carrier, and retire poor candidates instead of permanently striping across four streams.
