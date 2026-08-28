# Concurrent-carrier and heterogeneous multipath gate

**Status: explicitly disabled; no striping or aggregation implementation.**

The candidate proves sequential UDP-primary/TCP-fallback recovery, validated
migration-back, per-carrier health scoring, and an isolated netem harness. It
does not contain controlled A/B evidence that concurrent UDP+TCP improves a
target metric. The current ICMP/netem results validate the harness only; they do
not exercise simultaneous application delivery over heterogeneous paths.

Concurrent striping also lacks the required complete design:

- a connection-level data sequence independent of carrier packet numbers;
- bounded cross-path reordering and duplicate/conflict handling;
- receiver memory and gap-time limits under asymmetric RTT and blackholes;
- congestion coupling so TCP and UDP do not compete unfairly for one bottleneck;
- ACK attribution without promoting carrier feedback to Session delivery;
- retransmission ownership when a range is sent on more than one carrier;
- scheduler behavior for TCP HOL versus UDP loss/reordering;
- path-generation, key-phase and migration interactions;
- controlled comparisons against failover-only operation.

Therefore both concurrent UDP+TCP and heterogeneous multipath aggregation remain
disabled. The existing design continues to select one active carrier, with a
validated failover/migration transition. A future gate requires a socket-free
DSN/reordering/congestion-coupling model first, deterministic asymmetric-path
and blackhole tests, then isolated netns A/B results showing a declared metric
benefit without unbounded queueing, duplicate bytes, unfairness, or path flaps.

No scheduler, wire field, listener, WAN experiment, production configuration or
performance/superiority claim is introduced by this decision.
