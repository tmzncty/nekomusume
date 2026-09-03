#!/usr/bin/env python3
"""Bounded shell-free periodic Session wrapper for one server and local client.

Plans are private.  An SSH server endpoint separates the remote Nekomusume
identity/argv from the locally verified SSH transport.  Its transport must run
remote-endpoint-exec.py, which accepts one JSON request on stdin and verifies
the remote bytes immediately before a direct spawn.
"""
from __future__ import annotations
import argparse, hashlib, json, os, pathlib, re, shutil, signal, subprocess, sys, time
from typing import Any

MAX_ARGV=64; HEX40=re.compile(r'^[0-9a-f]{40}$'); HEX64=re.compile(r'^[0-9a-f]{64}$')
REMOTE_EXEC_PROTOCOL='nekomusume.remote-exec.v1'
class PlanError(ValueError): pass

def checked_argv(value: Any, name: str) -> list[str]:
    if not isinstance(value,list) or not 1<=len(value)<=MAX_ARGV or any(not isinstance(x,str) or not x or '\0' in x or len(x)>4096 for x in value): raise PlanError(f'invalid {name}')
    return list(value)
def checked_binary(value: Any, name: str, commit: str) -> dict[str,Any]:
    if (not isinstance(value,dict) or set(value)!={'path','sha256','bytes','git_commit'} or not isinstance(value['path'],str) or not value['path'] or '\0' in value['path'] or not isinstance(value['sha256'],str) or not HEX64.fullmatch(value['sha256']) or isinstance(value['bytes'],bool) or not isinstance(value['bytes'],int) or value['bytes']<=0 or value['git_commit']!=commit): raise PlanError(f'invalid {name} binary identity')
    return dict(value)
def executable_path(value: str) -> pathlib.Path:
    candidate=pathlib.Path(value).expanduser()
    if not candidate.is_absolute() and candidate.parent==pathlib.Path('.'):
        found=shutil.which(value)
        if found is None: raise PlanError('command executable is unavailable')
        candidate=pathlib.Path(found)
    try: return candidate.resolve(strict=True)
    except OSError as exc: raise PlanError('command executable is unavailable') from exc
def same_executable(left: str, right: str) -> bool:
    try: return os.path.samefile(executable_path(left),executable_path(right))
    except OSError as exc: raise PlanError('cannot compare command executable') from exc

def load_plan(path: str) -> dict[str,Any]:
    try: raw=json.loads(pathlib.Path(path).read_text(encoding='utf-8'))
    except (OSError,json.JSONDecodeError) as exc: raise PlanError('invalid plan JSON') from exc
    required={'git_commit','server','client','cleanup','timeout_seconds'}
    if not isinstance(raw,dict) or set(raw)!=required: raise PlanError('plan must contain exactly required fields')
    commit=raw['git_commit']
    if not isinstance(commit,str) or not HEX40.fullmatch(commit): raise PlanError('invalid git_commit')
    timeout=raw['timeout_seconds']
    if isinstance(timeout,bool) or not isinstance(timeout,int) or not 1<=timeout<=600: raise PlanError('invalid timeout_seconds')
    endpoints={}
    for role in ('server','client'):
        item=raw[role]; execution=item.get('execution') if isinstance(item,dict) else None
        keys={'execution','binary','argv'} | ({'ssh_executable','transport_argv'} if execution=='ssh' else set())
        if not isinstance(item,dict) or set(item)!=keys or execution not in ('local','ssh'): raise PlanError(f'invalid {role} endpoint')
        if role=='client' and execution!='local': raise PlanError('periodic client must be local')
        binary=checked_binary(item['binary'],role,commit); argv=checked_argv(item['argv'],f'{role} argv')
        if argv[0]!=binary['path']: raise PlanError(f'{role} argv executable differs from underlying binary')
        if len(argv)<2 or argv[1]!=f'periodic-{role}': raise PlanError(f'{role} argv does not dispatch periodic-{role}')
        endpoint={'execution':execution,'binary':binary,'argv':argv}
        if execution=='ssh':
            transport=checked_argv(item['transport_argv'],'SSH transport argv'); declared=item['ssh_executable']
            if not isinstance(declared,str) or not declared or not same_executable(transport[0],declared): raise PlanError('SSH transport executable differs from declared SSH executable')
            endpoint.update(ssh_executable=declared,transport_argv=transport)
        endpoints[role]=endpoint
    cleanup=raw['cleanup']
    expected={'local_argv'} | ({'remote_transport_argv'} if endpoints['server']['execution']=='ssh' else set())
    if not isinstance(cleanup,dict) or set(cleanup)!=expected: raise PlanError('invalid cleanup descriptor')
    normalized_cleanup={'local_argv':checked_argv(cleanup['local_argv'],'local cleanup argv')}
    if 'remote_transport_argv' in cleanup:
        normalized_cleanup['remote_transport_argv']=checked_argv(cleanup['remote_transport_argv'],'remote cleanup transport argv')
        if not same_executable(normalized_cleanup['remote_transport_argv'][0],endpoints['server']['ssh_executable']): raise PlanError('remote cleanup transport executable differs from declared SSH executable')
    return {'git_commit':commit,'server':endpoints['server'],'client':endpoints['client'],'cleanup':normalized_cleanup,'timeout_seconds':timeout}

def verify_local(endpoint: dict[str,Any]) -> None:
    binary=endpoint['binary']
    if not same_executable(endpoint['argv'][0],binary['path']): raise PlanError('local endpoint executable differs from declared binary')
    try:
        with pathlib.Path(binary['path']).open('rb') as stream:
            stat=os.fstat(stream.fileno()); digest=hashlib.file_digest(stream,'sha256').hexdigest(); post=os.fstat(stream.fileno())
    except OSError as exc: raise PlanError('local endpoint binary unavailable') from exc
    if stat.st_size!=binary['bytes'] or post.st_size!=stat.st_size or digest!=binary['sha256']: raise PlanError('local endpoint binary identity mismatch')
def dispatch(endpoint: dict[str,Any]) -> tuple[list[str],bytes|None]:
    if endpoint['execution']=='local': return endpoint['argv'],None
    request={'protocol':REMOTE_EXEC_PROTOCOL,'role':'server','binary':endpoint['binary'],'argv':endpoint['argv']}
    return endpoint['transport_argv'],json.dumps(request,separators=(',',':')).encode()
def stop(process: subprocess.Popen[bytes]|None) -> None:
    if process is None or process.poll() is not None:return
    try:
        if os.name!='nt': os.killpg(process.pid,signal.SIGTERM)
        else: process.terminate()
        process.wait(timeout=2)
    except (ProcessLookupError,subprocess.TimeoutExpired):
        if process.poll() is None:
            if os.name!='nt': os.killpg(process.pid,signal.SIGKILL)
            else: process.kill()
            process.wait()
def cleanup_check(argv: list[str], scope: str) -> dict[str,Any]:
    try: completed=subprocess.run(argv,shell=False,capture_output=True,text=True,timeout=5,start_new_session=(os.name!='nt'))
    except (OSError,subprocess.TimeoutExpired) as exc: return {'scope':scope,'status':'failed','error_type':type(exc).__name__}
    try:
        value=json.loads(completed.stdout); listeners=value['listeners_remaining']; processes=value['processes_remaining']
        if set(value)!={'listeners_remaining','processes_remaining'} or any(isinstance(x,bool) or not isinstance(x,int) or x<0 for x in (listeners,processes)): raise ValueError
    except (json.JSONDecodeError,KeyError,TypeError,ValueError): return {'scope':scope,'status':'failed','exit':completed.returncode,'failure_reason':'invalid_cleanup_evidence'}
    return {'scope':scope,'status':'verified' if completed.returncode==0 and listeners==0 and processes==0 else 'failed','exit':completed.returncode,'listeners_remaining':listeners,'processes_remaining':processes}

def public_endpoint(endpoint: dict[str,Any]) -> dict[str,Any]:
    return {'execution':endpoint['execution'],'binary':endpoint['binary'],'argv':endpoint['argv']}
def run(plan_path: str, output: str|None, validate_only: bool, dry_run: bool) -> int:
    plan=load_plan(plan_path); remote=plan['server']['execution']=='ssh'
    report={'schema':'nekomusume.periodic-command.v2','dry_run':dry_run,'live_evidence':not dry_run and not validate_only,'validate_only':validate_only,'git_commit':plan['git_commit'],'endpoints':{'server':public_endpoint(plan['server']),'client':public_endpoint(plan['client'])},'dispatch':{'periodic_client_entered':plan['client']['argv'][1]=='periodic-client'},'resources':{'server':{'status':'not_collected_remote'} if remote else {'status':'local_process'},'client':{'status':'local_process'}},'cleanup':{'required':['local']+(['remote'] if remote else [])}}
    if validate_only or dry_run:
        report['dispatch']['entered']=True
        if output and not validate_only:pathlib.Path(output).write_text(json.dumps(report,sort_keys=True,separators=(',',':'))+'\n',encoding='utf-8')
        return 0
    children=[]; timed_out=False; startup_error=None; server=None; client=None
    try:
        verify_local(plan['client'])
        if not remote: verify_local(plan['server'])
        server_argv,server_input=dispatch(plan['server'])
        try:
            server=subprocess.Popen(server_argv,stdin=subprocess.PIPE if server_input is not None else subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,start_new_session=(os.name!='nt'),shell=False); children.append(server)
            if server_input is not None:
                assert server.stdin is not None; server.stdin.write(server_input); server.stdin.close()
            client=subprocess.Popen(plan['client']['argv'],stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,start_new_session=(os.name!='nt'),shell=False); children.append(client)
        except OSError as exc: startup_error=type(exc).__name__
        deadline=time.monotonic()+plan['timeout_seconds']
        while time.monotonic()<deadline:
            if all(p.poll() is not None for p in children):break
            time.sleep(.02)
        else: timed_out=True
    finally:
        stop(client); stop(server)
    checks=[cleanup_check(plan['cleanup']['local_argv'],'local processes/listeners')]
    if remote: checks.append(cleanup_check(plan['cleanup']['remote_transport_argv'],'remote processes/listeners'))
    report['cleanup']['postchecks']=checks
    cleanup_ok=all(x['status']=='verified' for x in checks)
    if startup_error: status='start_error'
    elif timed_out: status='timeout'
    elif not cleanup_ok: status='cleanup_failed'
    else: status='passed' if server is not None and client is not None and server.returncode==0 and client.returncode==0 else 'failed'
    report['result']={'status':status,'server_exit':server.returncode if server else None,'client_exit':client.returncode if client else None}
    if startup_error:report['result']['error_type']=startup_error
    if output:pathlib.Path(output).write_text(json.dumps(report,sort_keys=True,separators=(',',':'))+'\n',encoding='utf-8')
    return 0 if status=='passed' else 1

def main()->int:
    p=argparse.ArgumentParser();p.add_argument('--plan',required=True);p.add_argument('--output');p.add_argument('--validate-only',action='store_true');p.add_argument('--dry-run',action='store_true');a=p.parse_args()
    if a.validate_only and a.dry_run:p.error('validate-only and dry-run are exclusive')
    return run(a.plan,a.output,a.validate_only,a.dry_run)
if __name__=='__main__':
    try:raise SystemExit(main())
    except PlanError as exc:print(f'plan error: {exc}',file=sys.stderr);raise SystemExit(2)
