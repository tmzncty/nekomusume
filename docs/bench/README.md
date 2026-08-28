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


## Privileged netns/netem matrix

`scripts/bench/run-netns.sh` creates two uniquely named temporary namespaces and
one veth pair, runs baseline, 20 ms RTT, random 1/5/10% loss, burst loss,
reorder, 10 Mbit rate and 100% blackhole cases, writes JSON, and removes both
namespaces through an EXIT/INT/TERM trap. The committed execution evidence is
`latest-netns.json`; its current run has nine scenarios, zero harness failures,
0.050 ms median measured ping RTT and 10.054 ms P95. These are isolated ICMP/netem
measurements, not Nekomusume throughput or HY2 comparison results.

## Controlled HY2 comparison boundary

`scripts/bench/compare-hy2.sh` is deliberately fail-closed. It requires exact
commands for both implementations, a controlled server and route/window ID,
equal MTU/security/load metadata, at least three runs, `NEKO_ISOLATED_LAB=yes`,
and explicit command-evaluation permission. No HY2 comparison was executed: no
controlled HY2 endpoint/command and matching Nekomusume application benchmark
were supplied. The scaffold must not be cited as comparative evidence.
