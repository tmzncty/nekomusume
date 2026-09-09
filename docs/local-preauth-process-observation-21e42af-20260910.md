# Local bounded pre-auth process observation — `21e42af`

Developer-run exact-tree validation of reachable implementation/test commit `21e42af49a9bbe639aefa3dca887e17b03cad07c`.

## Workload and result

- deterministic model coverage exercises the unchanged `ProcessPreauthLimits::default()` candidate boundaries: 8 same-source states, 1,024 global states, 256 queued states, and 2,048 response bytes / 4 response packets; each first excess operation rejects without exceeding accounting, and an abandoned fourth response remains charged
- process observation launches one long-lived loopback failover server, sends exactly 8 malformed five-byte UDP negotiation datagrams, waits 100 ms, then completes one authenticated 16-byte / one-record failover exchange through the same process
- this exact tree created one fresh UDP socket per attempt but did not retain all sockets or assert port uniqueness; exact `976f90b` supersedes the source-domain claim by pre-binding and retaining eight sockets and asserting eight unique source ports before sending. Neither tree is same-source saturation.
- malformed attempts produce no successful authentication, Delivery, PathValidated or ACK evidence; the later valid exchange emits the expected bounded success
- Linux `/proc` snapshots, when available, assert file-descriptor growth no greater than one and RSS growth no greater than 4,096 KiB across the malformed workload; these are regression bounds, not capacity recommendations
- after process exit, both leased UDP and TCP listener ports rebind successfully; this exact tree attempted temporary identity removal without asserting it, while exact `976f90b` adds asserted removal and absence checks

## Exact-tree gate

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- gate: `2026-09-09T19:15:18Z` -> `2026-09-09T19:17:10Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

This is bounded deterministic/model and local process recovery evidence for the exact workload and candidate values exercised. It is not a stress benchmark, maximum-capacity result, default-limit production-suitability finding, D019 resolution, independent security review, public-listener approval, RC, release or production authorization.
