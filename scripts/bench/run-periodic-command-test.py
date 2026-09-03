#!/usr/bin/env python3
from __future__ import annotations
import hashlib,importlib.util,json,pathlib,shutil,sys,tempfile
HERE=pathlib.Path(__file__).resolve().parent;S=HERE/'run-periodic-command.py';REMOTE=HERE/'remote-endpoint-exec.py'
sp=importlib.util.spec_from_file_location('periodic',S);m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
SCRIPT=r'''#!PYTHON
import sys,time
role=sys.argv[1]; mode=sys.argv[sys.argv.index('--test-mode')+1]
if role=='periodic-server':
 if mode=='early': sys.exit(7)
 if mode=='timeout': time.sleep(2);sys.exit(0)
 if mode=='malformed': print('periodic_server_ready transport=tcp port=bad reconnect=unsupported',flush=True);sys.exit(0)
 if mode=='huge': print('x'*2000000,flush=True);sys.exit(0)
 if mode=='delayed': time.sleep(.12)
 print('periodic_server_ready transport=tcp port=40123 reconnect=unsupported',flush=True)
 time.sleep(.25)
 print('periodic_server_summary authenticated=true received=2 confirmed=1 duplicates=0 elapsed_ms=250 cleanup=verified signal=false',flush=True)
else:
 print('periodic_client_authenticated session=7201 stream=1 reconnect=unsupported')
 print('periodic_interval seq=1 sent=true confirmed=true missing=false duplicate=false latency_ms=5')
 print('periodic_interval seq=2 sent=true confirmed=false missing=true duplicate=false latency_ms=null')
 if mode!='missing': print('periodic_summary transport=tcp session=7201 stream=1 attempted=2 confirmed=1 missing=1 duplicates=0 p50_confirmation_latency_ms=5 p95_confirmation_latency_ms=5 reconnects=0 elapsed_ms=200 application_bytes=64 cleanup=verified signal=false')
'''
def make_binary(td):
 p=td/'neko';p.write_text(SCRIPT.replace('PYTHON',sys.executable));p.chmod(0o700);return str(p)
def binary(path,sha=None):
 data=pathlib.Path(path).read_bytes();return {'path':path,'sha256':sha or hashlib.sha256(data).hexdigest(),'bytes':len(data),'git_commit':'0'*40}
def cleanup(marker=None,count=0):
 prefix=f'import pathlib;pathlib.Path({str(marker)!r}).write_text("called");' if marker else ''
 return [sys.executable,'-c',prefix+f'import json;print(json.dumps({{"listeners_remaining":{count},"processes_remaining":0}}))']
def plan(td,remote=False,server_mode='good',client_mode='good'):
 exe=make_binary(td); server={'execution':'local','binary':binary(exe),'argv':[exe,'periodic-server','--transport','tcp','--port','40123','--test-mode',server_mode]}
 clean={'local_argv':cleanup()}
 if remote:
  server.update(execution='ssh',ssh_executable=sys.executable,transport_argv=[sys.executable,str(REMOTE)]);clean['remote_transport_argv']=cleanup()
 return {'git_commit':'0'*40,'server':server,'client':{'execution':'local','binary':binary(exe),'argv':[exe,'periodic-client','--bytes','32','--test-mode',client_mode]},'cleanup':clean,'timeout_seconds':2}
def write(value,td):p=td/'plan.json';p.write_text(json.dumps(value));return str(p)
def execute(p,td):out=td/'out';rc=m.run(write(p,td),str(out),False,False);return rc,json.loads(out.read_text())
def test_delayed_readiness_exact_accounting_and_private_logs():
 with tempfile.TemporaryDirectory() as raw:
  td=pathlib.Path(raw);rc,r=execute(plan(td,server_mode='delayed'),td);assert rc==0 and r['result']['status']=='passed';assert r['readiness']['status']=='ready' and r['dispatch']['periodic_client_entered'];assert {k:r['result'][k] for k in ('attempted','confirmed','missing','duplicates','conflicts','application_bytes')}=={'attempted':2,'confirmed':1,'missing':1,'duplicates':0,'conflicts':0,'application_bytes':64};assert r['result']['confirmation_latencies_ms']==[5,None] and r['result']['median_confirmation_latency_ms']==5;assert r['private_logs']['tracked'] is False and not r['private_logs']['truncated'];assert r['resources']['client']['status']=='collected_local_direct_child';assert 'argv' not in json.dumps(r['endpoints']) and '/tmp/' not in json.dumps(r['endpoints']) and '40123' in json.dumps(r) and all(x['status']=='verified' for x in r['cleanup']['postchecks'])
def test_remote_shaped_readiness_and_resources():
 with tempfile.TemporaryDirectory() as raw:
  td=pathlib.Path(raw);rc,r=execute(plan(td,True,'delayed'),td);assert rc==0;assert r['resources']['server']=={'status':'not_collected_remote'} and len(r['cleanup']['postchecks'])==2;assert 'transport_argv' not in json.dumps(r)
def test_early_exit_timeout_malformed_and_no_client():
 old=m.STARTUP_TIMEOUT_SECONDS;m.STARTUP_TIMEOUT_SECONDS=.2
 try:
  for mode,status in [('early','remote_early_exit'),('timeout','start_timeout'),('malformed','malformed_readiness')]:
   with tempfile.TemporaryDirectory() as raw:
    td=pathlib.Path(raw);rc,r=execute(plan(td,server_mode=mode),td);assert rc==1 and r['result']['status']==status and not r['dispatch']['periodic_client_entered'];assert all(x['status']=='verified' for x in r['cleanup']['postchecks'])
 finally:m.STARTUP_TIMEOUT_SECONDS=old
def test_missing_summary_fails_closed():
 with tempfile.TemporaryDirectory() as raw:
  td=pathlib.Path(raw);rc,r=execute(plan(td,client_mode='missing'),td);assert rc==1 and r['result']['status']=='malformed_result' and 'periodic_summary' in r['result']['failure_reason']
def test_contradictory_accounting_fails_closed():
 with tempfile.TemporaryDirectory() as raw:
  td=pathlib.Path(raw);p=plan(td);exe=pathlib.Path(p['client']['binary']['path']);exe.write_text(exe.read_text().replace('attempted=2 confirmed=1 missing=1','attempted=2 confirmed=2 missing=1'));p['server']['binary']=p['client']['binary']=binary(str(exe));rc,r=execute(p,td);assert rc==1 and r['result']['status']=='malformed_result' and 'contradiction' in r['result']['failure_reason']
def test_bounded_logs_fail_closed():
 old=m.MAX_LOG_BYTES;m.MAX_LOG_BYTES=1024
 try:
  with tempfile.TemporaryDirectory() as raw:
   td=pathlib.Path(raw);rc,r=execute(plan(td,server_mode='huge'),td);assert rc==1 and r['result']['status']=='remote_early_exit';assert r['private_logs']['truncated']
 finally:m.MAX_LOG_BYTES=old
def test_dry_and_validate_are_non_live_and_private():
 with tempfile.TemporaryDirectory() as raw:
  td=pathlib.Path(raw);p=plan(td,True);marker=td/'cleanup';p['cleanup']['local_argv']=cleanup(marker);out=td/'out';assert m.run(write(p,td),str(out),False,True)==0;assert not marker.exists();r=json.loads(out.read_text());assert not r['live_evidence'] and not r['dispatch']['periodic_client_entered'];assert m.run(write(p,td),str(td/'never'),True,False)==0 and not (td/'never').exists()
def test_hash_mismatch_and_cleanup_failure():
 with tempfile.TemporaryDirectory() as raw:
  td=pathlib.Path(raw);p=plan(td);p['client']['binary']['sha256']='f'*64
  try:execute(p,td);assert False
  except m.PlanError as exc:assert 'identity mismatch' in str(exc)
 with tempfile.TemporaryDirectory() as raw:
  td=pathlib.Path(raw);p=plan(td);p['cleanup']['local_argv']=cleanup(count=1);rc,r=execute(p,td);assert rc==1 and r['result']['status']=='cleanup_failed'
def main():
 for name,value in sorted(globals().items()):
  if name.startswith('test_'):value()
 print('periodic command tests: ok')
if __name__=='__main__':main()
