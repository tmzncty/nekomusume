#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$ROOT"
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
cat >"$tmp/adapter" <<'EOF_ADAPTER'
#!/usr/bin/env bash
printf '{"application_bytes":%s,"payload_sha256":"%s","fd_count":4,"wire_bytes":null}\n' "$BENCH_PAYLOAD_BYTES" "$BENCH_PAYLOAD_SHA256"
EOF_ADAPTER
chmod +x "$tmp/adapter"
export NEKO_BENCH_CMD="$tmp/adapter" HY2_BENCH_CMD="$tmp/adapter"
export BENCH_SERVER_ID=owned-fixture BENCH_ROUTE_ID=loopback BENCH_MTU=65536
export BENCH_SECURITY_PROFILE=authenticated-encrypted-fixture BENCH_LOAD_PROFILE=one-stream
export BENCH_TARGET_HOST=127.0.0.1 NEKO_ISOLATED_LAB=yes NEKO_ALLOW_COMMAND_EVAL=yes
export BENCH_RUNS=3 BENCH_PAYLOAD_BYTES=25 BENCH_TIMEOUT_SEC=2
run_compare(){ (cd "$tmp" && "$ROOT/scripts/bench/compare-hy2.sh" "$1"); }
assert_hy2_failures(){
  local file=$1 expected=$2
  test "$(jq '[.samples[]|select(.implementation=="hy2")|.failures]|add' "$tmp/$file")" = "$expected"
  jq -e '.samples | length == 6' "$tmp/$file" >/dev/null
}
run_compare good.json
assert_hy2_failures good.json 0
jq -e '[.samples[].wire_bytes] | all(. == null)' "$tmp/good.json" >/dev/null
run_compare repeat.json
jq -e '.schema == "nekomusume.benchmark-result.v1" and ([.samples[].failures] | add == 0)' "$tmp/repeat.json" >/dev/null

write_hy2(){
  printf '#!/usr/bin/env bash\n%s\n' "$1" >"$tmp/hy2"
  chmod +x "$tmp/hy2"
  export HY2_BENCH_CMD="$tmp/hy2"
}
run_bad_case(){
  local name=$1 body=$2
  write_hy2 "$body"
  run_compare "$name.json"
  assert_hy2_failures "$name.json" 3
}

write_hy2 'printf '\''{"application_bytes":%s,"payload_sha256":"%s","fd_count":4,"wire_bytes":1234}\n'\'' "$BENCH_PAYLOAD_BYTES" "$BENCH_PAYLOAD_SHA256"'
run_compare wire.json
assert_hy2_failures wire.json 0
jq -e '[.samples[]|select(.implementation=="hy2")|.wire_bytes] | all(. == 1234)' "$tmp/wire.json" >/dev/null

run_bad_case wrong-hash 'printf '\''{"application_bytes":%s,"payload_sha256":"%064d","fd_count":4,"wire_bytes":null}\n'\'' "$BENCH_PAYLOAD_BYTES" 0'
jq -e '[.samples[]|select(.implementation=="hy2")] | all(.payload_sha256 == ("0" * 64) and .application_bytes == 25 and .fd_count == 4 and .wire_bytes == null)' "$tmp/wrong-hash.json" >/dev/null
run_bad_case empty ':'
run_bad_case malformed 'printf '\''{"application_bytes":'\'''
run_bad_case contaminated 'printf '\''{"application_bytes":%s,"payload_sha256":"%s","fd_count":4,"wire_bytes":null}\ngarbage\n'\'' "$BENCH_PAYLOAD_BYTES" "$BENCH_PAYLOAD_SHA256"'
run_bad_case multiple 'printf '\''{"application_bytes":%s,"payload_sha256":"%s","fd_count":4,"wire_bytes":null}\n{}\n'\'' "$BENCH_PAYLOAD_BYTES" "$BENCH_PAYLOAD_SHA256"'
run_bad_case array 'printf '\''[]\n'\'''
run_bad_case scalar 'printf '\''42\n'\'''
run_bad_case missing-fd 'printf '\''{"application_bytes":%s,"payload_sha256":"%s","wire_bytes":null}\n'\'' "$BENCH_PAYLOAD_BYTES" "$BENCH_PAYLOAD_SHA256"'
run_bad_case missing-application 'printf '\''{"payload_sha256":"%s","fd_count":4,"wire_bytes":null}\n'\'' "$BENCH_PAYLOAD_SHA256"'
run_bad_case missing-hash 'printf '\''{"application_bytes":%s,"fd_count":4,"wire_bytes":null}\n'\'' "$BENCH_PAYLOAD_BYTES"'
run_bad_case wrong-fd-type 'printf '\''{"application_bytes":%s,"payload_sha256":"%s","fd_count":"4","wire_bytes":null}\n'\'' "$BENCH_PAYLOAD_BYTES" "$BENCH_PAYLOAD_SHA256"'
run_bad_case wrong-wire-type 'printf '\''{"application_bytes":%s,"payload_sha256":"%s","fd_count":4,"wire_bytes":"unknown"}\n'\'' "$BENCH_PAYLOAD_BYTES" "$BENCH_PAYLOAD_SHA256"'
run_bad_case wrong-application-type 'printf '\''{"application_bytes":"%s","payload_sha256":"%s","fd_count":4,"wire_bytes":null}\n'\'' "$BENCH_PAYLOAD_BYTES" "$BENCH_PAYLOAD_SHA256"'
run_bad_case wrong-hash-type 'printf '\''{"application_bytes":%s,"payload_sha256":0,"fd_count":4,"wire_bytes":null}\n'\'' "$BENCH_PAYLOAD_BYTES"'

if (cd "$tmp" && BENCH_TARGET_HOST=example.invalid "$ROOT/scripts/bench/compare-hy2.sh" wan.json) >/dev/null 2>&1; then
  echo 'compare-hy2 tests: WAN guard failed' >&2; exit 1
fi
echo 'compare HY2 tests: PASS'
