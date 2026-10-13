#!/usr/bin/env bash
# Reproducible local lab for D076 causal hypotheses.
#
# This lab is OPTIONAL and requires Linux, root, iproute2/tc and iperf3.
# It does not modify the production protocol. It builds three temporary network
# namespaces and lets us reproduce two important confounders:
#   1) time-window recovery while the test sequence is running;
#   2) source-port dependent fast/slow treatment (toy ECMP/classifier analogue).
#
# Usage:
#   sudo bash tools/d076_netns_lab.sh setup
#   sudo bash tools/d076_netns_lab.sh time-window
#   sudo bash tools/d076_netns_lab.sh port-sweep
#   sudo bash tools/d076_netns_lab.sh cleanup
set -euo pipefail

SRC=neko-d076-src
MID=neko-d076-mid
DST=neko-d076-dst

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing dependency: $1" >&2
    exit 1
  }
}

require_root() {
  if [[ ${EUID:-$(id -u)} -ne 0 ]]; then
    echo "run as root (for ip netns/tc)" >&2
    exit 1
  fi
}

cleanup() {
  ip netns del "$SRC" 2>/dev/null || true
  ip netns del "$MID" 2>/dev/null || true
  ip netns del "$DST" 2>/dev/null || true
}

setup() {
  cleanup

  ip netns add "$SRC"
  ip netns add "$MID"
  ip netns add "$DST"

  ip link add s0 type veth peer name m0
  ip link add m1 type veth peer name d0
  ip link set s0 netns "$SRC"
  ip link set m0 netns "$MID"
  ip link set m1 netns "$MID"
  ip link set d0 netns "$DST"

  ip -n "$SRC" addr add 10.76.0.1/24 dev s0
  ip -n "$MID" addr add 10.76.0.254/24 dev m0
  ip -n "$MID" addr add 10.76.1.254/24 dev m1
  ip -n "$DST" addr add 10.76.1.1/24 dev d0

  ip -n "$SRC" link set lo up
  ip -n "$MID" link set lo up
  ip -n "$DST" link set lo up
  ip -n "$SRC" link set s0 up
  ip -n "$MID" link set m0 up
  ip -n "$MID" link set m1 up
  ip -n "$DST" link set d0 up

  ip -n "$SRC" route add default via 10.76.0.254
  ip -n "$DST" route add default via 10.76.1.254
  ip netns exec "$MID" sysctl -q -w net.ipv4.ip_forward=1

  # Default laboratory path: 200 Mbps, 80 ms RTT-ish (40 ms each direction).
  tc -n "$SRC" qdisc add dev s0 root netem delay 40ms
  tc -n "$DST" qdisc add dev d0 root netem delay 40ms
  set_mid_rate 200mbit

  start_server
  echo "D076 lab ready: 10.76.0.1 -> 10.76.1.1"
}

start_server() {
  ip netns exec "$DST" pkill -x iperf3 2>/dev/null || true
  ip netns exec "$DST" iperf3 -s -D
}

set_mid_rate() {
  local rate=$1
  tc -n "$MID" qdisc del dev m1 root 2>/dev/null || true
  tc -n "$MID" qdisc add dev m1 root handle 1: htb default 10
  tc -n "$MID" class add dev m1 parent 1: classid 1:10 htb rate "$rate" ceil "$rate"
}

probe() {
  local streams=$1
  echo "--- N=$streams ---"
  ip netns exec "$SRC" iperf3 -c 10.76.1.1 -t 8 -P "$streams" --get-server-output
}

time_window() {
  echo "# time-window confound demo"
  echo "# First two probes are path-limited; path recovers before N=4."
  set_mid_rate 600kbit
  probe 1
  probe 2
  set_mid_rate 200mbit
  probe 4
  probe 8

  echo
  echo "# A/B/A check after recovery: later N=1 should now also be fast."
  probe 1
}

configure_port_classes() {
  # Rebuild qdisc with a slow default class and a fast class. Selected source ports
  # receive the fast class. This is not a literal ECMP implementation; it is a
  # controlled analogue for testing the inference "flow identity matters".
  tc -n "$MID" qdisc del dev m1 root 2>/dev/null || true
  tc -n "$MID" qdisc add dev m1 root handle 1: htb default 10
  tc -n "$MID" class add dev m1 parent 1: classid 1:10 htb rate 600kbit ceil 600kbit
  tc -n "$MID" class add dev m1 parent 1: classid 1:20 htb rate 200mbit ceil 200mbit

  # Mark a few deterministic "lucky" source ports as fast.
  for p in 30002 30006 30010; do
    tc -n "$MID" filter add dev m1 protocol ip parent 1: prio 10 flower \
      ip_proto tcp src_ip 10.76.0.1 src_port "$p" dst_ip 10.76.1.1 dst_port 5201 \
      classid 1:20
  done
}

port_sweep() {
  configure_port_classes
  echo "# source-port sweep: stable fast/slow identities should appear"
  for p in $(seq 30000 30011); do
    echo "--- source port $p ---"
    ip netns exec "$SRC" iperf3 -c 10.76.1.1 -t 4 --cport "$p" || true
  done
}

main() {
  require_root
  need ip
  need tc
  need iperf3

  case "${1:-}" in
    setup) setup ;;
    time-window) time_window ;;
    port-sweep) port_sweep ;;
    cleanup) cleanup ;;
    *)
      echo "usage: $0 {setup|time-window|port-sweep|cleanup}" >&2
      exit 2
      ;;
  esac
}

main "$@"
