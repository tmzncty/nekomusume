# Bounded remote experiment runner

`scripts/remote/run-experiment.sh` is a research-only, Linux runner. It is
**dry-run by default** and has an explicit `--execute` escape hatch for the
same host. Execution is strictly loopback TCP (`127.0.0.1`, ports 40080–40100),
with a maximum of eight exchanges and 30 seconds. It never accepts a remote
address, opens a public listener, changes routes/firewalls/tunnels, or contacts
the public WAN.

Each invocation creates a unique constrained `experiment_id` and records the
ordered phases: `prepare`, `deploy`, `start`, `verify`, `run`, `capture`,
`stop`, `collect`, `cleanup`, and `verify-clean`. Capture is metadata-only;
keys, payloads, environment variables, and packet captures are not recorded.
Identity files and PID files are removed before the final cleanup verification.
Artifacts are mode 0700/0600 and should be treated as disposable evidence.

```sh
./scripts/remote/run-experiment.sh --dry-run
./scripts/remote/run-experiment-test.sh
```
