#!/usr/bin/env bash
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd); source_script=$root/scripts/bench/compare-hy2-owned-lab.sh; tmp=$(mktemp -d); validation_out=artifacts/hy2-owned-lab/validation-test-$$.json; default_out=$root/artifacts/hy2-owned-lab/result.json; trap 'rm -f "$validation_out" "$validation_out.samples.jsonl"; rm -rf "$tmp"' EXIT
cp "$source_script" "$tmp/adapter.sh"; cp "$root/scripts/bench/owned-lab-control-plane.sh" "$tmp/"; s=$tmp/adapter.sh
cat >"$tmp/ssh" <<'SH'
#!/bin/sh
if [ "$1" = -G ]; then
  [ "${MOCK_SSH_CONFIG_FAIL:-0}" -eq 0 ] || exit 3
  printf '%s\n' 'hostname 192.0.2.8' "user ${LAB_SSH_MOCK_USER:-labuser}"
  exit
fi
case "${MOCK_SSH_REMOTE_RESULT:-success}" in
  success) printf '%s\n' "${MOCK_REMOTE_INTERFACES:-[]}";;
  auth) exit 255;;
  timeout) exit 124;;
  command) exit "${MOCK_SSH_COMMAND_RC:-7}";;
  *) exit 70;;
esac
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
base="PATH=$tmp:$PATH NEKO_OWNED_LAB=yes LAB_SSH_TARGET=owned LAB_ENDPOINT_ID=owned-vps LAB_ENDPOINT_SHA256=$hash LAB_REMOTE_ADDRESS=192.0.2.8 LAB_SSH_EXPECTED_USER=labuser NEKO_BIN=$tmp/neko HY2_BIN=$tmp/hy2"
interfaces='[{"ifname":"eth0","addr_info":[{"family":"inet","local":"192.0.2.9","prefixlen":24}]}]'
# A dedicated assigned address validates even when it differs from the SSH/connection address.
env $base LAB_REMOTE_BIND_ADDRESS=192.0.2.9 MOCK_REMOTE_INTERFACES="$interfaces" "$s" --validate "$validation_out" | grep -qx validated
[ ! -e "$validation_out" ] && [ ! -e "$validation_out.samples.jsonl" ]
# Fail closed when the dedicated address is absent or unsafe.
for address in '' 0.0.0.0 :: 127.0.0.1 ::1 224.0.0.1 ff02::1; do
  if env $base LAB_REMOTE_BIND_ADDRESS="$address" MOCK_REMOTE_INTERFACES="$interfaces" "$s" --validate "$validation_out" >/dev/null 2>&1; then echo "unsafe bind accepted: $address" >&2; exit 1; fi
done
if env $base LAB_REMOTE_BIND_ADDRESS=192.0.2.10 MOCK_REMOTE_INTERFACES="$interfaces" "$s" --validate "$validation_out" >/dev/null 2>&1; then echo 'nonlocal bind accepted' >&2; exit 1; fi
# Every advertised preflight case executes the adapter, retains a typed artifact,
# and removes its isolated disposable runtime directory without contacting a VPS.
runtime_root=$tmp/preflight-runtime; mkdir "$runtime_root"
artifact_root=artifacts/hy2-owned-lab/preflight-test-$$; mkdir -p "$artifact_root"
trap 'rm -rf "$artifact_root"; rm -f "$validation_out" "$validation_out.samples.jsonl"; rm -rf "$tmp"' EXIT
for spec in 'identity-match:ssh-bind-address:success:0:0:labuser' 'wrong-user:ssh-user-mismatch:success:0:0:other' 'config:ssh-config:success:0:1:labuser' 'auth:ssh-auth:auth:0:0:labuser' 'timeout:ssh-timeout:timeout:0:0:labuser' 'remote-command:ssh-command:command:7:0:labuser'; do
  IFS=: read -r mode expected_stage remote_result command_rc config_fail expected_user <<EOF
$spec
EOF
  out=$artifact_root/$mode.json
  interfaces_for_case=$interfaces; [ "$mode" != identity-match ] || interfaces_for_case=[]
  set +e
  env $base LAB_SSH_EXPECTED_USER="$expected_user" LAB_REMOTE_BIND_ADDRESS=192.0.2.9 \
    MOCK_REMOTE_INTERFACES="${interfaces_for_case:-$interfaces}" MOCK_SSH_REMOTE_RESULT="$remote_result" \
    MOCK_SSH_COMMAND_RC="$command_rc" MOCK_SSH_CONFIG_FAIL="$config_fail" \
    OWNED_LAB_RUNTIME_TEMPLATE="$runtime_root/$mode.XXXXXXXX" "$s" "$out" >/dev/null 2>&1
  rc=$?
  set -e
  [ "$rc" -eq 2 ] || { echo "$mode returned $rc, want 2" >&2; exit 1; }
  python3 "$root/scripts/bench/validate-hy2-owned-lab.py" validate-result "$out" | grep -qx validated
  jq -e --arg stage "$expected_stage" '.status=="BLOCKED_HARNESS" and .failure_stage==$stage and (.samples|length)==0 and .contract.payload_prepared==false and .contract.payload_sha256==null' "$out" >/dev/null
  jq -e '.cleanup_status=="failed" and .cleanup_evidence=={"local_processes_reaped":null,"local_listeners_remaining":null,"remote_process_groups_reaped":null,"remote_listeners_remaining":null,"remote_temp_path_removed":null}' "$out" >/dev/null
  unset interfaces_for_case
  [ -z "$(find "$runtime_root" -mindepth 1 -print -quit)" ] || { echo "$mode leaked test runtime residue" >&2; exit 1; }
done
rm -rf "$artifact_root"
# Wrong HY2 identity must fail before execution in the authoritative script.
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
# Execute the production run_client body under set -u without a VPS.  This reaches
# the real local initializer; the former same-statement $impl expansion aborts here.
awk '/^run_client\(\)\{/{copy=1} copy{print} copy && /^}$/{exit}' "$source_script" >"$tmp/run-client-production.sh"
(
  set -eu
  source "$tmp/run-client-production.sh"
  run="$tmp/run-client-runtime"; mkdir "$run"
  validator=unused; BYTES=1; TIMEOUT=1; payload_hash=$(printf x | sha256sum | awk '{print $1}')
  declare -A client_identity=([nekomusume]=neko-fixture [hy2]=hy2-fixture)
  failure_stage=fixture
  run_bounded(){ printf '%s\n' "$*" >>"$tmp/run-client-calls"; return 0; }
  python3(){ printf '%s\n' '{"failures":0}'; }
  atomic_append(){ cat "$1" >>"$tmp/run-client-appended.json"; }
  run_client nekomusume 1 40080 true
  run_client hy2 1 40081 true
  [ "$(grep -Fc '"failures":0' "$tmp/run-client-appended.json")" -eq 2 ]
  grep -Fq -- '--identity sha256:neko-fixture' "$tmp/run-client-calls"
  grep -Fq -- '--identity sha256:hy2-fixture' "$tmp/run-client-calls"
)
grep -Fq 'sampler-created process group' "$root/scripts/bench/validate-hy2-owned-lab.py"
! grep -Fq 'hy2_client_pid' "$source_script"
grep -Fq 'client_lifecycle:"fresh transport per timed sample"' "$source_script"
grep -Fq 'observed_clients != expected_clients' "$root/scripts/bench/validate-hy2-owned-lab.py"
# Endpoint mismatch and unsafe output remain rejected.
if env $base LAB_ENDPOINT_SHA256=$(printf %064d 0) LAB_REMOTE_BIND_ADDRESS=192.0.2.9 MOCK_REMOTE_INTERFACES="$interfaces" "$s" --validate "$validation_out" >/dev/null 2>&1; then exit 1; fi
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
python3 "$root/scripts/bench/validate-hy2-owned-lab.py" make-sample --implementation nekomusume --run 1 --return-code 9 --time "$tmp/neko-first.time" --resource "$tmp/neko-first.resource" --client-output "$tmp/neko-first.out" --client-diagnostics "$tmp/neko-first.out" --diagnostic-bundle "$tmp/first-private.json" --diagnostic-started-at 2026-09-03T00:00:00Z --diagnostic-ended-at 2026-09-03T00:00:01Z --diagnostic-stage client_started --bytes 1200 --payload-hash "$hash0" >"$tmp/first.jsonl" 2>"$tmp/make.err"
[ ! -s "$tmp/make.err" ]; [ "$(wc -l <"$tmp/first.jsonl")" -eq 1 ]
jq -e '.name=="nekomusume-1" and .failures==1 and .exit_code==9 and .failure_stage=="client_exit" and .application_bytes==0 and .payload_sha256==null and .client_diagnostic.category=="path" and .client_diagnostic.last_success_stage=="client_started"' "$tmp/first.jsonl" >/dev/null
! grep -q -- '--argjson' "$source_script"
python3 "$root/scripts/bench/validate-hy2-owned-lab.py" blocked --records "$tmp/first.jsonl" --output "$tmp/blocked.json" --stage nekomusume-1-client --commit "$(printf %040d 0)" --runs 5 --bytes 1200 --payload-prepared true --payload-hash "$hash0" --local-reaped true --local-listeners 0 --remote-reaped true --remote-listeners 0 --remote-path-removed true
python3 "$root/scripts/bench/validate-hy2-owned-lab.py" validate-result "$tmp/blocked.json" | grep -qx validated
jq -e '.status=="BLOCKED_HARNESS" and (.samples|length)==1 and .cleanup_status=="verified"' "$tmp/blocked.json" >/dev/null
# Canonical cleanup booleans preserve true and false exactly, including shell 1/0 conversion.
canonical_bool=$(awk '/^canonical_bool\(\)/{print; exit}' "$source_script")
[ "$(eval "$canonical_bool"; canonical_bool 1)" = true ]
[ "$(eval "$canonical_bool"; canonical_bool 0)" = false ]
[ "$(eval "$canonical_bool"; canonical_bool unknown)" = unknown ]
python3 "$root/scripts/bench/validate-hy2-owned-lab.py" blocked --records /dev/null --output "$tmp/cleanup-true.json" --stage cleanup --commit "$(printf %040d 0)" --runs 5 --bytes 1200 --payload-prepared false --local-reaped true --local-listeners 0 --remote-reaped true --remote-listeners 0 --remote-path-removed true
python3 "$root/scripts/bench/validate-hy2-owned-lab.py" blocked --records /dev/null --output "$tmp/cleanup-false.json" --stage cleanup --commit "$(printf %040d 0)" --runs 5 --bytes 1200 --payload-prepared false --local-reaped false --local-listeners 0 --remote-reaped false --remote-listeners 0 --remote-path-removed false
jq -e '.cleanup_evidence.local_processes_reaped==true and .cleanup_evidence.remote_process_groups_reaped==true and .cleanup_evidence.remote_temp_path_removed==true' "$tmp/cleanup-true.json" >/dev/null
jq -e '.cleanup_evidence.local_processes_reaped==false and .cleanup_evidence.remote_process_groups_reaped==false and .cleanup_evidence.remote_temp_path_removed==false' "$tmp/cleanup-false.json" >/dev/null
# Future nonzero client diagnostics retain bounded private evidence and only fixed public metadata.
for spec in 'tls:TLS certificate pin failed' 'auth:authentication failed password=hunter2' 'config:invalid config yaml' 'path:connection refused 10.23.45.67:443 /home/private/key' 'readiness:listener not ready'; do
  category=${spec%%:*}; message=${spec#*:}; printf '%s\n' "$message" >"$tmp/diagnostic.err"
  bundle="$tmp/$category-private.json"
  python3 "$root/scripts/bench/validate-hy2-owned-lab.py" make-sample --implementation hy2 --run 1 --return-code 9 --time "$tmp/neko-first.time" --resource "$tmp/neko-first.resource" --client-output /dev/null --client-diagnostics "$tmp/diagnostic.err" --diagnostic-bundle "$bundle" --diagnostic-started-at 2026-09-03T00:00:00Z --diagnostic-ended-at 2026-09-03T00:00:01Z --diagnostic-stage client_started --bytes 1200 --payload-hash "$hash0" >"$tmp/diagnostic.json"
  jq -e --arg category "$category" '.failure_stage=="client_exit" and .client_diagnostic.category==$category and .client_diagnostic.last_success_stage=="client_started" and (.client_diagnostic.bundle_sha256|length)==64 and .client_diagnostic.bundle_bytes>0 and .client_diagnostic.started_at=="2026-09-03T00:00:00Z" and .client_diagnostic.ended_at=="2026-09-03T00:00:01Z"' "$tmp/diagnostic.json" >/dev/null
  [ "$(sha256sum "$bundle"|awk '{print $1}')" = "$(jq -r .client_diagnostic.bundle_sha256 "$tmp/diagnostic.json")" ]
  [ "$(wc -c <"$bundle")" -eq "$(jq -r .client_diagnostic.bundle_bytes "$tmp/diagnostic.json")" ]
  ! grep -Eq 'hunter2|10\.23\.45\.67|/home/private' "$tmp/diagnostic.json" "$bundle"
done
printf '%s\n' 'QUIC UDP connection established to 10.23.45.67:443' >"$tmp/diagnostic.err"
python3 "$root/scripts/bench/validate-hy2-owned-lab.py" make-sample --implementation hy2 --run 1 --return-code 9 --time "$tmp/neko-first.time" --resource "$tmp/neko-first.resource" --client-output /dev/null --client-diagnostics "$tmp/diagnostic.err" --diagnostic-bundle "$tmp/quic-private.json" --diagnostic-started-at 2026-09-03T00:00:00Z --diagnostic-ended-at 2026-09-03T00:00:01Z --diagnostic-stage server_bound --bytes 1200 --payload-hash "$hash0" >"$tmp/quic.json"
jq -e '.client_diagnostic.last_success_stage=="quic_udp"' "$tmp/quic.json" >/dev/null
printf '%s\n' 'authentication succeeded credential=private-value' >"$tmp/diagnostic.err"
python3 "$root/scripts/bench/validate-hy2-owned-lab.py" make-sample --implementation hy2 --run 1 --return-code 9 --time "$tmp/neko-first.time" --resource "$tmp/neko-first.resource" --client-output /dev/null --client-diagnostics "$tmp/diagnostic.err" --diagnostic-bundle "$tmp/authenticated-private.json" --diagnostic-started-at 2026-09-03T00:00:00Z --diagnostic-ended-at 2026-09-03T00:00:01Z --diagnostic-stage client_started --bytes 1200 --payload-hash "$hash0" >"$tmp/authenticated.json"
jq -e '.client_diagnostic.last_success_stage=="tls_authenticated"' "$tmp/authenticated.json" >/dev/null
! grep -q private-value "$tmp/authenticated.json" "$tmp/authenticated-private.json"
# Input and private bundle remain bounded; private file permissions are owner-only.
{ printf 'connection refused password=top-secret pinSHA256=AA:BB private_key=/private/key host.internal.example 10.23.45.67:443 '; head -c 8192 /dev/zero | tr '\000' x; } >"$tmp/diagnostic.err"
python3 "$root/scripts/bench/validate-hy2-owned-lab.py" make-sample --implementation hy2 --run 1 --return-code 9 --time "$tmp/neko-first.time" --resource "$tmp/neko-first.resource" --client-output /dev/null --client-diagnostics "$tmp/diagnostic.err" --diagnostic-bundle "$tmp/bounded-private.json" --diagnostic-started-at 2026-09-03T00:00:00Z --diagnostic-ended-at 2026-09-03T00:00:01Z --diagnostic-stage client_started --bytes 1200 --payload-hash "$hash0" >"$tmp/bounded.json"
[ "$(wc -c <"$tmp/bounded-private.json")" -le 2560 ]
[ "$(stat -c %a "$tmp/bounded-private.json")" = 600 ]
jq -e '.truncated==true and (.sanitized_text|utf8bytelength)<=2048' "$tmp/bounded-private.json" >/dev/null
! grep -Eq 'top-secret|10\.23\.45\.67' "$tmp/bounded-private.json" "$tmp/bounded.json"
python3 "$root/scripts/bench/validate-hy2-owned-lab.py" make-sample --implementation hy2 --run 1 --return-code 9 --time "$tmp/neko-first.time" --resource "$tmp/neko-first.resource" --client-output /dev/null --client-diagnostics /dev/null --diagnostic-bundle "$tmp/absent.json" --diagnostic-started-at 2026-09-03T00:00:00Z --diagnostic-ended-at 2026-09-03T00:00:01Z --diagnostic-stage client_started --bytes 1200 --payload-hash "$hash0" >"$tmp/no-diagnostic.json"
jq -e '.failure_stage=="client_exit" and .client_diagnostic==null' "$tmp/no-diagnostic.json" >/dev/null
[ ! -e "$tmp/absent.json" ]
# A routine validation in a disposable repository cannot create or alter default evidence.
sentinel=$tmp/sentinel-repo; mkdir -p "$sentinel/scripts/bench" "$sentinel/artifacts/hy2-owned-lab"
cp "$source_script" "$root/scripts/bench/owned-lab-control-plane.sh" \
  "$root/scripts/bench/parse-listener.py" "$root/scripts/bench/validate-hy2-owned-lab.py" "$sentinel/scripts/bench/"
printf '\000real-result\n' >"$sentinel/artifacts/hy2-owned-lab/result.json"
printf 'sample\000companion\n' >"$sentinel/artifacts/hy2-owned-lab/result.json.samples.jsonl"
( cd "$sentinel"; git init -q; git config user.email test@example.invalid; git config user.name test; git add .; git commit -qm sentinel )
rm -f "$sentinel/artifacts/hy2-owned-lab/result.json" "$sentinel/artifacts/hy2-owned-lab/result.json.samples.jsonl"
( cd "$sentinel"; env $base LAB_REMOTE_BIND_ADDRESS=192.0.2.9 MOCK_REMOTE_INTERFACES="$interfaces" scripts/bench/compare-hy2-owned-lab.sh --validate | grep -qx validated )
[ ! -e "$sentinel/artifacts/hy2-owned-lab/result.json" ] && [ ! -e "$sentinel/artifacts/hy2-owned-lab/result.json.samples.jsonl" ]
printf '\000real-result\n' >"$sentinel/artifacts/hy2-owned-lab/result.json"
printf 'sample\000companion\n' >"$sentinel/artifacts/hy2-owned-lab/result.json.samples.jsonl"
before=$(sha256sum "$sentinel/artifacts/hy2-owned-lab/result.json" "$sentinel/artifacts/hy2-owned-lab/result.json.samples.jsonl")
( cd "$sentinel"; env $base LAB_REMOTE_BIND_ADDRESS=192.0.2.9 MOCK_REMOTE_INTERFACES="$interfaces" scripts/bench/compare-hy2-owned-lab.sh --validate | grep -qx validated )
after=$(sha256sum "$sentinel/artifacts/hy2-owned-lab/result.json" "$sentinel/artifacts/hy2-owned-lab/result.json.samples.jsonl")
[ "$before" = "$after" ]
# Cleanup order and fail-closed evidence contracts remain explicit.
grep -Fq '[ "$cleanup_done" -eq 0 ] || return "$rc"' "$source_script"
grep -Fq 'olcp_cleanup_owned "$roots_file" 100 "${ports[@]}"' "$source_script"
grep -Fq "olcp_cleanup_owned '\$remote/pids' 100" "$source_script"
grep -Fq 'remote_temp_path_removed=true' "$source_script"
grep -Fq 'trap - EXIT INT TERM' "$source_script"
echo compare-hy2-owned-lab-test-ok
