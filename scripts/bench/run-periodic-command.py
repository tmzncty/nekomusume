#!/usr/bin/env python3
"""Bounded shell-free periodic Session wrapper for one server and local client.

Plans are private.  An SSH server endpoint separates the remote Nekomusume
identity/argv from the locally verified SSH transport.  Its transport must run
remote-endpoint-exec.py, which accepts one JSON request on stdin and verifies
the remote bytes immediately before a direct spawn.
"""
from __future__ import annotations
import argparse, hashlib, json, os, pathlib, re, shutil, signal, subprocess, sys, threading, time
from typing import Any

MAX_ARGV=64; MAX_LOG_BYTES=1024*1024; STARTUP_TIMEOUT_SECONDS=5; HEX40=re.compile(r'^[0-9a-f]{40}$'); HEX64=re.compile(r'^[0-9a-f]{64}$')
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
    # argv and SSH transport can contain private addresses and paths.
    binary=endpoint['binary']
    return {'execution':endpoint['execution'],'binary':{'sha256':binary['sha256'],'bytes':binary['bytes'],'git_commit':binary['git_commit']}}

def option(argv: list[str], name: str) -> str|None:
    positions=[i for i,value in enumerate(argv) if value==name]
    if len(positions)>1:raise PlanError(f'duplicate {name}')
    if not positions:return None
    index=positions[0]
    if index+1>=len(argv):raise PlanError(f'missing value for {name}')
    return argv[index+1]

def positive_option(argv: list[str], name: str, maximum: int) -> int:
    raw=option(argv,name)
    if raw is None:raise PlanError(f'periodic argv lacks {name}')
    try:value=int(raw)
    except ValueError as exc:raise PlanError(f'invalid {name}') from exc
    if not 1<=value<=maximum:raise PlanError(f'invalid {name}')
    return value

def periodic_contract(server: list[str], client: list[str]) -> dict[str,Any]:
    contracts=[]
    for argv in (server,client):
        transport=option(argv,'--transport')
        if transport!='tcp':raise PlanError('invalid periodic transport')
        contracts.append({'transport':transport,'port':positive_option(argv,'--port',65535),'bytes':positive_option(argv,'--bytes',1200),'count':positive_option(argv,'--count',600),'duration_seconds':positive_option(argv,'--duration',600),'interval_ms':positive_option(argv,'--interval-ms',5000),'setup_timeout_ms':positive_option(argv,'--setup-timeout-ms',10000),'ack_timeout_ms':positive_option(argv,'--ack-timeout-ms',10000)})
    if contracts[0]!=contracts[1]:raise PlanError('server/client periodic contract mismatch')
    for argv,name in ((server,'--bind'),(client,'--addr')):
        endpoint=option(argv,name)
        if endpoint is None:raise PlanError(f'periodic argv lacks {name}')
        try:endpoint_port=int(endpoint.rsplit(':',1)[1])
        except (ValueError,IndexError) as exc:raise PlanError(f'invalid {name}') from exc
        if endpoint_port!=contracts[0]['port']:raise PlanError(f'{name} port mismatch')
    return contracts[0]

def sample_local_process(pid: int, current: dict[str,Any]) -> None:
    if os.name=='nt': return
    try:
        values=pathlib.Path(f'/proc/{pid}/stat').read_text().rsplit(')',1)[1].split(); ticks=os.sysconf('SC_CLK_TCK')
        current['cpu_user_seconds']=int(values[11])/ticks; current['cpu_system_seconds']=int(values[12])/ticks
    except (OSError,ValueError,IndexError): pass
    try:
        for line in pathlib.Path(f'/proc/{pid}/status').read_text().splitlines():
            if line.startswith('VmRSS:'):
                rss=int(line.split()[1]);current['max_rss_kib']=max(current.get('max_rss_kib',0),rss);break
    except (OSError,ValueError,IndexError): pass

def remote_protocol_entered(text: str) -> bool:
    matches=[line for line in text.splitlines() if line.startswith('remote_exec_protocol_accepted')]
    if len(matches)!=1:return False
    try:
        name,value=fields(matches[0])
        return name=='remote_exec_protocol_accepted' and value=={'protocol':REMOTE_EXEC_PROTOCOL,'role':'server'}
    except ValueError:return False

def diagnostic_class(server_exit: int|None, readiness: str|None, log: str, truncated: bool) -> str|None:
    if truncated:return 'log_overflow'
    if server_exit==255:return 'ssh_transport_exit'
    if server_exit==2:return 'remote_binary_identity_reject' if 'identity mismatch' in log else 'remote_exec_protocol_reject'
    if server_exit not in (None,0):return 'server_runtime_exit_before_ready'
    if readiness=='start_timeout':return 'start_timeout_no_terminal_evidence'
    return None

class BoundedCapture:
    def __init__(self, process: subprocess.Popen[bytes]):
        self.buffers={'stdout':bytearray(),'stderr':bytearray()}; self.truncated=False; self.lock=threading.Lock(); self.threads=[]
        for name,stream in (('stdout',process.stdout),('stderr',process.stderr)):
            assert stream is not None
            thread=threading.Thread(target=self._drain,args=(name,stream),daemon=True);thread.start();self.threads.append(thread)
    def _drain(self, name: str, stream: Any) -> None:
        while True:
            chunk=stream.read(65536)
            if not chunk:break
            with self.lock:
                room=MAX_LOG_BYTES-len(self.buffers[name])
                if room>0:self.buffers[name].extend(chunk[:room])
                if len(chunk)>room:self.truncated=True
    def text(self, name: str) -> str:
        with self.lock:return bytes(self.buffers[name]).decode('utf-8','replace')
    def finish(self) -> None:
        for thread in self.threads:thread.join(timeout=2)


def fields(line: str) -> tuple[str,dict[str,str]]:
    parts=line.split(); values={}
    for item in parts[1:]:
        if item.count('=')!=1: raise ValueError('malformed field')
        key,value=item.split('=',1)
        if not key or key in values: raise ValueError('duplicate field')
        values[key]=value
    return parts[0] if parts else '',values

def one_event(text: str, prefix: str) -> dict[str,str]:
    matches=[line for line in text.splitlines() if line.startswith(prefix)]
    if len(matches)!=1: raise ValueError(f'expected exactly one {prefix}')
    name,value=fields(matches[0])
    if name!=prefix: raise ValueError(f'malformed {prefix}')
    return value

def nat(values: dict[str,str], key: str) -> int:
    value=values.get(key,'')
    if not value.isdigit(): raise ValueError(f'invalid {key}')
    return int(value)

def boolean(values: dict[str,str], key: str) -> bool:
    if values.get(key) not in ('true','false'): raise ValueError(f'invalid {key}')
    return values[key]=='true'

def wait_ready(process: subprocess.Popen[bytes], capture: BoundedCapture, transport: str, port: int, timeout: float) -> tuple[str,dict[str,Any]|None]:
    deadline=time.monotonic()+timeout
    while time.monotonic()<deadline:
        text=capture.text('stdout')
        lines=[x for x in text.splitlines() if x.startswith('periodic_server_ready')]
        if lines:
            try:
                if len(lines)!=1: raise ValueError
                name,value=fields(lines[0])
                if name!='periodic_server_ready' or set(value)!={'transport','port','reconnect'} or value['transport']!=transport or nat(value,'port')!=port or value['reconnect']!='unsupported': raise ValueError
            except ValueError: return 'malformed_readiness',None
            return 'ready',{'transport':transport,'port':port,'address':'redacted','source':'periodic_server_ready'}
        if process.poll() is not None:return 'remote_early_exit',None
        time.sleep(.02)
    return 'start_timeout',None

def nearest_rank(samples: list[int], numerator: int, denominator: int=100) -> int:
    if not samples:raise ValueError('confirmed latency samples missing')
    ordered=sorted(samples);index=max(0,(len(ordered)*numerator+denominator-1)//denominator-1)
    return ordered[index]

def parse_result(client_text: str, server_text: str, elapsed_ms: int, declared: dict[str,Any]) -> dict[str,Any]:
    auth=one_event(client_text,'periodic_client_authenticated')
    if set(auth)!={'session','stream','reconnect'} or auth['reconnect']!='unsupported':raise ValueError('malformed client authentication')
    summary=one_event(client_text,'periodic_summary');server=one_event(server_text,'periodic_server_summary')
    required={'transport','session','stream','attempted','confirmed','missing','duplicates','p50_confirmation_latency_ms','p95_confirmation_latency_ms','reconnects','elapsed_ms','application_bytes','cleanup','signal'}
    if set(summary)!=required or summary['transport']!=declared['transport'] or summary['cleanup']!='verified' or summary['session']!=auth['session'] or summary['stream']!=auth['stream']:raise ValueError('malformed client summary')
    server_required={'authenticated','received','confirmed','duplicates','elapsed_ms','cleanup','signal'}
    if set(server)!=server_required or server['cleanup']!='verified' or not boolean(server,'authenticated'):raise ValueError('malformed server summary')
    if boolean(summary,'signal') or boolean(server,'signal'):raise ValueError('signal interruption')
    intervals=[]
    for line in client_text.splitlines():
        if not line.startswith('periodic_interval'):continue
        name,value=fields(line)
        if name!='periodic_interval' or set(value)!={'seq','sent','confirmed','missing','duplicate','latency_ms'}:raise ValueError('malformed interval')
        confirmed_value=boolean(value,'confirmed');missing_value=boolean(value,'missing');latency=value['latency_ms']
        if latency=='null':latency_value=None
        elif latency.isdigit():latency_value=int(latency)
        else:raise ValueError('invalid latency_ms')
        if confirmed_value==(latency_value is None) or confirmed_value==missing_value:raise ValueError('interval latency/accounting contradiction')
        intervals.append({'seq':nat(value,'seq'),'sent':boolean(value,'sent'),'confirmed':confirmed_value,'missing':missing_value,'duplicate':boolean(value,'duplicate'),'latency_ms':latency_value})
    attempted=nat(summary,'attempted');confirmed=nat(summary,'confirmed');missing=nat(summary,'missing');client_duplicates=nat(summary,'duplicates');reconnects=nat(summary,'reconnects')
    if len(intervals)!=attempted or [x['seq'] for x in intervals]!=list(range(1,attempted+1)):raise ValueError('interval sequence/accounting contradiction')
    if sum(x['sent'] for x in intervals)!=attempted or sum(x['confirmed'] for x in intervals)!=confirmed or sum(x['missing'] for x in intervals)!=missing or sum(x['duplicate'] for x in intervals)!=client_duplicates:raise ValueError('interval accounting contradiction')
    server_received=nat(server,'received');server_confirmed=nat(server,'confirmed');server_duplicates=nat(server,'duplicates');application_bytes=nat(summary,'application_bytes')
    if attempted!=declared['count'] or confirmed!=attempted or missing!=0 or reconnects!=0:raise ValueError('declared completion contradiction')
    if server_received!=attempted or server_confirmed!=confirmed:raise ValueError('server/client accounting contradiction')
    if application_bytes!=attempted*declared['bytes']:raise ValueError('application byte accounting contradiction')
    latencies=[x['latency_ms'] for x in intervals if x['confirmed']]
    p50=nat(summary,'p50_confirmation_latency_ms');p95=nat(summary,'p95_confirmation_latency_ms')
    if p50!=nearest_rank(latencies,50) or p95!=nearest_rank(latencies,95):raise ValueError('latency percentile contradiction')
    return {'declared':declared,'actual_duration_ms':nat(summary,'elapsed_ms'),'wrapper_elapsed_ms':elapsed_ms,'attempted':attempted,'confirmed':confirmed,'missing':missing,'client_duplicates':client_duplicates,'server_duplicates':server_duplicates,'reconnects':reconnects,'application_bytes':application_bytes,'confirmation_latencies_ms':latencies,'median_confirmation_latency_ms':p50,'p95_confirmation_latency_ms':p95,'client_elapsed_ms':nat(summary,'elapsed_ms'),'server_elapsed_ms':nat(server,'elapsed_ms')}

def run(plan_path: str, output: str|None, validate_only: bool, dry_run: bool) -> int:
    plan=load_plan(plan_path);remote=plan['server']['execution']=='ssh';declared=periodic_contract(plan['server']['argv'],plan['client']['argv']);transport=declared['transport'];port=declared['port']
    report={'schema':'nekomusume.periodic-command.v3','dry_run':dry_run,'live_evidence':not dry_run and not validate_only,'validate_only':validate_only,'git_commit':plan['git_commit'],'endpoints':{'server':public_endpoint(plan['server']),'client':public_endpoint(plan['client'])},'endpoint_provenance':{'transport':transport,'port':port,'address':'redacted','source':'declared_periodic_server_argv'},'dispatch':{'periodic_client_entered':False},'resources':{'server':{'status':'not_collected_remote'} if remote else {'status':'not_collected_local_server'},'client':{'status':'not_started'}},'cleanup':{'required':['local']+(['remote'] if remote else [])}}
    if validate_only or dry_run:
        report['dispatch']['entered']=False
        if output and not validate_only:pathlib.Path(output).write_text(json.dumps(report,sort_keys=True,separators=(',',':'))+'\n',encoding='utf-8')
        return 0
    server=client=None; server_capture=client_capture=None; startup_error=None; readiness=None; timed_out=False; parsed=None; parse_error=None
    started=time.monotonic(); client_resources={}
    try:
        verify_local(plan['client'])
        if not remote:verify_local(plan['server'])
        server_argv,server_input=dispatch(plan['server'])
        try:
            server=subprocess.Popen(server_argv,stdin=subprocess.PIPE if server_input is not None else subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=(os.name!='nt'),shell=False);server_capture=BoundedCapture(server)
            if server_input is not None:
                assert server.stdin is not None;server.stdin.write(server_input);server.stdin.close()
            readiness,observed=wait_ready(server,server_capture,transport,port,min(STARTUP_TIMEOUT_SECONDS,plan['timeout_seconds']))
            report['readiness']={'status':readiness}
            if observed:report['endpoint_provenance']=observed
            if readiness=='ready':
                client=subprocess.Popen(plan['client']['argv'],stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=(os.name!='nt'),shell=False);client_capture=BoundedCapture(client);report['dispatch']['periodic_client_entered']=True
                deadline=time.monotonic()+plan['timeout_seconds']
                while time.monotonic()<deadline:
                    if client.poll() is None:sample_local_process(client.pid,client_resources)
                    if client.poll() is not None and server.poll() is not None:break
                    time.sleep(.02)
                else:timed_out=True
                sample_local_process(client.pid,client_resources)
        except OSError as exc:startup_error=type(exc).__name__
    finally:
        stop(client);stop(server)
        if client_capture:client_capture.finish()
        if server_capture:server_capture.finish()
    server_text=server_capture.text('stdout') if server_capture else ''
    client_text=client_capture.text('stdout') if client_capture else ''
    truncated=bool((server_capture and server_capture.truncated) or (client_capture and client_capture.truncated))
    diagnostic_streams=[]
    for scope,capture in (('server',server_capture),('client',client_capture)):
        if capture:
            for stream in ('stdout','stderr'):
                payload=capture.text(stream).encode('utf-8')
                diagnostic_streams.append({'scope':scope+'_'+stream,'sha256':hashlib.sha256(payload).hexdigest(),'bytes':len(payload),'truncated':capture.truncated})
    report['private_logs']={'retained_for_parsing':True,'tracked':False,'storage':'bounded_memory','bounded_bytes_per_stream':MAX_LOG_BYTES,'streams':['server_stdout','server_stderr','client_stdout','client_stderr'],'truncated':truncated,'diagnostics':diagnostic_streams}
    report['diagnostics']={'capture_started':server_capture is not None,'protocol_entered':remote_protocol_entered(server_text),'readiness_observed':readiness=='ready','class':diagnostic_class(server.returncode if server else None,readiness,server_text,truncated)}
    if readiness=='ready' and not timed_out and not startup_error and not truncated:
        try:parsed=parse_result(client_text,server_text,int((time.monotonic()-started)*1000),declared)
        except ValueError as exc:parse_error=str(exc)
    checks=[cleanup_check(plan['cleanup']['local_argv'],'local processes/listeners')]
    if remote:checks.append(cleanup_check(plan['cleanup']['remote_transport_argv'],'remote processes/listeners'))
    report['cleanup']['postchecks']=checks; cleanup_ok=all(x['status']=='verified' for x in checks)
    if client is not None:
        report['resources']['client']={'status':'collected_local_direct_child','source':'/proc/<client-pid>','cpu_user_seconds':client_resources.get('cpu_user_seconds'),'cpu_system_seconds':client_resources.get('cpu_system_seconds'),'max_rss_kib':client_resources.get('max_rss_kib')}
    if startup_error:status='start_error'
    elif readiness and readiness!='ready':status=readiness
    elif timed_out:status='timeout'
    elif truncated:status='log_limit_exceeded'
    elif parse_error:status='malformed_result'
    elif not cleanup_ok:status='cleanup_failed'
    else:status='passed' if parsed is not None and server and client and server.returncode==0 and client.returncode==0 else 'failed'
    report['result']={'status':status,'server_exit':server.returncode if server else None,'client_exit':client.returncode if client else None}
    if parsed:report['result'].update(parsed)
    if startup_error:report['result']['error_type']=startup_error
    if parse_error:report['result']['failure_reason']=parse_error
    if output:pathlib.Path(output).write_text(json.dumps(report,sort_keys=True,separators=(',',':'))+'\n',encoding='utf-8')
    return 0 if status=='passed' else 1

def main()->int:
    p=argparse.ArgumentParser();p.add_argument('--plan',required=True);p.add_argument('--output');p.add_argument('--validate-only',action='store_true');p.add_argument('--dry-run',action='store_true');a=p.parse_args()
    if a.validate_only and a.dry_run:p.error('validate-only and dry-run are exclusive')
    return run(a.plan,a.output,a.validate_only,a.dry_run)
if __name__=='__main__':
    try:raise SystemExit(main())
    except PlanError as exc:print(f'plan error: {exc}',file=sys.stderr);raise SystemExit(2)
