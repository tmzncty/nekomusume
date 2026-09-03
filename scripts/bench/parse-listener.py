#!/usr/bin/env python3
import ipaddress, sys
if len(sys.argv) != 4 or sys.argv[1] not in ('tcp','udp'):
    raise SystemExit(2)
protocol, expected, port = sys.argv[1], sys.argv[2], int(sys.argv[3])
try: expected = ipaddress.ip_address(expected)
except ValueError: raise SystemExit(2)
found = 0
for line in sys.stdin:
    fields = line.split()
    if len(fields) < 5 or fields[0].lower() != protocol: continue
    local = fields[4]
    if local.startswith('['):
        end = local.rfind(']'); host, sep, p = local[1:end], True, local[end+2:]
    else:
        parts = local.rsplit(':', 1); host, p, sep = (parts[0], parts[1], True) if len(parts) == 2 else ('', '', False)
    if not sep or not p.isdigit() or int(p) != port: continue
    try: addr = ipaddress.ip_address(host)
    except ValueError: continue
    if addr == expected and not addr.is_unspecified and not addr.is_loopback: found += 1
if found != 1: raise SystemExit(1)
