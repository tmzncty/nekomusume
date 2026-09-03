#!/usr/bin/env python3
import importlib.util,json,pathlib,tempfile
p=pathlib.Path(__file__).with_name('validate-hy2-owned-lab.py'); spec=importlib.util.spec_from_file_location('validator',p); v=importlib.util.module_from_spec(spec); spec.loader.exec_module(v)
tmp=pathlib.Path(tempfile.mkdtemp()); good=tmp/'time'
good.write_text('Command exited with non-zero status 7\n{"sentinel":"nekomusume.gnu-time.v1","elapsed_seconds":1.25,"cpu_user_seconds":0.1,"cpu_system_seconds":0.2,"rss_kib":12,"exit_code":7}\n')
assert v.parse_time(good)['elapsed_seconds']==1.25
for text in ('diagnostic only\n','{"sentinel":"nekomusume.gnu-time.v1","elapsed_seconds":-1,"cpu_user_seconds":0,"cpu_system_seconds":0,"rss_kib":1,"exit_code":0}\n','{"sentinel":"nekomusume.gnu-time.v1","elapsed_seconds":NaN,"cpu_user_seconds":0,"cpu_system_seconds":0,"rss_kib":1,"exit_code":0}\n',good.read_text()+good.read_text()):
 good.write_text(text)
 try: v.parse_time(good)
 except ValueError: pass
 else: raise AssertionError('malformed time accepted')
h='0'*64
def row(impl,run,fail=0):
 return {'name':f'{impl}-{run}','implementation':impl,'run':run,'failures':fail,'elapsed_seconds':1.0 if not fail else None,'cpu_user_seconds':.1 if not fail else None,'cpu_system_seconds':.1 if not fail else None,'rss_kib':1 if not fail else None,'fd_count':1 if not fail else None,'application_bytes':1200 if not fail else 0,'payload_sha256':h if not fail else None,'wire_bytes':None,'exit_code':0 if not fail else 1,'failure_stage':None if not fail else 'client_exit'}
rows=[]
for i in range(1,5): rows += [row('nekomusume',i),row('hy2',i)]
rows += [row('nekomusume',5,1)]
records=tmp/'records.jsonl'; records.write_text(''.join(json.dumps(x)+'\n' for x in rows))
blocked=tmp/'blocked.json'; doc={'schema':'nekomusume.benchmark-blocked-harness.v1','experiment_id':'hy2-owned-lab-paired','git_commit':'0'*40,'status':'BLOCKED_HARNESS','failure_stage':'hy2-5-client','contract':{'runs_per_implementation':5,'payload_bytes':1200,'payload_prepared':True,'payload_sha256':h},'samples':v.load_jsonl(records),'cleanup_status':'verified','cleanup_evidence':{'local_processes_reaped':True,'local_listeners_remaining':0,'remote_process_groups_reaped':True,'remote_listeners_remaining':0,'remote_temp_path_removed':True}}
v.atomic_write(blocked,doc); v.validate_result(blocked); assert len(json.load(open(blocked))['samples'])==9 and 'summary' not in doc
for bad in (rows+[rows[0]], [rows[1],rows[0]], [dict(rows[0],application_bytes=1)]+rows[1:]):
 try: v.validate_samples(bad,doc['contract'],False)
 except ValueError: pass
 else: raise AssertionError('invalid samples accepted')
# Diagnostic/multiline non-JSON output with nonzero status is typed, retained and valid.
diag=tmp/'diagnostic'; diag.write_text('first diagnostic\nsecond diagnostic\n')
good.write_text('Command exited with non-zero status 9\n{"sentinel":"nekomusume.gnu-time.v1","elapsed_seconds":.2,"cpu_user_seconds":0,"cpu_system_seconds":0,"rss_kib":1,"exit_code":9}\n')
resource=tmp/'resource'; resource.write_text(json.dumps({'experiment_id':'nekomusume-owned-lab-1','implementation':'nekomusume','role':'client','fd':{'peak_count':4},'sampling':{'scope':'sampler-created process group'},'cpu':{'user_seconds':.2,'system_seconds':.3},'rss':{'max_kib':9},'fd':{'peak_count':4},'exit':{'code':9,'timed_out':False},'cleanup':{'process_reaped':True,'process_group_empty':True,'owned_sockets_after_exit':0,'complete':True}}))
failed=v.make_sample('nekomusume',1,9,good,resource,diag,1200,h)
assert failed['failures']==1 and failed['exit_code']==9 and failed['application_bytes']==0 and failed['payload_sha256'] is None
failed_doc=dict(doc, samples=[failed], failure_stage='nekomusume-1-client')
failed_doc['cleanup_status']='failed'; failed_doc['cleanup_evidence']=dict(failed_doc['cleanup_evidence'], local_processes_reaped=False)
v.atomic_write(blocked,failed_doc); v.validate_result(blocked)
assert json.load(open(blocked))['status']=='BLOCKED_HARNESS'
# Pre-payload failure is valid only with explicit absent provenance.
pre=dict(doc); pre['contract']=dict(doc['contract'],payload_prepared=False,payload_sha256=None); pre['samples']=[]; pre['cleanup_status']='failed'; pre['cleanup_evidence']=dict(doc['cleanup_evidence'],remote_process_groups_reaped=None,remote_listeners_remaining=None,remote_temp_path_removed=None)
v.atomic_write(blocked,pre); v.validate_result(blocked)
for contract in (dict(pre['contract'],payload_sha256=h),dict(doc['contract'],payload_prepared=False)):
 bad=dict(pre,contract=contract); v.atomic_write(blocked,bad)
 try: v.validate_result(blocked)
 except ValueError: pass
 else: raise AssertionError('contradictory payload evidence accepted')
bad=dict(pre,cleanup_status='verified'); v.atomic_write(blocked,bad)
try: v.validate_result(blocked)
except ValueError: pass
else: raise AssertionError('unknown cleanup accepted as verified')
# Complete results require one process-group resource row for every fresh client sample.
complete_rows=[]
for i in range(1,6): complete_rows += [row('nekomusume',i),row('hy2',i)]
def client_resource(impl,run):
 return {'experiment_id':f'{impl}-owned-lab-{run}','implementation':impl,'identity':'sha256:'+('a'*64 if impl=='nekomusume' else 'b'*64),'role':'client','sampling':{'scope':'sampler-created process group'},'cpu':{'user_seconds':.2,'system_seconds':.3},'rss':{'max_kib':9},'fd':{'peak_count':4},'exit':{'code':9,'timed_out':False},'cleanup':{'process_reaped':True,'process_group_empty':True,'owned_sockets_after_exit':0,'complete':True}}
complete_contract={'runs_per_implementation':5,'payload_bytes':1200,'payload_prepared':True,'payload_sha256':h,'client_lifecycle':'fresh transport per timed sample','client_resource_scope':'sampler-created process group','enforced_global_deadline_ms':540000,'work_deadline_ms':480000,'cleanup_reserve_ms':60000,'whole_lab_deadline_ms':540000,'nekomusume_binary_sha256':'a'*64,'hy2_binary_sha256':'b'*64}
complete={'schema':'nekomusume.benchmark-result.v1','experiment_id':'hy2-owned-lab-paired','git_commit':'0'*40,'contract':complete_contract,'samples':complete_rows,'summary':v.expected_summary(complete_rows),'bounds':{'maximum_duration_ms':540000,'application_bytes_max':1200*5*2},'resources':[client_resource(impl,i) for impl in ('nekomusume','hy2') for i in range(1,6)],'cleanup_status':'verified','cleanup_evidence':doc['cleanup_evidence']}
result=tmp/'result.json'; v.atomic_write(result,complete); v.validate_result(result)
for bad in (dict(complete,resources=complete['resources'][:-1]),dict(complete,contract=dict(complete_contract,client_lifecycle='persistent'))):
 v.atomic_write(result,bad)
 try: v.validate_result(result)
 except ValueError: pass
 else: raise AssertionError('unfair lifecycle/resource result accepted')

# Complete result rejects missing process-group metrics and mismatched exit evidence.
for key in ('cpu', 'rss', 'fd'):
 bad_resource=client_resource('nekomusume',1); bad_resource.pop(key)
 bad=dict(complete,resources=[bad_resource if x is complete['resources'][0] else x for x in complete['resources']])
 v.atomic_write(result,bad)
 try: v.validate_result(result)
 except ValueError: pass
 else: raise AssertionError('missing resource metric accepted')
# make_sample attributes transport metrics, not GNU time wrapper values.
resource.write_text(json.dumps({'experiment_id':'nekomusume-owned-lab-1','implementation':'nekomusume','role':'client','identity':'sha256:7','cpu':{'user_seconds':7,'system_seconds':8},'rss':{'max_kib':99},'fd':{'peak_count':11},'exit':{'code':0,'timed_out':False},'sampling':{'scope':'sampler-created process group'},'cleanup':{'process_reaped':True,'process_group_empty':True,'owned_sockets_after_exit':0,'complete':True}}))
good.write_text('{"sentinel":"nekomusume.gnu-time.v1","elapsed_seconds":1,"cpu_user_seconds":0.1,"cpu_system_seconds":0.2,"rss_kib":1,"exit_code":0}\n')
out=v.make_sample('nekomusume',1,0,good,resource,diag,1200,h,'sha256:7')
assert (out['cpu_user_seconds'],out['cpu_system_seconds'],out['rss_kib'],out['fd_count'])==(7,8,99,11)

print('validate-hy2-owned-lab-test-ok')
