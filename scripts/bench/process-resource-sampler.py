#!/usr/bin/env python3
"""Finite Linux process resource sampler for bounded benchmark children."""
from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import re
import signal
import sys
import time
from pathlib import Path

SCHEMA_VERSION = "nekomusume.process-resource.v1"
ID_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$")
IDENTITY_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._:+/@=-]{0,191}$")


def fail(message: str) -> "None":
    raise SystemExit(f"resource-sampler: {message}")


def iso_utc(now: dt.datetime | None = None) -> str:
    return (now or dt.datetime.now(dt.timezone.utc)).isoformat(timespec="milliseconds").replace("+00:00", "Z")


def read_proc(pid: int, owned_ports: set[int]):
    cpu = rss = fd_count = socket_count = None
    sources = {"cpu": None, "rss": None, "fd": None, "socket": None}
    try:
        stat = Path(f"/proc/{pid}/stat").read_text().split()
        ticks = os.sysconf("SC_CLK_TCK")
        cpu = (int(stat[13]) / ticks, int(stat[14]) / ticks)
        sources["cpu"] = "/proc/<pid>/stat"
    except (OSError, ValueError, IndexError):
        pass
    try:
        for line in Path(f"/proc/{pid}/status").read_text().splitlines():
            if line.startswith("VmRSS:"):
                rss = int(line.split()[1])
                sources["rss"] = "/proc/<pid>/status VmRSS"
                break
    except (OSError, ValueError, IndexError):
        pass
    socket_inodes: set[str] = set()
    try:
        entries = list(Path(f"/proc/{pid}/fd").iterdir())
        fd_count = len(entries)
        sources["fd"] = "/proc/<pid>/fd"
        for entry in entries:
            try:
                target = os.readlink(entry)
            except OSError:
                continue
            match = re.fullmatch(r"socket:\[(\d+)\]", target)
            if match:
                socket_inodes.add(match.group(1))
    except OSError:
        pass
    if sources["fd"] and owned_ports:
        try:
            owned_inodes: set[str] = set()
            # tcp state 0A is LISTEN. UDP has no listen state; an explicitly
            # supplied local port is sufficient, and remains process-scoped by fd inode.
            for name in ("tcp", "tcp6", "udp", "udp6"):
                lines = Path(f"/proc/net/{name}").read_text().splitlines()[1:]
                for line in lines:
                    fields = line.split()
                    local_port = int(fields[1].rsplit(":", 1)[1], 16)
                    if local_port not in owned_ports:
                        continue
                    if name.startswith("tcp") and fields[3] != "0A":
                        continue
                    owned_inodes.add(fields[9])
            socket_count = len(socket_inodes & owned_inodes)
            sources["socket"] = "/proc/<pid>/fd + /proc/net/{tcp,tcp6,udp,udp6}; caller-owned ports only"
        except (OSError, ValueError, IndexError):
            pass
    return cpu, rss, fd_count, socket_count, sources


def maximum(current, candidate):
    return candidate if current is None else current if candidate is None else max(current, candidate)


def parse_args():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--experiment-id", required=True)
    p.add_argument("--implementation", required=True)
    p.add_argument("--role", required=True)
    p.add_argument("--identity", required=True, help="caller-supplied git or binary identity; no probing")
    p.add_argument("--application-bytes", required=True, type=int)
    p.add_argument("--owned-port", action="append", type=int, default=[])
    p.add_argument("--interval-ms", type=int, default=50)
    p.add_argument("--max-seconds", type=float, default=30.0)
    p.add_argument("--output", required=True)
    p.add_argument("command", nargs=argparse.REMAINDER)
    a = p.parse_args()
    if a.command and a.command[0] == "--":
        a.command = a.command[1:]
    for label, value in (("experiment id", a.experiment_id), ("implementation", a.implementation), ("role", a.role)):
        if not ID_RE.fullmatch(value): fail(f"invalid {label}")
    if not IDENTITY_RE.fullmatch(a.identity): fail("invalid identity")
    if not 10 <= a.interval_ms <= 5000: fail("interval-ms must be 10..5000")
    if not 0.1 <= a.max_seconds <= 600: fail("max-seconds must be 0.1..600")
    if not 0 <= a.application_bytes <= 256 * 1024 * 1024: fail("application-bytes exceeds bounded range")
    if len(a.owned_port) > 32 or any(not 1024 <= port <= 65535 for port in a.owned_port) or len(set(a.owned_port)) != len(a.owned_port):
        fail("owned-port must be unique unprivileged ports (max 32)")
    if not a.command or len(a.command) > 64 or any(not arg or len(arg) > 4096 or "\x00" in arg for arg in a.command):
        fail("invalid or missing bounded command")
    out = Path(a.output)
    if not out.is_absolute() or out.exists() or not out.parent.is_dir(): fail("output must be a new absolute path in an existing directory")
    return a


def main():
    a = parse_args()
    start_wall = dt.datetime.now(dt.timezone.utc)
    start_mono = time.monotonic()
    pid = os.fork()
    if pid == 0:
        try:
            os.setpgid(0, 0)
            os.execvp(a.command[0], a.command)
        except OSError as exc:
            print(f"resource-sampler child exec failed: {exc}", file=sys.stderr)
            os._exit(127)
    try:
        os.setpgid(pid, pid)
    except (OSError, ProcessLookupError):
        pass  # child may already have execed or exited
    peak_rss = peak_fd = peak_socket = None
    samples = 0
    last_sources = {"cpu": None, "rss": None, "fd": None, "socket": None}
    last_cpu = (None, None)
    timed_out = False
    status = usage = None
    deadline = start_mono + a.max_seconds
    while status is None:
        cpu, rss, fds, sockets, sources = read_proc(pid, set(a.owned_port))
        samples += 1
        if cpu[0] is not None: last_cpu = cpu
        for key, value in sources.items():
            if value is not None: last_sources[key] = value
        peak_rss, peak_fd, peak_socket = maximum(peak_rss, rss), maximum(peak_fd, fds), maximum(peak_socket, sockets)
        waited, raw_status, rusage = os.wait4(pid, os.WNOHANG)
        if waited == pid:
            status, usage = raw_status, rusage
            break
        if time.monotonic() >= deadline:
            timed_out = True
            try:
                os.killpg(pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            grace = time.monotonic() + 1.0
            while time.monotonic() < grace:
                waited, raw_status, rusage = os.wait4(pid, os.WNOHANG)
                if waited == pid:
                    status, usage = raw_status, rusage
                    break
                time.sleep(0.01)
            if status is None:
                try:
                    os.killpg(pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                _, status, usage = os.wait4(pid, 0)
            break
        time.sleep(a.interval_ms / 1000)
    end_wall = dt.datetime.now(dt.timezone.utc)
    elapsed = time.monotonic() - start_mono
    if usage is not None:
        cpu_user, cpu_system = usage.ru_utime, usage.ru_stime
        cpu_source = "wait4 rusage"
        peak_rss = maximum(peak_rss, usage.ru_maxrss)
        rss_source = "max(sampled /proc VmRSS, wait4 ru_maxrss); Linux KiB"
    else:
        cpu_user, cpu_system = last_cpu
        cpu_source, rss_source = last_sources["cpu"], last_sources["rss"]
    exit_code = os.waitstatus_to_exitcode(status)
    result = {
        "schema_version": SCHEMA_VERSION,
        "experiment_id": a.experiment_id,
        "implementation": a.implementation,
        "role": a.role,
        "identity": a.identity,
        "started_at": iso_utc(start_wall), "ended_at": iso_utc(end_wall), "elapsed_seconds": round(elapsed, 6),
        "exit": {"code": exit_code if exit_code >= 0 else None, "signal": -exit_code if exit_code < 0 else None, "timed_out": timed_out},
        "cpu": {"user_seconds": round(cpu_user, 6) if cpu_user is not None else None, "system_seconds": round(cpu_system, 6) if cpu_system is not None else None, "source": cpu_source},
        "rss": {"max_kib": peak_rss, "unit": "KiB", "source": rss_source if peak_rss is not None else None},
        "fd": {"peak_count": peak_fd, "source": last_sources["fd"] if peak_fd is not None else None},
        "owned_experimental_sockets": {"peak_count": peak_socket, "ports_supplied_count": len(a.owned_port), "source": last_sources["socket"] if peak_socket is not None else None},
        "application_bytes": a.application_bytes,
        "sampling": {"interval_ms": a.interval_ms, "sample_count": samples, "max_seconds": a.max_seconds, "scope": "direct child process only"},
        "cleanup": {"process_reaped": True, "owned_sockets_after_exit": 0, "complete": True, "scope": "sampled direct child only"},
    }
    tmp = Path(str(a.output) + ".tmp")
    try:
        tmp.write_text(json.dumps(result, sort_keys=True, indent=2) + "\n")
        os.chmod(tmp, 0o600)
        os.replace(tmp, a.output)
    finally:
        try: tmp.unlink()
        except FileNotFoundError: pass
    return exit_code if exit_code >= 0 else 128 - exit_code

if __name__ == "__main__":
    sys.exit(main())
