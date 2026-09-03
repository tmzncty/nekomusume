#!/usr/bin/env python3
import importlib.util,json,pathlib,subprocess,sys,tempfile
S=pathlib.Path(__file__).with_name('run-periodic-command.py'); sp=importlib.util.spec_from_file_location('x',S); m=importlib.util.module_from_spec(sp); sp.loader.exec_module(m)
def plan(): return {'git_commit':'0'*40,'binary':{'path':'/fake/neko','sha256':'a'*64,'bytes':42},'server_argv':['/fake/neko','periodic-server','--label','a;b|$(touch p)'],'client_argv':['/fake/neko','periodic-client','--label','quoted, literal'],'cleanup_argv':['/fake/cleanup','--scope','periodic'],'timeout_seconds':3}
def write(x,d): p=pathlib.Path(d)/'p.json'; p.write_text(json.dumps(x)); return str(p)
def test_rejects():
 with tempfile.TemporaryDirectory() as d:
  p=plan(); p['server_argv']='not argv'
  try:m.load_plan(write(p,d)); assert False
  except m.PlanError:pass
  try:m.load_plan(write({'x':1},d)); assert False
  except m.PlanError:pass
def test_literals_and_dry_run():
 with tempfile.TemporaryDirectory() as d:
  out=pathlib.Path(d)/'o'; assert m.run(write(plan(),d),str(out),False,True)==0; r=json.loads(out.read_text()); assert r['live_evidence'] is False and r['dispatch']['entered'] and r['dispatch']['periodic_client_entered']; assert r['dispatch']['server_argv'][-1]=='a;b|$(touch p)'
def test_validate_no_side_effects():
 with tempfile.TemporaryDirectory() as d:
  out=pathlib.Path(d)/'o'; assert m.run(write(plan(),d),str(out),True,False)==0; assert not out.exists()
def test_timeout_cleanup():
 with tempfile.TemporaryDirectory() as d:
  p=plan(); p['server_argv']=[sys.executable,'-c','import time; time.sleep(30)']; p['client_argv']=[sys.executable,'-c','import time; time.sleep(30)']; p['binary']['path']=sys.executable
  out=pathlib.Path(d)/'o'; assert m.run(write(p,d),str(out),False,False)==1; r=json.loads(out.read_text()); assert r['result']['status']=='timeout' and r['cleanup']['processes_remaining']==0
def main():
 for n,v in sorted(globals().items()):
  if n.startswith('test_'):v()
if __name__=='__main__':main()

def test_missing_client_start_is_bounded_and_reported():
 with tempfile.TemporaryDirectory() as d:
  p=plan(); p['client_argv']=['/missing/client','periodic-client']; p['binary']['path']='/missing/client'; p['server_argv']=['/missing/server','periodic-server']; out=pathlib.Path(d)/'o'; assert m.run(write(p,d),str(out),False,False)==1; r=json.loads(out.read_text()); assert r['result']['status']=='start_error' and r['cleanup']['processes_remaining']==0
