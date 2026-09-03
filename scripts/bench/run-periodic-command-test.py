#!/usr/bin/env python3
from __future__ import annotations
import hashlib,importlib.util,json,pathlib,sys,tempfile
HERE=pathlib.Path(__file__).resolve().parent;S=HERE/'run-periodic-command.py';REMOTE=HERE/'remote-endpoint-exec.py'
sp=importlib.util.spec_from_file_location('periodic',S);m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
SCRIPT=r'''#!PYTHON
import sys,time
role=sys.argv[1];mode=sys.argv[sys.argv.index('--test-mode')+1]
if role=='periodic-server':
 if mode=='early':sys.exit(7)
 if mode=='timeout':time.sleep(2);sys.exit(0)
 if mode=='malformed':print('periodic_server_ready transport=tcp port=bad reconnect=unsupported',flush=True);sys.exit(0)
 if mode=='huge':print('x'*2000000,flush=True);sys.exit(0)
 if mode=='delayed':time.sleep(.12)
 print('periodic_server_ready transport=tcp port=40123 reconnect=unsupported',flush=True);time.sleep(.25)
 signal='true' if mode=='signal' else 'false';sdup=1 if mode=='serverdup' else 0
 print(f'periodic_server_summary authenticated=true received=2 confirmed=2 duplicates={sdup} elapsed_ms=250 cleanup=verified signal={signal}',flush=True)
else:
 print('periodic_client_authenticated session=7201 stream=1 reconnect=unsupported')
 print('periodic_interval seq=1 sent=true confirmed=true missing=false duplicate=false latency_ms=5')
 print('periodic_interval seq=2 sent=true confirmed=true missing=false duplicate=false latency_ms=9')
 if mode!='missing':
  attempted=1 if mode=='short' else 2;confirmed=1 if mode=='short' else 2;missing=1 if mode=='missing-record' else 0;reconnects=1 if mode=='reconnect' else 0;p50=9 if mode=='badp50' else 5;signal='true' if mode=='signal' else 'false'
  print(f'periodic_summary transport=tcp session=7201 stream=1 attempted={attempted} confirmed={confirmed} missing={missing} duplicates=0 p50_confirmation_latency_ms={p50} p95_confirmation_latency_ms=9 reconnects={reconnects} elapsed_ms=200 application_bytes=64 cleanup=verified signal={signal}')
'''
def make_binary(td):p=td/'neko';p.write_text(SCRIPT.replace('PYTHON',sys.executable));p.chmod(0o700);return str(p)
def binary(path):data=pathlib.Path(path).read_bytes();return {'path':path,'sha256':hashlib.sha256(data).hexdigest(),'bytes':len(data),'git_commit':'0'*40}
def cleanup(count=0):return [sys.executable,'-c',f'import json;print(json.dumps({{"listeners_remaining":{count},"processes_remaining":0}}))']
def args(exe,role,mode):return [exe,f'periodic-{role}','--transport','tcp','--port','40123']+(['--bind','0.0.0.0:40123'] if role=='server' else ['--addr','127.0.0.1:40123'])+['--bytes','32','--count','2','--duration','2','--interval-ms','100','--setup-timeout-ms','200','--ack-timeout-ms','100','--test-mode',mode]
def plan(td,remote=False,server_mode='good',client_mode='good'):
 exe=make_binary(td);server={'execution':'local','binary':binary(exe),'argv':args(exe,'server',server_mode)};clean={'local_argv':cleanup()}
 if remote:server.update(execution='ssh',ssh_executable=sys.executable,transport_argv=[sys.executable,str(REMOTE)]);clean['remote_transport_argv']=cleanup()
 return {'git_commit':'0'*40,'server':server,'client':{'execution':'local','binary':binary(exe),'argv':args(exe,'client',client_mode)},'cleanup':clean,'timeout_seconds':2}
def write(value,td):p=td/'plan.json';p.write_text(json.dumps(value));return str(p)
def execute(p,td):out=td/'out';rc=m.run(write(p,td),str(out),False,False);return rc,json.loads(out.read_text())
def test_pass_contract_and_duplicate_domains():
 with tempfile.TemporaryDirectory() as raw:
  td=pathlib.Path(raw);rc,r=execute(plan(td,server_mode='serverdup'),td);x=r['result'];assert rc==0 and x['status']=='passed';assert x['attempted']==x['confirmed']==x['declared']['count']==2 and x['missing']==x['reconnects']==0;assert x['client_duplicates']==0 and x['server_duplicates']==1 and 'conflicts' not in x;assert x['confirmation_latencies_ms']==[5,9] and (x['median_confirmation_latency_ms'],x['p95_confirmation_latency_ms'])==(5,9);assert x['application_bytes']==64

def test_normalized_contract_mismatch_and_duplicates_rejected_before_spawn():
 for role,key,value in [('client','--count','3'),('server','--bytes','33'),('client','--duration','3'),('server','--interval-ms','101'),('client','--setup-timeout-ms','201'),('server','--ack-timeout-ms','101'),('client','--transport','udp'),('client','--port','40124'),('client','--addr','127.0.0.1:40124'),('server','--bind','0.0.0.0:40124')]:
  with tempfile.TemporaryDirectory() as raw:
   td=pathlib.Path(raw);p=plan(td);a=p[role]['argv'];a[a.index(key)+1]=value
   try:m.run(write(p,td),str(td/'out'),False,False);assert False
   except m.PlanError:pass
 with tempfile.TemporaryDirectory() as raw:
  td=pathlib.Path(raw);p=plan(td);p['client']['argv']+=['--count','2']
  try:m.run(write(p,td),str(td/'out'),False,False);assert False
  except m.PlanError as e:assert 'duplicate' in str(e)

def test_incomplete_reconnect_signal_and_percentile_fail_closed():
 for cm,sm,reason in [('short','good','contradiction'),('missing-record','good','interval accounting'),('reconnect','good','declared completion'),('signal','good','signal interruption'),('good','signal','signal interruption'),('badp50','good','percentile')]:
  with tempfile.TemporaryDirectory() as raw:
   td=pathlib.Path(raw);rc,r=execute(plan(td,client_mode=cm,server_mode=sm),td);assert rc==1 and r['result']['status']=='malformed_result' and reason in r['result']['failure_reason']
def test_delayed_remote_readiness_private_and_cleanup():
 with tempfile.TemporaryDirectory() as raw:
  td=pathlib.Path(raw);rc,r=execute(plan(td,True,'delayed'),td);raw_result=json.dumps(r);assert rc==0 and r['readiness']['status']=='ready';assert r['resources']['server']=={'status':'not_collected_remote'};assert r['private_logs']['tracked'] is False and not r['private_logs']['truncated'];assert 'argv' not in raw_result and 'transport_argv' not in raw_result and all(x['status']=='verified' for x in r['cleanup']['postchecks'])
def test_readiness_failures_never_launch_client():
 old=m.STARTUP_TIMEOUT_SECONDS;m.STARTUP_TIMEOUT_SECONDS=.2
 try:
  for mode,status in [('early','remote_early_exit'),('timeout','start_timeout'),('malformed','malformed_readiness')]:
   with tempfile.TemporaryDirectory() as raw:
    td=pathlib.Path(raw);rc,r=execute(plan(td,server_mode=mode),td);assert rc==1 and r['result']['status']==status and not r['dispatch']['periodic_client_entered']
 finally:m.STARTUP_TIMEOUT_SECONDS=old
def test_missing_summary_bounded_logs_dry_validate_cleanup():
 with tempfile.TemporaryDirectory() as raw:
  td=pathlib.Path(raw);rc,r=execute(plan(td,client_mode='missing'),td);assert rc==1 and r['result']['status']=='malformed_result'
 old=m.MAX_LOG_BYTES;m.MAX_LOG_BYTES=1024
 try:
  with tempfile.TemporaryDirectory() as raw:
   td=pathlib.Path(raw);rc,r=execute(plan(td,server_mode='huge'),td);assert rc==1 and r['private_logs']['truncated']
 finally:m.MAX_LOG_BYTES=old
 with tempfile.TemporaryDirectory() as raw:
  td=pathlib.Path(raw);p=plan(td,True);out=td/'out';assert m.run(write(p,td),str(out),False,True)==0 and not json.loads(out.read_text())['live_evidence'];assert m.run(write(p,td),str(td/'never'),True,False)==0 and not (td/'never').exists()
 with tempfile.TemporaryDirectory() as raw:
  td=pathlib.Path(raw);p=plan(td);p['cleanup']['local_argv']=cleanup(1);rc,r=execute(p,td);assert rc==1 and r['result']['status']=='cleanup_failed'
def main():
 for name,value in sorted(globals().items()):
  if name.startswith('test_'):value()
 print('periodic command tests: ok')
if __name__=='__main__':main()
