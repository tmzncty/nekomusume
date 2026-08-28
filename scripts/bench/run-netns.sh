#!/usr/bin/env bash
set -euo pipefail
# Privileged, isolated two-namespace lab. It changes only namespaces/interfaces
# bearing this script's unique prefix and always removes them through trap.
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
RUN_ID="${NEKO_NETNS_RUN_ID:-$$}"
A="neko-a-${RUN_ID}"; B="neko-b-${RUN_ID}"
VA="nva-${RUN_ID}"; VB="nvb-${RUN_ID}"
OUT=${1:-"$ROOT/docs/bench/latest-netns.json"}
cleanup() {
  sudo -n ip netns del "$A" 2>/dev/null || true
  sudo -n ip netns del "$B" 2>/dev/null || true
}
trap cleanup EXIT INT TERM
cleanup
sudo -n ip netns add "$A"; sudo -n ip netns add "$B"
sudo -n ip link add "$VA" type veth peer name "$VB"
sudo -n ip link set "$VA" netns "$A"; sudo -n ip link set "$VB" netns "$B"
sudo -n ip -n "$A" addr add 10.203.0.1/30 dev "$VA"
sudo -n ip -n "$B" addr add 10.203.0.2/30 dev "$VB"
sudo -n ip -n "$A" link set lo up; sudo -n ip -n "$B" link set lo up
sudo -n ip -n "$A" link set "$VA" up; sudo -n ip -n "$B" link set "$VB" up
TMP=$(mktemp); trap 'rm -f "$TMP"; cleanup' EXIT INT TERM
run_case() {
  local name=$1 qdisc=$2 expect=$3
  sudo -n ip netns exec "$A" tc qdisc del dev "$VA" root 2>/dev/null || true
  if [ "$qdisc" != none ]; then sudo -n ip netns exec "$A" tc qdisc add dev "$VA" root netem $qdisc; fi
  local start end output rc received transmitted loss avg
  start=$(date +%s%N); set +e
  output=$(sudo -n ip netns exec "$A" ping -n -q -c 20 -W 1 10.203.0.2 2>&1); rc=$?
  set -e; end=$(date +%s%N)
  transmitted=$(printf '%s\n' "$output" | sed -nE 's/^([0-9]+) packets transmitted.*/\1/p' | head -1); transmitted=${transmitted:-0}
  received=$(printf '%s\n' "$output" | sed -nE 's/^[0-9]+ packets transmitted, ([0-9]+) received.*/\1/p' | head -1); received=${received:-0}
  loss=$(printf '%s\n' "$output" | sed -nE 's/.* ([0-9.]+)% packet loss.*/\1/p' | head -1); loss=${loss:-100}
  avg=$(printf '%s\n' "$output" | sed -nE 's#^rtt .* = [0-9.]+/([0-9.]+)/.*#\1#p' | head -1); avg=${avg:-null}
  local pass=false
  if [ "$expect" = success ] && [ "$rc" -eq 0 ]; then pass=true; fi
  if [ "$expect" = blackhole ] && [ "$received" -eq 0 ]; then pass=true; fi
  jq -nc --arg name "$name" --arg qdisc "$qdisc" --argjson tx "$transmitted" --argjson rx "$received" --argjson loss "$loss" --arg avg "$avg" --argjson elapsed "$(( (end-start)/1000 ))" --argjson pass "$pass" '{name:$name,qdisc:$qdisc,transmitted:$tx,received:$rx,loss_percent:$loss,avg_rtt_ms:(if $avg=="null" then null else ($avg|tonumber) end),elapsed_us:$elapsed,pass:$pass}' >> "$TMP"
}
run_case baseline none success
run_case rtt-20ms 'delay 10ms' success
run_case loss-1pct 'loss 1%' success
run_case loss-5pct 'loss 5%' success
run_case loss-10pct 'loss 10%' success
run_case burst-loss 'loss gemodel 5% 50% 90% 1%' success
run_case reorder 'delay 10ms reorder 25% 50%' success
run_case bandwidth-10mbit 'rate 10mbit' success
run_case blackhole 'loss 100%' blackhole
jq -sc '{schema:"nekomusume.netns-bench.v0",mode:"privileged-isolated-netns",network_scope:"two temporary namespaces and one veth pair",samples:.,summary:{scenarios:length,failures:([.[]|select(.pass|not)]|length),median_rtt_ms:([.[].avg_rtt_ms|select(.!=null)]|sort|if length==0 then null else .[length/2|floor] end),p95_rtt_ms:([.[].avg_rtt_ms|select(.!=null)]|sort|if length==0 then null else .[((length-1)*0.95|round)] end)}}' "$TMP" > "$OUT"
jq -e '.summary.failures==0' "$OUT" >/dev/null
cat "$OUT"
