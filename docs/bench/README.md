# Isolated benchmark harness

`../scripts/bench/run-isolated.sh` produces a machine-readable JSON summary for
bounded deterministic recovery fixtures. It does **not** create netns/veth,
change routes/firewalls, open sockets, contact WAN endpoints, probe third
parties, or compare Hysteria2. It covers baseline, modeled RTT, deterministic
1/5/10% first-send loss, reversal/reorder, modeled bandwidth and the carrier
blackhole recovery fixture. Each result includes scenario parameters,
frames/delivery, retransmission count, rounds, failures, median and P95 timing.

The timing fields measure local harness execution and are not network latency.
They must not be presented as throughput or WAN performance. A future Linux
lab harness may add explicit netns/veth/`tc netem` scenarios while keeping its
artifacts separate from this deterministic fixture.
