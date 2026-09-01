#!/usr/bin/env bash
set -euo pipefail
fail(){ echo "compare-hy2-owned-lab: $*" >&2; exit 2; }
need(){ [ -n "${!1:-}" ] || fail "missing $1"; }
for v in LAB_SSH_TARGET LAB_ENDPOINT_ID LAB_ENDPOINT_SHA256 LAB_REMOTE_ADDRESS NEKO_BIN HY2_BIN; do need "$v"; done
[ "${NEKO_OWNED_LAB:-}" = yes ] || fail 'NEKO_OWNED_LAB=yes is required'
case "$LAB_SSH_TARGET" in *[!A-Za-z0-9._-]*|'') fail 'invalid SSH target';; esac
case "$LAB_ENDPOINT_ID" in *[!A-Za-z0-9._-]*|'') fail 'invalid endpoint id';; esac
[[ "$LAB_ENDPOINT_SHA256" =~ ^[0-9a-f]{64}$ ]] || fail 'invalid endpoint SHA-256'
[[ "$LAB_REMOTE_ADDRESS" =~ ^[A-Za-z0-9.:_-]+$ ]] || fail 'invalid remote address'
resolved=$(ssh -G "$LAB_SSH_TARGET" 2>/dev/null | awk '$1=="hostname"{print $2;exit}')
[ -n "$resolved" ] || fail 'SSH target does not resolve'
[ "$(printf %s "$resolved" | sha256sum | awk '{print $1}')" = "$LAB_ENDPOINT_SHA256" ] || fail 'SSH endpoint contract mismatch'
[ "$resolved" = "$LAB_REMOTE_ADDRESS" ] || fail 'remote address differs from SSH endpoint'
RUNS=${BENCH_RUNS:-5}; BYTES=${BENCH_PAYLOAD_BYTES:-1200}; TIMEOUT=${BENCH_TIMEOUT_SEC:-30}
[[ "$RUNS" =~ ^[0-9]+$ && "$RUNS" -ge 3 && "$RUNS" -le 10 ]] || fail 'BENCH_RUNS must be 3..10'
[[ "$BYTES" =~ ^[0-9]+$ && "$BYTES" -gt 0 && "$BYTES" -le 1200 ]] || fail 'BENCH_PAYLOAD_BYTES must be 1..1200'
[[ "$TIMEOUT" =~ ^[0-9]+$ && "$TIMEOUT" -gt 0 && "$TIMEOUT" -le 30 ]] || fail 'BENCH_TIMEOUT_SEC must be 1..30'
ports=("${NEKO_PORT:-40097}" "${HY2_UDP_PORT:-40098}" "${HY2_LOCAL_PORT:-40099}" "${ECHO_PORT:-40100}")
for p in "${ports[@]}"; do [[ "$p" =~ ^[0-9]+$ && "$p" -ge 40080 && "$p" -le 40100 ]] || fail 'ports must be 40080..40100'; done
[ "$(printf '%s\n' "${ports[@]}" | sort -u | wc -l)" -eq 4 ] || fail 'ports must be distinct'
[ -x "$NEKO_BIN" ] || fail 'NEKO_BIN is not executable'; [ -x "$HY2_BIN" ] || fail 'HY2_BIN is not executable'
[ "$(sha256sum "$HY2_BIN" | awk '{print $1}')" = 66dbdb0608f25f3057b433afe975a9fc1af2ca8e512479e294988b3ef363d6c1 ] || fail 'HY2 artifact is not pinned v2.9.3'
command -v jq >/dev/null || fail 'jq required'; command -v openssl >/dev/null || fail 'openssl required'; [ -x /usr/bin/time ] || fail 'GNU time required'
out=${1:-artifacts/hy2-owned-lab/result.json}; case "$out" in /*|*..*) fail 'output must be relative and non-traversing';; esac
[ "${1:-}" != --validate ] || { echo validated; exit 0; }
root=$(git rev-parse --show-toplevel); cd "$root"; mkdir -p "$(dirname "$out")"
run=$(mktemp -d /tmp/neko-hy2-owned.XXXXXXXX); remote="/tmp/$(basename "$run")"; records=$run/records.jsonl; : >"$records"
cleanup_status=failed
cleanup(){
  set +e
  [ -n "${hy2_client_pid:-}" ] && kill "$hy2_client_pid" 2>/dev/null
  ssh -o BatchMode=yes "$LAB_SSH_TARGET" "pkill -TERM -f '^$remote/(hysteria|neko-cli|echo-server.py)' 2>/dev/null || true; test ! -e '$remote/pids' || while read p; do kill \"\$p\" 2>/dev/null || true; done <'$remote/pids'; rm -rf '$remote'" >/dev/null 2>&1
  rm -rf "$run"
}
trap cleanup EXIT INT TERM
for p in "${ports[@]}"; do ! ss -H -lntup "sport = :$p" | grep -q . || fail "local experimental port $p occupied"; ssh -o BatchMode=yes "$LAB_SSH_TARGET" "! ss -H -lntup 'sport = :$p' | grep -q ." || fail "remote experimental port $p occupied"; done
payload=$run/payload.bin; dd if=/dev/zero of="$payload" bs=1 count="$BYTES" status=none; payload_hash=$(sha256sum "$payload"|awk '{print $1}')
cp "$NEKO_BIN" "$run/neko-cli"; cp "$HY2_BIN" "$run/hysteria"; cp scripts/bench/process-resource-sampler.py scripts/bench/echo-payload.py "$run/"
"$run/neko-cli" keygen --identity "$run/client.identity" >"$run/client-key"; "$run/neko-cli" keygen --identity "$run/server.identity" >"$run/server-key"
client_pub=$(sed -n 's/^client_public_key=//p' "$run/client-key"); server_pub=$(sed -n 's/^client_public_key=//p' "$run/server-key"); [ -n "$client_pub" ] && [ -n "$server_pub" ] || fail 'identity generation failed'
mkdir -m700 "$run/tls"; openssl req -x509 -newkey rsa:2048 -nodes -days 1 -subj /CN=neko-owned-lab -addext "subjectAltName=IP:$LAB_REMOTE_ADDRESS" -keyout "$run/tls/key.pem" -out "$run/tls/cert.pem" >/dev/null 2>&1
auth=$(openssl rand -hex 24)
cat >"$run/hy2-server.yaml" <<CFG
listen: :${ports[1]}
tls:
  cert: $remote/tls/cert.pem
  key: $remote/tls/key.pem
auth:
  type: password
  password: $auth
CFG
cat >"$run/hy2-client.yaml" <<CFG
server: $LAB_REMOTE_ADDRESS:${ports[1]}
auth: $auth
tls:
  insecure: true
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
ssh -o BatchMode=yes "$LAB_SSH_TARGET" "umask 077; mkdir '$remote'"; tar -C "$run" -cf - neko-cli hysteria process-resource-sampler.py echo-payload.py echo-server.py payload.bin server.identity tls hy2-server.yaml | ssh -o BatchMode=yes "$LAB_SSH_TARGET" "tar -C '$remote' -xf -; chmod 700 '$remote/neko-cli' '$remote/hysteria'; : >'$remote/pids'"
ssh -o BatchMode=yes "$LAB_SSH_TARGET" "nohup python3 '$remote/echo-server.py' '${ports[3]}' '$RUNS' '$BYTES' >'$remote/echo.log' 2>&1 </dev/null & echo \$! >>'$remote/pids'; nohup python3 '$remote/process-resource-sampler.py' --experiment-id hy2-owned-lab --implementation hy2-v2.9.3 --role server --identity sha256:66dbdb0608f25f30 --application-bytes '$((RUNS*BYTES))' --owned-port '${ports[1]}' --interval-ms 10 --max-seconds '$((RUNS*TIMEOUT+10))' --output '$remote/hy2-server-resource.json' -- '$remote/hysteria' server -c '$remote/hy2-server.yaml' >'$remote/hy2-server.log' 2>&1 </dev/null & echo \$! >>'$remote/pids'"
python3 "$run/process-resource-sampler.py" --experiment-id hy2-owned-lab --implementation hy2-v2.9.3 --role client --identity sha256:66dbdb0608f25f30 --application-bytes "$((RUNS*BYTES))" --owned-port "${ports[2]}" --interval-ms 10 --max-seconds "$((RUNS*TIMEOUT+10))" --output "$run/hy2-client-resource.json" -- "$run/hysteria" client -c "$run/hy2-client.yaml" >"$run/hy2-client.log" 2>&1 & hy2_client_pid=$!
for _ in $(seq 1 100); do
  ss -H -lnt "sport = :${ports[2]}"|grep -q . && break
  if ! kill -0 "$hy2_client_pid" 2>/dev/null; then
    sed "s/$LAB_REMOTE_ADDRESS/<owned-endpoint>/g" "$run/hy2-client.log" >&2
    ssh -o BatchMode=yes "$LAB_SSH_TARGET" "sed 's/$LAB_REMOTE_ADDRESS/<owned-endpoint>/g' '$remote/hy2-server.log'" >&2 || true
    fail 'HY2 client exited before readiness'
  fi
  sleep .1
done
ss -H -lnt "sport = :${ports[2]}"|grep -q . || fail 'HY2 forwarding listener not ready'
run_client(){
 impl=$1 run_no=$2 cmd=$3 raw=$run/raw.json stats=$run/stats
 : >"$raw"; : >"$stats"; set +e; timeout "$TIMEOUT" /usr/bin/time -f '%e %U %S %M %x' -o "$stats" bash -c "$cmd" >"$raw" 2>"$run/client.err"; rc=$?; set -e
 elapsed=$(awk '{print $1}' "$stats" 2>/dev/null || true); user=$(awk '{print $2}' "$stats" 2>/dev/null || true); sys=$(awk '{print $3}' "$stats" 2>/dev/null || true); rss=$(awk '{print $4}' "$stats" 2>/dev/null || true); fd=$(jq -r '.fd_count//empty' "$raw" 2>/dev/null||true); app=$(jq -r '.application_bytes//empty' "$raw" 2>/dev/null||true); h=$(jq -r '.payload_sha256//empty' "$raw" 2>/dev/null||true); failure=0
 [ "$rc" -eq 0 ] && [ "$app" = "$BYTES" ] && [ "$h" = "$payload_hash" ] && [[ "$fd" =~ ^[0-9]+$ ]] || failure=1
 jq -nc --arg name "$impl-$run_no" --arg impl "$impl" --argjson run "$run_no" --argjson failures "$failure" --arg e "$elapsed" --arg u "$user" --arg s "$sys" --arg r "$rss" --arg f "$fd" --argjson bytes "$BYTES" --arg hash "$h" --argjson rc "$rc" '{name:$name,implementation:$impl,run:$run,failures:$failures,elapsed_seconds:(if $e=="" then null else ($e|tonumber) end),cpu_user_seconds:(if $u=="" then null else ($u|tonumber) end),cpu_system_seconds:(if $s=="" then null else ($s|tonumber) end),rss_kib:(if $r=="" then null else ($r|tonumber) end),fd_count:(if $f=="" then null else ($f|tonumber) end),application_bytes:$bytes,payload_sha256:(if $hash=="" then null else $hash end),wire_bytes:null,exit_code:$rc}' >>"$records"
}
for i in $(seq 1 "$RUNS"); do
 ssh -o BatchMode=yes "$LAB_SSH_TARGET" "python3 '$remote/process-resource-sampler.py' --experiment-id neko-owned-lab-$i --implementation nekomusume --role server --identity sha256:$(sha256sum "$run/neko-cli"|cut -c1-16) --application-bytes '$BYTES' --owned-port '${ports[0]}' --interval-ms 10 --max-seconds '$TIMEOUT' --output '$remote/neko-server-$i-resource.json' -- '$remote/neko-cli' server --transport tcp --bind '0.0.0.0:${ports[0]}' --port '${ports[0]}' --identity '$remote/server.identity' --client-key '$client_pub' --bytes '$BYTES' --count 1 --duration '$TIMEOUT' >'$remote/neko-server-$i.log' 2>&1" & spid=$!
 sleep .15
 run_client nekomusume "$i" "'$run/neko-cli' client --transport tcp --addr '$LAB_REMOTE_ADDRESS:${ports[0]}' --port '${ports[0]}' --identity '$run/client.identity' --server-key '$server_pub' --bytes '$BYTES' --count 1 --duration '$TIMEOUT' --payload-file '$payload' --json"
 wait "$spid" || true
 run_client hy2 "$i" "python3 '$run/echo-payload.py' --host 127.0.0.1 --port '${ports[2]}' --payload-file '$payload' --timeout '$TIMEOUT'"
done
kill "$hy2_client_pid" 2>/dev/null || true; wait "$hy2_client_pid" || true; unset hy2_client_pid
ssh -o BatchMode=yes "$LAB_SSH_TARGET" "while read p; do kill \"\$p\" 2>/dev/null || true; done <'$remote/pids'; sleep .2; for p in '${ports[0]}' '${ports[1]}' '${ports[3]}'; do ! ss -H -lntup \"sport = :\$p\"|grep -q . || exit 1; done"
for p in "${ports[2]}"; do ! ss -H -lntup "sport = :$p"|grep -q . || fail 'local cleanup failed'; done
remote_resources=$run/remote-resources.json; ssh -o BatchMode=yes "$LAB_SSH_TARGET" "jq -s . '$remote'/*-resource.json" >"$remote_resources"
resources=$run/resources.json; jq -s 'add' <(jq -s . "$run/hy2-client-resource.json") "$remote_resources" >"$resources"
mtu=$(ip route get "$LAB_REMOTE_ADDRESS" | sed -n 's/.* mtu \([0-9]*\).*/\1/p'); [ -n "$mtu" ] || mtu=$(cat /sys/class/net/$(ip route get "$LAB_REMOTE_ADDRESS"|awk '{for(i=1;i<=NF;i++)if($i=="dev"){print $(i+1);exit}}')/mtu)
commit=$(git rev-parse HEAD); neko_hash=$(sha256sum "$run/neko-cli"|awk '{print $1}'); now=$(date -u +%FT%TZ)
jq -s --arg commit "$commit" --arg endpoint "$LAB_ENDPOINT_ID" --arg mtu "$mtu" --arg hash "$payload_hash" --arg neko_hash "$neko_hash" --arg at "$now" --argjson runs "$RUNS" --argjson bytes "$BYTES" --slurpfile resources "$resources" 'def pct($x;$p): if ($x|length)==0 then null else ($x|sort|.[((($x|length)-1)*$p|ceil)]) end; {schema:"nekomusume.benchmark-result.v1",experiment_id:"hy2-owned-lab-paired",git_commit:$commit,mode:"controlled-owned-lab",transport:"deterministic",scope:"self-owned-client-vps",bounds:{maximum_duration_ms:300000,application_bytes_max:($bytes*$runs*2)},contract:{endpoint_id:$endpoint,route_id:"same-client-vps-nearby-interleaved",mtu:($mtu|tonumber),security_profile:"authenticated encrypted research configuration",load_profile:"single-stream exact authenticated echo",payload_bytes:$bytes,payload_sha256:$hash,runs_per_implementation:$runs,nekomusume_binary_sha256:$neko_hash,hy2_version:"v2.9.3",hy2_binary_sha256:"66dbdb0608f25f3057b433afe975a9fc1af2ca8e512479e294988b3ef363d6c1",wire_bytes_nullable:true,captured_at:$at},samples:.,summary:(group_by(.implementation)|map({implementation:.[0].implementation,failures:(map(.failures)|add),median_exchange_latency_ms:(pct([.[]|select(.failures==0)|.elapsed_seconds*1000];0.5)),p95_exchange_latency_ms:(pct([.[]|select(.failures==0)|.elapsed_seconds*1000];0.95)),application_bytes:(map(.application_bytes)|add),wire_bytes:null})),resources:$resources[0],cleanup_status:"verified"}' "$records" >"$out"
cleanup_status=verified
ssh -o BatchMode=yes "$LAB_SSH_TARGET" "rm -rf '$remote'"; rm -rf "$run"; trap - EXIT INT TERM
echo "$out"
