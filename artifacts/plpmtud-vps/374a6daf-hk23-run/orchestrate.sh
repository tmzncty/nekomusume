#!/bin/bash
# PLPMTUD changed-hypothesis VPS run orchestrator (2026-10-01)
# 85461-authorized: ufw allow 40080:40100/udp with comment, revoke + diff after.
set -u
J23="ssh -o ConnectTimeout=10 -J tmzn-hk root@23.147.120.24"
HK="ssh -o ConnectTimeout=10 -i $HOME/.ssh/id_ed25519_tmzn_hk tmzn-hk"
PORT=40098
R=/tmp/neko-exp

echo "=== 1. ufw backup + allow ==="
$J23 'ufw status numbered' > /tmp/vpsprep/ufw_before.txt && head -3 /tmp/vpsprep/ufw_before.txt
$J23 "ufw allow 40080:40100/udp comment 'neko-plpmtud-exp-20261001'" 2>&1 | tail -1

revoke() {
  echo "=== revoking ufw ==="
  $J23 'ufw delete allow 40080:40100/udp' 2>&1 | tail -1
  $J23 'ufw status numbered' > /tmp/vpsprep/ufw_after.txt
  if diff /tmp/vpsprep/ufw_before.txt /tmp/vpsprep/ufw_after.txt > /tmp/vpsprep/ufw.diff; then
    echo "UFW_DIFF_CLEAN"
  else
    echo "UFW_DIFF_MISMATCH"; cat /tmp/vpsprep/ufw.diff
  fi
}

echo "=== 2. runs A1-A3 (raise=3s) B1-B3 (default 300s) ==="
FAILED=0
for tag in A1 A2 A3 B1 B2 B3; do
  case $tag in
    A*) EXTRA="--plpmtud-raise-interval 3";;
    B*) EXTRA="";;
  esac
  $J23 "bash $R/remote-ctl.sh start $PORT" || { echo "SERVER_START_FAIL $tag"; FAILED=1; break; }
  sleep 1
  $HK "/tmp/neko-exp/neko-cli client --transport udp --addr 23.147.120.24:$PORT --port $PORT --identity /tmp/neko-exp/cli.identity --server-key \"\$(cat /tmp/neko-exp/server.pub)\" --count 8 --bytes 32 --duration 28 --plpmtud $EXTRA --diagnostic --json --experiment-id plpmtud-vps-$tag > $R/$tag.out 2> $R/$tag.err; echo RC=\$? >> $R/$tag.out"
  $J23 "bash $R/remote-ctl.sh stop $PORT" >/dev/null 2>&1
  CONV=$($HK "grep -c plpmtud_converged $R/$tag.out" 2>/dev/null | tr -d '\r')
  echo "[$tag] converged_lines=$CONV"
  if [ "${CONV:-0}" -lt 1 ]; then echo "RUN_FAILED $tag"; FAILED=1; break; fi
done

echo "=== 3. revoke + verify ==="
revoke

if [ "$FAILED" = "1" ]; then echo "ORCHESTRATION_FAILED_EARLY"; exit 1; fi
echo "=== 4. collect results ==="
mkdir -p /tmp/vpsprep/runs
for tag in A1 A2 A3 B1 B2 B3; do
  $HK "cat $R/$tag.out" > /tmp/vpsprep/runs/$tag.out 2>/dev/null
  $HK "cat $R/$tag.err" > /tmp/vpsprep/runs/$tag.err 2>/dev/null
done
$J23 'cat /tmp/neko-exp/server.log' > /tmp/vpsprep/runs/server-last.log 2>/dev/null
echo "collected: $(ls /tmp/vpsprep/runs/ | tr '\n' ' ')"
for tag in A1 A2 A3 B1 B2 B3; do
  echo "--- $tag converged:"; grep 'plpmtud_converged' /tmp/vpsprep/runs/$tag.out | head -3
done
