#!/usr/bin/env python3
from __future__ import annotations
import importlib.util, json, os, pathlib, subprocess, sys, tempfile

SCRIPT = pathlib.Path(__file__).with_name("run-repeated-warm-failover-command.py")
spec = importlib.util.spec_from_file_location("command", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)

def rejected(value, text="invalid"):
    with tempfile.TemporaryDirectory() as td:
        path = pathlib.Path(td) / "plan.json"
        path.write_text(value if isinstance(value, str) else json.dumps(value))
        try: module.load_plan(str(path))
        except module.PlanError as error: assert text in str(error), error
        else: raise AssertionError("invalid plan accepted")

def test_malformed_and_missing_json_fail_closed():
    rejected("{")
    rejected({}, "exactly")
    plan = module.fake_plan(); plan["cycles"][0]["endpoints"][0]["argv"] = "env failover-server,"
    rejected(plan, "endpoint argv")
    plan = module.fake_plan(); plan["cycles"][0]["endpoints"][1]["argv"] = ["ok", ""]
    rejected(plan, "endpoint argv")

def test_shell_boundary_strings_remain_single_argv_entries():
    plan = module.fake_plan(); loaded = module.load_plan(write(plan))
    env = module.adapter_env(loaded, 1)
    server = json.loads(env["NEKO_FAILOVER_ENDPOINTS_JSON"])[0]["argv"]
    assert server == ["/harmless/fake binary", "failover-server", "failover-server,", "--label", "comma,quote\"kept"]
    assert "failover-server," in server and server.index("failover-server,") == 2

def write(plan):
    path = pathlib.Path(tempfile.mkdtemp()) / "plan.json"
    path.write_text(json.dumps(plan)); return str(path)

def test_preflight_enters_runner_and_dispatches_six_without_shell():
    with tempfile.TemporaryDirectory() as td:
        output = pathlib.Path(td) / "preflight.json"
        done = subprocess.run([sys.executable, str(SCRIPT), "--preflight", "--output", str(output)], capture_output=True, text=True)
        assert done.returncode == 0, done.stderr
        report = json.loads(output.read_text())
        assert report["dry_run"] is True and report["live_evidence"] is False
        assert report["runner_entered"] is True and report["completed_dispatches"] == 6
        assert len(report["dispatches"]) == 6
        for index, item in enumerate(report["dispatches"], 1):
            assert item["cycle_index"] == index
            assert item["adapter_argv"] == [sys.executable, str(SCRIPT.with_name("run-live-warm-failover-cycle.py"))]
            assert json.loads(item["environment"]["NEKO_FAILOVER_ENDPOINTS_JSON"])[0]["argv"][2] == "failover-server,"

def test_dispatch_uses_argv_and_exact_environment():
    plan = module.fake_plan(); path = write(plan)
    seen = {}
    original = module.subprocess.run
    def fake(command, **kwargs):
        seen["command"] = command; seen["env"] = kwargs["env"]
        return subprocess.CompletedProcess(command, 0)
    module.subprocess.run = fake
    old = os.environ.get("NEKO_FAILOVER_CYCLE_INDEX"); os.environ["NEKO_FAILOVER_CYCLE_INDEX"] = "6"
    try: assert module.dispatch(path, False) == 0
    finally:
        module.subprocess.run = original
        if old is None: os.environ.pop("NEKO_FAILOVER_CYCLE_INDEX", None)
        else: os.environ["NEKO_FAILOVER_CYCLE_INDEX"] = old
    assert seen["command"] == [sys.executable, str(SCRIPT.with_name("run-live-warm-failover-cycle.py"))]
    assert seen["env"]["NEKO_FAILOVER_GIT_COMMIT"] == "0" * 40
    assert seen["env"]["NEKO_FAILOVER_UDP_PORT"] == "40091"
    endpoints = json.loads(seen["env"]["NEKO_FAILOVER_ENDPOINTS_JSON"])
    assert endpoints[1]["argv"][3] == "client,'quoted'"
    assert [endpoint["role"] for endpoint in endpoints] == ["server", "client"]

def main():
    for name, value in sorted(globals().items()):
        if name.startswith("test_"): value()
if __name__ == "__main__": main()

def test_structured_ssh_descriptor_survives_without_shell_reparse():
    plan = module.fake_plan(); endpoint = plan["cycles"][0]["endpoints"][0]
    endpoint["execution"] = "ssh"; endpoint["transport_argv"] = [sys.executable, "transport helper,quoted"]
    endpoint["ssh_executable"] = sys.executable
    loaded = module.load_plan(write(plan)); env = module.adapter_env(loaded, 1)
    actual = json.loads(env["NEKO_FAILOVER_ENDPOINTS_JSON"])[0]
    assert actual["execution"] == "ssh" and actual["transport_argv"][1] == "transport helper,quoted"
