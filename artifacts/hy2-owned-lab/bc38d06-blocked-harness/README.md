# Exact-bc38d06 HY2 blocked-harness evidence

- Harness invocation count: exactly one.
- SSH preflight: succeeded.
- Payload: prepared (1200 bytes); the validator-valid result records its SHA-256.
- Samples: zero. No paired statistics or comparison exists.
- Outcome: `BLOCKED_HARNESS` during setup because `run_client` expanded `impl` before assignment under `set -u` at the then-current line 186.
- Result artifact: `result.json`, SHA-256 `596ad4b73058143db1918613dd970e44e8e6bf3a1b89602ac0012f911b6d2653`.
- Artifact-recorded cleanup failed: remote listeners remaining `1`, remote process groups reaped `false`, and remote temp-path removal not observed (`null`).
- Separate manual post-run cleanup subsequently verified no experiment ports, processes, or temporary paths remained. This later observation does not rewrite the artifact's failed cleanup result.

The artifact contains no endpoint address, credentials, secret material, comparative summary, or performance conclusion.
