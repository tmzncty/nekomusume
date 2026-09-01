#!/usr/bin/env bash
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd); s=$root/scripts/bench/compare-hy2-owned-lab.sh; tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
cat >"$tmp/ssh" <<'SH'
#!/bin/sh
[ "$1" = -G ] && { echo 'hostname 192.0.2.8'; exit; }
exit 0
SH
chmod +x "$tmp/ssh"; cp /bin/true "$tmp/neko"; cp /bin/true "$tmp/hy2"
hash=$(printf %s 192.0.2.8|sha256sum|awk '{print $1}')
base="PATH=$tmp:$PATH NEKO_OWNED_LAB=yes LAB_SSH_TARGET=owned LAB_ENDPOINT_ID=owned-vps LAB_ENDPOINT_SHA256=$hash LAB_REMOTE_ADDRESS=192.0.2.8 NEKO_BIN=$tmp/neko HY2_BIN=$tmp/hy2"
# Wrong HY2 identity must fail before execution.
if env $base "$s" --validate >/dev/null 2>&1; then echo 'unpinned HY2 accepted' >&2; exit 1; fi
# Static safety contracts: separate from loopback harness; no production config/service operations.
grep -q "HY2 artifact is not pinned" "$s"; ! grep -Eq '/etc/hysteria|systemctl|pkill.*hysteria|0\.0\.0\.0.*hy2' "$s"
# Endpoint mismatch and unsafe output must be rejected.
if env $base LAB_ENDPOINT_SHA256=$(printf %064d 0) "$s" --validate >/dev/null 2>&1; then exit 1; fi
grep -q "BENCH_RUNS must be 3..10" "$s"; grep -q "ports must be distinct" "$s"
echo compare-hy2-owned-lab-test-ok
