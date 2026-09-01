#!/usr/bin/env python3
"""Dependency-free fail-closed validator for process-resource.v1 JSON."""
import argparse, datetime, json, math, re, sys
from pathlib import Path

ID = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$")
TOP = {"schema_version", "experiment_id", "implementation", "role", "identity", "started_at", "ended_at", "elapsed_seconds", "exit", "cpu", "rss", "fd", "owned_experimental_sockets", "application_bytes", "sampling", "cleanup"}

def bad(msg): raise ValueError(msg)
def exact(obj, keys, where):
    if not isinstance(obj, dict) or set(obj) != set(keys): bad(f"{where}: unexpected or missing fields")
def number(v, lo, hi, where, nullable=False, integer=False):
    if nullable and v is None: return
    if isinstance(v, bool) or not isinstance(v, (int if integer else (int, float))) or not math.isfinite(v) or v < lo or v > hi: bad(where)
def timestamp(value, where):
    if not isinstance(value, str) or not value.endswith("Z"): bad(where)
    try: datetime.datetime.fromisoformat(value[:-1] + "+00:00")
    except ValueError: bad(where)
def paired(metric, values):
    present = [values[k] is not None for k in metric]
    if len(set(present)) != 1: bad("metric values/source nullability mismatch")
def validate(d):
    exact(d, TOP, "root")
    if d["schema_version"] != "nekomusume.process-resource.v1": bad("schema_version")
    for k in ("experiment_id", "implementation", "role"):
        if not isinstance(d[k], str) or not ID.fullmatch(d[k]): bad(k)
    if not isinstance(d["identity"], str) or not 1 <= len(d["identity"]) <= 192 or not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._:+/@=-]*", d["identity"]): bad("identity")
    timestamp(d["started_at"], "started_at"); timestamp(d["ended_at"], "ended_at")
    number(d["elapsed_seconds"], 0, 602, "elapsed_seconds")
    exact(d["exit"], {"code", "signal", "timed_out"}, "exit")
    number(d["exit"]["code"], 0, 255, "exit.code", True, True); number(d["exit"]["signal"], 1, 127, "exit.signal", True, True)
    if (d["exit"]["code"] is None) == (d["exit"]["signal"] is None) or type(d["exit"]["timed_out"]) is not bool: bad("exit")
    exact(d["cpu"], {"user_seconds", "system_seconds", "source"}, "cpu")
    number(d["cpu"]["user_seconds"], 0, 1e9, "cpu.user", True); number(d["cpu"]["system_seconds"], 0, 1e9, "cpu.system", True)
    paired(("user_seconds", "system_seconds", "source"), d["cpu"])
    exact(d["rss"], {"max_kib", "unit", "source"}, "rss"); number(d["rss"]["max_kib"], 1, 2**63-1, "rss.max", True, True)
    if d["rss"]["unit"] != "KiB": bad("rss.unit"); paired(("max_kib", "source"), d["rss"])
    exact(d["fd"], {"peak_count", "source"}, "fd"); number(d["fd"]["peak_count"], 0, 2**31-1, "fd.peak", True, True); paired(("peak_count", "source"), d["fd"])
    s=d["owned_experimental_sockets"]; exact(s, {"peak_count", "ports_supplied_count", "source"}, "sockets")
    number(s["peak_count"], 0, 2**31-1, "socket.peak", True, True); number(s["ports_supplied_count"], 0, 32, "socket.ports", integer=True)
    if s["ports_supplied_count"] == 0:
        if s["peak_count"] is not None or s["source"] is not None: bad("socket metric must be null without supplied ports")
    else: paired(("peak_count", "source"), s)
    number(d["application_bytes"], 0, 268435456, "application_bytes", integer=True)
    q=d["sampling"]; exact(q, {"interval_ms", "sample_count", "max_seconds", "scope"}, "sampling")
    number(q["interval_ms"], 10, 5000, "interval", integer=True); number(q["sample_count"], 1, 60001, "samples", integer=True); number(q["max_seconds"], .1, 600, "max_seconds")
    if q["scope"] != "direct child process only": bad("sampling.scope")
    c=d["cleanup"]; exact(c, {"process_reaped", "owned_sockets_after_exit", "complete", "scope"}, "cleanup")
    if c != {"process_reaped": True, "owned_sockets_after_exit": 0, "complete": True, "scope": "sampled direct child only"}: bad("cleanup")
    for metric in (d["cpu"], d["rss"], d["fd"], s):
        if metric.get("source") is not None and (not isinstance(metric["source"], str) or len(metric["source"]) > 256): bad("source")

def main():
    p=argparse.ArgumentParser(); p.add_argument("sample"); a=p.parse_args()
    try:
        raw=Path(a.sample).read_bytes()
        if len(raw)>65536: bad("document too large")
        text=raw.decode("utf-8")
        d=json.loads(text, parse_constant=lambda x: bad("non-finite number"))
        validate(d)
    except (OSError, UnicodeError, json.JSONDecodeError, ValueError) as e:
        print(f"invalid process resource sample: {e}", file=sys.stderr); return 1
    print(f"valid process resource sample: {a.sample}"); return 0
if __name__ == "__main__": sys.exit(main())
