#!/usr/bin/env python3
import subprocess
from pathlib import Path

root = Path(__file__).resolve().parents[2]
parser = root / "scripts/bench/parse-listener.py"
filtered = (root / "fixtures/ss/listener-filtered.txt").read_text()
state_first = (root / "fixtures/ss/listener-state-first.txt").read_text()

def check(data, proto, addr, port, expected):
    result = subprocess.run(["python3", str(parser), proto, addr, str(port)], input=data, text=True)
    assert (result.returncode == 0) is expected, (proto, addr, port, result.returncode)

check(filtered, "tcp", "192.0.2.10", 443, True)
check(filtered, "udp", "2001:db8::10", 8443, True)
check(state_first, "tcp", "192.0.2.10", 443, True)
check(state_first, "udp", "2001:db8::10", 8443, True)
for bad in [
    ("tcp", "0.0.0.0", 443), ("udp", "::", 8443),
    ("udp", "192.0.2.10", 443), ("tcp", "192.0.2.11", 443),
    ("tcp", "127.0.0.1", 443), ("tcp", "192.0.2.10", 8443),
]:
    check(filtered, *bad, False)
check("tcp LISTEN 0 4096 192.0.2.10:443 0.0.0.0:*\n" + filtered, "tcp", "192.0.2.10", 443, False)
check("tcp LISTEN 0 4096 malformed 0.0.0.0:*", "tcp", "192.0.2.10", 443, False)
print("parse-listener fixtures: ok")
