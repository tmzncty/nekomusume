#!/usr/bin/env python3
"""Deterministic harmless process/FD/listener and validator regressions."""
import importlib.util, json, os, pathlib, signal, socket, subprocess, sys, tempfile, textwrap, time
ROOT=pathlib.Path(__file__).resolve().parents[2]
SAMPLER=ROOT/"scripts/bench/process-resource-sampler.py"
VALIDATOR=ROOT/"scripts/bench/validate-process-resource.py"

def run(*args, check=True): return subprocess.run(args, text=True, capture_output=True, check=check)

# Guaranteed-missing PID: every /proc read must fail without changing the
# caller-visible return shape or inventing zero-valued metrics.
spec=importlib.util.spec_from_file_location("process_resource_sampler", SAMPLER)
sampler=importlib.util.module_from_spec(spec); spec.loader.exec_module(sampler)
missing_pid=max(4_000_000, os.getpid()+1_000_000)
assert not pathlib.Path(f"/proc/{missing_pid}").exists()
cpu,rss,fds,sockets,sources=sampler.read_proc(missing_pid,{65535})
assert cpu == (None,None) and rss is None and fds is None and sockets is None
assert sources == {"cpu":None,"rss":None,"fd":None,"socket":None}

with tempfile.TemporaryDirectory() as td_raw:
    td=pathlib.Path(td_raw)
    child=td/"known_child.py"
    child.write_text(textwrap.dedent("""\
        import os, socket, sys, time
        port=int(sys.argv[1])
        files=[open('/dev/null','rb') for _ in range(5)]
        listener=socket.socket(); listener.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1)
        listener.bind(('127.0.0.1',port)); listener.listen(1)
        time.sleep(.25)
        sys.exit(7)
    """))
    probe=socket.socket(); probe.bind(("127.0.0.1",0)); port=probe.getsockname()[1]; probe.close()
    sample=td/"sample.json"
    cp=run(sys.executable, str(SAMPLER), "--experiment-id", "fixture.known-fd", "--implementation", "fixture", "--role", "server", "--identity", "binary:fixture-v1", "--application-bytes", "1234", "--owned-port", str(port), "--interval-ms", "10", "--max-seconds", "2", "--output", str(sample), "--", sys.executable, str(child), str(port), check=False)
    assert cp.returncode == 7, (cp.returncode, cp.stderr)
    d=json.loads(sample.read_text())
    assert d["exit"] == {"code": 7, "signal": None, "timed_out": False}
    assert d["fd"]["peak_count"] >= 9, d["fd"]
    assert d["owned_experimental_sockets"]["peak_count"] == 1, d["owned_experimental_sockets"]
    assert d["application_bytes"] == 1234 and d["cleanup"]["complete"] is True
    run(sys.executable, str(VALIDATOR), str(sample))

    # Normal direct-child exit must terminate a same-group listener grandchild.
    grand=td/"grand.py"; pidfile=td/"grand.pid"; normal=td/"normal.json"
    grand.write_text("import os,socket,subprocess,sys\ns=socket.socket(); s.bind(('127.0.0.1',int(sys.argv[1]))); s.listen()\np=subprocess.Popen([sys.executable,'-c','import time; time.sleep(60)'])\nopen(sys.argv[2],'w').write(str(p.pid))\n")
    probe=socket.socket(); probe.bind(('127.0.0.1',0)); grand_port=probe.getsockname()[1]; probe.close()
    run(sys.executable,str(SAMPLER),'--experiment-id','fixture.normal-group','--implementation','fixture','--role','server','--identity','binary:fixture-v1','--application-bytes','0','--owned-port',str(grand_port),'--interval-ms','10','--max-seconds','2','--output',str(normal),'--',sys.executable,str(grand),str(grand_port),str(pidfile))
    n=json.loads(normal.read_text()); assert n['cleanup']['process_group_empty'] is True and n['cleanup']['owned_sockets_after_exit']==0
    gpid=int(pidfile.read_text()); assert not pathlib.Path(f'/proc/{gpid}').exists()
    with socket.socket() as check:
        assert check.connect_ex(('127.0.0.1',grand_port)) != 0

    # Exit-race: a child that is gone before the first sleep still yields wait4
    # CPU/RSS and a truthful null sampled-FD metric rather than fake zero.
    short=td/"short.json"
    run(sys.executable, str(SAMPLER), "--experiment-id", "fixture.exit-race", "--implementation", "fixture", "--role", "client", "--identity", "git:0123456789abcdef", "--application-bytes", "0", "--interval-ms", "50", "--max-seconds", "1", "--output", str(short), "--", "/bin/true")
    run(sys.executable, str(VALIDATOR), str(short))
    s=json.loads(short.read_text()); assert s["exit"]["code"] == 0 and s["cpu"]["source"] == "wait4 rusage"

    # Timeout terminates the child process group, including a harmless grandchild.
    timeout_child=td/"timeout_child.py"
    timeout_child.write_text("import subprocess,sys,time\np=subprocess.Popen([sys.executable,\"-c\",\"import time; time.sleep(10)\"])\nprint(p.pid, flush=True)\ntime.sleep(10)\n")
    timed=td/"timed.json"
    cp=run(sys.executable, str(SAMPLER), "--experiment-id", "fixture.timeout-group", "--implementation", "fixture", "--role", "server", "--identity", "binary:fixture-v1", "--application-bytes", "0", "--interval-ms", "10", "--max-seconds", ".1", "--output", str(timed), "--", sys.executable, str(timeout_child), check=False)
    assert cp.returncode == 143
    t=json.loads(timed.read_text()); assert t["exit"]["timed_out"] is True and t["exit"]["signal"] == 15
    run(sys.executable, str(VALIDATOR), str(timed))

    # Fail closed on malformed metadata, unknown fields, fake unavailable zero,
    # contradictory exit state, and oversized bounds.
    mutations=[]
    for mutate in (
        lambda x: x.update(extra="no"),
        lambda x: x.pop("identity"),
        lambda x: x.update(experiment_id="../bad"),
        lambda x: x["rss"].update(max_kib=0, source=None),
        lambda x: x["exit"].update(code=0, signal=9),
        lambda x: x["sampling"].update(max_seconds=601),
    ):
        x=json.loads(sample.read_text()); mutate(x); mutations.append(x)
    for i,x in enumerate(mutations):
        bad=td/f"bad-{i}.json"; bad.write_text(json.dumps(x))
        assert run(sys.executable, str(VALIDATOR), str(bad), check=False).returncode != 0

    # Collector metadata itself fails closed before spawning.
    badout=td/"bad-output.json"
    cp=run(sys.executable, str(SAMPLER), "--experiment-id", "../bad", "--implementation", "fixture", "--role", "client", "--identity", "git:x", "--application-bytes", "0", "--output", str(badout), "--", "/bin/true", check=False)
    assert cp.returncode == 1 and not badout.exists()

# TERM and INT must stop and reap the sampler-owned child before executable-path deletion.
for stop_signal in (signal.SIGTERM, signal.SIGINT):
    with tempfile.TemporaryDirectory() as race_raw:
        race = pathlib.Path(race_raw)
        child = race / "child.py"
        child.write_text("import socket,time,sys\ns=socket.socket(); s.bind(('127.0.0.1',int(sys.argv[1]))); s.listen(); time.sleep(60)\n")
        probe = socket.socket(); probe.bind(("127.0.0.1", 0)); race_port = probe.getsockname()[1]; probe.close()
        race_out = race / "race.json"
        proc = subprocess.Popen([sys.executable, str(SAMPLER), "--experiment-id", "signal-race", "--implementation", "fake", "--role", "client", "--identity", "sha256:00", "--application-bytes", "0", "--owned-port", str(race_port), "--interval-ms", "10", "--max-seconds", "60", "--output", str(race_out), "--", sys.executable, str(child), str(race_port)])
        for _ in range(100):
            try:
                with socket.create_connection(("127.0.0.1", race_port), timeout=.02): break
            except OSError: time.sleep(.01)
        else: raise AssertionError("race listener did not start")
        proc.send_signal(stop_signal); assert proc.wait(timeout=3) == 128 + stop_signal
        with socket.socket() as check:
            try: check.connect(("127.0.0.1", race_port))
            except OSError: pass
            else: raise AssertionError("listener remained after sampler signal")
        child.unlink(); assert not child.exists()
print("process resource sampler tests: PASS")
