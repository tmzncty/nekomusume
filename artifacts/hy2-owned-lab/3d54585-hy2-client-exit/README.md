# Exact-3d54585 HY2 client-exit evidence

- Harness invocation count: exactly one, after exact `3d54585` had green stable and nightly fuzz CI.
- Payload: prepared (1200 bytes); the validator-valid result records its SHA-256.
- Valid sample prefix: two records in order — `nekomusume-1` succeeded, then `hy2-1` failed with `client_exit`.
- Outcome: `BLOCKED_HARNESS` at `hy2-1-failed`.
- Complete pairs/comparative summary: none. The prefix must not be used for a performance comparison.
- Result artifact: `result.json`, SHA-256 `dc7d4a0887ebc5617dbc34b5146563af7178445ea2ba05d30da05276f4558602`.
- Sample companion: `result.json.samples.jsonl`, SHA-256 `83fcf3d2de64dfe83773e086cb9168686973ab36e3b668755f9449f0bc466826`.
- Artifact-recorded automatic cleanup failed solely because `remote_process_groups_reaped=false`; it records remote/local listeners remaining `0`, remote temporary-path removal `true`, and local processes reaped `true`.
- Separate later serialized double-end postchecks found no experiment ports, processes, or temporary paths. This later observation does not rewrite the artifact's failed cleanup result.

The retained files contain no endpoint address, credentials, secret material, or private topology. They establish a typed negative result only, not comparative performance.
