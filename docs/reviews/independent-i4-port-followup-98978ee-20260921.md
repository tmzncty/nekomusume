# Independent I4-PORT follow-up — process-test platform scope

**Reviewed anchor:** reachable `main` at `98978eec7ed02a46d1f7577334349eba3f630db3`.

**Developer review inputs:**
- `5f24cbffcbe13386e4016059f8a8253a5fd1a434` — UDP adapter bounded re-challenge;
- `e8fbc65e1ccd8766e000d48054411d53fcf94555` — TCP adapter bounded re-challenge;
- `98978eec7ed02a46d1f7577334349eba3f630db3` — I4-PORT cross-platform CLI/process-test re-challenge.

This is a source/test review only. It makes **no reviewer-local CI, hosted-CI, live-WAN, or performance claim**.

## Result

The UDP and TCP adapter no-finding notes are accepted within their stated D012/current-Carrier scopes. I found no new source-decided adapter defect in those commits.

The I4-PORT no-finding needs one narrow qualification before it can serve as a complete cross-platform **process-test** closure. `crates/neko-cli/tests/probe.rs` defines `signal_term()` by executing the external program `kill -TERM <pid>`, and both `sigterm_after_ready_stops_and_releases_tcp_and_udp_bindings` and `periodic_server_signal_cleanup_is_bounded` call that helper without a Unix/Linux cfg guard. The same test owner also contains Linux `/proc` process-resource observations. Those are legitimate Linux release-evidence fixtures, but an unguarded dependency on the Unix `kill` executable is not a portable process-test contract.

This does **not** establish a production/runtime correctness HIGH and does not change the first-RC Linux target. It also does not contradict the narrower claim that the Rust CLI source can compile on a non-Unix target. The issue is test-scope/evidence truth: the current note title/result must not be read as proving non-Unix execution of Linux/POSIX lifecycle fixtures.

## READY_LOCAL repair / closure contract

Use the smallest current-contract repair:

1. make the SIGTERM/process-lifecycle fixtures explicitly Unix/Linux-scoped (for example with the narrowest applicable cfg on the tests/helper), **or** use an already-supported platform-aware signaling mechanism if the repository intentionally claims those lifecycle tests cross-platform;
2. keep Linux `/proc`, process-group, `setsid`, `kill`, and socket-release evidence explicitly labelled Linux/POSIX evidence rather than protocol portability evidence;
3. preserve the current Linux behavior and deterministic cleanup assertions; do not broaden the release target or add a new portability policy;
4. if executable test code changes, run the repository-required final pushed-SHA clean exact-tree gate and record provenance. No decoder/framing change is involved, so no mechanical fuzz run is required.

After that narrow repair/qualification, continue directly to I4-CLI-M rather than waiting for another reviewer cadence.

**READY_LIVE: none.** No new real-network question was created by this review.
