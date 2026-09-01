#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$ROOT"
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
cat >"$tmp/adapter" <<'EOF'
#!/usr/bin/env bash
printf '{"application_bytes":%s,"payload_sha256":"%s","fd_count":4,"wire_bytes":null}\n' "$BENCH_PAYLOAD_BYTES" "$BENCH_PAYLOAD_SHA256"
EOF
chmod +x "$tmp/adapter"
export NEKO_BENCH_CMD="$tmp/adapter" HY2_BENCH_CMD="$tmp/adapter"
export BENCH_SERVER_ID=owned-fixture BENCH_ROUTE_ID=loopback BENCH_MTU=65536
export BENCH_SECURITY_PROFILE=authenticated-encrypted-fixture BENCH_LOAD_PROFILE=one-stream
export BENCH_TARGET_HOST=127.0.0.1 NEKO_ISOLATED_LAB=yes NEKO_ALLOW_COMMAND_EVAL=yes
export BENCH_RUNS=3 BENCH_PAYLOAD_BYTES=25 BENCH_TIMEOUT_SEC=2
(cd "$tmp" && "$ROOT/scripts/bench/compare-hy2.sh" good.json)
test "$(jq '[.samples[]|.failures]|add' "$tmp/good.json")" = 0
cat >"$tmp/bad" <<'EOF'
#!/usr/bin/env bash
printf '{"application_bytes":%s,"payload_sha256":"%064d","fd_count":4,"wire_bytes":null}\n' "$BENCH_PAYLOAD_BYTES" 0
EOF
chmod +x "$tmp/bad"; export HY2_BENCH_CMD="$tmp/bad"
(cd "$tmp" && "$ROOT/scripts/bench/compare-hy2.sh" bad.json)
test "$(jq '[.samples[]|select(.implementation=="hy2")|.failures]|add' "$tmp/bad.json")" = 3
if (cd "$tmp" && BENCH_TARGET_HOST=example.invalid "$ROOT/scripts/bench/compare-hy2.sh" wan.json) >/dev/null 2>&1; then
  echo 'compare-hy2 tests: WAN guard failed' >&2; exit 1
fi
echo 'compare HY2 tests: PASS'
