#!/usr/bin/env bash
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd); source_script=$root/scripts/bench/compare-hy2-owned-lab.sh; tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
cp "$source_script" "$tmp/adapter.sh"; s=$tmp/adapter.sh
cat >"$tmp/ssh" <<'SH'
#!/bin/sh
[ "$1" = -G ] && { echo 'hostname 192.0.2.8'; exit; }
printf '%s\n' "${MOCK_REMOTE_INTERFACES:-[]}"
SH
chmod +x "$tmp/ssh"; cp /bin/true "$tmp/neko"; cp /bin/true "$tmp/hy2"
hy2_hash=$(sha256sum "$tmp/hy2"|awk '{print $1}')
# Patch only the disposable test copy so validation can reach address guards.
python3 - "$s" "$hy2_hash" <<'PY'
from pathlib import Path
import sys
p=Path(sys.argv[1]); p.write_text(p.read_text().replace('66dbdb0608f25f3057b433afe975a9fc1af2ca8e512479e294988b3ef363d6c1', sys.argv[2]))
PY
hash=$(printf %s 192.0.2.8|sha256sum|awk '{print $1}')
base="PATH=$tmp:$PATH NEKO_OWNED_LAB=yes LAB_SSH_TARGET=owned LAB_ENDPOINT_ID=owned-vps LAB_ENDPOINT_SHA256=$hash LAB_REMOTE_ADDRESS=192.0.2.8 NEKO_BIN=$tmp/neko HY2_BIN=$tmp/hy2"
interfaces='[{"ifname":"eth0","addr_info":[{"family":"inet","local":"192.0.2.9","prefixlen":24}]}]'
# A dedicated assigned address validates even when it differs from the SSH/connection address.
env $base LAB_REMOTE_BIND_ADDRESS=192.0.2.9 MOCK_REMOTE_INTERFACES="$interfaces" "$s" --validate | grep -qx validated
# Fail closed when the dedicated address is absent or unsafe.
for address in '' 0.0.0.0 :: 127.0.0.1 ::1 224.0.0.1 ff02::1; do
  if env $base LAB_REMOTE_BIND_ADDRESS="$address" MOCK_REMOTE_INTERFACES="$interfaces" "$s" --validate >/dev/null 2>&1; then echo "unsafe bind accepted: $address" >&2; exit 1; fi
done
if env $base LAB_REMOTE_BIND_ADDRESS=192.0.2.10 MOCK_REMOTE_INTERFACES="$interfaces" "$s" --validate >/dev/null 2>&1; then echo 'nonlocal bind accepted' >&2; exit 1; fi
# Wrong HY2 identity must fail before execution in the authoritative script.
if env $base LAB_REMOTE_BIND_ADDRESS=192.0.2.9 MOCK_REMOTE_INTERFACES="$interfaces" "$source_script" --validate >/dev/null 2>&1; then echo 'unpinned HY2 accepted' >&2; exit 1; fi
# Static safety contracts: exact dedicated-address YAML; no wildcard HY2 or production config/service operations.
grep -Fq 'listen: ${bind_authority}:${ports[1]}' "$source_script"
grep -Fq 'server: ${connect_authority}:${ports[1]}' "$source_script"
grep -Fq -- '--addr '"'"'${connect_authority}:${ports[0]}'"'"'' "$source_script"
grep -Fq 'ip route get "$LAB_REMOTE_ADDRESS"' "$source_script"
! grep -Fq 'ip route get "$LAB_REMOTE_BIND_ADDRESS"' "$source_script"
grep -Fq -- '--bind '"'"'${bind_authority}:${ports[0]}'"'"'' "$source_script"
! grep -Eq 'listen: :|listen: 0\.0\.0\.0:|listen: \[::\]:' "$source_script"
! grep -Eq '/etc/hysteria|systemctl|pkill.*hysteria|0\.0\.0\.0.*hy2' "$source_script"
grep -q "HY2 artifact is not pinned" "$source_script"
# Exact disposable-certificate pin is mandatory; bare insecure is forbidden.
grep -Fq 'pinSHA256: $cert_pin' "$source_script"
grep -Fq 'openssl x509 -in "$run/tls/cert.pem" -noout -fingerprint -sha256' "$source_script"
python3 - "$source_script" <<'PYTLS'
from pathlib import Path
import re, sys
text=Path(sys.argv[1]).read_text()
tls=re.search(r'cat >"\$run/hy2-client.yaml".*?<<CFG\n(.*?)\nCFG', text, re.S).group(1)
assert 'insecure: true' in tls and 'pinSHA256: $cert_pin' in tls
assert not re.search(r'insecure: true\s*\ntcpForwarding:', text)
PYTLS
# A fresh transport client runs inside every HY2 timed sampler process group.
grep -Fq "hysteria' client -c" "$source_script"
grep -Fq 'run_client hy2 "$i"' "$source_script"
grep -Fq 'sampler-created process group' "$root/scripts/bench/validate-hy2-owned-lab.py"
! grep -Fq 'hy2_client_pid' "$source_script"
grep -Fq 'client_lifecycle:"fresh transport per timed sample"' "$source_script"
grep -Fq 'observed_clients != expected_clients' "$root/scripts/bench/validate-hy2-owned-lab.py"
# Endpoint mismatch and unsafe output remain rejected.
if env $base LAB_ENDPOINT_SHA256=$(printf %064d 0) LAB_REMOTE_BIND_ADDRESS=192.0.2.9 MOCK_REMOTE_INTERFACES="$interfaces" "$s" --validate >/dev/null 2>&1; then exit 1; fi
grep -q "BENCH_RUNS must be 3..10" "$source_script"; grep -q "ports must be distinct" "$source_script"
# Fake-command timing evidence: diagnostics and nonzero exit stay distinct from the sentinel.
cat >"$tmp/fake-time" <<'FAKE'
#!/bin/sh
out=$1
printf '%s\n' 'Command exited with non-zero status 9' >"$out"
printf '%s\n' '{"sentinel":"nekomusume.gnu-time.v1","elapsed_seconds":0.25,"cpu_user_seconds":0.01,"cpu_system_seconds":0.02,"rss_kib":9,"exit_code":9}' >>"$out"
exit 9
FAKE
chmod +x "$tmp/fake-time"
set +e; "$tmp/fake-time" "$tmp/fake-time.out"; fake_rc=$?; set -e
[ "$fake_rc" -eq 9 ]
python3 "$root/scripts/bench/validate-hy2-owned-lab.py" parse-time "$tmp/fake-time.out" | grep -q '"exit_code":9'
# Cleanup/reap and retention contracts are executable/static invariants of the harness.
grep -Fq 'trap on_exit EXIT; trap on_signal INT TERM' "$source_script"
grep -Fq 'wait "$pid" 2>/dev/null || true' "$source_script"
grep -Fq 'remote_process_groups_reaped' "$source_script"
grep -Fq 'atomic_append "$row"' "$source_script"
# Changed-hypothesis regression: first Nekomusume diagnostic/nonzero output becomes
# one typed retained failure row; record construction never feeds diagnostics to jq.
hash0=$(printf test-payload | sha256sum | awk '{print $1}')
printf '%s\n' 'client diagnostic: connection refused' 'second diagnostic line' >"$tmp/neko-first.out"
printf '%s\n' 'Command exited with non-zero status 9' '{"sentinel":"nekomusume.gnu-time.v1","elapsed_seconds":0.25,"cpu_user_seconds":0.01,"cpu_system_seconds":0.02,"rss_kib":9,"exit_code":9}' >"$tmp/neko-first.time"
printf '%s\n' '{"experiment_id":"nekomusume-owned-lab-1","implementation":"nekomusume","role":"client","fd":{"peak_count":4},"sampling":{"scope":"sampler-created process group"},"cleanup":{"process_reaped":true,"process_group_empty":true,"owned_sockets_after_exit":0,"complete":true}}' >"$tmp/neko-first.resource"
python3 "$root/scripts/bench/validate-hy2-owned-lab.py" make-sample --implementation nekomusume --run 1 --return-code 9 --time "$tmp/neko-first.time" --resource "$tmp/neko-first.resource" --client-output "$tmp/neko-first.out" --bytes 1200 --payload-hash "$hash0" >"$tmp/first.jsonl" 2>"$tmp/make.err"
[ ! -s "$tmp/make.err" ]; [ "$(wc -l <"$tmp/first.jsonl")" -eq 1 ]
jq -e '.name=="nekomusume-1" and .failures==1 and .exit_code==9 and .failure_stage=="client_exit" and .application_bytes==0 and .payload_sha256==null' "$tmp/first.jsonl" >/dev/null
! grep -q -- '--argjson' "$source_script"
python3 "$root/scripts/bench/validate-hy2-owned-lab.py" blocked --records "$tmp/first.jsonl" --output "$tmp/blocked.json" --stage nekomusume-1-client --commit "$(printf %040d 0)" --runs 5 --bytes 1200 --payload-prepared true --payload-hash "$hash0" --local-reaped true --local-listeners 0 --remote-reaped true --remote-listeners 0 --remote-path-removed true
python3 "$root/scripts/bench/validate-hy2-owned-lab.py" validate-result "$tmp/blocked.json" | grep -qx validated
jq -e '.status=="BLOCKED_HARNESS" and (.samples|length)==1 and .cleanup_status=="verified"' "$tmp/blocked.json" >/dev/null
# Executable/path deletion race: kill and reap the process group before deleting its path.
mkdir "$tmp/race"; cat >"$tmp/race/child" <<'RACE'
#!/bin/sh
trap 'exit 0' TERM INT
while :; do sleep 1; done
RACE
chmod +x "$tmp/race/child"
setsid "$tmp/race/child" & race_pid=$!
for _ in $(seq 1 50); do kill -0 "$race_pid" 2>/dev/null && break; sleep .01; done
sleep .05
kill -TERM -- "-$race_pid" 2>/dev/null || kill -TERM "$race_pid" 2>/dev/null || true
wait "$race_pid" || true
! kill -0 "$race_pid" 2>/dev/null
rm -rf "$tmp/race"; [ ! -e "$tmp/race" ]
# Cleanup order and idempotent signal/exit guards remain explicit.
grep -Fq '[ "$cleanup_done" -eq 0 ] || return "$rc"' "$source_script"
grep -Fq 'terminate_local && local_processes_reaped=true' "$source_script"
grep -Fq 'remote_temp_path_removed=true' "$source_script"
grep -Fq 'trap - EXIT INT TERM' "$source_script"
echo compare-hy2-owned-lab-test-ok
