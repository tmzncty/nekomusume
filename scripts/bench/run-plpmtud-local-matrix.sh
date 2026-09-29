#!/usr/bin/env bash
# PLPMTUD local netns+tc matrix (D067) — bounded, self-owned, host-local only.
#
# Grid: family(v4/v6) x path(router-icmp at base/mid/clean, silent-blackhole at
# 1400) x size-selective loss(0/1/5%) x session(short/long), plus a DF-off
# control cell and the IPv6x1278 N/A cell.
#
# Design notes (recorded in the summary):
# - Loss is SIZE-SELECTIVE (netem only on packets above the traffic base: v4 IP
#   >1278, v6 IP >1298), one direction (A->B, the probe direction), so the
#   Session layer (handshake/echo, all <= base size) is not the failure source.
#   Ack loss is observationally equivalent to probe loss (timeout+retry).
# - Binary is FROZEN for the whole matrix (and the later VPS run): the script
#   refuses to run unless the sha matches FROZEN_SHA.
# - Known wart of the frozen binary: the v6 search base is 1278 (not 1298); the
#   search still converges to the true MTU, with a few extra probes.
# - FAIL (as opposed to recorded) means a false success: a DF-on cell confirming
#   an MTU above the path's true limit.
set -euo pipefail

REPO=$(cd "$(dirname "$0")/../.." && pwd)
BIN=${BIN:-$REPO/target/release/neko-cli}
FROZEN_SHA=b443ad9905c2657fcbc756ff35a8280e8e7550419a5f509f24a9b7a7e4e4339b
OUT=${1:-/tmp/plpmtud-matrix-$(date +%Y%m%d-%H%M%S)}
PORT_V4=40090
PORT_V6=40091
SUDO="sudo -n"
CONFIGS=${CONFIGS:-"v4-rc1278 v4-rc1400 v4-rc1500 v4-sb1400 v6-rc1298 v6-rc1350 v6-rc1500 v6-sb1400"}
LOSSES=${LOSSES:-"0 1 5"}
SESSIONS=${SESSIONS:-"short long"}
SMOKE=${SMOKE:-0}

# ---- preflight ---------------------------------------------------------------
sha=$(sha256sum "$BIN" | awk '{print $1}')
if [ "$sha" != "$FROZEN_SHA" ] && [ "${FORCE:-0}" != 1 ]; then
  echo "binary sha $sha != frozen $FROZEN_SHA; refusing (FORCE=1 to override)" >&2
  exit 2
fi
$SUDO true || { echo "sudo -n unavailable" >&2; exit 2; }
for t in ip tc python3 timeout; do
  command -v "$t" >/dev/null || [ -x "/sbin/$t" ] || [ -x "/usr/sbin/$t" ] || { echo "missing $t" >&2; exit 2; }
done
mkdir -p "$OUT/cells"
echo "{\"binary_sha\":\"$sha\",\"kernel\":\"$(uname -r)\",\"started\":\"$(date -u +%FT%TZ)\",\"cells\":[" > "$OUT/summary.json"

TAG=plm$$
NS_A=a-$TAG; NS_R=r-$TAG; NS_B=b-$TAG
E0=e0-$TAG; E1=e1-$TAG; E2=e2-$TAG; E3=e3-$TAG
cleanup() {
  $SUDO ip netns pids "$NS_B" 2>/dev/null | xargs -r $SUDO kill -9 2>/dev/null || true
  for n in "$NS_A" "$NS_R" "$NS_B"; do $SUDO ip netns del "$n" 2>/dev/null || true; done
}
trap cleanup EXIT

# setup_topo <family> <bottleneck_mtu|-> <silent_threshold|->
setup_topo() {
  local fam=$1 mtu=$2 silent=$3
  RT6=0
  cleanup
  for n in "$NS_A" "$NS_R" "$NS_B"; do
    $SUDO ip netns add "$n"; $SUDO ip -n "$n" link set lo up
  done
  $SUDO ip link add "$E0" type veth peer name "$E1"
  $SUDO ip link add "$E2" type veth peer name "$E3"
  $SUDO ip link set "$E0" netns "$NS_A"; $SUDO ip link set "$E1" netns "$NS_R"
  $SUDO ip link set "$E2" netns "$NS_R"; $SUDO ip link set "$E3" netns "$NS_B"
  [ "$mtu" != - ] && $SUDO ip -n "$NS_R" link set "$E2" mtu "$mtu"
  if [ "$fam" = v4 ]; then
    $SUDO ip -n "$NS_A" addr add 10.91.1.1/24 dev "$E0"
    $SUDO ip -n "$NS_R" addr add 10.91.1.2/24 dev "$E1"
    $SUDO ip -n "$NS_R" addr add 10.91.2.1/24 dev "$E2"
    $SUDO ip -n "$NS_B" addr add 10.91.2.2/24 dev "$E3"
    B_ADDR=10.91.2.2; LOSS_PROTO=ip; LOSS_MATCH="cmp(u16 at 2 layer network gt 1278)"
    LOSS_MATCH6="cmp(u16 at 4 layer network gt 1250)"; SB_PROTO=ip; SB_MATCH="cmp(u16 at 2 layer network gt 1400)"
    ROUTE_A="10.91.2.0/24 via 10.91.1.2"; ROUTE_B="10.91.1.0/24 via 10.91.2.1"
  else
    $SUDO ip -n "$NS_A" addr add fd91:1::1/64 dev "$E0" nodad
    $SUDO ip -n "$NS_R" addr add fd91:1::2/64 dev "$E1" nodad
    $SUDO ip -n "$NS_R" addr add fd91:2::1/64 dev "$E2" nodad
    $SUDO ip -n "$NS_B" addr add fd91:2::2/64 dev "$E3" nodad
    B_ADDR=fd91:2::2; LOSS_PROTO=ipv6; LOSS_MATCH="cmp(u16 at 4 layer network gt 1250)"
    LOSS_MATCH6="cmp(u16 at 4 layer network gt 1250)"; SB_PROTO=ipv6; SB_MATCH="cmp(u16 at 4 layer network gt 1360)"
    ROUTE_A="fd91:2::/64 via fd91:1::2"; ROUTE_B="fd91:1::/64 via fd91:2::1"; RT6=1
  fi
  for p in "$NS_A $E0" "$NS_R $E1" "$NS_R $E2" "$NS_B $E3"; do
    set -- $p; $SUDO ip -n "$1" link set "$2" up
  done
  if [ "${RT6:-0}" = 1 ]; then
    $SUDO ip -n "$NS_A" -6 route add $ROUTE_A
    $SUDO ip -n "$NS_B" -6 route add $ROUTE_B
  else
    $SUDO ip -n "$NS_A" route add $ROUTE_A
    $SUDO ip -n "$NS_B" route add $ROUTE_B
  fi
  $SUDO ip netns exec "$NS_R" sysctl -qw net.ipv4.ip_forward=1 net.ipv6.conf.all.forwarding=1
  # e2 (A->B egress at R): prio root; band 2 = netem loss (size-selective via
  # filters); silent drop filter (if any) has the highest priority.
  $SUDO ip netns exec "$NS_R" tc qdisc add dev "$E2" root handle 1: prio bands 3
  $SUDO ip netns exec "$NS_R" tc qdisc add dev "$E2" parent 1:2 handle 20: netem loss 0%
  if [ "$silent" != - ]; then
    $SUDO ip netns exec "$NS_R" tc filter add dev "$E2" parent 1: protocol "$SB_PROTO" prio 1 basic match "$SB_MATCH" action drop
  fi
  $SUDO ip netns exec "$NS_R" tc filter add dev "$E2" parent 1: protocol "$LOSS_PROTO" prio 2 basic match "$LOSS_MATCH" flowid 1:2
  $SUDO ip netns exec "$NS_R" tc filter add dev "$E2" parent 1: protocol "$LOSS_PROTO" prio 3 basic match "cmp(u16 at 2 layer network gt 0)" flowid 1:1 2>/dev/null || \
    $SUDO ip netns exec "$NS_R" tc filter add dev "$E2" parent 1: protocol "$LOSS_PROTO" prio 3 basic match "cmp(u16 at 4 layer network gt 0)" flowid 1:1
}

set_loss() { # <pct>
  $SUDO ip netns exec "$NS_R" tc qdisc change dev "$E2" parent 1:2 handle 20: netem loss "$1%"
}

wait_no_pids() {
  for _ in $(seq 20); do
    [ -z "$($SUDO ip netns pids "$NS_B" 2>/dev/null)" ] && return 0
    sleep 0.5
  done
  return 1
}

# run_cell <config> <family> <bottleneck> <blackhole> <loss> <session> <count> <dur> <port>
run_cell() {
  local cfg=$1 fam=$2 bneck=$3 bh=$4 loss=$5 sess=$6 count=$7 dur=$8 port=$9
  local cell="$cfg-l$loss-$sess"
  local dir="$OUT/cells/$cell"; mkdir -p "$dir"
  set_loss "$loss"
  # identities per config (reused across its cells)
  local sid="$OUT/id-srv-$cfg" cid="$OUT/id-cli-$cfg"
  if [ ! -f "$sid" ]; then "$BIN" keygen --identity "$sid" >/dev/null; fi
  if [ ! -f "$cid" ]; then "$BIN" keygen --identity "$cid" >/dev/null; fi
  local sk ck
  sk=$("$BIN" keygen --identity "$sid" | sed 's/^client_public_key=//')
  ck=$("$BIN" keygen --identity "$cid" | sed 's/^client_public_key=//')
  local bind addr
  if [ "$fam" = v4 ]; then bind="$B_ADDR:$port"; addr="$B_ADDR:$port"; else bind="[$B_ADDR]:$port"; addr="[$B_ADDR]:$port"; fi
  wait_no_pids
  $SUDO ip netns exec "$NS_B" "$BIN" server --transport udp --port "$port" \
    --bind "$bind" --identity "$sid" \
    --client-key "$ck" --duration 30 --plpmtud \
    >"$dir/server.log" 2>&1 &
  local ready=""
  for _ in $(seq 20); do
    grep -q "lifecycle_state=READY readiness=true" "$dir/server.log" 2>/dev/null && { ready=1; break; }
    sleep 0.5
  done
  if [ -z "$ready" ]; then
    echo "server not ready in $cell" | tee -a "$dir/note.txt"
  fi
  local icmp_before icmp6_before
  icmp_before=$($SUDO ip netns exec "$NS_A" nstat -az IcmpInDestUnreachs 2>/dev/null | awk '/Unreachs/{print $2}')
  icmp6_before=$($SUDO ip netns exec "$NS_A" nstat -az Icmp6InPktTooBigs 2>/dev/null | awk '/TooBig/{print $2}')
  local t0=$SECONDS rc
  timeout 45 $SUDO ip netns exec "$NS_A" "$BIN" client --transport udp --port "$port" \
    --addr "$addr" --server-key "$sk" --identity "$cid" \
    --count "$count" --bytes 32 --duration "$dur" --plpmtud --diagnostic \
    --experiment-id "plpmtud-$cell" >"$dir/client.log" 2>&1 || rc=$?
  rc=${rc:-0}
  local elapsed=$((SECONDS - t0))
  local icmp_after icmp6_after icmp_delta
  icmp_after=$($SUDO ip netns exec "$NS_A" nstat -az IcmpInDestUnreachs 2>/dev/null | awk '/Unreachs/{print $2}')
  icmp6_after=$($SUDO ip netns exec "$NS_A" nstat -az Icmp6InPktTooBigs 2>/dev/null | awk '/TooBig/{print $2}')
  icmp_delta=$(( ${icmp_after:-0} - ${icmp_before:-0} ))
  $SUDO ip netns pids "$NS_B" 2>/dev/null | xargs -r $SUDO kill -TERM 2>/dev/null || true
  wait_no_pids || true
  python3 - "$dir" "$cell" "$cfg" "$fam" "$bneck" "$bh" "$loss" "$sess" "$rc" "$elapsed" "$icmp_delta" \
    "${icmp6_before:-0}" "${icmp6_after:-0}" "$OUT/summary.json" <<'PY'
import json, re, sys
(d, cell, cfg, fam, bneck, bh, loss, sess, rc, elapsed, icmp4d, i6b, i6a, sj) = sys.argv[1:15]
log = open(f"{d}/client.log", errors="replace").read()
def count(ev): return len(re.findall(f'"event":"{ev}"', log))
conv = re.search(r"plpmtud_converged .*confirmed_mtu=(\d+)", log)
confirmed = int(conv.group(1)) if conv else None
sent, acked, to = count("plpmtud_probe_sent"), count("plpmtud_probe_acked"), count("plpmtud_probe_timeout_retry") + count("plpmtud_probe_timeout_lowered_bound")
deadline = '"event":"plpmtud_deadline"' in log
bneck_i = int(bneck)
false_success = confirmed is not None and confirmed > bneck_i
handshake_fail = rc != 0 and "probe_ok" not in log
verdict = ("FAIL_FALSE_SUCCESS" if false_success else
           "converged_exact" if confirmed == bneck_i else
           "converged_lower" if confirmed is not None else
           "deadline_bounded" if deadline else
           "handshake_or_exchange_failed" if handshake_fail else
           "client_exit_%s" % rc)
cell_json = {"cell": cell, "config": cfg, "family": fam, "bottleneck": bneck_i,
             "blackhole": bh, "loss_pct": int(loss), "session": sess,
             "exit": int(rc), "elapsed_s": int(elapsed), "converged": confirmed,
             "probes_sent": sent, "probes_acked": acked, "probe_timeouts": to,
             "deadline": deadline, "icmp4_delta": int(icmp4d),
             "icmp6_delta": int(i6a) - int(i6b), "verdict": verdict}
with open(sj, "a") as f:
    f.write(json.dumps(cell_json) + ",\n")
print(f"{cell}: exit={rc} conv={confirmed} sent/ack/to={sent}/{acked}/{to} "
      f"deadline={deadline} icmp4+{icmp4d} -> {verdict} [{elapsed}s]")
PY
}

# ---- main grid ---------------------------------------------------------------
run_config() { # <cfg>
  local cfg=$1
  local fam mtu silent bneck bh port
  case $cfg in
    v4-rc1278) fam=v4; mtu=1278; silent=-; bneck=1278; bh=router-icmp; port=$PORT_V4 ;;
    v4-rc1400) fam=v4; mtu=1400; silent=-; bneck=1400; bh=router-icmp; port=$PORT_V4 ;;
    v4-rc1500) fam=v4; mtu=-;  silent=-; bneck=1500; bh=none;          port=$PORT_V4 ;;
    v4-sb1400) fam=v4; mtu=-;  silent=1400; bneck=1400; bh=silent;    port=$PORT_V4 ;;
    v6-rc1298) fam=v6; mtu=1298; silent=-; bneck=1298; bh=router-icmp; port=$PORT_V6 ;;
    v6-rc1350) fam=v6; mtu=1350; silent=-; bneck=1350; bh=router-icmp; port=$PORT_V6 ;;
    v6-rc1500) fam=v6; mtu=-;  silent=-; bneck=1500; bh=none;          port=$PORT_V6 ;;
    v6-sb1400) fam=v6; mtu=-;  silent=1400; bneck=1400; bh=silent;    port=$PORT_V6 ;;
    *) echo "unknown config $cfg" >&2; exit 2 ;;
  esac
  echo "== topo $cfg (family=$fam mtu=${mtu/-/1500} silent=${silent/-/none}) =="
  setup_topo "$fam" "$mtu" "$silent"
  for loss in $LOSSES; do
    for sess in $SESSIONS; do
      if [ "$sess" = short ]; then c=2; d=2; else c=8; d=5; fi
      run_cell "$cfg" "$fam" "$bneck" "$bh" "$loss" "$sess" "$c" "$d" "$port"
    done
  done
}

if [ "$SMOKE" = 1 ]; then
  CONFIGS="v4-rc1400"; LOSSES="0"; SESSIONS="short"
fi

for cfg in $CONFIGS; do run_config "$cfg"; done

# ---- DF-off control cell (v4, router-icmp 1278, loss 0) ----------------------
control_df() {
  echo "== control: DF-off on a 1278 router-icmp path =="
  setup_topo v4 1278 -
  local dir="$OUT/cells/ctrl-df-off"; mkdir -p "$dir"
  cat > "$dir/dfx.py" <<'PY'
import socket, sys, errno
fam = socket.AF_INET6 if sys.argv[1] == '6' else socket.AF_INET
dst, size, mode = sys.argv[2], int(sys.argv[3]), sys.argv[4]
s = socket.socket(fam, socket.SOCK_DGRAM)
if mode == 'probe':
    opt = 23 if fam == socket.AF_INET6 else 10
    s.setsockopt(socket.IPPROTO_IPV6 if fam == socket.AF_INET6 else socket.IPPROTO_IP, opt, 3)
try:
    s.sendto(b'x' * size, (dst, 40099)); print('sent')
except OSError as e:
    print('ERR', errno.errorcode.get(e.errno, e.errno))
PY
  cat > "$dir/dfr.py" <<'PY'
import socket
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s.bind(('0.0.0.0', 40099)); s.settimeout(1.5)
try:
    d, _ = s.recvfrom(9000); print('recv', len(d))
except socket.timeout:
    print('timeout')
PY
  local res="" size outcome
  for size in 1389 1445 1500; do
    $SUDO ip netns exec "$NS_B" python3 "$dir/dfr.py" > "$dir/rx-$size.out" 2>&1 &
    local rp=$!; sleep 0.3
    $SUDO ip netns exec "$NS_A" python3 "$dir/dfx.py" 4 "$B_ADDR" $((size - 28)) default > "$dir/tx1-$size.out"
    sleep 0.3
    $SUDO ip netns exec "$NS_A" python3 "$dir/dfx.py" 4 "$B_ADDR" $((size - 28)) default > "$dir/tx2-$size.out"
    wait $rp
    local first second
    first=$(cat "$dir/tx1-$size.out"); second=$(cat "$dir/tx2-$size.out"); outcome=$(cat "$dir/rx-$size.out")
    res="$res\n  size $size: send1=$first send2=$second rx=$outcome"
    sleep 0.3
  done
  # DF-on (PROBE) confirmation on the same path: never delivered.
  $SUDO ip netns exec "$NS_B" python3 "$dir/dfr.py" > "$dir/rx-probe.out" 2>&1 &
  local rp=$!; sleep 0.3
  $SUDO ip netns exec "$NS_A" python3 "$dir/dfx.py" 4 "$B_ADDR" 1472 probe > "$dir/tx-probe.out"
  wait $rp
  python3 - "$res" "$(cat "$dir/tx-probe.out")" "$(cat "$dir/rx-probe.out")" "$OUT/summary.json" <<'PY'
import json, sys
res, txp, rxp, sj = sys.argv[1:5]
false_success = all("recv" in l for l in res.splitlines() if "rx=" in l)
probe_blocked = "timeout" in rxp
control = {"cell": "ctrl-df-off", "path": "v4 router-icmp 1278",
           "df_off_second_send_delivered": false_success,
           "df_on_probe_delivered": not probe_blocked,
           "verdict": "PASS" if (false_success and probe_blocked) else "FAIL",
           "detail": res, "df_on_send": txp, "df_on_rx": rxp}
with open(sj, "a") as f:
    f.write(json.dumps(control) + ",\n")
print("control df-off:", control["verdict"])
print(res)
print("  df-on(PROBE) 1500B:", txp, "=>", rxp)
PY
}
control_df

# ---- IPv6 x 1278 N/A cell -----------------------------------------------------
na_v6_1278() {
  echo "== N/A: IPv6 over a 1278 link =="
  setup_topo v6 1278 - 2>/dev/null || true
  local dir="$OUT/cells/na-v6-1278"; mkdir -p "$dir"
  local addrs rc_out
  # The authoritative fact: a link below 1280 loses its IPv6 addresses on the
  # router side of the bottleneck (E2 in NS_R), so no v6 route survives.
  addrs=$($SUDO ip -n "$NS_R" -6 addr show dev "$E2" scope global 2>/dev/null | grep -c inet6 || true)
  rc_out=$($SUDO ip netns exec "$NS_A" python3 -c "
import socket
try:
    s = socket.socket(socket.AF_INET6, socket.SOCK_DGRAM)
    s.sendto(b'x' * 100, ('fd91:2::2', 40099)); print('sent')
except OSError as e:
    print('ERR', e.errno)
" 2>&1)
  python3 - "$addrs" "$rc_out" "$OUT/summary.json" <<'PY'
import json, sys
addrs, rc, sj = sys.argv[1:4]
na = {"cell": "na-v6-1278", "family": "v6", "bottleneck": 1278,
      "v6_addrs_on_bottleneck": int(addrs), "send_result": rc,
      "verdict": "N/A" if int(addrs) == 0 else "UNEXPECTED"}
with open(sj, "a") as f:
    f.write(json.dumps(na) + ",\n")
print("na v6x1278:", na["verdict"], f"(bottleneck v6 addrs={addrs}, send={rc.strip()})")
PY
}
na_v6_1278

# ---- finalize summary ---------------------------------------------------------
python3 - "$OUT/summary.json" "$OUT/summary.md" "$FROZEN_SHA" <<'PY'
import json, sys
sj, sm, sha = sys.argv[1:4]
raw = open(sj).read().rstrip(",\n") + "]}"
data = json.loads(raw)
cells = [c for c in data["cells"] if c.get("cell", "").startswith(("v4-", "v6-"))]
ctrl = next((c for c in data["cells"] if c["cell"] == "ctrl-df-off"), None)
na = next((c for c in data["cells"] if c["cell"] == "na-v6-1278"), None)
fails = [c for c in cells if c["verdict"].startswith("FAIL")]
lines = [f"# PLPMTUD local matrix — binary {sha[:12]}", "",
         "| cell | exit | converged | sent/ack/to | deadline | icmp | verdict |",
         "|---|---|---|---|---|---|---|"]
for c in cells:
    icmp = c.get("icmp4_delta", 0) + c.get("icmp6_delta", 0)
    lines.append(f"| {c['cell']} | {c['exit']} | {c['converged']} | "
                 f"{c['probes_sent']}/{c['probes_acked']}/{c['probe_timeouts']} | "
                 f"{c['deadline']} | {icmp} | {c['verdict']} |")
if ctrl:
    lines += ["", f"Control (DF-off, v4 1278 path): {ctrl['verdict']} — "
                  f"second send delivered={ctrl['df_off_second_send_delivered']}, "
                  f"DF-on probe delivered={ctrl['df_on_probe_delivered']}"]
if na:
    lines += ["", f"IPv6 x 1278: {na['verdict']} ({na['send_result'].strip()})"]
lines += ["", f"Cells: {len(cells)}; false-success failures: {len(fails)}",
          "Loss is size-selective (v4 IP>1278 / v6 IP>1298), A->B direction only; "
          "the frozen binary's v6 search base is 1278 (converges to truth with extra probes)."]
open(sm, "w").write("\n".join(lines) + "\n")
data["cells"] = cells
data["control"] = ctrl
data["na"] = na
data["finished"] = __import__("datetime").datetime.now(__import__("datetime").timezone.utc).isoformat()
open(sj, "w").write(json.dumps(data, indent=1))
print(f"wrote {sm} and {sj}: {len(cells)} cells, {len(fails)} failures")
PY
