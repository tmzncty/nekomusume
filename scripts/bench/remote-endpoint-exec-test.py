#!/usr/bin/env python3
from __future__ import annotations
import hashlib, json, pathlib, subprocess, sys, tempfile
HERE=pathlib.Path(__file__).resolve().parent; HELPER=HERE/"remote-endpoint-exec.py"
def request(path: str, sha: str | None=None, size: int | None=None, argv: list[str] | None=None):
 data=pathlib.Path(path).read_bytes()
 return {"protocol":"nekomusume.remote-exec.v1","role":"server","binary":{"path":path,"sha256":sha or hashlib.sha256(data).hexdigest(),"bytes":size or len(data),"git_commit":"a"*40},"argv":argv or [path,"-c","print('REMOTE_OK')"]}
def run(value):
 return subprocess.run([sys.executable,str(HELPER)],input=json.dumps(value),text=True,capture_output=True,timeout=10)
r=run(request(sys.executable)); assert r.returncode==0 and r.stdout.splitlines()==["remote_exec_protocol_accepted protocol=nekomusume.remote-exec.v1 role=server","REMOTE_OK"],r
r=run(request(sys.executable,sha="0"*64)); assert r.returncode==2 and r.stdout=="" and "identity mismatch" in r.stderr
r=run(request(sys.executable,size=pathlib.Path(sys.executable).stat().st_size+1)); assert r.returncode==2 and r.stdout=="" and "identity mismatch" in r.stderr
with tempfile.TemporaryDirectory() as td:
 wrapper=pathlib.Path(td)/"wrapper"; wrapper.write_text("#!/bin/sh\nexit 99\n"); wrapper.chmod(0o700)
 r=run(request(sys.executable,argv=[str(wrapper),"-c","print('BAD')"])); assert r.returncode==2 and r.stdout=="" and "argv executable differs" in r.stderr
source=HELPER.read_text(); assert "shell=False" in source and "shell=True" not in source and "os.system" not in source
print("remote endpoint verifier tests: ok")
