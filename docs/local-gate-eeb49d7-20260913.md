# Developer-local exact-tree provenance — `eeb49d7` repaired R7 runtime surface

Exact pushed source SHA: `eeb49d71572df1746f2924e72d4b515d8fa3327d` (`feat(carrier): expose manager_mut and recovery_engine accessors on runtime`), the final tree after closing the reviewer's second-round R7 findings.

Developer-local clean exact-tree gate. Hosted GitHub checks are separate cross-evidence; this is the developer-local anchor. Not a WAN/live claim, not an independent security/release approval.

## Commands run on a clean worktree of the exact SHA

```bash
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --short
```

## Result

- UTC start `2026-09-13T14:10:20Z`, UTC end `2026-09-13T14:12:18Z`.
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit 0.
- `git diff --check`: exit 0.
- `git status --short` before/after: empty.
- Host `Linux 6.8.0-137-generic x86_64 GNU/Linux`; `rustc 1.98.0 (88d9e12ae 2026-08-18)`.

## Second-round R7 repairs closed in this tree

- **BLOCKER-GATE-001** (`a19d1a4`): `cargo fmt --check` red on `reliable_udp_runtime.rs` — repaired; gate green.
- **H-RUDP-001C** (`0bd4270`): manager-facing health is now the resolved *interval delta* (sent/lost since last consumption), not the lifetime cumulative ratio — a clean outcome never replays an old loss burst.
- **H-RUDP-005** (`4270f82`): `take_ack` consumes a single pending obligation; ACK-only packets never schedule an ACK-of-ACK.
- **H-RUDP-007** (`0bd4270`): stable `FrameId` separated from packet number; `packet_frames` maps real packet→frame; `retransmit_frames` retain/reseal never release; `on_retransmit_sent` re-sends a stable frame under a fresh nonce.
- **H-RUDP-008** (`1e53ffe`): `deliver_logical` keys application delivery by stable `(stream, offset)` — late original vs replacement dedups, conflict fails closed.
- **M-RUDP-009** (`4f98483`): real cwnd admission — refused sends charge/track nothing; `CongestionWindowFull`; pacing deadline exposed.
- **M-RUDP-010** (`c661921`): explicit `WarmFallback(ConcurrentSwitchEvent)` only on successful promotion; `FallbackFailed` on error, no swallowed transitions.

The coherent `ReliableUdpRuntime` + `reliable_udp_runtime` fixture compose all of these in one actual flow (Q6). `READY_LIVE: none` and all release flags unchanged.
