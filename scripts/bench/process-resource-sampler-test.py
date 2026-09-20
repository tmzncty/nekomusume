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
        ready=sys.argv[2]
        release=sys.argv[3]
        files=[open('/dev/null','rb') for _ in range(5)]
        listener=socket.socket(); listener.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1)
        listener.bind(('127.0.0.1',port)); listener.listen(1)
        # Readiness marker after resources exist; the child then stays alive
        # until the sampler's observation-release marker appears — a real
        # happens-before handshake, not a fixed sleep. Bounded by the
        # sampler's own --max-seconds deadline.
        with open(ready,'w') as f: f.write('ready')
        deadline=time.monotonic()+3.0
        while not os.path.exists(release) and time.monotonic()<deadline:
            time.sleep(0.02)
        sys.exit(7)
    """))
    probe=socket.socket(); probe.bind(("127.0.0.1",0)); port=probe.getsockname()[1]; probe.close()
    sample=td/"sample.json"; ready=td/"child.ready"; release=td/"sampler.release"
    cp=run(sys.executable, str(SAMPLER), "--experiment-id", "fixture.known-fd", "--implementation", "fixture", "--role", "server", "--identity", "binary:fixture-v1", "--application-bytes", "1234", "--owned-port", str(port), "--interval-ms", "10", "--max-seconds", "2", "--release-on-observed", str(release), "--release-fd-min", "9", "--output", str(sample), "--", sys.executable, str(child), str(port), str(ready), str(release), check=False)
    assert cp.returncode == 7, (cp.returncode, cp.stderr)
    # Handshake: child signalled after resources existed AND the sampler's
    # observation-release marker appeared before the child exited.
    assert ready.exists(), "child readiness marker was written"
    assert release.exists(), "sampler observed >=9 fds + socket and released"
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

    # H-R9-082: a setsid-escaped descendant holding the owned listener must NOT
    # be certified as cleaned up merely because the original process group is
    # empty — owned_sockets_after_exit must be None and complete false.
    esc=td/"esc.py"; esc_ready=td/"esc.ready"; esc_json=td/"esc.json"; esc_pid=td/"esc.pid"
    esc.write_text(
        "import os,socket,subprocess,sys,time\n"
        "port=int(sys.argv[1]); ready=sys.argv[2]; pidf=sys.argv[3]\n"
        "code='import os,socket,sys,time; os.setsid(); s=socket.socket(); s.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1); s.bind((\"127.0.0.1\",int(sys.argv[1]))); s.listen(); open(sys.argv[2],\"w\").write(\"r\"); open(sys.argv[3],\"w\").write(str(os.getpid())); time.sleep(30)'\n"
        "subprocess.Popen([sys.executable,'-c',code,str(port),ready,pidf])\n"
        "# Happens-before: wait for the escaped descendant to bind+signal ready\n"
        "# before exiting, so the sampler cannot certify cleanup first.\n"
        "deadline=time.monotonic()+10\n"
        "while not os.path.exists(ready) and time.monotonic()<deadline:\n"
        "    time.sleep(0.02)\n"
        "sys.exit(0 if os.path.exists(ready) else 1)\n"
    )
    probe=socket.socket(); probe.bind(('127.0.0.1',0)); esc_port=probe.getsockname()[1]; probe.close()
    run(sys.executable,str(SAMPLER),'--experiment-id','fixture.escaped-descendant','--implementation','fixture','--role','server','--identity','binary:fixture-v1','--application-bytes','0','--owned-port',str(esc_port),'--interval-ms','10','--max-seconds','2','--output',str(esc_json),'--',sys.executable,str(esc),str(esc_port),str(esc_ready),str(esc_pid))
    # Wait for the escaped descendant to bind the owned listener before
    # asserting terminal cleanup — deterministic readiness handshake, not
    # scheduler luck.
    for _ in range(200):
        if esc_ready.exists():
            break
        time.sleep(0.01)
    assert esc_ready.exists(), "escaped descendant did not bind listener"
    try:
        # Readiness: the escaped owner demonstrably bound the listener before
        # the sampler finished — the marker was written by the escaped child.
        assert esc_ready.exists(), "escaped TCP listener signalled bound"
        e=json.loads(esc_json.read_text())
        assert e['cleanup']['process_group_empty'] is True, e['cleanup']
        assert e['cleanup']['owned_sockets_after_exit'] is None, e['cleanup']
        assert e['cleanup']['complete'] is False, e['cleanup']
    finally:
        # Bounded reaping even if an assertion fails — verify /proc/<pid> gone.
        if esc_pid.exists():
            try:
                pid = int(esc_pid.read_text().strip())
                os.kill(pid, signal.SIGKILL)
                for _ in range(100):
                    if not pathlib.Path(f"/proc/{pid}").exists():
                        break
                    time.sleep(0.01)
                assert not pathlib.Path(f"/proc/{pid}").exists(), f"escaped helper {pid} still alive"
            except (OSError, ValueError):
                pass
    # H-R9-083: a setsid-escaped descendant holding a bound UDP socket on the
    # owned port must also keep cleanup incomplete (UDP has no LISTEN state).
    escu=td/"escu.py"; escu_ready=td/"escu.ready"; escu_json=td/"escu.json"; escu_pid=td/"escu.pid"
    escu.write_text(
        "import os,socket,subprocess,sys,time\n"
        "port=int(sys.argv[1]); ready=sys.argv[2]; pidf=sys.argv[3]\n"
        "code='import os,socket,sys,time; os.setsid(); s=socket.socket(socket.AF_INET,socket.SOCK_DGRAM); s.bind((\"127.0.0.1\",int(sys.argv[1]))); open(sys.argv[2],\"w\").write(\"r\"); open(sys.argv[3],\"w\").write(str(os.getpid())); time.sleep(30)'\n"
        "subprocess.Popen([sys.executable,'-c',code,str(port),ready,pidf])\n"
        "deadline=time.monotonic()+10\n"
        "while not os.path.exists(ready) and time.monotonic()<deadline:\n"
        "    time.sleep(0.02)\n"
        "sys.exit(0 if os.path.exists(ready) else 1)\n"
    )
    probe=socket.socket(); probe.bind(('127.0.0.1',0)); escu_port=probe.getsockname()[1]; probe.close()
    run(sys.executable,str(SAMPLER),'--experiment-id','fixture.escaped-udp','--implementation','fixture','--role','server','--identity','binary:fixture-v1','--application-bytes','0','--owned-port',str(escu_port),'--interval-ms','10','--max-seconds','2','--output',str(escu_json),'--',sys.executable,str(escu),str(escu_port),str(escu_ready),str(escu_pid))
    for _ in range(200):
        if escu_ready.exists():
            break
        time.sleep(0.01)
    assert escu_ready.exists(), "escaped UDP descendant did not bind"
    try:
        assert escu_ready.exists(), "escaped UDP socket signalled bound"
        eu=json.loads(escu_json.read_text())
        assert eu['cleanup']['process_group_empty'] is True, eu['cleanup']
        assert eu['cleanup']['owned_sockets_after_exit'] is None, eu['cleanup']
        assert eu['cleanup']['complete'] is False, eu['cleanup']
    finally:
        if escu_pid.exists():
            try:
                pid = int(escu_pid.read_text().strip())
                os.kill(pid, signal.SIGKILL)
                for _ in range(100):
                    if not pathlib.Path(f"/proc/{pid}").exists():
                        break
                    time.sleep(0.01)
                assert not pathlib.Path(f"/proc/{pid}").exists(), f"escaped UDP helper {pid} still alive"
            except (OSError, ValueError):
                pass
    # H-R9-083: a partial terminal /proc observation (a required table
    # unreadable, or an unparseable row) must yield unknown — never absent.
    netdir=td/"procnet"; netdir.mkdir()
    hdr="  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode\n"
    row="   0: 0100007F:9C40 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 12345 1 0000000000000000 100 0 0 10 0\n"
    for n in ("tcp","tcp6","udp"):
        (netdir/n).write_text(hdr+row)
    # udp6 table missing entirely -> unknown (40001 not present in any table).
    assert sampler.owned_port_sockets_present({40001}, net_dir=str(netdir)) is None
    # A truncated TCP row lacking the state field on the owned port -> unknown.
    (netdir/"udp6").write_text(hdr)
    (netdir/"tcp").write_text(hdr+"   0: 0100007F:9C41\n")
    assert sampler.owned_port_sockets_present({40001}, net_dir=str(netdir)) is None
    # All four tables clean and no owned port -> absent.
    (netdir/"tcp").write_text(hdr+row)
    assert sampler.owned_port_sockets_present({40001}, net_dir=str(netdir)) is False
    # Result-construction oracle: a partial terminal observation (net_dir
    # missing a required table) must surface as owned_sockets_after_exit=None
    # and complete=false in the sampler's result JSON — never promoted.
    pd=td/"pdnet"; pd.mkdir()
    hdr="  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode\n"
    for n in ("tcp","tcp6","udp"):
        (pd/n).write_text(hdr)
    pdjson=td/"pd.json"
    run(sys.executable,str(SAMPLER),'--experiment-id','fixture.partial-obs','--implementation','fixture','--role','server','--identity','binary:fixture-v1','--application-bytes','0','--owned-port','40002','--interval-ms','10','--max-seconds','1','--net-dir',str(pd),'--output',str(pdjson),'--','/bin/true')
    pj=json.loads(pdjson.read_text())
    assert pj['cleanup']['owned_sockets_after_exit'] is None, pj['cleanup']
    assert pj['cleanup']['complete'] is False, pj['cleanup']
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
