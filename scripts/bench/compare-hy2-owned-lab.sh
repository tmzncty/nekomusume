#!/usr/bin/env bash
set -euo pipefail
fail(){ echo "compare-hy2-owned-lab: $*" >&2; exit 2; }
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
LISTENER_PARSER=$SCRIPT_DIR/parse-listener.py
. "$SCRIPT_DIR/owned-lab-control-plane.sh"
need(){ [ -n "${!1:-}" ] || fail "missing $1"; }
canonical_bool(){ case "$1" in 1|true) printf true;; 0|false) printf false;; *) printf unknown;; esac; }
for v in LAB_SSH_TARGET LAB_ENDPOINT_ID LAB_ENDPOINT_SHA256 LAB_REMOTE_ADDRESS LAB_REMOTE_BIND_ADDRESS NEKO_BIN HY2_BIN; do need "$v"; done
[ "${NEKO_OWNED_LAB:-}" = yes ] || fail 'NEKO_OWNED_LAB=yes is required'
case "$LAB_SSH_TARGET" in *[!A-Za-z0-9._-]*|'') fail 'invalid SSH target';; esac
case "$LAB_ENDPOINT_ID" in *[!A-Za-z0-9._-]*|'') fail 'invalid endpoint id';; esac
[[ "$LAB_ENDPOINT_SHA256" =~ ^[0-9a-f]{64}$ ]] || fail 'invalid endpoint SHA-256'
[[ "$LAB_REMOTE_ADDRESS" =~ ^[A-Za-z0-9.:_-]+$ ]] || fail 'invalid remote address'
[[ "$LAB_REMOTE_BIND_ADDRESS" =~ ^[0-9A-Fa-f:.]+$ ]] || fail 'invalid remote bind address'
command -v python3 >/dev/null || fail 'python3 required'
authorities_text=$(python3 - "$LAB_REMOTE_BIND_ADDRESS" "$LAB_REMOTE_ADDRESS" <<'PY'
import ipaddress, sys
try:
    address = ipaddress.ip_address(sys.argv[1])
    connect = ipaddress.ip_address(sys.argv[2])
except ValueError:
    raise SystemExit(1)
check = address.ipv4_mapped if isinstance(address, ipaddress.IPv6Address) and address.ipv4_mapped else address
if check.is_unspecified or check.is_loopback or check.is_multicast or str(check) == "255.255.255.255":
    raise SystemExit(1)
print(f"[{address}]" if address.version == 6 else address)
print(f"[{connect}]" if connect.version == 6 else connect)
PY
) || fail 'remote bind/connect address is wildcard, unspecified, loopback, multicast, broadcast, or invalid'
mapfile -t authorities <<<"$authorities_text"
[ "${#authorities[@]}" -eq 2 ] || fail 'cannot derive bind/connect authorities'
bind_authority=${authorities[0]}
connect_authority=${authorities[1]}
RUNS=${BENCH_RUNS:-5}; BYTES=${BENCH_PAYLOAD_BYTES:-1200}; TIMEOUT=${BENCH_TIMEOUT_SEC:-30}
[[ "$RUNS" =~ ^[0-9]+$ && "$RUNS" -ge 3 && "$RUNS" -le 10 ]] || fail 'BENCH_RUNS must be 3..10'
[[ "$BYTES" =~ ^[0-9]+$ && "$BYTES" -gt 0 && "$BYTES" -le 1200 ]] || fail 'BENCH_PAYLOAD_BYTES must be 1..1200'
[[ "$TIMEOUT" =~ ^[0-9]+$ && "$TIMEOUT" -gt 0 && "$TIMEOUT" -le 30 ]] || fail 'BENCH_TIMEOUT_SEC must be 1..30'
SETUP_SEC=30; READINESS_SEC=10; DIAGNOSTIC_SEC=20; CLEANUP_SEC=60
WORK_SEC=$((SETUP_SEC + RUNS * 2 * (TIMEOUT + READINESS_SEC + 2) + DIAGNOSTIC_SEC))
WHOLE_LAB_SEC=$((WORK_SEC + CLEANUP_SEC)); olcp_init_deadlines "$WORK_SEC" "$WHOLE_LAB_SEC" || fail "whole-lab budget exceeds 600s: ${WHOLE_LAB_SEC}s"
# Initialize the typed evidence context before any SSH-dependent operation.  Every
# preflight failure is therefore emitted as a schema-valid BLOCKED_HARNESS artifact.
validate_only=0; out=${1:-artifacts/hy2-owned-lab/result.json}; [ "$out" != --validate ] || { validate_only=1; out=${2:-artifacts/hy2-owned-lab/result.json}; }; case "$out" in /*|*..*) fail 'output must be relative and non-traversing';; esac
root=$(git rev-parse --show-toplevel); cd "$root"; records="$out.samples.jsonl"
runtime_template=${OWNED_LAB_RUNTIME_TEMPLATE:-/tmp/neko-hy2-owned.XXXXXXXX}
run=; remote=
if [ "$validate_only" -eq 0 ]; then
  mkdir -p "$(dirname "$out")"
  run=$(mktemp -d "$runtime_template"); remote="/tmp/$(basename "$run")"; touch "$records"
fi
early_cleanup(){ local rc=${1:-$?}; trap - EXIT INT TERM; rm -rf "$run"; return "$rc"; }
on_early_exit(){ local rc=$?; early_cleanup "$rc"; exit "$rc"; }
trap on_early_exit EXIT; trap 'exit 130' INT TERM
validator=$root/scripts/bench/validate-hy2-owned-lab.py
cleanup_done=0; cleanup_ok=0; failure_stage=preflight; local_pids=(); active_spid=; remote_started=0; remote_resources=$run/remote-resources.json
local_processes_reaped=false; local_listeners_remaining=unknown; remote_process_groups_reaped=unknown; remote_listeners_remaining=unknown; remote_temp_path_removed=unknown; payload_prepared=false
atomic_append(){ local source=$1 temporary; temporary=$(mktemp "$(dirname "$records")/.samples.XXXXXXXX"); cat "$records" "$source" >"$temporary"; mv -f "$temporary" "$records"; }
preflight_blocked(){
  failure_stage=$1
  [ "$validate_only" -eq 0 ] || fail "validation failed: $1"
  python3 "$validator" blocked --records "$records" --output "$out" --stage "$1" --commit "$(git rev-parse HEAD)" --runs "$RUNS" --bytes "$BYTES" --payload-prepared false --local-reaped unknown --local-listeners unknown --remote-reaped unknown --remote-listeners unknown --remote-path-removed unknown
  exit 2
}
ssh_bin=${OWNED_LAB_SSH_BIN:-ssh}
ssh_config=$($ssh_bin -G "$LAB_SSH_TARGET" 2>/dev/null) || preflight_blocked ssh-config
resolved=$(printf '%s\n' "$ssh_config" | awk '$1=="hostname"{print $2;exit}')
expected_user=$(printf '%s\n' "$ssh_config" | awk '$1=="user"{print $2;exit}')
[ -n "$resolved" ] || preflight_blocked ssh-config
[ -n "${LAB_SSH_EXPECTED_USER:-}" ] || preflight_blocked ssh-user-config
[ "$expected_user" = "$LAB_SSH_EXPECTED_USER" ] || preflight_blocked ssh-user-mismatch
[ "$(printf %s "$resolved" | sha256sum | awk '{print $1}')" = "$LAB_ENDPOINT_SHA256" ] || preflight_blocked ssh-endpoint-mismatch
[ "$resolved" = "$LAB_REMOTE_ADDRESS" ] || preflight_blocked ssh-endpoint-mismatch
remote_interfaces=$(ssh_bounded "$LAB_SSH_TARGET" "ip -j address show") || { rc=$?; [ "$rc" -eq 124 ] && preflight_blocked ssh-timeout; [ "$rc" -eq 255 ] && preflight_blocked ssh-auth; preflight_blocked ssh-command; }
printf '%s' "$remote_interfaces" | python3 -c 'import json,sys; expected=sys.argv[1]; data=json.load(sys.stdin); raise SystemExit(0 if any(a.get("local")==expected for i in data for a in i.get("addr_info",[])) else 1)' "$LAB_REMOTE_BIND_ADDRESS" || preflight_blocked ssh-bind-address
[ "$validate_only" -eq 0 ] || { echo validated; exit 0; }
ports=("${NEKO_PORT:-40097}" "${HY2_UDP_PORT:-40098}" "${HY2_LOCAL_PORT:-40099}" "${ECHO_PORT:-40100}")
for p in "${ports[@]}"; do [[ "$p" =~ ^[0-9]+$ && "$p" -ge 40080 && "$p" -le 40100 ]] || fail 'ports must be 40080..40100'; done
[ "$(printf '%s\n' "${ports[@]}" | sort -u | wc -l)" -eq 4 ] || fail 'ports must be distinct'
[ -x "$NEKO_BIN" ] || fail 'NEKO_BIN is not executable'; [ -x "$HY2_BIN" ] || fail 'HY2_BIN is not executable'
[ "$(sha256sum "$HY2_BIN" | awk '{print $1}')" = 66dbdb0608f25f3057b433afe975a9fc1af2ca8e512479e294988b3ef363d6c1 ] || fail 'HY2 artifact is not pinned v2.9.3'
command -v jq >/dev/null || fail 'jq required'; command -v openssl >/dev/null || fail 'openssl required'; [ -x /usr/bin/time ] || fail 'GNU time required'
failure_stage=setup
terminate_local(){
  local pid roots_file=$run/local-pids
  : >"$roots_file"
  [ -z "${active_spid:-}" ] || printf '%s\n' "$active_spid" >>"$roots_file"
  for pid in "${local_pids[@]:-}"; do [ -z "$pid" ] || printf '%s\n' "$pid" >>"$roots_file"; done
  olcp_cleanup_owned "$roots_file" 100 "${ports[@]}"
  local_processes_reaped=false; [ "$OLCP_PROCESSES_REAPED" -eq 1 ] && local_processes_reaped=true
  local_listeners_remaining=$OLCP_LISTENERS_REMAINING
  [ -z "${active_spid:-}" ] || wait "$active_spid" 2>/dev/null || true
  for pid in "${local_pids[@]:-}"; do [ -z "$pid" ] || wait "$pid" 2>/dev/null || true; done
  active_spid=; local_pids=()
  [ "$local_processes_reaped" = true ] && [ "$local_listeners_remaining" -eq 0 ]
}
cleanup(){
  local rc=${1:-$?} remote_cleanup marker remote_rc
  [ "$cleanup_done" -eq 0 ] || return "$rc"
  cleanup_done=1; cleanup_mode=1; set +e; trap - EXIT INT TERM
  terminate_local
  if [ "$remote_started" -eq 1 ]; then
    remote_cleanup=$(cat <<REMOTE
set +e
. '$remote/owned-lab-control-plane.sh'
olcp_cleanup_owned '$remote/pids' 100 '${ports[0]}' '${ports[1]}' '${ports[2]}' '${ports[3]}'
cleanup_rc=\$?
jq -s . '$remote'/*-resource.json 2>/dev/null || printf '[]'
printf '\n__CLEANUP__ %s %s\n' "\$OLCP_PROCESSES_REAPED" "\$OLCP_LISTENERS_REMAINING"
exit "\$cleanup_rc"
REMOTE
)
    ssh_bounded "$LAB_SSH_TARGET" "$remote_cleanup" >"$remote_resources.tmp" 2>/dev/null
    remote_rc=$?
    marker=$(tail -n 1 "$remote_resources.tmp" 2>/dev/null)
    sed '$d' "$remote_resources.tmp" >"$remote_resources" 2>/dev/null; rm -f "$remote_resources.tmp"
    remote_reaped_flag=$(printf '%s' "$marker" | awk '/^__CLEANUP__ [01] [0-9]+$/{print $2}')
    remote_process_groups_reaped=$(canonical_bool "$remote_reaped_flag")
    remote_listeners_remaining=$(printf '%s' "$marker" | awk '/^__CLEANUP__ [01] [0-9]+$/{print $3}')
    : "${remote_listeners_remaining:=unknown}"
    if [ "$remote_rc" -eq 0 ] && [ "$remote_process_groups_reaped" = true ] && [ "$remote_listeners_remaining" -eq 0 ]; then
      for _ in $(seq 1 20); do
        ssh_bounded "$LAB_SSH_TARGET" "rm -rf '$remote'; test ! -e '$remote'" >/dev/null 2>&1 && { remote_temp_path_removed=true; break; }
        "${OWNED_LAB_SLEEP_BIN:-sleep}" .05
      done
    fi
  else remote_process_groups_reaped=true; remote_listeners_remaining=0; remote_temp_path_removed=true; fi
  [ "$local_processes_reaped" = true ] && [ "$local_listeners_remaining" -eq 0 ] && [ "$remote_process_groups_reaped" = true ] && [ "$remote_listeners_remaining" -eq 0 ] && [ "$remote_temp_path_removed" = true ] && cleanup_ok=1
  return "$rc"
}
blocked(){
  local stage=$1; failure_stage=$stage; cleanup 0
  python3 "$validator" blocked --records "$records" --output "$out" --stage "$stage" --commit "$(git rev-parse HEAD)" --runs "$RUNS" --bytes "$BYTES" --payload-prepared "$payload_prepared" ${payload_hash:+--payload-hash "$payload_hash"} --local-reaped "$local_processes_reaped" --local-listeners "$local_listeners_remaining" --remote-reaped "$remote_process_groups_reaped" --remote-listeners "$remote_listeners_remaining" --remote-path-removed "$remote_temp_path_removed"
  [ "$cleanup_ok" -eq 0 ] || rm -rf "$run"
  echo "$out" >&2; exit 2
}
on_signal(){ blocked signal; }
on_exit(){ local rc=$?; [ "$rc" -eq 0 ] || blocked "$failure_stage"; }
trap on_exit EXIT; trap on_signal INT TERM
for p in "${ports[@]}"; do ! ss -H -lntup "sport = :$p" | grep -q . || fail "local experimental port $p occupied"; ssh_bounded "$LAB_SSH_TARGET" "! ss -H -lntup 'sport = :$p' | grep -q ." || fail "remote experimental port $p occupied"; done
payload=$run/payload.bin; dd if=/dev/zero of="$payload" bs=1 count="$BYTES" status=none; payload_prepared=true; payload_hash=$(sha256sum "$payload"|awk '{print $1}')
cp "$NEKO_BIN" "$run/neko-cli"; cp "$HY2_BIN" "$run/hysteria"; cp scripts/bench/process-resource-sampler.py scripts/bench/echo-payload.py scripts/bench/parse-listener.py scripts/bench/owned-lab-control-plane.sh "$run/"
"$run/neko-cli" keygen --identity "$run/client.identity" >"$run/client-key"; "$run/neko-cli" keygen --identity "$run/server.identity" >"$run/server-key"
client_pub=$(sed -n 's/^client_public_key=//p' "$run/client-key"); server_pub=$(sed -n 's/^client_public_key=//p' "$run/server-key"); [ -n "$client_pub" ] && [ -n "$server_pub" ] || fail 'identity generation failed'
mkdir -m700 "$run/tls"; openssl req -x509 -newkey rsa:2048 -nodes -days 1 -subj /CN=neko-owned-lab -addext "subjectAltName=IP:$LAB_REMOTE_BIND_ADDRESS" -keyout "$run/tls/key.pem" -out "$run/tls/cert.pem" >/dev/null 2>&1
cert_pin=$(openssl x509 -in "$run/tls/cert.pem" -noout -fingerprint -sha256 | sed -n 's/^sha256 Fingerprint=//Ip')
[[ "$cert_pin" =~ ^([0-9A-F]{2}:){31}[0-9A-F]{2}$ ]] || fail 'cannot derive disposable HY2 certificate pin'
auth=$(openssl rand -hex 24)
cat >"$run/hy2-server.yaml" <<CFG
listen: ${bind_authority}:${ports[1]}
tls:
  cert: $remote/tls/cert.pem
  key: $remote/tls/key.pem
auth:
  type: password
  password: $auth
CFG
cat >"$run/hy2-client.yaml" <<CFG
server: ${connect_authority}:${ports[1]}
auth: $auth
tls:
  insecure: true
  pinSHA256: $cert_pin
tcpForwarding:
  - listen: 127.0.0.1:${ports[2]}
    remote: 127.0.0.1:${ports[3]}
CFG
cat >"$run/echo-server.py" <<'PY'
import socket,sys
p=int(sys.argv[1]); n=int(sys.argv[2]); lim=int(sys.argv[3]); s=socket.socket(); s.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1); s.bind(('127.0.0.1',p)); s.listen(1); s.settimeout(60)
for _ in range(n):
 c,_=s.accept(); c.settimeout(30); b=b''
 while len(b)<lim:
  x=c.recv(lim-len(b))
  if not x: break
  b+=x
 c.sendall(b); c.close()
s.close()
PY
chmod 700 "$run"/* "$run/tls"/* 2>/dev/null || true
ssh_bounded "$LAB_SSH_TARGET" "umask 077; mkdir '$remote'"; remote_started=1; tar -C "$run" -cf - neko-cli hysteria process-resource-sampler.py echo-payload.py echo-server.py parse-listener.py owned-lab-control-plane.sh payload.bin server.identity tls hy2-server.yaml | ssh_bounded "$LAB_SSH_TARGET" "tar -C '$remote' -xf -; chmod 700 '$remote/neko-cli' '$remote/hysteria'; : >'$remote/pids'"
ssh_bounded "$LAB_SSH_TARGET" "nohup setsid python3 '$remote/echo-server.py' '${ports[3]}' '$RUNS' '$BYTES' >'$remote/echo.log' 2>&1 </dev/null & echo \$! >>'$remote/pids'; nohup setsid python3 '$remote/process-resource-sampler.py' --experiment-id hy2-owned-lab --implementation hy2-v2.9.3 --role server --identity sha256:66dbdb0608f25f3057b433afe975a9fc1af2ca8e512479e294988b3ef363d6c1 --application-bytes '$((RUNS*BYTES))' --owned-port '${ports[1]}' --interval-ms 10 --max-seconds '$((RUNS*TIMEOUT+10))' --output '$remote/hy2-server-resource.json' -- '$remote/hysteria' server -c '$remote/hy2-server.yaml' >'$remote/hy2-server.log' 2>&1 </dev/null & echo \$! >>'$remote/pids'"
require_remote_listener udp "$LAB_REMOTE_BIND_ADDRESS" "${ports[1]}" 200 "" hy2-server-readiness
run_client(){
 local impl=$1 run_no=$2 owned_port=$3 cmd=$4 raw stats row resource diagnostics rc
 raw=$run/raw.json; stats=$run/stats.jsonl; row=$run/record.json; resource=$run/$impl-client-$run_no-resource.json
 : >"$raw"; : >"$stats"; failure_stage="$impl-$run_no-client"
 set +e
 run_bounded "$TIMEOUT" /usr/bin/time -a \
   -f '{"sentinel":"nekomusume.gnu-time.v1","elapsed_seconds":%e,"cpu_user_seconds":%U,"cpu_system_seconds":%S,"rss_kib":%M,"exit_code":%x}' -o "$stats" \
   python3 "$run/process-resource-sampler.py" --experiment-id "$impl-owned-lab-$run_no" --implementation "$impl" --role client --identity "sha256:${client_identity[$impl]}" --application-bytes "$BYTES" --owned-port "$owned_port" --interval-ms 10 --max-seconds "$TIMEOUT" --output "$resource" -- bash -c "$cmd" >"$raw" 2>"$run/client.err"
 rc=$?; set -e
 diagnostics=$run/client.err; [ "$impl" != hy2 ] || diagnostics=$run/hy2-client-$run_no.log
 python3 "$validator" make-sample --implementation "$impl" --run "$run_no" --return-code "$rc" --time "$stats" --resource "$resource" --client-output "$raw" --client-diagnostics "$diagnostics" --bytes "$BYTES" --payload-hash "$payload_hash" --expected-identity "sha256:${client_identity[$impl]}" >"$row"
 atomic_append "$row"
 [ "$(jq -r .failures "$row")" -eq 0 ] || { failure_stage="$impl-$run_no-failed"; blocked "$failure_stage"; }
}
neko_identity=$(sha256sum "$run/neko-cli"|awk '{print $1}')
declare -A client_identity=([nekomusume]="$neko_identity" [hy2]=66dbdb0608f25f3057b433afe975a9fc1af2ca8e512479e294988b3ef363d6c1)
for i in $(seq 1 "$RUNS"); do
 [ "$(remaining_sec)" -gt 0 ] || { failure_stage=deadline; blocked "$failure_stage"; }
 ssh_bounded "$LAB_SSH_TARGET" "python3 '$remote/process-resource-sampler.py' --experiment-id neko-owned-lab-$i --implementation nekomusume --role server --identity sha256:$(sha256sum "$run/neko-cli"|awk '{print $1}') --application-bytes '$BYTES' --owned-port '${ports[0]}' --interval-ms 10 --max-seconds '$TIMEOUT' --output '$remote/neko-server-$i-resource.json' -- '$remote/neko-cli' server --transport tcp --bind '${bind_authority}:${ports[0]}' --port '${ports[0]}' --identity '$remote/server.identity' --client-key '$client_pub' --bytes '$BYTES' --count 1 --duration '$TIMEOUT' >'$remote/neko-server-$i.log' 2>&1" & spid=$!; active_spid=$spid
 require_remote_listener tcp "$LAB_REMOTE_BIND_ADDRESS" "${ports[0]}" 200 "$spid" nekomusume-readiness
 run_client nekomusume "$i" "${ports[0]}" "'$run/neko-cli' client --transport tcp --addr '${connect_authority}:${ports[0]}' --port '${ports[0]}' --identity '$run/client.identity' --server-key '$server_pub' --bytes '$BYTES' --count 1 --duration '$TIMEOUT' --payload-file '$payload' --json"
 wait "$spid" || true; active_spid=
 run_client hy2 "$i" "${ports[2]}" "
   '$run/hysteria' client -c '$run/hy2-client.yaml' >'$run/hy2-client-$i.log' 2>&1 & transport_pid=\$!
   trap 'kill "\$transport_pid" 2>/dev/null || true; wait "\$transport_pid" 2>/dev/null || true' EXIT INT TERM
   ready=0; for _ in \$(seq 1 100); do
     ss -H -lnt 'sport = :${ports[2]}' | grep -q . && { ready=1; break; }
     kill -0 "\$transport_pid" 2>/dev/null || break
     sleep .05
   done
   [ "\$ready" -eq 1 ] || exit 70
   python3 '$run/echo-payload.py' --host 127.0.0.1 --port '${ports[2]}' --payload-file '$payload' --timeout '$TIMEOUT'
   rc=\$?; kill "\$transport_pid" 2>/dev/null || true; wait "\$transport_pid" 2>/dev/null || true; trap - EXIT INT TERM; exit "\$rc"
 "
done
failure_stage=cleanup; cleanup 0; [ "$cleanup_ok" -eq 1 ] || fail 'cleanup verification failed'
resources=$run/resources.json; jq -s 'add' <(jq -s . "$run"/*-client-*-resource.json) "$remote_resources" >"$resources"
mtu=$(ip route get "$LAB_REMOTE_ADDRESS" | sed -n 's/.* mtu \([0-9]*\).*/\1/p'); [ -n "$mtu" ] || mtu=$(cat /sys/class/net/$(ip route get "$LAB_REMOTE_ADDRESS"|awk '{for(i=1;i<=NF;i++)if($i=="dev"){print $(i+1);exit}}')/mtu)
commit=$(git rev-parse HEAD); neko_hash=$(sha256sum "$run/neko-cli"|awk '{print $1}'); now=$(date -u +%FT%TZ)
failure_stage=final_assembly
jq -s --arg commit "$commit" --arg endpoint "$LAB_ENDPOINT_ID" --arg mtu "$mtu" --arg hash "$payload_hash" --arg neko_hash "$neko_hash" --arg at "$now" --argjson runs "$RUNS" --argjson bytes "$BYTES" --argjson duration_ms "$GLOBAL_DEADLINE_MS" --slurpfile resources "$resources" 'def pct($x;$p): if ($x|length)==0 then null else ($x|sort|.[((($x|length)-1)*$p|ceil)]) end; {schema:"nekomusume.benchmark-result.v1",experiment_id:"hy2-owned-lab-paired",git_commit:$commit,mode:"controlled-owned-lab",transport:"deterministic",scope:"self-owned-client-vps",bounds:{maximum_duration_ms:($duration_ms),application_bytes_max:($bytes*$runs*2)},contract:{endpoint_id:$endpoint,route_id:"same-client-vps-nearby-interleaved",mtu:($mtu|tonumber),security_profile:"authenticated encrypted research configuration",client_lifecycle:"fresh transport per timed sample",client_resource_scope:"sampler-created process group",server_resource_scope:"separately labelled; excluded from client timing",load_profile:"single-stream exact authenticated echo",payload_bytes:$bytes,payload_prepared:true,payload_sha256:$hash,runs_per_implementation:$runs,enforced_global_deadline_ms:$duration_ms,work_deadline_ms:($duration_ms-60000),cleanup_reserve_ms:60000,whole_lab_deadline_ms:$duration_ms,cleanup_reserve_sec:60,nekomusume_binary_sha256:$neko_hash,hy2_version:"v2.9.3",hy2_binary_sha256:"66dbdb0608f25f3057b433afe975a9fc1af2ca8e512479e294988b3ef363d6c1",wire_bytes_nullable:true,captured_at:$at},samples:.,summary:(group_by(.implementation)|map({implementation:.[0].implementation,failures:(map(.failures)|add),median_exchange_latency_ms:(pct([.[]|select(.failures==0)|.elapsed_seconds*1000];0.5)),p95_exchange_latency_ms:(pct([.[]|select(.failures==0)|.elapsed_seconds*1000];0.95)),application_bytes:(map(select(.failures==0)|.application_bytes)|add//0),wire_bytes:null})),resources:$resources[0],cleanup_status:"verified",cleanup_evidence:{local_processes_reaped:true,local_listeners_remaining:0,remote_process_groups_reaped:true,remote_listeners_remaining:0,remote_temp_path_removed:true}}' "$records" >"$out.tmp"
python3 "$validator" validate-result "$out.tmp" >/dev/null || blocked validation
mv -f "$out.tmp" "$out"; rm -f "$records"; rm -rf "$run"; trap - EXIT INT TERM; echo "$out"
