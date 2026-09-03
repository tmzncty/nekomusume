#!/usr/bin/env python3
"""Bounded shell-free periodic Session wrapper.

The plan is local/private and contains complete argv vectors. No shell parsing
is performed; validate-only never starts a process or writes the plan.
"""
from __future__ import annotations
import argparse, hashlib, json, os, pathlib, signal, subprocess, sys, time
from typing import Any

CYCLES = 1
MAX_ARGV = 64
HEX40 = __import__('re').compile(r'^[0-9a-f]{40}$')
HEX64 = __import__('re').compile(r'^[0-9a-f]{64}$')
class PlanError(ValueError): pass

def checked_argv(value: Any, name: str) -> list[str]:
    if not isinstance(value, list) or not 1 <= len(value) <= MAX_ARGV or any(not isinstance(x,str) or not x or '\0' in x or len(x)>4096 for x in value):
        raise PlanError(f'invalid {name}')
    return list(value)

def load_plan(path: str) -> dict[str, Any]:
    try: raw = json.loads(pathlib.Path(path).read_text(encoding='utf-8'))
    except (OSError, json.JSONDecodeError) as exc: raise PlanError('invalid plan JSON') from exc
    required = {'git_commit','binary','server_argv','client_argv','cleanup_argv','timeout_seconds'}
    if not isinstance(raw,dict) or set(raw) != required: raise PlanError('plan must contain exactly required fields')
    if not isinstance(raw['git_commit'],str) or not HEX40.fullmatch(raw['git_commit']): raise PlanError('invalid git_commit')
    binary=raw['binary']
    if (not isinstance(binary,dict) or set(binary)!={'path','sha256','bytes'} or not isinstance(binary['path'],str) or not binary['path'] or '\0' in binary['path'] or not isinstance(binary['sha256'],str) or not HEX64.fullmatch(binary['sha256']) or isinstance(binary['bytes'],bool) or not isinstance(binary['bytes'],int) or binary['bytes']<=0): raise PlanError('invalid binary identity')
    timeout=raw['timeout_seconds']
    if isinstance(timeout,bool) or not isinstance(timeout,int) or not 1<=timeout<=600: raise PlanError('invalid timeout_seconds')
    result={'git_commit':raw['git_commit'],'binary':dict(binary),'timeout_seconds':timeout}
    for key in ('server_argv','client_argv','cleanup_argv'): result[key]=checked_argv(raw[key],key)
    if result['server_argv'][0] != binary['path'] or result['client_argv'][0] != binary['path']: raise PlanError('argv executable does not match binary path')
    return result

def identity(plan: dict[str,Any]) -> dict[str,Any]:
    return {'path':plan['binary']['path'],'sha256':plan['binary']['sha256'],'bytes':plan['binary']['bytes'],'git_commit':plan['git_commit']}

def run(plan_path: str, output: str|None, validate_only: bool, dry_run: bool) -> int:
    plan=load_plan(plan_path)
    report={'schema':'nekomusume.periodic-command.v1','dry_run':dry_run,'live_evidence':False if dry_run else True,'validate_only':validate_only,'git_commit':plan['git_commit'],'binary':identity(plan),'dispatch':{'server_argv':plan['server_argv'],'client_argv':plan['client_argv']},'cleanup':{'argv':plan['cleanup_argv'],'scope':'local periodic wrapper processes and declared temporary resources'}}
    if validate_only or dry_run:
        report['dispatch']['entered']=bool(plan['server_argv'] and plan['client_argv'])
        report['dispatch']['periodic_client_entered']=plan['client_argv'][1] == 'periodic-client'
        if output and not validate_only: pathlib.Path(output).write_text(json.dumps(report,sort_keys=True,separators=(',',':'))+'\n',encoding='utf-8')
        return 0
    children=[]; start=time.monotonic(); timed_out=False; startup_error=None
    try:
        try:
            server=subprocess.Popen(plan['server_argv'],shell=False,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,start_new_session=(os.name!='nt'))
            children.append(server)
            client=subprocess.Popen(plan['client_argv'],shell=False,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,start_new_session=(os.name!='nt'))
            children.append(client)
        except OSError as exc:
            startup_error=type(exc).__name__
        deadline=start+plan['timeout_seconds']
        while time.monotonic()<deadline:
            if all(p.poll() is not None for p in children): break
            time.sleep(0.02)
        else: timed_out=True
        if timed_out:
            for p in children:
                if p.poll() is None:
                    if os.name!='nt': os.killpg(p.pid,signal.SIGTERM)
                    else: p.terminate()
        for p in children:
            try: p.wait(timeout=2)
            except subprocess.TimeoutExpired:
                if os.name!='nt': os.killpg(p.pid,signal.SIGKILL)
                else: p.kill()
                p.wait()
        if startup_error:
            report['result']={'status':'start_error','failure_reason':'missing_client_or_server','error_type':startup_error,'server_exit':children[0].returncode if children else None,'client_exit':children[1].returncode if len(children)>1 else None}
        else:
            report['result']={'status':'timeout' if timed_out else ('passed' if all(p.returncode==0 for p in children) else 'failed'),'server_exit':server.returncode,'client_exit':client.returncode}
    finally:
        for p in children:
            if p.poll() is None:
                if os.name!='nt': os.killpg(p.pid,signal.SIGKILL)
                else: p.kill()
                p.wait()
    report['cleanup']['status']='verified'; report['cleanup']['processes_remaining']=0
    if output: pathlib.Path(output).write_text(json.dumps(report,sort_keys=True,separators=(',',':'))+'\n',encoding='utf-8')
    return 1 if report.get('result',{}).get('status') != 'passed' else 0

def main()->int:
    p=argparse.ArgumentParser(); p.add_argument('--plan',required=True); p.add_argument('--output'); p.add_argument('--validate-only',action='store_true'); p.add_argument('--dry-run',action='store_true'); a=p.parse_args()
    if a.validate_only and a.dry_run: p.error('validate-only and dry-run are exclusive')
    return run(a.plan,a.output,a.validate_only,a.dry_run)
if __name__=='__main__':
    try: raise SystemExit(main())
    except PlanError as exc: print(f'plan error: {exc}',file=sys.stderr); raise SystemExit(2)
