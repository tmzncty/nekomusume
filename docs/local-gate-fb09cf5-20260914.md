# Developer-local exact-tree provenance — `fb09cf5` R9-2 demux/ownership repair

Exact pushed source SHA: `fb09cf59e81782a5b2ecf12466bf47f72a9e51d6` (`fix(cli): R9-2 multi-record demux + reserved-record ownership`).

Developer-local clean exact-tree gate. Not a WAN/live claim, not an independent security/release approval.

## Gate (clean detached worktree of exact SHA)

```bash
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh   # exit 0
git diff --check                                  # exit 0
git status --short                                # empty before/after
```

- UTC start `2026-09-14T00:01:12Z`, UTC end `2026-09-14T00:03:11Z`.
- Host `Linux 6.8.0-137-generic x86_64 GNU/Linux`; `rustc 1.98.0`.

## Repairs

- H-R9-005: under `--reliable-udp` + migration-back the reserved final record
  is no longer consumed early — `uncertain_start=2` but both the track loop
  and the direct uncertain send are bounded by `uncertain_end`; the reserved
  index is never tracked or wire-sent before post-promotion return-to-UDP.
- M-R9-004: the second reliable-owned record receives its own independent
  Session `DeliveryAck` (`recv_udp_delivery_ack` demux per record); test pins
  offset-16 `r9_udp_delivery_ack_validated` and uncertain send at offset 32.

`READY_LIVE: none` unchanged.
