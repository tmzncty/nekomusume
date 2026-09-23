# Independent bounded challenge — R-REC-2 loss/retransmit ownership

**Date:** 2026-09-24  
**Exact developer source anchor:** `6c01d63e2b477b099f154461d8fdba1d8220d0d4` (`main` at challenge start)  
**Owner:** `crates/neko-reliable/src/lib.rs` — `Recovery::on_ack` loss declaration, `release_frame`, `outstanding_frames`/`acked_frames` lifecycle  
**Classification:** independent source/control-flow challenge; no production protocol change

## Challenged invariant

Loss declaration (packet-threshold `largest - n >= 3` or loss-delay `now - sent_at >= delay`) must own retransmit scheduling truthfully: a frame already positively ACKed on another copy is never retransmission-eligible; a frame's last outstanding copy retiring makes it retransmit-eligible; `outstanding_frames` count decrements are copy-lifetime, not retransmit work.

## Source reasoning

`on_ack` loss path (line 375–405):

- `lost` collects only `sent` entries that pass `largest - n >= PACKET_THRESHOLD` or `delay > 0 && now - sent_at >= delay`. Purged/never-sent numbers cannot appear because they are not in `sent`.
- For each lost packet: `sent.remove` + `watermark_on_reserve.remove` + `lost_bytes`/`retransmit_bytes` accumulation + `lost_packets` record.
- For each frame in the lost packet:
  - `self.acked_frames.contains(&f)` → `release_frame` + `acked_frames.remove` + `continue` — the frame was already positively ACKed on another copy; its removal here is copy lifetime only, **not** scheduled retransmit.
  - Otherwise `release_frame` → `frames.insert` — the frame's last outstanding copy retired; it is retransmit-eligible.
- `release_frame` (line 412): `count > 1` → decrement, `false`; `count == 1` → remove, `true`; `None` → `false`. `true` → `frames.insert`.
- `out.retransmit_frames = frames.into_iter().collect()` — only frames whose last outstanding copy retired appear here.
- `if !out.acked_packets.is_empty() { self.pto_count = 0 }` — PTO count resets only on positive ACK, not on loss.

## Result

No concrete defect found. Loss declaration and retransmit ownership are consistent: ACKed-frame suppression prevents duplicate retransmit of a frame already delivered on another copy; `release_frame` correctly distinguishes copy-lifetime decrement from last-copy retirement. R-REC-2 is a no-finding challenge on the exact-current owner.

## Evidence boundary

Developer-source reasoning, not reviewer-local execution, hosted CI, WAN evidence or performance conclusion. No decoder/parser/crypto framing change; fuzz not required. No release flag, D019, or policy value touched. `READY_LIVE: none`.
