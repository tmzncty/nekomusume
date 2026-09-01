#!/usr/bin/env bash
set -euo pipefail
fail(){ echo "compare-hy2: $*" >&2; exit 2; }
need(){ [ -n "${!1:-}" ] || fail "missing $1"; }
for v in NEKO_BENCH_CMD HY2_BENCH_CMD BENCH_SERVER_ID BENCH_ROUTE_ID BENCH_MTU BENCH_SECURITY_PROFILE BENCH_LOAD_PROFILE BENCH_TARGET_HOST; do need "$v"; done
[ "${NEKO_ISOLATED_LAB:-}" = yes ] || fail 'NEKO_ISOLATED_LAB=yes is required'
[ "${NEKO_ALLOW_COMMAND_EVAL:-}" = yes ] || fail 'NEKO_ALLOW_COMMAND_EVAL=yes is required'
case "$BENCH_TARGET_HOST" in 127.0.0.1|::1) ;; *) fail 'target must be loopback; WAN execution is forbidden';; esac
RUNS=${BENCH_RUNS:-5}; PAYLOAD_BYTES=${BENCH_PAYLOAD_BYTES:-1200}; TIMEOUT=${BENCH_TIMEOUT_SEC:-30}
[[ "$RUNS" =~ ^[0-9]+$ && "$RUNS" -ge 3 && "$RUNS" -le 100 ]] || fail 'BENCH_RUNS must be 3..100'
[[ "$PAYLOAD_BYTES" =~ ^[0-9]+$ && "$PAYLOAD_BYTES" -gt 0 && "$PAYLOAD_BYTES" -le 1048576 ]] || fail 'BENCH_PAYLOAD_BYTES must be 1..1048576'
[[ "$TIMEOUT" =~ ^[0-9]+$ && "$TIMEOUT" -gt 0 && "$TIMEOUT" -le 3600 ]] || fail 'BENCH_TIMEOUT_SEC must be 1..3600'
command -v jq >/dev/null || fail 'jq is required'; [ -x /usr/bin/time ] || fail 'GNU time is required'
out=${1:-hy2-comparison.json}; case "$out" in /*|*..*) fail 'output path must be local and non-traversing';; esac
mkdir -p "$(dirname "$out")"; tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
payload="$tmp/payload.bin"; dd if=/dev/zero of="$payload" bs=1 count="$PAYLOAD_BYTES" status=none
hash=$(sha256sum "$payload" | awk '{print $1}'); records="$tmp/records"; : >"$records"
run_one(){
  local impl=$1 cmd=$2 run=$3 raw=$tmp/raw stats=$tmp/stats rc elapsed user sys rss exitcode app wire fd failure
  : >"$raw"; : >"$stats"; set +e
  BENCH_PAYLOAD_FILE="$payload" BENCH_PAYLOAD_SHA256="$hash" BENCH_PAYLOAD_BYTES="$PAYLOAD_BYTES" BENCH_TARGET_HOST="$BENCH_TARGET_HOST" BENCH_TARGET_PORT="${BENCH_TARGET_PORT:-}" timeout --signal=TERM "$TIMEOUT" /usr/bin/time -f '%e %U %S %M %x' -o "$stats" bash -lc "$cmd" >"$raw" 2>/dev/null
  rc=$?; set -e
  elapsed=$(awk '{print $1}' "$stats" 2>/dev/null || true); user=$(awk '{print $2}' "$stats" 2>/dev/null || true); sys=$(awk '{print $3}' "$stats" 2>/dev/null || true); rss=$(awk '{print $4}' "$stats" 2>/dev/null || true); exitcode=$(awk '{print $5}' "$stats" 2>/dev/null || true)
  reported_hash=$(jq -r '.payload_sha256 // empty' "$raw" 2>/dev/null || true); app=$(jq -r '.application_bytes // empty' "$raw" 2>/dev/null || true); wire=$(jq -c '.wire_bytes // null' "$raw" 2>/dev/null || echo null); fd=$(jq -r '.fd_count // empty' "$raw" 2>/dev/null || true)
  failure=0; [ "$rc" -eq 0 ] && [ "$exitcode" = 0 ] && [[ "$app" =~ ^[0-9]+$ ]] && [ "$app" -eq "$PAYLOAD_BYTES" ] && [ "$reported_hash" = "$hash" ] && [[ "$fd" =~ ^[0-9]+$ ]] || failure=1
  jq -nc --arg implementation "$impl" --argjson run "$run" --argjson failures "$failure" --arg elapsed "$elapsed" --arg user "$user" --arg sys "$sys" --arg rss "$rss" --arg app "$app" --arg reported_hash "$reported_hash" --argjson wire "$wire" --arg fd "$fd" --argjson rc "$rc" '{name:($implementation+"-"+($run|tostring)),implementation:$implementation,run:$run,failures:$failures,elapsed_seconds:(if $elapsed=="" then null else ($elapsed|tonumber) end),cpu_user_seconds:(if $user=="" then null else ($user|tonumber) end),cpu_system_seconds:(if $sys=="" then null else ($sys|tonumber) end),rss_kib:(if $rss=="" then null else ($rss|tonumber) end),fd_count:(if $fd=="" then null else ($fd|tonumber) end),application_bytes:(if $app=="" then null else ($app|tonumber) end),payload_sha256:(if $reported_hash=="" then null else $reported_hash end),wire_bytes:$wire,exit_code:$rc}' >>"$records"
}
for impl in nekomusume hy2; do cmd=$NEKO_BENCH_CMD; [ "$impl" = hy2 ] && cmd=$HY2_BENCH_CMD; for run in $(seq 1 "$RUNS"); do run_one "$impl" "$cmd" "$run"; done; done
jq -s --arg commit "$(git rev-parse HEAD 2>/dev/null || echo unknown)" --arg hash "$hash" --arg server "$BENCH_SERVER_ID" --arg route "$BENCH_ROUTE_ID" --arg mtu "$BENCH_MTU" --arg security "$BENCH_SECURITY_PROFILE" --arg load "$BENCH_LOAD_PROFILE" --argjson bytes "$PAYLOAD_BYTES" --argjson runs "$RUNS" 'def pct($xs;$p): if ($xs|length)==0 then null else ($xs|sort|.[((($xs|length)-1)*$p|ceil)]) end; {schema:"nekomusume.benchmark-result.v1",experiment_id:"local-hy2-comparison",git_commit:$commit,mode:"controlled-local",transport:"deterministic",scope:"loopback-only",contract:{payload_sha256:$hash,payload_bytes:$bytes,runs:$runs,server_id:$server,route_id:$route,mtu:$mtu,security_profile:$security,load_profile:$load,wire_bytes_nullable:true},samples:.,summary:(group_by(.implementation)|map({implementation:.[0].implementation,failures:(map(.failures)|add),successful_latency_seconds:pct([.[]|select(.failures==0 and .elapsed_seconds!=null)|.elapsed_seconds];0.5),p95_latency_seconds:pct([.[]|select(.failures==0 and .elapsed_seconds!=null)|.elapsed_seconds];0.95)})),cleanup_status:"verified"}' "$records" >"$out"
