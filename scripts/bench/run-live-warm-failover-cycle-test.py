#!/usr/bin/env python3
from __future__ import annotations
import json, os, pathlib, subprocess, sys, tempfile

HERE = pathlib.Path(__file__).resolve().parent
spec_path = HERE / "run-repeated-warm-failover.py"
import importlib.util
spec = importlib.util.spec_from_file_location("repeated_warm", spec_path); assert spec and spec.loader
runner = importlib.util.module_from_spec(spec); spec.loader.exec_module(runner)
ADAPTER = HERE / "run-live-warm-failover-cycle.py"
RUNNER = HERE / "run-repeated-warm-failover.py"
FAKE = r'''#!/usr/bin/env python3
import json, os, sys
mode=sys.argv[1]; scenario=os.environ.get("SCENARIO","success")
def j(event,seq=0,**kw): print(json.dumps(dict(event=event,seq=seq,**kw)),flush=True)
if mode == "failover-server":
 j("start",count=3,record_payload_bytes=16,max_seconds=15); print("carrier_event name=udp_negotiated session=7001 generation=0 version=0"); print("carrier_event name=udp_authenticated session=7001 generation=0");
 if scenario != "missing_resume": print("carrier_event name=tcp_negotiated session=7001 generation=1 version=0\ncarrier_event name=tcp_authenticated session=7001 generation=1\ncarrier_event name=tcp_resume_validated session=7001 generation=1")
 if scenario != "missing_readiness":
  [j("tcp_readiness_response",n,admitted=True) for n in (1,2,3)]
 if scenario not in ("timeout","missing_resume","missing_readiness","missing_timing","missing_accounting"):
  j("tcp_delivery_ack_sent",1); j("tcp_delivery_ack_sent",2); print("carrier_event name=tcp_resumed session=7001 generation=1"); j("summary",3,records=3,application_bytes_total=48)
 sys.exit(7 if scenario=="server_mismatch" else 0)
if mode == "failover-client":
 j("start",count=3,record_payload_bytes=16,max_seconds=12); print("carrier_event name=udp_authenticated session=7001 generation=0"); j("udp_delivery_ack_validated",1); j("udp_uncertain_range_sent",2,len=16)
 if scenario != "missing_readiness":
  [j("tcp_warm_readiness",n,warm=n==3) for n in (1,2,3)]; print("carrier_event name=tcp_warm session=7001 generation=1 readiness=3 application_data=0")
 if scenario not in ("timeout","missing_resume","missing_readiness","missing_accounting"):
  j("tcp_delivery_ack_validated",1); j("tcp_delivery_ack_validated",2)
 if scenario not in ("timeout","missing_timing"):
  j("failover_timing",3,failure_decided_at_us=100,first_resumed_data_accepted_us=130,first_resumed_ack_at_us=140,recovery_latency_us=30)
 if scenario not in ("timeout","missing_resume","missing_readiness","missing_timing","missing_accounting"):
  j("failover_accounting",3,udp_confirmed_records=1,udp_confirmed_bytes=16,uncertain_records=2,uncertain_bytes=32,replayed_records=2,replayed_bytes=32,confirmed_records=3,confirmed_bytes=48,duplicate_records=0,duplicate_bytes=0,lost_records=0,lost_bytes=0,conflicting_records=0,conflicting_bytes=0); print("carrier_event name=ordered_records_complete session=7001 count=3 bytes=16"); j("summary",3,records=3,application_bytes_total=48)
 sys.exit(124 if scenario=="timeout" else 0)
if mode == "cleanup":
 print(json.dumps({"listeners_remaining":1 if scenario=="cleanup" else 0})); sys.exit(1 if scenario=="cleanup" else 0)
'''

def invoke(scenario="success", marker=""):

 with tempfile.TemporaryDirectory() as td:
  fake=pathlib.Path(td)/"fake.py"; fake.write_text(FAKE); fake.chmod(0o700)
  env=os.environ.copy(); env.update(SCENARIO=scenario,NEKO_FAILOVER_CYCLE_INDEX="1",NEKO_FAILOVER_GIT_COMMIT="a"*40,NEKO_FAILOVER_BINARY=sys.executable,NEKO_FAILOVER_UDP_PORT="40081",NEKO_FAILOVER_TCP_PORT="40080",NEKO_FAILOVER_SERVER_STARTUP_SECONDS="0.01")
  base=[sys.executable,str(fake)]
  suffix=[marker] if marker else []
  env["NEKO_FAILOVER_SERVER_COMMAND_JSON"]=json.dumps(base+["failover-server","--diagnostic-id","fixture-server","--cease-udp-replies-after","1"]+suffix)
  env["NEKO_FAILOVER_CLIENT_COMMAND_JSON"]=json.dumps(base+["failover-client","--diagnostic-id","fixture-client","--automatic-health-failover"]+suffix)
  env["NEKO_FAILOVER_CLEANUP_COMMAND_JSON"]=json.dumps(base+["cleanup"])
  p=subprocess.run([sys.executable,str(ADAPTER)],env=env,text=True,capture_output=True,timeout=30)
  return p, json.loads(p.stdout) if p.stdout.strip() else None

p,row=invoke(); runner.validate_cycle(row, 1); assert p.returncode==0 and row["result"]["status"]=="passed"; assert row["semantic"]["readiness_proofs"]==3; assert row["accounting"]["uncertain_records"]==2
p,row=invoke("timeout"); runner.validate_cycle(row, 1); assert p.returncode==0 and row["result"]=={"status":"failed","client_exit_code":124,"server_exit_code":0,"failure_stage":"client","failure_reason":"client_timeout"}; assert row["cleanup"]["status"]=="verified"
for scenario in ("server_mismatch","missing_resume","missing_readiness","missing_timing","missing_accounting","cleanup"):
 p,row=invoke(scenario); assert p.returncode==0 and row["result"]["status"]=="failed", scenario
 if scenario=="cleanup": assert row["cleanup"]["status"]=="failed"
 p,row=invoke("success", "super-secret-key")
assert p.returncode==0 and "super-secret-key" not in p.stdout and "super-secret-key" not in json.dumps(row)
print("live warm failover adapter tests: ok")
