# D075 — SOCKS pump pipeline and next measurement sequence

**Status:** Proposed

**Scope:** research implementation only; no protocol freeze, release, production, or security-readiness change.

## Why this exists

D074 established two useful facts:

1. the bulk UDP client had a per-window stop-and-wait shape that was removable without changing wire semantics; and
2. after that repair, loopback improved dramatically while the 78 ms mesh path barely moved, exposing a second limiter instead of justifying more blind tuning.

The current SOCKS5 research entry still has a deliberately simple per-record stop-and-wait pump. That implementation was the correct MVP shape for D071/D072, but W2/W4 throughput is now low enough that the pump itself is a first-order suspect. The next work should therefore be split into small, falsifiable PRs rather than another broad feature PR.

This note fixes the order and acceptance questions for that work.

---

## PR 1 — D075: replace SOCKS per-record stop-and-wait with a bounded ACK-driven pipeline

### Question

Can the existing SOCKS5 entry move substantially more application data by removing the per-record stop-and-wait pump while preserving all current security, resource, and half-close semantics?

### Intended change

Keep the existing D071/D072 surface and wire semantics. Change only the application pump scheduling shape:

```text
old:
read app -> queue one record -> wait for confirmation -> read/queue next

candidate:
read app -> fill bounded in-flight budget
                  ^              |
                  |              v
             ACK/retire <- receive/confirm

half-close remains a tagged application event and does not tear down the reverse direction.
```

The implementation should be driven by readiness/ACK progress, not by an unbounded producer loop.

### Must not change

- Noise authentication and trust policy;
- SOCKS5 CONNECT subset semantics;
- client listener restricted to loopback;
- server egress allowlist and fail-closed startup;
- refusal of disallowed/internal targets;
- target-address logging policy;
- pre-auth accounting semantics;
- application half-close semantics;
- measurement gate;
- current default resource ceilings unless a separate reviewed decision explicitly changes them;
- wire format.

### Required observability

Add bounded, non-address-bearing counters sufficient to distinguish scheduling failure from carrier failure. At minimum:

- records admitted;
- records retired/confirmed;
- maximum in-flight records/bytes observed;
- pump wake/turn count;
- application bytes in each direction;
- half-close events;
- queue-full/backpressure events.

Counters should be available only in the existing research/measurement surface and must not introduce target addresses into logs.

### Acceptance criteria

The PR is successful only if all of the following hold:

1. existing SOCKS functional tests remain green;
2. CONNECT success/failure behavior is unchanged;
3. bidirectional transfer still works;
4. half-close works independently in both directions;
5. queue and memory ceilings remain bounded and test-covered;
6. malformed/early SOCKS input still cannot bypass the preceding authenticated session;
7. a fixed local or controlled test workload shows a material throughput gain over the current per-record stop-and-wait baseline, **or** the PR records a typed negative result that falsifies the pump-as-primary-limiter hypothesis;
8. no wire-format change is required.

Do not declare a specific percentage improvement in advance. Preserve the old baseline and record the exact workload, payload size, elapsed time, application bytes, and build identity so a non-improvement remains useful evidence.

### Stop rule

If the pipeline implementation reaches the same throughput region as the old pump while the new counters show the in-flight budget is being filled and ACK/retire progress is continuous, stop tuning this PR. Treat the result as evidence that the next limiter is elsewhere.

---

## PR 2 — R1 large-record measurement closeout

### Question

Does increasing record size actually amortize a fixed per-record cost, and where does the benefit stop?

The implementation already exposes the measurement-gated record tiers:

- 1170 bytes (existing small-record reference);
- 4096 bytes;
- 14336 bytes;
- 28672 bytes.

This PR should prefer measurement and evidence capture over new transport behavior.

### Minimum matrix

For each tier, under the same host pair / route / payload total / build:

- application throughput;
- wall-clock duration;
- records per second;
- CPU time or CPU utilization when reproducibly available;
- send/receive syscall counts when reproducibly available;
- encryption/seal/open calls per transferred MiB when cheaply observable;
- p50/p95 record completion latency if the existing instrumentation can expose it without perturbing the test materially.

### Interpretation rule

The result must distinguish at least these cases:

- **throughput scales roughly with record size:** strong evidence for a per-record tax;
- **CPU drops but throughput does not rise:** path or pacing dominates;
- **neither CPU nor throughput changes materially:** the hypothesized record tax is not the active limiter;
- **large tiers regress or destabilize:** keep the optimization research-only and investigate framing/buffering effects before any wider use.

Do not promote a large tier into a default path from this PR.

---

## PR 3 — isolate the UDP ~227 ms/record secondary limiter

### Question

Where does the extra per-record lifetime appear on the 78 ms mesh path after D074 removed the local turn-structure bottleneck?

This PR is instrumentation-first. It should avoid changing cwnd, Reno rules, retransmission thresholds, ACK policy, or record size.

### Lifecycle stamps

Capture monotonic relative timestamps or aggregate histograms for the closest practical equivalents of:

```text
queue/admit
seal complete
send syscall
peer receive
peer ACK generation
peer ACK send
local ACK receive
record retire
```

If all eight points cannot be observed without invasive changes, prefer a smaller trustworthy set over speculative timestamps.

### Privacy / evidence rule

- no production addresses;
- no peer target logging beyond the repository's existing redaction policy;
- aggregate timing buckets are preferred;
- record exact commit/build identity and test topology class;
- do not infer a cause merely because one timestamp is absent.

### Success condition

A successful PR narrows the unexplained latency to one or two stages strongly enough to justify the next optimization hypothesis. A typed inconclusive result is acceptable if the instrumentation cannot separate the stages without materially changing behavior.

---

## PR 4 — UDP multistream N=2 probe only

### Question

Is the remaining UDP bottleneck per-flow/per-socket enough that one additional independent stream yields useful parallelism, or is the path dominated by a shared queue/resource?

### Scope

Implement only the smallest useful probe:

```text
--streams 1|2
```

Each stream must retain independent state where D074 requires it, including its own socket / recovery / congestion-control / secure-session state. Do not change wire semantics merely to support the probe.

### Decision rule

Use the single-stream controlled baseline from the same build and path.

- If N=2 delivers at least ~70% of ideal linear gain under the controlled probe, a later PR may justify the N=4/8 matrix.
- If N=2 is heavily discounted, stop. First identify the shared limiter (path queue, gateway PPS, server loop, CPU, or another shared resource).

The ~70% value is a scheduling threshold for whether a larger matrix is worth engineering effort, not a protocol performance promise.

---

## Explicitly deferred while this sequence is open

Unless new evidence changes priority, do **not** use this sequence as justification to add:

- new experimental carriers;
- UDP+TCP aggregation/striping;
- new camouflage/DPI-evasion behavior;
- FEC expansion beyond current bounded research state;
- protocol freeze or release claims.

Those are separate questions. The immediate goal is to make the existing TCP/UDP + SOCKS path explainable before expanding the feature surface.

---

## Recommended execution order

1. D075 SOCKS bounded ACK-driven pipeline;
2. R1 large-record measurement closeout;
3. UDP secondary-limiter timing isolation;
4. UDP N=2 multistream probe;
5. only then decide whether N=4/8, further ACK/pacing work, or another path-specific change is justified.

This ordering is intentionally evidence-driven: each PR should either improve one known bottleneck or falsify the hypothesis that motivated it.
