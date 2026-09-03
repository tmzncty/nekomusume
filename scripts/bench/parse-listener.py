#!/usr/bin/env python3
"""Strictly match one externally-owned listener in ss -H output."""
import ipaddress
import sys


def parse(argv, stream):
    if len(argv) != 4 or argv[1] not in ("tcp", "udp"):
        return 2
    protocol, expected_text, port_text = argv[1:]
    try:
        expected = ipaddress.ip_address(expected_text)
        port = int(port_text)
    except ValueError:
        return 2
    if not 1 <= port <= 65535:
        return 2
    states = {"tcp": "LISTEN", "udp": "UNCONN"}
    matches = 0
    for line in stream:
        fields = line.split()
        if not fields:
            continue
        # ss -H -ltn/-lun emits STATE first; some filtered variants retain NETID.
        if fields[0].lower() == protocol:
            if len(fields) < 5 or fields[1].upper() != states[protocol]:
                continue
            local = fields[4]
        else:
            if len(fields) < 4 or fields[0].upper() != states[protocol]:
                continue
            local = fields[3]
        if local.startswith("["):
            end = local.find("]")
            if end < 0 or end + 2 > len(local) or local[end + 1] != ":":
                continue
            host, port_text = local[1:end], local[end + 2 :]
        else:
            host, separator, port_text = local.rpartition(":")
            if not separator:
                continue
        if not port_text.isdigit() or int(port_text) != port:
            continue
        try:
            address = ipaddress.ip_address(host)
        except ValueError:
            continue
        if address == expected and not address.is_unspecified and not address.is_loopback:
            matches += 1
    return 0 if matches == 1 else 1


if __name__ == "__main__":
    raise SystemExit(parse(sys.argv, sys.stdin))
