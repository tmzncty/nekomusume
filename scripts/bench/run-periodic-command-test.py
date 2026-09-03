#!/usr/bin/env python3
from __future__ import annotations
import hashlib,importlib.util,json,pathlib,shutil,sys,tempfile
HERE=pathlib.Path(__file__).resolve().parent;S=HERE/'run-periodic-command.py';REMOTE=HERE/'remote-endpoint-exec.py'
sp=importlib.util.spec_from_file_location('periodic',S);m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
def make_binary(td:pathlib.Path)->str:
 p=td/'neko';p.write_text('#!'+sys.executable+'\nimport pathlib,sys\nif len(sys.argv)>2:pathlib.Path(sys.argv[2]).write_text(sys.argv[3] if len(sys.argv)>3 else "entered")\n');p.chmod(0o700);return str(p)
def binary(path:str,sha=None,size=None):
 data=pathlib.Path(path).read_bytes();return {'path':path,'sha256':sha or hashlib.sha256(data).hexdigest(),'bytes':size or len(data),'git_commit':'0'*40}
def cleanup(marker:pathlib.Path|None=None,count=0):
 prefix=f'import pathlib;pathlib.Path({str(marker)!r}).write_text("called");' if marker else ''
 return [sys.executable,'-c',prefix+f'import json;print(json.dumps({{"listeners_remaining":{count},"processes_remaining":0}}))']
def local_plan(td:pathlib.Path):
 exe=make_binary(td);return {'git_commit':'0'*40,'server':{'execution':'local','binary':binary(exe),'argv':[exe,'periodic-server']},'client':{'execution':'local','binary':binary(exe),'argv':[exe,'periodic-client']},'cleanup':{'local_argv':cleanup()},'timeout_seconds':3}
def remote_plan(td:pathlib.Path):
 exe=make_binary(td);return {'git_commit':'0'*40,'server':{'execution':'ssh','binary':binary(exe),'argv':[exe,'periodic-server'],'ssh_executable':sys.executable,'transport_argv':[sys.executable,str(REMOTE)]},'client':{'execution':'local','binary':binary(exe),'argv':[exe,'periodic-client']},'cleanup':{'local_argv':cleanup(),'remote_transport_argv':cleanup()},'timeout_seconds':3}
def write(value,td):p=pathlib.Path(td)/'plan.json';p.write_text(json.dumps(value));return str(p)
def expect_reject(value,td,text):
 try:m.load_plan(write(value,td));assert False
 except m.PlanError as exc:assert text in str(exc),exc
def test_remote_dry_shape_and_literals():
 with tempfile.TemporaryDirectory() as raw:
  td=pathlib.Path(raw);p=remote_plan(td);literal='a;b|$(touch NEVER)';p['server']['argv'] += [str(td/'literal'),literal];out=td/'out';assert m.run(write(p,td),str(out),False,True)==0;r=json.loads(out.read_text());assert not r['live_evidence'] and r['dispatch']['periodic_client_entered'];assert r['endpoints']['server']['argv'][-1]==literal;assert r['resources']['server']=={'status':'not_collected_remote'};assert 'transport_argv' not in json.dumps(r)
def test_transport_executable_samefile_rejected():
 with tempfile.TemporaryDirectory() as raw:
  td=pathlib.Path(raw);p=remote_plan(td);fake=td/'ssh';shutil.copyfile(sys.executable,fake);fake.chmod(0o700);p['server']['transport_argv'][0]=str(fake);expect_reject(p,td,'SSH transport executable differs')
def test_remote_hash_mismatch_rejected_before_spawn():
 with tempfile.TemporaryDirectory() as raw:
  td=pathlib.Path(raw);marker=td/'server';p=remote_plan(td);p['server']['binary']['sha256']='f'*64;p['server']['argv'] += [str(marker),'bad'];out=td/'out';assert m.run(write(p,td),str(out),False,False)==1;r=json.loads(out.read_text());assert r['result']['server_exit']==2 and not marker.exists()
def test_real_remote_periodic_client_and_literal_dispatch_cleanup():
 with tempfile.TemporaryDirectory() as raw:
  td=pathlib.Path(raw);client=td/'client';literal=td/'literal';remote_cleanup=td/'remote-cleanup';p=remote_plan(td);text='x;$(touch nope)|&'
  p['server']['argv'] += [str(literal),text];p['client']['argv'] += [str(client),'entered'];p['cleanup']['remote_transport_argv']=cleanup(remote_cleanup)
  out=td/'out';assert m.run(write(p,td),str(out),False,False)==0;r=json.loads(out.read_text());assert client.read_text()=='entered' and literal.read_text()==text;assert remote_cleanup.read_text()=='called';assert len(r['cleanup']['postchecks'])==2 and all(x['status']=='verified' for x in r['cleanup']['postchecks']);assert r['resources']['server']=={'status':'not_collected_remote'}
def test_remote_cleanup_is_required_and_zero():
 with tempfile.TemporaryDirectory() as raw:
  td=pathlib.Path(raw);p=remote_plan(td);del p['cleanup']['remote_transport_argv'];expect_reject(p,td,'cleanup descriptor')
  p=remote_plan(td);p['cleanup']['remote_transport_argv']=cleanup(count=1);out=td/'o';assert m.run(write(p,td),str(out),False,False)==1;assert json.loads(out.read_text())['result']['status']=='cleanup_failed'
def test_unchanged_local_samefile_and_live_semantics():
 with tempfile.TemporaryDirectory() as raw:
  td=pathlib.Path(raw);p=local_plan(td);out=td/'o';assert m.run(write(p,td),str(out),False,False)==0;r=json.loads(out.read_text());assert r['resources']['server']=={'status':'local_process'} and len(r['cleanup']['postchecks'])==1
  other=td/'other';shutil.copyfile(p['client']['binary']['path'],other);other.chmod(0o700);p['client']['argv'][0]=str(other);expect_reject(p,td,'underlying binary')
def test_validate_only_no_output_or_execution():
 with tempfile.TemporaryDirectory() as raw:
  td=pathlib.Path(raw);p=remote_plan(td);p['server']['binary']['sha256']='0'*64;out=td/'o';assert m.run(write(p,td),str(out),True,False)==0 and not out.exists()
def main():
 for name,value in sorted(globals().items()):
  if name.startswith('test_'):value()
 print('periodic command tests: ok')
if __name__=='__main__':main()
