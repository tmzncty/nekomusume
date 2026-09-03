#!/usr/bin/env bash
set -euo pipefail
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd); tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
cat >"$tmp/date" <<'SH_DATE'
#!/bin/sh
printf '%s\n' "${FAKE_NOW_MS:?}"
SH_DATE
cat >"$tmp/ssh" <<'SH_SSH'
#!/bin/sh
printf 'ssh %s\n' "$*" >>"$TRACE"
case "$LISTENER_CASE" in
 exact-tcp|local-same-port) printf '%s\n' 'LISTEN 0 128 192.0.2.9:40097 0.0.0.0:*';;
 exact-udp) printf '%s\n' 'UNCONN 0 128 192.0.2.9:40098 0.0.0.0:*';;
 wildcard) printf '%s\n' 'LISTEN 0 128 0.0.0.0:40097 0.0.0.0:*';;
 wrong-addr) printf '%s\n' 'LISTEN 0 128 192.0.2.8:40097 0.0.0.0:*';;
 wrong-protocol) printf '%s\n' 'UNCONN 0 128 192.0.2.9:40097 0.0.0.0:*';;
 malformed) printf '%s\n' 'LISTEN 0 128 [192.0.2.9:40097 0.0.0.0:*';;
 ambiguous) printf '%s\n' 'LISTEN 0 128 192.0.2.9:40097 0.0.0.0:*' 'LISTEN 0 128 192.0.2.9:40097 0.0.0.0:*';;
 absent|early-exit) exit 1;;
 hang) sleep 20;;
 cleanup) printf ok;;
 *) exit 2;;
esac
SH_SSH
cat >"$tmp/timeout" <<'SH_TIMEOUT'
#!/bin/sh
limit=$1; shift; printf 'timeout %s %s\n' "$limit" "$*" >>"$TRACE"
[ "$LISTENER_CASE" = hang ] && exit 124
exec "$@"
SH_TIMEOUT
cat >"$tmp/sleep" <<'SH_SLEEP'
#!/bin/sh
:
SH_SLEEP
chmod +x "$tmp"/*
export TRACE=$tmp/trace FAKE_NOW_MS=100000 OWNED_LAB_DATE_BIN=$tmp/date OWNED_LAB_SSH_BIN=$tmp/ssh OWNED_LAB_TIMEOUT_BIN=$tmp/timeout OWNED_LAB_SLEEP_BIN=$tmp/sleep
export LISTENER_PARSER=$root/scripts/bench/parse-listener.py LAB_SSH_TARGET=fake READINESS_SEC=2
. "$root/scripts/bench/owned-lab-control-plane.sh"
olcp_init_deadlines 8 12
expect_ready(){ LISTENER_CASE=$1; export LISTENER_CASE; remote_listener_ready "$2" 192.0.2.9 "$3"; }
expect_ready exact-tcp tcp 40097
grep -q 'ss -H -ltn' "$TRACE"
: >"$TRACE"
expect_ready exact-udp udp 40098
grep -q 'ss -H -lun' "$TRACE"
# A same-numbered local listener is irrelevant: readiness consumes only fake remote SSH output.
expect_ready local-same-port tcp 40097
for c in wildcard wrong-addr wrong-protocol malformed ambiguous; do
  if expect_ready "$c" tcp 40097; then echo "$c accepted" >&2; exit 1; fi
done
blocked_hook(){ printf '%s\n' "$1" >"$tmp/stage"; return 19; }
export -f blocked_hook; export OWNED_LAB_BLOCKED_FN=blocked_hook
LISTENER_CASE=absent; export LISTENER_CASE
set +e; require_remote_listener udp 192.0.2.9 40098 2 '' hy2-server-readiness; rc=$?; set -e
[ "$rc" -eq 19 ] && grep -qx hy2-server-readiness "$tmp/stage" && ! grep -q timed-client "$TRACE"
set +e; require_remote_listener tcp 192.0.2.9 40097 2 '' nekomusume-readiness; rc=$?; set -e
[ "$rc" -eq 19 ] && grep -qx nekomusume-readiness "$tmp/stage" && ! grep -q timed-client "$TRACE"
# An already-exited remote launcher stops readiness after the first probe.
: >"$TRACE"; LISTENER_CASE=early-exit; export LISTENER_CASE
sh -c 'exit 7' & exited_pid=$!; sleep .05
set +e; require_remote_listener tcp 192.0.2.9 40097 200 "$exited_pid" nekomusume-readiness; rc=$?; set -e
[ "$rc" -eq 19 ] && [ "$(grep -c '^ssh ' "$TRACE")" -eq 1 ]
# Hanging control operations use the lesser stage/work remainder.
: >"$TRACE"; LISTENER_CASE=hang; export LISTENER_CASE
set +e; run_bounded 3 "$tmp/ssh" fake operation; rc=$?; set -e
[ "$rc" -eq 124 ] && grep -q '^timeout 3 ' "$TRACE"
# Cleanup retains its independent reserve after work expiry.
FAKE_NOW_MS=109000; export FAKE_NOW_MS cleanup_mode=1; LISTENER_CASE=cleanup; export LISTENER_CASE
ssh_bounded fake cleanup | grep -qx ok
grep -q '^timeout 3 ' "$TRACE"
# Invalid/over-budget plans fail before any SSH operation.
: >"$TRACE"; if olcp_init_deadlines 600 600; then exit 1; fi; [ ! -s "$TRACE" ]
printf '%s\n' owned-lab-control-plane-test-ok
