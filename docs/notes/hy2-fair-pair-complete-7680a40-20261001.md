# HY2 fair pair — changed-hypothesis rerun COMPLETE (2026-10-01)

The fair-pair comparison finally produced a **complete, validated pair** at
`7680a40`-tree scripts + `374a6daf` neko-cli: **10/10 samples, 0 failures,
both implementations completing the full 1200-byte bounded echo exchange** on
the real public path HK -> 23.147.120.24 (RTT ~213 ms).

## Environment (85461-approved protocol; amendment recorded in the
`hy2-hy2-1-client-exit-rootcause` note)

- Client side = HK (`43.154.97.90`), the measured path's origin; server side
  = `23.147.120.24`. The harness ran on HK with `LAB_SSH_TARGET=neko23`
  (expendable ed25519 key authorized for the run; see teardown).
- Ports: neko TCP 40097, hysteria UDP 40098, HK-local forwarding TCP 40099,
  remote loopback echo TCP 40100. ufw window: `40097/tcp` + `40098/udp`,
  comment `neko-hy2-exp-20261001`; before/after snapshots identical
  (`ufw_before.txt` / `ufw_after.txt`, diff clean), closed on every exit
  path including the two early harness failures.
- Pinned hysteria v2.9.3 re-fetched from upstream and SHA-256-verified
  (`66dbdb06…`) before adoption (the retired vps-104's copy is gone).
- Single changed-hypothesis invocation (third attempt overall: two early
  attempts failed in harness argument validation / final-result validation
  **before any sample was affected** — no VPS data re-collected after them;
  see the defect note below).

## Result (complete, validated offline after the validator fix)

| implementation | failures | median | p95 | app bytes |
|---|---|---|---|---|
| hy2 v2.9.3 | 0 | 1100 ms | 1190 ms | 6000 |
| nekomusume | 0 | 2150 ms | 2400 ms | 6000 |

10 samples (5 interleaved pairs), 16 resource records, cleanup verified.
Artifacts: `artifacts/hy2-owned-lab/7680a40-hy2-fair-pair-rerun/`
(`result.json.validated` + samples + ufw snapshots + diagnostic bundles).

**Interpretation boundary (deliberate):** this is a bounded same-workload
pair on one path at one moment — it says both implementations complete the
1200-byte echo exchange there, and gives medians under that workload. It is
not a general performance claim, and the earlier methodology note stands:
hy2's TCP forwarding does not tolerate the pre-read half-close posture, so
the workload is the 85461-decided no-half-close variant. No superiority
claim is made in either direction.

## The defect this run exposed (validator, latent since 09-03)

`validate-hy2-owned-lab.py`'s complete-result check required client resource
`exit == {"code":0,"timed_out":false}` — an exact dict that can never match
the real sampler output, which has emitted `{"code","signal","timed_out"}`
since its first revision (`b191dd8`, 09-01). The check was written 09-03 and
was **never exercised by a complete run** — every previous invocation ended
BLOCKED. The first complete run (this one) hit it and was spuriously
classified BLOCKED_HARNESS at `validation` stage despite 10/10 good samples
(all retained in `result.json.tmp` per the harness's own failure discipline).
Fix (gated): accept the real three-field shape, still reject nonzero code /
signalled / timed-out exits; test fixture updated to the real shape plus a
negative (a SIGTERM'd client must fail validate-result). The retained
complete result then validates cleanly offline — no VPS re-run needed.
