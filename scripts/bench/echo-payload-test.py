#!/usr/bin/env python3
import json, pathlib, socket, subprocess, sys, tempfile, threading
ROOT=pathlib.Path(__file__).resolve().parents[2]
TOOL=ROOT/"scripts/bench/echo-payload.py"
with tempfile.TemporaryDirectory() as td:
    payload=pathlib.Path(td)/"payload"; payload.write_bytes(b"equal-application-payload")
    listener=socket.socket(); listener.bind(("127.0.0.1",0)); listener.listen(1)
    port=listener.getsockname()[1]
    def echo():
        conn,_=listener.accept()
        with conn:
            while data:=conn.recv(4096): conn.sendall(data)
        listener.close()
    thread=threading.Thread(target=echo); thread.start()
    out=subprocess.run([sys.executable,str(TOOL),"--host","127.0.0.1","--port",str(port),"--payload-file",str(payload),"--timeout","2"],text=True,capture_output=True,check=True)
    thread.join()
    d=json.loads(out.stdout)
    assert d["application_bytes"]==25 and d["payload_sha256"]=="cf241de87cf4e86eca5350ac13106043592ab1a8bceb97851833d90440b52cef"
    assert isinstance(d["fd_count"],int) and d["wire_bytes"] is None
    bad=subprocess.run([sys.executable,str(TOOL),"--host","127.0.0.1","--port","80","--payload-file",str(payload)],capture_output=True)
    assert bad.returncode != 0
print("echo payload tests: PASS")
