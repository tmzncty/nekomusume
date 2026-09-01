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
grep -Fq 'server: ${bind_authority}:${ports[1]}' "$source_script"
grep -Fq -- '--bind '"'"'${bind_authority}:${ports[0]}'"'"'' "$source_script"
! grep -Eq 'listen: :|listen: 0\.0\.0\.0:|listen: \[::\]:' "$source_script"
! grep -Eq '/etc/hysteria|systemctl|pkill.*hysteria|0\.0\.0\.0.*hy2' "$source_script"
grep -q "HY2 artifact is not pinned" "$source_script"
# Endpoint mismatch and unsafe output remain rejected.
if env $base LAB_ENDPOINT_SHA256=$(printf %064d 0) LAB_REMOTE_BIND_ADDRESS=192.0.2.9 MOCK_REMOTE_INTERFACES="$interfaces" "$s" --validate >/dev/null 2>&1; then exit 1; fi
grep -q "BENCH_RUNS must be 3..10" "$source_script"; grep -q "ports must be distinct" "$source_script"
echo compare-hy2-owned-lab-test-ok
