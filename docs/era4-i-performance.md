# Era-4 I performance evidence slice

Status: **bounded measurement harness only**. This is not a superiority claim,
benchmark publication, release gate, or production capacity estimate.

## Reproduction

From the repository root:

```sh
NEKO_BENCH_ITERS=1000 ./scripts/bench/run-era4-i.sh
```

The harness clamps its own default to 1,000 and accepts 1–10,000 iterations.
Each operation is timed with `std::time::Instant`; output is JSON Lines with
median, P95, and failures in nanoseconds. The workload is local and synchronous:
wire frame encode/decode, outer record encode/decode, Noise transport seal/open,
`FairScheduler::next_frame`, and deterministic recovery ACK/loss processing.
`instrumentation_counter` is only a proxy for the cost of one explicit counter
increment, not runtime-wide instrumentation overhead.

## VPS observation at commit

Host/repository: authorized VPS `192.168.122.1`, isolated detached worktree,
parent `9a03d6b7125a0e04c05d6f3ba154d1004308749e`.

Observed with release build and 1,000 iterations (2026-08-30, UTC+8):

| operation | median ns | P95 ns | failures |
|---|---:|---:|---:|
| encode/decode | 230 | 300 | 0 |
| outer record encode/decode | 70 | 90 | 0 |
| crypto seal | 2,070 | 2,080 | 0 |
| crypto open | 2,070 | 2,340 | 0 |
| scheduler next frame | 190 | 200 | 0 |
| recovery ACK/loss | 810 | 1,230 | 0 |
| instrumentation counter proxy | 30 | 40 | 0 |

These values are one bounded observation, without warm-up protocol, CPU pinning,
statistical confidence intervals, or comparison baseline. They must not be
interpreted as superiority or end-to-end throughput/latency.

## Honest boundary / remaining I

CPU and memory are not reported: this slice has no stable cross-platform,
process-scoped sampler. Network behavior, scheduler fairness under realistic
multi-stream load, retransmission under controlled loss, instrumentation in the
actual runtime, repeated VPS runs, confidence intervals, and independent review
remain open. No Era-4 track status is promoted by this evidence.
