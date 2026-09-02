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
blocked=tmp/'blocked.json'; doc={'schema':'nekomusume.benchmark-blocked-harness.v1','experiment_id':'hy2-owned-lab-paired','git_commit':'0'*40,'status':'BLOCKED_HARNESS','failure_stage':'hy2-5-client','contract':{'runs_per_implementation':5,'payload_bytes':1200,'payload_sha256':h},'samples':v.load_jsonl(records),'cleanup_status':'verified','cleanup_evidence':{'local_processes_reaped':True,'local_listeners_remaining':0,'remote_process_groups_reaped':True,'remote_listeners_remaining':0,'remote_temp_path_removed':True}}
v.atomic_write(blocked,doc); v.validate_result(blocked); assert len(json.load(open(blocked))['samples'])==9 and 'summary' not in doc
for bad in (rows+[rows[0]], [rows[1],rows[0]], [dict(rows[0],application_bytes=1)]+rows[1:]):
 try: v.validate_samples(bad,doc['contract'],False)
 except ValueError: pass
 else: raise AssertionError('invalid samples accepted')
# Diagnostic/multiline non-JSON output with nonzero status is typed, retained and valid.
diag=tmp/'diagnostic'; diag.write_text('first diagnostic\nsecond diagnostic\n')
good.write_text('Command exited with non-zero status 9\n{"sentinel":"nekomusume.gnu-time.v1","elapsed_seconds":.2,"cpu_user_seconds":0,"cpu_system_seconds":0,"rss_kib":1,"exit_code":9}\n')
failed=v.make_sample('nekomusume',1,9,good,diag,1200,h)
assert failed['failures']==1 and failed['exit_code']==9 and failed['application_bytes']==0 and failed['payload_sha256'] is None
failed_doc=dict(doc, samples=[failed], failure_stage='nekomusume-1-client')
failed_doc['cleanup_status']='failed'; failed_doc['cleanup_evidence']=dict(failed_doc['cleanup_evidence'], local_processes_reaped=False)
v.atomic_write(blocked,failed_doc); v.validate_result(blocked)
assert json.load(open(blocked))['status']=='BLOCKED_HARNESS'
print('validate-hy2-owned-lab-test-ok')
