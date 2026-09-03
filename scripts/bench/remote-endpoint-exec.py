#!/usr/bin/env python3
"""Bounded SSH-side verifier/spawner for one Nekomusume evidence endpoint."""
from __future__ import annotations
import hashlib, json, os, pathlib, signal, subprocess, sys
from typing import Any, NoReturn
MAX_REQUEST=256*1024; MAX_ARGV=64
child: subprocess.Popen[bytes] | None = None

def fail(message: str) -> NoReturn:
 print(f"remote endpoint rejected: {message}",file=sys.stderr); raise SystemExit(2)
def exact_int(value: Any) -> bool: return isinstance(value,int) and not isinstance(value,bool)
def relay(signum: int, _frame: Any) -> None:
 if child is not None and child.poll() is None:
  try: os.killpg(child.pid,signum)
  except ProcessLookupError: pass

def main() -> int:
 global child
 raw=sys.stdin.buffer.read(MAX_REQUEST+1)
 if not raw or len(raw)>MAX_REQUEST: fail("invalid request size")
 try: request=json.loads(raw)
 except json.JSONDecodeError: fail("invalid JSON")
 if not isinstance(request,dict) or set(request)!={"protocol","role","binary","argv"}: fail("invalid request shape")
 if request["protocol"]!="nekomusume.remote-exec.v1" or request["role"] not in ("server","client"): fail("invalid protocol or role")
 binary=request["binary"]; argv=request["argv"]
 if (not isinstance(binary,dict) or set(binary)!={"path","sha256","bytes","git_commit"} or
     not isinstance(binary["path"],str) or not binary["path"] or "\0" in binary["path"] or
     not isinstance(binary["sha256"],str) or len(binary["sha256"])!=64 or
     not exact_int(binary["bytes"]) or binary["bytes"]<=0 or
     not isinstance(binary["git_commit"],str) or len(binary["git_commit"])!=40 or
     not isinstance(argv,list) or not 1<=len(argv)<=MAX_ARGV or
     any(not isinstance(arg,str) or not arg or "\0" in arg or len(arg)>4096 for arg in argv)): fail("invalid binary or argv")
 path=pathlib.Path(binary["path"])
 if argv[0]!=binary["path"] or not path.is_absolute(): fail("argv executable differs from underlying binary")
 try:
  with path.open("rb") as stream:
   stat=os.fstat(stream.fileno()); digest=hashlib.file_digest(stream,"sha256").hexdigest(); post=os.fstat(stream.fileno())
 except OSError: fail("underlying binary unavailable")
 if stat.st_size!=binary["bytes"] or post.st_size!=stat.st_size or digest!=binary["sha256"]: fail("underlying binary identity mismatch")
 for signum in (signal.SIGTERM,signal.SIGINT): signal.signal(signum,relay)
 try:
  child=subprocess.Popen(argv,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,start_new_session=True,shell=False)
  assert child.stdout is not None
  while True:
   chunk=child.stdout.read(65536)
   if not chunk: break
   sys.stdout.buffer.write(chunk); sys.stdout.buffer.flush()
  return child.wait()
 except OSError: fail("spawn failed")
if __name__=="__main__": raise SystemExit(main())
