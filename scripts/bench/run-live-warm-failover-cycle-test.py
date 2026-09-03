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
role="server" if mode == "failover-server" else "client"
experiment_id="warm-cycle-1-"+role
def j(event,seq=0,**kw): print(json.dumps(dict(experiment_id=experiment_id,role=role,event=event,seq=seq,**kw)),flush=True)
if mode == "failover-server":
 j("start",count=4 if scenario in ("start_mismatch","adversarial") else 3,record_payload_bytes=16,application_bytes_total=48,udp_port=40081,tcp_port=40080,max_seconds=15);
 if scenario in ("duplicate_start","adversarial"): j("start",count=3,record_payload_bytes=16,application_bytes_total=48,udp_port=40081,tcp_port=40080,max_seconds=15)
 if scenario in ("malformed_json","adversarial"): print("  {not-json")
 if scenario == "invalid_event_object": print("  {}")
 if scenario == "json_list": print("  []")
 if scenario == "json_scalar": print("  42")
 if scenario == "malformed_list": print("  [not-json")
 print("carrier_event name=udp_negotiated session=7001 generation=0 version=0"); print("carrier_event name=udp_authenticated session=7001 generation=0");
 if scenario != "missing_resume": print("carrier_event name=tcp_negotiated session=7001 generation=1 version=0\ncarrier_event name=tcp_authenticated session=7001 generation=1\ncarrier_event name=tcp_resume_validated session=7001 generation=1")
 if scenario != "missing_readiness":
  [j("tcp_readiness_response",n,admitted=True) for n in (1,2,3)]
 if scenario not in ("timeout","missing_resume","missing_readiness","missing_timing","missing_accounting"):
  j("tcp_delivery_ack_sent",1); j("tcp_delivery_ack_sent",2); print("carrier_event name=tcp_resumed session=7001 generation=1"); j("summary",3,records=3,application_bytes_total=48)
 sys.exit(7 if scenario=="server_mismatch" else 0)
if mode == "failover-client":
 j("start",count=3,record_payload_bytes=16,application_bytes_total=48,udp_port=40081,tcp_port=40080,max_seconds=12);
 print("carrier_event name=udp_authenticated session=7001 generation=0"); j("udp_delivery_ack_validated",1); j("udp_uncertain_range_sent",2,len=16)
 if scenario != "missing_readiness":
  [j("tcp_warm_readiness",n,warm=n==3) for n in (1,2,3)]; print("carrier_event name=tcp_warm session=7001 generation=1 readiness=3 application_data=0")
 if scenario not in ("timeout","missing_resume","missing_readiness","missing_accounting"):
  j("tcp_delivery_ack_validated",1); j("tcp_delivery_ack_validated",2)
 if scenario not in ("timeout","missing_timing"):
  timing=dict(failure_decided_at_us=100,first_resumed_data_accepted_us=130,first_resumed_ack_at_us=140,recovery_latency_us=30)
  if scenario == "negative_timing": timing["failure_decided_at_us"]=-1
  if scenario == "reversed_timing": timing["first_resumed_data_accepted_us"]=150
  if scenario == "inconsistent_timing": timing["recovery_latency_us"]=29
  if scenario == "boolean_timing": timing["failure_decided_at_us"]=True
  if scenario == "boundary_timing": timing=dict(failure_decided_at_us=100,first_resumed_data_accepted_us=100,first_resumed_ack_at_us=100,recovery_latency_us=0)
  j("failover_timing",3,**timing)
  if scenario in ("duplicate_timing","adversarial"): j("failover_timing",3,failure_decided_at_us=100,first_resumed_data_accepted_us=130,first_resumed_ack_at_us=140,recovery_latency_us=30)
 if scenario not in ("timeout","missing_resume","missing_readiness","missing_timing","missing_accounting"):
  accounting=dict(udp_confirmed_records=1,udp_confirmed_bytes=16,uncertain_records=2,uncertain_bytes=32,replayed_records=2,replayed_bytes=32,confirmed_records=3,confirmed_bytes=48,duplicate_records=0,duplicate_bytes=0,lost_records=0,lost_bytes=0,conflicting_records=0,conflicting_bytes=0)
  if scenario == "contradictory_accounting": accounting["udp_confirmed_bytes"]=15
  if scenario == "wrong_uncertain": accounting["uncertain_records"]=1; accounting["uncertain_bytes"]=16
  if scenario == "wrong_replayed": accounting["replayed_records"]=1; accounting["replayed_bytes"]=16
  if scenario == "negative_accounting": accounting["lost_records"]=-1
  if scenario == "boolean_accounting": accounting["confirmed_records"]=True
  j("failover_accounting",3,**accounting)
  if scenario in ("duplicate_accounting","adversarial"): j("failover_accounting",3,udp_confirmed_records=1,udp_confirmed_bytes=16,uncertain_records=2,uncertain_bytes=32,replayed_records=2,replayed_bytes=32,confirmed_records=3,confirmed_bytes=48,duplicate_records=0,duplicate_bytes=0,lost_records=0,lost_bytes=0,conflicting_records=0,conflicting_bytes=0)
  print("carrier_event name=ordered_records_complete session=7001 count=3 bytes=16"); j("summary",3,records=3,application_bytes_total=48)
  if scenario == "duplicate_summary": j("summary",3,records=3,application_bytes_total=48)
 sys.exit(124 if scenario=="timeout" else 0)
if mode == "cleanup":
 value={"listeners_remaining":1 if scenario=="cleanup" else 0}
 if scenario != "missing_remote_process_postcheck": value["processes_remaining"]=0
 print(json.dumps(value)); sys.exit(1 if scenario=="cleanup" else 0)
'''

HEAD = subprocess.run(
 ["git", "-C", str(HERE), "rev-parse", "HEAD"],
 check=True, capture_output=True, text=True,
).stdout.strip()

def invoke(scenario="success", marker="", server_executable=None, client_executable=None, binary=None, commit=None, endpoints=None):

 with tempfile.TemporaryDirectory() as td:
  fake=pathlib.Path(td)/"fake.py"; fake.write_text(FAKE); fake.chmod(0o700)
  env=os.environ.copy(); env.update(SCENARIO=scenario,NEKO_FAILOVER_CYCLE_INDEX="1",NEKO_FAILOVER_GIT_COMMIT=commit or HEAD,NEKO_FAILOVER_BINARY=binary or sys.executable,NEKO_FAILOVER_UDP_PORT="40081",NEKO_FAILOVER_TCP_PORT="40080",NEKO_FAILOVER_SERVER_STARTUP_SECONDS="0.01")
  suffix=[marker] if marker else []
  env["NEKO_FAILOVER_SERVER_COMMAND_JSON"]=json.dumps([server_executable or sys.executable,str(fake),"failover-server","--diagnostic","--experiment-id","warm-cycle-1-server","--cease-udp-replies-after","1"]+suffix)
  env["NEKO_FAILOVER_CLIENT_COMMAND_JSON"]=json.dumps([client_executable or sys.executable,str(fake),"failover-client","--diagnostic","--experiment-id","warm-cycle-1-client","--automatic-health-failover"]+suffix)
  env["NEKO_FAILOVER_CLEANUP_COMMAND_JSON"]=json.dumps([sys.executable,str(fake),"cleanup"])
  if endpoints is not None: env["NEKO_FAILOVER_ENDPOINTS_JSON"]=json.dumps(endpoints(fake))
  p=subprocess.run([sys.executable,str(ADAPTER)],env=env,text=True,capture_output=True,timeout=30)
  return p, json.loads(p.stdout) if p.stdout.strip() else None

p,row=invoke(); runner.validate_cycle(row, 1); assert p.returncode==0 and row["result"]["status"]=="passed"; assert row["semantic"]["readiness_proofs"]==3; assert row["accounting"]["uncertain_records"]==2
with tempfile.TemporaryDirectory() as td:
 symlink=pathlib.Path(td)/"python-symlink"
 try:
  symlink.symlink_to(sys.executable)
 except (NotImplementedError, OSError):
  pass
 else:
  p,row=invoke(server_executable=str(symlink),client_executable=str(symlink)); runner.validate_cycle(row, 1); assert p.returncode==0 and row["result"]["status"]=="passed"
for role in ("server", "client"):
 with tempfile.TemporaryDirectory() as td:
  decoy=pathlib.Path(td)/"decoy"; decoy.write_bytes(pathlib.Path(sys.executable).read_bytes()); decoy.chmod(0o700)
  kwargs={f"{role}_executable":str(decoy)}
  p,row=invoke(**kwargs); assert p.returncode==2 and p.stdout == "" and row is None, (role,p.returncode,p.stdout,p.stderr)
p,row=invoke(commit="b"*40); assert p.returncode==2 and p.stdout == "" and row is None, (p.returncode,p.stdout,p.stderr)
p,row=invoke("timeout"); runner.validate_cycle(row, 1); assert p.returncode==0 and row["result"]=={"status":"failed","client_exit_code":124,"server_exit_code":0,"failure_stage":"client","failure_reason":"client_timeout"}; assert row["cleanup"]["status"]=="verified"
for scenario in ("server_mismatch","missing_resume","missing_readiness","missing_timing","missing_accounting","cleanup"):
 p,row=invoke(scenario); assert p.returncode==0 and row["result"]["status"]=="failed", scenario
 if scenario=="cleanup": assert row["cleanup"]["status"]=="failed"
for scenario in ("malformed_json", "invalid_event_object", "json_list", "json_scalar", "malformed_list", "duplicate_start", "duplicate_timing", "duplicate_accounting", "duplicate_summary", "start_mismatch", "adversarial"):
 p,row=invoke(scenario); assert p.returncode==2 and p.stdout == "" and row is None, (scenario,p.returncode,p.stdout,p.stderr)
print("ADVERSARIAL_REJECTED")
for scenario in ("negative_timing", "reversed_timing", "inconsistent_timing", "boolean_timing", "contradictory_accounting", "wrong_uncertain", "wrong_replayed", "negative_accounting", "boolean_accounting"):
 p,row=invoke(scenario); assert p.returncode==2 and p.stdout == "" and row is None, (scenario,p.returncode,p.stdout,p.stderr)
p,row=invoke("boundary_timing"); runner.validate_cycle(row, 1); assert p.returncode==0 and row["result"]["status"]=="passed" and row["timing"]["recovery_latency_us"]==0
p,row=invoke("success", "super-secret-key")
assert p.returncode==0 and "super-secret-key" not in p.stdout and "super-secret-key" not in json.dumps(row)

# A no-network transport may exercise the generic request shape, but cannot claim verified SSH.
def fake_cross_host(fake):
 binary=pathlib.Path(sys.executable); data=binary.read_bytes()
 identity={"path":str(binary),"sha256":__import__("hashlib").sha256(data).hexdigest(),"bytes":len(data),"git_commit":HEAD}
 helper=str(HERE/"remote-endpoint-exec.py")
 def endpoint(role, execution):
  argv=[str(binary),str(fake),"failover-"+role,"--diagnostic","--experiment-id","warm-cycle-1-"+role]
  argv += ["--cease-udp-replies-after","1"] if role=="server" else ["--automatic-health-failover"]
  value={"role":role,"execution":execution,"binary":identity,"argv":argv}
  if execution=="ssh": value["transport_argv"]=[sys.executable,helper]; value["ssh_executable"]="/definitely/not/the/python/executable"
  return value
 return [endpoint("server","ssh"),endpoint("client","local")]
p,row=invoke(endpoints=fake_cross_host)
assert p.returncode==2 and p.stdout=="" and row is None
# A same-file declared transport is accepted as the mechanically verified class.
def verified_cross_host(fake):
 values=fake_cross_host(fake); values[0]["ssh_executable"]=sys.executable; return values
p,row=invoke(endpoints=verified_cross_host); runner.validate_cycle(row,1)
assert p.returncode==0 and row["result"]["status"]=="passed"
assert [(e["role"],e["execution"]) for e in row["endpoint_provenance"]]==[("server","ssh"),("client","local")]
assert row["resources"]["server"] == {"status":"not_collected_remote"} and row["resources"]["client"] is not None
p,row=invoke(scenario="missing_remote_process_postcheck", endpoints=verified_cross_host)
assert p.returncode==2 and p.stdout=="" and row is None
def confused(fake):
 values=verified_cross_host(fake); values[0]["argv"][0]=sys.executable+"-wrapper"; return values
p,row=invoke(endpoints=confused); assert p.returncode==2 and p.stdout=="" and row is None
print("live warm failover adapter tests: ok")
