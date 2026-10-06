#!/usr/bin/env bash
# Throughput benchmark for impair-proxy (loopback, single direction).
#
# Measures c2s forwarding throughput through the proxy vs a direct baseline:
#   sender --(UDP/TCP)--> [proxy] --> sink
#   sender --(UDP/TCP)----------------> sink   (baseline)
#
# Usage: ./bench.sh [release-bin-path]
# Env:   BENCH_MODE=udp|tcp  BENCH_WINDOW=2.0  BENCH_MSS=1400
set -euo pipefail

BIN="${1:-$(cd "$(dirname "$0")" && pwd)/target/release/impair-proxy}"
WINDOW="${BENCH_WINDOW:-2.0}"
MODE="${BENCH_MODE:-udp}"
MSS="${BENCH_MSS:-1400}"

if [[ ! -x "$BIN" ]]; then
    echo "binary not found: $BIN (run: cargo build --release)" >&2
    exit 1
fi

FREE_PORT() { python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1]); s.close()'; }
SINK_PORT=$(FREE_PORT)
PROXY_PORT=$(FREE_PORT)

# sink: counts payload bytes for WINDOW seconds from first byte, prints "BYTES PKTS"
SINK_PY='import socket, sys, time
mode, port, window = sys.argv[1], int(sys.argv[2]), float(sys.argv[3])
if mode == "udp":
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.bind(("127.0.0.1", port))
    s.settimeout(0.05)
    total = 0; start = None; pkts = 0
    while True:
        try:
            d, _ = s.recvfrom(65535)
        except socket.timeout:
            if start is not None and time.monotonic() - start > window + 0.5:
                break
            continue
        if start is None:
            start = time.monotonic()
        total += len(d); pkts += 1
        if time.monotonic() - start > window:
            break
    print(f"{total} {pkts}")
else:
    srv = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    srv.bind(("127.0.0.1", port)); srv.listen(4)
    srv.settimeout(0.05)
    total = 0; start = None; conns = []
    while True:
        try:
            c, _ = srv.accept()
            c.settimeout(0.05)
            conns.append(c)
        except socket.timeout:
            pass
        for c in list(conns):
            try:
                d = c.recv(1 << 16)
                if d:
                    if start is None:
                        start = time.monotonic()
                    total += len(d)
                else:
                    conns.remove(c)
            except socket.timeout:
                pass
            except OSError:
                conns.remove(c)
        if start is not None and time.monotonic() - start > window:
            break
    print(f"{total} 0")
'

# sender: saturating send for WINDOW+0.2 s; tolerates sink teardown at the end
SENDER_PY='import socket, sys, time
mode, port, window, mss = sys.argv[1], int(sys.argv[2]), float(sys.argv[3]), int(sys.argv[4])
deadline = time.monotonic() + window + 0.2
payload = bytes(mss)
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM if mode == "udp" else socket.SOCK_STREAM)
if mode != "udp":
    s.connect(("127.0.0.1", port))
while time.monotonic() < deadline:
    try:
        if mode == "udp":
            for _ in range(200):
                s.sendto(payload, ("127.0.0.1", port))
        else:
            for _ in range(64):
                s.sendall(payload)
    except OSError:
        break
'

run_case() { # $1=label $2=target_port  -> sets BYTES
    local out; out=$(mktemp)
    python3 -c "$SINK_PY" "$MODE" "$2" "$WINDOW" > "$out" &
    local sink_pid=$!
    sleep 0.3
    python3 -c "$SENDER_PY" "$MODE" "$2" "$WINDOW" "$MSS"
    wait $sink_pid
    read -r BYTES _ < "$out"
    rm -f "$out"
}

mbps() { python3 -c "print(f'{$1*8/1e6/$WINDOW:.1f}')"; }

echo "== mode=$MODE window=${WINDOW}s mss=$MSS =="

# direct baseline
run_case direct "$SINK_PORT"
DIRECT=$BYTES
echo "direct : $(mbps "$DIRECT") Mbit/s ($DIRECT bytes)"

# through proxy
PROXY_STATS=$(mktemp)
"$BIN" --mode "$MODE" --bind "127.0.0.1:$PROXY_PORT" --upstream "127.0.0.1:$SINK_PORT" \
    --stats-file "$PROXY_STATS" --duration-ms $(( ${WINDOW%.*} * 1000 + 1500 )) &
PROXY_PID=$!
sleep 0.3
python3 -c "$SENDER_PY" "$MODE" "$PROXY_PORT" "$WINDOW" "$MSS" || true
OUT=$(mktemp)
python3 -c "$SINK_PY" "$MODE" "$SINK_PORT" "$WINDOW" > "$OUT" &
SINK_PID=$!
# re-run sender now that proxy+sink are both up
python3 -c "$SENDER_PY" "$MODE" "$PROXY_PORT" "$WINDOW" "$MSS" || true
wait $SINK_PID
read -r PROXY _ < "$OUT"
wait $PROXY_PID 2>/dev/null || true
echo "proxy  : $(mbps "$PROXY") Mbit/s ($PROXY bytes)"
python3 - "$DIRECT" "$PROXY" <<'EOF'
import sys
d, p = int(sys.argv[1]), int(sys.argv[2])
print(f"ratio  : {p/d*100:.1f}% of direct" if d else "ratio  : n/a")
EOF
rm -f "$OUT" "$PROXY_STATS"
