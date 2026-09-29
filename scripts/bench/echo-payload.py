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
# Failure-path stderr is the diagnostic seam consumed by the owned-lab harness
# (it flows into the private diagnostic bundle and is category-classified).
# Phrases are chosen so the validator classifies transport failures as "path"
# and payload-exchange failures as "payload"; avoid the literal "echo-payload"
# prefix here because its hyphen makes "payload" a \b-word that the "path"
# category pattern would capture first.
try:
    with socket.create_connection((a.host, a.port), timeout=a.timeout) as s:
        s.settimeout(a.timeout)
        s.sendall(payload)
        # 85461-decided benchmark posture (2026-09-30): send, receive the full
        # bounded response, then close. No pre-read half-close — hysteria
        # v2.9.3 client forwarding tears down both directions on either
        # io.Copy exit, so an early FIN discards the return path (see
        # docs/notes/hy2-hy2-1-client-exit-rootcause-20260930.md). Half-close
        # tolerance is a conformance question, not this benchmark's workload.
        chunks, received = [], 0
        while received < len(payload):
            chunk = s.recv(min(65536, len(payload) - received))
            if not chunk:
                break
            chunks.append(chunk)
            received += len(chunk)
except OSError as exc:
    raise SystemExit(f"payload exchange failed: {exc}")
echo = b"".join(chunks)
if received < len(payload):
    raise SystemExit(f"payload exchange truncated: connection closed after {received} of {len(payload)} bytes")
if echo != payload:
    raise SystemExit(f"payload exchange mismatch: echoed {received} of {len(payload)} bytes with different content")
print(json.dumps({
    "application_bytes": len(payload),
    "payload_sha256": hashlib.sha256(payload).hexdigest(),
    "fd_count": len(os.listdir("/proc/self/fd")),
    "wire_bytes": None,
}, separators=(",", ":")))
