#!/usr/bin/env bash
# Shared control-plane primitives for compare-hy2-owned-lab.sh and its no-VPS tests.
olcp_now_ms(){ "${OWNED_LAB_DATE_BIN:-date}" +%s%3N; }
olcp_init_deadlines(){
  local work=$1 whole=$2
  [[ "$work" =~ ^[0-9]+$ && "$whole" =~ ^[0-9]+$ ]] || return 2
  [ "$work" -lt "$whole" ] && [ "$whole" -le 600 ] || return 2
  START_MS=$(olcp_now_ms); WORK_DEADLINE_MS=$((work * 1000)); GLOBAL_DEADLINE_MS=$((whole * 1000))
  DEADLINE_MS=$((START_MS + WORK_DEADLINE_MS)); CLEANUP_DEADLINE_MS=$((START_MS + GLOBAL_DEADLINE_MS)); cleanup_mode=0; STAGE_DEADLINE_MS=0
}
remaining_sec(){
  local end=$DEADLINE_MS now left
  [ "$cleanup_mode" -eq 1 ] && end=$CLEANUP_DEADLINE_MS
  [ "${STAGE_DEADLINE_MS:-0}" -gt 0 ] && [ "$STAGE_DEADLINE_MS" -lt "$end" ] && end=$STAGE_DEADLINE_MS
  now=$(olcp_now_ms); left=$(((end - now + 999) / 1000)); [ "$left" -gt 0 ] && echo "$left" || echo 0
}
ssh_bounded(){
  local left; left=$(remaining_sec); [ "$left" -gt 0 ] || return 124
  "${OWNED_LAB_TIMEOUT_BIN:-timeout}" "$left" "${OWNED_LAB_SSH_BIN:-ssh}" -o ConnectTimeout="$left" -o BatchMode=yes "$@"
}
remote_listener_ready(){
  local protocol=$1 address=$2 port=$3 ss_args
  case "$protocol" in tcp) ss_args='-H -ltn';; udp) ss_args='-H -lun';; *) return 2;; esac
  ssh_bounded "$LAB_SSH_TARGET" "ss $ss_args" | "${OWNED_LAB_PYTHON_BIN:-python3}" "$LISTENER_PARSER" "$protocol" "$address" "$port" >/dev/null 2>&1
}
wait_remote_listener(){
  local protocol=$1 address=$2 port=$3 attempts=$4 watched_pid=${5:-} i
  STAGE_DEADLINE_MS=$(($(olcp_now_ms) + READINESS_SEC * 1000))
  for i in $(seq 1 "$attempts"); do
    remote_listener_ready "$protocol" "$address" "$port" && { STAGE_DEADLINE_MS=0; return 0; }
    [ -z "$watched_pid" ] || kill -0 "$watched_pid" 2>/dev/null || { STAGE_DEADLINE_MS=0; return 1; }
    [ "$(remaining_sec)" -gt 0 ] || break
    "${OWNED_LAB_SLEEP_BIN:-sleep}" .05
  done
  STAGE_DEADLINE_MS=0; return 1
}
run_bounded(){
  local stage_limit=$1 left
  STAGE_DEADLINE_MS=$(($(olcp_now_ms) + stage_limit * 1000)); left=$(remaining_sec)
  [ "$left" -gt 0 ] || { STAGE_DEADLINE_MS=0; return 124; }
  local rc=0
  "${OWNED_LAB_TIMEOUT_BIN:-timeout}" "$left" "${@:2}" || rc=$?
  STAGE_DEADLINE_MS=0; return "$rc"
}
require_remote_listener(){
  local protocol=$1 address=$2 port=$3 attempts=$4 watched_pid=$5 stage=$6
  wait_remote_listener "$protocol" "$address" "$port" "$attempts" "$watched_pid" && return 0
  failure_stage=$stage
  [ -z "$watched_pid" ] || wait "$watched_pid" 2>/dev/null || true
  "${OWNED_LAB_BLOCKED_FN:-blocked}" "$stage"
}
