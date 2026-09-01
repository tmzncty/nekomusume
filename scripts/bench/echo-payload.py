#!/usr/bin/env python3
"""One bounded exact-payload TCP echo exchange for comparison wrappers."""
import argparse
import hashlib
import json
import os
import socket
from pathlib import Path

p = argparse.ArgumentParser()
p.add_argument("--host", required=True)
p.add_argument("--port", type=int, required=True)
p.add_argument("--payload-file", type=Path, required=True)
p.add_argument("--timeout", type=float, default=10.0)
a = p.parse_args()
if not 1024 <= a.port <= 65535:
    p.error("port must be unprivileged")
if not 0 < a.timeout <= 30:
    p.error("timeout must be in (0,30]")
payload = a.payload_file.read_bytes()
if not 0 < len(payload) <= 1024 * 1024:
    p.error("payload must be 1..1048576 bytes")
with socket.create_connection((a.host, a.port), timeout=a.timeout) as s:
    s.settimeout(a.timeout)
    s.sendall(payload)
    s.shutdown(socket.SHUT_WR)
    chunks, received = [], 0
    while received < len(payload):
        chunk = s.recv(min(65536, len(payload) - received))
        if not chunk:
            break
        chunks.append(chunk)
        received += len(chunk)
echo = b"".join(chunks)
if echo != payload:
    raise SystemExit("echo-payload: exact payload mismatch")
print(json.dumps({
    "application_bytes": len(payload),
    "payload_sha256": hashlib.sha256(payload).hexdigest(),
    "fd_count": len(os.listdir("/proc/self/fd")),
    "wire_bytes": None,
}, separators=(",", ":")))
