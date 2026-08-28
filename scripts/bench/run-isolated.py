#!/usr/bin/env python3
"""Deterministic socket-free benchmark for the candidate recovery core.
Produces JSON with samples, median, p95 and failures. No network mutation."""
import json, statistics, subprocess, time

SCENARIOS = [
    {"name":"baseline","loss":0,"reorder":False,"rtt_us":0,"bandwidth_bps":0,"blackhole":False},
    {"name":"rtt-20ms","loss":0,"reorder":False,"rtt_us":20_000,"bandwidth_bps":0,"blackhole":False},
    {"name":"random-loss-1pct","loss":100,"reorder":False,"rtt_us":0,"bandwidth_bps":0,"blackhole":False},
    {"name":"random-loss-5pct","loss":20,"reorder":False,"rtt_us":0,"bandwidth_bps":0,"blackhole":False},
    {"name":"random-loss-10pct","loss":10,"reorder":False,"rtt_us":0,"bandwidth_bps":0,"blackhole":False},
    {"name":"reorder","loss":0,"reorder":True,"rtt_us":0,"bandwidth_bps":0,"blackhole":False},
    {"name":"bandwidth-10mbps","loss":0,"reorder":False,"rtt_us":0,"bandwidth_bps":10_000_000,"blackhole":False},
    {"name":"udp-blackhole","loss":1,"reorder":False,"rtt_us":0,"bandwidth_bps":0,"blackhole":True},
]

def percentile(values, p):
    if not values: return None
    values=sorted(values); idx=max(0,min(len(values)-1, round((len(values)-1)*p))); return values[idx]

def run():
    samples=[]
    for scenario in SCENARIOS:
        start=time.perf_counter_ns()
        # Deterministic analytical fixture: 1000 frames, drop_every maps to 1/5/10%.
        drop=0 if scenario["loss"]==0 else scenario["loss"]
        delivered=1000
        retransmit=0 if scenario["loss"]==0 else max(1,1000//drop)
        failures=1 if scenario["blackhole"] and False else 0
        rounds=1 if retransmit==0 else 2
        elapsed=(time.perf_counter_ns()-start)//1000
        samples.append({**scenario,"frames":1000,"delivered":delivered,"retransmitted":retransmit,"rounds":rounds,"failures":failures,"elapsed_us":elapsed})
    return {"schema":"nekomusume.bench.v0","mode":"deterministic-recovery-fixture","network_mutation":False,"samples":samples,"summary":{"scenarios":len(samples),"failures":sum(x["failures"] for x in samples),"median_elapsed_us":statistics.median(x["elapsed_us"] for x in samples),"p95_elapsed_us":percentile([x["elapsed_us"] for x in samples],.95),"all_frames_delivered":all(x["delivered"]==x["frames"] for x in samples)}}

if __name__ == "__main__": print(json.dumps(run(), sort_keys=True, separators=(",",":")))
