# P0 preauth admission-reject fix — gate provenance

**Scope:** F1 from `docs/reviews/review-2026-10-07-lane-a-core.md` /
T0-1 + T0-2 from `docs/reviews/review-2026-10-07-lane-c-tests.md`.

**Defect:** the three TCP server entrypoints (`neko server --transport tcp`
main.rs, `periodic-server` periodic.rs, `multistream --mode server`
multistream.rs) exited the whole process (`fail()`, exit 2) whenever a
pre-auth admission rejection or any single-connection pre-auth handshake
failure occurred. An unauthenticated remote could terminate the listener
with one connection (or, in the engine's accounting view, the 9th
same-source state). The `neko server` UDP path had the same shape for
admit/charge/hello rejections.

**Fix shape (reject + count + continue, mirroring failover_server's
non-pending UDP path at old main.rs:1984-1993):**

- `server` TCP (main.rs): admission reject, staged-frame read failure,
  `server_accept_hello` failure, response charge/send failure, and
  `receive_first` unauthorized handshake each release the admission, emit
  the `malformed_or_unadmitted` classification diagnostic, close the
  connection, and `continue` the accept loop. Only a fully authenticated
  exchange proceeds to the data phase and the normal STOPPED exit.
- `server` UDP (main.rs): admit reject, input-charge reject, and
  `server_accept_hello` reject emit `malformed_or_unadmitted` + continue.
- `periodic-server` (periodic.rs): `handshake_server` returns
  `Result<SecureSession, &'static str>`; the accept loop retries until an
  authenticated Session within the duration window. Rejections print a
  `periodic_server_rejected` line (stdout, human contract line, not JSON)
  and close the connection. No diagnostic-mode flag exists for the
  periodic fixture, so the rejection count line is the observable oracle.
- `multistream` (multistream.rs): `server_handshake` returns
  `Result`; the accept loop retries until authentication. Rejections go to
  stderr (`neko: ... (connection closed)`) to preserve the single-line JSON
  stdout contract.

**Preserved fail() semantics (local invariants, not remote-triggerable):**
`negotiation setup failed` (constant table), `negotiation binding failed`,
`handshake setup failed` (local crypto init), `data admission denied`
(post-auth state machine invariant), `bad data` / `auth failure` /
`seal failure` (post-authenticated exchange), `accept failed`, bind/setup
failures. Post-authentication single-peer divergence in the `server` TCP
data phase still exits — that is the documented single-exchange server
contract; connection-level resilience there is part of the later handshake
orchestration slice (review item 6), not this fix.

**Tests (red → green):**

- Red run on the unfixed tree (`git stash` of the fix,
  `cargo test -p neko-cli --test preauth_rejection`):
  `3 failed; 1 passed` — each TCP test died at or before the first abusive
  connection (`server died at abusive connection 0` / `Connection refused`
  after the listener died), the failover UDP regression passed.
- Green run on the fixed tree: `4 passed; 0 failed`.
- `crates/neko-cli/tests/preauth_rejection.rs`:
  - `server_tcp_survives_preauth_rejections_then_serves_client`
  - `periodic_server_survives_preauth_rejections_then_serves_client`
  - `multistream_server_survives_preauth_rejections_then_serves_client`
  - `failover_server_udp_admission_rejections_are_counted_and_survived`
    (T0-2 regression lock: two fresh-source malformed datagrams each emit
    `malformed_or_unadmitted` and the listener survives).
- Two existing multistream tests asserted the old exit-2 bug behavior and
  were updated to the new contract (survive + legitimate client succeeds):
  - `unauthorized_client_is_rejected_by_allowlist`
  - `executable_rejects_unsupported_only_negotiation_before_noise_or_data`
- Full `probe.rs` suite: 78 passed / 0 failed on the fixed tree.
- Full `multistream.rs` suite: 7 passed / 0 failed on the fixed tree.

**Gate:** `bash scripts/check.sh` on the committed tree — see the result
table in the commit message / docs/status.md note. Log retained at
`/tmp/neko-check-p0.log`.

**Not changed:** no policy numbers (limits stay `ProcessPreauthLimits::
default()`), no governance flags (RELEASE/PRODUCTION/FREEZE/RELEASED
unchanged), no architecture changes (handshake-orchestration extraction
remains review item 6). Loopback-only testing; no VPS/WAN/sudo.

**Environment note:** the repository had an unpushed sibling commit
`0c23aa2` (`feat(tools): impair-proxy`, from a parallel session) plus its
uncommitted `tools/impair-proxy/tests/e2e.rs` follow-up in the worktree.
This fix's commit contains only the files listed below; the e2e.rs
working-tree change stays uncommitted and untouched.

**Files in the fix commit:**

- `crates/neko-cli/src/main.rs`
- `crates/neko-cli/src/periodic.rs`
- `crates/neko-cli/src/multistream.rs`
- `crates/neko-cli/tests/preauth_rejection.rs` (new)
- `crates/neko-cli/tests/multistream.rs` (two contract updates)
- `docs/provenance/p0-preauth-admission-fix-20261007.md` (this file; the exact fixing SHA is recorded in the follow-up gate note)
- `docs/status.md` (RSEC-001 factual note)
