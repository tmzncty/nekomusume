# HY2 `hy2-1` client_exit — root cause found and diagnostic seam added (2026-09-30)

The 2026-09-08 VPS-owned-lab run's `hy2-1` sample (`category=unknown`,
`last_success_stage=client_started`, exit 1, 0 application bytes, 0.29 s) is now
fully attributed with local evidence only. **No VPS call was consumed.**

## Root cause (two stacked defects, both reproduced locally)

**Defect 1 — transport: hysteria v2.9.3 client TCP forwarding does not support
half-close.** The echo client (`scripts/bench/echo-payload.py`) sends the
payload then calls `shutdown(SHUT_WR)` before reading the echo — correct
bounded-exchange practice. Upstream
`app/internal/forwarding/tcp.go` (tag `app/v2.9.3`, commit `2d973f9`) runs two
`io.Copy` goroutines and returns from `handle()` when **either** finishes
(`closeErr = <-copyErrChan` once), then closes both directions via `defer`.
When the local half-close arrives, `io.Copy(rc, conn)` returns EOF, `handle()`
exits, and `rc.Close()` tears down the tunnel before the return-path data can
reach the local socket. Local minimal topology (loopback hysteria server +
client, `tcpForwarding` 40099→40100, echo server on 40100, hysteria v2.9.3
binary pinned by the harness, sha `66dbdb06…`):

- send 1200 B then immediate `shutdown(SHUT_WR)` → **0 bytes echoed**, exit 1
  (the VPS signature);
- same but wait 3 s before the half-close (echo already delivered) → **1200
  bytes** (race eliminated: it is not a generic data-vs-FIN race, the loss
  happens only when the FIN precedes the return-path data through the
  forwarding layer);
- no half-close at all → **1200 bytes** (both orderings).

**Defect 2 — harness: the echo client's stderr never reached the diagnostic
bundle.** The 699-byte private bundle for `hy2-1` contained only the hysteria
client log (connected/TCP forwarding listening/graceful shutdown — all
healthy), while the actual failure reason (`exact payload mismatch`) landed in
`client.err`, which the harness overwrote per run and discarded. Hence
`category=unknown` despite the evidence existing.

Incidental observation (out of scope, recorded for completeness): the hysteria
client startup performs an update check against `https://api.hy2.io/v1/update`;
on this host the DNS answer for `api.hy2.io` is a Meta fake-IP (`198.18.0.4`),
so the check is routed through the Meta TUN. It did not interact with the
reproduced failure (the echo exchange fails identically with the check present
or absent from the picture), but any future VPS run that measures egress should
account for it.

## The seam (this slice)

1. `scripts/bench/echo-payload.py` — failure paths now emit classified
   phrases: `payload exchange failed: <OSError>` (transport errors keep their
   `path` classification), `payload exchange truncated: connection closed after
   N of M bytes`, `payload exchange mismatch: echoed N of M bytes with
   different content`. The literal `echo-payload` prefix is deliberately absent
   because its hyphen makes `payload` a `\b`-word that the `path` category
   pattern would capture first (verified).
2. `scripts/bench/compare-hy2-owned-lab.sh` `run_client` — for hy2 runs the
   diagnostics file is now the transport log **plus** the echo client stderr
   (marker `--- echo-client stderr ---`), so the bundle carries the exchange
   failure reason.
3. `scripts/bench/validate-hy2-owned-lab.py` — new `payload` diagnostic
   category (`\bpayload exchange\b`, ordered last so subsystem evidence keeps
   precedence).

## Verification

- `python3 scripts/bench/echo-payload-test.py` PASS.
- `bash scripts/bench/compare-hy2-owned-lab-test.sh` PASS, including the two
  new `'payload:...'` fixtures in the category loop (both single-sample and
  blocked-pair paths).
- End-to-end against the reproduced topology: the same minimal loopback
  failure that produced `category=unknown` now produces
  `failure_stage=client_exit`, `category=payload`, with the truncated-exchange
  phrase retained in the sanitized bundle tail. Bundle bounds, sanitization
  (no IP/host leaks) and file mode 600 unchanged (covered by existing test
  pins).

## Rerun plan amendment (2026-10-01, 85461-approved; environment per Session 26434)

- **Endpoint**: server = `23.147.120.24` (Virginia, AS54286; the PLPMTUD-run
  machine, cleanup-verified). Client-side = **HK** (`43.154.97.90`): the
  workstation's direct route to 23 traverses the Meta TUN, which would
  invalidate the measurement; HK->23 is the real public path (RTT ~213 ms).
  The harness therefore runs on HK with `LAB_SSH_TARGET` reaching 23 through
  the workstation's jump (`ProxyJump tmzn-hk` in the ssh config used there).
- **Ports** (distinct from the PLPMTUD run's single UDP port): neko TCP
  `40097`, hysteria UDP `40098`, local TCP forwarding `40099`, remote
  loopback TCP echo `40100` (loopback-only, not firewalled). The ufw window
  must cover **40097/tcp and 40098/udp** on 23: two rules
  (`ufw allow 40097/tcp` + `ufw allow 40098/udp`), comment
  `neko-hy2-exp-20261001`, opened after the before-snapshot, closed with
  delete + diff before/after identical, on failure too.
- **Pinned hysteria artifact**: `/usr/local/bin/hysteria` on the retired
  vps-104 is gone with the machine. v2.9.3 re-fetched from the upstream
  release (`apernet/hysteria` `app/v2.9.3`, `hysteria-linux-amd64`) on HK;
  SHA-256 verified equal to the pin `66dbdb06…` before adoption.
- **Estimated window**: `RUNS=5`, `BENCH_TIMEOUT_SEC=30` (defaults) ->
  work ~7 min, whole-lab ~8 min; plus setup/teardown the ufw window is
  **~15 minutes maximum**, revoked immediately after.
- **HK runtime**: repo scripts via `git archive` (no .git on HK; the result
  records `git_commit` from the harness invocation — the source tree is
  `ba1df72`-equivalent), neko-cli from the gated `7680a40` build
  (`374a6daf…`), hysteria pinned above. Result + samples land under
  `~/neko-hy2-lab/artifacts/` on HK and are copied back to the workstation
  evidence tree.

## Consequences

- The fair-pair comparison methodology defect is understood: the pair is not
  measuring hy2 transport performance on this workload, because the workload
  (half-close echo) is unsupported by hy2's client forwarding. Any future
  owned-lab rerun must either drop the half-close from the echo client or
  accept that hy2 samples fail with `category=payload` by design; this is a
  methodology decision for 85461/the administrator, not something to change
  unilaterally.
- The VPS evidence file and `AGENTS.md` status line remain to be updated when
  the mainline-2 slice is reviewed.
