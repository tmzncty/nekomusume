# Bounded Linux process-resource sampler

`scripts/bench/process-resource-sampler.py` runs one caller-supplied command as a
direct child and writes one `nekomusume.process-resource.v1` JSON document. It
is intended for small, controlled Nekomusume/HY2 experiments—not general host
monitoring or capacity testing.

## Contract

The caller must supply the experiment ID, implementation, process role,
git-or-binary identity, workload application-byte count, output path, and any
owned experimental local ports. Metadata is constrained and the command is
bounded to 600 seconds, 256 MiB of declared application bytes, 32 unprivileged
ports, and 64 arguments. Output must be a new absolute path.

The sampler records UTC start/end, monotonic elapsed time, exit/signal/timeout,
child CPU user/system time (`wait4`), maximum sampled/current RSS combined with
Linux `ru_maxrss` (KiB), peak sampled FD count, and peak process-owned
experimental listener/socket count. Socket counting intersects only the direct
child's socket FDs with caller-supplied ports in `/proc/net`; it emits counts,
not addresses, peer details, environment, payload, or secrets. TCP counts only
LISTEN sockets. UDP counts a bound socket on an owned port because Linux UDP has
no LISTEN state.

Resource sampling is deliberately limited to the direct child. Cleanup is
scoped more broadly but still safely: the sampler creates one process group,
adopts orphaned descendants, and verifies that group is empty after normal,
signal, and timeout exits. Containers, non-Linux systems, kernel/network wire
bytes, and host-wide resources are not claimed. Metrics unavailable because `/proc` or a process sample raced with
exit are JSON `null`, never invented zero. `application_bytes` is caller-owned
workload metadata, not inferred traffic. Cleanup completion means the direct child was reaped and the sampler-created
process group was observed empty. `owned_sockets_after_exit=0` is emitted only
after that direct post-exit observation; it does not certify unrelated processes
or files.

## Example

```bash
out=$(mktemp -d)
python3 scripts/bench/process-resource-sampler.py \
  --experiment-id followup-b-smoke-01 \
  --implementation nekomusume \
  --role server \
  --identity git:0123456789abcdef \
  --application-bytes 1024 \
  --owned-port 40080 \
  --interval-ms 25 \
  --max-seconds 10 \
  --output "$out/server.json" \
  -- target/debug/neko server ...
python3 scripts/bench/validate-process-resource.py "$out/server.json"
```

The sampler returns the child's exit status (or `128 + signal`) after writing
the record. Normal exit, timeout, SIGTERM, and SIGINT all terminate remaining same-group
descendants if needed and prove the group empty. A timeout sends `SIGTERM`, waits
one second, then uses `SIGKILL`.
Callers needing server/client orchestration should start a bounded server under
one sampler and separately sample the client; cleanup traps remain the
orchestrator's responsibility.

## Validation and tests

- JSON Schema: `scripts/bench/process-resource.schema.json`
- dependency-free fail-closed validator:
  `scripts/bench/validate-process-resource.py SAMPLE.json`
- deterministic regression:
  `python3 scripts/bench/process-resource-sampler-test.py`

The regression uses a harmless local child with five known extra FDs and one
owned TCP listener, checks a non-zero process exit, covers an immediate-exit
race, and rejects malformed metadata/schema/bounds. It requires Linux `/proc`
and Python 3. No root privilege is needed.
