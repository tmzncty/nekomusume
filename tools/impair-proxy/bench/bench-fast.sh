#!/usr/bin/env bash
# High-throughput TCP/UDP bench using rust sender+sink (no python bottleneck).
# Compiles the helpers on first use. Usage: ./bench-fast.sh <mode tcp|udp> <window_ms> <mss>
set -u
MODE="${1:-tcp}"
WIN="${2:-2000}"
MSS="${3:-65536}"
HERE="$(cd "$(dirname "$0")" && pwd)"
PROXY="$(cd "$HERE/.." && pwd)/target/release/impair-proxy"
export PATH="$HOME/.cargo/bin:$PATH"

if [[ ! -x "$HERE/sink" ]]; then rustc -O "$HERE/sink.rs" -o "$HERE/sink" || exit 1; fi
if [[ ! -x "$HERE/send" ]]; then rustc -O "$HERE/send.rs" -o "$HERE/send" || exit 1; fi
if [[ ! -x "$PROXY" ]]; then echo "build first: cargo build --release" >&2; exit 1; fi

free_port() { python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1]); s.close()'; }

mbps() { python3 -c "import sys; b=int(sys.argv[1]); print(f'{b*8/1e6/($WIN/1000):.0f}')" "$1"; }

run() { # $1 port -> echoes bytes
    local port=$1
    "$HERE/sink" "$port" "$WIN" "$MODE" > "$HERE/.sink.out" &
    local sp=$!
    sleep 0.3
    "$HERE/send" "$port" "$WIN" "$MSS" "$MODE" >/dev/null 2>&1
    wait $sp
    cat "$HERE/.sink.out"
}

SP=$(free_port)
DIRECT=$(run $SP)
echo "mode=$MODE window=${WIN}ms mss=$MSS"
echo "direct : $(mbps "$DIRECT") Mbit/s ($DIRECT bytes)"

SP2=$(free_port)
PP=$(free_port)
"$HERE/sink" "$SP2" "$WIN" "$MODE" > "$HERE/.sink2.out" &
SPID=$!
"$PROXY" --mode "$MODE" --bind "127.0.0.1:$PP" --upstream "127.0.0.1:$SP2" \
    --stats-file "$HERE/.stats.json" --duration-ms $((WIN + 1500)) >/dev/null 2>&1 &
PXPID=$!
sleep 0.3
"$HERE/send" "$PP" "$WIN" "$MSS" "$MODE" >/dev/null 2>&1
wait $SPID
wait $PXPID 2>/dev/null
PX=$(cat "$HERE/.sink2.out")
echo "proxy  : $(mbps "$PX") Mbit/s ($PX bytes)"
python3 -c "import sys; d=int(sys.argv[1]); p=int(sys.argv[2]); print(f'ratio  : {p/d*100:.1f}%')" "$DIRECT" "$PX" 2>/dev/null || echo "ratio  : n/a"
rm -f "$HERE/.sink.out" "$HERE/.sink2.out" "$HERE/.stats.json"
