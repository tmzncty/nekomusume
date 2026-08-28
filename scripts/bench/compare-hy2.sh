#!/usr/bin/env bash
set -euo pipefail
# Controlled-comparison scaffold. It refuses to run unless both exact commands,
# an isolated lab marker, and equality metadata are supplied explicitly.
: "${NEKO_BENCH_CMD:?set exact candidate command}"
: "${HY2_BENCH_CMD:?set exact Hysteria2 command}"
: "${BENCH_SERVER_ID:?set controlled server identifier}"
: "${BENCH_ROUTE_ID:?set route/window identifier}"
: "${BENCH_MTU:?set equal MTU}"
: "${BENCH_SECURITY_PROFILE:?set equal security profile}"
: "${BENCH_LOAD_PROFILE:?set equal application load profile}"
[ "${NEKO_ISOLATED_LAB:-}" = yes ] || { echo 'refusing outside explicitly isolated lab' >&2; exit 2; }
[ "${NEKO_ALLOW_COMMAND_EVAL:-}" = yes ] || { echo 'refusing command execution without NEKO_ALLOW_COMMAND_EVAL=yes' >&2; exit 2; }
RUNS=${BENCH_RUNS:-5}; [ "$RUNS" -ge 3 ] || { echo 'BENCH_RUNS must be >=3' >&2; exit 2; }
out=${1:-hy2-comparison.jsonl}; : > "$out"
for impl in nekomusume hy2; do
  if [ "$impl" = nekomusume ]; then cmd=$NEKO_BENCH_CMD; else cmd=$HY2_BENCH_CMD; fi
  for run in $(seq 1 "$RUNS"); do
    start=$(date +%s%N); set +e; bash -lc "$cmd" >/dev/null 2>&1; rc=$?; set -e; end=$(date +%s%N)
    jq -nc --arg impl "$impl" --argjson run "$run" --argjson rc "$rc" --argjson elapsed "$(( (end-start)/1000 ))" --arg server "$BENCH_SERVER_ID" --arg route "$BENCH_ROUTE_ID" --arg mtu "$BENCH_MTU" --arg security "$BENCH_SECURITY_PROFILE" --arg load "$BENCH_LOAD_PROFILE" '{implementation:$impl,run:$run,exit_code:$rc,elapsed_us:$elapsed,server_id:$server,route_id:$route,mtu:$mtu,security_profile:$security,load_profile:$load}' >> "$out"
  done
done
